use super::dd_publish::{
    TEMP_DIR_PREFIX, TEMP_FILE_PREFIX, register_temp_root, resolve_temp_file, resolve_temp_share,
};
use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, StatusCode};
use urlencoding::{decode, encode};

#[path = "handler_static_source/ini.rs"]
mod ini;
#[path = "handler_static_source/json.rs"]
mod json;
#[path = "handler_static_source/md.rs"]
mod md;
#[path = "handler_static_source/structured_dispatcher.rs"]
mod structured_dispatcher;
#[path = "handler_static_source/structured_renderer.rs"]
mod structured_renderer;
#[path = "handler_static_source/template_common.rs"]
mod template_common;
#[path = "handler_static_source/toml.rs"]
mod toml;
#[path = "handler_static_source/xml.rs"]
mod xml;
#[path = "handler_static_source/yaml.rs"]
mod yaml;

use self::md::{create_markdown_response, is_markdown_file};
use self::structured_dispatcher::{create_structured_data_response, is_structured_data_file};
use self::template_common::{ContentMode, ModeSwitchVariant};
use super::common::{create_error_response, format_display_path, get_web_content_type};
use super::handler_dump::handle_dump_request;
use super::handler_editor::handle_editor_request;
use super::handler_resource_meta::{handle_resource_meta_request, is_resource_meta_request};
use super::handler_slow::handle_slow_request;
use super::handler_status::handle_status_request;
use crate::web_server::WebMarkdownHighlightConfig;

const DIRECTORY_LISTING_TEMPLATE: &str = r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Index of __PAGE_TITLE__</title>
__MAIN_CSS_LINK__
<style>
* { box-sizing: border-box; }
body { color: #aaa; background: #000; margin: 0; font-family: "Segoe UI", "Yu Gothic UI", "Meiryo", "Hiragino Kaku Gothic ProN", sans-serif; line-height: 1.6; --sidebar-width: 200px; --left-pane-width: var(--sidebar-width); --left-pane-padding: 16px 12px; --left-pane-h2-margin: 0 0 8px 4px; }
#main { margin-left: var(--sidebar-width); width: calc(100vw - var(--sidebar-width)); padding: 16px 24px; overflow-wrap: anywhere; word-break: break-word; }
#header { display: flex; justify-content: space-between; align-items: center; gap: 8px; margin: 0 0 10px; }
#path-actions { display: flex; align-items: center; gap: 8px; min-width: 0; }
#directory-link { display: inline-flex; align-items: center; justify-content: center; color: #bbb; text-decoration: none; font-size: 14px; line-height: 1; }
#directory-link:hover { color: #fff; }
#main-header-path { flex: 0 1 auto; min-width: 0; line-height: 1.2; color: #777; font-size: 12px; font-family: "Consolas", "Cascadia Code", "SFMono-Regular", "Menlo", "Monaco", "Courier New", monospace; overflow-wrap: anywhere; word-break: break-word; }
#main-separator { height: 1px; background: #222; margin: 0 0 12px; }
#main > ul { list-style: none; margin: 0; padding: 0; border-top: none; }
#main > ul > li { border-bottom: 1px solid #161616; }
#main > ul > li > a { display: block; color: #ccc; text-decoration: none; padding: 8px 4px; border-radius: 2px; }
#main > ul > li > a:hover { color: #fff; background: #1a1a1a; }
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
#path-copy-btn { display: inline-flex; align-items: center; justify-content: center; margin: 0; padding: 3px 7px; min-height: 20px; background: #333; color: #fff; border: 1px solid #555; border-radius: 2px; cursor: pointer; font-size: 9px; line-height: 1.2; font-family: inherit; appearance: none; }
#path-copy-btn:hover { background: #666; color: #fff; }
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
<body class="mclocks-directory">
<aside id="sidebar">
<div id="sidebar-controls">__MODE_SWITCH_HTML__</div>
<h2>Summary</h2>
<ul id="summary-list">__SUMMARY_ITEMS__</ul>
</aside>
<div id="main">
<div id="header">
<div id="path-actions">
__DIRECTORY_LINK_HTML__
<div id="main-header-path">__ABSOLUTE_PATH__</div>
<button id="path-copy-btn" class="mode-btn" type="button">Copy</button>
</div>
</div>
<div id="main-separator"></div>
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
const pathCopyBtn = document.getElementById("path-copy-btn");
const pathLabel = document.getElementById("main-header-path");
if (pathCopyBtn && pathLabel) {
	pathCopyBtn.addEventListener("click", () => {
		navigator.clipboard.writeText(pathLabel.textContent || "");
		pathCopyBtn.textContent = "Copied!";
		pathCopyBtn.blur();
		setTimeout(() => {
			pathCopyBtn.textContent = "Copy";
			pathCopyBtn.blur();
		}, 2000);
	});
}
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
const summaryList = document.getElementById("summary-list");
if (summaryList) {
	summaryList.querySelectorAll("li").forEach((li) => {
		const label = li.querySelector(".label");
		const value = li.querySelector(".value");
		if (!label || !value) {
			return;
		}
		if (label.textContent?.trim() !== "Last Mod") {
			return;
		}
		value.textContent = toLocalTime(value.textContent?.trim() || "-");
	});
}
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

const SOURCE_VIEW_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
__MAIN_CSS_LINK__
__HIGHLIGHT_CSS_LINK__
<style>
body{line-height:1.6;min-height:100vh;--source-sidebar-width:200px;--left-pane-width:var(--source-sidebar-width);--left-pane-padding:16px 12px;--left-pane-bg:#050505;--left-pane-border-right:1px solid #1b1b1b;--left-pane-h2-margin:0 0 8px 4px;--sidebar-controls-gap:8px;--sidebar-actions-gap:6px}
#sidebar{max-width:45vw;overflow:auto}
#notices{margin-top:10px}
pre{margin:0;padding:12px;background:#111;border:1px solid #222;border-radius:4px;overflow-x:auto;overflow-y:hidden;overflow-wrap:normal;word-break:normal}
code{font-family:"Consolas","Cascadia Code","SFMono-Regular","Menlo","Monaco","Courier New",monospace;font-size:12px;line-height:1.5;white-space:pre;overflow-wrap:normal;word-break:normal}
</style>
</head>
<body class="mclocks-source">
<aside id="sidebar">
<div id="sidebar-controls">
<div id="sidebar-actions">
__MODE_SWITCH_HTML__
</div>
</div>
<h2>Summary</h2>
<ul id="summary-list">__SUMMARY_ITEMS__</ul>
<div id="notices"></div>
</aside>
<div id="main">
__COMMON_HEADER_HTML__
<div id="main-separator"></div>
<pre><code class="__LANGUAGE_CLASS__">__SOURCE_HTML__</code></pre>
</div>
__HIGHLIGHT_JS_SCRIPT__
__MAIN_JS_SCRIPT__
<script>
const pathCopyBtn = document.getElementById("path-copy-btn");
const pathLabel = document.getElementById("main-header-path");
if (pathCopyBtn && pathLabel) {
	pathCopyBtn.addEventListener("click", () => {
		navigator.clipboard.writeText(pathLabel.textContent || "");
		pathCopyBtn.textContent = "Copied!";
		pathCopyBtn.blur();
		setTimeout(() => {
			pathCopyBtn.textContent = "Copy";
			pathCopyBtn.blur();
		}, 2000);
	});
}
if (window.hljs) {
	document.querySelectorAll("pre code").forEach((code) => {
		window.hljs.highlightElement(code);
	});
}
const summaryList = document.getElementById("summary-list");
if (summaryList) {
	const pad2 = (n) => String(n).padStart(2, "0");
	const toLocalTime = (value) => {
		const n = Number(value);
		if (!Number.isFinite(n)) {
			return value;
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
	summaryList.querySelectorAll("li").forEach((li) => {
		const label = li.querySelector(".label");
		const value = li.querySelector(".value");
		if (!label || !value) {
			return;
		}
		if (label.textContent?.trim() !== "Last Mod") {
			return;
		}
		value.textContent = toLocalTime(value.textContent?.trim() || "-");
	});
}
const updateSummaryValue = (labelName, valueText) => {
	if (!summaryList) {
		return;
	}
	const target = Array.from(summaryList.querySelectorAll("li")).find((li) => {
		const label = li.querySelector(".label");
		return label && label.textContent?.trim() === labelName;
	});
	if (!target) {
		return;
	}
	const value = target.querySelector(".value");
	if (value) {
		value.textContent = valueText;
	}
};
const formatDuration = (seconds) => {
	if (!Number.isFinite(seconds) || seconds < 0) {
		return "-";
	}
	const total = Math.floor(seconds);
	const h = Math.floor(total / 3600);
	const m = Math.floor((total % 3600) / 60);
	const s = total % 60;
	const pad2 = (n) => String(n).padStart(2, "0");
	if (h > 0) {
		return `${h}:${pad2(m)}:${pad2(s)}`;
	}
	return `${m}:${pad2(s)}`;
};
const mediaImage = document.getElementById("source-media-image");
if (mediaImage) {
	const applyImageSize = () => {
		const w = Number(mediaImage.naturalWidth);
		const h = Number(mediaImage.naturalHeight);
		updateSummaryValue("Width", Number.isFinite(w) && w > 0 ? `${w}px` : "-");
		updateSummaryValue("Height", Number.isFinite(h) && h > 0 ? `${h}px` : "-");
	};
	mediaImage.addEventListener("load", applyImageSize);
	mediaImage.addEventListener("error", () => {
		updateSummaryValue("Width", "-");
		updateSummaryValue("Height", "-");
	});
	if (mediaImage.complete) {
		applyImageSize();
	}
}
const mediaAudio = document.getElementById("source-media-audio");
if (mediaAudio) {
	const applyDuration = () => {
		updateSummaryValue("Duration", formatDuration(mediaAudio.duration));
	};
	mediaAudio.addEventListener("loadedmetadata", applyDuration);
	mediaAudio.addEventListener("durationchange", applyDuration);
	mediaAudio.addEventListener("error", () => updateSummaryValue("Duration", "-"));
}
const mediaVideo = document.getElementById("source-media-video");
if (mediaVideo) {
	const applyDuration = () => {
		updateSummaryValue("Duration", formatDuration(mediaVideo.duration));
	};
	mediaVideo.addEventListener("loadedmetadata", applyDuration);
	mediaVideo.addEventListener("durationchange", applyDuration);
	mediaVideo.addEventListener("error", () => updateSummaryValue("Duration", "-"));
}
</script>
</body>
</html>
"##;

const SOURCE_MEDIA_VIEW_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
__MAIN_CSS_LINK__
<style>
body{line-height:1.6;min-height:100vh;--source-sidebar-width:200px;--left-pane-width:var(--source-sidebar-width);--left-pane-padding:16px 12px;--left-pane-bg:#050505;--left-pane-border-right:1px solid #1b1b1b;--left-pane-h2-margin:0 0 8px 4px;--sidebar-controls-gap:8px;--sidebar-actions-gap:6px}
#sidebar{max-width:45vw;overflow:auto}
#notices{margin-top:10px}
#source-media-wrap{margin:0;padding:12px;background:#111;border:1px solid #222;border-radius:4px}
#source-media-wrap img{display:block;max-width:100%;height:auto}
#source-media-wrap audio,#source-media-wrap video{display:block;max-width:100%;width:100%}
</style>
</head>
<body class="mclocks-source">
<aside id="sidebar">
<div id="sidebar-controls">
<div id="sidebar-actions">
__MODE_SWITCH_HTML__
</div>
</div>
<h2>Summary</h2>
<ul id="summary-list">__SUMMARY_ITEMS__</ul>
<div id="notices"></div>
</aside>
<div id="main">
__COMMON_HEADER_HTML__
<div id="main-separator"></div>
<div id="source-media-wrap">__MEDIA_HTML__</div>
</div>
__MAIN_JS_SCRIPT__
<script>
const pathCopyBtn = document.getElementById("path-copy-btn");
const pathLabel = document.getElementById("main-header-path");
if (pathCopyBtn && pathLabel) {
	pathCopyBtn.addEventListener("click", () => {
		navigator.clipboard.writeText(pathLabel.textContent || "");
		pathCopyBtn.textContent = "Copied!";
		pathCopyBtn.blur();
		setTimeout(() => {
			pathCopyBtn.textContent = "Copy";
			pathCopyBtn.blur();
		}, 2000);
	});
}
const summaryList = document.getElementById("summary-list");
if (summaryList) {
	const pad2 = (n) => String(n).padStart(2, "0");
	const toLocalTime = (value) => {
		const n = Number(value);
		if (!Number.isFinite(n)) {
			return value;
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
	summaryList.querySelectorAll("li").forEach((li) => {
		const label = li.querySelector(".label");
		const value = li.querySelector(".value");
		if (!label || !value) {
			return;
		}
		if (label.textContent?.trim() !== "Last Mod") {
			return;
		}
		value.textContent = toLocalTime(value.textContent?.trim() || "-");
	});
}
</script>
</body>
</html>
"##;

fn append_directory_entry(list_items: &mut String, dir_url: &str, dir_name: &str) {
    let _ = write!(
        list_items,
        "<li class=\"dir\"><a href=\"{}\" data-meta-path=\"{}\" data-entry-link=\"1\"><span class=\"entry-label\">📁</span>{}/</a></li>\n",
        html_escape(dir_url),
        html_escape(dir_name),
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

fn build_parent_directory_href(path: &str, query: &str, mode: ContentMode) -> String {
    let trimmed = path.trim_end_matches('/');
    let parent = match trimmed.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(pos) => format!("{}/", &trimmed[..pos]),
    };
    template_common::build_mode_href(&parent, query, mode)
}

fn resolve_source_parent_directory_href(
    url_path: &str,
    url_query: &str,
    mode: ContentMode,
    file_path: &Path,
) -> String {
    if url_path.starts_with(TEMP_FILE_PREFIX) {
        if let Some(parent) = file_path.parent() {
            if let Ok(hash) = register_temp_root(parent) {
                let temp_dir_path = format!("{}{}/", TEMP_DIR_PREFIX, hash);
                return template_common::build_mode_href(&temp_dir_path, url_query, mode);
            }
        }
    }
    build_parent_directory_href(url_path, url_query, mode)
}

fn resolve_directory_parent_directory_href(
    url_path: &str,
    url_query: &str,
    mode: ContentMode,
    dir_path: &Path,
) -> Option<String> {
    if url_path == "/" {
        return None;
    }
    if url_path.starts_with(TEMP_DIR_PREFIX) {
        if let Some(parent) = dir_path.parent() {
            if let Ok(hash) = register_temp_root(parent) {
                let temp_dir_path = format!("{}{}/", TEMP_DIR_PREFIX, hash);
                return Some(template_common::build_mode_href(
                    &temp_dir_path,
                    url_query,
                    mode,
                ));
            }
        }
    }
    Some(build_parent_directory_href(url_path, url_query, mode))
}

fn render_directory_summary_items(
    entry_count: usize,
    dir_count: usize,
    file_count: usize,
    last_modified_ms: Option<u64>,
    status: &str,
) -> String {
    let mut html = String::new();
    let fields = [
        ("Entries", entry_count.to_string()),
        ("Dirs", dir_count.to_string()),
        ("Files", file_count.to_string()),
        (
            "Last Mod",
            last_modified_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Status", status.to_string()),
    ];
    for (label, value) in fields {
        html.push_str("<li><span class=\"label\">");
        html.push_str(label);
        html.push_str("</span><span class=\"value\">");
        html.push_str(&html_escape(&value));
        html.push_str("</span></li>");
    }
    html
}

fn create_directory_listing(
    dir_path: &Path,
    url_path: &str,
    url_query: &str,
    current_mode: ContentMode,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
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
    let mut dir_count = 0usize;
    let mut file_count = 0usize;
    let mut summary_status = "Directory Listing";
    let mut has_entries = false;

    let parent_directory_href =
        resolve_directory_parent_directory_href(url_path, url_query, current_mode, dir_path);
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
            dir_count = dirs.len();
            file_count = files.len();

            // Add directory entries
            for dir in dirs {
                let encoded_dir = encode(&dir);
                let dir_url = if url_path == "/" {
                    format!("/{}/", encoded_dir)
                } else {
                    let base = url_path.trim_end_matches('/');
                    format!("{}/{}/", base, encoded_dir)
                };
                append_directory_entry(&mut list_items, &dir_url, &dir);
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
                append_file_entry(&mut list_items, &file_url, &file);
                has_entries = true;
            }
        }
        Err(_) => {
            list_items.push_str("<li class=\"error\">Error reading directory</li>\n");
            summary_status = "Error reading directory";
        }
    }

    if !has_entries {
        list_items.push_str("<li class=\"empty\">(empty)</li>\n");
    }
    let summary_items = render_directory_summary_items(
        dir_count + file_count,
        dir_count,
        file_count,
        get_last_modified_ms(dir_path),
        summary_status,
    );
    let absolute_path = format_display_path(dir_path);
    let mode_switch_html = template_common::build_mode_switch_html(
        url_path,
        url_query,
        current_mode,
        "directory-mode",
        ModeSwitchVariant::DirectoryRawSwitch,
    );
    let directory_link_html = match parent_directory_href.as_deref() {
        Some(href) => format!(
            "<a id=\"directory-link\" href=\"{}\" title=\"Open directory\">📁</a>",
            html_escape(href)
        ),
        None => "".to_string(),
    };
    let main_css_link = match markdown_highlight {
        Some(cfg) => format!(
            "<link rel=\"stylesheet\" href=\"{}\" />",
            html_escape(&cfg.main_css_url)
        ),
        None => "".to_string(),
    };
    let html = DIRECTORY_LISTING_TEMPLATE
        .replace("__PAGE_TITLE__", &html_escape(&decoded_path))
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace("__DISPLAY_PATH__", &html_escape(&decoded_path))
        .replace("__ABSOLUTE_PATH__", &html_escape(&absolute_path))
        .replace("__DIRECTORY_LINK_HTML__", &directory_link_html)
        .replace("__MODE_SWITCH_HTML__", &mode_switch_html)
        .replace("__SUMMARY_ITEMS__", &summary_items)
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

fn human_bytes(size: usize) -> String {
    if size < 1024 {
        return format!("{}B", size);
    }
    let kb = size as f64 / 1024.0;
    if kb < 1024.0 {
        return format!("{:.2}KB", kb);
    }
    let mb = kb / 1024.0;
    if mb < 1024.0 {
        return format!("{:.2}MB", mb);
    }
    let gb = mb / 1024.0;
    format!("{:.2}GB", gb)
}

fn system_time_to_unix_ms(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

fn get_last_modified_ms(file_path: &Path) -> Option<u64> {
    fs::metadata(file_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(system_time_to_unix_ms)
}

fn render_source_summary_items(
    size_bytes: usize,
    last_modified_ms: Option<u64>,
    status: &str,
) -> String {
    let mut html = String::new();
    let fields = [
        ("Raw Size", human_bytes(size_bytes)),
        (
            "Last Mod",
            last_modified_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Status", status.to_string()),
    ];
    for (label, value) in fields {
        html.push_str("<li><span class=\"label\">");
        html.push_str(label);
        html.push_str("</span><span class=\"value\">");
        html.push_str(&html_escape(&value));
        html.push_str("</span></li>");
    }
    html
}

fn is_probably_binary(content: &[u8]) -> bool {
    if content.is_empty() {
        return false;
    }
    if content.contains(&0) {
        return true;
    }
    let mut suspicious = 0usize;
    let sample_len = content.len().min(8192);
    for b in &content[..sample_len] {
        let is_text_char = matches!(*b, 0x09 | 0x0A | 0x0D | 0x20..=0x7E);
        if !is_text_char {
            suspicious += 1;
        }
    }
    suspicious * 10 > sample_len * 3
}

fn sanitize_language_class(file_path: &Path) -> String {
    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext.is_empty() {
        return "language-plaintext".to_string();
    }
    if ext
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return format!("language-{}", ext);
    }
    "language-plaintext".to_string()
}

fn create_source_text_response(
    file_path: &Path,
    source: &str,
    source_size_bytes: usize,
    parent_directory_href: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    mode_switch_html: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "Source".to_string());
    let absolute_path = format_display_path(file_path);
    let language_class = sanitize_language_class(file_path);
    let summary_items = render_source_summary_items(
        source_size_bytes,
        get_last_modified_ms(file_path),
        "Syntax Highlight",
    );
    let (main_css_link, main_js_script, highlight_css_link, highlight_js_script) =
        match markdown_highlight {
            Some(cfg) => (
                format!(
                    "<link rel=\"stylesheet\" href=\"{}\" />",
                    html_escape(&cfg.main_css_url)
                ),
                format!(
                    "<script src=\"{}\"></script>",
                    html_escape(&cfg.main_js_url)
                ),
                format!(
                    "<link rel=\"stylesheet\" href=\"{}\" />",
                    html_escape(&cfg.css_url)
                ),
                format!("<script src=\"{}\"></script>", html_escape(&cfg.js_url)),
            ),
            None => (
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ),
        };
    let html = SOURCE_VIEW_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace("__HIGHLIGHT_CSS_LINK__", &highlight_css_link)
        .replace("__MAIN_JS_SCRIPT__", &main_js_script)
        .replace("__HIGHLIGHT_JS_SCRIPT__", &highlight_js_script)
        .replace(
            "__COMMON_HEADER_HTML__",
            &template_common::render_main_header_html(
                &absolute_path,
                Some(parent_directory_href),
                None,
            ),
        )
        .replace("__MODE_SWITCH_HTML__", mode_switch_html)
        .replace("__SUMMARY_ITEMS__", &summary_items)
        .replace("__LANGUAGE_CLASS__", &language_class)
        .replace("__SOURCE_HTML__", &html_escape(source));
    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}

#[derive(Clone, Copy)]
enum SourceMediaKind {
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageWebp,
    ImageBmp,
    AudioMp3,
    AudioM4a,
    AudioWav,
    AudioOgg,
    AudioFlac,
    AudioAac,
    VideoMp4,
    VideoM4v,
    VideoMov,
    VideoWebm,
    VideoOgv,
}

fn is_png_content(content: &[u8]) -> bool {
    let sig = b"\x89PNG\r\n\x1a\n";
    content.len() >= sig.len() && &content[..sig.len()] == sig
}

fn is_mp3_content(content: &[u8]) -> bool {
    if content.len() >= 3 && &content[..3] == b"ID3" {
        return true;
    }
    content.len() >= 2 && content[0] == 0xFF && (content[1] & 0xE0) == 0xE0
}

fn is_mp4_content(content: &[u8]) -> bool {
    content.len() >= 12 && &content[4..8] == b"ftyp"
}

fn is_jpeg_content(content: &[u8]) -> bool {
    content.len() >= 3 && content[0] == 0xFF && content[1] == 0xD8 && content[2] == 0xFF
}

fn is_gif_content(content: &[u8]) -> bool {
    content.len() >= 6 && (&content[..6] == b"GIF87a" || &content[..6] == b"GIF89a")
}

fn is_webp_content(content: &[u8]) -> bool {
    content.len() >= 12 && &content[..4] == b"RIFF" && &content[8..12] == b"WEBP"
}

fn is_bmp_content(content: &[u8]) -> bool {
    content.len() >= 26 && &content[..2] == b"BM"
}

fn is_webm_content(content: &[u8]) -> bool {
    if content.len() < 4 {
        return false;
    }
    if content[0] != 0x1A || content[1] != 0x45 || content[2] != 0xDF || content[3] != 0xA3 {
        return false;
    }
    let window_len = content.len().min(512);
    let window = &content[..window_len];
    window.windows(4).any(|w| w.eq_ignore_ascii_case(b"webm"))
}

fn is_wav_content(content: &[u8]) -> bool {
    content.len() >= 12 && &content[..4] == b"RIFF" && &content[8..12] == b"WAVE"
}

fn is_ogg_content(content: &[u8]) -> bool {
    content.len() >= 4 && &content[..4] == b"OggS"
}

fn is_flac_content(content: &[u8]) -> bool {
    content.len() >= 4 && &content[..4] == b"fLaC"
}

fn is_aac_adts_content(content: &[u8]) -> bool {
    content.len() >= 2 && content[0] == 0xFF && (content[1] & 0xF0) == 0xF0
}

fn parse_png_dimensions(content: &[u8]) -> Option<(u32, u32)> {
    if content.len() < 24 {
        return None;
    }
    let sig = b"\x89PNG\r\n\x1a\n";
    if &content[..8] != sig {
        return None;
    }
    if &content[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([content[16], content[17], content[18], content[19]]);
    let height = u32::from_be_bytes([content[20], content[21], content[22], content[23]]);
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn parse_jpeg_dimensions(content: &[u8]) -> Option<(u32, u32)> {
    if !is_jpeg_content(content) {
        return None;
    }
    let mut pos = 2usize;
    while pos + 3 < content.len() {
        while pos < content.len() && content[pos] != 0xFF {
            pos += 1;
        }
        if pos + 1 >= content.len() {
            return None;
        }
        while pos + 1 < content.len() && content[pos + 1] == 0xFF {
            pos += 1;
        }
        if pos + 1 >= content.len() {
            return None;
        }
        let marker = content[pos + 1];
        pos += 2;
        if marker == 0xD8 || marker == 0xD9 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            continue;
        }
        if pos + 1 >= content.len() {
            return None;
        }
        let segment_len = usize::from(u16::from_be_bytes([content[pos], content[pos + 1]]));
        if segment_len < 2 || pos + segment_len > content.len() {
            return None;
        }
        let is_sof = matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        );
        if is_sof {
            if segment_len < 7 {
                return None;
            }
            let base = pos + 2;
            let height = u32::from(u16::from_be_bytes([content[base + 1], content[base + 2]]));
            let width = u32::from(u16::from_be_bytes([content[base + 3], content[base + 4]]));
            if width == 0 || height == 0 {
                return None;
            }
            return Some((width, height));
        }
        pos += segment_len;
    }
    None
}

fn parse_gif_dimensions(content: &[u8]) -> Option<(u32, u32)> {
    if !is_gif_content(content) || content.len() < 10 {
        return None;
    }
    let width = u32::from(u16::from_le_bytes([content[6], content[7]]));
    let height = u32::from(u16::from_le_bytes([content[8], content[9]]));
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn parse_webp_dimensions(content: &[u8]) -> Option<(u32, u32)> {
    if !is_webp_content(content) {
        return None;
    }
    let mut pos = 12usize;
    while pos + 8 <= content.len() {
        let chunk_type = &content[pos..pos + 4];
        let chunk_size = usize::try_from(u32::from_le_bytes([
            content[pos + 4],
            content[pos + 5],
            content[pos + 6],
            content[pos + 7],
        ]))
        .ok()?;
        let data_start = pos + 8;
        let data_end = data_start.checked_add(chunk_size)?;
        if data_end > content.len() {
            return None;
        }
        if chunk_type == b"VP8X" && chunk_size >= 10 {
            let width_minus_one = u32::from(content[data_start + 4])
                | (u32::from(content[data_start + 5]) << 8)
                | (u32::from(content[data_start + 6]) << 16);
            let height_minus_one = u32::from(content[data_start + 7])
                | (u32::from(content[data_start + 8]) << 8)
                | (u32::from(content[data_start + 9]) << 16);
            let width = width_minus_one.saturating_add(1);
            let height = height_minus_one.saturating_add(1);
            if width > 0 && height > 0 {
                return Some((width, height));
            }
            return None;
        }
        if chunk_type == b"VP8 " && chunk_size >= 10 {
            let payload = &content[data_start..data_end];
            if payload[3] == 0x9D && payload[4] == 0x01 && payload[5] == 0x2A {
                let width = u32::from(u16::from_le_bytes([payload[6], payload[7]]) & 0x3FFF);
                let height = u32::from(u16::from_le_bytes([payload[8], payload[9]]) & 0x3FFF);
                if width > 0 && height > 0 {
                    return Some((width, height));
                }
                return None;
            }
        }
        if chunk_type == b"VP8L" && chunk_size >= 5 {
            let payload = &content[data_start..data_end];
            if payload[0] == 0x2F {
                let bits = u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]);
                let width = (bits & 0x3FFF).saturating_add(1);
                let height = ((bits >> 14) & 0x3FFF).saturating_add(1);
                if width > 0 && height > 0 {
                    return Some((width, height));
                }
                return None;
            }
        }
        let padded = chunk_size + (chunk_size % 2);
        pos = data_start.checked_add(padded)?;
    }
    None
}

fn parse_bmp_dimensions(content: &[u8]) -> Option<(u32, u32)> {
    if !is_bmp_content(content) || content.len() < 26 {
        return None;
    }
    let dib_size = u32::from_le_bytes([content[14], content[15], content[16], content[17]]);
    if dib_size == 12 {
        if content.len() < 26 {
            return None;
        }
        let width = u32::from(u16::from_le_bytes([content[18], content[19]]));
        let height = u32::from(u16::from_le_bytes([content[20], content[21]]));
        if width == 0 || height == 0 {
            return None;
        }
        return Some((width, height));
    }
    if dib_size >= 40 {
        if content.len() < 26 {
            return None;
        }
        let width_i32 = i32::from_le_bytes([content[18], content[19], content[20], content[21]]);
        let height_i32 = i32::from_le_bytes([content[22], content[23], content[24], content[25]]);
        let width = width_i32.unsigned_abs();
        let height = height_i32.unsigned_abs();
        if width == 0 || height == 0 {
            return None;
        }
        return Some((width, height));
    }
    None
}

fn parse_wav_duration_seconds(content: &[u8]) -> Option<f64> {
    if !is_wav_content(content) {
        return None;
    }
    let mut pos = 12usize;
    let mut byte_rate: Option<u32> = None;
    let mut data_size: Option<u32> = None;
    while pos + 8 <= content.len() {
        let chunk_id = &content[pos..pos + 4];
        let chunk_size = usize::try_from(u32::from_le_bytes([
            content[pos + 4],
            content[pos + 5],
            content[pos + 6],
            content[pos + 7],
        ]))
        .ok()?;
        let chunk_start = pos + 8;
        let chunk_end = chunk_start.checked_add(chunk_size)?;
        if chunk_end > content.len() {
            return None;
        }
        if chunk_id == b"fmt " && chunk_size >= 16 {
            byte_rate = Some(u32::from_le_bytes([
                content[chunk_start + 8],
                content[chunk_start + 9],
                content[chunk_start + 10],
                content[chunk_start + 11],
            ]));
        } else if chunk_id == b"data" {
            data_size = Some(u32::try_from(chunk_size).ok()?);
        }
        let padded = chunk_size + (chunk_size % 2);
        pos = chunk_start.checked_add(padded)?;
    }
    let br = byte_rate?;
    let ds = data_size?;
    if br == 0 || ds == 0 {
        return None;
    }
    Some((ds as f64) / (br as f64))
}

fn parse_ogg_duration_seconds(content: &[u8]) -> Option<f64> {
    if !is_ogg_content(content) {
        return None;
    }
    let mut pos = 0usize;
    let mut sample_rate: Option<u32> = None;
    let mut last_granule: Option<u64> = None;
    while pos + 27 <= content.len() {
        if &content[pos..pos + 4] != b"OggS" {
            return None;
        }
        let page_segments = usize::from(content[pos + 26]);
        if pos + 27 + page_segments > content.len() {
            return None;
        }
        let seg_table_start = pos + 27;
        let payload_size: usize = content[seg_table_start..seg_table_start + page_segments]
            .iter()
            .map(|v| usize::from(*v))
            .sum();
        let payload_start = seg_table_start + page_segments;
        let payload_end = payload_start.checked_add(payload_size)?;
        if payload_end > content.len() {
            return None;
        }
        let granule = u64::from_le_bytes([
            content[pos + 6],
            content[pos + 7],
            content[pos + 8],
            content[pos + 9],
            content[pos + 10],
            content[pos + 11],
            content[pos + 12],
            content[pos + 13],
        ]);
        if granule != u64::MAX {
            last_granule = Some(granule);
        }
        if sample_rate.is_none() && payload_size >= 16 {
            let payload = &content[payload_start..payload_end];
            if payload.len() >= 19 && &payload[..8] == b"OpusHead" {
                sample_rate = Some(u32::from_le_bytes([
                    payload[12],
                    payload[13],
                    payload[14],
                    payload[15],
                ]));
            } else if payload.len() >= 16 && payload[0] == 0x01 && &payload[1..7] == b"vorbis" {
                sample_rate = Some(u32::from_le_bytes([
                    payload[12],
                    payload[13],
                    payload[14],
                    payload[15],
                ]));
            }
        }
        pos = payload_end;
    }
    let sr = sample_rate?;
    let lg = last_granule?;
    if sr == 0 {
        return None;
    }
    Some((lg as f64) / (sr as f64))
}

fn parse_flac_duration_seconds(content: &[u8]) -> Option<f64> {
    if !is_flac_content(content) || content.len() < 42 {
        return None;
    }
    let mut pos = 4usize;
    while pos + 4 <= content.len() {
        let header = content[pos];
        let is_last = (header & 0x80) != 0;
        let block_type = header & 0x7F;
        let block_len = (u32::from(content[pos + 1]) << 16)
            | (u32::from(content[pos + 2]) << 8)
            | u32::from(content[pos + 3]);
        let block_len_usize = usize::try_from(block_len).ok()?;
        let block_start = pos + 4;
        let block_end = block_start.checked_add(block_len_usize)?;
        if block_end > content.len() {
            return None;
        }
        if block_type == 0 && block_len_usize >= 34 {
            let p = &content[block_start..block_end];
            let sample_rate =
                (u32::from(p[10]) << 12) | (u32::from(p[11]) << 4) | (u32::from(p[12]) >> 4);
            let total_samples = (u64::from(p[13] & 0x0F) << 32)
                | (u64::from(p[14]) << 24)
                | (u64::from(p[15]) << 16)
                | (u64::from(p[16]) << 8)
                | u64::from(p[17]);
            if sample_rate == 0 || total_samples == 0 {
                return None;
            }
            return Some((total_samples as f64) / (sample_rate as f64));
        }
        pos = block_end;
        if is_last {
            break;
        }
    }
    None
}

fn parse_aac_adts_duration_seconds(content: &[u8]) -> Option<f64> {
    if !is_aac_adts_content(content) {
        return None;
    }
    let sample_rate_table = [
        96000u32, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350,
        0, 0, 0,
    ];
    let mut pos = 0usize;
    let mut sample_rate: Option<u32> = None;
    let mut frames = 0u64;
    while pos + 7 <= content.len() {
        if content[pos] != 0xFF || (content[pos + 1] & 0xF0) != 0xF0 {
            break;
        }
        let sr_index = usize::from((content[pos + 2] & 0x3C) >> 2);
        if sr_index >= sample_rate_table.len() || sample_rate_table[sr_index] == 0 {
            return None;
        }
        if sample_rate.is_none() {
            sample_rate = Some(sample_rate_table[sr_index]);
        }
        let frame_len = (usize::from(content[pos + 3] & 0x03) << 11)
            | (usize::from(content[pos + 4]) << 3)
            | usize::from((content[pos + 5] & 0xE0) >> 5);
        if frame_len < 7 {
            return None;
        }
        let next = pos.checked_add(frame_len)?;
        if next > content.len() {
            break;
        }
        frames += 1;
        pos = next;
    }
    let sr = sample_rate?;
    if frames == 0 || sr == 0 {
        return None;
    }
    Some((frames as f64) * 1024.0 / (sr as f64))
}

fn parse_image_dimensions(kind: SourceMediaKind, content: &[u8]) -> Option<(u32, u32)> {
    match kind {
        SourceMediaKind::ImagePng => parse_png_dimensions(content),
        SourceMediaKind::ImageJpeg => parse_jpeg_dimensions(content),
        SourceMediaKind::ImageGif => parse_gif_dimensions(content),
        SourceMediaKind::ImageWebp => parse_webp_dimensions(content),
        SourceMediaKind::ImageBmp => parse_bmp_dimensions(content),
        _ => None,
    }
}

fn id3v2_tag_size(content: &[u8]) -> usize {
    if content.len() < 10 || &content[..3] != b"ID3" {
        return 0;
    }
    let b6 = usize::from(content[6] & 0x7F);
    let b7 = usize::from(content[7] & 0x7F);
    let b8 = usize::from(content[8] & 0x7F);
    let b9 = usize::from(content[9] & 0x7F);
    10 + (b6 << 21) + (b7 << 14) + (b8 << 7) + b9
}

fn parse_mp3_bitrate_kbps(content: &[u8]) -> Option<u32> {
    let start = id3v2_tag_size(content);
    if content.len() < start + 4 {
        return None;
    }
    let h = u32::from_be_bytes([
        content[start],
        content[start + 1],
        content[start + 2],
        content[start + 3],
    ]);
    if (h >> 21) & 0x7FF != 0x7FF {
        return None;
    }
    let version_id = (h >> 19) & 0x3;
    let layer = (h >> 17) & 0x3;
    if layer != 0x1 {
        return None;
    }
    let bitrate_index = ((h >> 12) & 0xF) as usize;
    if bitrate_index == 0 || bitrate_index == 0xF {
        return None;
    }
    let bitrate = match version_id {
        0x3 => [
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
        ][bitrate_index],
        0x2 | 0x0 => [
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0,
        ][bitrate_index],
        _ => 0,
    };
    if bitrate == 0 { None } else { Some(bitrate) }
}

fn estimate_mp3_duration_seconds(content: &[u8]) -> Option<f64> {
    let bitrate_kbps = parse_mp3_bitrate_kbps(content)?;
    let mut audio_bytes = content.len().saturating_sub(id3v2_tag_size(content));
    if audio_bytes >= 128 && &content[content.len() - 128..content.len() - 125] == b"TAG" {
        audio_bytes = audio_bytes.saturating_sub(128);
    }
    if audio_bytes == 0 {
        return None;
    }
    let bits = (audio_bytes as f64) * 8.0;
    let bps = (bitrate_kbps as f64) * 1000.0;
    Some(bits / bps)
}

fn parse_mp4_duration_seconds(content: &[u8]) -> Option<f64> {
    fn parse_boxes(data: &[u8], start: usize, end: usize) -> Option<(u32, u64)> {
        let mut pos = start;
        while pos + 8 <= end {
            let size32 =
                u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            let box_type = &data[pos + 4..pos + 8];
            let mut header_len = 8usize;
            let box_end = if size32 == 0 {
                end
            } else if size32 == 1 {
                if pos + 16 > end {
                    return None;
                }
                header_len = 16;
                let large_size = u64::from_be_bytes([
                    data[pos + 8],
                    data[pos + 9],
                    data[pos + 10],
                    data[pos + 11],
                    data[pos + 12],
                    data[pos + 13],
                    data[pos + 14],
                    data[pos + 15],
                ]);
                let large_size_usize = usize::try_from(large_size).ok()?;
                pos.checked_add(large_size_usize)?
            } else {
                pos.checked_add(size32 as usize)?
            };
            if box_end > end || box_end < pos + header_len {
                return None;
            }
            if box_type == b"moov" {
                if let Some(found) = parse_boxes(data, pos + header_len, box_end) {
                    return Some(found);
                }
            } else if box_type == b"mvhd" {
                if box_end < pos + header_len + 4 {
                    return None;
                }
                let payload = &data[pos + header_len..box_end];
                let version = payload[0];
                if version == 0 {
                    if payload.len() < 20 {
                        return None;
                    }
                    let timescale =
                        u32::from_be_bytes([payload[12], payload[13], payload[14], payload[15]]);
                    let duration =
                        u32::from_be_bytes([payload[16], payload[17], payload[18], payload[19]])
                            as u64;
                    return Some((timescale, duration));
                }
                if version == 1 {
                    if payload.len() < 32 {
                        return None;
                    }
                    let timescale =
                        u32::from_be_bytes([payload[20], payload[21], payload[22], payload[23]]);
                    let duration = u64::from_be_bytes([
                        payload[24],
                        payload[25],
                        payload[26],
                        payload[27],
                        payload[28],
                        payload[29],
                        payload[30],
                        payload[31],
                    ]);
                    return Some((timescale, duration));
                }
            }
            pos = box_end;
        }
        None
    }
    let (timescale, duration) = parse_boxes(content, 0, content.len())?;
    if timescale == 0 || duration == 0 {
        return None;
    }
    Some((duration as f64) / (timescale as f64))
}

fn format_duration_text(seconds: f64) -> Option<String> {
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    let total = seconds.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        Some(format!("{}:{:02}:{:02}", h, m, s))
    } else {
        Some(format!("{}:{:02}", m, s))
    }
}

fn detect_source_media_kind(
    file_path: &Path,
    content_type: &str,
    content: &[u8],
    is_binary_content: bool,
) -> Option<SourceMediaKind> {
    if !is_binary_content {
        return None;
    }
    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "png" && (content_type == "image/png" || content_type == "application/octet-stream") {
        if is_png_content(content) {
            return Some(SourceMediaKind::ImagePng);
        }
    }
    if (ext == "jpg" || ext == "jpeg")
        && (content_type == "image/jpeg" || content_type == "application/octet-stream")
        && is_jpeg_content(content)
    {
        return Some(SourceMediaKind::ImageJpeg);
    }
    if ext == "gif" && (content_type == "image/gif" || content_type == "application/octet-stream") {
        if is_gif_content(content) {
            return Some(SourceMediaKind::ImageGif);
        }
    }
    if ext == "webp"
        && (content_type == "image/webp" || content_type == "application/octet-stream")
        && is_webp_content(content)
    {
        return Some(SourceMediaKind::ImageWebp);
    }
    if ext == "bmp"
        && (content_type == "image/bmp" || content_type == "application/octet-stream")
        && is_bmp_content(content)
    {
        return Some(SourceMediaKind::ImageBmp);
    }
    if ext == "m4a"
        && (content_type == "audio/mp4"
            || content_type == "audio/m4a"
            || content_type == "audio/x-m4a"
            || content_type == "application/octet-stream")
        && is_mp4_content(content)
    {
        return Some(SourceMediaKind::AudioM4a);
    }
    if ext == "wav"
        && (content_type == "audio/wav"
            || content_type == "audio/x-wav"
            || content_type == "application/octet-stream")
        && is_wav_content(content)
    {
        return Some(SourceMediaKind::AudioWav);
    }
    if ext == "ogg"
        && (content_type == "audio/ogg" || content_type == "application/octet-stream")
        && is_ogg_content(content)
    {
        return Some(SourceMediaKind::AudioOgg);
    }
    if ext == "flac"
        && (content_type == "audio/flac"
            || content_type == "audio/x-flac"
            || content_type == "application/octet-stream")
        && is_flac_content(content)
    {
        return Some(SourceMediaKind::AudioFlac);
    }
    if ext == "aac"
        && (content_type == "audio/aac"
            || content_type == "audio/x-aac"
            || content_type == "application/octet-stream")
        && is_aac_adts_content(content)
    {
        return Some(SourceMediaKind::AudioAac);
    }
    if ext == "mp3" && (content_type == "audio/mpeg" || content_type == "application/octet-stream")
    {
        if is_mp3_content(content) {
            return Some(SourceMediaKind::AudioMp3);
        }
    }
    if ext == "mp4" && (content_type == "video/mp4" || content_type == "application/octet-stream") {
        if is_mp4_content(content) {
            return Some(SourceMediaKind::VideoMp4);
        }
    }
    if ext == "m4v" && (content_type == "video/mp4" || content_type == "application/octet-stream") {
        if is_mp4_content(content) {
            return Some(SourceMediaKind::VideoM4v);
        }
    }
    if ext == "m4v" && content_type == "video/x-m4v" && is_mp4_content(content) {
        return Some(SourceMediaKind::VideoM4v);
    }
    if ext == "mov"
        && (content_type == "video/quicktime"
            || content_type == "video/mp4"
            || content_type == "application/octet-stream")
    {
        if is_mp4_content(content) {
            return Some(SourceMediaKind::VideoMov);
        }
    }
    if ext == "webm"
        && (content_type == "video/webm" || content_type == "application/octet-stream")
        && is_webm_content(content)
    {
        return Some(SourceMediaKind::VideoWebm);
    }
    if ext == "ogv"
        && (content_type == "video/ogg" || content_type == "application/octet-stream")
        && is_ogg_content(content)
    {
        return Some(SourceMediaKind::VideoOgv);
    }
    None
}

fn render_source_media_html(
    media_kind: SourceMediaKind,
    raw_href: &str,
    file_name: &str,
) -> String {
    match media_kind {
        SourceMediaKind::ImagePng
        | SourceMediaKind::ImageJpeg
        | SourceMediaKind::ImageGif
        | SourceMediaKind::ImageWebp
        | SourceMediaKind::ImageBmp => {
            format!(
                "<img id=\"source-media-image\" src=\"{}\" alt=\"{}\" />",
                html_escape(raw_href),
                html_escape(file_name)
            )
        }
        SourceMediaKind::AudioMp3 => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/mpeg\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::AudioM4a => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/mp4\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::AudioWav => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/wav\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::AudioOgg => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/ogg\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::AudioFlac => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/flac\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::AudioAac => format!(
            "<audio id=\"source-media-audio\" controls preload=\"metadata\"><source src=\"{}\" type=\"audio/aac\" />Your browser does not support audio playback.</audio>",
            html_escape(raw_href)
        ),
        SourceMediaKind::VideoMp4 => format!(
            "<video id=\"source-media-video\" controls preload=\"metadata\"><source src=\"{}\" type=\"video/mp4\" />Your browser does not support video playback.</video>",
            html_escape(raw_href)
        ),
        SourceMediaKind::VideoM4v => format!(
            "<video id=\"source-media-video\" controls preload=\"metadata\"><source src=\"{}\" type=\"video/mp4\" />Your browser does not support video playback.</video>",
            html_escape(raw_href)
        ),
        SourceMediaKind::VideoMov => format!(
            "<video id=\"source-media-video\" controls preload=\"metadata\"><source src=\"{}\" type=\"video/quicktime\" />Your browser does not support video playback.</video>",
            html_escape(raw_href)
        ),
        SourceMediaKind::VideoWebm => format!(
            "<video id=\"source-media-video\" controls preload=\"metadata\"><source src=\"{}\" type=\"video/webm\" />Your browser does not support video playback.</video>",
            html_escape(raw_href)
        ),
        SourceMediaKind::VideoOgv => format!(
            "<video id=\"source-media-video\" controls preload=\"metadata\"><source src=\"{}\" type=\"video/ogg\" />Your browser does not support video playback.</video>",
            html_escape(raw_href)
        ),
    }
}

fn render_source_media_summary_items(
    media_kind: SourceMediaKind,
    content: &[u8],
    size_bytes: usize,
    last_modified_ms: Option<u64>,
) -> String {
    let mut html = String::new();
    let mut fields: Vec<(&str, String)> = vec![
        ("Raw Size", human_bytes(size_bytes)),
        (
            "Last Mod",
            last_modified_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Status", "Media Preview".to_string()),
    ];
    match media_kind {
        SourceMediaKind::ImagePng
        | SourceMediaKind::ImageJpeg
        | SourceMediaKind::ImageGif
        | SourceMediaKind::ImageWebp
        | SourceMediaKind::ImageBmp => {
            if let Some((w, h)) = parse_image_dimensions(media_kind, content) {
                fields.push(("Width", format!("{}px", w)));
                fields.push(("Height", format!("{}px", h)));
            }
        }
        SourceMediaKind::AudioMp3
        | SourceMediaKind::AudioM4a
        | SourceMediaKind::AudioWav
        | SourceMediaKind::AudioOgg
        | SourceMediaKind::AudioFlac
        | SourceMediaKind::AudioAac
        | SourceMediaKind::VideoMp4
        | SourceMediaKind::VideoM4v
        | SourceMediaKind::VideoMov
        | SourceMediaKind::VideoWebm
        | SourceMediaKind::VideoOgv => {
            let duration = match media_kind {
                SourceMediaKind::AudioMp3 => estimate_mp3_duration_seconds(content),
                SourceMediaKind::AudioM4a => parse_mp4_duration_seconds(content),
                SourceMediaKind::AudioWav => parse_wav_duration_seconds(content),
                SourceMediaKind::AudioOgg => parse_ogg_duration_seconds(content),
                SourceMediaKind::AudioFlac => parse_flac_duration_seconds(content),
                SourceMediaKind::AudioAac => parse_aac_adts_duration_seconds(content),
                SourceMediaKind::VideoMp4
                | SourceMediaKind::VideoM4v
                | SourceMediaKind::VideoMov => parse_mp4_duration_seconds(content),
                SourceMediaKind::VideoWebm | SourceMediaKind::VideoOgv => None,
                SourceMediaKind::ImagePng
                | SourceMediaKind::ImageJpeg
                | SourceMediaKind::ImageGif
                | SourceMediaKind::ImageWebp
                | SourceMediaKind::ImageBmp => None,
            };
            if let Some(duration_text) = duration.and_then(format_duration_text) {
                fields.push(("Duration", duration_text));
            }
        }
    }
    for (label, value) in fields {
        html.push_str("<li><span class=\"label\">");
        html.push_str(label);
        html.push_str("</span><span class=\"value\">");
        html.push_str(&html_escape(&value));
        html.push_str("</span></li>");
    }
    html
}

fn create_source_media_response(
    file_path: &Path,
    content: &[u8],
    source_size_bytes: usize,
    parent_directory_href: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    mode_switch_html: &str,
    media_kind: SourceMediaKind,
    raw_href: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "Source".to_string());
    let absolute_path = format_display_path(file_path);
    let summary_items = render_source_media_summary_items(
        media_kind,
        content,
        source_size_bytes,
        get_last_modified_ms(file_path),
    );
    let (main_css_link, main_js_script) = match markdown_highlight {
        Some(cfg) => (
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.main_css_url)
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(&cfg.main_js_url)
            ),
        ),
        None => ("".to_string(), "".to_string()),
    };
    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("media");
    let media_html = render_source_media_html(media_kind, raw_href, file_name);
    let html = SOURCE_MEDIA_VIEW_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace("__MAIN_JS_SCRIPT__", &main_js_script)
        .replace(
            "__COMMON_HEADER_HTML__",
            &template_common::render_main_header_html(
                &absolute_path,
                Some(parent_directory_href),
                None,
            ),
        )
        .replace("__MODE_SWITCH_HTML__", mode_switch_html)
        .replace("__SUMMARY_ITEMS__", &summary_items)
        .replace("__MEDIA_HTML__", &media_html);
    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}

fn resolve_content_type_and_download(file_path: &Path, content: &[u8]) -> (String, bool) {
    let detected = get_content_type(&file_path.to_path_buf());
    if detected != "application/octet-stream" {
        return (detected, false);
    }
    if is_probably_binary(content) {
        ("application/octet-stream".to_string(), true)
    } else {
        ("text/plain".to_string(), false)
    }
}

fn make_attachment_disposition(file_path: &Path) -> String {
    let filename = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.bin")
        .replace('"', "_")
        .replace(['\r', '\n'], "");
    format!("attachment; filename=\"{}\"", filename)
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
            let mode_switch_html = template_common::build_mode_switch_html(
                url_path,
                url_query,
                content_mode,
                "content-mode",
                ModeSwitchVariant::SourceView,
            );
            if is_markdown_file(file_path.as_path()) && content_mode == ContentMode::Source {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                let parent_directory_href = resolve_source_parent_directory_href(
                    url_path,
                    url_query,
                    content_mode,
                    file_path.as_path(),
                );
                return create_markdown_response(
                    file_path.as_path(),
                    &decoded,
                    content.len(),
                    &parent_directory_href,
                    allow_html_in_md,
                    markdown_open_external_link_in_new_tab,
                    markdown_highlight,
                    &mode_switch_html,
                );
            }
            if is_structured_data_file(file_path.as_path()) && content_mode == ContentMode::Source {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                let parent_directory_href = resolve_source_parent_directory_href(
                    url_path,
                    url_query,
                    content_mode,
                    file_path.as_path(),
                );
                return create_structured_data_response(
                    file_path.as_path(),
                    &decoded,
                    &parent_directory_href,
                    markdown_highlight,
                    &mode_switch_html,
                    content.len(),
                );
            }
            let (base_content_type, should_download) =
                resolve_content_type_and_download(file_path.as_path(), &content);
            let is_binary_content = is_probably_binary(&content);
            if content_mode == ContentMode::Source && !should_download {
                if let Some(media_kind) = detect_source_media_kind(
                    file_path.as_path(),
                    &base_content_type,
                    &content,
                    is_binary_content,
                ) {
                    let parent_directory_href = resolve_source_parent_directory_href(
                        url_path,
                        url_query,
                        content_mode,
                        file_path.as_path(),
                    );
                    let raw_href =
                        template_common::build_mode_href(url_path, url_query, ContentMode::Raw);
                    return create_source_media_response(
                        file_path.as_path(),
                        &content,
                        content.len(),
                        &parent_directory_href,
                        markdown_highlight,
                        &mode_switch_html,
                        media_kind,
                        &raw_href,
                    );
                }
            }
            if content_mode == ContentMode::Source
                && !should_download
                && is_text_type(&base_content_type)
            {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                let parent_directory_href = resolve_source_parent_directory_href(
                    url_path,
                    url_query,
                    content_mode,
                    file_path.as_path(),
                );
                return create_source_text_response(
                    file_path.as_path(),
                    &decoded,
                    content.len(),
                    &parent_directory_href,
                    markdown_highlight,
                    &mode_switch_html,
                );
            }
            let content_type = if content_mode == ContentMode::Raw && !is_binary_content {
                let encoding = detect_encoding(&content);
                let charset = encoding.name();
                format!("text/plain; charset={}", charset)
            } else if is_text_type(&base_content_type) {
                let encoding = detect_encoding(&content);
                let charset = encoding.name();
                format!("{}; charset={}", base_content_type, charset)
            } else {
                base_content_type
            };
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
                let mut response = Response::from_data(content)
                    .with_header(header)
                    .with_status_code(StatusCode(200));
                if should_download {
                    if let Ok(disposition) = Header::from_bytes(
                        &b"Content-Disposition"[..],
                        make_attachment_disposition(file_path.as_path()).as_bytes(),
                    ) {
                        response = response.with_header(disposition);
                    }
                }
                response
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
    get_web_content_type(path.as_path())
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
        active_root_path.clone()
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
                    if decoded.contains("..") {
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
            // Special case: if url_path is "/", show directory listing
            if url_path == "/" {
                if active_root_path.exists() && active_root_path.is_dir() {
                    return create_directory_listing(
                        active_root_path.as_path(),
                        public_url_path.as_str(),
                        request_query,
                        content_mode,
                        markdown_highlight,
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
                // Generate directory listing
                return create_directory_listing(
                    &file_path,
                    public_url_path.as_str(),
                    request_query,
                    content_mode,
                    markdown_highlight,
                );
            }
            return create_error_response(StatusCode(404), "Not Found");
        }
    };

    // If the normalized path is a directory, show directory listing
    if normalized_path.is_dir() {
        return create_directory_listing(
            &normalized_path,
            public_url_path.as_str(),
            request_query,
            content_mode,
            markdown_highlight,
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
