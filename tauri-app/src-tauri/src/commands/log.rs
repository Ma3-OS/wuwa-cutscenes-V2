use tauri::{AppHandle, Emitter};

pub fn emit_log(app: &AppHandle, message: &str) {
    let _ = app.emit("backend-log", message);
}
