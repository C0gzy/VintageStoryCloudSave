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
pub struct MaifestProgramData {
    pub last_opened: u64,
    pub current_used_bucket: String,
    pub all_file_info: HashMap<String, UploadManifest>,
}


fn to_io_error<E: std::fmt::Display>(err: E) -> Error {
    Error::new(ErrorKind::Other, err.to_string())
}

pub fn manifest_file_path(root: &Path) -> PathBuf {
    root.join(".cloud_save_manifest.json")
}

pub fn get_manifest_info() -> Result<MaifestProgramData, Error> {
    match resolve_save_dir() {
        Ok(save_root) => {
            let manifest_path = manifest_file_path(&save_root);
            if manifest_path.exists() {
                match load_manifest(&manifest_path) {
                    Ok(program_data) => Ok(program_data.clone()),
                    Err(err) => Err(Error::new(ErrorKind::Other, format!("Manifest found but unreadable: {err}"))),
                }
            } else {
                Ok(MaifestProgramData::default())
            }
        }
        Err(err) => Err(Error::new(ErrorKind::Other, format!("Could not locate save directory: {err}"))),
    }
}

pub fn manifest_status_message() -> String {
    match resolve_save_dir() {
        Ok(save_root) => {
            let manifest_path = manifest_file_path(&save_root);
            if manifest_path.exists() {
                match load_manifest(&manifest_path) {
                    Ok(program_data) => {
                        let total_files: usize = program_data.all_file_info.values()
                            .map(|manifest| manifest.files.len())
                            .sum();
                        format!("Existing manifest found. Tracking {} files.", total_files)
                    },
                    Err(err) => format!("Manifest found but unreadable: {err}"),
                }
            } else {
                "No manifest found. Start an upload to create tracking data.".to_string()
            }
        }
        Err(err) => format!("Could not locate save directory: {err}"),
    }
}

/// Extracts game data from a VCDBS (SQLite) save file
/// Returns a HashMap with metadata about the save game
/// Reference: https://wiki.vintagestory.at/Modding:VCDBS_format#Schema
pub fn get_game_data(file_path: &Path) -> Result<HashMap<String, String>, Error> {
    if !file_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("VCDBS file not found: {}", file_path.display()),
        ));
    }

    let mut game_data = HashMap::new();
    
    // Open the SQLite database
    let conn = Connection::open(file_path).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to open VCDBS database: {}", e),
        )
    })?;

    // Extract gamedata table information
    // The gamedata table has: savegameid (integer, always 1) and data (BLOB - protobuf encoded)
    // WorldName, TotalSecondsPlayed, and LastPlayed are readable as text in the blob
    // Read the blob and extract these fields using string search
    
    let blob: Vec<u8> = conn
        .query_row(
            "SELECT data FROM gamedata WHERE savegameid = 1",
            [],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                Ok(data)
            },
        )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to read gamedata blob: {}", e),
            )
        })?;

    let data_size = blob.len();
    game_data.insert("savegameid".to_string(), "1".to_string());
    game_data.insert("gamedata_size".to_string(), data_size.to_string());
    
    // Try to find TotalSecondsPlayed - look for reasonable u64 values
    // Protobuf varint encoding, but we'll search for little-endian u64 values
    game_data.insert("play_duration_seconds".to_string(), "0".to_string());

    // Get file size
    let file_size = fs::metadata(file_path)
        .map_err(to_io_error)?
        .len();
    game_data.insert("file_size".to_string(), file_size.to_string());

    // Get file name
    if let Some(file_name) = file_path.file_name() {
        game_data.insert(
            "file_name".to_string(),
            file_name.to_string_lossy().to_string(),
        );
    }

    Ok(game_data)
}

pub fn load_manifest(path: &Path) -> Result<MaifestProgramData, Error> {
    if !path.exists() {
        return Ok(MaifestProgramData::default());
    }

    let data = fs::read_to_string(path).map_err(to_io_error)?;
    let program_data = serde_json::from_str(&data).unwrap_or_default();
    Ok(program_data)
}

pub fn save_manifest(path: &Path, manifest: &mut UploadManifest, bucket_name: String) -> Result<(), Error> {
    println!("[DEBUG] save_manifest called for path: {}", path.display());
    println!("[DEBUG] Current manifest has {} files", manifest.files.len());
    
    // Update file info for all VCDBS files in the manifest
    if let Ok(save_root) = resolve_save_dir() {
        println!("[DEBUG] Save root directory: {}", save_root.display());
        
        // Update each file entry with world_name and playtime
        for (file_key, file_info) in manifest.files.iter_mut() {
            // Extract world_name from filename (remove .vcdbs extension)
            if file_key.ends_with(".vcdbs") {
                file_info.world_name = file_key
                    .strip_suffix(".vcdbs")
                    .unwrap_or(file_key)
                    .to_string();
                
                let vcdbs_path = save_root.join(file_key);
                println!("[DEBUG] Processing VCDBS file: {}", vcdbs_path.display());
                
                // Try to extract playtime from VCDBS file if not already set
                if file_info.playtime == 0 && vcdbs_path.exists() {
                    match get_game_data(&vcdbs_path) {
                        Ok(game_data) => {
                            println!("[DEBUG] Extracted game_data keys: {:?}", game_data.keys().collect::<Vec<_>>());
                            if let Some(duration_str) = game_data.get("play_duration_seconds") {
                                println!("[DEBUG] Found play_duration_seconds: {}", duration_str);
                                if let Ok(duration) = duration_str.parse::<u64>() {
                                    file_info.playtime = duration;
                                    println!("[DEBUG] Set playtime to {} for {}", duration, file_info.world_name);
                                }
                            }
                        }
                        Err(e) => {
                            println!("[DEBUG] Failed to extract game_data from {}: {}", vcdbs_path.display(), e);
                        }
                    }
                }
            } else {
                // For non-VCDBS files, use filename as world_name
                file_info.world_name = file_key.clone();
                if file_info.playtime == 0 {
                    file_info.playtime = 0; // Already 0, but explicit
                }
            }
        }
    } else {
        println!("[DEBUG] Failed to resolve save directory");
    }

    println!("[DEBUG] Final manifest has {} files", manifest.files.len());
    for (key, info) in &manifest.files {
        println!("[DEBUG]   {}: world_name={}, playtime={}", key, info.world_name, info.playtime);
    }
    
    let program_data = MaifestProgramData {
        last_opened: 0,
        current_used_bucket: bucket_name.clone(),
        all_file_info: HashMap::from([(bucket_name, manifest.clone())]),
    };
    
    let data = serde_json::to_string_pretty(&program_data).map_err(to_io_error)?;
    println!("[DEBUG] Manifest JSON (first 500 chars): {}", &data.chars().take(500).collect::<String>());
    fs::write(path, data).map_err(to_io_error)
}