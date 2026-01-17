use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, thread};
use tiny_http::Server;

use crate::config::{AppConfig, get_config_app_path};
use crate::util::open_with_system_command;
use crate::web::file::handle_web_request;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebConfig {
    pub root: String,
    #[serde(default = "df_web_port")]
    pub port: u16,
    #[serde(default = "df_open_browser_at_start")]
    pub open_browser_at_start: bool,
    #[serde(default = "df_dump")]
    pub dump: bool,
    #[serde(default = "df_slow")]
    pub slow: bool,
    #[serde(default = "df_status")]
    pub status: bool,
}

#[derive(Debug)]
pub struct WebServerConfig {
    pub root: String,
    pub port: u16,
    pub open_browser_at_start: bool,
    pub dump: bool,
    pub slow: bool,
    pub status: bool,
}

fn df_web_port() -> u16 { 3030 }
fn df_open_browser_at_start() -> bool { false }
fn df_dump() -> bool { false }
fn df_slow() -> bool { false }
fn df_status() -> bool { false }

pub fn start_web_server(root: String, port: u16, dump_enabled: bool, slow_enabled: bool, status_enabled: bool) {
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
            let response = handle_web_request(&mut request, &root_path, dump_enabled, slow_enabled, status_enabled);
            if let Err(e) = request.respond(response) {
                eprintln!("Failed to send response: {}", e);
            }
        }
    });
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
        slow: web_config.slow,
        status: web_config.status,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use tiny_http::Server;
    use crate::web::file::{handle_web_request, get_content_type};

    #[test]
    fn test_handle_web_request_root_with_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let index_file = root_path.join("index.html");
        fs::write(&index_file, "<html>test</html>").expect("Failed to create index.html");
        let port = 3053;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "<html>test</html>");
    }

    #[test]
    fn test_handle_web_request_root_without_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        // Create some files and directories
        fs::write(root_path.join("file1.txt"), "content1").expect("Failed to create file1.txt");
        fs::write(root_path.join("file2.html"), "<html>content2</html>").expect("Failed to create file2.html");
        fs::create_dir_all(root_path.join("subdir")).expect("Failed to create subdir");
        let port = 3054;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("Index of /"), "Should show directory listing");
        assert!(body.contains("file1.txt"), "Should list file1.txt");
        assert!(body.contains("file2.html"), "Should list file2.html");
        assert!(body.contains("subdir/"), "Should list subdir");
    }

    #[test]
    fn test_handle_web_request_directory_with_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let subdir = root_path.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdir");
        let index_file = subdir.join("index.html");
        fs::write(&index_file, "<html>subdir index</html>").expect("Failed to create index.html");
        let port = 3055;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/subdir/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "<html>subdir index</html>");
    }

    #[test]
    fn test_handle_web_request_directory_without_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let subdir = root_path.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdir");
        fs::write(subdir.join("file.txt"), "content").expect("Failed to create file.txt");
        let port = 3056;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/subdir/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("Index of /subdir/"), "Should show directory listing");
        assert!(body.contains("file.txt"), "Should list file.txt");
        assert!(body.contains("../"), "Should show parent directory link");
    }

    #[test]
    fn test_handle_web_request_directory_traversal() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3057;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();

        // Test double slash which should be rejected
        // Note: HTTP clients normalize /../ to /, so we can't test that directly
        // But we can test double slash which is also a security concern
        let response = client.get(&format!("http://127.0.0.1:{}/test//file.html", port))
            .send()
            .expect("Failed to send request");
        // The client might normalize // to /, so we check for either 400 or 404
        assert!(response.status() == 400 || response.status() == 404,
                "Double slash should be rejected or not found");
    }

    #[test]
    fn test_handle_web_request_double_slash() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3058;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/test//file.html", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 400);
    }

    #[test]
    fn test_handle_web_request_normal_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let test_file = root_path.join("test.html");
        fs::write(&test_file, "<html>test</html>").expect("Failed to create test.html");
        let port = 3059;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/test.html", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "<html>test</html>");
    }

    #[test]
    fn test_handle_web_request_with_query_string() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let test_file = root_path.join("test.html");
        fs::write(&test_file, "<html>test</html>").expect("Failed to create test.html");
        let port = 3060;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/test.html?key=value", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "<html>test</html>");
    }

    #[test]
    fn test_handle_web_request_nonexistent_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3061;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/nonexistent.html", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_get_content_type_html() {
        let path = PathBuf::from("test.html");
        assert_eq!(get_content_type(&path), "text/html");
    }

    #[test]
    fn test_get_content_type_css() {
        let path = PathBuf::from("style.css");
        assert_eq!(get_content_type(&path), "text/css");
    }

    #[test]
    fn test_get_content_type_js() {
        let path = PathBuf::from("script.js");
        assert_eq!(get_content_type(&path), "application/javascript");
    }

    #[test]
    fn test_get_content_type_json() {
        let path = PathBuf::from("data.json");
        assert_eq!(get_content_type(&path), "application/json");
    }

    #[test]
    fn test_get_content_type_md() {
        let path = PathBuf::from("readme.md");
        assert_eq!(get_content_type(&path), "text/markdown");
    }

    #[test]
    fn test_get_content_type_png() {
        let path = PathBuf::from("image.png");
        assert_eq!(get_content_type(&path), "image/png");
    }

    #[test]
    fn test_get_content_type_jpg() {
        let path = PathBuf::from("photo.jpg");
        assert_eq!(get_content_type(&path), "image/jpeg");
    }

    #[test]
    fn test_get_content_type_jpeg() {
        let path = PathBuf::from("photo.jpeg");
        assert_eq!(get_content_type(&path), "image/jpeg");
    }

    #[test]
    fn test_get_content_type_gif() {
        let path = PathBuf::from("animation.gif");
        assert_eq!(get_content_type(&path), "image/gif");
    }

    #[test]
    fn test_get_content_type_svg() {
        let path = PathBuf::from("icon.svg");
        assert_eq!(get_content_type(&path), "image/svg+xml");
    }

    #[test]
    fn test_get_content_type_ico() {
        let path = PathBuf::from("favicon.ico");
        assert_eq!(get_content_type(&path), "image/x-icon");
    }

    #[test]
    fn test_get_content_type_txt() {
        let path = PathBuf::from("readme.txt");
        assert_eq!(get_content_type(&path), "text/plain");
    }

    #[test]
    fn test_get_content_type_unknown() {
        let path = PathBuf::from("file.unknown");
        assert_eq!(get_content_type(&path), "application/octet-stream");
    }

    #[test]
    fn test_get_content_type_no_extension() {
        let path = PathBuf::from("file");
        assert_eq!(get_content_type(&path), "application/octet-stream");
    }

    #[test]
    fn test_get_content_type_subdirectory() {
        let path = PathBuf::from("subdir/file.html");
        assert_eq!(get_content_type(&path), "text/html");
    }

    fn start_test_server(root: PathBuf, port: u16, dump_enabled: bool, slow_enabled: bool, status_enabled: bool) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            let server = match Server::http(format!("127.0.0.1:{}", port)) {
                Ok(s) => s,
                Err(_) => return,
            };

            for mut request in server.incoming_requests() {
                let response = handle_web_request(&mut request, &root, dump_enabled, slow_enabled, status_enabled);
                let _ = request.respond(response);
            }
        })
    }

    #[test]
    fn test_handle_dump_request_basic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3031;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/dump", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-type").unwrap().to_str().unwrap(), "application/json");

        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["method"], "GET");
        assert_eq!(json["path"], "/");
        assert!(json["query"].is_null());
    }

    #[test]
    fn test_handle_dump_request_with_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3032;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/dump/test/path", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["method"], "GET");
        assert_eq!(json["path"], "/test/path");
    }

    #[test]
    fn test_handle_dump_request_with_query_string() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3033;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/dump?key1=value1&key2=value2", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["method"], "GET");
        assert_eq!(json["path"], "/");

        let query = json["query"].as_array().expect("Query should be an array");
        assert_eq!(query.len(), 2);
    }

    #[test]
    fn test_handle_dump_request_with_headers() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3034;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/dump", port))
            .header("X-Custom-Header", "custom-value")
            .header("User-Agent", "test-agent")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");

        let headers = json["headers"].as_array().expect("Headers should be an array");
        assert!(headers.len() > 0);
    }

    #[test]
    fn test_handle_dump_request_with_json_body() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3035;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let body = serde_json::json!({"test": "value"});
        let response = client.post(&format!("http://127.0.0.1:{}/dump", port))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["method"], "POST");

        let body_str = json["body"].as_str().expect("Body should be a string");
        assert!(body_str.contains("test"));
        assert!(body_str.contains("value"));

        let parsed_body = json["parsed_body"].as_object().expect("Parsed body should be an object");
        assert_eq!(parsed_body["test"], "value");
    }

    #[test]
    fn test_handle_dump_request_with_invalid_json_body() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3036;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.post(&format!("http://127.0.0.1:{}/dump", port))
            .header("Content-Type", "application/json")
            .body("{ invalid json }")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        let parsed_body = json["parsed_body"].as_str().expect("Parsed body should be error string");
        assert!(parsed_body.contains("ERROR"));
    }

    #[test]
    fn test_handle_dump_request_with_text_body() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3037;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.post(&format!("http://127.0.0.1:{}/dump", port))
            .header("Content-Type", "text/plain")
            .body("plain text body")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["method"], "POST");
        assert_eq!(json["body"], "plain text body");
        assert!(json["parsed_body"].is_null());
    }

    #[test]
    fn test_handle_slow_request_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3038;

        let _server_handle = start_test_server(root_path, port, false, true, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let start = std::time::Instant::now();
        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/slow/3", port))
            .send()
            .expect("Failed to send request");
        let elapsed = start.elapsed();

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "OK");
        // Should wait approximately 3 seconds (allow some tolerance)
        assert!(elapsed.as_secs() >= 3, "Should wait at least 3 seconds");
        assert!(elapsed.as_secs() < 5, "Should not wait more than 5 seconds");
    }

    #[test]
    fn test_handle_slow_request_with_seconds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3039;

        let _server_handle = start_test_server(root_path, port, false, true, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let start = std::time::Instant::now();
        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/slow/5", port))
            .send()
            .expect("Failed to send request");
        let elapsed = start.elapsed();

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "OK");
        // Should wait approximately 5 seconds (allow some tolerance)
        assert!(elapsed.as_secs() >= 5, "Should wait at least 5 seconds");
        assert!(elapsed.as_secs() < 7, "Should not wait more than 7 seconds");
    }

    #[test]
    fn test_handle_slow_request_post_method() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3040;

        let _server_handle = start_test_server(root_path, port, false, true, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let start = std::time::Instant::now();
        let client = reqwest::blocking::Client::new();
        let response = client.post(&format!("http://127.0.0.1:{}/slow/3", port))
            .send()
            .expect("Failed to send request");
        let elapsed = start.elapsed();

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "OK");
        // Should wait approximately 3 seconds (allow some tolerance)
        assert!(elapsed.as_secs() >= 3, "Should wait at least 3 seconds");
        assert!(elapsed.as_secs() < 5, "Should not wait more than 5 seconds");
    }

    #[test]
    fn test_handle_slow_request_invalid_seconds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3041;

        let _server_handle = start_test_server(root_path, port, false, true, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/slow/abc", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 400);
        assert_eq!(response.text().unwrap(), "Invalid seconds parameter");
    }

    #[test]
    fn test_handle_slow_request_disabled() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let index_file = root_path.join("index.html");
        fs::write(&index_file, "test").expect("Failed to create index.html");
        let port = 3042;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/slow/3", port))
            .send()
            .expect("Failed to send request");

        // When slow_enabled is false, /slow should not be handled and should fall through to file serving
        // Since there's no /slow/3 file, it should return 404
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_load_web_config_no_config_file() {
        let identifier = "test.app.nonexistent".to_string();
        let result = load_web_config(&identifier);
        assert!(result.is_ok(), "Should return Ok when config file doesn't exist");
        assert!(result.unwrap().is_none(), "Should return None when config file doesn't exist");
    }

    #[test]
    fn test_load_web_config_no_web_field() {
        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.noweb");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        // Use get_config_app_path to get the correct config file name
        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        let config_json = r#"{"clocks": [{"name": "UTC"}]}"#;
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.noweb".to_string();
        let result = load_web_config(&identifier);
        assert!(result.is_ok(), "Should return Ok when web field is missing");
        assert!(result.unwrap().is_none(), "Should return None when web field is missing");

        // Cleanup
        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_load_web_config_nonexistent_root() {
        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.badroot");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        // Use get_config_app_path to get the correct config file name
        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        // Use a path that definitely doesn't exist (use forward slashes for JSON)
        let nonexistent_path = if cfg!(windows) {
            "C:/nonexistent/path/that/does/not/exist"
        } else {
            "/nonexistent/path/that/does/not/exist"
        };
        let config_json = format!(r#"{{
            "web": {{
                "root": "{}"
            }}
        }}"#, nonexistent_path);
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.badroot".to_string();
        let result = load_web_config(&identifier);
        assert!(result.is_err(), "Should return error when root path doesn't exist. Got: {:?}", result);
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("web.root not exists") || err_msg.contains("not exists"),
                "Error message should indicate root doesn't exist. Got: {}", err_msg);

        // Cleanup
        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_load_web_config_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let web_root = temp_dir.path().join("webroot");
        fs::create_dir_all(&web_root).expect("Failed to create web root");

        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.success");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        // Use get_config_app_path to get the correct config file name
        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        // Use absolute path for web.root, escape backslashes for JSON
        let web_root_str = web_root.canonicalize().unwrap().to_string_lossy().replace('\\', "/");
        let config_json = format!(r#"{{
            "web": {{
                "root": "{}",
                "port": 8080,
                "openBrowserAtStart": true,
                "dump": true
            }}
        }}"#, web_root_str);
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.success".to_string();
        let result = load_web_config(&identifier);
        assert!(result.is_ok(), "Should successfully load web config. Got: {:?}", result);
        let web_config = result.unwrap();
        assert!(web_config.is_some(), "Should return Some(WebServerConfig). Got: {:?}", web_config);
        let config = web_config.unwrap();
        assert_eq!(config.port, 8080, "Port should match");
        assert_eq!(config.open_browser_at_start, true, "open_browser_at_start should be true");
        assert_eq!(config.dump, true, "dump should be true");

        // Cleanup
        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_load_web_config_default_values() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let web_root = temp_dir.path().join("webroot");
        fs::create_dir_all(&web_root).expect("Failed to create web root");

        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.defaults");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        // Use get_config_app_path to get the correct config file name
        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        // Use absolute path for web.root, escape backslashes for JSON
        let web_root_str = web_root.canonicalize().unwrap().to_string_lossy().replace('\\', "/");
        let config_json = format!(r#"{{
            "web": {{
                "root": "{}"
            }}
        }}"#, web_root_str);
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.defaults".to_string();
        let result = load_web_config(&identifier);
        assert!(result.is_ok(), "Should successfully load web config with defaults. Got: {:?}", result);
        let web_config = result.unwrap();
        assert!(web_config.is_some(), "Should return Some(WebServerConfig). Got: {:?}", web_config);
        let config = web_config.unwrap();
        assert_eq!(config.port, 3030, "Default port should be 3030");
        assert_eq!(config.open_browser_at_start, false, "Default open_browser_at_start should be false");
        assert_eq!(config.dump, false, "Default dump should be false");

        // Cleanup
        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_web_config_deserialize_empty() {
        // root is required, so we need at least that field
        let json = r#"{"root": "/test"}"#;
        let result: Result<WebConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize JSON with only root");
        let config = result.unwrap();

        // Check default values
        assert_eq!(config.root, "/test", "Root should match");
        assert_eq!(config.port, 3030, "Default port should be 3030");
        assert_eq!(config.open_browser_at_start, false, "Default open_browser_at_start should be false");
        assert_eq!(config.dump, false, "Default dump should be false");
    }

    #[test]
    fn test_web_config_deserialize_partial() {
        let json = r#"{"root": "/path/to/root", "port": 8080}"#;
        let result: Result<WebConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize partial JSON");
        let config = result.unwrap();

        // Check specified values
        assert_eq!(config.root, "/path/to/root", "Root should match");
        assert_eq!(config.port, 8080, "Port should match");

        // Check default values are still applied
        assert_eq!(config.open_browser_at_start, false, "Default open_browser_at_start should still apply");
        assert_eq!(config.dump, false, "Default dump should still apply");
    }

    #[test]
    fn test_web_config_deserialize_full() {
        let json = r#"{
            "root": "/path/to/root",
            "port": 8080,
            "openBrowserAtStart": true,
            "dump": true
        }"#;
        let result: Result<WebConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize full JSON");
        let config = result.unwrap();

        assert_eq!(config.root, "/path/to/root", "Root should match");
        assert_eq!(config.port, 8080, "Port should match");
        assert_eq!(config.open_browser_at_start, true, "open_browser_at_start should be true");
        assert_eq!(config.dump, true, "dump should be true");
    }

    #[test]
    fn test_web_config_deserialize_camel_case() {
        let json = r#"{
            "root": "/test",
            "openBrowserAtStart": true
        }"#;
        let result: Result<WebConfig, _> = serde_json::from_str(json);

        assert!(result.is_ok(), "Should deserialize camelCase field names");
        let config = result.unwrap();
        assert_eq!(config.root, "/test", "Root should match");
        assert_eq!(config.open_browser_at_start, true, "openBrowserAtStart should map to open_browser_at_start");
    }

    #[test]
    fn test_web_config_deserialize_invalid_json() {
        let json = "{ invalid json }";
        let result: Result<WebConfig, _> = serde_json::from_str(json);

        assert!(result.is_err(), "Should fail to deserialize invalid JSON");
    }

    #[test]
    fn test_handle_status_request_200() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3043;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/200", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "200 OK");
    }

    #[test]
    fn test_handle_status_request_418_im_a_teapot() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3044;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/418", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 418);
        assert_eq!(response.text().unwrap(), "I'm a teapot");
    }

    #[test]
    fn test_handle_status_request_204_no_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3045;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/204", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 204);
        assert_eq!(response.text().unwrap(), "");
    }

    #[test]
    fn test_handle_status_request_301_moved_permanently() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3046;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to create client");
        let response = client.get(&format!("http://127.0.0.1:{}/status/301", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 301);
        assert!(response.headers().contains_key("location"));
        assert_eq!(response.headers().get("location").unwrap().to_str().unwrap(), "/");
    }

    #[test]
    fn test_handle_status_request_401_unauthorized() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3047;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/401", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 401);
        assert!(response.headers().contains_key("www-authenticate"));
        assert_eq!(response.headers().get("www-authenticate").unwrap().to_str().unwrap(), "Basic realm=\"test\"");
    }

    #[test]
    fn test_handle_status_request_405_method_not_allowed() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3048;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/405", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 405);
        assert!(response.headers().contains_key("allow"));
        assert_eq!(response.headers().get("allow").unwrap().to_str().unwrap(), "GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS");
    }

    #[test]
    fn test_handle_status_request_invalid_status_code_too_low() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3049;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/99", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 400);
        assert_eq!(response.text().unwrap(), "Invalid status code");
    }

    #[test]
    fn test_handle_status_request_invalid_status_code_too_high() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3050;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/600", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 400);
        assert_eq!(response.text().unwrap(), "Invalid status code");
    }

    #[test]
    fn test_handle_status_request_disabled() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let index_file = root_path.join("index.html");
        fs::write(&index_file, "test").expect("Failed to create index.html");
        let port = 3051;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/200", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_handle_status_request_500_internal_server_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3052;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/500", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 500);
        assert_eq!(response.text().unwrap(), "500 Internal Server Error");
    }

    #[test]
    fn test_handle_status_request_429_too_many_requests() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3053;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client.get(&format!("http://127.0.0.1:{}/status/429", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 429);
        assert!(response.headers().contains_key("retry-after"));
        assert_eq!(response.headers().get("retry-after").unwrap().to_str().unwrap(), "60");
    }
}
