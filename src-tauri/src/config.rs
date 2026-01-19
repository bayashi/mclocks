use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, sync::Arc, path::PathBuf, collections::HashMap};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StickyNoteState {
    pub text: String,
    pub is_expanded: bool,
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

fn get_sticky_notes_file() -> String {
    let sticky_notes_file = "sticky-notes.json";
    if IS_DEV {
        format!("dev.{}", sticky_notes_file)
    } else {
        sticky_notes_file.to_string()
    }
}

const OLD_CONFIG_DIR: &str = if IS_DEV { "mclocks.dev" } else { "mclocks" };

fn get_old_config_app_path() -> String {
    vec![OLD_CONFIG_DIR, &get_config_file()].join("/")
}

pub fn get_config_app_path(identifier: &String) -> String {
    vec![identifier, get_config_file().as_str()].join("/")
}

fn get_sticky_notes_app_path(identifier: &String) -> String {
    vec![identifier, get_sticky_notes_file().as_str()].join("/")
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

#[tauri::command]
pub fn save_config(state: State<'_, Arc<ContextConfig>>, config: AppConfig) -> Result<(), String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(&state.app_identifier));
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| vec!["JSON config: ", &e.to_string()].join(""))?;
    ensure_config_file_exists(&config_path, &config_json)?;
    Ok(())
}

fn read_sticky_notes_file(sticky_notes_path: &PathBuf) -> Result<HashMap<String, StickyNoteState>, String> {
    if sticky_notes_path.exists() {
        let sticky_notes_json = fs::read_to_string(sticky_notes_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&sticky_notes_json).map_err(|e| vec!["JSON sticky notes: ", &e.to_string()].join(""))
    } else {
        Ok(HashMap::new())
    }
}

#[tauri::command]
pub fn load_sticky_note_state(state: State<'_, Arc<ContextConfig>>, label: String) -> Result<Option<StickyNoteState>, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let sticky_notes_path = base_dir.config_dir().join(get_sticky_notes_app_path(&state.app_identifier));
    let sticky_notes = read_sticky_notes_file(&sticky_notes_path)?;
    Ok(sticky_notes.get(&label).cloned())
}

#[tauri::command]
pub fn save_sticky_note_state(state: State<'_, Arc<ContextConfig>>, label: String, sticky_state: StickyNoteState) -> Result<(), String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let sticky_notes_path = base_dir.config_dir().join(get_sticky_notes_app_path(&state.app_identifier));
    let mut sticky_notes = read_sticky_notes_file(&sticky_notes_path)?;
    sticky_notes.insert(label, sticky_state);
    let sticky_notes_json = serde_json::to_string_pretty(&sticky_notes)
        .map_err(|e| vec!["JSON sticky notes: ", &e.to_string()].join(""))?;
    ensure_config_file_exists(&sticky_notes_path, &sticky_notes_json)?;
    Ok(())
}

#[tauri::command]
pub fn load_all_sticky_note_states(state: State<'_, Arc<ContextConfig>>) -> Result<HashMap<String, StickyNoteState>, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let sticky_notes_path = base_dir.config_dir().join(get_sticky_notes_app_path(&state.app_identifier));
    read_sticky_notes_file(&sticky_notes_path)
}

pub struct ContextConfig {
    pub app_identifier: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_config_app_path() {
        let identifier = "test.app".to_string();
        let path = get_config_app_path(&identifier);

        // Should contain identifier and config file name
        assert!(path.contains(&identifier), "Path should contain identifier");
        assert!(path.contains("config.json") || path.contains("dev.config.json"),
                "Path should contain config file name");

        // Should use forward slash as separator
        assert!(path.contains("/"), "Path should use forward slash separator");

        // Should have format: identifier/config_file
        let parts: Vec<&str> = path.split('/').collect();
        assert_eq!(parts.len(), 2, "Path should have exactly 2 parts");
        assert_eq!(parts[0], identifier, "First part should be identifier");
        assert!(parts[1].contains("config.json"), "Second part should contain config.json");
    }

    #[test]
    fn test_read_config_file_new_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");
        let old_config_path = temp_dir.path().join("old_config.json");

        let test_content = r#"{"clocks": [{"name": "Test"}]}"#;
        fs::write(&config_path, test_content).expect("Failed to write config file");

        let result = read_config_file(&config_path, &old_config_path);
        assert!(result.is_ok(), "Should successfully read new config file");
        assert_eq!(result.unwrap(), test_content, "Content should match");
    }

    #[test]
    fn test_read_config_file_old_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");
        let old_config_path = temp_dir.path().join("old_config.json");

        let test_content = r#"{"clocks": [{"name": "Old"}]}"#;
        fs::write(&old_config_path, test_content).expect("Failed to write old config file");

        let result = read_config_file(&config_path, &old_config_path);
        assert!(result.is_ok(), "Should successfully read old config file");
        assert_eq!(result.unwrap(), test_content, "Content should match");
    }

    #[test]
    fn test_read_config_file_neither_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");
        let old_config_path = temp_dir.path().join("old_config.json");

        let result = read_config_file(&config_path, &old_config_path);
        assert!(result.is_ok(), "Should return default empty JSON");
        assert_eq!(result.unwrap(), "{\n  \n}\n", "Should return default empty JSON");
    }

    #[test]
    fn test_ensure_config_file_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");
        let config_json = r#"{"clocks": [{"name": "Test"}]}"#;

        let result = ensure_config_file_exists(&config_path, config_json);
        assert!(result.is_ok(), "Should successfully create config file");

        // Verify file exists and has correct content
        assert!(config_path.exists(), "Config file should exist");
        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        assert_eq!(content, config_json, "File content should match");
    }

    #[test]
    fn test_ensure_config_file_exists_with_subdir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("subdir").join("config.json");
        let config_json = r#"{"font": "Arial"}"#;

        let result = ensure_config_file_exists(&config_path, config_json);
        assert!(result.is_ok(), "Should successfully create config file with subdirectory");

        // Verify file exists and subdirectory was created
        assert!(config_path.exists(), "Config file should exist");
        assert!(config_path.parent().unwrap().exists(), "Subdirectory should exist");

        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        assert_eq!(content, config_json, "File content should match");
    }

    #[test]
    fn test_ensure_config_file_exists_invalid_path() {
        let config_path = std::path::PathBuf::from("/");
        let config_json = r#"{"test": "value"}"#;

        let result = ensure_config_file_exists(&config_path, config_json);
        assert!(result.is_err(), "Should fail with invalid path");
        assert!(result.unwrap_err().contains("Invalid config path"),
                "Error message should indicate invalid path");
    }

    #[test]
    fn test_app_config_deserialize_empty() {
        let json = "{}";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize empty JSON");
        let config = result.unwrap();

        // Check default values
        assert_eq!(config.clocks.len(), 1, "Should have default clock");
        assert_eq!(config.clocks[0].name, "UTC", "Default clock name should be UTC");
        assert_eq!(config.clocks[0].timezone, "UTC", "Default clock timezone should be UTC");
        assert_eq!(config.font, "Courier, monospace", "Default font should be Courier, monospace");
        assert_eq!(config.color, "#fff", "Default color should be #fff");
        assert_eq!(config.format, "MM-DD ddd HH:mm", "Default format should match");
        assert_eq!(config.locale, "en", "Default locale should be en");
        assert_eq!(config.margin, "1.65em", "Default margin should be 1.65em");
        assert_eq!(config.timer_icon, "⧖ ", "Default timer icon should match");
        assert_eq!(config.max_timer_clock_number, 5, "Default max timer clock number should be 5");
        assert_eq!(config.epoch_clock_name, "Epoch", "Default epoch clock name should be Epoch");
        assert_eq!(config.disable_hover, true, "Default disable_hover should be true");
        assert_eq!(config.forefront, false, "Default forefront should be false");
        assert_eq!(config.without_notification, false, "Default without_notification should be false");
        assert_eq!(config.usetz, false, "Default usetz should be false");
        assert_eq!(config.convtz, "", "Default convtz should be empty");
        assert!(config.format2.is_none(), "Default format2 should be None");
        assert!(config.web.is_none(), "Default web should be None");
    }

    #[test]
    fn test_app_config_deserialize_partial() {
        let json = "{\
            \"clocks\": [{\"name\": \"JST\", \"timezone\": \"Asia/Tokyo\"}],\
            \"font\": \"Arial\",\
            \"color\": \"#000\"\
        }";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize partial JSON");
        let config = result.unwrap();

        // Check specified values
        assert_eq!(config.clocks.len(), 1, "Should have one clock");
        assert_eq!(config.clocks[0].name, "JST", "Clock name should be JST");
        assert_eq!(config.clocks[0].timezone, "Asia/Tokyo", "Clock timezone should be Asia/Tokyo");
        assert_eq!(config.font, "Arial", "Font should be Arial");
        assert_eq!(config.color, "#000", "Color should be #000");

        // Check default values are still applied
        assert_eq!(config.format, "MM-DD ddd HH:mm", "Default format should still apply");
        assert_eq!(config.locale, "en", "Default locale should still apply");
    }

    #[test]
    fn test_app_config_deserialize_full() {
        let json = "{\
            \"clocks\": [\
                {\"name\": \"UTC\", \"timezone\": \"UTC\"},\
                {\"name\": \"JST\", \"timezone\": \"Asia/Tokyo\"}\
            ],\
            \"font\": \"Courier New\",\
            \"size\": 16,\
            \"color\": \"#ff0000\",\
            \"format\": \"HH:mm:ss\",\
            \"format2\": \"YYYY-MM-DD\",\
            \"locale\": \"ja\",\
            \"forefront\": true,\
            \"margin\": \"2em\",\
            \"timerIcon\": \"⏱\",\
            \"withoutNotification\": true,\
            \"maxTimerClockNumber\": 10,\
            \"epochClockName\": \"Unix Time\",\
            \"usetz\": true,\
            \"convtz\": \"Asia/Tokyo\",\
            \"disableHover\": false\
        }";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize full JSON");
        let config = result.unwrap();

        assert_eq!(config.clocks.len(), 2, "Should have two clocks");
        assert_eq!(config.font, "Courier New", "Font should match");
        assert_eq!(config.color, "#ff0000", "Color should match");
        assert_eq!(config.format, "HH:mm:ss", "Format should match");
        assert_eq!(config.format2, Some("YYYY-MM-DD".to_string()), "Format2 should match");
        assert_eq!(config.locale, "ja", "Locale should match");
        assert_eq!(config.forefront, true, "Forefront should be true");
        assert_eq!(config.margin, "2em", "Margin should match");
        assert_eq!(config.timer_icon, "⏱", "Timer icon should match");
        assert_eq!(config.without_notification, true, "Without notification should be true");
        assert_eq!(config.max_timer_clock_number, 10, "Max timer clock number should match");
        assert_eq!(config.epoch_clock_name, "Unix Time", "Epoch clock name should match");
        assert_eq!(config.usetz, true, "Usetz should be true");
        assert_eq!(config.convtz, "Asia/Tokyo", "Convtz should match");
        assert_eq!(config.disable_hover, false, "Disable hover should be false");
    }

    #[test]
    fn test_app_config_deserialize_invalid_json() {
        let json = "{ invalid json }";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_err(), "Should fail to deserialize invalid JSON");
    }

    #[test]
    fn test_clock_deserialize_empty() {
        let json = "{}";
        let result: Result<Clock, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize empty Clock JSON");
        let clock = result.unwrap();

        assert_eq!(clock.name, "UTC", "Default name should be UTC");
        assert_eq!(clock.timezone, "UTC", "Default timezone should be UTC");
        assert!(clock.countdown.is_none(), "Countdown should be None");
        assert!(clock.target.is_none(), "Target should be None");
    }

    #[test]
    fn test_clock_deserialize_partial() {
        let json = "{\"name\": \"JST\"}";
        let result: Result<Clock, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize partial Clock JSON");
        let clock = result.unwrap();

        assert_eq!(clock.name, "JST", "Name should be JST");
        assert_eq!(clock.timezone, "UTC", "Default timezone should still be UTC");
        assert!(clock.countdown.is_none(), "Countdown should be None");
        assert!(clock.target.is_none(), "Target should be None");
    }

    #[test]
    fn test_clock_deserialize_full() {
        let json = "{\
            \"name\": \"EST\",\
            \"timezone\": \"America/New_York\",\
            \"countdown\": \"10:00\",\
            \"target\": \"2024-01-01 12:00:00\"\
        }";
        let result: Result<Clock, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize full Clock JSON");
        let clock = result.unwrap();

        assert_eq!(clock.name, "EST", "Name should match");
        assert_eq!(clock.timezone, "America/New_York", "Timezone should match");
        assert_eq!(clock.countdown, Some("10:00".to_string()), "Countdown should match");
        assert_eq!(clock.target, Some("2024-01-01 12:00:00".to_string()), "Target should match");
    }

    #[test]
    fn test_in_font_size_deserialize_int() {
        let json = "14";
        let result: Result<InFontSize, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize integer");
        match result.unwrap() {
            InFontSize::Int(value) => assert_eq!(value, 14, "Integer value should match"),
            _ => panic!("Should be Int variant"),
        }
    }

    #[test]
    fn test_in_font_size_deserialize_str() {
        let json = "\"16px\"";
        let result: Result<InFontSize, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize string");
        match result.unwrap() {
            InFontSize::Str(value) => assert_eq!(value, "16px", "String value should match"),
            _ => panic!("Should be Str variant"),
        }
    }

    #[test]
    fn test_app_config_size_int() {
        let json = "{\"size\": 18}";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize size as integer");
        match result.unwrap().size {
            InFontSize::Int(value) => assert_eq!(value, 18, "Size should be 18"),
            _ => panic!("Size should be Int variant"),
        }
    }

    #[test]
    fn test_app_config_size_str() {
        let json = "{\"size\": \"20px\"}";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize size as string");
        match result.unwrap().size {
            InFontSize::Str(value) => assert_eq!(value, "20px", "Size should be 20px"),
            _ => panic!("Size should be Str variant"),
        }
    }

    #[test]
    fn test_app_config_size_alias_font_size() {
        let json = "{\"fontSize\": 22}";
        let result: Result<AppConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize fontSize alias");
        match result.unwrap().size {
            InFontSize::Int(value) => assert_eq!(value, 22, "Size should be 22"),
            _ => panic!("Size should be Int variant"),
        }
    }
}

