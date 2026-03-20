use std::path::Path;
use tiny_http::{Response, StatusCode};

pub fn create_error_response(
    status_code: StatusCode,
    message: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(message).with_status_code(status_code)
}

pub fn get_web_content_type(path: &Path) -> String {
    let guessed = mime_guess::from_path(path).first_or_octet_stream();
    guessed.essence_str().to_string()
}

pub fn format_display_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if cfg!(windows) {
        if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", rest);
        }
        if let Some(rest) = raw.strip_prefix(r"\\?\") {
            return rest.to_string();
        }
    }
    raw.to_string()
}
