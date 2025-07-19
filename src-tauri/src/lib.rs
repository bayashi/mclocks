use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, sync::Arc};
use tauri::{Manager, State};

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
struct AppConfig {
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

fn get_config_app_path(identifier: &String) -> String {
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

#[tauri::command]
fn load_config(state: State<'_, Arc<ContextConfig>>) -> Result<AppConfig, String> {
    let mut config_json = "{\n  \n}\n".to_string();
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    let old_config_path = base_dir.config_dir().join(get_old_config_app_path());
    if config_path.exists() {
        config_json = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    } else {
        if old_config_path.exists() {
            config_json = fs::read_to_string(old_config_path).map_err(|e| e.to_string())?;
        }
        // just create config_path
        fs::create_dir_all(config_path.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut config_file = fs::File::create(config_path).map_err(|e| e.to_string())?;
        config_file.write_all(config_json.as_bytes()).map_err(|e| e.to_string())?;
    }

    Ok(serde_json::from_str(&config_json).map_err(|e| vec!["JSON config: ", &e.to_string()].join(""))?)
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
        app_identifier: identifier,
    });
    tbr = tbr.manage(context_config_clone);

    if IS_DEV {
        tbr = tbr.setup(|app| {
            let _window = app.get_webview_window(WINDOW_NAME).unwrap();
            #[cfg(debug_assertions)]
            {
                _window.open_devtools();
            }
            Ok(())
        })
    } else {
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
        ])
        .run(context)
        .expect("error while running tauri application");
}
