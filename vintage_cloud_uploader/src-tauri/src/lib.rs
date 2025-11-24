mod helper_functions;
mod manifest_info;
mod upload_core;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file if it exists (ignores errors if file doesn't exist)
    let _ = dotenv::dotenv();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, manifest_info::get_manifest_info, upload_core::run_upload, upload_core::run_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
