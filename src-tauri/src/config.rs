use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, sync::Arc, path::PathBuf};
use tauri::State;

use crate::web_server::WebConfig;

const IS_DEV: bool = tauri::is_dev();

#[derive(Serialize, Deserialize, Debug)]
pub struct Clock {
    #[serde(default = "df_name")]
    pub name: String,
    #[serde(default = "df_timezone")]
    pub timezone: String,
    pub countdown: Option<String>,
    pub target: Option<String>,
}

fn df_name() -> String { "UTC".to_string() }
fn df_timezone() -> String { "UTC".to_string() }

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum InFontSize {
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
    pub clocks: Vec<Clock>,
    #[serde(default = "df_font")]
    pub font: String,
    #[serde(default = "df_size")]
    #[serde(alias = "fontSize")]
    pub size: InFontSize,
    #[serde(default = "df_color")]
    #[serde(alias = "fontColor")]
    pub color: String,
    #[serde(default = "df_format")]
    #[serde(alias = "formatDateTime")]
    pub format: String,
    #[serde(default)]
    pub format2: Option<String>,
    #[serde(default = "df_locale")]
    #[serde(alias = "localeDateTime")]
    pub locale: String,
    #[serde(default)]
    #[serde(alias = "alwaysOnTop")]
    pub forefront: bool,
    #[serde(default = "df_margin")]
    pub margin: String,
    #[serde(default = "df_timer_icon")]
    pub timer_icon: String,
    #[serde(default)]
    pub without_notification: bool,
    #[serde(default = "df_max_timer_clock_number")]
    pub max_timer_clock_number: i32,
    #[serde(default = "df_epoch_clock_name")]
    pub epoch_clock_name: String,
    #[serde(default)]
    pub usetz: bool,
    #[serde(default)]
    pub convtz: String,
    #[serde(default = "df_disable_hover")]
    pub disable_hover: bool,
    #[serde(default)]
    pub web: Option<WebConfig>,
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
pub fn get_config_path(state: State<'_, Arc<ContextConfig>>) -> Result<String, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    // config_path is just a path string if it doesn't exist, and no matter there is the old config file.
    // It's only to open new config file path on frontend.
    Ok(config_path.to_string_lossy().to_string())
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
pub fn load_config(state: State<'_, Arc<ContextConfig>>) -> Result<AppConfig, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    let old_config_path = base_dir.config_dir().join(get_old_config_app_path());
    let config_json = read_config_file(&config_path, &old_config_path)?;
    if !config_path.exists() {
        ensure_config_file_exists(&config_path, &config_json)?;
    }

    serde_json::from_str(&config_json).map_err(|e| vec!["JSON config: ", &e.to_string()].join(""))
}

pub struct ContextConfig {
    pub app_identifier: String,
}

