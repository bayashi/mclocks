use tiny_http::{Response, StatusCode, Header};
use std::io::Cursor;

pub fn get_status_phrase(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}

pub fn should_have_response_body(code: u16) -> bool {
    match code {
        204 | 304 => false,
        _ => true,
    }
}

pub fn apply_status_headers(
    mut response: Response<Cursor<Vec<u8>>>,
    status_code: u16,
    _status: StatusCode,
    _has_body: bool,
) -> Response<Cursor<Vec<u8>>> {
    // Helper function to add header
    let add_header = |response: Response<Cursor<Vec<u8>>>, name: &[u8], value: &[u8]| -> Response<Cursor<Vec<u8>>> {
        if let Ok(header) = Header::from_bytes(name, value) {
            response.with_header(header)
        } else {
            response
        }
    };

    // Add status-specific headers
    match status_code {
        // 3xx Redirection - Location header required
        301 | 302 | 303 | 305 | 307 | 308 => {
            response = add_header(response, b"Location", b"/");
        },
        // 401 Unauthorized - WWW-Authenticate required
        401 => {
            response = add_header(response, b"WWW-Authenticate", b"Basic realm=\"test\"");
        },
        // 402 Payment Required - no special header
        402 => {},
        // 405 Method Not Allowed - Allow header required
        405 => {
            response = add_header(response, b"Allow", b"GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS");
        },
        // 407 Proxy Authentication Required - Proxy-Authenticate required
        407 => {
            response = add_header(response, b"Proxy-Authenticate", b"Basic realm=\"proxy\"");
        },
        // 408 Request Timeout - no special header
        408 => {},
        // 409 Conflict - no special header
        409 => {},
        // 410 Gone - no special header
        410 => {},
        // 411 Length Required - no special header
        411 => {},
        // 412 Precondition Failed - no special header
        412 => {},
        // 413 Payload Too Large - no special header
        413 => {},
        // 414 URI Too Long - no special header
        414 => {},
        // 415 Unsupported Media Type - no special header
        415 => {},
        // 416 Range Not Satisfiable - Content-Range required
        416 => {
            response = add_header(response, b"Content-Range", b"bytes */0");
        },
        // 417 Expectation Failed - no special header
        417 => {},
        // 418 I'm a teapot - special body already set in handle_status_request
        418 => {},
        // 421 Misdirected Request - no special header
        421 => {},
        // 422 Unprocessable Entity - no special header
        422 => {},
        // 423 Locked - no special header
        423 => {},
        // 424 Failed Dependency - no special header
        424 => {},
        // 425 Too Early - no special header
        425 => {},
        // 426 Upgrade Required - Upgrade header required
        426 => {
            response = add_header(response, b"Upgrade", b"TLS/1.0, HTTP/1.1");
        },
        // 428 Precondition Required - no special header
        428 => {},
        // 429 Too Many Requests - Retry-After recommended
        429 => {
            response = add_header(response, b"Retry-After", b"60");
        },
        // 431 Request Header Fields Too Large - no special header
        431 => {},
        // 451 Unavailable For Legal Reasons - no special header
        451 => {},
        // 500 Internal Server Error - no special header
        500 => {},
        // 501 Not Implemented - no special header
        501 => {},
        // 502 Bad Gateway - no special header
        502 => {},
        // 503 Service Unavailable - Retry-After recommended
        503 => {
            response = add_header(response, b"Retry-After", b"60");
        },
        // 504 Gateway Timeout - no special header
        504 => {},
        // 505 HTTP Version Not Supported - no special header
        505 => {},
        // 506 Variant Also Negotiates - no special header
        506 => {},
        // 507 Insufficient Storage - no special header
        507 => {},
        // 508 Loop Detected - no special header
        508 => {},
        // 510 Not Extended - no special header
        510 => {},
        // 511 Network Authentication Required - WWW-Authenticate required
        511 => {
            response = add_header(response, b"WWW-Authenticate", b"Basic realm=\"network\"");
        },
        // 1xx Informational
        100..=199 => {},
        // 2xx Success
        200..=299 => {},
        // 4xx Client Error (already handled above or default)
        400..=499 => {},
        // 5xx Server Error (already handled above or default)
        500..=599 => {},
        _ => {},
    }

    response
}

