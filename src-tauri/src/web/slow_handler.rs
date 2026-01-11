use std::time::Duration;
use tiny_http::{Response, StatusCode};
use std::thread;

use super::common::create_error_response;

pub fn handle_slow_request(request: &tiny_http::Request) -> Response<std::io::Cursor<Vec<u8>>> {
    let url = request.url();
    let path = url.split('?').next().unwrap_or("/");

    // Extract seconds from path: /slow or /slow/120
    // Note: This function is only called when path == "/slow" || path.starts_with("/slow/")
    let seconds = if path == "/slow" {
        30u64
    } else {
        // path.starts_with("/slow/") is guaranteed here
        let after_slow = &path[6..];
        // Extract the first segment (before next / if exists)
        let seconds_str = match after_slow.split('/').next() {
            Some(s) => s,
            None => after_slow,
        };
        match seconds_str.parse::<u64>() {
            Ok(secs) => secs,
            Err(_) => {
                return create_error_response(StatusCode(400), "Invalid seconds parameter");
            }
        }
    };

    thread::sleep(Duration::from_secs(seconds));
    Response::from_string("OK").with_status_code(StatusCode(200))
}
