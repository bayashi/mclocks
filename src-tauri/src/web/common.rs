use std::path::Path;
use tiny_http::{Response, StatusCode};

pub fn create_error_response(
    status_code: StatusCode,
    message: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(message).with_status_code(status_code)
}

pub fn is_supported_web_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("html") => true,
        Some(ext) if ext.eq_ignore_ascii_case("css") => true,
        Some(ext) if ext.eq_ignore_ascii_case("js") => true,
        Some(ext) if ext.eq_ignore_ascii_case("json") => true,
        Some(ext) if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") => true,
        Some(ext)
            if ext.eq_ignore_ascii_case("ini")
                || ext.eq_ignore_ascii_case("config")
                || ext.eq_ignore_ascii_case("cfg") =>
        {
            true
        }
        Some(ext) if ext.eq_ignore_ascii_case("toml") => true,
        Some(ext) if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") => true,
        Some(ext) if ext.eq_ignore_ascii_case("png") => true,
        Some(ext) if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") => true,
        Some(ext) if ext.eq_ignore_ascii_case("gif") => true,
        Some(ext) if ext.eq_ignore_ascii_case("svg") => true,
        Some(ext) if ext.eq_ignore_ascii_case("ico") => true,
        Some(ext) if ext.eq_ignore_ascii_case("txt") => true,
        _ => false,
    }
}

pub fn get_web_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("html") => "text/html",
        Some(ext) if ext.eq_ignore_ascii_case("css") => "text/css",
        Some(ext) if ext.eq_ignore_ascii_case("js") => "application/javascript",
        Some(ext) if ext.eq_ignore_ascii_case("json") => "application/json",
        Some(ext) if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") => {
            "application/yaml"
        }
        Some(ext)
            if ext.eq_ignore_ascii_case("ini")
                || ext.eq_ignore_ascii_case("config")
                || ext.eq_ignore_ascii_case("cfg") =>
        {
            "text/plain"
        }
        Some(ext) if ext.eq_ignore_ascii_case("toml") => "text/plain",
        Some(ext) if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") => {
            "text/markdown"
        }
        Some(ext) if ext.eq_ignore_ascii_case("png") => "image/png",
        Some(ext) if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") => {
            "image/jpeg"
        }
        Some(ext) if ext.eq_ignore_ascii_case("gif") => "image/gif",
        Some(ext) if ext.eq_ignore_ascii_case("svg") => "image/svg+xml",
        Some(ext) if ext.eq_ignore_ascii_case("ico") => "image/x-icon",
        Some(ext) if ext.eq_ignore_ascii_case("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}
