export interface FileInfo {
    world_name: string,
    playtime: number, // in seconds
    file_size: number | null, // file size in bytes for change detection
}

export interface UploadManifest {
    files: Record<string, FileInfo>,
}

export interface VintageProgramData {
    last_opened: number,
    current_used_bucket: string,
    all_file_info: Record<string, UploadManifest>,
}