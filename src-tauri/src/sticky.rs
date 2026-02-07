use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use std::fs;

use directories::BaseDirs;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::Url;
use uuid::Uuid;

use crate::config::{ContextConfig, load_config};

const IS_DEV: bool = tauri::is_dev();

#[allow(unused_macros)]
macro_rules! debug_log {
	($($arg:tt)*) => {
		if tauri::is_dev() {
			eprintln!($($arg)*);
		}
	};
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
	data: Mutex<HashMap<String, String>>,
}

impl StickyPersistStore {
	pub fn new(identifier: &str) -> Self {
		let file_name = if IS_DEV { "dev.sticky.json" } else { "sticky.json" };
		let file_path = BaseDirs::new()
			.map(|bd| bd.config_dir().join(identifier).join(file_name))
			.unwrap_or_else(|| PathBuf::from(file_name));

		let data = if file_path.exists() {
			fs::read_to_string(&file_path)
				.ok()
				.and_then(|content| serde_json::from_str(&content).ok())
				.unwrap_or_default()
		} else {
			HashMap::new()
		};

		debug_log!("[sticky] StickyPersistStore: path={:?} entries={}", file_path, data.len());

		Self {
			file_path,
			data: Mutex::new(data),
		}
	}

	fn write_file(&self, data: &HashMap<String, String>) -> Result<(), String> {
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

/// Spawn a sticky window as a child of the main window
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
			Err(e) => {
				debug_log!("[sticky] spawn_sticky_window: invalid url: {}", e);
				return;
			}
		};

		let app_for_closure = app_for_main.clone();
		let schedule_target = app_for_main.clone();
		let result = schedule_target.run_on_main_thread(move || {
			debug_log!("[sticky] spawn_sticky_window(main): build start label={}", label_for_build);

			let main = match app_for_closure.get_webview_window("main") {
				Some(w) => w,
				None => {
					debug_log!("[sticky] spawn_sticky_window(main): main window not found");
					return;
				}
			};

			let w = WebviewWindowBuilder::new(&app_for_closure, label_for_build.clone(), url)
				.title("mclocks")
				.transparent(true)
				.decorations(false)
				.shadow(false)
				.resizable(true)
				.minimizable(false)
				.maximizable(false)
				.inner_size(360.0, 100.0)
				.always_on_top(forefront);

			let w = match w.parent(&main) {
				Ok(next) => next,
				Err(e) => {
					debug_log!("[sticky] spawn_sticky_window(main): parent failed: {}", e);
					return;
				}
			};

			let w = w.center();

			match w.build() {
				Ok(_) => debug_log!("[sticky] spawn_sticky_window(main): build ok"),
				Err(e) => debug_log!("[sticky] spawn_sticky_window(main): build failed: {}", e),
			}
		});

		if let Err(e) = result {
			debug_log!("[sticky] spawn_sticky_window(main): schedule failed: {}", e);
		}
	});
}

#[tauri::command]
pub fn create_sticky(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>, text: String) -> Result<String, String> {
	debug_log!("[sticky] create_sticky: start");
	let id = uuid_v4();
	let label = format!("sticky-{}", id);
	debug_log!("[sticky] create_sticky: label={}", label);

	{
		let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		map.insert(label.clone(), text.clone());
	}

	// Persist to file
	{
		let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
		data.insert(label.clone(), text);
		if let Err(e) = persist.write_file(&data) {
			debug_log!("[sticky] create_sticky: persist failed: {}", e);
		}
	}

	let cfg = load_config(cfg_state)?;
	debug_log!("[sticky] create_sticky: load_config ok");

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
	data.insert(id, text);
	persist.write_file(&data)
}

/// Remove sticky from persistent store (called when user closes a sticky)
#[tauri::command]
pub fn delete_sticky_text(persist: State<'_, StickyPersistStore>, id: String) -> Result<(), String> {
	let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	data.remove(&id);
	persist.write_file(&data)
}

/// Restore all persisted stickies by recreating their windows
#[tauri::command]
pub fn restore_stickies(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>) -> Result<(), String> {
	let notes = {
		let data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
		data.clone()
	};

	if notes.is_empty() {
		debug_log!("[sticky] restore_stickies: no saved stickies");
		return Ok(());
	}

	debug_log!("[sticky] restore_stickies: restoring {} stickies", notes.len());

	let cfg = load_config(cfg_state)?;

	{
		let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		for (label, text) in &notes {
			map.insert(label.clone(), text.clone());
		}
	}

	for (label, _) in notes {
		spawn_sticky_window(app.clone(), label, cfg.forefront);
	}

	Ok(())
}