use super::structured_renderer::{
    JSON_COLORIZE_LIMIT_BYTES, StructuredViewKind, build_html_response, child_path, classify_json,
    get_last_modified_ms, html_escape, push_indent, render_error_notice, render_outline_items,
    render_summary_items, wrap_json_node,
};
use crate::web::common::format_display_path;
use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::Response;

pub fn is_yaml_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"),
        None => false,
    }
}

fn is_scalar_value(value: &Value) -> bool {
    matches!(
        value,
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null
    )
}

fn render_yaml_key(key: &str) -> String {
    let safe_plain = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    if safe_plain && !key.is_empty() {
        html_escape(key)
    } else {
        let escaped = key.replace('\'', "''");
        format!("'{}'", html_escape(&escaped))
    }
}

fn render_yaml_scalar(value: &Value, path: &str) -> String {
    match value {
        Value::String(s) => {
            let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
            wrap_json_node(
                path,
                format!(
                    "<span class=\"yaml-string json-string\">{}</span>",
                    html_escape(&escaped)
                ),
            )
        }
        Value::Number(n) => wrap_json_node(
            path,
            format!(
                "<span class=\"yaml-number json-number\">{}</span>",
                html_escape(&n.to_string())
            ),
        ),
        Value::Bool(b) => wrap_json_node(
            path,
            format!(
                "<span class=\"yaml-bool json-bool\">{}</span>",
                if *b { "true" } else { "false" }
            ),
        ),
        Value::Null => wrap_json_node(
            path,
            "<span class=\"yaml-null json-null\">null</span>".to_string(),
        ),
        _ => String::new(),
    }
}

fn render_colorized_yaml(value: &Value) -> String {
    let mut out = String::new();
    render_colorized_yaml_with_indent(value, "", 0, &mut out);
    out
}

fn render_colorized_yaml_with_indent(value: &Value, path: &str, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                out.push_str(&wrap_json_node(
                    path,
                    "<span class=\"yaml-delim json-delim\">{}</span>".to_string(),
                ));
                return;
            }
            let mut inner = String::new();
            for (index, (key, child)) in map.iter().enumerate() {
                if index > 0 {
                    inner.push('\n');
                }
                push_indent(&mut inner, indent);
                let child_path = child_path(path, key);
                inner.push_str("<span class=\"json-entry-key\" data-key-path=\"");
                inner.push_str(&html_escape(&child_path));
                inner.push_str("\"><span class=\"yaml-key json-key\">");
                inner.push_str(&render_yaml_key(key));
                inner.push_str("</span>:</span>");
                if is_scalar_value(child) {
                    inner.push(' ');
                    inner.push_str(&render_yaml_scalar(child, &child_path));
                } else {
                    inner.push('\n');
                    render_colorized_yaml_with_indent(child, &child_path, indent + 2, &mut inner);
                }
            }
            out.push_str(&wrap_json_node(path, inner));
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str(&wrap_json_node(
                    path,
                    "<span class=\"yaml-delim json-delim\">[]</span>".to_string(),
                ));
                return;
            }
            let mut inner = String::new();
            for (index, child) in arr.iter().enumerate() {
                if index > 0 {
                    inner.push('\n');
                }
                push_indent(&mut inner, indent);
                inner.push_str("<span class=\"yaml-delim json-delim\">-</span>");
                let child_path = child_path(path, &index.to_string());
                if is_scalar_value(child) {
                    inner.push(' ');
                    inner.push_str(&render_yaml_scalar(child, &child_path));
                } else {
                    inner.push('\n');
                    render_colorized_yaml_with_indent(child, &child_path, indent + 2, &mut inner);
                }
            }
            out.push_str(&wrap_json_node(path, inner));
        }
        _ => out.push_str(&render_yaml_scalar(value, path)),
    }
}

pub fn create_yaml_response(
    file_path: &Path,
    source: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    raw_content_toggle_href: &str,
    source_size_bytes: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "YAML".to_string());
    let absolute_path = format_display_path(file_path);
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => Ok(value),
        Err(err) => {
            let message = format!("Invalid YAML: {}", err);
            let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
            Err(serde_json::Error::io(io_err))
        }
    };
    let (root_type, _children_count, json_html, outline_items, notices_html, view_status) =
        match parsed {
            Ok(value) => {
                let (root_type, children_count) = classify_json(&value);
                let rendered = if should_colorize {
                    render_colorized_yaml(&value)
                } else {
                    html_escape(source)
                };
                (
                    root_type.to_string(),
                    children_count,
                    rendered,
                    render_outline_items(Some(&value)),
                    String::new(),
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
                    render_error_notice(&err.to_string()),
                    invalid_status,
                )
            }
        };
    let summary_items = render_summary_items(
        &root_type,
        source_size_bytes,
        get_last_modified_ms(file_path),
        &view_status,
    );
    build_html_response(
        &page_title,
        &absolute_path,
        &json_html,
        &outline_items,
        &notices_html,
        &summary_items,
        raw_content_toggle_href,
        markdown_highlight,
        StructuredViewKind::Yaml,
    )
}
