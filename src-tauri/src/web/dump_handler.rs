use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tiny_http::{Response, StatusCode, Header};

use super::common::create_error_response;

#[derive(Serialize, Deserialize)]
pub struct DumpResponse {
    pub method: String,
    pub path: String,
    pub query: Option<Vec<HashMap<String, String>>>,
    pub headers: Vec<HashMap<String, String>>,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_body: Option<serde_json::Value>,
}

pub fn handle_dump_request(request: &mut tiny_http::Request) -> Response<std::io::Cursor<Vec<u8>>> {
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
