mod web_server;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, sync::Arc, process::Command, path::PathBuf, thread};
use tauri::{Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use web_server::{WebConfig, start_web_server, open_url_in_browser, load_web_config};

const IS_DEV: bool = tauri::is_dev();

const WINDOW_NAME: &str = "main";

#[derive(Serialize, Deserialize, Debug)]
struct Clock {
    #[serde(default = "df_name")]
    name: String,
    #[serde(default = "df_timezone")]
    timezone: String,
    countdown: Option<String>,
    target: Option<String>,
}

fn df_name() -> String { "UTC".to_string() }
fn df_timezone() -> String { "UTC".to_string() }

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum InFontSize {
    Int(i32),
    Str(String),
}

fn df_clocks() -> Vec<Clock> {
    let mut cls: Vec<Clock> = Vec::new();
    cls.push(Clock {name: df_name(), timezone: df_timezone(), countdown: None, target: None});

    cls
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "df_clocks")]
    clocks: Vec<Clock>,
    #[serde(default = "df_font")]
    font: String,
    #[serde(default = "df_size")]
    #[serde(alias = "fontSize")]
    size: InFontSize,
    #[serde(default = "df_color")]
    #[serde(alias = "fontColor")]
    color: String,
    #[serde(default = "df_format")]
    #[serde(alias = "formatDateTime")]
    format: String,
    #[serde(default)]
    format2: Option<String>,
    #[serde(default = "df_locale")]
    #[serde(alias = "localeDateTime")]
    locale: String,
    #[serde(default)]
    #[serde(alias = "alwaysOnTop")]
    forefront: bool,
    #[serde(default = "df_margin")]
    margin: String,
    #[serde(default = "df_timer_icon")]
    timer_icon: String,
    #[serde(default)]
    without_notification: bool,
    #[serde(default = "df_max_timer_clock_number")]
    max_timer_clock_number: i32,
    #[serde(default = "df_epoch_clock_name")]
    epoch_clock_name: String,
    #[serde(default)]
    usetz: bool,
    #[serde(default)]
    convtz: String,
    #[serde(default = "df_disable_hover")]
    disable_hover: bool,
    #[serde(default)]
    web: Option<WebConfig>,
}

fn df_font() -> String { "Courier, monospace".to_string() }
fn df_size() -> InFontSize { InFontSize::Int(14) }
fn df_color() -> String { "#fff".to_string() }
fn df_format() -> String { "MM-DD ddd HH:mm".to_string() }
fn df_locale() -> String { "en".to_string() }
fn df_margin() -> String { "1.65em".to_string() }
fn df_timer_icon() -> String { "⧖ ".to_string() }
fn df_max_timer_clock_number() -> i32 { 5 }
fn df_epoch_clock_name() -> String { "Epoch".to_string() }
fn df_disable_hover() -> bool { true }

fn get_config_file() -> String {
    let config_file = "config.json";
    if IS_DEV {
        format!("dev.{}", config_file)
    } else {
        config_file.to_string()
    }
}

const OLD_CONFIG_DIR: &str = if IS_DEV { "mclocks.dev" } else { "mclocks" };

fn get_old_config_app_path() -> String {
    vec![OLD_CONFIG_DIR, &get_config_file()].join("/")
}

pub fn get_config_app_path(identifier: &String) -> String {
    vec![identifier, get_config_file().as_str()].join("/")
}

#[tauri::command]
fn get_config_path(state: State<'_, Arc<ContextConfig>>) -> Result<String, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    // config_path is just a path string if it doesn't exist, and no matter there is the old config file.
    // It's only to open new config file path on frontend.
    Ok(config_path.to_string_lossy().to_string())
}

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
fn open_text_in_editor(text: String) -> Result<(), String> {
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

fn read_config_file(config_path: &PathBuf, old_config_path: &PathBuf) -> Result<String, String> {
    if config_path.exists() {
        fs::read_to_string(config_path).map_err(|e| e.to_string())
    } else if old_config_path.exists() {
        fs::read_to_string(old_config_path).map_err(|e| e.to_string())
    } else {
        Ok("{\n  \n}\n".to_string())
    }
}

fn ensure_config_file_exists(config_path: &PathBuf, config_json: &str) -> Result<(), String> {
    fs::create_dir_all(config_path.parent().ok_or("Invalid config path")?)
        .map_err(|e| e.to_string())?;
    let mut config_file = fs::File::create(config_path).map_err(|e| e.to_string())?;
    config_file.write_all(config_json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_config(state: State<'_, Arc<ContextConfig>>) -> Result<AppConfig, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    let old_config_path = base_dir.config_dir().join(get_old_config_app_path());
    let config_json = read_config_file(&config_path, &old_config_path)?;
    if !config_path.exists() {
        ensure_config_file_exists(&config_path, &config_json)?;
    }

    serde_json::from_str(&config_json).map_err(|e| vec!["JSON config: ", &e.to_string()].join(""))
}

struct ContextConfig {
    app_identifier: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tbr = tauri::Builder::default();

    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let identifier: String = context.config().identifier.clone();
    let context_config_clone = Arc::new(ContextConfig {
        app_identifier: identifier.clone(),
    });
    tbr = tbr.manage(context_config_clone);

    let (web_error, web_config_for_startup) = match load_web_config(&identifier) {
        Ok(Some(config)) => (None, Some(config)),
        Ok(None) => (None, None),
        Err(e) => (Some(e), None),
    };

    let port_to_open = web_config_for_startup.as_ref().map(|config| {
        start_web_server(config.root.clone(), config.port, config.dump);
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
        ])
        .run(context)
        .expect("error while running tauri application");
}
