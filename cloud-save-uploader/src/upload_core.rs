use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Instant;
use crate::manifest_info::UploadManifest;
use tokio::runtime::Runtime;

use crate::helper_functions::resolve_save_dir;
use crate::manifest_info::{load_manifest, manifest_file_path, save_manifest};

#[derive(Debug, Clone)]
pub struct UploadProgress {
    pub uploaded_bytes: u64,
    pub total_bytes: u64,
    pub current_file: String,
    pub elapsed_secs: f32,
}

pub fn upload_save(folder: String, progress_tx: Option<Sender<UploadProgress>>) -> Result<(), Error> {
    let rt = Runtime::new().map_err(to_io_error)?;
    rt.block_on(run_upload(&folder, progress_tx))
}



fn check_if_upload_is_needed(files: &[PathBuf], folder_bucket: String) -> Result<bool, Error> {
    if files.is_empty() {
        return Ok(false);
    }

    let save_root = resolve_save_dir()?;
    let manifest_path = manifest_file_path(&save_root);
    let manifest = load_manifest(&manifest_path)?;

    for file in files {
        let metadata = fs::metadata(file).map_err(to_io_error)?;
        let size = metadata.len();
        let key = file_key(&save_root, file);
        let folder_manifest = manifest.all_file_info.get(&folder_bucket);
        // Check if file exists in manifest and size matches
        if folder_manifest.is_some() {
            match folder_manifest.unwrap().files.get(&key) {
                Some(file_info) => {
                    if let Some(stored_size) = file_info.file_size {
                        if stored_size != size {
                            return Ok(true); // Size changed, need to upload
                        }
                    } else {
                        // No size stored, assume we need to upload
                        return Ok(true);
                    }
                }
                None => return Ok(true), // New file, need to upload
            }
        } else {
            return Ok(true); // New folder, need to upload
        }
    }

    Ok(false)
}

async fn run_upload(
    folder_bucket: &str,
    progress_tx: Option<Sender<UploadProgress>>,
) -> Result<(), Error> {
    let config = build_b2_client().await?;
    let client = Client::new(&config);

    // Try compile-time env vars first (from build.rs), then fallback to runtime env vars
    let bucket = option_env!("B2_BUCKET")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_BUCKET").ok())
        .ok_or_else(|| Error::new(ErrorKind::Other, "B2_BUCKET not set"))?;
    let prefix = env::var("B2_PREFIX").unwrap_or_else(|_| folder_bucket.to_string());

    let save_root = resolve_save_dir()?;
    let files = gather_files(&save_root)?;

    if files.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("no files found under {}", save_root.display()),
        ));
    }

    if !check_if_upload_is_needed(&files, folder_bucket.to_string())? {
        return Ok(());
    }

    let manifest_path = manifest_file_path(&save_root);
    if manifest_path.exists() {
        println!("Existing manifest found. Current progress gathered.");
    } else {
        println!("No manifest found. Starting fresh upload progress tracking.");
    }
    let mut manifest = load_manifest(&manifest_path)?;
    let mut manifest_dirty = false;

    struct PendingUpload {
        path: PathBuf,
        s3_key: String,
        manifest_key: String,
        size: u64,
    }

    let mut pending_uploads: Vec<PendingUpload> = Vec::new();

    for file in files {
        let relative = file
            .strip_prefix(&save_root)
            .map_err(to_io_error)?
            .to_string_lossy()
            .replace('\\', "/");
        let key = format!("{}/{}", prefix, relative);
        let file_key = file_key(&save_root, &file);
        let metadata = fs::metadata(&file).map_err(to_io_error)?;
        let size = metadata.len();
        let folder_manifest = manifest.all_file_info.get(folder_bucket);
        let mut needs_upload = false;

        if folder_manifest.is_some()  {
            // Check if file needs uploading (compare by file size)
            needs_upload = match folder_manifest.unwrap().files.get(&file_key) {
                Some(file_info) => {
                    // File exists in manifest, check if size changed
                    match file_info.file_size {
                        Some(stored_size) => stored_size != size,
                        None => true, // No size stored, assume changed
                    }
                }
                None => true, // New file
            };
        } else {
            needs_upload = true;
        }

        if needs_upload {
            pending_uploads.push(PendingUpload {
                path: file.clone(),
                s3_key: key,
                manifest_key: file_key,
                size,
            });
        }
    }

    if pending_uploads.is_empty() {
        return Ok(());
    }

    let total_bytes: u64 = pending_uploads.iter().map(|entry| entry.size).sum();
    let mut uploaded_bytes: u64 = 0;
    let start_time = Instant::now();

    if let Some(tx) = &progress_tx {
        let _ = tx.send(UploadProgress {
            uploaded_bytes,
            total_bytes,
            current_file: String::new(),
            elapsed_secs: 0.0,
        });
    }

    for entry in pending_uploads {
        let body = ByteStream::from_path(&entry.path).await.map_err(to_io_error)?;

        client
            .put_object()
            .bucket(&bucket)
            .key(&entry.s3_key)
            .body(body)
            .send()
            .await
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to upload {} to {}: {}", entry.path.display(), entry.s3_key, err),
                )
            })?;

        let world_name = if entry.manifest_key.ends_with(".vcdbs") {
            entry
                .manifest_key
                .strip_suffix(".vcdbs")
                .unwrap_or(&entry.manifest_key)
                .to_string()
        } else {
            entry.manifest_key.clone()
        };

        let mut playtime = 0u64;
        if let Ok(game_data) = crate::manifest_info::get_game_data(&entry.path) {
            if let Some(duration_str) = game_data.get("play_duration_seconds") {
                if let Ok(duration) = duration_str.parse::<u64>() {
                    playtime = duration;
                }
            }
        }

        let file_info = crate::manifest_info::FileInfo {
            world_name,
            playtime,
            file_size: Some(entry.size),
        };

        let folder_manifest = manifest.all_file_info.get_mut(folder_bucket);
        if folder_manifest.is_some() {
            folder_manifest.unwrap().files.insert(entry.manifest_key.clone(), file_info);
        } else {
            manifest.all_file_info.insert(folder_bucket.to_string(), UploadManifest { files: HashMap::from([(entry.manifest_key.clone(), file_info)]) });
        }
        
        manifest_dirty = true;

        uploaded_bytes += entry.size;
        if let Some(tx) = &progress_tx {
            let _ = tx.send(UploadProgress {
                uploaded_bytes,
                total_bytes,
                current_file: entry.manifest_key.clone(),
                elapsed_secs: start_time.elapsed().as_secs_f32(),
            });
        }
    }

    if manifest_dirty {
        save_manifest(&manifest_path, &mut manifest.all_file_info.get_mut(folder_bucket).unwrap(), folder_bucket.to_string())?;
    }

    Ok(())
}

async fn build_b2_client() -> Result<aws_config::SdkConfig, Error> {
    // Try compile-time env vars first (from build.rs), then fallback to runtime env vars
    let key_id = option_env!("B2_KEY_ID")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_KEY_ID").ok())
        .ok_or_else(|| Error::new(ErrorKind::Other, "B2_KEY_ID not set"))?;
    
    let application_key = option_env!("B2_APPLICATION_KEY")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_APPLICATION_KEY").ok())
        .ok_or_else(|| Error::new(ErrorKind::Other, "B2_APPLICATION_KEY not set"))?;
    
    let region = option_env!("B2_REGION")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_REGION").ok())
        .unwrap_or_else(|| "us-west-000".to_string());
    
    let endpoint = option_env!("B2_ENDPOINT")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_ENDPOINT").ok())
        .unwrap_or_else(|| format!("https://s3.{}.backblazeb2.com", region));

    let credentials = Credentials::new(key_id, application_key, None, None, "b2");
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region))
        .endpoint_url(endpoint)
        .load()
        .await;

    Ok(shared_config)
}



fn gather_files(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).map_err(to_io_error)? {
            let entry = entry.map_err(to_io_error)?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

fn to_io_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(ErrorKind::Other, err.to_string())
}




fn file_key(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}


pub fn download_save(folder: String) -> Result<(), Error> {
    let rt = Runtime::new().map_err(to_io_error)?;
    rt.block_on(run_download(&folder))
}

async fn run_download(folder_bucket: &str) -> Result<(), Error> {
    let config = build_b2_client().await?;
    let client = Client::new(&config);

    // Try compile-time env vars first (from build.rs), then fallback to runtime env vars
    let bucket = option_env!("B2_BUCKET")
        .map(|s| s.to_string())
        .or_else(|| env::var("B2_BUCKET").ok())
        .ok_or_else(|| Error::new(ErrorKind::Other, "B2_BUCKET not set"))?;
    let prefix = env::var("B2_PREFIX").unwrap_or_else(|_| folder_bucket.to_string());

    let save_root = resolve_save_dir()?;
    
    // List all objects in the B2 bucket with the prefix
    let remote_files = list_remote_files(&client, &bucket, &prefix).await?;
    
    if remote_files.is_empty() {
        println!("No files found in cloud storage");
        return Ok(());
    }
    
    println!("Found {} files in cloud storage", remote_files.len());
    
    // Determine which files need to be downloaded
    let files_to_download = determine_files_to_download(&save_root, &prefix, &remote_files)?;
    
    if files_to_download.is_empty() {
        println!("All files are up to date. No download needed.");
        return Ok(());
    }
    
    println!("Downloading {} file(s)...", files_to_download.len());
    
    // Download each file
    for (remote_key, remote_size) in files_to_download {
        let local_path = save_root.join(remote_key.strip_prefix(&format!("{}/", prefix)).unwrap_or(&remote_key));
        
        // Create parent directories if needed
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).map_err(to_io_error)?;
        }
        
        println!("Downloading: {} -> {}", remote_key, local_path.display());
        
        let response = client
            .get_object()
            .bucket(&bucket)
            .key(&remote_key)
            .send()
            .await
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to download {}: {}", remote_key, err),
                )
            })?;
        
        let mut body = response.body.collect().await.map_err(|err| {
            Error::new(ErrorKind::Other, format!("failed to read download body: {}", err))
        })?;
        
        fs::write(&local_path, body.to_vec()).map_err(to_io_error)?;
        
        println!("Downloaded: {} ({} bytes)", local_path.display(), remote_size);
    }
    
    println!("Download complete!");
    Ok(())
}

async fn list_remote_files(
    client: &Client,
    bucket: &str,
    prefix: &str,
) -> Result<HashMap<String, u64>, Error> {
    let mut remote_files = HashMap::new();
    let mut continuation_token: Option<String> = None;
    
    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(format!("{}/", prefix));
        
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }
        
        let response = request.send().await.map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("failed to list objects from bucket: {}", err),
            )
        })?;
        
        if let Some(contents) = response.contents.as_ref() {
            for object in contents.iter() {
                if let (Some(key), Some(size)) = (object.key(), object.size()) {
                    remote_files.insert(key.to_string(), size as u64);
                }
            }
        }
        
        continuation_token = response.next_continuation_token().map(|s| s.to_string());
        if continuation_token.is_none() {
            break;
        }
    }
    
    Ok(remote_files)
}

fn determine_files_to_download(
    save_root: &Path,
    prefix: &str,
    remote_files: &HashMap<String, u64>,
) -> Result<HashMap<String, u64>, Error> {
    let mut files_to_download = HashMap::new();
    
    for (remote_key, remote_size) in remote_files {
        // Remove the prefix to get the relative path
        let relative_path = remote_key
            .strip_prefix(&format!("{}/", prefix))
            .unwrap_or(remote_key);
        
        let local_path = save_root.join(relative_path);
        
        // Check if file needs downloading
        let needs_download = if !local_path.exists() {
            true // File doesn't exist locally
        } else {
            // File exists, check if size matches
            match fs::metadata(&local_path) {
                Ok(metadata) => metadata.len() != *remote_size,
                Err(_) => true, // Can't read metadata, download to be safe
            }
        };
        
        if needs_download {
            files_to_download.insert(remote_key.clone(), *remote_size);
        }
    }
    
    Ok(files_to_download)
}

pub fn delete_save() -> Result<(), Error> {
    println!("Deleting save...");
    Ok(())
}

fn list_saves() -> Result<(), Error> {
    println!("Listing saves...");
    Ok(())
}
