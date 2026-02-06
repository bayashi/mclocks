use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;

use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::Url;
use uuid::Uuid;

use crate::config::{ContextConfig, load_config};

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

fn uuid_v4() -> String {
	Uuid::new_v4().to_string()
}

#[tauri::command]
pub fn create_sticky(app: AppHandle, cfg_state: State<'_, std::sync::Arc<ContextConfig>>, sticky_store: State<'_, StickyInitStore>, text: String) -> Result<String, String> {
	debug_log!("[sticky] create_sticky: start");
	let id = uuid_v4();
	let label = format!("sticky-{}", id);
	debug_log!("[sticky] create_sticky: label={}", label);

	{
		let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
		map.insert(label.clone(), text);
	}

	let cfg = load_config(cfg_state)?;
	debug_log!("[sticky] create_sticky: load_config ok");

	let forefront = cfg.forefront;
	let label_for_thread = label.clone();
	let app_for_thread = app.clone();
	let is_dev = tauri::is_dev();

	thread::spawn(move || {
		let app_for_main = app_for_thread.clone();
		let label_for_build = label_for_thread.clone();
		let url = if is_dev {
			Url::parse("http://localhost:1420/").map(WebviewUrl::External)
		} else {
			Ok(WebviewUrl::App("index.html".into()))
		};

		let url = match url {
			Ok(u) => u,
			Err(e) => {
				debug_log!("[sticky] create_sticky(main): invalid url: {}", e);
				return;
			}
		};

		let app_for_closure = app_for_main.clone();
		let schedule_target = app_for_main.clone();
		let result = schedule_target.run_on_main_thread(move || {
			debug_log!("[sticky] create_sticky(main): build start label={}", label_for_build);

			let main = match app_for_closure.get_webview_window("main") {
				Some(w) => w,
				None => {
					debug_log!("[sticky] create_sticky(main): main window not found");
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
					debug_log!("[sticky] create_sticky(main): parent failed: {}", e);
					return;
				}
			};

			let w = w.center();

			match w.build() {
				Ok(_) => debug_log!("[sticky] create_sticky(main): build ok"),
				Err(e) => debug_log!("[sticky] create_sticky(main): build failed: {}", e),
			}
		});

		if let Err(e) = result {
			debug_log!("[sticky] create_sticky(main): schedule failed: {}", e);
		}
	});

	Ok(label)
}

#[tauri::command]
pub fn sticky_take_init_text(sticky_store: State<'_, StickyInitStore>, id: String) -> Result<String, String> {
	let mut map = sticky_store.text_by_window_label.lock().map_err(|_| "Failed to lock sticky store".to_string())?;
	Ok(map.remove(&id).unwrap_or_default())
}

