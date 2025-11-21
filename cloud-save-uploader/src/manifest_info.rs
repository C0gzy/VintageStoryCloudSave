use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use crate::helper_functions::resolve_save_dir;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub world_name: String,
    pub playtime: u64, // in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>, // file size in bytes for change detection
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UploadManifest {
    pub files: HashMap<String, FileInfo>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VintageProgramData {
    pub last_opened: u64,
    pub current_used_bucket: String,
    pub all_file_info: HashMap<String, UploadManifest>,
}

pub fn manifest_file_path(root: &Path) -> PathBuf {
    root.join(".cloud_save_manifest.json")
}

pub fn update_vintage_program_data(bucket_name: String) -> Result<bool, Error> {
    println!("Updating vintage program data for bucket: {}", bucket_name);

    let mut current_manifest = get_manifest_info()?;

    current_manifest.current_used_bucket = bucket_name.clone();
    current_manifest.last_opened = 0;
    let save_root = resolve_save_dir()?;

    for file in fs::read_dir(save_root)? {
        let file = file?;
        let file_path = file.path();
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        if file_name.ends_with(".vcdbs") {
            let mut file_info = FileInfo::default();
            file_info.world_name = file_name.clone();
            file_info.playtime = 0;
            file_info.file_size = Some(file_path.metadata()?.len());
            current_manifest.all_file_info
                .entry(bucket_name.clone())
                .or_default()
                .files
                .insert(file_name.clone(), file_info);
        }
    }


    let success = save_vintage_program_data(&current_manifest);
    return success;
}

pub fn save_vintage_program_data(program_data: &VintageProgramData) -> Result<bool, Error> {

    let save_root = resolve_save_dir()?;
    let manifest_path = manifest_file_path(&save_root);
    let data = serde_json::to_string_pretty(&program_data).map_err(|e| Error::new(ErrorKind::Other, format!("Failed to save program data: {}", e)))?;
    fs::write(manifest_path, data).map_err(|e| Error::new(ErrorKind::Other, format!("Failed to save program data: {}", e)))?;

    return Ok(true);
}

pub fn manifest_status_message() -> Result<String, Error> {
    let manifest_info = get_manifest_info()?;
    let total_files: usize = manifest_info.all_file_info.values()
        .map(|manifest| manifest.files.len())
        .sum();
    return Ok(format!("Existing manifest found. Tracking {} files.", total_files));
}

pub fn get_manifest_info() -> Result<VintageProgramData, Error> {

    let save_root = resolve_save_dir()?;
    let manifest_path = manifest_file_path(&save_root);
    if manifest_path.exists() {
        let data = fs::read_to_string(manifest_path).map_err(|e| Error::new(ErrorKind::Other, format!("Failed to read program data: {}", e)))?;
        let program_data = serde_json::from_str(&data).map_err(|e| Error::new(ErrorKind::Other, format!("Failed to parse program data: {}", e)))?;
        return Ok(program_data);
    }
    else {
        return Ok(VintageProgramData::default());
    }

}
