use super::ini::{create_ini_response, is_ini_file};
use super::json::{create_json_response, is_json_file};
use super::toml::{create_toml_response, is_toml_file};
use super::yaml::{create_yaml_response, is_yaml_file};
use crate::web_server::WebMarkdownHighlightConfig;
use std::path::Path;
use tiny_http::Response;

pub fn is_structured_data_file(path: &Path) -> bool {
    is_json_file(path) || is_yaml_file(path) || is_toml_file(path) || is_ini_file(path)
}

pub fn create_structured_data_response(
    file_path: &Path,
    source: &str,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    raw_content_toggle_href: &str,
    source_size_bytes: usize,
) -> Response<std::io::Cursor<Vec<u8>>> {
    if is_json_file(file_path) {
        create_json_response(
            file_path,
            source,
            markdown_highlight,
            raw_content_toggle_href,
            source_size_bytes,
        )
    } else if is_yaml_file(file_path) {
        create_yaml_response(
            file_path,
            source,
            markdown_highlight,
            raw_content_toggle_href,
            source_size_bytes,
        )
    } else if is_toml_file(file_path) {
        create_toml_response(
            file_path,
            source,
            markdown_highlight,
            raw_content_toggle_href,
            source_size_bytes,
        )
    } else {
        create_ini_response(
            file_path,
            source,
            markdown_highlight,
            raw_content_toggle_href,
            source_size_bytes,
        )
    }
}
