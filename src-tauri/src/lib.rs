mod about_env;
mod chist;
mod config;
mod sticky;
mod tray;
mod util;
mod web;
mod web_server;

use config::{get_config_path, load_config};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use util::open_text_in_editor;
use web::dd_publish::{
    build_temp_file_url, build_temp_share_url, clear_temp_shares, register_temp_file,
    register_temp_root,
};
use web::markdown_live_reload::start_markdown_live_reload_server;
use web_server::{
    WebServerListenKind, default_web_server_config, load_web_config, open_url_in_browser,
    start_web_server,
};

/// Global lock to serialize all saveWindowState calls across windows.
/// Prevents potential deadlocks in the window-state plugin when multiple
/// windows (main + stickies) attempt to save state simultaneously.
struct WindowStateSaveLock(Mutex<()>);

/// Stores resolved web main port selected at startup.
struct WebMainPortStore(Mutex<Option<u16>>);

impl Default for WindowStateSaveLock {
    fn default() -> Self {
        Self(Mutex::new(()))
    }
}

impl Default for WebMainPortStore {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

#[tauri::command]
fn save_window_state_exclusive(
    app: tauri::AppHandle,
    lock: tauri::State<'_, WindowStateSaveLock>,
) -> Result<(), String> {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};
    let _guard = lock.0.lock().map_err(|e| e.to_string())?;
    app.save_window_state(StateFlags::all())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn register_temp_web_root(
    dropped_path: String,
    web_main_port: tauri::State<'_, WebMainPortStore>,
) -> Result<String, String> {
    let port = {
        let guard = web_main_port.0.lock().map_err(|e| e.to_string())?;
        guard.ok_or("Web server is not available.".to_string())?
    };

    let dropped = std::path::Path::new(&dropped_path);
    let url = if dropped.is_dir() {
        let hash = register_temp_root(dropped)?;
        build_temp_share_url(port, &hash)
    } else if dropped.is_file() {
        let hash = register_temp_file(dropped)?;
        build_temp_file_url(port, &hash, dropped)?
    } else {
        return Err(format!("Invalid drop target: {}", dropped.display()));
    };
    open_url_in_browser(&url)?;
    Ok(url)
}

const IS_DEV: bool = tauri::is_dev();

const WINDOW_NAME: &str = "main";

fn clamp_chist_max_entries(n: i32) -> usize {
    (n.max(1).min(1000)) as usize
}

fn clamp_chist_window_px(n: i32) -> f64 {
    (n.max(200).min(2000)) as f64
}

fn reset_temp_web_session_impl() -> Result<String, String> {
    let cleared = clear_temp_shares()?;
    Ok(format!(
        "Web D&D session has been reset ({} roots, {} files).",
        cleared.cleared_roots, cleared.cleared_files
    ))
}

#[tauri::command]
fn reset_temp_web_session() -> Result<String, String> {
    reset_temp_web_session_impl()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tbr = tauri::Builder::default();

    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let identifier: String = context.config().identifier.clone();
    let app_config = config::load_app_config_for_identifier(&identifier).unwrap_or_else(|e| {
        eprintln!("[mclocks] config load failed (using defaults): {}", e);
        serde_json::from_str("{}").expect("default AppConfig")
    });
    let clipboard_disabled = app_config.clipboard.disabled;
    let chist_max_entries = clamp_chist_max_entries(app_config.clipboard.max_clip_number);
    let chist_panel_w = clamp_chist_window_px(app_config.clipboard.window_width);
    let chist_panel_h = clamp_chist_window_px(app_config.clipboard.window_height);
    let clipboard_history_enabled = !clipboard_disabled;
    let context_config_clone = Arc::new(config::ContextConfig {
        app_identifier: identifier.clone(),
    });
    tbr = tbr.manage(context_config_clone);
    tbr = tbr.manage(sticky::StickyInitStore::default());
    tbr = tbr.manage(sticky::StickyPersistStore::new(&identifier));
    tbr = tbr.manage(WindowStateSaveLock::default());
    tbr = tbr.manage(WebMainPortStore::default());

    let mut web_error: Option<String> = None;
    let web_config_for_startup = match load_web_config(&identifier) {
        Ok(Some(config)) => Some(config),
        Ok(None) => match default_web_server_config(&identifier) {
            Ok(config) => Some(config),
            Err(e) => {
                web_error = Some(e);
                None
            }
        },
        Err(e) => {
            web_error = Some(e);
            None
        }
    };

    let port_to_open = web_config_for_startup
        .as_ref()
        .map(|config| {
            let mut markdown_live_reload_ws_port = config.markdown_live_reload_ws_port;
            if let Some(ws_port) = markdown_live_reload_ws_port {
                if !start_markdown_live_reload_server(ws_port) {
                    markdown_live_reload_ws_port = None;
                }
            }
            start_web_server(
                config.root.clone(),
                config.port,
                config.dump,
                config.slow,
                config.status,
                config.allow_html_in_md,
                config.markdown_open_external_link_in_new_tab,
                config.markdown_highlight.clone(),
                config.editor_repos_dir.clone(),
                config.editor_include_host,
                config.editor_command.clone(),
                config.editor_args.clone(),
                markdown_live_reload_ws_port,
                Some(config.local_preview_api_enabled.clone()),
                WebServerListenKind::Main,
            );
            if let Some(assets_server) = &config.assets_server {
                start_web_server(
                    assets_server.root.clone(),
                    assets_server.port,
                    false,
                    false,
                    false,
                    false,
                    true,
                    None,
                    None,
                    false,
                    "code".to_string(),
                    vec!["-g".to_string(), "{file}:{line}".to_string()],
                    None,
                    None,
                    WebServerListenKind::Assets,
                );
            }
            if config.open_browser_at_start {
                Some(config.port)
            } else {
                None
            }
        })
        .flatten();

    let error_msg = web_error.clone();
    let web_main_port_at_startup = web_config_for_startup.as_ref().map(|c| c.port);
    let clipboard_disabled_setup = clipboard_disabled;
    let chist_max_entries_setup = chist_max_entries;
    let chist_panel_w_setup = chist_panel_w;
    let chist_panel_h_setup = chist_panel_h;
    let clipboard_history_enabled_setup = clipboard_history_enabled;
    tbr = tbr.setup(move |app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        for window_config in app.config().app.windows.iter().filter(|w| !w.create) {
            tauri::WebviewWindowBuilder::from_config(app.handle(), window_config)?.build()?;
        }

        let chist_store = Arc::new(chist::ChistStore::new(
            chist_max_entries_setup,
            clipboard_disabled_setup,
            chist_panel_w_setup,
            chist_panel_h_setup,
        ));
        if !clipboard_disabled_setup {
            chist::spawn_chist_watcher(app.handle().clone(), chist_store.clone());
        }
        app.manage(chist_store);

        #[cfg(desktop)]
        {
            tray::setup_tray_menu(
                app.handle(),
                WINDOW_NAME,
                reset_temp_web_session_impl,
                clipboard_history_enabled_setup,
            )?;
        }

        let store = app.state::<WebMainPortStore>();
        if let Ok(mut guard) = store.0.lock() {
            *guard = web_main_port_at_startup;
        }
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
            if let Some(window) = _app.get_webview_window(WINDOW_NAME) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
    }

    let mut ws = tauri_plugin_window_state::Builder::new()
        .with_denylist(&[chist::WINDOW_LABEL]);
    if IS_DEV {
        let filename = format!("{}{}", ".dev", tauri_plugin_window_state::DEFAULT_FILENAME);
        ws = tauri_plugin_window_state::Builder::with_filename(ws, filename);
    }

    tbr = tbr
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(ws.build());

    tbr.invoke_handler(tauri::generate_handler![
        load_config,
        get_config_path,
        open_text_in_editor,
        register_temp_web_root,
        reset_temp_web_session,
        save_window_state_exclusive,
        sticky::create_sticky,
        sticky::create_sticky_image,
        sticky::sticky_take_init_content,
        sticky::save_sticky_text,
        sticky::delete_sticky_text,
        sticky::save_sticky_state,
        sticky::load_sticky_state,
        sticky::load_sticky_image,
        sticky::restore_stickies,
        chist::chist_list,
        chist::chist_apply,
        chist::chist_close_panel,
    ])
    .run(context)
    .expect("error while running tauri application");
}
