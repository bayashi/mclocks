use tiny_http::{Response, StatusCode, Header};

use crate::web::status_code::{get_status_phrase, should_have_response_body, apply_status_headers};
use super::common::create_error_response;

pub fn handle_status_request(_request: &tiny_http::Request, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    // Extract status code from path: /status/{code}
    if !path.starts_with("/status/") {
        return create_error_response(StatusCode(404), "Not Found");
    }

    let code_str = &path[8..]; // Skip "/status/"
    let code_str = code_str.split('/').next().unwrap_or(code_str); // Get only the first segment

    let status_code = match code_str.parse::<u16>() {
        Ok(code) if (100..=599).contains(&code) => code,
        _ => {
            return create_error_response(StatusCode(400), "Invalid status code");
        }
    };

    let status = StatusCode(status_code);

    // Handle status codes that don't allow response body
    let has_body = should_have_response_body(status_code);

    // Special handling for 418 I'm a teapot
    let body_content = if status_code == 418 {
        "I'm a teapot".to_string()
    } else if has_body {
        format!("{} {}", status_code, get_status_phrase(status_code))
    } else {
        String::new()
    };

    let response = if has_body {
        let mut resp = Response::from_string(body_content).with_status_code(status);
        // Helper function to add header
        let add_header = |response: Response<std::io::Cursor<Vec<u8>>>, name: &[u8], value: &[u8]| -> Response<std::io::Cursor<Vec<u8>>> {
            if let Ok(header) = Header::from_bytes(name, value) {
                response.with_header(header)
            } else {
                response
            }
        };
        resp = add_header(resp, b"Content-Type", b"text/plain; charset=utf-8");
        resp
    } else {
        // For 204 No Content and 304 Not Modified, use empty data
        Response::from_data(Vec::<u8>::new()).with_status_code(status)
    };

    // Apply status-specific headers
    apply_status_headers(response, status_code, status, has_body)
}
