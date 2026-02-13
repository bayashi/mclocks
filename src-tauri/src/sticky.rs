use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use std::fs;

use base64::{Engine as _, engine::general_purpose};
use directories::BaseDirs;
use tauri::{AppHandle, State, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::Url;
use uuid::Uuid;

use serde::{Serialize, Deserialize};

use crate::config::{ContextConfig, load_config};

const IS_DEV: bool = tauri::is_dev();

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024; // 10MB

/// Persistent data for a single sticky note
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StickyData {
	pub text: String,
	/// Content type: None or "text" for text, "image" for image sticky
	#[serde(default)]
	pub content_type: Option<String>,
	/// Image filename stored under sticky_images directory
	#[serde(default)]
	pub image_filename: Option<String>,
	#[serde(default)]
	pub is_open: bool,
	#[serde(default)]
	pub open_width: Option<f64>,
	#[serde(default)]
	pub open_height: Option<f64>,
	/// Per-sticky forefront override. None means inherit from main clock config.
	#[serde(default)]
	pub forefront: Option<bool>,
	/// Per-sticky lock state. None means unlocked (default).
	#[serde(default)]
	pub locked: Option<bool>,
}

impl StickyData {
	fn new(text: String) -> Self {
		Self { text, content_type: None, image_filename: None, is_open: false, open_width: None, open_height: None, forefront: None, locked: None }
	}

	fn new_image(image_filename: String) -> Self {
		Self { text: String::new(), content_type: Some("image".to_string()), image_filename: Some(image_filename), is_open: false, open_width: None, open_height: None, forefront: None, locked: None }
	}

	fn is_image(&self) -> bool {
		self.content_type.as_deref() == Some("image")
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
	pub locked: Option<bool>,
	pub content_type: Option<String>,
}

/// Init content returned to JS when a sticky window starts up
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickyInitContent {
	pub text: String,
	pub content_type: Option<String>,
}

pub struct StickyInitStore {
	pub init_by_window_label: Mutex<HashMap<String, StickyInitContent>>,
}

impl Default for StickyInitStore {
	fn default() -> Self {
		Self {
			init_by_window_label: Mutex::new(HashMap::new()),
		}
	}
}

/// Persistent file-backed store for sticky note contents
pub struct StickyPersistStore {
	file_path: PathBuf,
	images_dir: PathBuf,
	data: Mutex<HashMap<String, StickyData>>,
}

impl StickyPersistStore {
	pub fn new(identifier: &str) -> Self {
		let file_name = if IS_DEV { "dev.sticky.json" } else { "sticky.json" };
		let images_dir_name = if IS_DEV { "dev.sticky_images" } else { "sticky_images" };
		let base = BaseDirs::new()
			.map(|bd| bd.config_dir().join(identifier))
			.unwrap_or_else(|| PathBuf::from("."));
		let file_path = base.join(file_name);
		let images_dir = base.join(images_dir_name);

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
			images_dir,
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

	fn save_image(&self, filename: &str, data: &[u8]) -> Result<(), String> {
		fs::create_dir_all(&self.images_dir).map_err(|e| e.to_string())?;
		let path = self.images_dir.join(filename);
		fs::write(&path, data).map_err(|e| e.to_string())
	}

	fn load_image(&self, filename: &str) -> Result<Vec<u8>, String> {
		let path = self.images_dir.join(filename);
		fs::read(&path).map_err(|e| e.to_string())
	}

	fn delete_image(&self, filename: &str) -> Result<(), String> {
		let path = self.images_dir.join(filename);
		if path.exists() {
			fs::remove_file(&path).map_err(|e| e.to_string())?;
		}
		Ok(())
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
		let mut map = sticky_store.init_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		map.insert(label.clone(), StickyInitContent { text: text.clone(), content_type: None });
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
pub fn create_sticky_image(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>, image_base64: String) -> Result<String, String> {
	let image_bytes = general_purpose::STANDARD.decode(&image_base64)
		.map_err(|e| format!("Failed to decode base64: {}", e))?;

	if image_bytes.len() > MAX_IMAGE_BYTES {
		let size_mb = image_bytes.len() as f64 / (1024.0 * 1024.0);
		return Err(format!("Image is too large ({:.1} MB). Maximum size is 10 MB.", size_mb));
	}

	let id = uuid_v4();
	let label = format!("sticky-{}", id);
	let image_filename = format!("{}.png", id);

	// Save image file
	persist.save_image(&image_filename, &image_bytes)?;

	{
		let mut map = sticky_store.init_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		map.insert(label.clone(), StickyInitContent { text: String::new(), content_type: Some("image".to_string()) });
	}

	// Persist to file
	{
		let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
		data.insert(label.clone(), StickyData::new_image(image_filename));
		let _ = persist.write_file(&data);
	}

	let cfg = load_config(cfg_state)?;

	spawn_sticky_window(app, label.clone(), cfg.forefront);

	Ok(label)
}

#[tauri::command]
pub fn sticky_take_init_content(sticky_store: State<'_, StickyInitStore>, id: String) -> Result<StickyInitContent, String> {
	let mut map = sticky_store.init_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
	Ok(map.remove(&id).unwrap_or_else(|| StickyInitContent { text: String::new(), content_type: None }))
}

#[tauri::command]
pub fn load_sticky_image(persist: State<'_, StickyPersistStore>, id: String) -> Result<String, String> {
	let data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	let sticky = data.get(&id).ok_or("Sticky not found")?;
	let filename = sticky.image_filename.as_ref().ok_or("No image for this sticky")?;
	let bytes = persist.load_image(filename)?;
	Ok(general_purpose::STANDARD.encode(&bytes))
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
	// Delete associated image file if this is an image sticky
	if let Some(sticky) = data.get(&id) {
		if sticky.is_image() {
			if let Some(filename) = &sticky.image_filename {
				let _ = persist.delete_image(filename);
			}
		}
	}
	data.remove(&id);
	persist.write_file(&data)
}

/// Save sticky open/close state, open-mode size, and forefront override
#[tauri::command]
pub fn save_sticky_state(persist: State<'_, StickyPersistStore>, id: String, is_open: bool, open_width: Option<f64>, open_height: Option<f64>, forefront: Option<bool>, locked: Option<bool>) -> Result<(), String> {
	let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;
	let entry = data.entry(id).or_insert_with(|| StickyData::new(String::new()));
	entry.is_open = is_open;
	entry.open_width = open_width;
	entry.open_height = open_height;
	entry.forefront = forefront;
	entry.locked = locked;
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
		locked: d.locked,
		content_type: d.content_type.clone(),
	}))
}

/// Restore all persisted stickies by recreating their windows.
/// Also restores orphaned image files (present in sticky_images dir but not in sticky.json)
/// as default-state image stickies.
#[tauri::command]
pub fn restore_stickies(app: AppHandle, cfg_state: State<'_, Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, persist: State<'_, StickyPersistStore>) -> Result<(), String> {
	let notes = {
		let mut data = persist.data.lock().map_err(|_| "Failed to lock persist store".to_string())?;

		// Collect image filenames already tracked in sticky.json
		let known_images: std::collections::HashSet<String> = data.values()
			.filter_map(|s| s.image_filename.clone())
			.collect();

		// Scan sticky_images directory for orphaned image files
		let mut orphans_added = false;
		if persist.images_dir.exists() {
			if let Ok(entries) = fs::read_dir(&persist.images_dir) {
				for entry in entries.flatten() {
					let path = entry.path();
					if !path.is_file() {
						continue;
					}
					// Validate: must be {UUIDv4}.png and within size limit
					let stem = match path.file_stem().and_then(|s| s.to_str()) {
						Some(s) => s,
						None => continue,
					};
					let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
					if ext != "png" || Uuid::parse_str(stem).is_err() {
						continue;
					}
					if let Ok(meta) = entry.metadata() {
						if meta.len() as usize > MAX_IMAGE_BYTES {
							continue;
						}
					} else {
						continue;
					}
					let filename = format!("{}.png", stem);
					if known_images.contains(&filename) {
						continue;
					}
					let label = format!("sticky-{}", stem);
					// Avoid collision with existing labels
					if data.contains_key(&label) {
						continue;
					}
					data.insert(label, StickyData::new_image(filename));
					orphans_added = true;
				}
			}
		}

		// Persist newly added orphan entries so they appear in sticky.json
		if orphans_added {
			let _ = persist.write_file(&data);
		}

		data.clone()
	};

	if notes.is_empty() {
		return Ok(());
	}

	let cfg = load_config(cfg_state)?;

	{
		let mut map = sticky_store.init_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		for (label, sticky_data) in &notes {
			map.insert(label.clone(), StickyInitContent {
				text: sticky_data.text.clone(),
				content_type: sticky_data.content_type.clone(),
			});
		}
	}

	for (label, sticky_data) in notes {
		// Use per-sticky forefront if persisted, otherwise fall back to main clock config
		let forefront = sticky_data.forefront.unwrap_or(cfg.forefront);
		spawn_sticky_window(app.clone(), label, forefront);
	}

	Ok(())
}
