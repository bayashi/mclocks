use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use std::fs;

use directories::BaseDirs;
use tauri::{AppHandle, State, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::Url;
use uuid::Uuid;

use serde::{Serialize, Deserialize};

use crate::config::{ContextConfig, load_config};

const IS_DEV: bool = tauri::is_dev();

/// Persistent data for a single sticky note
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StickyData {
	pub text: String,
	#[serde(default)]
	pub is_open: bool,
	#[serde(default)]
	pub open_width: Option<f64>,
	#[serde(default)]
	pub open_height: Option<f64>,
	/// Per-sticky forefront override. None means inherit from main clock config.
	#[serde(default)]
	pub forefront: Option<bool>,
}

impl StickyData {
	fn new(text: String) -> Self {
		Self { text, is_open: false, open_width: None, open_height: None, forefront: None }
	}
}

/// State info returned to JS
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickyStateInfo {
	pub is_open: bool,
	pub open_width: Option<f64>,
	pub open_height: Option<f64>,
	pub forefront: Option<bool>,
}

pub struct StickyInitStore {
	pub	text_by_window_label: Mutex<HashMap<String, String>>,
}

impl Default for StickyInitStore {
	fn default() -> Self {
		Self {
			text_by_window_label: Mutex::new(HashMap::new()),
		}
	}
}

/// Persistent file-backed store for sticky note contents
pub struct StickyPersistStore {
	file_path: PathBuf,
	data: Mutex<HashMap<String, StickyData>>,
}

impl StickyPersistStore {
	pub fn new(identifier: &str) -> Self {
		let file_name = if IS_DEV { "dev.sticky.json" } else { "sticky.json" };
		let file_path = BaseDirs::new()
			.map(|bd| bd.config_dir().join(identifier).join(file_name))
			.unwrap_or_else(|| PathBuf::from(file_name));

		let data: HashMap<String, StickyData> = if file_path.exists() {
			fs::read_to_string(&file_path)
				.ok()
				.and_then(|content| serde_json::from_str(&content).ok())
				.unwrap_or_default()
		} else {
			HashMap::new()
		};

		Self {
			file_path,
			data: Mutex::new(data),
		}
	}

	fn write_file(&self, data: &HashMap<String, StickyData>) -> Result<(), String> {
		if let Some(parent) = self.file_path.parent() {
			fs::create_dir_all(parent).map_err(|e| e.to_string())?;
		}
		let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
		fs::write(&self.file_path, json).map_err(|e| e.to_string())
	}
}

fn uuid_v4() -> String {
	Uuid::new_v4().to_string()
}

/// Spawn a sticky window as an independent window (not a child of main)
fn spawn_sticky_window(app: AppHandle, label: String, forefront: bool) {
	let is_dev = tauri::is_dev();
	thread::spawn(move || {
		let app_for_main = app.clone();
		let label_for_build = label.clone();
		let url = if is_dev {
			Url::parse("http://localhost:1420/").map(WebviewUrl::External)
		} else {
			Ok(WebviewUrl::App("index.html".into()))
		};

		let url = match url {
			Ok(u) => u,
			Err(_) => return,
		};

		let app_for_closure = app_for_main.clone();
		let schedule_target = app_for_main.clone();
		let result = schedule_target.run_on_main_thread(move || {
			let w = WebviewWindowBuilder::new(&app_for_closure, label_for_build.clone(), url)
				.title("mclocks")
				.transparent(true)
				.decorations(false)
				.shadow(false)
				.resizable(true)
				.minimizable(false)
				.maximizable(false)
				.skip_taskbar(true)
				.inner_size(360.0, 100.0)
				.always_on_top(forefront);

			let w = w.center();

			let _ = w.build();
		});

		let _ = result;
	});
}

#[tauri::command]
pub fn create_sticky(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>, text: String) -> Result<String, String> {
	let id = uuid_v4();
	let label = format!("sticky-{}", id);

	{
		let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		map.insert(label.clone(), text.clone());
	}

	// Persist to file
	{
		let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
		data.insert(label.clone(), StickyData::new(text));
		let _ = persist.write_file(&data);
	}

	let cfg = load_config(cfg_state)?;

	spawn_sticky_window(app, label.clone(), cfg.forefront);

	Ok(label)
}

#[tauri::command]
pub fn sticky_take_init_text(sticky_store: State<'_, StickyInitStore>, id: String) -> Result<String, String> {
	let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
	Ok(map.remove(&id).unwrap_or_default())
}

/// Save sticky text to persistent store (called on textarea input with debounce)
#[tauri::command]
pub fn save_sticky_text(persist: State<'_, StickyPersistStore>, id: String, text: String) -> Result<(), String> {
	let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	let entry = data.entry(id).or_insert_with(|| StickyData::new(String::new()));
	entry.text = text;
	persist.write_file(&data)
}

/// Remove sticky from persistent store (called when user closes a sticky)
#[tauri::command]
pub fn delete_sticky_text(persist: State<'_, StickyPersistStore>, id: String) -> Result<(), String> {
	let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	data.remove(&id);
	persist.write_file(&data)
}

/// Save sticky open/close state, open-mode size, and forefront override
#[tauri::command]
pub fn save_sticky_state(persist: State<'_, StickyPersistStore>, id: String, is_open: bool, open_width: Option<f64>, open_height: Option<f64>, forefront: Option<bool>) -> Result<(), String> {
	let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	let entry = data.entry(id).or_insert_with(|| StickyData::new(String::new()));
	entry.is_open = is_open;
	entry.open_width = open_width;
	entry.open_height = open_height;
	entry.forefront = forefront;
	persist.write_file(&data)
}

/// Load sticky open/close state and open-mode size
#[tauri::command]
pub fn load_sticky_state(persist: State<'_, StickyPersistStore>, id: String) -> Result<Option<StickyStateInfo>, String> {
	let data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	Ok(data.get(&id).map(|d| StickyStateInfo {
		is_open: d.is_open,
		open_width: d.open_width,
		open_height: d.open_height,
		forefront: d.forefront,
	}))
}

/// Restore all persisted stickies by recreating their windows
#[tauri::command]
pub fn restore_stickies(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>) -> Result<(), String> {
	let notes = {
		let data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
		data.clone()
	};

	if notes.is_empty() {
		return Ok(());
	}

	let cfg = load_config(cfg_state)?;

	{
		let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		for (label, sticky_data) in &notes {
			map.insert(label.clone(), sticky_data.text.clone());
		}
	}

	for (label, sticky_data) in notes {
		// Use per-sticky forefront if persisted, otherwise fall back to main clock config
		let forefront = sticky_data.forefront.unwrap_or(cfg.forefront);
		spawn_sticky_window(app.clone(), label, forefront);
	}

	Ok(())
}
