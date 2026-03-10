use super::structured_renderer::{
    JSON_COLORIZE_LIMIT_BYTES, StructuredViewKind, build_html_response, child_path, classify_json,
    convert_toml_to_json, get_last_modified_ms, html_escape, render_error_notice,
    render_outline_items, render_summary_items, wrap_json_node,
};
use crate::web::common::format_display_path;
use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::Response;

pub fn is_toml_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("toml"),
        None => false,
    }
}

fn render_toml_scalar(value: &Value, path: &str) -> String {
    let token = match value {
        Value::String(s) => {
            let escaped = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
            format!(
                "<span class=\"toml-string json-string\">{}</span>",
                html_escape(&escaped)
            )
        }
        Value::Number(n) => format!(
            "<span class=\"toml-number json-number\">{}</span>",
            html_escape(&n.to_string())
        ),
        Value::Bool(b) => format!(
            "<span class=\"toml-bool json-bool\">{}</span>",
            if *b { "true" } else { "false" }
        ),
        Value::Null => "<span class=\"toml-null json-null\">null</span>".to_string(),
        Value::Array(arr) => {
            let mut items = String::new();
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    items.push_str(", ");
                }
                items.push_str(&match item {
                    Value::String(s) => {
                        serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
                    }
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => {
                        if *b {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        }
                    }
                    Value::Null => "null".to_string(),
                    _ => "\"<complex>\"".to_string(),
                });
            }
            format!(
                "<span class=\"toml-delim json-delim\">[</span>{}<span class=\"toml-delim json-delim\">]</span>",
                html_escape(&items)
            )
        }
        _ => "\"<complex>\"".to_string(),
    };
    wrap_json_node(path, token)
}

fn render_toml_table_entries(
    table_path: &str,
    map: &serde_json::Map<String, Value>,
    out: &mut String,
    is_root: bool,
) {
    let mut scalar_keys: Vec<&String> = Vec::new();
    let mut table_keys: Vec<&String> = Vec::new();
    for (key, value) in map {
        if matches!(value, Value::Object(_)) {
            table_keys.push(key);
        } else {
            scalar_keys.push(key);
        }
    }
    if !is_root && !table_path.is_empty() {
        out.push_str("<span class=\"toml-delim json-delim\">[</span>");
        out.push_str(&format!(
			"<span class=\"json-entry-key\" data-key-path=\"{}\"><span class=\"toml-key json-key\">{}</span></span>",
			html_escape(table_path),
			html_escape(table_path)
		));
        out.push_str("<span class=\"toml-delim json-delim\">]</span>\n");
    }
    for key in scalar_keys {
        let value = map.get(key).expect("value exists for key");
        let path = child_path(table_path, key);
        out.push_str(&format!(
			"<span class=\"json-entry-key\" data-key-path=\"{}\"><span class=\"toml-key json-key\">{}</span></span> <span class=\"toml-delim json-delim\">=</span> {}\n",
			html_escape(&path),
			html_escape(key),
			render_toml_scalar(value, &path)
		));
    }
    for key in table_keys {
        let value = map.get(key).expect("value exists for table key");
        if let Value::Object(child_map) = value {
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            let path = child_path(table_path, key);
            render_toml_table_entries(&path, child_map, out, false);
        }
    }
}

fn render_colorized_toml(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut out = String::new();
            render_toml_table_entries("", map, &mut out, true);
            out.trim_end().to_string()
        }
        _ => render_toml_scalar(value, ""),
    }
}

pub fn create_toml_response(
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
        .unwrap_or_else(|| "TOML".to_string());
    let absolute_path = format_display_path(file_path);
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = match toml::from_str::<toml::Value>(source) {
        Ok(value) => Ok(convert_toml_to_json(value)),
        Err(err) => {
            let message = format!("Invalid TOML: {}", err);
            let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
            Err(serde_json::Error::io(io_err))
        }
    };
    let (root_type, children_count, json_html, outline_items, notices_html, view_status) =
        match parsed {
            Ok(value) => {
                let (root_type, children_count) = classify_json(&value);
                let rendered = if should_colorize {
                    render_colorized_toml(&value)
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
        StructuredViewKind::Toml,
    )
}
