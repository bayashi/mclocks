mod web_server;
mod config;
mod util;
mod web;
mod sticky;

use std::{sync::Arc, thread};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use web_server::{start_web_server, open_url_in_browser, load_web_config};
use config::{get_config_path, load_config};
use util::open_text_in_editor;

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
    tbr = tbr.manage(sticky::StickyInitStore::default());

    let (web_error, web_config_for_startup) = match load_web_config(&identifier) {
        Ok(Some(config)) => (None, Some(config)),
        Ok(None) => (None, None),
        Err(e) => (Some(e), None),
    };

    let port_to_open = web_config_for_startup.as_ref().map(|config| {
        start_web_server(config.root.clone(), config.port, config.dump, config.slow, config.status, config.editor_repos_dir.clone(), config.editor_include_host, config.editor_command.clone(), config.editor_args.clone());
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

    tbr = tbr.plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(ws.build());

    tbr.invoke_handler(tauri::generate_handler![
            load_config,
            get_config_path,
            open_text_in_editor,
            sticky::create_sticky,
            sticky::sticky_take_init_text,
        ])
        .run(context)
        .expect("error while running tauri application");
}
