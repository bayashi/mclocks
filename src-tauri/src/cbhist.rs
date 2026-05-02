//! In-memory copy-history panel (tray-triggered); internal codename cbhist.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, Position, Runtime, Size, WebviewUrl};
use tauri::webview::Url;
use tauri::WebviewWindowBuilder;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::WINDOW_NAME;

pub const WINDOW_LABEL: &str = "cbhist";

const MAX_CLIPBOARD_UTF8_BYTES: usize = 1_048_576;
const POLL_INTERVAL: Duration = Duration::from_millis(140);

const TRAY_GAP_PX: i32 = 12;

#[derive(Clone, Debug)]
struct HistoryEntry {
	text: String,
	truncated_from_clipboard: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CbhistItemDto {
	pub text: String,
	pub utf8_byte_len: usize,
	pub unicode_scalar_count: usize,
	pub line_count: usize,
	pub truncated_from_clipboard: bool,
}

pub struct CbhistStore {
	deque: Mutex<VecDeque<HistoryEntry>>,
	last_raw_clipboard: Mutex<Option<String>>,
	pub max_entries: usize,
	pub disabled: bool,
	pub panel_width: f64,
	pub panel_height: f64,
}

impl CbhistStore {
	pub fn new(max_entries: usize, disabled: bool, panel_width: f64, panel_height: f64) -> Self {
		Self {
			deque: Mutex::new(VecDeque::new()),
			last_raw_clipboard: Mutex::new(None),
			max_entries,
			disabled,
			panel_width,
			panel_height,
		}
	}
}

fn truncate_utf8_to_max_bytes(s: &str, max_bytes: usize) -> (String, bool) {
	if s.len() <= max_bytes {
		return (s.to_string(), false);
	}
	let mut cut = max_bytes;
	while cut > 0 && !s.is_char_boundary(cut) {
		cut -= 1;
	}
	(s[..cut].to_string(), true)
}

fn line_count_metric(s: &str) -> usize {
	if s.is_empty() {
		return 1;
	}
	let n = s.bytes().filter(|&b| b == b'\n').count();
	n + 1
}

fn dto_from_entry(entry: &HistoryEntry) -> CbhistItemDto {
	let text_len = entry.text.len();
	let scalar_n = entry.text.chars().count();
	CbhistItemDto {
		text: entry.text.clone(),
		utf8_byte_len: text_len,
		unicode_scalar_count: scalar_n,
		line_count: line_count_metric(&entry.text),
		truncated_from_clipboard: entry.truncated_from_clipboard,
	}
}

fn physical_tray_bounds(tray: tauri::Rect) -> (i32, i32, u32, u32) {
	let (x, y) = match tray.position {
		Position::Physical(p) => (p.x, p.y),
		Position::Logical(l) => (l.x as i32, l.y as i32),
	};
	let (w, h) = match tray.size {
		Size::Physical(s) => (s.width, s.height),
		Size::Logical(s) => (s.width as u32, s.height as u32),
	};
	(x, y, w, h)
}

fn panel_physical_size(mon: &tauri::Monitor, logical_w: f64, logical_h: f64) -> (u32, u32) {
	let s = mon.scale_factor();
	let w = (logical_w * s).ceil() as u32;
	let h = (logical_h * s).ceil() as u32;
	(w.max(1), h.max(1))
}

fn clamp_to_work_area(x: i32, y: i32, win_w: u32, win_h: u32, mon: &tauri::Monitor) -> PhysicalPosition<i32> {
	let wa = mon.work_area();
	let wx = wa.position.x;
	let wy = wa.position.y;
	let ww = wa.size.width as i32;
	let wh = wa.size.height as i32;
	let mut cx = x.clamp(wx, (wx + ww - win_w as i32).max(wx));
	let mut cy = y.clamp(wy, (wy + wh - win_h as i32).max(wy));
	if cx + win_w as i32 > wx + ww {
		cx = wx + ww - win_w as i32;
	}
	if cy + win_h as i32 > wy + wh {
		cy = wy + wh - win_h as i32;
	}
	PhysicalPosition { x: cx, y: cy }
}

fn monitor_for_point<R: Runtime>(app: &AppHandle<R>, px: i32, py: i32) -> Option<tauri::Monitor> {
	let main = app.get_webview_window(WINDOW_NAME)?;
	let monitors = main.available_monitors().ok()?;
	for m in monitors {
		let p = m.position();
		let s = m.size();
		let right = p.x + s.width as i32;
		let bottom = p.y + s.height as i32;
		if px >= p.x && px < right && py >= p.y && py < bottom {
			return Some(m);
		}
	}
	main.primary_monitor().ok().flatten()
}

fn panel_position_for_tray<R: Runtime>(
	app: &AppHandle<R>,
	tray_rect: Option<tauri::Rect>,
	logical_w: f64,
	logical_h: f64,
) -> PhysicalPosition<i32> {
	if let Some(tr) = tray_rect {
		let (tx, ty, tw, th) = physical_tray_bounds(tr);
		let cx = tx + tw as i32 / 2;
		let top_ref = ty;
		if let Some(m) = monitor_for_point(app, cx, top_ref) {
			let (phys_w, phys_h) = panel_physical_size(&m, logical_w, logical_h);
			let x = cx - phys_w as i32 / 2;
			let wy = m.work_area().position.y;
			let y_above = top_ref - TRAY_GAP_PX - phys_h as i32;
			let y = if y_above < wy {
				top_ref + th as i32 + TRAY_GAP_PX
			} else {
				y_above
			};
			return clamp_to_work_area(x, y, phys_w, phys_h, &m);
		}
	}
	let main = app.get_webview_window(WINDOW_NAME);
	if let Some(w) = &main {
		if let Ok(Some(m)) = w.primary_monitor() {
			let (phys_w, phys_h) = panel_physical_size(&m, logical_w, logical_h);
			let wa = m.work_area();
			let wx = wa.position.x;
			let wy = wa.position.y;
			let ww = wa.size.width as i32;
			let wh = wa.size.height as i32;
			let x = wx + ww / 2 - phys_w as i32 / 2;
			let y = wy + wh / 2 - phys_h as i32 / 2;
			return clamp_to_work_area(x, y, phys_w, phys_h, &m);
		}
	}
	PhysicalPosition { x: 100, y: 100 }
}

fn maybe_record_clipboard_update<R: Runtime>(handle: &AppHandle<R>, store: &CbhistStore) {
	if store.disabled {
		return;
	}
	let text = match handle.clipboard().read_text() {
		Ok(t) => t,
		Err(_) => return,
	};
	{
		let mut last = match store.last_raw_clipboard.lock() {
			Ok(g) => g,
			Err(_) => return,
		};
		if last.as_ref() == Some(&text) {
			return;
		}
		*last = Some(text.clone());
	}
	let (normalized, trunc) = truncate_utf8_to_max_bytes(&text, MAX_CLIPBOARD_UTF8_BYTES);
	let mut dq = match store.deque.lock() {
		Ok(g) => g,
		Err(_) => return,
	};
	if dq
		.front()
		.map(|e| e.text.as_str())
		== Some(normalized.as_str())
	{
		return;
	}
	dq.retain(|e| e.text != normalized);
	dq.push_front(HistoryEntry {
		text: normalized,
		truncated_from_clipboard: trunc,
	});
	while dq.len() > store.max_entries {
		dq.pop_back();
	}
}

pub fn spawn_cbhist_watcher<R: Runtime>(app: AppHandle<R>, store: Arc<CbhistStore>) {
	thread::spawn(move || loop {
		thread::sleep(POLL_INTERVAL);
		let scheduler = app.clone();
		let handle_for_cb = app.clone();
		let s = store.clone();
		let _ = scheduler.run_on_main_thread(move || {
			maybe_record_clipboard_update(&handle_for_cb, &s);
		});
	});
}

fn build_panel_url() -> WebviewUrl {
	if tauri::is_dev() {
		return Url::parse("http://localhost:1420/")
			.map(WebviewUrl::External)
			.unwrap_or_else(|_| WebviewUrl::App("index.html".into()));
	}
	WebviewUrl::App("index.html".into())
}

pub fn show_cbhist_panel<R: Runtime>(app: &AppHandle<R>) {
	let Some(store) = app.try_state::<Arc<CbhistStore>>() else {
		return;
	};
	if store.disabled {
		return;
	}
	let lw = store.panel_width;
	let lh = store.panel_height;
	let tray_rect = app
		.tray_by_id("main")
		.and_then(|t| t.rect().ok().flatten());
	let pos = panel_position_for_tray(app, tray_rect, lw, lh);
	let url = build_panel_url();

	if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
		let _ = w.set_position(pos);
		let _ = w.set_size(LogicalSize::new(lw, lh));
		let _ = w.show();
		let _ = w.set_focus();
		return;
	}

	let app_h = app.clone();
	let _ = app.run_on_main_thread(move || {
		let win = match WebviewWindowBuilder::new(&app_h, WINDOW_LABEL, url)
			.title("Clipboard History")
			.decorations(false)
			.shadow(false)
			.transparent(true)
			.resizable(false)
			.minimizable(false)
			.maximizable(false)
			.skip_taskbar(true)
			.always_on_top(true)
			.inner_size(lw, lh)
			.visible(false)
			.build()
		{
			Ok(w) => w,
			Err(e) => {
				eprintln!("[cbhist] failed to build window: {}", e);
				return;
			}
		};
		let _ = win.set_position(pos);
		let _ = win.set_size(LogicalSize::new(lw, lh));
		let _ = win.show();
		let _ = win.set_focus();
	});
}

#[tauri::command]
pub fn cbhist_list(store: tauri::State<'_, Arc<CbhistStore>>) -> Result<Vec<CbhistItemDto>, String> {
	if store.disabled {
		return Ok(Vec::new());
	}
	let dq = store
		.deque
		.lock()
		.map_err(|_| "cbhist lock failed".to_string())?;
	let mut out = Vec::with_capacity(dq.len());
	for e in dq.iter() {
		out.push(dto_from_entry(e));
	}
	Ok(out)
}

#[tauri::command]
pub fn cbhist_apply(
	app: AppHandle,
	store: tauri::State<'_, Arc<CbhistStore>>,
	index: usize,
) -> Result<(), String> {
	if store.disabled {
		return Err("clipboard history is disabled".to_string());
	}
	let entry = {
		let dq = store
			.deque
			.lock()
			.map_err(|_| "cbhist lock failed".to_string())?;
		dq.get(index)
			.cloned()
			.ok_or_else(|| "invalid cbhist index".to_string())?
	};
	app
		.clipboard()
		.write_text(&entry.text)
		.map_err(|e| e.to_string())?;
	{
		let mut last = store
			.last_raw_clipboard
			.lock()
			.map_err(|_| "cbhist lock failed".to_string())?;
		*last = Some(entry.text.clone());
	}
	Ok(())
}

#[tauri::command]
pub fn cbhist_close_panel(app: AppHandle) -> Result<(), String> {
	if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
		w.hide().map_err(|e| e.to_string())?;
	}
	Ok(())
}
