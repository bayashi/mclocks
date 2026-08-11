//! Single TODO list panel (sticky-like look; persist like sticky.json).
//! Position/size: window-state plugin (same as sticky). Content: todo.json.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use tauri::webview::Url;
use tauri::{AppHandle, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder};

use crate::config::ContextConfig;

const IS_DEV: bool = tauri::is_dev();

pub const WINDOW_LABEL: &str = "todo";

const DEFAULT_WIDTH: f64 = 450.0;
const DEFAULT_HEIGHT: f64 = 320.0;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub status: String,
    #[serde(default)]
    pub memo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TodoPersistData {
    #[serde(default)]
    pub items: Vec<TodoItem>,
    /// Per-panel forefront override. None means inherit from main clock config.
    #[serde(default)]
    pub forefront: Option<bool>,
}

pub struct TodoPersistStore {
    file_path: PathBuf,
    data: Mutex<TodoPersistData>,
}

impl TodoPersistStore {
    pub fn new(identifier: &str) -> Self {
        let file_name = if IS_DEV {
            "dev.todo.json"
        } else {
            "todo.json"
        };
        let base = BaseDirs::new()
            .map(|bd| bd.config_dir().join(identifier))
            .unwrap_or_else(|| PathBuf::from("."));
        let file_path = base.join(file_name);

        let data: TodoPersistData = if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            TodoPersistData::default()
        };

        Self {
            file_path,
            data: Mutex::new(data),
        }
    }

    fn write_file(&self, data: &TodoPersistData) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(&self.file_path, json).map_err(|e| e.to_string())
    }
}

fn build_panel_url() -> WebviewUrl {
    if IS_DEV {
        return Url::parse("http://localhost:1420/")
            .map(WebviewUrl::External)
            .unwrap_or_else(|_| WebviewUrl::App("index.html".into()));
    }
    WebviewUrl::App("index.html".into())
}

fn resolve_forefront_for_app<R: Runtime>(app: &AppHandle<R>, store: &TodoPersistStore) -> bool {
    if let Ok(data) = store.data.lock() {
        if let Some(v) = data.forefront {
            return v;
        }
    }
    if let Some(ctx) = app.try_state::<std::sync::Arc<ContextConfig>>() {
        return crate::config::load_app_config_for_identifier(&ctx.app_identifier)
            .map(|c| c.forefront)
            .unwrap_or(false);
    }
    false
}

fn reveal_todo_panel<R: Runtime>(app: &AppHandle<R>, forefront: bool) {
    let Some(w) = app.get_webview_window(WINDOW_LABEL) else {
        return;
    };
    // Force on-top while revealing so a buried panel is findable.
    // Do not center: keep last position (window-state / live geometry).
    let _ = w.set_always_on_top(true);
    let _ = w.show();
    let _ = w.set_focus();
    if !forefront {
        let _ = w.set_always_on_top(false);
    }
}

#[tauri::command]
pub fn todo_show_panel(
    app: AppHandle,
    store: State<'_, TodoPersistStore>,
) -> Result<(), String> {
    let forefront = resolve_forefront_for_app(&app, &store);
    show_todo_panel_with_forefront(&app, forefront)
}

pub fn show_todo_panel<R: Runtime>(app: &AppHandle<R>) {
    let forefront = if let Some(store) = app.try_state::<TodoPersistStore>() {
        resolve_forefront_for_app(app, &store)
    } else {
        false
    };
    if let Err(e) = show_todo_panel_with_forefront(app, forefront) {
        eprintln!("[todo] show panel failed: {}", e);
    }
}

fn show_todo_panel_with_forefront<R: Runtime>(
    app: &AppHandle<R>,
    forefront: bool,
) -> Result<(), String> {
    if app.get_webview_window(WINDOW_LABEL).is_some() {
        reveal_todo_panel(app, forefront);
        return Ok(());
    }

    // Same pattern as sticky: build from a worker thread via run_on_main_thread.
    // Calling build() while the invoke/tray handler waits on run_on_main_thread deadlocks.
    // Initial .center() is a fallback; window-state restores position/size when present.
    let url = build_panel_url();
    let app_h = app.clone();
    std::thread::spawn(move || {
        let app_for_build = app_h.clone();
        let _ = app_h.run_on_main_thread(move || {
            let win = match WebviewWindowBuilder::new(&app_for_build, WINDOW_LABEL, url)
                .title("mclocks TODO")
                .decorations(false)
                .shadow(false)
                .transparent(true)
                .resizable(true)
                .minimizable(false)
                .maximizable(false)
                .skip_taskbar(true)
                .always_on_top(forefront)
                .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
                .visible(false)
                .center()
                .build()
            {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[todo] failed to build window: {}", e);
                    return;
                }
            };
            let _ = win.show();
            let _ = win.set_focus();
        });
    });
    Ok(())
}

#[tauri::command]
pub fn todo_close_panel(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn todo_load(store: State<'_, TodoPersistStore>) -> Result<TodoPersistData, String> {
    let data = store
        .data
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(data)
}

#[tauri::command]
pub fn todo_save(
    store: State<'_, TodoPersistStore>,
    items: Vec<TodoItem>,
    forefront: Option<bool>,
) -> Result<(), String> {
    let mut data = store.data.lock().map_err(|e| e.to_string())?;
    data.items = items;
    if forefront.is_some() {
        data.forefront = forefront;
    }
    store.write_file(&data)
}
