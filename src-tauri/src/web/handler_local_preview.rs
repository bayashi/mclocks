//! POST /preview — opens a file in the content viewer. Uses main web `127.0.0.1:port` (default 3030).
//! Body: line 1 = path to a previewable file (Markdown, JSON, YAML, TOML, INI, XML, …). Optional line 2 = `PWD` for a **relative** line 1; absolute line 1 ignores line 2.
//! Success: `200` and `text/plain` body is `OK` (preview URL is not returned).
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tiny_http::{Header, Request, Response, StatusCode};

use super::common::create_error_response;
use super::dd_publish::{build_temp_file_url, register_temp_file};
use crate::web_server::open_url_in_browser;

const MAX_BODY_BYTES: usize = 64 * 1024;

const PREVIEW_SUCCESS_BODY: &str = "OK";

/// Request-target may be origin-form (`/preview`) or absolute-form (`http://host:port/preview`).
fn path_only_from_request_target(raw: &str) -> &str {
    let no_query = raw.split('?').next().unwrap_or(raw);
    if let Some(rest) = no_query.strip_prefix("http://") {
        if let Some(i) = rest.find('/') {
            return &rest[i..];
        }
        return "/";
    }
    if let Some(rest) = no_query.strip_prefix("https://") {
        if let Some(i) = rest.find('/') {
            return &rest[i..];
        }
        return "/";
    }
    no_query
}

fn is_preview_request_path(path_only: &str) -> bool {
    let p = path_only.trim_end_matches('/');
    p.eq_ignore_ascii_case("/preview")
}

/// True when this request targets `/preview` with POST or OPTIONS (CORS preflight).
pub(crate) fn is_preview_route_request(request: &Request) -> bool {
    let m = request.method().as_str();
    if m != "POST" && m != "OPTIONS" {
        return false;
    }
    let path_only = path_only_from_request_target(request.url());
    is_preview_request_path(path_only)
}

/// Resolves the file path. Relative paths use `cwd_line` when set, otherwise the mclocks process current directory.
fn resolve_preview_file_path(path_line: &str, cwd_line: Option<&str>) -> Result<PathBuf, String> {
    let p = Path::new(path_line);
    if p.is_absolute() {
        return p
            .canonicalize()
            .map_err(|_| "Path could not be resolved".to_string());
    }
    let candidate = if let Some(cwd_s) = cwd_line.map(str::trim).filter(|s| !s.is_empty()) {
        let cwd = Path::new(cwd_s);
        let meta = cwd
            .metadata()
            .map_err(|_| "Second line (cwd): path not found".to_string())?;
        if !meta.is_dir() {
            return Err("Second line (cwd): must be a directory".to_string());
        }
        cwd.join(p)
    } else {
        p.to_path_buf()
    };
    candidate
        .canonicalize()
        .map_err(|_| "Path could not be resolved".to_string())
}

/// Extensions the web viewer can open in “source” preview (see `handler_static` markdown + structured data).
fn is_allowed_preview_path(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|e| {
            let e = e.to_ascii_lowercase();
            matches!(
                e.as_str(),
                "md" | "markdown"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "xml"
                    | "ini"
                    | "config"
                    | "cfg"
            )
        })
        .unwrap_or(false)
}

/// Handles `POST /preview` when local preview API is wired for this server instance.
/// Request body: plain text — line 1: path to a supported preview file; optional line 2: cwd for a relative line 1.
/// On success, response body is plain text: `OK` only (no preview URL in the body).
/// Returns `None` if the request path is not `/preview`.
pub fn try_handle_local_preview_request(
    request: &mut Request,
    enabled: &Arc<AtomicBool>,
    server_port: u16,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let url = request.url();
    let path_only = path_only_from_request_target(url);
    if !is_preview_request_path(path_only) {
        return None;
    }

    let method = request.method().as_str();
    if method == "OPTIONS" {
        let mut r = Response::from_string("");
        r = r.with_status_code(StatusCode(204));
        if let Ok(h) = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]) {
            r = r.with_header(h);
        }
        if let Ok(h) =
            Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, OPTIONS"[..])
        {
            r = r.with_header(h);
        }
        if let Ok(h) =
            Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..])
        {
            r = r.with_header(h);
        }
        return Some(r);
    }

    if method != "POST" {
        return Some(create_error_response(StatusCode(405), "Method Not Allowed"));
    }

    if !enabled.load(Ordering::SeqCst) {
        return Some(create_error_response(
            StatusCode(403),
            "Local preview API is disabled",
        ));
    }

    let mut buf = Vec::new();
    let reader = request.as_reader();
    if let Err(e) = reader.take(MAX_BODY_BYTES as u64).read_to_end(&mut buf) {
        return Some(create_error_response(
            StatusCode(400),
            &format!("Failed to read body: {}", e),
        ));
    }

    let body_text = String::from_utf8_lossy(&buf).into_owned();
    let mut lines = body_text.lines();
    let path_line = lines.next().unwrap_or("").trim();
    if path_line.is_empty() {
        return Some(create_error_response(
            StatusCode(400),
            "Body must be a non-empty file path (plain text, line 1)",
        ));
    }

    let cwd_line = lines.next().map(str::trim).filter(|s| !s.is_empty());

    let p = Path::new(path_line);
    if !is_allowed_preview_path(p) {
        return Some(create_error_response(
            StatusCode(400),
            "File type not supported for preview (use .md, .json, .yaml, .yml, .toml, .xml, .ini, .config, .cfg, …)",
        ));
    }

    let resolved = match resolve_preview_file_path(path_line, cwd_line) {
        Ok(x) => x,
        Err(msg) => {
            return Some(create_error_response(StatusCode(400), &msg));
        }
    };

    if !resolved.is_file() {
        return Some(create_error_response(
            StatusCode(400),
            "Path is not an existing file",
        ));
    }

    let hash = match register_temp_file(&resolved) {
        Ok(h) => h,
        Err(e) => {
            return Some(create_error_response(StatusCode(500), &e));
        }
    };

    let url = match build_temp_file_url(server_port, &hash, &resolved) {
        Ok(u) => u,
        Err(e) => {
            return Some(create_error_response(StatusCode(500), &e));
        }
    };

    if let Err(e) = open_url_in_browser(&url) {
        return Some(create_error_response(
            StatusCode(500),
            &format!("Failed to open browser: {}", e),
        ));
    }

    let mut r = Response::from_string(PREVIEW_SUCCESS_BODY);
    r = r.with_status_code(StatusCode(200));
    if let Ok(h) = Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]) {
        r = r.with_header(h);
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_relative_with_cwd_line() {
        let tmp = TempDir::new().expect("temp dir");
        let md = tmp.path().join("note.md");
        fs::write(&md, "# x").expect("write md");
        let cwd = tmp.path().to_str().expect("utf8 cwd");
        let got = resolve_preview_file_path("note.md", Some(cwd)).expect("resolve");
        assert_eq!(got, md.canonicalize().expect("canon md"));
    }

    #[test]
    fn resolve_absolute_ignores_cwd_line() {
        let tmp = TempDir::new().expect("temp dir");
        let md = tmp.path().join("x.md");
        fs::write(&md, "# x").expect("write md");
        let abs = md.canonicalize().expect("canon");
        let abs_s = abs.to_str().expect("utf8");
        let got = resolve_preview_file_path(abs_s, Some("/nope/not-a-real-dir-for-cwd"))
            .expect("absolute ignores bogus cwd");
        assert_eq!(got, abs);
    }
}
