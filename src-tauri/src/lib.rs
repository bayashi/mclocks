use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

const IS_DEV: bool = tauri::is_dev();

#[derive(Serialize, Deserialize, Debug)]
struct Clock {
    #[serde(default)]
    name: String,
    timezone: String,
    #[serde(default)]
    countdown: String,
    #[serde(default)]
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default)]
    clocks: Vec<Clock>,
    #[serde(default)]
    font: String,
    #[serde(default)]
    size: i32,
    #[serde(default)]
    color: String,
    #[serde(default)]
    format: String,
    #[serde(default)]
    locale: String,
    #[serde(default)]
    forefront: bool,
    #[serde(default)]
    margin: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct OldConfig {
    #[serde(default)]
    clocks: Vec<Clock>,
    #[serde(default)]
    font_size: i32,
    #[serde(default)]
    font_color: String,
    #[serde(default)]
    format_date_time: String,
    #[serde(default)]
    locale_date_time: String,
    #[serde(default)]
    always_on_top: bool,
}

const CONFIG_DIR: &str = if IS_DEV { "mclocks.dev" } else { "mclocks" };
const CONFIG_FILE: &str = "config.json";

fn get_config_app_path() -> String {
    vec![CONFIG_DIR, CONFIG_FILE].join("/")
}

fn merge_configs(old: OldConfig, new: Config) -> Config {
    Config {
        clocks: if new.clocks.len() > 0 {
            new.clocks
        } else if old.clocks.len() > 0 {
            old.clocks
        } else {
            vec![Clock {
                name: String::from("UTC"),
                timezone: String::from("UTC"),
                countdown: String::from(""),
                target: String::from(""),
            }]
        },
        font: if new.font != "" {
            new.font
        } else {
            "Courier, monospace".to_string()
        },
        size: if new.size != 0 {
            new.size
        } else if old.font_size != 0 {
            old.font_size
        } else {
            14
        },
        color: if new.color != "" {
            new.color
        } else if old.font_color != "" {
            old.font_color
        } else {
            "#fff".to_string()
        },
        format: if new.format != "" {
            new.format
        } else if old.format_date_time != "" {
            old.format_date_time
        } else {
            "MM-DD ddd HH:mm".to_string()
        },
        locale: if new.locale != "" {
            new.locale
        } else if old.locale_date_time != "" {
            old.locale_date_time
        } else {
            "en".to_string()
        },
        forefront: new.forefront | old.always_on_top,
        margin: if new.margin != "" {
            new.margin
        } else {
            "1.65em".to_string()
        },
    }
}

#[tauri::command]
fn load_config() -> Result<Config, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path());

    if !config_path.exists() {
        return Err(format!("Config file `{}` not found", config_path.display()));
    }

    let json = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let old_config: OldConfig = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let new_config: Config = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    Ok(merge_configs(old_config, new_config))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tbr = tauri::Builder::default();
    if IS_DEV {
        tbr = tbr.setup(|app| {
            let _window = app.get_webview_window("main").unwrap();
            #[cfg(debug_assertions)]
            {
                _window.open_devtools();
            }
            Ok(())
        })
    } else {
        tbr = tbr.plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            let _ = _app
                .get_webview_window("main")
                .expect("execute only main window")
                .set_focus();
        }))
    }

    tbr.plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .invoke_handler(tauri::generate_handler![load_config,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
