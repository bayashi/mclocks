use tiny_http::{Response, StatusCode};

pub fn create_error_response(status_code: StatusCode, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(message).with_status_code(status_code)
}
