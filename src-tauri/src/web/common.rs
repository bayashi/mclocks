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
        Some(ext) if ext.eq_ignore_ascii_case("xml") => true,
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
        Some(ext) if ext.eq_ignore_ascii_case("webp") => true,
        Some(ext) if ext.eq_ignore_ascii_case("bmp") => true,
        Some(ext) if ext.eq_ignore_ascii_case("svg") => true,
        Some(ext) if ext.eq_ignore_ascii_case("ico") => true,
        Some(ext) if ext.eq_ignore_ascii_case("mp3") => true,
        Some(ext) if ext.eq_ignore_ascii_case("m4a") => true,
        Some(ext) if ext.eq_ignore_ascii_case("wav") => true,
        Some(ext) if ext.eq_ignore_ascii_case("ogg") => true,
        Some(ext) if ext.eq_ignore_ascii_case("flac") => true,
        Some(ext) if ext.eq_ignore_ascii_case("aac") => true,
        Some(ext) if ext.eq_ignore_ascii_case("mp4") => true,
        Some(ext) if ext.eq_ignore_ascii_case("m4v") => true,
        Some(ext) if ext.eq_ignore_ascii_case("mov") => true,
        Some(ext) if ext.eq_ignore_ascii_case("webm") => true,
        Some(ext) if ext.eq_ignore_ascii_case("ogv") => true,
        Some(ext) if ext.eq_ignore_ascii_case("txt") => true,
        _ => false,
    }
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
