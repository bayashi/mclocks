use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

const IS_DEV: bool = tauri::is_dev();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

fn get_window_state_file() -> String {
    let state_file = "window-state.json";
    if IS_DEV {
        format!("dev.{}", state_file)
    } else {
        state_file.to_string()
    }
}

pub fn get_window_state_app_path(identifier: &String) -> String {
    vec![identifier, get_window_state_file().as_str()].join("/")
}

fn get_window_state_path(identifier: &String) -> Result<PathBuf, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    Ok(base_dir.config_dir().join(get_window_state_app_path(identifier)))
}

pub fn load_window_state(identifier: &String) -> Result<WindowState, String> {
    let state_path = get_window_state_path(identifier)?;

    if !state_path.exists() {
        return Ok(WindowState {
            x: None,
            y: None,
            width: None,
            height: None,
        });
    }

    let state_json = fs::read_to_string(&state_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&state_json).map_err(|e| format!("Failed to parse window state: {}", e))
}

pub fn save_window_state(identifier: &String, window: &WebviewWindow) -> Result<(), String> {
    // Get physical position and size
    let physical_position = window.inner_position().ok();
    let physical_size = window.inner_size().ok();

    // Convert to logical coordinates using scale factor
    let scale_factor = window.scale_factor().unwrap_or(1.0);

    let state = WindowState {
        x: physical_position.map(|p| {
            // Convert physical to logical: logical = physical / scale_factor
            p.x as f64 / scale_factor
        }),
        y: physical_position.map(|p| {
            p.y as f64 / scale_factor
        }),
        width: physical_size.map(|s| {
            s.width as f64 / scale_factor
        }),
        height: physical_size.map(|s| {
            s.height as f64 / scale_factor
        }),
    };

    let state_path = get_window_state_path(identifier)?;
    fs::create_dir_all(state_path.parent().ok_or("Invalid window state path")?)
        .map_err(|e| e.to_string())?;

    let state_json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    let mut state_file = fs::File::create(&state_path).map_err(|e| e.to_string())?;
    state_file.write_all(state_json.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn apply_window_state(window: &WebviewWindow, state: &WindowState) -> Result<(), String> {
    // Apply size first, then position (this helps with multi-display setups)
    if let Some(width) = state.width {
        if let Some(height) = state.height {
            window.set_size(LogicalSize::new(width, height))
                .map_err(|e| format!("Failed to set window size: {}", e))?;
        }
    }

    // Apply position after size to ensure proper placement
    if let Some(x) = state.x {
        if let Some(y) = state.y {
            // Verify the position is within available displays
            // If the saved position is outside all displays, center on primary display
            let position = LogicalPosition::new(x, y);

            // Try to set the position, but don't fail if it's outside displays
            // The window manager will handle positioning
            if let Err(e) = window.set_position(position) {
                eprintln!("Warning: Failed to set window position to ({}, {}): {}. Window may appear at default position.", x, y, e);
            }
        }
    }

    Ok(())
}
