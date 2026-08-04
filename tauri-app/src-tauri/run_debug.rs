use tauri_app_lib::pak::PakManager;
use tauri_app_lib::commands::keys::KeyManager;
use std::path::PathBuf;

fn main() {
    let paks_dir = PathBuf::from("D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks");
    // Since we need an AppHandle to create PakManager/KeyManager in the real app,
    // let's just bypass it and call parse_pak directly!
}
