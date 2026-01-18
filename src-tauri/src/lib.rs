mod web_server;
mod config;
mod util;
mod web;

use std::{sync::Arc, thread, fs};
use tauri::{Manager, AppHandle, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use web_server::{start_web_server, open_url_in_browser, load_web_config};
use config::{get_config_path, load_config, save_sticky_notes, load_sticky_notes};
use util::open_text_in_editor;

const IS_DEV: bool = tauri::is_dev();

const WINDOW_NAME: &str = "main";

#[tauri::command]
fn close_sticky_note_window(app: AppHandle, state: State<'_, Arc<config::ContextConfig>>, window_label: String) -> Result<(), String> {
    // Log that command was called
    let _ = debug_log(format!("close_sticky_note_window called for: {}", window_label));
    
    // Remove sticky note from saved data before closing window
    let all_notes = load_sticky_notes(state.clone())?;
    let _ = debug_log(format!("close_sticky_note_window: loaded {} notes", all_notes.len()));
    
    let filtered_notes: Vec<_> = all_notes.into_iter().filter(|n| n.id != window_label).collect();
    let filtered_count = filtered_notes.len();
    let _ = debug_log(format!("close_sticky_note_window: filtering to {} notes (removing {})", filtered_count, window_label));
    
    save_sticky_notes(state.clone(), filtered_notes)?;
    let _ = debug_log(format!("close_sticky_note_window: saved {} notes", filtered_count));

    // Close the window after updating JSON
    // The onCloseRequested handler prevents default close, so we need to close it here
    if let Some(window) = app.get_webview_window(&window_label) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        let _ = debug_log(format!("close_sticky_note_window: window closed for {}", window_label));
    } else {
        let _ = debug_log(format!("close_sticky_note_window: window not found for {}", window_label));
    }
    
    Ok(())
}

#[tauri::command]
fn debug_log(message: String) -> Result<(), String> {
    let log_dir = directories::BaseDirs::new()
        .ok_or("Failed to get base dir")?
        .cache_dir()
        .join("mclocks");
    fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log dir: {}", e))?;
    
    let log_file = log_dir.join("debug.log");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let log_line = format!("[{:.3}] {}\n", timestamp, message);
    
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(log_line.as_bytes())
        })
        .map_err(|e| format!("Failed to write log: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn update_sticky_note_position(state: State<'_, Arc<config::ContextConfig>>, window_label: String, x: f64, y: f64) -> Result<(), String> {
    let all_notes = load_sticky_notes(state.clone())?;
    let mut updated_notes = all_notes;

    if let Some(note) = updated_notes.iter_mut().find(|n| n.id == window_label) {
        note.x = Some(x);
        note.y = Some(y);
        save_sticky_notes(state, updated_notes)?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tbr = tauri::Builder::default();

    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let identifier: String = context.config().identifier.clone();
    let context_config_clone = Arc::new(config::ContextConfig {
        app_identifier: identifier.clone(),
    });
    tbr = tbr.manage(context_config_clone);

    let (web_error, web_config_for_startup) = match load_web_config(&identifier) {
        Ok(Some(config)) => (None, Some(config)),
        Ok(None) => (None, None),
        Err(e) => (Some(e), None),
    };

    let port_to_open = web_config_for_startup.as_ref().map(|config| {
        start_web_server(config.root.clone(), config.port, config.dump, config.slow, config.status);
        if config.open_browser_at_start {
            Some(config.port)
        } else {
            None
        }
    }).flatten();

    let error_msg = web_error.clone();
    tbr = tbr.setup(move |app| {
        if IS_DEV {
            let _window = app.get_webview_window(WINDOW_NAME).unwrap();
            #[cfg(debug_assertions)]
            {
                _window.open_devtools();
            }
        }

        if let Some(err) = error_msg {
            app.dialog()
                .message(&err)
                .kind(MessageDialogKind::Error)
                .title("Web Server Error")
                .blocking_show();
        }

        if let Some(port) = port_to_open {
            thread::sleep(std::time::Duration::from_millis(1000)); // Wait a bit for the server to start
            let url = format!("http://localhost:{}", port);
            if let Err(e) = open_url_in_browser(&url) {
                app.dialog()
                    .message(&format!("Failed to open browser: {}", e))
                    .kind(MessageDialogKind::Error)
                    .title("Web Server Error")
                    .blocking_show();
            }
        }
        Ok(())
    });

    if !IS_DEV {
        tbr = tbr.plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            let _ = _app
                .get_webview_window(WINDOW_NAME)
                .expect(&format!("execute only {} window", WINDOW_NAME))
                .set_focus();
        }))
    }

    let mut ws = tauri_plugin_window_state::Builder::new();
    if IS_DEV {
        let filename = format!("{}{}", ".dev", tauri_plugin_window_state::DEFAULT_FILENAME);
        ws = tauri_plugin_window_state::Builder::with_filename(ws, filename);
    }

    tbr.plugin(ws.build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_config,
            get_config_path,
            open_text_in_editor,
            util::create_sticky_note_html_file,
            close_sticky_note_window,
            save_sticky_notes,
            load_sticky_notes,
            update_sticky_note_position,
            debug_log,
        ])
        .run(context)
        .expect("error while running tauri application");
}
