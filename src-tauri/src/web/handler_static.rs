use super::dd_publish::{TEMP_DIR_PREFIX, resolve_temp_file, resolve_temp_share};
use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Response, StatusCode};
use urlencoding::{decode, encode};

#[path = "handler_static/ini.rs"]
mod ini;
#[path = "handler_static/json.rs"]
mod json;
#[path = "handler_static/md.rs"]
mod md;
#[path = "handler_static/structured_dispatcher.rs"]
mod structured_dispatcher;
#[path = "handler_static/structured_renderer.rs"]
mod structured_renderer;
#[path = "handler_static/toml.rs"]
mod toml;
#[path = "handler_static/yaml.rs"]
mod yaml;

use self::md::{create_markdown_response, is_markdown_file};
use self::structured_dispatcher::{create_structured_data_response, is_structured_data_file};
use super::common::{create_error_response, get_web_content_type};
use super::handler_dump::handle_dump_request;
use super::handler_editor::handle_editor_request;
use super::handler_resource_meta::{handle_resource_meta_request, is_resource_meta_request};
use super::handler_slow::handle_slow_request;
use super::handler_status::handle_status_request;
use crate::web_server::WebMarkdownHighlightConfig;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContentMode {
    Content,
    Raw,
    Source,
}

#[derive(Clone, Copy)]
enum ModeSwitchVariant {
    DirectoryRawSwitch,
    SourceView,
}

impl ContentMode {
    fn as_query_value(self) -> Option<&'static str> {
        match self {
            ContentMode::Content => Some("content"),
            ContentMode::Raw => Some("raw"),
            ContentMode::Source => Some("source"),
        }
    }

    fn as_label(self) -> &'static str {
        match self {
            ContentMode::Content => "Content",
            ContentMode::Raw => "Raw",
            ContentMode::Source => "Source",
        }
    }
}

const DIRECTORY_LISTING_TEMPLATE: &str = r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Index of __PAGE_TITLE__</title>
<style>
* { box-sizing: border-box; }
body { color: #aaa; background: #000; margin: 0; font-family: "Segoe UI", "Yu Gothic UI", "Meiryo", "Hiragino Kaku Gothic ProN", sans-serif; line-height: 1.6; }
#main { padding: 16px 24px; width: 100vw; overflow-wrap: anywhere; word-break: break-word; }
h1 { color: #ddd; font-size: 26px; margin: 0; }
.path { color: #666; margin: 10px 0 16px; }
#header { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
ul { list-style: none; margin: 0; padding: 0; border-top: 1px solid #222; }
li { border-bottom: 1px solid #161616; }
a { display: block; color: #ccc; text-decoration: none; padding: 8px 4px; border-radius: 2px; }
a:hover { color: #fff; background: #1a1a1a; }
.entry-label { display: inline-block; min-width: calc(1.7em - 2px); color: #666; }
.dir .entry-label { color: #777; }
.back .entry-label { color: #555; }
.no-link { display: block; color: #666; padding: 8px 4px; cursor: default; }
.empty { color: #666; font-style: italic; padding: 8px 4px; }
.error { color: #ff6b6b; padding: 8px 4px; }
.mode-switch { display: flex; gap: 6px; margin: 0; }
.mode-switch .mode-btn { display: inline-flex; align-items: center; justify-content: center; margin: 0; padding: 4px 8px; min-height: 24px; background: #333; color: #fff; border: 1px solid #555; border-radius: 2px; font-size: 11px; line-height: 1.2; text-decoration: none; }
.mode-switch .mode-btn:hover { background: #666; color: #fff; }
.mode-switch .mode-btn.is-active { background: #555; border-color: #777; }
.mode-switch.directory-switch .raw-switch { justify-content: space-between; gap: 8px; min-width: 72px; padding: 4px 7px; }
.mode-switch.directory-switch .raw-switch { background: transparent; border: none; box-shadow: none; }
.mode-switch.directory-switch .raw-switch:hover { background: transparent; border: none; }
.mode-switch.directory-switch .raw-switch .switch-label { font-size: 11px; line-height: 1; }
.mode-switch.directory-switch .raw-switch .switch-track { position: relative; width: 28px; height: 16px; border-radius: 999px; border: 1px solid #666; background: #222; transition: background-color .14s ease, border-color .14s ease; }
.mode-switch.directory-switch .raw-switch .switch-thumb { position: absolute; top: 1px; left: 1px; width: 12px; height: 12px; border-radius: 50%; background: #aaa; transition: transform .14s ease, background-color .14s ease; }
.mode-switch.directory-switch .raw-switch.is-active .switch-track { background: #0f6d3d; border-color: #2ea56a; }
.mode-switch.directory-switch .raw-switch.is-active .switch-thumb { transform: translateX(12px); background: #d6ffe8; }
#meta-tooltip { position: fixed; z-index: 9999; pointer-events: none; background: #101010; color: #ddd; border: 1px solid #333; border-radius: 4px; padding: 8px 10px; font-size: 12px; line-height: 1.4; box-shadow: 0 8px 24px rgba(0,0,0,0.45); width: min(840px, calc(100vw - 16px)); max-width: 840px; }
#meta-tooltip.hidden { display: none; }
#meta-tooltip table { border-collapse: collapse; border-spacing: 0; width: 100%; }
#meta-tooltip th { color: #777; text-align: left; font-weight: 400; padding: 0 8px 0 0; white-space: nowrap; width: 1%; }
#meta-tooltip td { padding: 0; }
#meta-tooltip .value { font-variant-numeric: tabular-nums; font-family: "Consolas", "Cascadia Code", "SFMono-Regular", "Menlo", "Monaco", "Courier New", monospace; white-space: nowrap; }
#meta-tooltip .value.preview { white-space: normal; overflow-wrap: anywhere; word-break: break-word; }
#meta-tooltip tr.preview-row th, #meta-tooltip tr.preview-row td { vertical-align: top; }
</style>
</head>
<body>
<div id="main">
<div id="header">
<h1>Index of __DISPLAY_PATH__</h1>
__MODE_SWITCH_HTML__
</div>
<div class="path">__DISPLAY_PATH__</div>
<ul>
__LIST_ITEMS__
</ul>
</div>
<div id="meta-tooltip" class="hidden"></div>
<script>
const metaEndpoint = "__METADATA_ENDPOINT__";
const tooltip = document.getElementById("meta-tooltip");
const metaCache = new Map();
let activeAnchor = null;
let hoverTimerId = null;
const HOVER_DELAY_MS = 750;
const MODE_STORAGE_KEY = "mclocks.web.content.mode";
const MODE_VALUES = new Set(["content", "raw", "source"]);
const normalizeMode = (value) => MODE_VALUES.has(value) ? value : "content";
const queryParams = new URLSearchParams(window.location.search);
const hasModeQuery = queryParams.has("mode");
const modeFromQuery = normalizeMode(queryParams.get("mode") || "content");
const modeFromStorage = normalizeMode(localStorage.getItem(MODE_STORAGE_KEY) || "content");
const currentMode = hasModeQuery ? modeFromQuery : modeFromStorage;
const directoryMode = currentMode === "raw" ? "raw" : "source";
localStorage.setItem(MODE_STORAGE_KEY, directoryMode);
document.querySelectorAll(".mode-switch [data-mode]").forEach((el) => {
	const activeMode = normalizeMode(el.getAttribute("data-active-mode") || (el.getAttribute("data-mode") || "content"));
	el.classList.toggle("is-active", activeMode === directoryMode);
	el.addEventListener("click", () => {
		const mode = normalizeMode(el.getAttribute("data-store-mode") || (el.getAttribute("data-mode") || "content"));
		localStorage.setItem(MODE_STORAGE_KEY, mode);
	});
});
const applyModeToHref = (href, mode) => {
	let resolved;
	try {
		resolved = new URL(href, window.location.origin);
	} catch (_) {
		return href;
	}
	if (mode === "content") {
		resolved.searchParams.delete("mode");
	} else {
		resolved.searchParams.set("mode", mode);
	}
	return `${resolved.pathname}${resolved.search}${resolved.hash}`;
};
document.querySelectorAll("a[data-entry-link]").forEach((a) => {
	const originalHref = a.getAttribute("href") || "";
	a.setAttribute("href", applyModeToHref(originalHref, directoryMode));
});
const escapeHtml = (s) => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&#x27;");
const pad2 = (n) => String(n).padStart(2, "0");
const toLocalTime = (value) => {
	if (value === null || value === undefined) {
		return "-";
	}
	const n = Number(value);
	if (!Number.isFinite(n)) {
		return "-";
	}
	const d = new Date(n);
	const y = d.getFullYear();
	const mo = pad2(d.getMonth() + 1);
	const da = pad2(d.getDate());
	const h = pad2(d.getHours());
	const mi = pad2(d.getMinutes());
	const s = pad2(d.getSeconds());
	return `${y}-${mo}-${da} ${h}:${mi}:${s}`;
};
const renderTooltip = (meta) => {
	const size = meta?.size_hr ?? "-";
	const preview = meta?.preview ?? "-";
	const modified = toLocalTime(meta?.modified_ms);
	const created = toLocalTime(meta?.created_ms);
	tooltip.innerHTML = `<table><tbody><tr><th scope="row">Size</th><td class="value">${escapeHtml(size)}</td></tr><tr><th scope="row">Modified</th><td class="value">${escapeHtml(modified)}</td></tr><tr><th scope="row">Created</th><td class="value">${escapeHtml(created)}</td></tr><tr class="preview-row"><th scope="row">Preview</th><td class="value preview">${escapeHtml(preview)}</td></tr></tbody></table>`;
};
const positionTooltip = (e) => {
	const pad = 14;
	const width = tooltip.offsetWidth || 220;
	const height = tooltip.offsetHeight || 72;
	let x = e.clientX + pad;
	let y = e.clientY + pad;
	if (x + width + 8 > window.innerWidth) {
		x = e.clientX - width - pad;
	}
	if (y + height + 8 > window.innerHeight) {
		y = e.clientY - height - pad;
	}
	tooltip.style.left = `${Math.max(8, x)}px`;
	tooltip.style.top = `${Math.max(8, y)}px`;
};
const clearHoverTimer = () => {
	if (hoverTimerId !== null) {
		clearTimeout(hoverTimerId);
		hoverTimerId = null;
	}
};
const hideTooltip = () => {
	clearHoverTimer();
	tooltip.classList.add("hidden");
	activeAnchor = null;
};
const loadMetadata = async (entryName) => {
	if (metaCache.has(entryName)) {
		return metaCache.get(entryName);
	}
	const res = await fetch(`${metaEndpoint}?path=${encodeURIComponent(entryName)}`, { headers: { "Accept": "application/json" } });
	if (!res.ok) {
		throw new Error(`HTTP ${res.status}`);
	}
	const data = await res.json();
	metaCache.set(entryName, data);
	return data;
};
document.querySelectorAll("a[data-meta-path]").forEach((a) => {
	a.addEventListener("mouseenter", (e) => {
		activeAnchor = a;
		clearHoverTimer();
		const mouseX = e.clientX;
		const mouseY = e.clientY;
		hoverTimerId = setTimeout(async () => {
			if (activeAnchor !== a) {
				return;
			}
			const entryName = a.getAttribute("data-meta-path") || "";
			try {
				const meta = await loadMetadata(entryName);
				if (activeAnchor !== a) {
					return;
				}
				renderTooltip(meta);
				tooltip.classList.remove("hidden");
				positionTooltip({ clientX: mouseX, clientY: mouseY });
			} catch (_) {
				if (activeAnchor !== a) {
					return;
				}
				tooltip.textContent = "Failed to load metadata";
				tooltip.classList.remove("hidden");
				positionTooltip({ clientX: mouseX, clientY: mouseY });
			} finally {
				hoverTimerId = null;
			}
		}, HOVER_DELAY_MS);
	});
	a.addEventListener("mousemove", (e) => {
		if (activeAnchor === a && !tooltip.classList.contains("hidden")) {
			positionTooltip(e);
		}
	});
	a.addEventListener("mouseleave", hideTooltip);
});
</script>
</body>
</html>
"##;

fn append_parent_entry(list_items: &mut String, parent_url: &str) {
    let _ = write!(
        list_items,
        "<li class=\"back\"><a href=\"{}\" data-entry-link=\"1\"><span class=\"entry-label\">↩️</span>. . /</a></li>\n",
        html_escape(parent_url)
    );
}

fn append_directory_entry(list_items: &mut String, dir_url: &str, dir_name: &str) {
    let _ = write!(
        list_items,
        "<li class=\"dir\"><a href=\"{}\" data-meta-path=\"{}\" data-entry-link=\"1\"><span class=\"entry-label\">📁</span>{}/</a></li>\n",
        html_escape(dir_url),
        html_escape(dir_name),
        html_escape(dir_name)
    );
}

fn append_directory_entry_no_link(list_items: &mut String, dir_name: &str) {
    let _ = write!(
        list_items,
        "<li class=\"dir\"><span class=\"no-link\"><span class=\"entry-label\">📁</span>{}/</span></li>\n",
        html_escape(dir_name)
    );
}

fn append_file_entry(list_items: &mut String, file_url: &str, file_name: &str) {
    let _ = write!(
        list_items,
        "<li class=\"file\"><a href=\"{}\" data-meta-path=\"{}\" data-entry-link=\"1\"><span class=\"entry-label\">📄</span>{}</a></li>\n",
        html_escape(file_url),
        html_escape(file_name),
        html_escape(file_name)
    );
}

fn append_file_entry_no_link(list_items: &mut String, file_name: &str) {
    let _ = write!(
        list_items,
        "<li class=\"file\"><span class=\"no-link\"><span class=\"entry-label\">📄</span>{}</span></li>\n",
        html_escape(file_name)
    );
}

fn is_tmpdir_root_listing(url_path: &str) -> bool {
    if !url_path.starts_with(TEMP_DIR_PREFIX) {
        return false;
    }
    let trimmed = url_path.trim_end_matches('/');
    let suffix = &trimmed[TEMP_DIR_PREFIX.len()..];
    if suffix.is_empty() {
        return false;
    }
    !suffix.contains('/')
}

fn parse_content_mode(url: &str) -> ContentMode {
    let query = match url.split('?').nth(1) {
        Some(q) => q.split('#').next().unwrap_or(q),
        None => return ContentMode::Content,
    };
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");
        if key != "mode" {
            continue;
        }
        return match value {
            "raw" => ContentMode::Raw,
            "source" => ContentMode::Source,
            "content" => ContentMode::Content,
            _ => ContentMode::Content,
        };
    }
    ContentMode::Content
}

fn split_url_path_and_query(url: &str) -> (&str, &str) {
    let no_fragment = url.split('#').next().unwrap_or(url);
    let mut parts = no_fragment.splitn(2, '?');
    let path = parts.next().unwrap_or("/");
    let query = parts.next().unwrap_or("");
    (path, query)
}

fn build_mode_href(path: &str, query: &str, target_mode: ContentMode) -> String {
    let mut kept_pairs: Vec<String> = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let key = pair.splitn(2, '=').next().unwrap_or("");
        if key == "mode" || key == "raw" {
            continue;
        }
        kept_pairs.push(pair.to_string());
    }
    if target_mode == ContentMode::Content {
        // Keep content mode URL clean as implicit default.
    } else if let Some(mode_value) = target_mode.as_query_value() {
        kept_pairs.push(format!("mode={}", mode_value));
    }
    if kept_pairs.is_empty() {
        path.to_string()
    } else {
        format!("{}?{}", path, kept_pairs.join("&"))
    }
}

fn build_mode_switch_html(
    path: &str,
    query: &str,
    current_mode: ContentMode,
    id: &str,
    variant: ModeSwitchVariant,
) -> String {
    let container_class = match variant {
        ModeSwitchVariant::DirectoryRawSwitch => "mode-switch directory-switch",
        ModeSwitchVariant::SourceView => "mode-switch",
    };
    let mut html = String::new();
    let _ = write!(
        html,
        "<div class=\"{}\" role=\"group\" aria-label=\"Display mode\">",
        container_class
    );
    match variant {
        ModeSwitchVariant::DirectoryRawSwitch => {
            let is_raw = current_mode == ContentMode::Raw;
            let target_mode = if is_raw {
                ContentMode::Source
            } else {
                ContentMode::Raw
            };
            let active_class = if is_raw { " is-active" } else { "" };
            let href = build_mode_href(path, query, target_mode);
            let store_mode = target_mode.as_query_value().unwrap_or("source");
            let _ = write!(
                html,
                "<a id=\"{}-raw\" class=\"header-action-btn mode-btn raw-switch{}\" href=\"{}\" data-mode=\"raw\" data-active-mode=\"raw\" data-store-mode=\"{}\" aria-pressed=\"{}\"><span class=\"switch-label\">Raw</span><span class=\"switch-track\" aria-hidden=\"true\"><span class=\"switch-thumb\"></span></span></a>",
                id,
                active_class,
                html_escape(&href),
                store_mode,
                if is_raw { "true" } else { "false" }
            );
        }
        ModeSwitchVariant::SourceView => {
            for mode in [ContentMode::Raw, ContentMode::Content] {
                let href = build_mode_href(path, query, mode);
                let active_class = if mode == current_mode {
                    " is-active"
                } else {
                    ""
                };
                let _ = write!(
                    html,
                    "<a id=\"{}-{}\" class=\"header-action-btn mode-btn{}\" href=\"{}\" data-mode=\"{}\">{}</a>",
                    id,
                    mode.as_query_value().unwrap_or("content"),
                    active_class,
                    html_escape(&href),
                    mode.as_query_value().unwrap_or("content"),
                    mode.as_label()
                );
            }
        }
    }
    html.push_str("</div>");
    html
}

fn create_directory_listing(
    dir_path: &Path,
    url_path: &str,
    url_query: &str,
    current_mode: ContentMode,
) -> Response<std::io::Cursor<Vec<u8>>> {
    // Decode URL path for display (each segment separately)
    let decoded_path = if url_path == "/" {
        "/".to_string()
    } else {
        let segments: Vec<String> = url_path
            .split('/')
            .map(|segment| {
                if segment.is_empty() {
                    String::new()
                } else {
                    match decode(segment) {
                        Ok(decoded) => decoded.into_owned(),
                        Err(_) => segment.to_string(),
                    }
                }
            })
            .collect();
        segments.join("/")
    };
    let metadata_endpoint = if url_path == "/" {
        "/.resource-meta".to_string()
    } else {
        format!("{}/.resource-meta", url_path.trim_end_matches('/'))
    };

    let mut list_items = String::new();
    let mut has_entries = false;

    // Add parent directory link if not at root
    if url_path != "/" && !is_tmpdir_root_listing(url_path) {
        let trimmed = url_path.trim_end_matches('/');
        let parent_url = if trimmed == "" {
            "/".to_string()
        } else {
            match trimmed.rfind('/') {
                Some(pos) => {
                    let parent = &trimmed[..pos];
                    if parent.is_empty() {
                        "/".to_string()
                    } else {
                        format!("{}/", parent)
                    }
                }
                None => "/".to_string(),
            }
        };
        append_parent_entry(&mut list_items, &parent_url);
        has_entries = true;
    }

    // Read directory entries
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut dirs: Vec<String> = Vec::new();
            let mut files: Vec<String> = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    if let Some(name) = file_name.to_str() {
                        let metadata = entry.metadata();
                        if let Ok(meta) = metadata {
                            if meta.is_dir() {
                                dirs.push(name.to_string());
                            } else {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }

            // Sort directories and files
            dirs.sort();
            files.sort();

            // Add directory entries
            for dir in dirs {
                let encoded_dir = encode(&dir);
                let dir_url = if url_path == "/" {
                    format!("/{}/", encoded_dir)
                } else {
                    let base = url_path.trim_end_matches('/');
                    format!("{}/{}/", base, encoded_dir)
                };
                if dir.starts_with('.') {
                    append_directory_entry_no_link(&mut list_items, &dir);
                } else {
                    append_directory_entry(&mut list_items, &dir_url, &dir);
                }
                has_entries = true;
            }

            // Add file entries
            for file in files {
                let encoded_file = encode(&file);
                let file_url = if url_path == "/" {
                    format!("/{}", encoded_file)
                } else {
                    let base = url_path.trim_end_matches('/');
                    format!("{}/{}", base, encoded_file)
                };
                if file.starts_with('.') {
                    append_file_entry_no_link(&mut list_items, &file);
                } else {
                    append_file_entry(&mut list_items, &file_url, &file);
                }
                has_entries = true;
            }
        }
        Err(_) => {
            list_items.push_str("<li class=\"error\">Error reading directory</li>\n");
        }
    }

    if !has_entries {
        list_items.push_str("<li class=\"empty\">(empty)</li>\n");
    }
    let mode_switch_html = build_mode_switch_html(
        url_path,
        url_query,
        current_mode,
        "directory-mode",
        ModeSwitchVariant::DirectoryRawSwitch,
    );
    let html = DIRECTORY_LISTING_TEMPLATE
        .replace("__PAGE_TITLE__", &html_escape(&decoded_path))
        .replace("__DISPLAY_PATH__", &html_escape(&decoded_path))
        .replace("__MODE_SWITCH_HTML__", &mode_switch_html)
        .replace("__LIST_ITEMS__", &list_items)
        .replace("__METADATA_ENDPOINT__", &html_escape(&metadata_endpoint));

    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}

fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn detect_encoding(content: &[u8]) -> &'static Encoding {
    // Check for BOM first (highest priority)
    if content.len() >= 3 && &content[0..3] == [0xEF, 0xBB, 0xBF] {
        return UTF_8;
    }
    if content.len() >= 2 && &content[0..2] == [0xFF, 0xFE] {
        return encoding_rs::UTF_16LE;
    }
    if content.len() >= 2 && &content[0..2] == [0xFE, 0xFF] {
        return encoding_rs::UTF_16BE;
    }

    // Try UTF-8 decoding first (fast path for common case)
    if std::str::from_utf8(content).is_ok() {
        return UTF_8;
    }

    // Use chardetng to detect encoding for non-UTF-8 content
    let mut detector = EncodingDetector::new();
    detector.feed(content, true);
    let encoding = detector.guess(None, true);

    // chardetng returns &'static Encoding directly
    encoding
}

fn create_file_response(
    file_path: &PathBuf,
    allow_html_in_md: bool,
    markdown_open_external_link_in_new_tab: bool,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    content_mode: ContentMode,
    url_path: &str,
    url_query: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    match fs::read(file_path) {
        Ok(content) => {
            let mode_switch_html = build_mode_switch_html(
                url_path,
                url_query,
                content_mode,
                "content-mode",
                ModeSwitchVariant::SourceView,
            );
            if is_markdown_file(file_path.as_path()) && content_mode == ContentMode::Source {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                return create_markdown_response(
                    file_path.as_path(),
                    &decoded,
                    content.len(),
                    allow_html_in_md,
                    markdown_open_external_link_in_new_tab,
                    markdown_highlight,
                    &mode_switch_html,
                );
            }
            if is_structured_data_file(file_path.as_path()) && content_mode == ContentMode::Source {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                return create_structured_data_response(
                    file_path.as_path(),
                    &decoded,
                    markdown_highlight,
                    &mode_switch_html,
                    content.len(),
                );
            }
            let base_content_type = if content_mode == ContentMode::Raw {
                "text/plain".to_string()
            } else {
                get_content_type(file_path)
            };
            let content_type = if is_text_type(&base_content_type) {
                let encoding = detect_encoding(&content);
                let charset = encoding.name();
                format!("{}; charset={}", base_content_type, charset)
            } else {
                base_content_type
            };
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
                Response::from_data(content)
                    .with_header(header)
                    .with_status_code(StatusCode(200))
            } else {
                Response::from_data(content).with_status_code(StatusCode(200))
            }
        }
        Err(_) => create_error_response(StatusCode(500), "Internal Server Error"),
    }
}

fn is_text_type(content_type: &str) -> bool {
    content_type.starts_with("text/")
        || content_type == "application/javascript"
        || content_type == "application/json"
        || content_type == "application/yaml"
        || content_type == "image/svg+xml"
}

pub fn get_content_type(path: &PathBuf) -> String {
    get_web_content_type(path.as_path()).to_string()
}

pub fn handle_web_request(
    request: &mut tiny_http::Request,
    root_path: &PathBuf,
    dump_enabled: bool,
    slow_enabled: bool,
    status_enabled: bool,
    allow_html_in_md: bool,
    markdown_open_external_link_in_new_tab: bool,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    editor_repos_dir: &Option<String>,
    editor_include_host: bool,
    editor_command: &str,
    editor_args: &[String],
) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    let (path, request_query) = split_url_path_and_query(url);
    let content_mode = parse_content_mode(url);
    if let Some(shared_file_path) = resolve_temp_file(path) {
        return create_file_response(
            &shared_file_path,
            allow_html_in_md,
            markdown_open_external_link_in_new_tab,
            markdown_highlight,
            content_mode,
            path,
            request_query,
        );
    }
    let (active_root_path, active_path, public_url_path) = match resolve_temp_share(path) {
        Some((temp_root, temp_relative_path, temp_public_prefix)) => {
            let public_path = if temp_relative_path == "/" {
                format!("{}/", temp_public_prefix)
            } else {
                format!("{}{}", temp_public_prefix, temp_relative_path)
            };
            (temp_root, temp_relative_path, public_path)
        }
        None => (root_path.clone(), path.to_string(), path.to_string()),
    };

    if is_resource_meta_request(active_path.as_str()) {
        return handle_resource_meta_request(url, &active_root_path, active_path.as_str());
    }

    // Check if this is a /editor request
    if editor_repos_dir.is_some() {
        if active_path == "/editor" || active_path.starts_with("/editor/") {
            return handle_editor_request(
                request,
                editor_repos_dir,
                editor_include_host,
                editor_command,
                editor_args,
            );
        }
    }

    // Check if this is a /status request (including /status/ and any subpaths)
    if status_enabled {
        if active_path.starts_with("/status/") {
            return handle_status_request(request, active_path.as_str());
        }
    }

    // Check if this is a /slow request (including /slow/ and any subpaths)
    if slow_enabled {
        if active_path == "/slow" || active_path.starts_with("/slow/") {
            return handle_slow_request(request);
        }
    }

    // Check if this is a /dump request (including /dump/ and any subpaths)
    if dump_enabled {
        if active_path == "/dump" || active_path.starts_with("/dump/") {
            return handle_dump_request(request);
        }
    }

    let url_path = active_path.as_str();

    // Security: Check for directory traversal attempts (pre-decode)
    if url_path.contains("..") || url_path.contains("//") {
        return create_error_response(StatusCode(400), "Bad Request");
    }

    // Determine the actual file path
    let file_path = if url_path == "/" {
        active_root_path.join("index.html")
    } else {
        let relative_path = url_path.trim_start_matches('/');
        if relative_path.starts_with('/') || (cfg!(windows) && relative_path.contains(':')) {
            return create_error_response(StatusCode(400), "Bad Request");
        }
        // Decode URL-encoded path components (each segment separately)
        let mut decoded_segments = Vec::new();
        for segment in relative_path.split('/') {
            match decode(segment) {
                Ok(decoded) => {
                    // Security: Reject traversal after URL decoding (%2e%2e bypass)
                    if decoded.contains("..") || decoded.starts_with('.') {
                        return create_error_response(StatusCode(400), "Bad Request");
                    }
                    decoded_segments.push(decoded.into_owned());
                }
                Err(_) => return create_error_response(StatusCode(400), "Bad Request"),
            }
        }
        active_root_path.join(decoded_segments.join("/"))
    };

    // Check if the path exists and is within root_path
    let normalized_root = match active_root_path.canonicalize() {
        Ok(p) => p,
        Err(_) => active_root_path.clone(),
    };

    let normalized_path = match file_path.canonicalize() {
        Ok(p) => {
            if !p.starts_with(&normalized_root) {
                return create_error_response(StatusCode(404), "Not Found");
            }
            p
        }
        Err(_) => {
            // canonicalize() failed, check if file_path exists
            // Special case: if url_path is "/", check for index.html first
            if url_path == "/" {
                let index_path = active_root_path.join("index.html");
                if index_path.exists() && index_path.is_file() {
                    return create_file_response(
                        &index_path,
                        allow_html_in_md,
                        markdown_open_external_link_in_new_tab,
                        markdown_highlight,
                        content_mode,
                        public_url_path.as_str(),
                        request_query,
                    );
                }
                // If index.html doesn't exist, show directory listing
                if active_root_path.exists() && active_root_path.is_dir() {
                    return create_directory_listing(
                        active_root_path.as_path(),
                        public_url_path.as_str(),
                        request_query,
                        content_mode,
                    );
                }
                return create_error_response(StatusCode(404), "Not Found");
            }
            // Check if it's a file
            if file_path.exists() && file_path.is_file() {
                // Security: Verify file is within root_path
                if !file_path.starts_with(active_root_path.as_path()) {
                    return create_error_response(StatusCode(404), "Not Found");
                }
                return create_file_response(
                    &file_path,
                    allow_html_in_md,
                    markdown_open_external_link_in_new_tab,
                    markdown_highlight,
                    content_mode,
                    public_url_path.as_str(),
                    request_query,
                );
            }
            // Check if it's a directory request
            if file_path.exists() && file_path.is_dir() {
                // Security: Verify directory is within root_path
                if !file_path.starts_with(active_root_path.as_path()) {
                    return create_error_response(StatusCode(404), "Not Found");
                }
                // Check for index.html in the directory
                let index_path = file_path.join("index.html");
                if index_path.exists() && index_path.is_file() {
                    return create_file_response(
                        &index_path,
                        allow_html_in_md,
                        markdown_open_external_link_in_new_tab,
                        markdown_highlight,
                        content_mode,
                        public_url_path.as_str(),
                        request_query,
                    );
                }
                // Generate directory listing
                return create_directory_listing(
                    &file_path,
                    public_url_path.as_str(),
                    request_query,
                    content_mode,
                );
            }
            return create_error_response(StatusCode(404), "Not Found");
        }
    };

    // If the normalized path is a directory, check for index.html
    if normalized_path.is_dir() {
        let index_path = normalized_path.join("index.html");
        if index_path.exists() {
            return create_file_response(
                &index_path,
                allow_html_in_md,
                markdown_open_external_link_in_new_tab,
                markdown_highlight,
                content_mode,
                public_url_path.as_str(),
                request_query,
            );
        }
        // Generate directory listing
        return create_directory_listing(
            &normalized_path,
            public_url_path.as_str(),
            request_query,
            content_mode,
        );
    }

    // It's a file, serve it
    create_file_response(
        &normalized_path,
        allow_html_in_md,
        markdown_open_external_link_in_new_tab,
        markdown_highlight,
        content_mode,
        public_url_path.as_str(),
        request_query,
    )
}
