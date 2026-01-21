mod web_server;
mod config;
mod util;
mod web;
mod window_state;

use std::{sync::Arc, thread, time::{Duration, Instant}};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use web_server::{start_web_server, open_url_in_browser, load_web_config};
use config::{get_config_path, load_config, save_config, load_sticky_note_state, save_sticky_note_state, delete_sticky_note_state, load_all_sticky_note_states, cleanup_window_state, ContextConfig};
use util::open_text_in_editor;
use window_state::{load_window_state, save_window_state, apply_window_state, load_window_state_by_label, save_window_state_by_label, delete_window_state_by_label};
use tauri::State;

const IS_DEV: bool = tauri::is_dev();

const WINDOW_NAME: &str = "main";

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
    let identifier_for_setup = identifier.clone();
    tbr = tbr.setup(move |app| {
        let window = app.get_webview_window(WINDOW_NAME).unwrap();

        // Load and apply saved window state
        if let Ok(state) = load_window_state(&identifier_for_setup) {
            if let Err(e) = apply_window_state(&window, &state) {
                eprintln!("Failed to apply window state: {}", e);
            }
        }

        if IS_DEV {
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
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

        // Set up window state saving on move/resize with proper debouncing
        let window_clone = window.clone();
        let identifier_for_events = identifier_for_setup.clone();
        let last_event_time = Arc::new(std::sync::Mutex::new(Instant::now()));
        let save_pending = Arc::new(std::sync::atomic::AtomicBool::new(false));

        window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                    let window_for_save = window_clone.clone();
                    let identifier_for_save = identifier_for_events.clone();
                    let last_event = last_event_time.clone();
                    let pending = save_pending.clone();

                    // Update last event time
                    *last_event.lock().unwrap() = Instant::now();

                    // Only spawn a new thread if one isn't already pending
                    if !pending.swap(true, std::sync::atomic::Ordering::Acquire) {
                        let window_save = window_for_save.clone();
                        let identifier_save = identifier_for_save.clone();
                        let last_event_save = last_event_time.clone();
                        let pending_save = save_pending.clone();

                        if let Err(e) = thread::Builder::new()
                            .name("window-state-saver".to_string())
                            .spawn(move || {
                                loop {
                                    thread::sleep(Duration::from_millis(500));

                                    let elapsed = last_event_save.lock().unwrap().elapsed();
                                    if elapsed >= Duration::from_millis(500) {
                                        // Save window state
                                        if let Err(e) = save_window_state(&identifier_save, &window_save) {
                                            eprintln!("Failed to save window state: {}", e);
                                        }
                                        pending_save.store(false, std::sync::atomic::Ordering::Release);
                                        break;
                                    }
                                }
                            }) {
                            // If thread spawn fails, reset the flag and log error
                            eprintln!("Failed to spawn window state save thread: {}", e);
                            pending.store(false, std::sync::atomic::Ordering::Release);
                        }
                    }
                }
                _ => {}
            }
        });

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

    tbr
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            get_config_path,
            open_text_in_editor,
            load_sticky_note_state,
            save_sticky_note_state,
            delete_sticky_note_state,
            load_all_sticky_note_states,
            cleanup_window_state,
            load_sticky_note_window_state,
            save_sticky_note_window_state,
            delete_sticky_note_window_state,
        ])
        .run(context)
        .expect("error while running tauri application");
}

#[tauri::command]
fn load_sticky_note_window_state(state: State<'_, Arc<ContextConfig>>, label: String) -> Result<window_state::WindowState, String> {
    load_window_state_by_label(&state.app_identifier, &label)
}

#[tauri::command]
fn save_sticky_note_window_state(app: tauri::AppHandle, state: State<'_, Arc<ContextConfig>>, label: String) -> Result<(), String> {
    let window = app.get_webview_window(&label)
        .ok_or_else(|| format!("Window with label '{}' not found", label))?;
    save_window_state_by_label(&state.app_identifier, &label, &window)
}

#[tauri::command]
fn delete_sticky_note_window_state(state: State<'_, Arc<ContextConfig>>, label: String) -> Result<(), String> {
    delete_window_state_by_label(&state.app_identifier, &label)
}
