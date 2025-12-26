use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, thread, collections::HashMap};
use tiny_http::{Server, Response, StatusCode, Header};

use crate::config::{AppConfig, get_config_app_path};
use crate::util::open_with_system_command;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebConfig {
    pub root: String,
    #[serde(default = "df_web_port")]
    pub port: u16,
    #[serde(default = "df_open_browser_at_start")]
    pub open_browser_at_start: bool,
    #[serde(default = "df_dump")]
    pub dump: bool,
}

pub struct WebServerConfig {
    pub root: String,
    pub port: u16,
    pub open_browser_at_start: bool,
    pub dump: bool,
}

fn df_web_port() -> u16 { 3030 }
fn df_open_browser_at_start() -> bool { false }
fn df_dump() -> bool { false }

fn create_error_response(status_code: StatusCode, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(message).with_status_code(status_code)
}

fn parse_and_validate_path(url: &str, root_path: &PathBuf) -> Result<PathBuf, StatusCode> {
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

#[derive(Serialize, Deserialize)]
struct DumpResponse {
    method: String,
    path: String,
    query: Option<Vec<HashMap<String, String>>>,
    headers: Vec<HashMap<String, String>>,
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parsed_body: Option<serde_json::Value>,
}

fn handle_dump_request(request: &mut tiny_http::Request) -> Response<std::io::Cursor<Vec<u8>>> {
    let method = request.method().to_string();
    let full_url = request.url();

    // Extract path and query string
    let (full_path, query_string) = match full_url.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (full_url, None),
    };

    // Extract path part after /dump or /dump/
    let path = if full_path.starts_with("/dump/") {
        format!("/{}", &full_path[6..])
    } else {
        "/".to_string()
    };

    // Parse query string into key-value pairs as array of objects (order preserved)
    let query = query_string.map(|qs| {
        qs.split('&')
            .filter_map(|param| {
                if param.is_empty() {
                    None
                } else {
                    let mut map = HashMap::new();
                    match param.split_once('=') {
                        Some((key, value)) => {
                            map.insert(key.to_string(), value.to_string());
                            Some(map)
                        }
                        None => {
                            map.insert(param.to_string(), String::new());
                            Some(map)
                        }
                    }
                }
            })
            .collect()
    });

    // Collect headers as array of objects (order preserved)
    let headers: Vec<HashMap<String, String>> = request.headers()
        .iter()
        .filter_map(|header| {
            if let Ok(value) = std::str::from_utf8(header.value.as_bytes()) {
                let mut map = HashMap::new();
                map.insert(header.field.to_string(), value.to_string());
                Some(map)
            } else {
                None
            }
        })
        .collect();

    // Read body if present
    let mut body_content = Vec::new();
    let body = if request.as_reader().read_to_end(&mut body_content).is_ok() && !body_content.is_empty() {
        String::from_utf8(body_content).ok()
    } else {
        None
    };

    // Check if Content-Type indicates JSON and parse body if so
    let parsed_body = if let Some(ref body_str) = body {
        let is_json = headers.iter().any(|header_map| {
            header_map.iter().any(|(key, value)| {
                key.eq_ignore_ascii_case("content-type") && value.contains("json")
            })
        });
        if is_json {
            match serde_json::from_str(body_str) {
                Ok(value) => Some(value),
                Err(e) => {
                    Some(serde_json::Value::String(format!("ERROR: Failed to parse JSON body: {}", e)))
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let dump_data = DumpResponse {
        method,
        path,
        query,
        headers,
        body,
        parsed_body,
    };

    match serde_json::to_string_pretty(&dump_data) {
        Ok(json) => {
            if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], b"application/json") {
                Response::from_string(json).with_header(header).with_status_code(StatusCode(200))
            } else {
                Response::from_string(json).with_status_code(StatusCode(200))
            }
        }
        Err(_) => create_error_response(StatusCode(500), "Internal Server Error"),
    }
}

fn handle_web_request(request: &mut tiny_http::Request, root_path: &PathBuf, dump_enabled: bool) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    // Check if this is a /dump request (including /dump/ and any subpaths)
    if dump_enabled {
        let path = url.split('?').next().unwrap_or("/");
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

pub fn start_web_server(root: String, port: u16, dump_enabled: bool) {
    thread::spawn(move || {
        let server = match Server::http(format!("127.0.0.1:{}", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to start web server on port {}: {}", port, e);
                return;
            }
        };

        let root_path = PathBuf::from(root);
        if !root_path.exists() {
            eprintln!("Web root path does not exist: {}", root_path.display());
            return;
        }

        println!("Web server started on http://localhost:{}", port);

        for mut request in server.incoming_requests() {
            let response = handle_web_request(&mut request, &root_path, dump_enabled);
            if let Err(e) = request.respond(response) {
                eprintln!("Failed to send response: {}", e);
            }
        }
    });
}

fn get_content_type(path: &PathBuf) -> String {
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

pub fn open_url_in_browser(url: &str) -> Result<(), String> {
    open_with_system_command(url, "Failed to open URL in browser")
}

pub fn load_web_config(identifier: &String) -> Result<Option<WebServerConfig>, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(identifier));

    if !config_path.exists() {
        return Ok(None);
    }

    let config_json = fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;
    let config: AppConfig = serde_json::from_str(&config_json)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let web_config = match config.web {
        Some(wc) => wc,
        None => return Ok(None),
    };

    let root_path = PathBuf::from(&web_config.root);
    if !root_path.exists() {
        return Err(format!("web.root not exists: {}", root_path.display()));
    }

    Ok(Some(WebServerConfig {
        root: web_config.root,
        port: web_config.port,
        open_browser_at_start: web_config.open_browser_at_start,
        dump: web_config.dump,
    }))
}

