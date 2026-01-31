use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tiny_http::{Response, StatusCode, Header, Method};
use serde::Deserialize;
use rand::rngs::OsRng;
use rand::RngCore;

use super::common::create_error_response;

const TOKEN_TTL: Duration = Duration::from_secs(15);
const MAX_TOKENS: usize = 3;
static EDITOR_TOKENS: OnceLock<Mutex<HashMap<String, TokenEntry>>> = OnceLock::new();

struct TokenEntry {
	expires_at: Instant,
	expected_path: String,
}

#[derive(Deserialize)]
struct EditorRequest {
	path: String,
	line: Option<u32>,
	repos_dir: Option<String>,
	token: Option<String>,
}

pub fn handle_editor_request(
	request: &mut tiny_http::Request,
	repos_dir: &Option<String>,
	include_host: bool,
	editor_command: &str,
	editor_args: &[String],
) -> Response<std::io::Cursor<Vec<u8>>> {
	match request.method() {
		Method::Get => handle_get_request(request),
		Method::Post => handle_post_request(request, repos_dir, include_host, editor_command, editor_args),
		_ => create_error_response(StatusCode(405), "Method Not Allowed"),
	}
}

fn handle_get_request(request: &tiny_http::Request) -> Response<std::io::Cursor<Vec<u8>>> {
	let url = request.url();
	let url_path = url.split('?').next().unwrap_or("/");
	let github_path = url_path.strip_prefix("/editor").unwrap_or(url_path);
	let github_path = normalize_github_path_for_token(github_path);
	let token = issue_one_time_token(&github_path);
	let html = generate_editor_html(&github_path, &token);

	if let Ok(header) = Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8") {
		Response::from_string(html).with_header(header).with_status_code(StatusCode(200))
	} else {
		Response::from_string(html).with_status_code(StatusCode(200))
	}
}

fn handle_post_request(
	request: &mut tiny_http::Request,
	repos_dir: &Option<String>,
	include_host: bool,
	editor_command: &str,
	editor_args: &[String],
) -> Response<std::io::Cursor<Vec<u8>>> {
	let mut body = Vec::new();
	if let Err(_) = request.as_reader().read_to_end(&mut body) {
		return create_error_html_response_with_status(StatusCode(400), "Bad Request: Failed to read request body", None, None, None);
	}

	let body_str = match String::from_utf8(body) {
		Ok(s) => s,
		Err(_) => return create_error_html_response_with_status(StatusCode(400), "Bad Request: Invalid UTF-8", None, None, None),
	};

	let params: EditorRequest = match serde_json::from_str(&body_str) {
		Ok(p) => p,
		Err(_) => return create_error_html_response_with_status(StatusCode(400), "Bad Request: Invalid JSON", None, None, None),
	};

	let github_owner_repo = extract_github_owner_repo(&params.path);

	let token = params.token.as_deref().unwrap_or("");
	if token.is_empty() {
		return create_error_html_response_with_status(StatusCode(403), "Forbidden: Missing token. Reload and try again.", github_owner_repo.as_deref(), None, None);
	}
	let normalized_path = normalize_github_path_for_token(&params.path);
	if !consume_one_time_token(token, &normalized_path) {
		return create_error_html_response_with_status(StatusCode(403), "Forbidden: Invalid or expired token. Reload and try again.", github_owner_repo.as_deref(), None, None);
	}

	let local_path = match convert_github_path_to_local(&normalized_path, repos_dir, &params.repos_dir, include_host) {
		Ok(path) => path,
		Err(e) => return create_error_html_response_with_status(StatusCode(404), &format!("Not Found: {}", e), github_owner_repo.as_deref(), None, None),
	};

	let line = params.line.unwrap_or(1);
	let rendered_args = render_editor_args(editor_args, &local_path, line);
	let command_preview = format_command_preview(editor_command, &rendered_args);

	if !local_path.exists() {
		return create_error_html_response_with_status(StatusCode(404), "File not found", github_owner_repo.as_deref(), Some(local_path.display().to_string()), Some(command_preview));
	}

	let file_for_check = local_path.display().to_string();
	if let Some(ch) = find_unsafe_path_char(&file_for_check) {
		return create_error_html_response_with_status(StatusCode(400), &format!("Unsafe character '{}' in local file path", ch), github_owner_repo.as_deref(), Some(file_for_check), Some(command_preview));
	}

	match execute_command(editor_command, &rendered_args) {
		Ok(_) => {
			let message = "OK".to_string();
			Response::from_string(message).with_status_code(StatusCode(200))
		}
		Err(e) => create_error_html_response_with_status(StatusCode(500), &format!("Failed to execute command: {}", e), github_owner_repo.as_deref(), Some(local_path.display().to_string()), Some(command_preview)),
	}
}

fn generate_editor_html(_path: &str, token: &str) -> String {
	format!(
		r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{
	font-family: monospace;
	margin: 20px;
	background-color: #f5f5f5;
}}
.error {{
	background-color: #fff;
	border: 1px solid #ddd;
	border-radius: 4px;
	padding: 20px;
	margin: 20px 0;
}}
.error h2 {{
	color: #d32f2f;
	margin-top: 0;
}}
.error p {{
	color: #666;
	margin: 10px 0;
}}
.repo-link {{
	margin-top: 15px;
}}
.repo-link a {{
	color: #1976d2;
	text-decoration: none;
	font-weight: bold;
}}
.repo-link a:hover {{
	text-decoration: underline;
}}
</style>
</head>
<body>
<div id="result"></div>
<script>
let data = new Object();
data.token = '{}';
let pathname = document.location.pathname;
// Remove /editor prefix if present
if (pathname.startsWith('/editor')) {{
	pathname = pathname.substring(7); // Remove '/editor'
	if (!pathname.startsWith('/')) {{
		pathname = '/' + pathname;
	}}
}}
data.path = pathname;
// Extract line number from hash (e.g., #L42)
let hash = document.location.hash;
let line = null;
if (hash && hash.startsWith('#L')) {{
	let lineNum = parseInt(hash.substring(2));
	if (!isNaN(lineNum)) {{
		line = lineNum;
	}}
}}
data.line = line;
let request = new XMLHttpRequest();
request.open('POST', '/editor');
request.setRequestHeader('Content-Type', 'application/json');
request.onload = function() {{
	if (request.status === 200) {{
		window.close();
	}} else {{
		document.getElementById('result').innerHTML = request.responseText;
	}}
}};
request.onerror = function() {{
	document.getElementById('result').innerHTML = '<div class="error"><h2>Error</h2><p>Network error occurred.</p></div>';
}};
request.send(JSON.stringify(data));
</script>
</body>
</html>"#
		,
		js_escape_single_quoted(token)
	)
}

fn create_error_html_response_with_status(status_code: StatusCode, message: &str, owner_repo: Option<&str>, local_full_path: Option<String>, command_preview: Option<String>) -> Response<std::io::Cursor<Vec<u8>>> {
	let details_section = if let Some(local_full_path) = local_full_path {
		let cmd_section = if let Some(command_preview) = command_preview {
			format!(
				r#"
	<p>Command:</p>
	<pre>{}</pre>"#,
				html_escape(&command_preview)
			)
		} else {
			String::new()
		};
		format!(
			r#"
<div class="details">
	<p>Local path:</p>
	<pre>{}</pre>
	{}
</div>"#,
			html_escape(&local_full_path),
			cmd_section
		)
	} else {
		String::new()
	};

	let repo_section = if let Some(owner_repo) = owner_repo {
		let href = format!("https://github.com/{}", owner_repo);
		let label = owner_repo;
		format!(
			r#"
<div class="repo-link">
	<p>If you don't have this repository locally, clone it first.</p>
	<p>From <a href="{}" target="_blank">{}</a></p>
</div>"#,
			href, label
		)
	} else {
		String::new()
	};

	let html = format!(
		r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{
	font-family: monospace;
	margin: 20px;
	background-color: #f5f5f5;
}}
.error {{
	background-color: #fff;
	border: 1px solid #ddd;
	border-radius: 4px;
	padding: 20px;
	margin: 20px 0;
}}
.error h2 {{
	color: #d32f2f;
	margin-top: 0;
}}
.error p {{
	color: #666;
	margin: 10px 0;
}}
.repo-link {{
	margin-top: 15px;
}}
.repo-link a {{
	color: #1976d2;
	text-decoration: none;
	font-weight: bold;
}}
.repo-link a:hover {{
	text-decoration: underline;
}}
.details {{
	margin-top: 15px;
}}
.details pre {{
	background-color: #f0f0f0;
	border: 1px solid #ddd;
	border-radius: 4px;
	padding: 10px;
	white-space: pre-wrap;
	word-break: break-all;
}}
</style>
</head>
<body>
<div class="error">
	<h2>Error</h2>
	<p>{}</p>
	{}
	{}
</div>
</body>
</html>"#,
		html_escape(message),
		details_section,
		repo_section
	);

	if let Ok(header) = Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8") {
		Response::from_string(html).with_header(header).with_status_code(status_code)
	} else {
		Response::from_string(html).with_status_code(status_code)
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

fn js_escape_single_quoted(s: &str) -> String {
	s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn token_store() -> &'static Mutex<HashMap<String, TokenEntry>> {
	EDITOR_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn issue_one_time_token(expected_path: &str) -> String {
	let mut bytes = [0u8; 16];
	OsRng.fill_bytes(&mut bytes);
	let token: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

	let now = Instant::now();
	let expires_at = now + TOKEN_TTL;
	let store = token_store();
	let mut map = store.lock().unwrap();
	map.retain(|_, v| v.expires_at > now);
	map.insert(token.clone(), TokenEntry {
		expires_at,
		expected_path: expected_path.to_string(),
	});
	while map.len() > MAX_TOKENS {
		let oldest_key = map.iter()
			.min_by_key(|(_, v)| v.expires_at)
			.map(|(k, _)| k.clone());
		match oldest_key {
			Some(k) => {
				let _ = map.remove(&k);
			}
			None => break,
		}
	}
	token
}

fn consume_one_time_token(token: &str, expected_path: &str) -> bool {
	let now = Instant::now();
	let store = token_store();
	let mut map = store.lock().unwrap();
	map.retain(|_, v| v.expires_at > now);
	match map.remove(token) {
		Some(entry) => entry.expires_at > now && entry.expected_path == expected_path,
		None => false,
	}
}

fn normalize_github_path_for_token(path: &str) -> String {
	if path.is_empty() {
		"/".to_string()
	} else if path.starts_with('/') {
		path.to_string()
	} else {
		format!("/{}", path)
	}
}

fn extract_github_owner_repo(github_path: &str) -> Option<String> {
	let parts: Vec<&str> = github_path.split('/').filter(|s| !s.is_empty()).collect();
	if parts.len() < 2 {
		return None;
	}
	let account_name = parts.get(0)?;
	let repo_name = parts.get(1)?;
	Some(format!("{}/{}", account_name, repo_name))
}

fn convert_github_path_to_local(
	github_path: &str,
	default_repos_dir: &Option<String>,
	request_repos_dir: &Option<String>,
	include_host: bool,
) -> Result<PathBuf, String> {
	let repos_dir = request_repos_dir
		.as_ref()
		.or(default_repos_dir.as_ref())
		.ok_or("web.editor.reposDir not configured")?;

	let normalized_repos_dir = normalize_repos_dir(repos_dir)?;
	let rel_path = get_local_lib_path(github_path, include_host)?;
	let full_path = PathBuf::from(&normalized_repos_dir).join(rel_path);
	Ok(full_path)
}

fn normalize_repos_dir(repos_dir: &str) -> Result<String, String> {
	let mut normalized = repos_dir.to_string();
	if normalized.starts_with("~") {
		let home = std::env::var("HOME")
			.or_else(|_| std::env::var("USERPROFILE"))
			.map_err(|_| "HOME or USERPROFILE environment variable not set")?;
		normalized = normalized.replacen("~", &home, 1);
	}
	let path = PathBuf::from(&normalized);
	if !path.exists() {
		return Err(format!("web.editor.reposDir does not exist: {}", normalized));
	}
	if !path.is_dir() {
		return Err(format!("web.editor.reposDir is not a directory: {}", normalized));
	}
	Ok(normalized)
}

fn get_local_lib_path(github_path: &str, include_host: bool) -> Result<PathBuf, String> {
	let parts: Vec<&str> = github_path.split('/').filter(|s| !s.is_empty()).collect();
	if parts.len() < 4 {
		return Err("Invalid GitHub path format".to_string());
	}

	let owner_name = parts.get(0).ok_or("Invalid path")?;
	let repo_name = parts.get(1).ok_or("Invalid path")?;
	let blob_or_tree = parts.get(2).ok_or("Invalid path")?;
	if *blob_or_tree != "blob" && *blob_or_tree != "tree" {
		return Err("Path must contain 'blob' or 'tree'".to_string());
	}

	// parts: [account, repo, blob|tree, branch, ...paths]
	let file_path_parts: Vec<&str> = if parts.len() > 4 { parts[4..].to_vec() } else { Vec::new() };
	let mut rel_path = PathBuf::new();
	if include_host {
		rel_path.push("github.com");
	}
	rel_path.push(owner_name);
	rel_path.push(repo_name);
	for p in file_path_parts {
		rel_path.push(p);
	}
	Ok(rel_path)
}

fn render_editor_args(args: &[String], file_path: &PathBuf, line: u32) -> Vec<String> {
	let file_str = file_path.display().to_string();
	let line_str = line.to_string();

	args.iter()
		.map(|a| a.replace("{file}", &file_str).replace("{line}", &line_str))
		.collect()
}

fn format_command_preview(command: &str, args: &[String]) -> String {
	let mut parts: Vec<String> = Vec::new();
	parts.push(command.to_string());
	for a in args {
		parts.push(quote_arg_for_display(a));
	}
	parts.join(" ")
}

fn quote_arg_for_display(arg: &str) -> String {
	if arg.contains(' ') || arg.contains('\t') || arg.contains('"') {
		format!("\"{}\"", arg.replace('"', "\\\""))
	} else {
		arg.to_string()
	}
}

fn find_unsafe_path_char(s: &str) -> Option<char> {
	// Windows cmd metacharacters that can lead to command injection.
	for ch in ['&', '|', '<', '>', '^', '"'] {
		if s.contains(ch) {
			return Some(ch);
		}
	}
	if s.contains('\r') {
		return Some('\r');
	}
	if s.contains('\n') {
		return Some('\n');
	}
	None
}

fn execute_command(command: &str, args: &[String]) -> Result<(), String> {
	if cfg!(target_os = "windows") {
		let mut cmd = Command::new("cmd");
		cmd.args(&["/C", command]);
		for a in args {
			cmd.arg(a);
		}
		cmd
			.spawn()
			.map_err(|e| format!("Failed to execute command: {}", e))?;
	} else {
		let mut cmd = Command::new(command);
		for a in args {
			cmd.arg(a);
		}
		cmd.spawn()
			.map_err(|e| format!("Failed to execute command: {}", e))?;
	}
	Ok(())
}

