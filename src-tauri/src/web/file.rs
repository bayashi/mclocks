use std::fs;
use std::path::PathBuf;
use tiny_http::{Response, StatusCode, Header};

use super::common::create_error_response;
use super::status_handler::handle_status_request;
use super::slow_handler::handle_slow_request;
use super::dump_handler::handle_dump_request;

pub fn parse_and_validate_path(url: &str, root_path: &PathBuf) -> Result<PathBuf, StatusCode> {
    // Parse URL to get path component (remove query string and fragment)
    let path = match url.split('?').next() {
        Some(p) => p.split('#').next().unwrap_or(p),
        None => "/",
    };

    // Security: Check for directory traversal attempts
    if path.contains("..") || path.contains("//") {
        return Err(StatusCode(400));
    }

    let file_path = if path == "/" {
        root_path.join("index.html")
    } else {
        // Remove leading slash and join with root_path
        let relative_path = path.trim_start_matches('/');
        // Additional check: ensure no absolute path components
        if relative_path.starts_with('/') || (cfg!(windows) && relative_path.contains(':')) {
            return Err(StatusCode(400));
        }
        root_path.join(relative_path)
    };

    // Normalize the path to resolve any symlinks and ensure it's within root_path
    let normalized_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            if file_path.exists() && file_path.starts_with(root_path) {
                file_path
            } else {
                return Err(StatusCode(404));
            }
        }
    };

    let normalized_root = match root_path.canonicalize() {
        Ok(p) => p,
        Err(_) => root_path.clone(),
    };

    if normalized_path.starts_with(&normalized_root) {
        Ok(normalized_path)
    } else {
        Err(StatusCode(404))
    }
}

fn create_file_response(file_path: &PathBuf) -> Response<std::io::Cursor<Vec<u8>>> {
    match fs::read(file_path) {
        Ok(content) => {
            let content_type = get_content_type(file_path);
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
                Response::from_data(content).with_header(header).with_status_code(StatusCode(200))
            } else {
                Response::from_data(content).with_status_code(StatusCode(200))
            }
        }
        Err(_) => create_error_response(StatusCode(500), "Internal Server Error")
    }
}

pub fn get_content_type(path: &PathBuf) -> String {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("md") => "text/markdown",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",
    }.to_string()
}

pub fn handle_web_request(request: &mut tiny_http::Request, root_path: &PathBuf, dump_enabled: bool, slow_enabled: bool, status_enabled: bool) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    let path = url.split('?').next().unwrap_or("/");

    // Check if this is a /status request (including /status/ and any subpaths)
    if status_enabled {
        if path.starts_with("/status/") {
            return handle_status_request(request, path);
        }
    }

    // Check if this is a /slow request (including /slow/ and any subpaths)
    if slow_enabled {
        if path == "/slow" || path.starts_with("/slow/") {
            return handle_slow_request(request);
        }
    }

    // Check if this is a /dump request (including /dump/ and any subpaths)
    if dump_enabled {
        if path == "/dump" || path.starts_with("/dump/") {
            return handle_dump_request(request);
        }
    }

    match parse_and_validate_path(url, root_path) {
        Ok(file_path) => create_file_response(&file_path),
        Err(status_code) => {
            let message = match status_code {
                StatusCode(400) => "Bad Request",
                StatusCode(404) => "Not Found",
                _ => "Internal Server Error",
            };
            create_error_response(status_code, message)
        },
    }
}
