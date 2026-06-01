//! Three-month calendar panel (tray menu).

use tauri::WebviewWindowBuilder;
use tauri::webview::Url;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl};

pub const WINDOW_LABEL: &str = "calendar";

const IS_DEV: bool = tauri::is_dev();

const DEFAULT_WIDTH: f64 = 720.0;
const DEFAULT_HEIGHT: f64 = 220.0;

fn build_panel_url() -> WebviewUrl {
    if IS_DEV {
        return Url::parse("http://localhost:1420/")
            .map(WebviewUrl::External)
            .unwrap_or_else(|_| WebviewUrl::App("index.html".into()));
    }
    WebviewUrl::App("index.html".into())
}

pub fn show_calendar_panel<R: Runtime>(app: &AppHandle<R>) {
    let url = build_panel_url();

    if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
        let _ = w.set_always_on_top(true);
        let _ = w.eval("window.dispatchEvent(new Event('mclocks-calendar-show'));");
        return;
    }

    let app_h = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Err(e) = WebviewWindowBuilder::new(&app_h, WINDOW_LABEL, url)
            .title("mclocks")
            .decorations(false)
            .shadow(false)
            .transparent(true)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
            .visible(false)
            .center()
            .build()
        {
            eprintln!("[calendar] failed to build window: {}", e);
        }
    });
}

#[tauri::command]
pub fn calendar_close_panel(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
