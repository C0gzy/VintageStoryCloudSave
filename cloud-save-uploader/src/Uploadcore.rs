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

pub fn upload_save() -> Result<(), Error> {
    let rt = Runtime::new().map_err(to_io_error)?;
    rt.block_on(run_upload())
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

async fn run_upload() -> Result<(), Error> {
    let config = build_b2_client().await?;
    let client = Client::new(&config);

    let bucket = env::var("B2_BUCKET").map_err(to_io_error)?;
    let prefix = env::var("B2_PREFIX").unwrap_or_else(|_| "vintagestory".to_string());

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


pub fn download_save() -> Result<(), Error> {
    println!("Downloading save...");
    Ok(())
}

pub fn delete_save() -> Result<(), Error> {
    println!("Deleting save...");
    Ok(())
}

fn list_saves() -> Result<(), Error> {
    println!("Listing saves...");
    Ok(())
}