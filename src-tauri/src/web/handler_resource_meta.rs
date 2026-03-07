use serde_json::json;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, StatusCode};
use urlencoding::decode;

use super::common::create_error_response;

const PREVIEW_BYTE_LEN: usize = 300;
const BINARY_DETECT_BYTE_LEN: usize = 8000;

fn normalize_preview_text(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut prev_space = false;
    for c in raw.chars() {
        let mapped = if c == '\r' || c == '\n' || c == '\t' || c.is_control() {
            ' '
        } else {
            c
        };
        if mapped == ' ' {
            if !prev_space {
                normalized.push(' ');
                prev_space = true;
            }
            continue;
        }
        prev_space = false;
        normalized.push(mapped);
    }
    normalized.trim().to_string()
}

fn read_preview_for_file(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut buf = vec![0u8; BINARY_DETECT_BYTE_LEN];
    let read_len = file.read(&mut buf).ok()?;
    buf.truncate(read_len);
    if buf.contains(&0) {
        return None;
    }
    let preview_len = PREVIEW_BYTE_LEN.min(buf.len());
    let preview = String::from_utf8_lossy(&buf[..preview_len]);
    let normalized = normalize_preview_text(&preview);
    if normalized.is_empty() {
        Some("(empty)".to_string())
    } else {
        Some(normalized)
    }
}

fn count_children_for_directory(path: &Path) -> Option<(u64, u64)> {
    let mut file_count = 0u64;
    let mut dir_count = 0u64;
    let entries = fs::read_dir(path).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        match entry.file_type() {
            Ok(file_type) => {
                if file_type.is_dir() {
                    dir_count += 1;
                } else {
                    file_count += 1;
                }
            }
            Err(_) => {
                file_count += 1;
            }
        }
    }
    Some((file_count, dir_count))
}

fn parse_query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split('?').nth(1)?.split('#').next().unwrap_or("");
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let k = kv.next().unwrap_or("");
        let v = kv.next().unwrap_or("");
        if k == key {
            if let Ok(decoded) = decode(v) {
                return Some(decoded.into_owned());
            }
            return None;
        }
    }
    None
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut idx = 0usize;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", bytes, UNITS[idx])
    } else {
        format!("{:.1} {}", value, UNITS[idx])
    }
}

fn system_time_to_unix_ms(value: SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

fn create_json_response(
    body: String,
    status_code: StatusCode,
) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], b"application/json; charset=utf-8")
    {
        Response::from_string(body)
            .with_header(header)
            .with_status_code(status_code)
    } else {
        Response::from_string(body).with_status_code(status_code)
    }
}

pub fn is_resource_meta_request(path: &str) -> bool {
    path == "/.resource-meta" || path.ends_with("/.resource-meta")
}

pub fn handle_resource_meta_request(
    request_url: &str,
    active_root_path: &PathBuf,
    active_path: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let base_url_path = match active_path.strip_suffix("/.resource-meta") {
        Some("") => "/",
        Some(p) => p,
        None => return create_error_response(StatusCode(400), "Bad Request"),
    };
    if base_url_path.contains("..") || base_url_path.contains("//") {
        return create_error_response(StatusCode(400), "Bad Request");
    }
    let entry_name = match parse_query_param(request_url, "path") {
        Some(v) if !v.is_empty() => v,
        _ => return create_error_response(StatusCode(400), "Bad Request"),
    };
    if entry_name.starts_with('.')
        || entry_name.contains('/')
        || entry_name.contains('\\')
        || entry_name.contains("..")
    {
        return create_error_response(StatusCode(400), "Bad Request");
    }
    let listing_dir = if base_url_path == "/" {
        active_root_path.clone()
    } else {
        let mut decoded_segments = Vec::new();
        for segment in base_url_path.trim_start_matches('/').split('/') {
            if segment.is_empty() {
                return create_error_response(StatusCode(400), "Bad Request");
            }
            match decode(segment) {
                Ok(decoded) => {
                    if decoded.contains("..") {
                        return create_error_response(StatusCode(400), "Bad Request");
                    }
                    decoded_segments.push(decoded.into_owned());
                }
                Err(_) => return create_error_response(StatusCode(400), "Bad Request"),
            }
        }
        active_root_path.join(decoded_segments.join("/"))
    };
    let normalized_root = match active_root_path.canonicalize() {
        Ok(p) => p,
        Err(_) => active_root_path.clone(),
    };
    let normalized_listing_dir = match listing_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return create_error_response(StatusCode(404), "Not Found"),
    };
    if !normalized_listing_dir.starts_with(&normalized_root) {
        return create_error_response(StatusCode(404), "Not Found");
    }
    let target_path = normalized_listing_dir.join(&entry_name);
    let normalized_target_path = match target_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return create_error_response(StatusCode(404), "Not Found"),
    };
    if !normalized_target_path.starts_with(&normalized_listing_dir) {
        return create_error_response(StatusCode(404), "Not Found");
    }
    let metadata = match fs::metadata(&normalized_target_path) {
        Ok(m) => m,
        Err(_) => return create_error_response(StatusCode(404), "Not Found"),
    };
    let size_hr = if metadata.is_file() {
        human_readable_size(metadata.len())
    } else {
        "-".to_string()
    };
    let preview = if metadata.is_file() {
        read_preview_for_file(&normalized_target_path).unwrap_or_else(|| "-".to_string())
    } else if metadata.is_dir() {
        match count_children_for_directory(&normalized_target_path) {
            Some((file_count, dir_count)) => format!("files: {}, dirs: {}", file_count, dir_count),
            None => "-".to_string(),
        }
    } else {
        "-".to_string()
    };
    let modified_ms = metadata.modified().ok().and_then(system_time_to_unix_ms);
    let created_ms = metadata.created().ok().and_then(system_time_to_unix_ms);
    let payload = json!({
        "size_hr": size_hr,
        "preview": preview,
        "modified_ms": modified_ms,
        "created_ms": created_ms
    });
    create_json_response(payload.to_string(), StatusCode(200))
}
