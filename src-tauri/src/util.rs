use std::{fs, process::Command};
use directories::BaseDirs;

pub fn open_with_system_command(path_or_url: &str, error_context: &str) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "", path_or_url])
            .spawn()
            .map_err(|e| format!("{}: {}", error_context, e))?;
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path_or_url)
            .spawn()
            .map_err(|e| format!("{}: {}", error_context, e))?;
    } else {
        Command::new("xdg-open")
            .arg(path_or_url)
            .spawn()
            .map_err(|e| format!("{}: {}", error_context, e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_text_in_editor(text: String) -> Result<(), String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let temp_dir = base_dir.cache_dir();

    // Create a temporary text file
    let temp_file = temp_dir.join(format!("mclocks_quote_{}.txt",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()));

    fs::write(&temp_file, text).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let temp_file_str = temp_file.to_string_lossy().to_string();
    open_with_system_command(&temp_file_str, "Failed to open file in editor")
}

