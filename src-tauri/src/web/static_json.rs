use serde_json::Value;
use std::path::Path;
use tiny_http::{Header, Response, StatusCode};

const JSON_VIEW_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
<style>
:root {
	--sidebar-width: 300px;
}
* {
	box-sizing: border-box;
}
body {
	margin: 0;
	background: #000;
	color: #bbb;
	display: flex;
	font-family: "Segoe UI", "Yu Gothic UI", "Meiryo", "Hiragino Kaku Gothic ProN", sans-serif;
	line-height: 1.5;
}
#sidebar {
	position: fixed;
	top: 0;
	left: 0;
	width: var(--sidebar-width);
	height: 100vh;
	background: #0a0a0a;
	border-right: 1px solid #222;
	padding: 12px 12px 14px;
	overflow-y: auto;
}
#sidebar h2 {
	margin: 0 0 8px;
	font-size: 11px;
	letter-spacing: 0.1em;
	text-transform: uppercase;
	color: #666;
}
#summary-list,
#outline-list {
	list-style: none;
	margin: 0;
	padding: 0;
}
#summary-list li {
	display: flex;
	gap: 8px;
	margin: 0 0 6px;
	font-size: 12px;
}
.label {
	color: #777;
	min-width: 72px;
}
.value {
	color: #ccc;
	overflow-wrap: anywhere;
	word-break: break-word;
}
#notices {
	margin: 0 0 12px;
}
.notice {
	margin: 0 0 8px;
	padding: 8px 10px;
	border: 1px solid #333;
	background: #101010;
	color: #ddd;
	font-size: 12px;
	overflow-wrap: anywhere;
	word-break: break-word;
}
.notice.error {
	border-color: #563232;
	background: #1a0f0f;
	color: #ffadad;
}
#outline-list li {
	margin: 0 0 4px;
	font-size: 12px;
	color: #aaa;
	white-space: normal;
	overflow-wrap: anywhere;
	word-break: break-word;
}
#outline-list ul {
	list-style: none;
	margin: 4px 0 0;
	padding-left: 14px;
	border-left: 1px solid #1c1c1c;
}
#sidebar-footer {
	margin-top: 12px;
	padding-top: 10px;
	border-top: 1px solid #222;
}
#raw-toggle {
	display: inline-block;
	color: #ccc;
	text-decoration: none;
	font-size: 12px;
	padding: 4px 8px;
	border: 1px solid #444;
	border-radius: 2px;
}
#raw-toggle:hover {
	background: #1a1a1a;
	color: #fff;
}
#resizer {
	position: fixed;
	top: 0;
	left: calc(var(--sidebar-width) - 3px);
	width: 6px;
	height: 100vh;
	cursor: col-resize;
	background: transparent;
	z-index: 20;
}
#resizer:hover {
	background: #222;
}
#resizer.is-dragging {
	background: #333;
}
#main {
	margin-left: var(--sidebar-width);
	width: calc(100vw - var(--sidebar-width));
	padding: 16px 24px;
}
#json-view {
	margin: 0;
	white-space: pre-wrap;
	overflow-wrap: anywhere;
	word-break: break-word;
	font-family: "Consolas", "Cascadia Code", "SFMono-Regular", "Menlo", "Monaco", "Courier New", monospace;
	line-height: 1.45;
	color: #ddd;
}
.json-key {
	color: #93c5fd;
	font-weight: 700;
}
.json-string {
	color: #86efac;
}
.json-number {
	color: #fdba74;
}
.json-bool {
	color: #c4b5fd;
}
.json-null {
	color: #9ca3af;
}
</style>
</head>
<body>
<aside id="sidebar">
<h2>Summary</h2>
<ul id="summary-list">__SUMMARY_ITEMS__</ul>
<div id="notices">__NOTICE_ITEMS__</div>
<h2>Outline</h2>
<ul id="outline-list">__OUTLINE_ITEMS__</ul>
<div id="sidebar-footer">
<a id="raw-toggle" href="__RAW_TOGGLE_HREF__">Raw</a>
</div>
</aside>
<div id="resizer" aria-label="Resize sidebar" title="Drag to resize"></div>
<main id="main">
<pre id="json-view">__JSON_VIEW_HTML__</pre>
</main>
<script>
const root = document.documentElement;
const resizer = document.getElementById("resizer");
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 680;
const SIDEBAR_WIDTH_STORAGE_KEY = "mclocks-json-sidebar-width";

const storedWidthRaw = localStorage.getItem(SIDEBAR_WIDTH_STORAGE_KEY);
if (storedWidthRaw !== null) {
	const storedWidth = Number(storedWidthRaw);
	if (Number.isFinite(storedWidth)) {
		const clamped = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, storedWidth));
		root.style.setProperty("--sidebar-width", `${clamped}px`);
	}
}

let isResizing = false;
const setSidebarWidth = (rawWidth) => {
	const clamped = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, rawWidth));
	root.style.setProperty("--sidebar-width", `${clamped}px`);
	localStorage.setItem(SIDEBAR_WIDTH_STORAGE_KEY, String(clamped));
};

resizer.addEventListener("mousedown", (e) => {
	e.preventDefault();
	isResizing = true;
	resizer.classList.add("is-dragging");
});

window.addEventListener("mousemove", (e) => {
	if (!isResizing) {
		return;
	}
	setSidebarWidth(e.clientX);
});

window.addEventListener("mouseup", () => {
	if (!isResizing) {
		return;
	}
	isResizing = false;
	resizer.classList.remove("is-dragging");
});
</script>
</body>
</html>
"##;

const JSON_COLORIZE_LIMIT_BYTES: usize = 10 * 1024 * 1024;
const OUTLINE_MAX_DEPTH: usize = 3;
const OUTLINE_MAX_CHILDREN: usize = 24;

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

fn classify_json(value: &Value) -> (&'static str, usize) {
    match value {
        Value::Object(map) => ("object", map.len()),
        Value::Array(arr) => ("array", arr.len()),
        Value::String(_) => ("string", 1),
        Value::Number(_) => ("number", 1),
        Value::Bool(_) => ("boolean", 1),
        Value::Null => ("null", 0),
    }
}

fn human_bytes(size: usize) -> String {
    if size < 1024 {
        return format!("{} B", size);
    }
    let kb = size as f64 / 1024.0;
    if kb < 1024.0 {
        return format!("{:.1} KB", kb);
    }
    let mb = kb / 1024.0;
    if mb < 1024.0 {
        return format!("{:.1} MB", mb);
    }
    let gb = mb / 1024.0;
    format!("{:.2} GB", gb)
}

fn render_summary_items(
    root_type: &str,
    children: usize,
    size_bytes: usize,
    view_status: &str,
) -> String {
    let mut html = String::new();
    let fields = [
        ("Root", root_type.to_string()),
        ("Children", children.to_string()),
        ("Raw Size", human_bytes(size_bytes)),
        ("Status", view_status.to_string()),
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

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn render_colorized_json(value: &Value) -> String {
    let mut out = String::new();
    render_colorized_json_with_indent(value, 0, &mut out);
    out
}

fn render_colorized_json_with_indent(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) => {
            out.push('{');
            if map.is_empty() {
                out.push('}');
                return;
            }
            out.push('\n');
            let len = map.len();
            for (idx, (key, child)) in map.iter().enumerate() {
                push_indent(out, indent + 2);
                let escaped_key =
                    serde_json::to_string(key).unwrap_or_else(|_| "\"<key>\"".to_string());
                out.push_str("<span class=\"json-key\">");
                out.push_str(&html_escape(&escaped_key));
                out.push_str("</span>: ");
                render_colorized_json_with_indent(child, indent + 2, out);
                if idx + 1 < len {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent);
            out.push('}');
        }
        Value::Array(arr) => {
            out.push('[');
            if arr.is_empty() {
                out.push(']');
                return;
            }
            out.push('\n');
            let len = arr.len();
            for (idx, child) in arr.iter().enumerate() {
                push_indent(out, indent + 2);
                render_colorized_json_with_indent(child, indent + 2, out);
                if idx + 1 < len {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent);
            out.push(']');
        }
        Value::String(s) => {
            let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
            out.push_str("<span class=\"json-string\">");
            out.push_str(&html_escape(&escaped));
            out.push_str("</span>");
        }
        Value::Number(n) => {
            out.push_str("<span class=\"json-number\">");
            out.push_str(&html_escape(&n.to_string()));
            out.push_str("</span>");
        }
        Value::Bool(b) => {
            out.push_str("<span class=\"json-bool\">");
            out.push_str(if *b { "true" } else { "false" });
            out.push_str("</span>");
        }
        Value::Null => {
            out.push_str("<span class=\"json-null\">null</span>");
        }
    }
}

fn value_shape_label(value: &Value) -> String {
    match value {
        Value::Object(map) => format!("obj{{{}}}", map.len()),
        Value::Array(arr) => format!("ary[{}]", arr.len()),
        Value::String(_) => "string".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Null => "null".to_string(),
    }
}

fn render_outline_node_html(label: &str, value: &Value, depth: usize) -> String {
    let mut html = String::new();
    html.push_str("<li><span class=\"value\">");
    if label.is_empty() {
        html.push_str(&html_escape(&value_shape_label(value)));
    } else {
        html.push_str("<strong>");
        html.push_str(&html_escape(label));
        html.push_str("</strong>: ");
        html.push_str(&html_escape(&value_shape_label(value)));
    }
    html.push_str("</span>");
    if depth < OUTLINE_MAX_DEPTH {
        match value {
            Value::Object(map) => {
                if !map.is_empty() {
                    html.push_str("<ul>");
                    for (index, (key, child)) in map.iter().enumerate() {
                        if index >= OUTLINE_MAX_CHILDREN {
                            html.push_str("<li><span class=\"value\">");
                            html.push_str(&html_escape(&format!(
                                "... (+{} more)",
                                map.len() - OUTLINE_MAX_CHILDREN
                            )));
                            html.push_str("</span></li>");
                            break;
                        }
                        html.push_str(&render_outline_node_html(key, child, depth + 1));
                    }
                    html.push_str("</ul>");
                }
            }
            Value::Array(arr) => {
                if !arr.is_empty() {
                    html.push_str("<ul>");
                    for (index, child) in arr.iter().enumerate() {
                        if index >= OUTLINE_MAX_CHILDREN {
                            html.push_str("<li><span class=\"value\">");
                            html.push_str(&html_escape(&format!(
                                "... (+{} more)",
                                arr.len() - OUTLINE_MAX_CHILDREN
                            )));
                            html.push_str("</span></li>");
                            break;
                        }
                        let child_path = index.to_string();
                        html.push_str(&render_outline_node_html(&child_path, child, depth + 1));
                    }
                    html.push_str("</ul>");
                }
            }
            _ => {}
        }
    }
    html.push_str("</li>");
    html
}

fn render_outline_items(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "<li><span class=\"value\">Outline is unavailable</span></li>".to_string();
    };
    render_outline_node_html("", value, 0)
}

fn render_notice_items(parse_error: Option<&serde_json::Error>) -> String {
    let mut html = String::new();
    if let Some(err) = parse_error {
        let message = format!(
            "Invalid JSON: {} (line {}, column {})",
            err,
            err.line(),
            err.column()
        );
        html.push_str("<div class=\"notice error\">");
        html.push_str(&html_escape(&message));
        html.push_str("</div>");
    }
    html
}

pub fn is_json_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("json"),
        None => false,
    }
}

pub fn create_json_response(
    file_path: &Path,
    json_source: &str,
    raw_content_toggle_href: &str,
    source_size_bytes: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "JSON".to_string());
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = serde_json::from_str::<Value>(json_source);

    let (root_type, children_count, json_html, outline_items, notices_html, view_status) =
        match parsed {
            Ok(value) => {
                let (root_type, children_count) = classify_json(&value);
                let rendered = if should_colorize {
                    render_colorized_json(&value)
                } else {
                    match serde_json::to_string_pretty(&value) {
                        Ok(pretty) => html_escape(&pretty),
                        Err(_) => html_escape(json_source),
                    }
                };
                (
                    root_type.to_string(),
                    children_count,
                    rendered,
                    render_outline_items(Some(&value)),
                    render_notice_items(None),
                    if should_colorize {
                        "Parse OK".to_string()
                    } else {
                        "Parse OK (Colorize: disabled >10 MB)".to_string()
                    },
                )
            }
            Err(err) => {
                let mut invalid_status = format!(
                    "Parse Error: {} (line {}, column {})",
                    err,
                    err.line(),
                    err.column()
                );
                if !should_colorize {
                    invalid_status.push_str(" / Colorize: disabled >10 MB");
                }
                (
                    "invalid".to_string(),
                    0usize,
                    html_escape(json_source),
                    render_outline_items(None),
                    render_notice_items(Some(&err)),
                    invalid_status,
                )
            }
        };

    let summary_items =
        render_summary_items(&root_type, children_count, source_size_bytes, &view_status);

    let html = JSON_VIEW_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__SUMMARY_ITEMS__", &summary_items)
        .replace("__NOTICE_ITEMS__", &notices_html)
        .replace("__OUTLINE_ITEMS__", &outline_items)
        .replace("__RAW_TOGGLE_HREF__", &html_escape(raw_content_toggle_href))
        .replace("__JSON_VIEW_HTML__", &json_html);

    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}
