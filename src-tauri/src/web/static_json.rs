use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::{Header, Response, StatusCode};

#[derive(Clone, Copy)]
enum StructuredDataFormat {
    Json,
    Yaml,
    Toml,
}

impl StructuredDataFormat {
    fn file_label(self) -> &'static str {
        match self {
            StructuredDataFormat::Json => "JSON",
            StructuredDataFormat::Yaml => "YAML",
            StructuredDataFormat::Toml => "TOML",
        }
    }

    fn invalid_prefix(self) -> &'static str {
        match self {
            StructuredDataFormat::Json => "Invalid JSON",
            StructuredDataFormat::Yaml => "Invalid YAML",
            StructuredDataFormat::Toml => "Invalid TOML",
        }
    }
}

const JSON_VIEW_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
__MAIN_CSS_LINK__
__STATIC_JSON_CSS_LINK__
</head>
<body class="mclocks-json">
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
__MAIN_JS_SCRIPT__
__STATIC_JSON_JS_SCRIPT__
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

fn inject_indent_after_opening_tag(html: &str, indent: usize) -> String {
    if let Some(tag_end) = html.find('>') {
        let mut out = String::with_capacity(html.len() + indent);
        out.push_str(&html[..=tag_end]);
        push_indent(&mut out, indent);
        out.push_str(&html[tag_end + 1..]);
        return out;
    }
    let mut out = String::with_capacity(html.len() + indent);
    push_indent(&mut out, indent);
    out.push_str(html);
    out
}

fn render_colorized_json(value: &Value) -> String {
    let mut out = String::new();
    render_colorized_json_with_indent(value, "", 0, &mut out);
    out
}

fn child_path(base_path: &str, segment: &str) -> String {
    if base_path.is_empty() {
        segment.to_string()
    } else {
        format!("{}.{}", base_path, segment)
    }
}

fn wrap_json_node(path: &str, inner_html: String) -> String {
    if path.is_empty() {
        return inner_html;
    }
    format!(
        "<span class=\"json-node\" data-path=\"{}\">{}</span>",
        html_escape(path),
        inner_html
    )
}

fn render_delimiter(delimiter: &str) -> String {
    format!("<span class=\"json-delim\">{}</span>", delimiter)
}

fn render_colorized_json_with_indent(value: &Value, path: &str, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut inner = String::new();
            inner.push_str(&render_delimiter("{"));
            if map.is_empty() {
                inner.push_str(&render_delimiter("}"));
                out.push_str(&wrap_json_node(path, inner));
                return;
            }
            inner.push('\n');
            let len = map.len();
            for (idx, (key, child)) in map.iter().enumerate() {
                push_indent(&mut inner, indent + 2);
                let escaped_key =
                    serde_json::to_string(key).unwrap_or_else(|_| "\"<key>\"".to_string());
                let child_path = child_path(path, key);
                inner.push_str("<span class=\"json-entry-key\" data-key-path=\"");
                inner.push_str(&html_escape(&child_path));
                inner.push_str("\"><span class=\"json-key\">");
                inner.push_str(&html_escape(&escaped_key));
                inner.push_str("</span>:</span> ");
                render_colorized_json_with_indent(child, &child_path, indent + 2, &mut inner);
                if idx + 1 < len {
                    inner.push(',');
                }
                inner.push('\n');
            }
            push_indent(&mut inner, indent);
            inner.push_str(&render_delimiter("}"));
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::Array(arr) => {
            let mut inner = String::new();
            inner.push_str(&render_delimiter("["));
            if arr.is_empty() {
                inner.push_str(&render_delimiter("]"));
                out.push_str(&wrap_json_node(path, inner));
                return;
            }
            inner.push('\n');
            let len = arr.len();
            for (idx, child) in arr.iter().enumerate() {
                let child_path = child_path(path, &idx.to_string());
                if matches!(child, Value::Object(_) | Value::Array(_)) {
                    let mut child_inner = String::new();
                    render_colorized_json_with_indent(
                        child,
                        &child_path,
                        indent + 2,
                        &mut child_inner,
                    );
                    inner.push_str(&inject_indent_after_opening_tag(&child_inner, indent + 2));
                } else {
                    push_indent(&mut inner, indent + 2);
                    render_colorized_json_with_indent(child, &child_path, indent + 2, &mut inner);
                }
                if idx + 1 < len {
                    inner.push(',');
                }
                inner.push('\n');
            }
            push_indent(&mut inner, indent);
            inner.push_str(&render_delimiter("]"));
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::String(s) => {
            let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
            let inner = format!(
                "<span class=\"json-string\">{}</span>",
                html_escape(&escaped)
            );
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::Number(n) => {
            let inner = format!(
                "<span class=\"json-number\">{}</span>",
                html_escape(&n.to_string())
            );
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::Bool(b) => {
            let inner = format!(
                "<span class=\"json-bool\">{}</span>",
                if *b { "true" } else { "false" }
            );
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::Null => {
            out.push_str(&wrap_json_node(
                path,
                "<span class=\"json-null\">null</span>".to_string(),
            ));
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

fn render_outline_node_html(label: &str, full_path: &str, value: &Value, depth: usize) -> String {
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

fn render_outline_items(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "<li><span class=\"value\">Outline is unavailable</span></li>".to_string();
    };
    render_outline_node_html("", "", value, 0)
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

pub fn is_yaml_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"),
        None => false,
    }
}

pub fn is_toml_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("toml"),
        None => false,
    }
}

pub fn is_structured_data_file(path: &Path) -> bool {
    is_json_file(path) || is_yaml_file(path) || is_toml_file(path)
}

fn detect_structured_data_format(path: &Path) -> StructuredDataFormat {
    if is_yaml_file(path) {
        StructuredDataFormat::Yaml
    } else if is_toml_file(path) {
        StructuredDataFormat::Toml
    } else {
        StructuredDataFormat::Json
    }
}

fn parse_structured_data(
    format: StructuredDataFormat,
    source: &str,
) -> Result<Value, serde_json::Error> {
    match format {
        StructuredDataFormat::Json => serde_json::from_str::<Value>(source),
        StructuredDataFormat::Yaml => match serde_yaml::from_str::<Value>(source) {
            Ok(value) => Ok(value),
            Err(err) => {
                let message = format!("{}: {}", StructuredDataFormat::Yaml.invalid_prefix(), err);
                let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
                Err(serde_json::Error::io(io_err))
            }
        },
        StructuredDataFormat::Toml => match toml::from_str::<toml::Value>(source) {
            Ok(value) => Ok(convert_toml_to_json(value)),
            Err(err) => {
                let message = format!("{}: {}", format.invalid_prefix(), err);
                let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
                Err(serde_json::Error::io(io_err))
            }
        },
    }
}

fn convert_toml_to_json(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(n) => Value::Number(n.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Array(items) => {
            Value::Array(items.into_iter().map(convert_toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, item) in table {
                map.insert(key, convert_toml_to_json(item));
            }
            Value::Object(map)
        }
    }
}

pub fn create_structured_data_response(
    file_path: &Path,
    source: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    raw_content_toggle_href: &str,
    source_size_bytes: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let format = detect_structured_data_format(file_path);
    let file_label = format.file_label();
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| file_label.to_string());
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = parse_structured_data(format, source);

    let (root_type, children_count, json_html, outline_items, notices_html, view_status) =
        match parsed {
            Ok(value) => {
                let (root_type, children_count) = classify_json(&value);
                let rendered = if should_colorize {
                    render_colorized_json(&value)
                } else {
                    match serde_json::to_string_pretty(&value) {
                        Ok(pretty) => html_escape(&pretty),
                        Err(_) => html_escape(source),
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
                let mut invalid_status = format!("Parse Error: {}", err);
                if !should_colorize {
                    invalid_status.push_str(" / Colorize: disabled >10 MB");
                }
                (
                    "invalid".to_string(),
                    0usize,
                    html_escape(source),
                    render_outline_items(None),
                    if matches!(format, StructuredDataFormat::Json) {
                        render_notice_items(Some(&err))
                    } else {
                        let mut html = String::new();
                        html.push_str("<div class=\"notice error\">");
                        html.push_str(&html_escape(&err.to_string()));
                        html.push_str("</div>");
                        html
                    },
                    invalid_status,
                )
            }
        };

    let summary_items =
        render_summary_items(&root_type, children_count, source_size_bytes, &view_status);

    let (main_css_link, static_json_css_link, main_js_script, static_json_js_script) =
        match markdown_highlight {
            Some(cfg) => (
                format!(
                    "<link rel=\"stylesheet\" href=\"{}\" />",
                    html_escape(&cfg.main_css_url)
                ),
                format!(
                    "<link rel=\"stylesheet\" href=\"{}\" />",
                    html_escape(&cfg.static_json_css_url)
                ),
                format!(
                    "<script src=\"{}\"></script>",
                    html_escape(&cfg.main_js_url)
                ),
                format!(
                    "<script src=\"{}\"></script>",
                    html_escape(&cfg.static_json_js_url)
                ),
            ),
            None => (
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ),
        };
    let html = JSON_VIEW_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace("__STATIC_JSON_CSS_LINK__", &static_json_css_link)
        .replace("__MAIN_JS_SCRIPT__", &main_js_script)
        .replace("__STATIC_JSON_JS_SCRIPT__", &static_json_js_script)
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
