use super::template_common;
use crate::web_server::WebMarkdownHighlightConfig;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, StatusCode};

pub const JSON_COLORIZE_LIMIT_BYTES: usize = 10 * 1024 * 1024;
const OUTLINE_MAX_DEPTH: usize = 3;
const OUTLINE_MAX_CHILDREN: usize = 24;

const STRUCTURED_VIEW_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
__MAIN_CSS_LINK__
__STRUCTURED_COMMON_CSS_LINK__
__STRUCTURED_FORMAT_CSS_LINK__
</head>
<body class="mclocks-json">
<aside id="sidebar">
<h2>Summary</h2>
<ul id="summary-list">__SUMMARY_ITEMS__</ul>
<div id="notices">__NOTICE_ITEMS__</div>
__OUTLINE_SECTION__
</aside>
<div id="resizer" aria-label="Resize sidebar" title="Drag to resize"></div>
<main id="main">
__COMMON_HEADER_HTML__
<div id="main-separator"></div>
<pre id="json-view">__JSON_VIEW_HTML__</pre>
</main>
__MAIN_JS_SCRIPT__
__STRUCTURED_COMMON_JS_SCRIPT__
__STRUCTURED_FORMAT_JS_SCRIPT__
</body>
</html>
"##;

pub enum StructuredViewKind {
    Json,
    Yaml,
    Toml,
    Ini,
}

pub fn html_escape(s: &str) -> String {
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

pub fn child_path(base_path: &str, segment: &str) -> String {
    if base_path.is_empty() {
        segment.to_string()
    } else {
        format!("{}.{}", base_path, segment)
    }
}

pub fn wrap_json_node(path: &str, inner_html: String) -> String {
    if path.is_empty() {
        return inner_html;
    }
    format!(
        "<span class=\"json-node\" data-path=\"{}\">{}</span>",
        html_escape(path),
        inner_html
    )
}

pub fn classify_json(value: &serde_json::Value) -> (&'static str, usize) {
    match value {
        serde_json::Value::Object(map) => ("object", map.len()),
        serde_json::Value::Array(arr) => ("array", arr.len()),
        serde_json::Value::String(_) => ("string", 1),
        serde_json::Value::Number(_) => ("number", 1),
        serde_json::Value::Bool(_) => ("boolean", 1),
        serde_json::Value::Null => ("null", 0),
    }
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

fn raw_size_display(size: usize) -> String {
    human_bytes(size)
}

pub fn render_summary_items(
    root_type: &str,
    size_bytes: usize,
    last_modified_ms: Option<u64>,
    view_status: &str,
) -> String {
    let mut html = String::new();
    let _ = root_type;
    let fields = [
        ("Raw Size", raw_size_display(size_bytes)),
        (
            "Last Mod",
            last_modified_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
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

fn system_time_to_unix_ms(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
}

pub fn get_last_modified_ms(file_path: &Path) -> Option<u64> {
    fs::metadata(file_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(system_time_to_unix_ms)
}

fn value_shape_label(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => format!("obj{{{}}}", map.len()),
        serde_json::Value::Array(arr) => format!("ary[{}]", arr.len()),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Null => "null".to_string(),
    }
}

fn render_outline_node_html(
    label: &str,
    full_path: &str,
    value: &serde_json::Value,
    depth: usize,
) -> String {
    let mut html = String::new();
    if full_path.is_empty() {
        html.push_str("<li><span class=\"value\">");
    } else {
        html.push_str("<li data-path=\"");
        html.push_str(&html_escape(full_path));
        html.push_str("\"><span class=\"value\">");
    }
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
            serde_json::Value::Object(map) => {
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
                        let child_full_path = child_path(full_path, key);
                        html.push_str(&render_outline_node_html(
                            key,
                            &child_full_path,
                            child,
                            depth + 1,
                        ));
                    }
                    html.push_str("</ul>");
                }
            }
            serde_json::Value::Array(arr) => {
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
                        let child_label = index.to_string();
                        let child_full_path = child_path(full_path, &child_label);
                        html.push_str(&render_outline_node_html(
                            &child_label,
                            &child_full_path,
                            child,
                            depth + 1,
                        ));
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

pub fn render_outline_items(value: Option<&serde_json::Value>) -> String {
    let Some(value) = value else {
        return "<li><span class=\"value\">Outline is unavailable</span></li>".to_string();
    };
    render_outline_node_html("", "", value, 0)
}

pub fn render_json_notice_items(parse_error: Option<&serde_json::Error>) -> String {
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

pub fn render_error_notice(err_text: &str) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"notice error\">");
    html.push_str(&html_escape(err_text));
    html.push_str("</div>");
    html
}

pub fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

pub fn build_html_response(
    page_title: &str,
    absolute_path: &str,
    parent_directory_href: &str,
    json_html: &str,
    outline_items: &str,
    show_outline: bool,
    notices_html: &str,
    summary_items: &str,
    mode_switch_html: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    view_kind: StructuredViewKind,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let (
        main_css_link,
        structured_common_css_link,
        structured_format_css_link,
        main_js_script,
        structured_common_js_script,
        structured_format_js_script,
    ) = match markdown_highlight {
        Some(cfg) => (
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.main_css_url)
            ),
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.static_structured_common_css_url)
            ),
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(match view_kind {
                    StructuredViewKind::Json => &cfg.static_json_css_url,
                    StructuredViewKind::Yaml => &cfg.static_yaml_css_url,
                    StructuredViewKind::Toml => &cfg.static_toml_css_url,
                    StructuredViewKind::Ini => &cfg.static_ini_css_url,
                })
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(&cfg.main_js_url)
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(&cfg.static_structured_common_js_url)
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(match view_kind {
                    StructuredViewKind::Json => &cfg.static_json_js_url,
                    StructuredViewKind::Yaml => &cfg.static_yaml_js_url,
                    StructuredViewKind::Toml => &cfg.static_toml_js_url,
                    StructuredViewKind::Ini => &cfg.static_ini_js_url,
                })
            ),
        ),
        None => (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ),
    };
    let outline_section = if show_outline {
        format!(
            "<h2>Outline</h2><ul id=\"outline-list\">{}</ul>",
            outline_items
        )
    } else {
        "".to_string()
    };
    let html = STRUCTURED_VIEW_TEMPLATE
        .replace("__PAGE_TITLE__", page_title)
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace(
            "__STRUCTURED_COMMON_CSS_LINK__",
            &structured_common_css_link,
        )
        .replace(
            "__STRUCTURED_FORMAT_CSS_LINK__",
            &structured_format_css_link,
        )
        .replace("__MAIN_JS_SCRIPT__", &main_js_script)
        .replace(
            "__STRUCTURED_COMMON_JS_SCRIPT__",
            &structured_common_js_script,
        )
        .replace(
            "__STRUCTURED_FORMAT_JS_SCRIPT__",
            &structured_format_js_script,
        )
        .replace("__SUMMARY_ITEMS__", summary_items)
        .replace("__NOTICE_ITEMS__", notices_html)
        .replace("__OUTLINE_SECTION__", &outline_section)
        .replace(
            "__COMMON_HEADER_HTML__",
            &template_common::render_main_header_html(
                absolute_path,
                Some(parent_directory_href),
                Some(mode_switch_html),
            ),
        )
        .replace("__JSON_VIEW_HTML__", json_html);
    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}

pub fn parse_ini_to_json(source: &str) -> Result<serde_json::Value, String> {
    let mut parser = configparser::ini::Ini::new();
    let parsed = parser.read(source.to_string())?;
    let mut root = serde_json::Map::new();
    let mut global = serde_json::Map::new();
    let mut sections = serde_json::Map::new();
    for (section_name, entries) in parsed {
        let mut section_map = serde_json::Map::new();
        for (key, value) in entries {
            section_map.insert(
                key,
                match value {
                    Some(v) => serde_json::Value::String(v),
                    None => serde_json::Value::Null,
                },
            );
        }
        if section_name.eq_ignore_ascii_case("default") {
            for (key, value) in section_map {
                global.insert(key, value);
            }
        } else {
            sections.insert(section_name, serde_json::Value::Object(section_map));
        }
    }
    if !global.is_empty() {
        root.insert("_global".to_string(), serde_json::Value::Object(global));
    }
    root.insert("sections".to_string(), serde_json::Value::Object(sections));
    Ok(serde_json::Value::Object(root))
}

pub fn convert_toml_to_json(value: toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(n) => serde_json::Value::Number(n.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(convert_toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, item) in table {
                map.insert(key, convert_toml_to_json(item));
            }
            serde_json::Value::Object(map)
        }
    }
}
