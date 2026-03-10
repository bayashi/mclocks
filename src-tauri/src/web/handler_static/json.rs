use super::structured_renderer::{
    JSON_COLORIZE_LIMIT_BYTES, StructuredViewKind, build_html_response, child_path, classify_json,
    get_last_modified_ms, html_escape, push_indent, render_json_notice_items, render_outline_items,
    render_summary_items, wrap_json_node,
};
use crate::web::common::format_display_path;
use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::Response;

pub fn is_json_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("json"),
        None => false,
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

fn render_delimiter(delimiter: &str) -> String {
    format!("<span class=\"json-delim\">{}</span>", delimiter)
}

fn render_colorized_json(value: &Value) -> String {
    let mut out = String::new();
    render_colorized_json_with_indent(value, "", 0, &mut out);
    out
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

pub fn create_json_response(
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
        .unwrap_or_else(|| "JSON".to_string());
    let absolute_path = format_display_path(file_path);
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = serde_json::from_str::<Value>(source);
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
                    render_json_notice_items(None),
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
                    render_json_notice_items(Some(&err)),
                    invalid_status,
                )
            }
        };
    let summary_items = render_summary_items(
        &root_type,
        children_count,
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
        StructuredViewKind::Json,
    )
}
