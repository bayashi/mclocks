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

fn create_temp_file_with_text(text: String) -> Result<std::path::PathBuf, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let temp_dir = base_dir.cache_dir();

    // Create a temporary text file
    let temp_file = temp_dir.join(format!("mclocks_quote_{}.txt",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()));

    fs::write(&temp_file, text).map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(temp_file)
}

#[tauri::command]
pub fn open_text_in_editor(text: String) -> Result<(), String> {
    let temp_file = create_temp_file_with_text(text)?;
    let temp_file_str = temp_file.to_string_lossy().to_string();
    open_with_system_command(&temp_file_str, "Failed to open file in editor")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_file_with_text() {
        let test_text = "Test content for temporary file";
        let result = create_temp_file_with_text(test_text.to_string());

        // The function should succeed in creating the temp file
        assert!(result.is_ok(), "Should successfully create temp file");

        let temp_file = result.unwrap();

        // Verify the file exists
        assert!(temp_file.exists(), "Temporary file should exist");

        // Verify the file name pattern
        let file_name = temp_file.file_name().unwrap().to_string_lossy();
        assert!(
            file_name.starts_with("mclocks_quote_"),
            "File name should start with 'mclocks_quote_'"
        );
        assert!(
            file_name.ends_with(".txt"),
            "File name should end with '.txt'"
        );

        // Verify the content
        let content = fs::read_to_string(&temp_file)
            .expect("Failed to read temp file");
        assert_eq!(content, test_text, "File content should match input text");

        // Clean up: remove the test file
        let _ = fs::remove_file(&temp_file);
    }
}

