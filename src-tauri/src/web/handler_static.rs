use super::dd_publish::{resolve_temp_file, resolve_temp_share};
use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};
use std::fs;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Response, StatusCode};
use urlencoding::{decode, encode};

use super::common::create_error_response;
use super::handler_dump::handle_dump_request;
use super::handler_editor::handle_editor_request;
use super::static_md::{
    build_raw_toggle_href, create_markdown_response, is_markdown_file, should_serve_raw_markdown,
};
use super::handler_slow::handle_slow_request;
use super::handler_status::handle_status_request;

fn create_directory_listing(dir_path: &Path, url_path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    // Decode URL path for display (each segment separately)
    let decoded_path = if url_path == "/" {
        "/".to_string()
    } else {
        let segments: Vec<String> = url_path
            .split('/')
            .map(|segment| {
                if segment.is_empty() {
                    String::new()
                } else {
                    match decode(segment) {
                        Ok(decoded) => decoded.into_owned(),
                        Err(_) => segment.to_string(),
                    }
                }
            })
            .collect();
        segments.join("/")
    };

    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<title>Index of ");
    html.push_str(&html_escape(&decoded_path));
    html.push_str("</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: monospace; margin: 20px; }\n");
    html.push_str("h1 { color: #333; }\n");
    html.push_str("ul { list-style-type: none; padding-left: 0; }\n");
    html.push_str("li { padding: 5px 0; }\n");
    html.push_str("a { text-decoration: none; color: #0066cc; }\n");
    html.push_str("a:hover { text-decoration: underline; }\n");
    html.push_str(".dir::before { content: '📁'; }\n");
    html.push_str(".file::before { content: '📄'; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("<h1>Index of ");
    html.push_str(&html_escape(&decoded_path));
    html.push_str("</h1>\n<ul>\n");

    // Add parent directory link if not at root
    if url_path != "/" {
        let trimmed = url_path.trim_end_matches('/');
        let parent_url = if trimmed == "" {
            "/".to_string()
        } else {
            match trimmed.rfind('/') {
                Some(pos) => {
                    let parent = &trimmed[..pos];
                    if parent.is_empty() {
                        "/".to_string()
                    } else {
                        format!("{}/", parent)
                    }
                }
                None => "/".to_string(),
            }
        };
        html.push_str("<li><a href=\"");
        html.push_str(&html_escape(&parent_url));
        html.push_str("\">../</a></li>\n");
    }

    // Read directory entries
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut dirs: Vec<String> = Vec::new();
            let mut files: Vec<String> = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    if let Some(name) = file_name.to_str() {
                        // Skip hidden files (starting with .)
                        if name.starts_with('.') {
                            continue;
                        }
                        let metadata = entry.metadata();
                        if let Ok(meta) = metadata {
                            if meta.is_dir() {
                                dirs.push(name.to_string());
                            } else {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
            }

            // Sort directories and files
            dirs.sort();
            files.sort();

            // Add directory entries
            for dir in dirs {
                let encoded_dir = encode(&dir);
                let dir_url = if url_path == "/" {
                    format!("/{}/", encoded_dir)
                } else {
                    let base = url_path.trim_end_matches('/');
                    format!("{}/{}/", base, encoded_dir)
                };
                html.push_str("<li class=\"dir\"><a href=\"");
                html.push_str(&html_escape(&dir_url));
                html.push_str("\">");
                html.push_str(&html_escape(&dir));
                html.push_str("/</a></li>\n");
            }

            // Add file entries
            for file in files {
                let encoded_file = encode(&file);
                let file_url = if url_path == "/" {
                    format!("/{}", encoded_file)
                } else {
                    let base = url_path.trim_end_matches('/');
                    format!("{}/{}", base, encoded_file)
                };
                html.push_str("<li class=\"file\"><a href=\"");
                html.push_str(&html_escape(&file_url));
                html.push_str("\">");
                html.push_str(&html_escape(&file));
                html.push_str("</a></li>\n");
            }
        }
        Err(_) => {
            html.push_str("<li>Error reading directory</li>\n");
        }
    }

    html.push_str("</ul>\n</body>\n</html>");

    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}

fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn detect_encoding(content: &[u8]) -> &'static Encoding {
    // Check for BOM first (highest priority)
    if content.len() >= 3 && &content[0..3] == [0xEF, 0xBB, 0xBF] {
        return UTF_8;
    }
    if content.len() >= 2 && &content[0..2] == [0xFF, 0xFE] {
        return encoding_rs::UTF_16LE;
    }
    if content.len() >= 2 && &content[0..2] == [0xFE, 0xFF] {
        return encoding_rs::UTF_16BE;
    }

    // Try UTF-8 decoding first (fast path for common case)
    if std::str::from_utf8(content).is_ok() {
        return UTF_8;
    }

    // Use chardetng to detect encoding for non-UTF-8 content
    let mut detector = EncodingDetector::new();
    detector.feed(content, true);
    let encoding = detector.guess(None, true);

    // chardetng returns &'static Encoding directly
    encoding
}

fn create_file_response(
    file_path: &PathBuf,
    allow_html_in_md: bool,
    serve_raw_markdown: bool,
    raw_toggle_href: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    match fs::read(file_path) {
        Ok(content) => {
            if is_markdown_file(file_path.as_path()) && !serve_raw_markdown {
                let encoding = detect_encoding(&content);
                let (decoded, _, _) = encoding.decode(&content);
                return create_markdown_response(
                    file_path.as_path(),
                    &decoded,
                    allow_html_in_md,
                    raw_toggle_href,
                );
            }
            let base_content_type = get_content_type(file_path);
            let content_type = if is_text_type(&base_content_type) {
                let encoding = detect_encoding(&content);
                let charset = encoding.name();
                format!("{}; charset={}", base_content_type, charset)
            } else {
                base_content_type
            };
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
                Response::from_data(content)
                    .with_header(header)
                    .with_status_code(StatusCode(200))
            } else {
                Response::from_data(content).with_status_code(StatusCode(200))
            }
        }
        Err(_) => create_error_response(StatusCode(500), "Internal Server Error"),
    }
}

fn is_text_type(content_type: &str) -> bool {
    content_type.starts_with("text/")
        || content_type == "application/javascript"
        || content_type == "application/json"
        || content_type == "image/svg+xml"
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
    }
    .to_string()
}

pub fn handle_web_request(
    request: &mut tiny_http::Request,
    root_path: &PathBuf,
    dump_enabled: bool,
    slow_enabled: bool,
    status_enabled: bool,
    allow_html_in_md: bool,
    editor_repos_dir: &Option<String>,
    editor_include_host: bool,
    editor_command: &str,
    editor_args: &[String],
) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    let path = url.split('?').next().unwrap_or("/");
    if let Some(shared_file_path) = resolve_temp_file(path) {
        let serve_raw_markdown = should_serve_raw_markdown(url);
        let raw_toggle_href = build_raw_toggle_href(url);
        return create_file_response(
            &shared_file_path,
            allow_html_in_md,
            serve_raw_markdown,
            &raw_toggle_href,
        );
    }
    let (active_root_path, active_path, public_url_path) = match resolve_temp_share(path) {
        Some((temp_root, temp_relative_path, temp_public_prefix)) => {
            let public_path = if temp_relative_path == "/" {
                format!("{}/", temp_public_prefix)
            } else {
                format!("{}{}", temp_public_prefix, temp_relative_path)
            };
            (temp_root, temp_relative_path, public_path)
        }
        None => (root_path.clone(), path.to_string(), path.to_string()),
    };
    let serve_raw_markdown = should_serve_raw_markdown(url);
    let raw_toggle_href = build_raw_toggle_href(url);

    // Check if this is a /editor request
    if editor_repos_dir.is_some() {
        if active_path == "/editor" || active_path.starts_with("/editor/") {
            return handle_editor_request(
                request,
                editor_repos_dir,
                editor_include_host,
                editor_command,
                editor_args,
            );
        }
    }

    // Check if this is a /status request (including /status/ and any subpaths)
    if status_enabled {
        if active_path.starts_with("/status/") {
            return handle_status_request(request, active_path.as_str());
        }
    }

    // Check if this is a /slow request (including /slow/ and any subpaths)
    if slow_enabled {
        if active_path == "/slow" || active_path.starts_with("/slow/") {
            return handle_slow_request(request);
        }
    }

    // Check if this is a /dump request (including /dump/ and any subpaths)
    if dump_enabled {
        if active_path == "/dump" || active_path.starts_with("/dump/") {
            return handle_dump_request(request);
        }
    }

    let url_path = active_path.as_str();

    // Security: Check for directory traversal attempts (pre-decode)
    if url_path.contains("..") || url_path.contains("//") {
        return create_error_response(StatusCode(400), "Bad Request");
    }

    // Determine the actual file path
    let file_path = if url_path == "/" {
        active_root_path.join("index.html")
    } else {
        let relative_path = url_path.trim_start_matches('/');
        if relative_path.starts_with('/') || (cfg!(windows) && relative_path.contains(':')) {
            return create_error_response(StatusCode(400), "Bad Request");
        }
        // Decode URL-encoded path components (each segment separately)
        let mut decoded_segments = Vec::new();
        for segment in relative_path.split('/') {
            match decode(segment) {
                Ok(decoded) => {
                    // Security: Reject traversal after URL decoding (%2e%2e bypass)
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

    // Check if the path exists and is within root_path
    let normalized_root = match active_root_path.canonicalize() {
        Ok(p) => p,
        Err(_) => active_root_path.clone(),
    };

    let normalized_path = match file_path.canonicalize() {
        Ok(p) => {
            if !p.starts_with(&normalized_root) {
                return create_error_response(StatusCode(404), "Not Found");
            }
            p
        }
        Err(_) => {
            // canonicalize() failed, check if file_path exists
            // Special case: if url_path is "/", check for index.html first
            if url_path == "/" {
                let index_path = active_root_path.join("index.html");
                if index_path.exists() && index_path.is_file() {
                    return create_file_response(
                        &index_path,
                        allow_html_in_md,
                        serve_raw_markdown,
                        &raw_toggle_href,
                    );
                }
                // If index.html doesn't exist, show directory listing
                if active_root_path.exists() && active_root_path.is_dir() {
                    return create_directory_listing(
                        active_root_path.as_path(),
                        public_url_path.as_str(),
                    );
                }
                return create_error_response(StatusCode(404), "Not Found");
            }
            // Check if it's a file
            if file_path.exists() && file_path.is_file() {
                // Security: Verify file is within root_path
                if !file_path.starts_with(active_root_path.as_path()) {
                    return create_error_response(StatusCode(404), "Not Found");
                }
                return create_file_response(
                    &file_path,
                    allow_html_in_md,
                    serve_raw_markdown,
                    &raw_toggle_href,
                );
            }
            // Check if it's a directory request
            if file_path.exists() && file_path.is_dir() {
                // Security: Verify directory is within root_path
                if !file_path.starts_with(active_root_path.as_path()) {
                    return create_error_response(StatusCode(404), "Not Found");
                }
                // Check for index.html in the directory
                let index_path = file_path.join("index.html");
                if index_path.exists() && index_path.is_file() {
                    return create_file_response(
                        &index_path,
                        allow_html_in_md,
                        serve_raw_markdown,
                        &raw_toggle_href,
                    );
                }
                // Generate directory listing
                return create_directory_listing(&file_path, public_url_path.as_str());
            }
            return create_error_response(StatusCode(404), "Not Found");
        }
    };

    // If the normalized path is a directory, check for index.html
    if normalized_path.is_dir() {
        let index_path = normalized_path.join("index.html");
        if index_path.exists() {
            return create_file_response(
                &index_path,
                allow_html_in_md,
                serve_raw_markdown,
                &raw_toggle_href,
            );
        }
        // Generate directory listing
        return create_directory_listing(&normalized_path, public_url_path.as_str());
    }

    // It's a file, serve it
    create_file_response(
        &normalized_path,
        allow_html_in_md,
        serve_raw_markdown,
        &raw_toggle_href,
    )
}
