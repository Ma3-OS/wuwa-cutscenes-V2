#![allow(warnings)]
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod config;
pub mod commands;
pub mod pak;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            config::init_config(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            config::get_config,
            config::update_config,
            commands::downloader::download_tools,
            commands::downloader::download_data,
            commands::wwiser::run_wwiser,
            commands::status::check_dependencies,
            commands::status::open_folder,
            commands::status::open_output_dir,
            commands::generator::generate_single_video_info,
            commands::generator::get_video_audio_event,
            commands::renderer::render_video,
            commands::pak_reader::get_pak_files,
            commands::pak_reader::process_cutscene,
            commands::pak_reader::test_pak_mount,
            commands::pak_reader::extract_audio_banks,
            commands::keys::fetch_latest_keys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
