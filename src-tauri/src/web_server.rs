use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, net::TcpListener, path::PathBuf, thread};
use tiny_http::Server;

use crate::config::{AppConfig, get_config_app_path};
use crate::util::open_with_system_command;
use crate::web::handler_static::handle_web_request;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditorConfig {
    #[serde(default)]
    pub repos_dir: Option<String>,
    #[serde(default)]
    pub include_host: bool,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebMarkdownConfig {
    #[serde(
        default = "df_allow_html_in_md",
        rename = "allowRawHTML",
        alias = "allowRawHtml"
    )]
    pub allow_raw_html: bool,
    #[serde(
        default = "df_markdown_open_external_link_in_new_tab",
        rename = "openExternalLinkInNewTab",
        alias = "openLinksInNewTab"
    )]
    pub open_external_link_in_new_tab: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebAssetsConfig {
    #[serde(default)]
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebContentConfig {
    #[serde(default)]
    pub markdown: Option<WebMarkdownConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebConfig {
    pub root: String,
    #[serde(default = "df_web_port", deserialize_with = "deserialize_web_port")]
    pub port: u16,
    #[serde(default = "df_open_browser_at_start")]
    pub open_browser_at_start: bool,
    #[serde(default = "df_dump")]
    pub dump: bool,
    #[serde(default = "df_slow")]
    pub slow: bool,
    #[serde(default = "df_status")]
    pub status: bool,
    #[serde(default)]
    pub content: Option<WebContentConfig>,
    #[serde(default)]
    pub assets: Option<WebAssetsConfig>,
    #[serde(default)]
    pub editor: Option<EditorConfig>,
}

#[derive(Debug, Clone)]
pub struct WebMarkdownHighlightConfig {
    pub main_css_url: String,
    pub main_js_url: String,
    pub static_md_css_url: String,
    pub static_md_js_url: String,
    pub static_structured_common_css_url: String,
    pub static_structured_common_js_url: String,
    pub static_json_css_url: String,
    pub static_json_js_url: String,
    pub static_yaml_css_url: String,
    pub static_yaml_js_url: String,
    pub static_toml_css_url: String,
    pub static_toml_js_url: String,
    pub static_ini_css_url: String,
    pub static_ini_js_url: String,
    pub css_url: String,
    pub js_url: String,
}

#[derive(Debug, Clone)]
pub struct WebAssetsServerConfig {
    pub root: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct WebServerConfig {
    pub root: String,
    pub port: u16,
    pub open_browser_at_start: bool,
    pub dump: bool,
    pub slow: bool,
    pub status: bool,
    pub allow_html_in_md: bool,
    pub markdown_open_external_link_in_new_tab: bool,
    pub markdown_highlight: Option<WebMarkdownHighlightConfig>,
    pub assets_server: Option<WebAssetsServerConfig>,
    pub editor_repos_dir: Option<String>,
    pub editor_include_host: bool,
    pub editor_command: String,
    pub editor_args: Vec<String>,
}

fn df_web_port() -> u16 {
    3030
}
fn df_open_browser_at_start() -> bool {
    false
}
fn df_dump() -> bool {
    false
}
fn df_slow() -> bool {
    false
}
fn df_status() -> bool {
    false
}
fn df_allow_html_in_md() -> bool {
    false
}
fn df_markdown_open_external_link_in_new_tab() -> bool {
    true
}

const MIN_WEB_PORT: u16 = 2000;

fn deserialize_web_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let port = u16::deserialize(deserializer)?;
    if port < MIN_WEB_PORT {
        return Err(serde::de::Error::custom(format!(
            "web.port must be >= {}",
            MIN_WEB_PORT
        )));
    }
    Ok(port)
}

fn is_local_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn find_available_port_downward(start_port: u16, min_port: u16, role: &str) -> Result<u16, String> {
    if start_port < min_port {
        return Err(format!(
            "Failed to resolve {} port: start port {} is below minimum {}",
            role, start_port, min_port
        ));
    }
    let mut candidate = start_port;
    loop {
        if is_local_port_available(candidate) {
            return Ok(candidate);
        }
        if candidate == min_port {
            break;
        }
        candidate -= 1;
    }
    Err(format!(
        "Failed to resolve {} port: no available port in range {}..={}",
        role, min_port, start_port
    ))
}

const EMBEDDED_HIGHLIGHT_JS: &str = include_str!("../../web-assets/highlight/highlight.min.js");
const EMBEDDED_HIGHLIGHT_CSS: &str = include_str!("../../web-assets/highlight/github-dark.min.css");
const EMBEDDED_MCLOCKS_MAIN_CSS: &str = include_str!("../../web-assets/mclocks/main.css");
const EMBEDDED_MCLOCKS_MAIN_JS: &str = include_str!("../../web-assets/mclocks/main.js");
const EMBEDDED_MCLOCKS_STATIC_MD_CSS: &str = include_str!("../../web-assets/mclocks/static/md.css");
const EMBEDDED_MCLOCKS_STATIC_MD_JS: &str = include_str!("../../web-assets/mclocks/static/md.js");
const EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured-common.css");
const EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured-common.js");
const EMBEDDED_MCLOCKS_STATIC_JSON_CSS: &str =
    include_str!("../../web-assets/mclocks/static/json.css");
const EMBEDDED_MCLOCKS_STATIC_JSON_JS: &str =
    include_str!("../../web-assets/mclocks/static/json.js");
const EMBEDDED_MCLOCKS_STATIC_YAML_CSS: &str =
    include_str!("../../web-assets/mclocks/static/yaml.css");
const EMBEDDED_MCLOCKS_STATIC_YAML_JS: &str =
    include_str!("../../web-assets/mclocks/static/yaml.js");
const EMBEDDED_MCLOCKS_STATIC_TOML_CSS: &str =
    include_str!("../../web-assets/mclocks/static/toml.css");
const EMBEDDED_MCLOCKS_STATIC_TOML_JS: &str =
    include_str!("../../web-assets/mclocks/static/toml.js");
const EMBEDDED_MCLOCKS_STATIC_INI_CSS: &str =
    include_str!("../../web-assets/mclocks/static/ini.css");
const EMBEDDED_MCLOCKS_STATIC_INI_JS: &str = include_str!("../../web-assets/mclocks/static/ini.js");
const HIGHLIGHT_JS_REL_PATH: &str = "highlight/highlight.min.js";
const HIGHLIGHT_CSS_REL_PATH: &str = "highlight/github-dark.min.css";
const MCLOCKS_MAIN_JS_REL_PATH: &str = "mclocks/main.js";
const MCLOCKS_MAIN_CSS_REL_PATH: &str = "mclocks/main.css";
const MCLOCKS_STATIC_MD_JS_REL_PATH: &str = "mclocks/static/md.js";
const MCLOCKS_STATIC_MD_CSS_REL_PATH: &str = "mclocks/static/md.css";
const MCLOCKS_STATIC_STRUCTURED_COMMON_JS_REL_PATH: &str = "mclocks/static/structured-common.js";
const MCLOCKS_STATIC_STRUCTURED_COMMON_CSS_REL_PATH: &str = "mclocks/static/structured-common.css";
const MCLOCKS_STATIC_JSON_JS_REL_PATH: &str = "mclocks/static/json.js";
const MCLOCKS_STATIC_JSON_CSS_REL_PATH: &str = "mclocks/static/json.css";
const MCLOCKS_STATIC_YAML_JS_REL_PATH: &str = "mclocks/static/yaml.js";
const MCLOCKS_STATIC_YAML_CSS_REL_PATH: &str = "mclocks/static/yaml.css";
const MCLOCKS_STATIC_TOML_JS_REL_PATH: &str = "mclocks/static/toml.js";
const MCLOCKS_STATIC_TOML_CSS_REL_PATH: &str = "mclocks/static/toml.css";
const MCLOCKS_STATIC_INI_JS_REL_PATH: &str = "mclocks/static/ini.js";
const MCLOCKS_STATIC_INI_CSS_REL_PATH: &str = "mclocks/static/ini.css";
const MCLOCKS_ASSETS_VERSION: &str = "20260314-2";

pub fn start_web_server(
    root: String,
    port: u16,
    dump_enabled: bool,
    slow_enabled: bool,
    status_enabled: bool,
    allow_html_in_md: bool,
    markdown_open_external_link_in_new_tab: bool,
    markdown_highlight: Option<WebMarkdownHighlightConfig>,
    editor_repos_dir: Option<String>,
    editor_include_host: bool,
    editor_command: String,
    editor_args: Vec<String>,
) {
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
            let response = handle_web_request(
                &mut request,
                &root_path,
                dump_enabled,
                slow_enabled,
                status_enabled,
                allow_html_in_md,
                markdown_open_external_link_in_new_tab,
                markdown_highlight.as_ref(),
                &editor_repos_dir,
                editor_include_host,
                &editor_command,
                &editor_args,
            );
            if let Err(e) = request.respond(response) {
                eprintln!("Failed to send response: {}", e);
            }
        }
    });
}

pub fn open_url_in_browser(url: &str) -> Result<(), String> {
    open_with_system_command(url, "Failed to open URL in browser")
}

fn prepare_markdown_assets_root(identifier: &String) -> Result<String, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let assets_root = base_dir
        .config_dir()
        .join(identifier)
        .join("web-assets")
        .to_path_buf();
    fs::create_dir_all(assets_root.join("highlight"))
        .map_err(|e| format!("Failed to create markdown assets dir: {}", e))?;
    fs::create_dir_all(assets_root.join("mclocks"))
        .map_err(|e| format!("Failed to create mclocks assets dir: {}", e))?;
    fs::create_dir_all(assets_root.join("mclocks").join("static"))
        .map_err(|e| format!("Failed to create mclocks static assets dir: {}", e))?;
    fs::write(
        assets_root.join(HIGHLIGHT_JS_REL_PATH),
        EMBEDDED_HIGHLIGHT_JS,
    )
    .map_err(|e| format!("Failed to write embedded highlight.js: {}", e))?;
    fs::write(
        assets_root.join(HIGHLIGHT_CSS_REL_PATH),
        EMBEDDED_HIGHLIGHT_CSS,
    )
    .map_err(|e| format!("Failed to write embedded highlight.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_MAIN_JS_REL_PATH),
        EMBEDDED_MCLOCKS_MAIN_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks main.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_MAIN_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_MAIN_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks main.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_MD_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_MD_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/md.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_MD_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_MD_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/md.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_STRUCTURED_COMMON_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured-common.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_STRUCTURED_COMMON_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured-common.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_JSON_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_JSON_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/json.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_JSON_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_JSON_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/json.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_YAML_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_YAML_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/yaml.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_YAML_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_YAML_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/yaml.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_TOML_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_TOML_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/toml.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_TOML_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_TOML_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/toml.css: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_INI_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_INI_JS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/ini.js: {}", e))?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_INI_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_INI_CSS,
    )
    .map_err(|e| format!("Failed to write embedded mclocks static/ini.css: {}", e))?;
    Ok(assets_root.to_string_lossy().to_string())
}

pub fn load_web_config(identifier: &String) -> Result<Option<WebServerConfig>, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let config_path = base_dir.config_dir().join(get_config_app_path(identifier));

    if !config_path.exists() {
        return Ok(None);
    }

    let config_json =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;
    let config_value: serde_json::Value =
        serde_json::from_str(&config_json).map_err(|e| format!("Failed to parse config: {}", e))?;
    let config: AppConfig =
        serde_json::from_str(&config_json).map_err(|e| format!("Failed to parse config: {}", e))?;

    let web_config = match config.web {
        Some(wc) => wc,
        None => return Ok(None),
    };

    let root_path = PathBuf::from(&web_config.root);
    if !root_path.exists() {
        return Err(format!("web.root not exists: {}", root_path.display()));
    }

    let editor_repos_dir = match web_config
        .editor
        .as_ref()
        .and_then(|e| e.repos_dir.as_ref())
    {
        Some(repos_dir) => Some(normalize_editor_repos_dir(repos_dir)?),
        None => None,
    };
    let allow_html_in_md = web_config
        .content
        .as_ref()
        .and_then(|c| c.markdown.as_ref())
        .map(|m| m.allow_raw_html)
        .unwrap_or(false);
    let markdown_open_external_link_in_new_tab = web_config
        .content
        .as_ref()
        .and_then(|c| c.markdown.as_ref())
        .map(|m| m.open_external_link_in_new_tab)
        .unwrap_or(true);
    let has_explicit_web_port = config_value.pointer("/web/port").is_some();
    let main_port = if has_explicit_web_port {
        if !is_local_port_available(web_config.port) {
            return Err(format!(
                "web.port {} is already in use. Please free the port or change web.port.",
                web_config.port
            ));
        }
        web_config.port
    } else {
        find_available_port_downward(web_config.port, MIN_WEB_PORT, "main web")?
    };
    let assets_start_port = main_port
        .checked_sub(1)
        .ok_or("Failed to resolve assets port: main web port is too low to derive assets port")?;
    let assets_port = find_available_port_downward(assets_start_port, MIN_WEB_PORT, "assets")?;
    let assets_root = prepare_markdown_assets_root(identifier)?;
    let assets_server = Some(WebAssetsServerConfig {
        root: assets_root,
        port: assets_port,
    });
    let markdown_highlight = Some(WebMarkdownHighlightConfig {
        main_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_MAIN_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        main_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_MAIN_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_md_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_MD_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_md_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_MD_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_structured_common_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_STRUCTURED_COMMON_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_structured_common_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_STRUCTURED_COMMON_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_json_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_JSON_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_json_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_JSON_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_yaml_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_YAML_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_yaml_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_YAML_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_toml_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_TOML_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_toml_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_TOML_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_ini_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_INI_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_ini_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_INI_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, HIGHLIGHT_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, HIGHLIGHT_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
    });
    let editor_include_host = web_config
        .editor
        .as_ref()
        .map(|e| e.include_host)
        .unwrap_or(false);
    let editor_command = web_config
        .editor
        .as_ref()
        .and_then(|e| e.command.as_ref())
        .cloned()
        .unwrap_or("code".to_string());
    let editor_args = web_config
        .editor
        .as_ref()
        .and_then(|e| e.args.as_ref())
        .cloned()
        .unwrap_or(vec!["-g".to_string(), "{file}:{line}".to_string()]);

    Ok(Some(WebServerConfig {
        root: web_config.root,
        port: main_port,
        open_browser_at_start: web_config.open_browser_at_start,
        dump: web_config.dump,
        slow: web_config.slow,
        status: web_config.status,
        allow_html_in_md,
        markdown_open_external_link_in_new_tab,
        markdown_highlight,
        assets_server,
        editor_repos_dir,
        editor_include_host,
        editor_command,
        editor_args,
    }))
}

fn normalize_editor_repos_dir(repos_dir: &str) -> Result<String, String> {
    let mut normalized = repos_dir.to_string();

    if normalized.starts_with("~") {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "HOME or USERPROFILE environment variable not set")?;
        normalized = normalized.replacen("~", &home, 1);
    }

    let path = PathBuf::from(&normalized);
    if !path.exists() {
        return Err(format!("web.editor.reposDir not exists: {}", normalized));
    }
    if !path.is_dir() {
        return Err(format!(
            "web.editor.reposDir is not a directory: {}",
            normalized
        ));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::dd_publish::{TEMP_DIR_PREFIX, register_temp_root};
    use crate::web::handler_static::{get_content_type, handle_web_request};
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use tempfile::TempDir;
    use tiny_http::Server;
    use urlencoding::encode;

    #[test]
    fn test_handle_web_request_root_with_index() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let index_file = root_path.join("index.html");
        fs::write(&index_file, "<html>test</html>").expect("Failed to create index.html");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/", port))
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
        fs::write(root_path.join("file2.html"), "<html>content2</html>")
            .expect("Failed to create file2.html");
        fs::create_dir_all(root_path.join("subdir")).expect("Failed to create subdir");
        let port = 3054;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        assert!(body.contains("file1.txt"), "Should list file1.txt");
        assert!(body.contains("file2.html"), "Should list file2.html");
        assert!(body.contains("subdir/"), "Should list subdir");
        assert!(
            body.contains("mclocks.web.content.mode")
                && body.contains("data-mode=\"raw\"")
                && body.contains("data-active-mode=\"raw\""),
            "Directory listing should include mode switch UI"
        );
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/subdir/", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/subdir/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        assert!(body.contains("file.txt"), "Should list file.txt");
        assert!(
            body.contains("id=\"directory-link\""),
            "Should show directory icon link in header"
        );
        assert!(
            body.contains("href=\"/\""),
            "Directory icon link should point to parent directory"
        );
        assert!(!body.contains(". . /"), "Should not show ../ entry");
    }

    #[test]
    fn test_handle_web_request_directory_listing_shows_hidden_with_links() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let subdir = root_path.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdir");
        fs::create_dir_all(subdir.join(".hidden-dir")).expect("Failed to create hidden dir");
        fs::write(subdir.join(".hidden.txt"), "hidden").expect("Failed to create hidden file");
        fs::write(subdir.join("visible.txt"), "visible").expect("Failed to create visible file");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/subdir/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains(".hidden.txt"), "Should show hidden file name");
        assert!(body.contains(".hidden-dir/"), "Should show hidden dir name");
        assert!(
            body.contains("href=\"/subdir/.hidden.txt\""),
            "Hidden file should be linked"
        );
        assert!(
            body.contains("href=\"/subdir/.hidden-dir/\""),
            "Hidden directory should be linked"
        );
        assert!(
            body.contains("href=\"/subdir/visible.txt\""),
            "Visible file should be linked"
        );
    }

    #[test]
    fn test_handle_web_request_hidden_file_access_is_allowed() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        fs::write(root_path.join(".secret.txt"), "secret").expect("Failed to create hidden file");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/.secret.txt", port))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.text().expect("Body should be readable"),
            "secret",
            "Hidden file should be served like regular files"
        );
    }

    #[test]
    fn test_resource_meta_rejects_hidden_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        fs::write(root_path.join(".secret.txt"), "secret").expect("Failed to create hidden file");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/.resource-meta?path=.secret.txt",
                port
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 400);
    }

    #[test]
    fn test_resource_meta_directory_preview_counts_entries() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let target = root_path.join("target");
        fs::create_dir_all(target.join("dir-a")).expect("Failed to create dir-a");
        fs::create_dir_all(target.join("dir-b")).expect("Failed to create dir-b");
        fs::write(target.join("a.txt"), "a").expect("Failed to create a.txt");
        fs::write(target.join("b.bin"), [1_u8, 2_u8]).expect("Failed to create b.bin");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/.resource-meta?path=target",
                port
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        assert_eq!(json["preview"], "files: 2, dirs: 2");
        assert_eq!(json["size_hr"], "-");
    }

    #[test]
    fn test_tmpdir_root_and_subdir_show_header_parent_link() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let subdir = root_path.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdir");
        fs::write(subdir.join("file.txt"), "content").expect("Failed to create file.txt");
        let share_hash =
            register_temp_root(root_path.as_path()).expect("Failed to register temp root");
        let parent_hash = register_temp_root(
            root_path
                .parent()
                .expect("Temp root should have a parent directory"),
        )
        .expect("Failed to register parent temp root");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path.clone(), port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let root_listing = client
            .get(&format!(
                "http://127.0.0.1:{}{}{}/",
                port, TEMP_DIR_PREFIX, share_hash
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(root_listing.status(), 200);
        let root_body = root_listing.text().expect("Failed to read body");
        assert!(
            root_body.contains("id=\"directory-link\""),
            "tmpdir root should show header parent link"
        );
        assert!(
            root_body.contains(&format!(
                "id=\"directory-link\" href=\"{}{}",
                TEMP_DIR_PREFIX, parent_hash
            )),
            "tmpdir root header parent link should point to registered parent tempdir"
        );
        assert!(
            !root_body.contains(". . /"),
            "tmpdir root should not show ../ entry"
        );

        let subdir_listing = client
            .get(&format!(
                "http://127.0.0.1:{}{}{}/subdir/",
                port, TEMP_DIR_PREFIX, share_hash
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(subdir_listing.status(), 200);
        let subdir_body = subdir_listing.text().expect("Failed to read body");
        assert!(
            subdir_body.contains("id=\"directory-link\""),
            "tmpdir subdir should show header parent link"
        );
        assert!(
            !subdir_body.contains(". . /"),
            "tmpdir subdir should not show ../ entry"
        );
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/test//file.html", port))
            .send()
            .expect("Failed to send request");
        // The client might normalize // to /, so we check for either 400 or 404
        assert!(
            response.status() == 400 || response.status() == 404,
            "Double slash should be rejected or not found"
        );
    }

    #[test]
    fn test_handle_web_request_double_slash() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3058;

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/test//file.html", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/test.html", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "<html>test</html>");
    }

    #[test]
    fn test_handle_web_request_html_file_raw_query_returns_plain_text() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let html_file = root_path.join("raw.html");
        fs::write(
            &html_file,
            "<!doctype html><html><body><h1>Raw</h1></body></html>",
        )
        .expect("Failed to create raw.html");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.html?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw HTML response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(
            body,
            "<!doctype html><html><body><h1>Raw</h1></body></html>"
        );
    }

    #[test]
    fn test_handle_web_request_markdown_file_renders_html() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("readme.md");
        fs::write(
            &md_file,
            "# Title\n\n## Section\n\n```js\nconsole.log('hello')\n```\n",
        )
        .expect("Failed to create readme.md");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/readme.md?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/html"),
            "Markdown response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            !body.contains("marked.min.js"),
            "Rendered markdown page should not depend on CDN"
        );
        assert!(
            body.contains("<h1 id=\"") && body.contains("<h2 id=\""),
            "Rendered markdown page should include heading ids"
        );
        assert!(
            body.contains("id=\"summary-list\"") && body.contains("Summary"),
            "Rendered markdown page should include summary pane"
        );
        assert!(
            body.contains("id=\"toc-list\"") && body.contains("Index"),
            "Rendered markdown page should include TOC"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/readme.md?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/readme.md\""),
            "Rendered markdown page should include mode switch links"
        );
        assert!(
            !body.contains("<style>") && !body.contains("mclocks-md-toc-width"),
            "Rendered markdown page should not embed inline markdown CSS/JS"
        );
    }

    #[test]
    fn test_handle_web_request_markdown_file_raw_query_returns_markdown_text() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("raw.md");
        fs::write(&md_file, "# Raw Title\n\n<script>alert('x')</script>\n")
            .expect("Failed to create raw.md");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.md?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw markdown response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "# Raw Title\n\n<script>alert('x')</script>\n");
    }

    #[test]
    fn test_handle_web_request_markdown_file_without_mode_serves_markdown_text() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("plain.md");
        fs::write(&md_file, "# Plain Title\n").expect("Failed to create plain.md");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/plain.md", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/markdown"),
            "Default markdown response should be text/markdown, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "# Plain Title\n");
    }

    #[test]
    fn test_handle_web_request_invalid_mode_is_treated_as_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("invalid-mode.md");
        fs::write(&md_file, "# Invalid Mode\n").expect("Failed to create invalid-mode.md");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/invalid-mode.md?mode=unknown",
                port
            ))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/markdown"),
            "Invalid mode should fall back to default response, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "# Invalid Mode\n");
    }

    #[test]
    fn test_handle_web_request_markdown_extension_renders_html() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("readme.markdown");
        fs::write(
            &md_file,
            "# Title\n\n## Section\n\n```js\nconsole.log('hello')\n```\n",
        )
        .expect("Failed to create readme.markdown");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/readme.markdown?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/html"),
            "Markdown(.markdown) response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"summary-list\"") && body.contains("Summary"),
            "Rendered markdown(.markdown) page should include summary pane"
        );
        assert!(
            body.contains("id=\"toc-list\"") && body.contains("Index"),
            "Rendered markdown(.markdown) page should include TOC"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/readme.markdown?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/readme.markdown\""),
            "Rendered markdown(.markdown) page should include mode switch links"
        );
    }

    #[test]
    fn test_handle_web_request_markdown_extension_raw_query_returns_markdown_text() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("raw.markdown");
        fs::write(&md_file, "# Raw Title\n\n<script>alert('x')</script>\n")
            .expect("Failed to create raw.markdown");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.markdown?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw markdown(.markdown) response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "# Raw Title\n\n<script>alert('x')</script>\n");
    }

    #[test]
    fn test_handle_web_request_json_file_renders_html_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("data.json");
        fs::write(
            &json_file,
            r#"{"user":{"name":"alice","enabled":true},"items":[1,2,3]}"#,
        )
        .expect("Failed to create data.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/data.json?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/html"),
            "JSON view response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"summary-list\""),
            "JSON view should include summary pane"
        );
        assert!(
            !body.contains("<span class=\"label\">Root</span>"),
            "JSON summary should not include root label"
        );
        assert!(
            body.contains("id=\"outline-list\""),
            "JSON view should include outline pane"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/data.json?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/data.json\""),
            "JSON view should include mode switch links"
        );
        assert!(
            body.contains("json-key") && body.contains("json-string"),
            "JSON view should include colorized tokens"
        );
    }

    #[test]
    fn test_handle_web_request_json_array_object_hover_wrap_includes_opening_brace_line() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("array-obj.json");
        fs::write(&json_file, r#"{"items":[{"name":"alice"},{"name":"bob"}]}"#)
            .expect("Failed to create array-obj.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/array-obj.json?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("data-key-path=\"items\""),
            "Container key should be annotated for hover consistency"
        );
        assert!(
            body.contains("data-path=\"items.0\">    <span class=\"json-delim\">{</span>"),
            "Array object node should include opening brace line inside hover target"
        );
        assert!(
            body.contains("<span class=\"json-delim\">}</span>"),
            "Array object node should include closing brace line inside hover target"
        );
        assert!(
            body.contains("data-path=\"items.0.name\""),
            "Nested path under array object should keep full path"
        );
    }

    #[test]
    fn test_handle_web_request_json_file_raw_query_returns_raw_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("raw.json");
        fs::write(&json_file, r#"{"name":"raw","count":1}"#).expect("Failed to create raw.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.json?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw JSON response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, r#"{"name":"raw","count":1}"#);
    }

    #[test]
    fn test_handle_web_request_json_file_without_mode_serves_json_text() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("plain.json");
        fs::write(&json_file, r#"{"name":"plain"}"#).expect("Failed to create plain.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/plain.json", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("application/json"),
            "Default JSON response should be application/json, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, r#"{"name":"plain"}"#);
    }

    #[test]
    fn test_handle_web_request_yaml_file_renders_html_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let yaml_file = root_path.join("data.yaml");
        fs::write(
            &yaml_file,
            "user:\n  name: alice\n  enabled: true\nitems:\n  - 1\n  - 2\n  - 3\n",
        )
        .expect("Failed to create data.yaml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/data.yaml?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/html"),
            "YAML view response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"summary-list\""),
            "YAML view should include summary pane"
        );
        assert!(
            body.contains("id=\"outline-list\""),
            "YAML view should include outline pane"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/data.yaml?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/data.yaml\""),
            "YAML view should include mode switch links"
        );
        assert!(
            body.contains("yaml-key") && body.contains("yaml-bool"),
            "YAML view should include colorized tokens"
        );
        assert!(
            !body.contains("<span class=\"yaml-delim json-delim\">{</span>"),
            "YAML view should render YAML content instead of JSON object delimiters"
        );
    }

    #[test]
    fn test_handle_web_request_yaml_file_raw_query_returns_raw_yaml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let yaml_file = root_path.join("raw.yml");
        fs::write(&yaml_file, "name: raw\ncount: 1\n").expect("Failed to create raw.yml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.yml?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw YAML response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "name: raw\ncount: 1\n");
    }

    #[test]
    fn test_handle_web_request_ini_family_extensions_render_html_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let ini_source = "[user]\nname=alice\nenabled=true\n";
        for ext in ["ini", "config", "cfg"] {
            let file_path = root_path.join(format!("settings.{}", ext));
            fs::write(&file_path, ini_source).expect("Failed to create INI family test file");
        }
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        for ext in ["ini", "config", "cfg"] {
            let response = client
                .get(&format!(
                    "http://127.0.0.1:{}/settings.{}?mode=source",
                    port, ext
                ))
                .send()
                .expect("Failed to send request");
            assert_eq!(response.status(), 200);
            let content_type = response
                .headers()
                .get("content-type")
                .expect("Content-Type header should exist")
                .to_str()
                .expect("Content-Type should be valid string");
            assert!(
                content_type.starts_with("text/html"),
                "INI family view response should be HTML, got: {}",
                content_type
            );

            let body = response.text().expect("Body should be readable");
            assert!(
                body.contains("id=\"summary-list\""),
                "INI family view should include summary pane"
            );
            assert!(
                body.contains("id=\"outline-list\""),
                "INI family view should include outline pane"
            );
            assert!(
                body.contains("data-mode=\"raw\"")
                    && body.contains(&format!("href=\"/settings.{}?mode=raw\"", ext))
                    && body.contains("data-mode=\"content\"")
                    && body.contains(&format!("href=\"/settings.{}\"", ext)),
                "INI family view should include mode switch links"
            );
            assert!(
                body.contains("json-key") && body.contains("json-string"),
                "INI family view should include colorized tokens"
            );
        }
    }

    #[test]
    fn test_handle_web_request_ini_file_raw_query_returns_raw_ini() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let ini_file = root_path.join("raw.ini");
        let ini_source = "[main]\nname=raw\ncount=1\n";
        fs::write(&ini_file, ini_source).expect("Failed to create raw.ini");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.ini?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw INI response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, ini_source);
    }

    #[test]
    fn test_handle_web_request_toml_file_renders_html_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let toml_file = root_path.join("data.toml");
        fs::write(
            &toml_file,
            "[user]\nname = \"alice\"\nenabled = true\nitems = [1, 2, 3]\n[owner]\ndob = 1979-05-27T07:32:00-08:00\n",
        )
        .expect("Failed to create data.toml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/data.toml?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/html"),
            "TOML view response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"summary-list\""),
            "TOML view should include summary pane"
        );
        assert!(
            body.contains("id=\"outline-list\""),
            "TOML view should include outline pane"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/data.toml?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/data.toml\""),
            "TOML view should include mode switch links"
        );
        assert!(
            body.contains("json-key") && body.contains("json-bool"),
            "TOML view should include colorized tokens"
        );
        assert!(
            body.contains("1979-05-27T07:32:00-08:00"),
            "TOML datetime should be rendered as plain string"
        );
        assert!(
            !body.contains("$__toml_private_datetime"),
            "TOML internal datetime marker must not be exposed"
        );
    }

    #[test]
    fn test_handle_web_request_toml_file_raw_query_returns_raw_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let toml_file = root_path.join("raw.toml");
        fs::write(&toml_file, "name = \"raw\"\ncount = 1\n").expect("Failed to create raw.toml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.toml?mode=raw", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let content_type = response
            .headers()
            .get("content-type")
            .expect("Content-Type header should exist")
            .to_str()
            .expect("Content-Type should be valid string");
        assert!(
            content_type.starts_with("text/plain"),
            "Raw TOML response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, "name = \"raw\"\ncount = 1\n");
    }

    #[test]
    fn test_handle_web_request_invalid_json_file_shows_error_and_raw_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("invalid.json");
        fs::write(&json_file, r#"{"name": }"#).expect("Failed to create invalid.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/invalid.json?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("Invalid JSON:"),
            "Invalid JSON view should show parse error"
        );
        assert!(
            body.contains("&quot;name&quot;"),
            "Invalid JSON view should show escaped raw content"
        );
    }

    #[test]
    fn test_handle_web_request_large_json_disables_colorization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let json_file = root_path.join("large.json");
        let large_text = "x".repeat(10 * 1024 * 1024 + 64);
        let large_json = format!(r#"{{"payload":"{}"}}"#, large_text);
        fs::write(&json_file, large_json).expect("Failed to create large.json");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/large.json?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("Status") && body.contains("Parse OK (Colorize: disabled &gt;10 MB)"),
            "Large JSON view should show colorization status in summary"
        );
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/test.html?key=value", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/nonexistent.html", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_handle_web_request_multibyte_filename() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let test_file = root_path.join("美乳雀のソーダ.txt");
        fs::write(&test_file, "マルチバイト文字のテスト")
            .expect("Failed to create file with multibyte name");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        // URL encode the filename
        let encoded_filename = encode("美乳雀のソーダ.txt");
        let response = client
            .get(&format!("http://127.0.0.1:{}/{}", port, encoded_filename))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().unwrap(), "マルチバイト文字のテスト");
    }

    #[test]
    fn test_handle_web_request_multibyte_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let multibyte_dir = root_path.join("美乳雀のソーダ");
        fs::create_dir_all(&multibyte_dir).expect("Failed to create directory with multibyte name");
        fs::write(multibyte_dir.join("test.txt"), "content").expect("Failed to create file");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        // URL encode the directory name
        let encoded_dir = encode("美乳雀のソーダ");
        let response = client
            .get(&format!("http://127.0.0.1:{}/{}/", port, encoded_dir))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        assert!(
            body.contains("美乳雀のソーダ"),
            "Should show decoded directory name in title"
        );
        assert!(body.contains("test.txt"), "Should list test.txt");
    }

    #[test]
    fn test_handle_web_request_multibyte_directory_listing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let multibyte_file = root_path.join("日本語ファイル.txt");
        let multibyte_dir = root_path.join("日本語ディレクトリ");
        fs::write(&multibyte_file, "日本語ファイルの内容")
            .expect("Failed to create file with multibyte name");
        fs::create_dir_all(&multibyte_dir).expect("Failed to create directory with multibyte name");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        // Check that multibyte characters are displayed correctly (not URL encoded)
        assert!(
            body.contains("日本語ファイル.txt"),
            "Should show decoded filename"
        );
        assert!(
            body.contains("日本語ディレクトリ"),
            "Should show decoded directory name"
        );
        // Check that links are URL encoded
        let encoded_file = encode("日本語ファイル.txt");
        let encoded_dir = encode("日本語ディレクトリ");
        assert!(
            body.contains(encoded_file.as_ref()),
            "Should contain URL encoded filename in link"
        );
        assert!(
            body.contains(&format!("{}/", encoded_dir.as_ref())),
            "Should contain URL encoded directory name in link"
        );
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
        assert_eq!(get_content_type(&path), "text/javascript");
    }

    #[test]
    fn test_get_content_type_json() {
        let path = PathBuf::from("data.json");
        assert_eq!(get_content_type(&path), "application/json");
    }

    #[test]
    fn test_get_content_type_yaml() {
        let path = PathBuf::from("data.yaml");
        assert_eq!(get_content_type(&path), "text/x-yaml");
    }

    #[test]
    fn test_get_content_type_yml() {
        let path = PathBuf::from("data.yml");
        assert_eq!(get_content_type(&path), "text/x-yaml");
    }

    #[test]
    fn test_get_content_type_toml() {
        let path = PathBuf::from("data.toml");
        assert_eq!(get_content_type(&path), "text/x-toml");
    }

    #[test]
    fn test_get_content_type_ini() {
        let path = PathBuf::from("settings.ini");
        assert_eq!(get_content_type(&path), "text/plain");
    }

    #[test]
    fn test_get_content_type_config() {
        let path = PathBuf::from("settings.config");
        assert_eq!(get_content_type(&path), "application/xml");
    }

    #[test]
    fn test_get_content_type_cfg() {
        let path = PathBuf::from("settings.cfg");
        assert_eq!(get_content_type(&path), "text/plain");
    }

    #[test]
    fn test_get_content_type_md() {
        let path = PathBuf::from("readme.md");
        assert_eq!(get_content_type(&path), "text/markdown");
    }

    #[test]
    fn test_get_content_type_markdown() {
        let path = PathBuf::from("readme.markdown");
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

    fn find_available_port() -> u16 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::net::TcpListener;

        let mut hasher = DefaultHasher::new();
        // Current time with nanosecond precision
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        // Thread ID for uniqueness in concurrent tests
        std::thread::current().id().hash(&mut hasher);
        let hash = hasher.finish();
        let base_port = 30000 + ((hash % 10000) as u16);
        for offset in 0..1000 {
            let port = ((base_port as u32 + offset) % 10000) as u16 + 30000;
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return port;
            }
        }
        30000
    }

    fn start_test_server_with_options(
        root: PathBuf,
        port: u16,
        dump_enabled: bool,
        slow_enabled: bool,
        status_enabled: bool,
        allow_html_in_md: bool,
    ) -> std::thread::JoinHandle<()> {
        thread::spawn(move || {
            let server = match Server::http(format!("127.0.0.1:{}", port)) {
                Ok(s) => s,
                Err(_) => return,
            };
            let editor_args: Vec<String> = vec!["-g".to_string(), "{file}:{line}".to_string()];
            let editor_command = "code".to_string();

            for mut request in server.incoming_requests() {
                let response = handle_web_request(
                    &mut request,
                    &root,
                    dump_enabled,
                    slow_enabled,
                    status_enabled,
                    allow_html_in_md,
                    true,
                    None,
                    &None,
                    false,
                    &editor_command,
                    &editor_args,
                );
                let _ = request.respond(response);
            }
        })
    }

    fn start_test_server(
        root: PathBuf,
        port: u16,
        dump_enabled: bool,
        slow_enabled: bool,
        status_enabled: bool,
    ) -> std::thread::JoinHandle<()> {
        start_test_server_with_options(
            root,
            port,
            dump_enabled,
            slow_enabled,
            status_enabled,
            false,
        )
    }

    #[test]
    fn test_handle_web_request_markdown_file_blocks_raw_html_by_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("raw-html.md");
        fs::write(&md_file, "# Title\n\n<script>alert('x')</script>\n")
            .expect("Failed to create raw-html.md");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/raw-html.md?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("&lt;script&gt;alert("),
            "Raw HTML should be escaped by default"
        );
        assert!(
            !body.contains("<script>alert("),
            "Raw HTML script tag should not be rendered"
        );
    }

    #[test]
    fn test_handle_web_request_markdown_file_allows_raw_html_when_enabled() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let md_file = root_path.join("raw-html.md");
        fs::write(&md_file, "# Title\n\n<script>alert('x')</script>\n")
            .expect("Failed to create raw-html.md");
        let port = find_available_port();

        let _server_handle =
            start_test_server_with_options(root_path, port, false, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/raw-html.md?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<script>alert('x')</script>"),
            "Raw HTML should be rendered when enabled"
        );
    }

    #[test]
    fn test_handle_dump_request_basic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3031;

        let _server_handle = start_test_server(root_path, port, true, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/dump", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );

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
        let response = client
            .get(&format!("http://127.0.0.1:{}/dump/test/path", port))
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
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/dump?key1=value1&key2=value2",
                port
            ))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/dump", port))
            .header("X-Custom-Header", "custom-value")
            .header("User-Agent", "test-agent")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");

        let headers = json["headers"]
            .as_array()
            .expect("Headers should be an array");
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
        let response = client
            .post(&format!("http://127.0.0.1:{}/dump", port))
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

        let parsed_body = json["parsed_body"]
            .as_object()
            .expect("Parsed body should be an object");
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
        let response = client
            .post(&format!("http://127.0.0.1:{}/dump", port))
            .header("Content-Type", "application/json")
            .body("{ invalid json }")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().expect("Failed to parse JSON");
        let parsed_body = json["parsed_body"]
            .as_str()
            .expect("Parsed body should be error string");
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
        let response = client
            .post(&format!("http://127.0.0.1:{}/dump", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/slow/3", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/slow/5", port))
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
        let response = client
            .post(&format!("http://127.0.0.1:{}/slow/3", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/slow/abc", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/slow/3", port))
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
        assert!(
            result.is_ok(),
            "Should return Ok when config file doesn't exist"
        );
        assert!(
            result.unwrap().is_none(),
            "Should return None when config file doesn't exist"
        );
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
        assert!(
            result.unwrap().is_none(),
            "Should return None when web field is missing"
        );

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
        let config_json = format!(
            r#"{{
            "web": {{
                "root": "{}"
            }}
        }}"#,
            nonexistent_path
        );
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.badroot".to_string();
        let result = load_web_config(&identifier);
        assert!(
            result.is_err(),
            "Should return error when root path doesn't exist. Got: {:?}",
            result
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("web.root not exists") || err_msg.contains("not exists"),
            "Error message should indicate root doesn't exist. Got: {}",
            err_msg
        );

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
        let web_root_str = web_root
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let config_json = format!(
            r#"{{
            "web": {{
                "root": "{}",
                "port": 8080,
                "openBrowserAtStart": true,
                "dump": true
            }}
        }}"#,
            web_root_str
        );
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.success".to_string();
        let result = load_web_config(&identifier);
        assert!(
            result.is_ok(),
            "Should successfully load web config. Got: {:?}",
            result
        );
        let web_config = result.unwrap();
        assert!(
            web_config.is_some(),
            "Should return Some(WebServerConfig). Got: {:?}",
            web_config
        );
        let config = web_config.unwrap();
        assert_eq!(config.port, 8080, "Port should match");
        assert_eq!(
            config.open_browser_at_start, true,
            "open_browser_at_start should be true"
        );
        assert_eq!(config.dump, true, "dump should be true");

        // Cleanup
        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_load_web_config_with_assets_for_markdown_highlight() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let web_root = temp_dir.path().join("webroot");
        fs::create_dir_all(&web_root).expect("Failed to create web root");

        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.assets");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        let web_root_str = web_root
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let config_json = format!(
            r#"{{
            "web": {{
                "root": "{}",
                "assets": {{
                    "port": 4040
                }}
            }}
        }}"#,
            web_root_str
        );
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.assets".to_string();
        let result = load_web_config(&identifier);
        assert!(
            result.is_ok(),
            "Should load config with assets: {:?}",
            result
        );
        let config = result
            .unwrap()
            .expect("Should return Some(WebServerConfig)");
        let assets_server = config
            .assets_server
            .expect("assets server should be configured");
        let markdown_highlight = config
            .markdown_highlight
            .expect("markdown highlight should be configured");
        let expected_assets_port = config
            .port
            .checked_sub(1)
            .expect("resolved main port should allow assets offset");
        assert_eq!(assets_server.port, expected_assets_port);
        assert!(
            assets_server.root.ends_with("web-assets"),
            "assets root should be auto-managed: {}",
            assets_server.root
        );
        assert_eq!(
            markdown_highlight.main_js_url,
            format!(
                "http://127.0.0.1:{}/mclocks/main.js?v={}",
                expected_assets_port, MCLOCKS_ASSETS_VERSION
            )
        );
        assert_eq!(
            markdown_highlight.main_css_url,
            format!(
                "http://127.0.0.1:{}/mclocks/main.css?v={}",
                expected_assets_port, MCLOCKS_ASSETS_VERSION
            )
        );
        assert_eq!(
            markdown_highlight.js_url,
            format!(
                "http://127.0.0.1:{}/highlight/highlight.min.js?v={}",
                expected_assets_port, MCLOCKS_ASSETS_VERSION
            )
        );
        assert_eq!(
            markdown_highlight.css_url,
            format!(
                "http://127.0.0.1:{}/{}?v={}",
                expected_assets_port, HIGHLIGHT_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
            )
        );

        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_load_web_config_explicit_port_in_use_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let web_root = temp_dir.path().join("webroot");
        fs::create_dir_all(&web_root).expect("Failed to create web root");
        let occupied_port = find_available_port();
        let _busy_listener =
            TcpListener::bind(("127.0.0.1", occupied_port)).expect("Failed to occupy port");

        let base_dir = BaseDirs::new().expect("Failed to get base dir");
        let config_dir = base_dir.config_dir();
        let test_config_dir = config_dir.join("test.app.explicit-port-busy");
        fs::create_dir_all(&test_config_dir).expect("Failed to create config dir");

        let config_file_name = if cfg!(debug_assertions) && tauri::is_dev() {
            "dev.config.json"
        } else {
            "config.json"
        };
        let config_path = test_config_dir.join(config_file_name);
        let web_root_str = web_root
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let config_json = format!(
            r#"{{
            "web": {{
                "root": "{}",
                "port": {}
            }}
        }}"#,
            web_root_str, occupied_port
        );
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.explicit-port-busy".to_string();
        let result = load_web_config(&identifier);
        assert!(
            result.is_err(),
            "Should fail when explicit web.port is already in use"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("already in use"),
            "Error message should indicate port conflict. Got: {}",
            err_msg
        );

        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_dir(&test_config_dir);
    }

    #[test]
    fn test_find_available_port_downward_for_assets_fallback_when_adjacent_busy() {
        let main_port = find_available_port();
        let busy_assets_port = main_port
            .checked_sub(1)
            .expect("main_port should be >= 2 for this test");
        let _busy_listener =
            TcpListener::bind(("127.0.0.1", busy_assets_port)).expect("Failed to occupy port");
        let resolved_assets_port =
            find_available_port_downward(busy_assets_port, MIN_WEB_PORT, "assets")
                .expect("Should resolve fallback assets port");
        assert_eq!(
            resolved_assets_port,
            busy_assets_port - 1,
            "Should fallback to the next lower available port"
        );
    }

    #[test]
    fn test_find_available_port_downward_for_main_port_when_preferred_busy() {
        let preferred_port = find_available_port();
        let _busy_listener =
            TcpListener::bind(("127.0.0.1", preferred_port)).expect("Failed to occupy port");
        let resolved_main_port =
            find_available_port_downward(preferred_port, MIN_WEB_PORT, "main web")
                .expect("Should resolve fallback main port");
        assert_eq!(
            resolved_main_port,
            preferred_port - 1,
            "Should fallback to next lower available port"
        );
    }

    #[test]
    fn test_web_config_deserialize_rejects_too_small_port() {
        let json = r#"{"root": "/test", "port": 1999}"#;
        let result: Result<WebConfig, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Should reject port smaller than 2000");
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
        let web_root_str = web_root
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let config_json = format!(
            r#"{{
            "web": {{
                "root": "{}"
            }}
        }}"#,
            web_root_str
        );
        fs::write(&config_path, config_json).expect("Failed to write config file");

        let identifier = "test.app.defaults".to_string();
        let result = load_web_config(&identifier);
        assert!(
            result.is_ok(),
            "Should successfully load web config with defaults. Got: {:?}",
            result
        );
        let web_config = result.unwrap();
        assert!(
            web_config.is_some(),
            "Should return Some(WebServerConfig). Got: {:?}",
            web_config
        );
        let config = web_config.unwrap();
        assert!(
            config.port <= df_web_port() && config.port >= MIN_WEB_PORT,
            "Default port should be resolved from preferred port downward in allowed range"
        );
        assert_eq!(
            config.open_browser_at_start, false,
            "Default open_browser_at_start should be false"
        );
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
        assert_eq!(
            config.port,
            df_web_port(),
            "Default port should follow environment default"
        );
        assert_eq!(
            config.open_browser_at_start, false,
            "Default open_browser_at_start should be false"
        );
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
        assert_eq!(
            config.open_browser_at_start, false,
            "Default open_browser_at_start should still apply"
        );
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
        assert_eq!(
            config.open_browser_at_start, true,
            "open_browser_at_start should be true"
        );
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
        assert_eq!(
            config.open_browser_at_start, true,
            "openBrowserAtStart should map to open_browser_at_start"
        );
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/200", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/418", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/204", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/301", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 301);
        assert!(response.headers().contains_key("location"));
        assert_eq!(
            response
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap(),
            "/"
        );
    }

    #[test]
    fn test_handle_status_request_401_unauthorized() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3047;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/401", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 401);
        assert!(response.headers().contains_key("www-authenticate"));
        assert_eq!(
            response
                .headers()
                .get("www-authenticate")
                .unwrap()
                .to_str()
                .unwrap(),
            "Basic realm=\"test\""
        );
    }

    #[test]
    fn test_handle_status_request_405_method_not_allowed() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3048;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/405", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 405);
        assert!(response.headers().contains_key("allow"));
        assert_eq!(
            response.headers().get("allow").unwrap().to_str().unwrap(),
            "GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS"
        );
    }

    #[test]
    fn test_handle_status_request_invalid_status_code_too_low() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = 3049;

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/99", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/600", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/200", port))
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
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/500", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 500);
        assert_eq!(response.text().unwrap(), "500 Internal Server Error");
    }

    #[test]
    fn test_handle_status_request_429_too_many_requests() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, true);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/status/429", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 429);
        assert!(response.headers().contains_key("retry-after"));
        assert_eq!(
            response
                .headers()
                .get("retry-after")
                .unwrap()
                .to_str()
                .unwrap(),
            "60"
        );
    }
}
