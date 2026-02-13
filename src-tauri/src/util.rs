use std::process::Command;

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
    // Create a temporary text file using tempfile crate
    let named_file = tempfile::Builder::new()
        .prefix("mclocks_tmp_")
        .suffix(".txt")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    // Keep the file (prevent auto-deletion on drop) and get the path
    let (mut file, path): (std::fs::File, std::path::PathBuf) = named_file.keep()
        .map_err(|e| format!("Failed to persist temp file: {}", e))?;

    std::io::Write::write_all(&mut file, text.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    Ok(path)
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
    use std::fs;

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
            file_name.starts_with("mclocks_tmp_"),
            "File name should start with 'mclocks_tmp_'"
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

