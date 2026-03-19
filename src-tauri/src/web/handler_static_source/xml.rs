use super::structured_renderer::{
    JSON_COLORIZE_LIMIT_BYTES, StructuredViewKind, build_html_response, child_path, classify_json,
    get_last_modified_ms, html_escape, parse_xml_to_json, push_indent, render_error_notice,
    render_outline_items, render_summary_items, wrap_json_node,
};
use crate::web::common::format_display_path;
use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::Response;

pub fn is_xml_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("xml"),
        None => false,
    }
}

fn render_colorized_xml(value: &Value) -> String {
    let mut out = String::new();
    if let Value::Object(map) = value {
        for (key, child) in map {
            render_xml_element(key, child, key, key, 0, &mut out);
        }
    } else {
        out.push_str(&render_xml_scalar(value));
    }
    out
}

fn scalar_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Null => "null".to_string(),
        _ => "null".to_string(),
    }
}

fn scalar_class(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "xml-string json-string",
        Value::Number(_) => "xml-number json-number",
        Value::Bool(_) => "xml-bool json-bool",
        Value::Null => "xml-null json-null",
        _ => "xml-null json-null",
    }
}

fn render_xml_scalar(value: &Value) -> String {
    format!(
        "<span class=\"xml-text {}\">{}</span>",
        scalar_class(value),
        html_escape(&scalar_text(value))
    )
}

fn render_xml_attr(name: &str, value: &Value) -> String {
    format!(
        " <span class=\"xml-attr-name xml-key json-key\">{}</span><span class=\"xml-delim\">=</span><span class=\"xml-attr-quote\">\"</span><span class=\"xml-attr-value xml-string json-string\">{}</span><span class=\"xml-attr-quote\">\"</span>",
        html_escape(name),
        html_escape(&scalar_text(value)),
    )
}

enum TagCloseKind {
    Normal,
    SelfClose,
}

fn render_tag(name: &str, key_path: &str, attrs_html: &str, close_kind: TagCloseKind) -> String {
    let close = match close_kind {
        TagCloseKind::Normal => "&gt;",
        TagCloseKind::SelfClose => " /&gt;",
    };
    format!(
        "<span class=\"json-entry-key\" data-key-path=\"{}\"><span class=\"xml-tag xml-delim\">&lt;<span class=\"xml-tag-name xml-key json-key\">{}</span>{}{}</span></span>",
        html_escape(key_path),
        html_escape(name),
        attrs_html,
        close
    )
}

fn render_close_tag(name: &str) -> String {
    format!(
        "<span class=\"xml-tag xml-delim\">&lt;/<span class=\"xml-tag-name xml-key json-key\">{}</span>&gt;</span>",
        html_escape(name)
    )
}

fn render_open_tag(name: &str, key_path: &str, attrs_html: &str) -> String {
    render_tag(name, key_path, attrs_html, TagCloseKind::Normal)
}

fn render_self_close_tag(name: &str, key_path: &str, attrs_html: &str) -> String {
    render_tag(name, key_path, attrs_html, TagCloseKind::SelfClose)
}

struct XmlObjectParts<'a> {
    attrs_html: String,
    text_value: Option<&'a Value>,
    child_entries: Vec<(&'a str, &'a Value)>,
}

fn split_xml_object_parts(map: &serde_json::Map<String, Value>) -> XmlObjectParts<'_> {
    let mut attrs_html = String::new();
    let mut text_value: Option<&Value> = None;
    let mut child_entries: Vec<(&str, &Value)> = Vec::new();
    for (child_key, child_value) in map {
        if let Some(attr_name) = child_key.strip_prefix('@') {
            attrs_html.push_str(&render_xml_attr(attr_name, child_value));
        } else if child_key == "#text" {
            text_value = Some(child_value);
        } else {
            child_entries.push((child_key.as_str(), child_value));
        }
    }
    XmlObjectParts {
        attrs_html,
        text_value,
        child_entries,
    }
}

fn render_xml_object_with_children(
    name: &str,
    key_path: &str,
    node_path: &str,
    indent: usize,
    parts: XmlObjectParts<'_>,
    out: &mut String,
) {
    let mut node_html = String::new();
    push_indent(&mut node_html, indent);
    node_html.push_str(&render_open_tag(name, key_path, &parts.attrs_html));
    node_html.push('\n');
    for (child_name, child_value) in parts.child_entries {
        let child_key_path = child_path(key_path, child_name);
        match child_value {
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        node_html.push('\n');
                    }
                    let item_node_path = child_path(&child_key_path, &index.to_string());
                    render_xml_element(
                        child_name,
                        item,
                        &child_key_path,
                        &item_node_path,
                        indent + 2,
                        &mut node_html,
                    );
                    node_html.push('\n');
                }
            }
            _ => {
                render_xml_element(
                    child_name,
                    child_value,
                    &child_key_path,
                    &child_key_path,
                    indent + 2,
                    &mut node_html,
                );
                node_html.push('\n');
            }
        }
    }
    if let Some(text) = parts.text_value {
        push_indent(&mut node_html, indent + 2);
        node_html.push_str(&render_xml_scalar(text));
        node_html.push('\n');
    }
    push_indent(&mut node_html, indent);
    node_html.push_str(&render_close_tag(name));
    out.push_str(&wrap_json_node(node_path, node_html));
}

fn render_xml_object_simple(
    name: &str,
    key_path: &str,
    node_path: &str,
    indent: usize,
    parts: XmlObjectParts<'_>,
    out: &mut String,
) {
    let mut node_html = String::new();
    push_indent(&mut node_html, indent);
    match parts.text_value {
        None => {
            node_html.push_str(&render_self_close_tag(name, key_path, &parts.attrs_html));
        }
        Some(text) => {
            node_html.push_str(&render_open_tag(name, key_path, &parts.attrs_html));
            node_html.push_str(&render_xml_scalar(text));
            node_html.push_str(&render_close_tag(name));
        }
    }
    out.push_str(&wrap_json_node(node_path, node_html));
}

fn render_xml_object(
    name: &str,
    map: &serde_json::Map<String, Value>,
    key_path: &str,
    node_path: &str,
    indent: usize,
    out: &mut String,
) {
    let parts = split_xml_object_parts(map);
    if parts.child_entries.is_empty() {
        render_xml_object_simple(name, key_path, node_path, indent, parts, out);
        return;
    }
    render_xml_object_with_children(name, key_path, node_path, indent, parts, out);
}

fn render_xml_element(
    name: &str,
    value: &Value,
    key_path: &str,
    node_path: &str,
    indent: usize,
    out: &mut String,
) {
    match value {
        Value::Object(map) => render_xml_object(name, map, key_path, node_path, indent, out),
        Value::Array(arr) => {
            for (index, item) in arr.iter().enumerate() {
                if index > 0 {
                    out.push('\n');
                }
                let item_path = child_path(node_path, &index.to_string());
                render_xml_element(name, item, key_path, &item_path, indent, out);
            }
        }
        _ => {
            let mut node_html = String::new();
            push_indent(&mut node_html, indent);
            node_html.push_str(&render_open_tag(name, key_path, ""));
            node_html.push_str(&render_xml_scalar(value));
            node_html.push_str(&render_close_tag(name));
            out.push_str(&wrap_json_node(node_path, node_html));
        }
    }
}

pub fn create_xml_response(
    file_path: &Path,
    source: &str,
    parent_directory_href: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    mode_switch_html: &str,
    source_size_bytes: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "XML".to_string());
    let absolute_path = format_display_path(file_path);
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = match parse_xml_to_json(source) {
        Ok(value) => Ok(value),
        Err(err) => {
            let message = format!("Invalid XML: {}", err);
            let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
            Err(serde_json::Error::io(io_err))
        }
    };
    let (
        root_type,
        _children_count,
        json_html,
        outline_items,
        show_outline,
        notices_html,
        view_status,
    ) = match parsed {
        Ok(value) => {
            let (root_type, children_count) = classify_json(&value);
            let rendered = if should_colorize {
                render_colorized_xml(&value)
            } else {
                html_escape(source)
            };
            (
                root_type.to_string(),
                children_count,
                rendered,
                render_outline_items(Some(&value)),
                true,
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
                false,
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
        parent_directory_href,
        &json_html,
        &outline_items,
        show_outline,
        &notices_html,
        &summary_items,
        mode_switch_html,
        markdown_highlight,
        StructuredViewKind::Xml,
    )
}
