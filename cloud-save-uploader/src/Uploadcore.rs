use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

pub fn upload_save(folder: String) -> Result<(), Error> {
    let rt = Runtime::new().map_err(to_io_error)?;
    rt.block_on(run_upload(&folder))
}

pub fn manifest_status_message() -> String {
    match resolve_save_dir() {
        Ok(save_root) => {
            let manifest_path = manifest_file_path(&save_root);
            if manifest_path.exists() {
                match load_manifest(&manifest_path) {
                    Ok(manifest) => format!(
                        "Existing manifest found. Tracking {} files.",
                        manifest.files.len()
                    ),
                    Err(err) => format!("Manifest found but unreadable: {err}"),
                }
            } else {
                "No manifest found. Start an upload to create tracking data.".to_string()
            }
        }
        Err(err) => format!("Could not locate save directory: {err}"),
    }
}

fn check_if_upload_is_needed(files: &[PathBuf]) -> Result<bool, Error> {
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

        match manifest.files.get(&key) {
            Some(stored_size) if *stored_size == size => continue,
            _ => return Ok(true),
        }
    }

    Ok(false)
}

async fn run_upload(folder_bucket: &str) -> Result<(), Error> {
    let config = build_b2_client().await?;
    let client = Client::new(&config);

    let bucket = env::var("B2_BUCKET").map_err(to_io_error)?;
    let prefix = env::var("B2_PREFIX").unwrap_or_else(|_| folder_bucket.to_string());

    let save_root = resolve_save_dir()?;
    let files = gather_files(&save_root)?;

    if files.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("no files found under {}", save_root.display()),
        ));
    }

    if !check_if_upload_is_needed(&files)? {
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

        if let Some(stored_size) = manifest.files.get(&file_key) {
            if *stored_size == size {
                continue;
            }
        }

        let body = ByteStream::from_path(&file).await.map_err(to_io_error)?;

        client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to upload {} to {}: {}", file.display(), key, err),
                )
            })?;

        manifest.files.insert(file_key, size);
        manifest_dirty = true;
    }

    if manifest_dirty {
        save_manifest(&manifest_path, &manifest)?;
    }

    Ok(())
}

async fn build_b2_client() -> Result<aws_config::SdkConfig, Error> {
    let key_id = env::var("B2_KEY_ID").map_err(to_io_error)?;
    let application_key = env::var("B2_APPLICATION_KEY").map_err(to_io_error)?;
    let region = env::var("B2_REGION").unwrap_or_else(|_| "us-west-000".to_string());
    let endpoint = env::var("B2_ENDPOINT")
        .unwrap_or_else(|_| format!("https://s3.{}.backblazeb2.com", region));

    let credentials = Credentials::new(key_id, application_key, None, None, "b2");
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region))
        .endpoint_url(endpoint)
        .load()
        .await;

    Ok(shared_config)
}

fn resolve_save_dir() -> Result<PathBuf, Error> {
    if let Ok(overridden) = env::var("VS_SAVE_DIR") {
        return Ok(PathBuf::from(overridden));
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = env::var("APPDATA").map_err(to_io_error)?;
        return Ok(Path::new(&appdata)
            .join("VintagestoryData")
            .join("Saves"));
    }

    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").map_err(to_io_error)?;
        return Ok(Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("VintagestoryData")
            .join("Saves"));
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(Error::new(
            ErrorKind::Unsupported,
            "automatic save directory detection not implemented for this OS; set VS_SAVE_DIR",
        ))
    }
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct UploadManifest {
    files: HashMap<String, u64>,
}

fn manifest_file_path(root: &Path) -> PathBuf {
    root.join(".cloud_save_manifest.json")
}

fn load_manifest(path: &Path) -> Result<UploadManifest, Error> {
    if !path.exists() {
        return Ok(UploadManifest::default());
    }

    let data = fs::read_to_string(path).map_err(to_io_error)?;
    let manifest = serde_json::from_str(&data).unwrap_or_default();
    Ok(manifest)
}

fn save_manifest(path: &Path, manifest: &UploadManifest) -> Result<(), Error> {
    let data = serde_json::to_string_pretty(manifest).map_err(to_io_error)?;
    fs::write(path, data).map_err(to_io_error)
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

    let bucket = env::var("B2_BUCKET").map_err(to_io_error)?;
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
