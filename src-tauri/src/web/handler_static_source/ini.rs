use super::structured_renderer::{
    JSON_COLORIZE_LIMIT_BYTES, StructuredViewKind, build_html_response, child_path, classify_json,
    get_last_modified_ms, html_escape, parse_ini_to_json, render_error_notice,
    render_outline_items, render_summary_items, wrap_json_node,
};
use crate::web::common::format_display_path;
use crate::web_server::WebMarkdownHighlightConfig;
use serde_json::Value;
use std::path::Path;
use tiny_http::Response;

pub fn is_ini_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => {
            ext.eq_ignore_ascii_case("ini")
                || ext.eq_ignore_ascii_case("config")
                || ext.eq_ignore_ascii_case("cfg")
        }
        None => false,
    }
}

fn render_ini_value(value: &Value, path: &str) -> String {
    match value {
        Value::String(s) => wrap_json_node(
            path,
            format!(
                "<span class=\"ini-string json-string\">{}</span>",
                html_escape(s)
            ),
        ),
        Value::Null => wrap_json_node(
            path,
            "<span class=\"ini-null json-null\"></span>".to_string(),
        ),
        Value::Bool(b) => wrap_json_node(
            path,
            format!(
                "<span class=\"ini-bool json-bool\">{}</span>",
                if *b { "true" } else { "false" }
            ),
        ),
        Value::Number(n) => wrap_json_node(
            path,
            format!(
                "<span class=\"ini-number json-number\">{}</span>",
                html_escape(&n.to_string())
            ),
        ),
        _ => wrap_json_node(
            path,
            "<span class=\"ini-null json-null\"></span>".to_string(),
        ),
    }
}

fn render_ini_pairs(parent_path: &str, map: &serde_json::Map<String, Value>, out: &mut String) {
    for (key, value) in map {
        let path = child_path(parent_path, key);
        out.push_str(&format!(
			"<span class=\"json-entry-key\" data-key-path=\"{}\"><span class=\"ini-key json-key\">{}</span></span><span class=\"ini-delim json-delim\">=</span>{}\n",
			html_escape(&path),
			html_escape(key),
			render_ini_value(value, &path)
		));
    }
}

fn render_colorized_ini(value: &Value) -> String {
    let Value::Object(root) = value else {
        return String::new();
    };
    let mut out = String::new();
    if let Some(Value::Object(global)) = root.get("_global") {
        render_ini_pairs("_global", global, &mut out);
        if !global.is_empty() {
            out.push('\n');
        }
    }
    if let Some(Value::Object(sections)) = root.get("sections") {
        for (section_name, section_value) in sections {
            out.push_str(&format!(
				"<span class=\"ini-delim json-delim\">[</span><span class=\"json-entry-key\" data-key-path=\"{}\"><span class=\"ini-key json-key\">{}</span></span><span class=\"ini-delim json-delim\">]</span>\n",
				html_escape(&child_path("sections", section_name)),
				html_escape(section_name)
			));
            if let Value::Object(section_map) = section_value {
                render_ini_pairs(&child_path("sections", section_name), section_map, &mut out);
            }
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

pub fn create_ini_response(
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
        .unwrap_or_else(|| "INI".to_string());
    let absolute_path = format_display_path(file_path);
    let should_colorize = source_size_bytes <= JSON_COLORIZE_LIMIT_BYTES;
    let parsed = match parse_ini_to_json(source) {
        Ok(value) => Ok(value),
        Err(err) => {
            let message = format!("Invalid INI: {}", err);
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
                render_colorized_ini(&value)
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
        StructuredViewKind::Ini,
    )
}
