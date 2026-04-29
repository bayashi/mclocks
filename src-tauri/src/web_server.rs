use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{fs, net::TcpListener, path::PathBuf, thread};
use tiny_http::Server;

use crate::config::{AppConfig, get_config_app_path, parse_config_json_to_value};
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
    #[serde(default)]
    pub enable_preview_api: bool,
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
    pub static_xml_css_url: String,
    pub static_xml_js_url: String,
    pub css_url: String,
    pub js_url: String,
    pub mermaid_js_url: String,
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
    /// Local WebSocket port for rendered-Markdown live reload (below assets port when present).
    pub markdown_live_reload_ws_port: Option<u16>,
    pub editor_repos_dir: Option<String>,
    pub editor_include_host: bool,
    pub editor_command: String,
    pub editor_args: Vec<String>,
    /// When `true`, the main web server handles `POST /preview`. Set from `web.content.markdown.enablePreviewApi` at startup.
    pub local_preview_api_enabled: Arc<AtomicBool>,
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
const EMBEDDED_MERMAID_JS: &str = include_str!("../../web-assets/mermaid/mermaid.min.js");
const EMBEDDED_MCLOCKS_MAIN_CSS: &str = include_str!("../../web-assets/mclocks/main.css");
const EMBEDDED_MCLOCKS_MAIN_JS: &str = include_str!("../../web-assets/mclocks/main.js");
const EMBEDDED_MCLOCKS_STATIC_MD_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/md.css");
const EMBEDDED_MCLOCKS_STATIC_MD_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/md.js");
const EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured.css");
const EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured.js");
const EMBEDDED_MCLOCKS_STATIC_JSON_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/json.css");
const EMBEDDED_MCLOCKS_STATIC_JSON_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/json.js");
const EMBEDDED_MCLOCKS_STATIC_YAML_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/yaml.css");
const EMBEDDED_MCLOCKS_STATIC_YAML_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/yaml.js");
const EMBEDDED_MCLOCKS_STATIC_TOML_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/toml.css");
const EMBEDDED_MCLOCKS_STATIC_TOML_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/toml.js");
const EMBEDDED_MCLOCKS_STATIC_INI_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/ini.css");
const EMBEDDED_MCLOCKS_STATIC_INI_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/ini.js");
const EMBEDDED_MCLOCKS_STATIC_XML_CSS: &str =
    include_str!("../../web-assets/mclocks/static/structured/xml.css");
const EMBEDDED_MCLOCKS_STATIC_XML_JS: &str =
    include_str!("../../web-assets/mclocks/static/structured/xml.js");
const HIGHLIGHT_JS_REL_PATH: &str = "highlight/highlight.min.js";
const HIGHLIGHT_CSS_REL_PATH: &str = "highlight/github-dark.min.css";
const MERMAID_JS_REL_PATH: &str = "mermaid/mermaid.min.js";
const MCLOCKS_MAIN_JS_REL_PATH: &str = "mclocks/main.js";
const MCLOCKS_MAIN_CSS_REL_PATH: &str = "mclocks/main.css";
const MCLOCKS_STATIC_MD_JS_REL_PATH: &str = "mclocks/static/structured/md.js";
const MCLOCKS_STATIC_MD_CSS_REL_PATH: &str = "mclocks/static/structured/md.css";
const MCLOCKS_STATIC_STRUCTURED_COMMON_JS_REL_PATH: &str = "mclocks/static/structured.js";
const MCLOCKS_STATIC_STRUCTURED_COMMON_CSS_REL_PATH: &str = "mclocks/static/structured.css";
const MCLOCKS_STATIC_JSON_JS_REL_PATH: &str = "mclocks/static/structured/json.js";
const MCLOCKS_STATIC_JSON_CSS_REL_PATH: &str = "mclocks/static/structured/json.css";
const MCLOCKS_STATIC_YAML_JS_REL_PATH: &str = "mclocks/static/structured/yaml.js";
const MCLOCKS_STATIC_YAML_CSS_REL_PATH: &str = "mclocks/static/structured/yaml.css";
const MCLOCKS_STATIC_TOML_JS_REL_PATH: &str = "mclocks/static/structured/toml.js";
const MCLOCKS_STATIC_TOML_CSS_REL_PATH: &str = "mclocks/static/structured/toml.css";
const MCLOCKS_STATIC_INI_JS_REL_PATH: &str = "mclocks/static/structured/ini.js";
const MCLOCKS_STATIC_INI_CSS_REL_PATH: &str = "mclocks/static/structured/ini.css";
const MCLOCKS_STATIC_XML_JS_REL_PATH: &str = "mclocks/static/structured/xml.js";
const MCLOCKS_STATIC_XML_CSS_REL_PATH: &str = "mclocks/static/structured/xml.css";
const MCLOCKS_ASSETS_VERSION: &str = "20260425-13";

#[derive(Clone, Copy, Debug)]
pub enum WebServerListenKind {
    Main,
    Assets,
}

impl WebServerListenKind {
    const fn log_label(self) -> &'static str {
        match self {
            WebServerListenKind::Main => "Main Server",
            WebServerListenKind::Assets => "Assets Server",
        }
    }

    /// Parenthetical purpose in the startup log when the document root exists.
    const fn log_purpose_with_root(self) -> &'static str {
        match self {
            WebServerListenKind::Main => "User web root",
            WebServerListenKind::Assets => "Bundled app static assets",
        }
    }

    /// Parenthetical purpose when there is no on-disk root (temp-share only).
    const fn log_purpose_no_root(self) -> &'static str {
        match self {
            WebServerListenKind::Main => "Temp-share only",
            WebServerListenKind::Assets => "Bundled assets",
        }
    }
}

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
    markdown_live_reload_ws_port: Option<u16>,
    local_preview_api: Option<Arc<AtomicBool>>,
    listen_kind: WebServerListenKind,
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
        let label = listen_kind.log_label();
        if root_path.exists() && root_path.is_dir() {
            println!(
                "{}: http://localhost:{} ({}) {}",
                label,
                port,
                listen_kind.log_purpose_with_root(),
                root_path.display()
            );
        } else {
            println!(
                "{}: http://localhost:{} ({}) {}",
                label,
                port,
                listen_kind.log_purpose_no_root(),
                "-"
            );
        }

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
                markdown_live_reload_ws_port,
                local_preview_api.as_ref(),
                port,
            );
            if let Err(e) = request.respond(response) {
                eprintln!("Failed to send response: {}", e);
            }
        }
    });
}

fn build_unconfigured_web_root_path(identifier: &String) -> Result<String, String> {
    let base_dir = BaseDirs::new().ok_or("Failed to get base dir")?;
    let root = base_dir
        .config_dir()
        .join(identifier)
        .join("__mclocks_unconfigured_web_root__");
    Ok(root.to_string_lossy().to_string())
}

pub fn default_web_server_config(identifier: &String) -> Result<WebServerConfig, String> {
    let main_port = find_available_port_downward(df_web_port(), MIN_WEB_PORT, "main web")?;
    let markdown_ws_start = main_port
        .checked_sub(1)
        .ok_or("Failed to resolve markdown live reload ws port: main web port is too low")?;
    let markdown_live_reload_ws_port = Some(find_available_port_downward(
        markdown_ws_start,
        MIN_WEB_PORT,
        "markdown live reload ws",
    )?);
    Ok(WebServerConfig {
        root: build_unconfigured_web_root_path(identifier)?,
        port: main_port,
        open_browser_at_start: false,
        dump: false,
        slow: false,
        status: false,
        allow_html_in_md: false,
        markdown_open_external_link_in_new_tab: true,
        markdown_highlight: None,
        assets_server: None,
        markdown_live_reload_ws_port,
        editor_repos_dir: None,
        editor_include_host: false,
        editor_command: "code".to_string(),
        editor_args: vec!["-g".to_string(), "{file}:{line}".to_string()],
        local_preview_api_enabled: Arc::new(AtomicBool::new(false)),
    })
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
    fs::create_dir_all(assets_root.join("highlight").join("languages"))
        .map_err(|e| format!("Failed to create highlight languages dir: {}", e))?;
    fs::create_dir_all(assets_root.join("mermaid"))
        .map_err(|e| format!("Failed to create mermaid assets dir: {}", e))?;
    fs::create_dir_all(assets_root.join("mclocks"))
        .map_err(|e| format!("Failed to create mclocks assets dir: {}", e))?;
    fs::create_dir_all(assets_root.join("mclocks").join("static"))
        .map_err(|e| format!("Failed to create mclocks static assets dir: {}", e))?;
    fs::create_dir_all(
        assets_root
            .join("mclocks")
            .join("static")
            .join("structured"),
    )
    .map_err(|e| {
        format!(
            "Failed to create mclocks static structured assets dir: {}",
            e
        )
    })?;
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
    fs::write(assets_root.join(MERMAID_JS_REL_PATH), EMBEDDED_MERMAID_JS)
        .map_err(|e| format!("Failed to write embedded mermaid.js: {}", e))?;
    let source_csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../web-assets/highlight/languages/csv.min.js");
    if source_csv_path.exists() && source_csv_path.is_file() {
        let dest_csv_path = assets_root
            .join("highlight")
            .join("languages")
            .join("csv.min.js");
        fs::copy(&source_csv_path, &dest_csv_path).map_err(|e| {
            format!(
                "Failed to copy highlight language asset {}: {}",
                source_csv_path.display(),
                e
            )
        })?;
    }
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
            "Failed to write embedded mclocks static/structured.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_STRUCTURED_COMMON_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_STRUCTURED_COMMON_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_JSON_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_JSON_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/json.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_JSON_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_JSON_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/json.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_YAML_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_YAML_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/yaml.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_YAML_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_YAML_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/yaml.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_TOML_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_TOML_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/toml.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_TOML_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_TOML_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/toml.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_INI_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_INI_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/ini.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_INI_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_INI_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/ini.css: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_XML_JS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_XML_JS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/xml.js: {}",
            e
        )
    })?;
    fs::write(
        assets_root.join(MCLOCKS_STATIC_XML_CSS_REL_PATH),
        EMBEDDED_MCLOCKS_STATIC_XML_CSS,
    )
    .map_err(|e| {
        format!(
            "Failed to write embedded mclocks static/structured/xml.css: {}",
            e
        )
    })?;
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
    let config_value = parse_config_json_to_value(&config_json)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    let config: AppConfig = serde_json::from_value(config_value.clone())
        .map_err(|e| format!("Failed to parse config: {}", e))?;

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
    let enable_preview_api = web_config
        .content
        .as_ref()
        .and_then(|c| c.markdown.as_ref())
        .map(|m| m.enable_preview_api)
        .unwrap_or(false);
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
    let markdown_ws_start = assets_port
        .checked_sub(1)
        .ok_or("Failed to resolve markdown live reload ws port: assets port is too low")?;
    let markdown_live_reload_ws_port = Some(find_available_port_downward(
        markdown_ws_start,
        MIN_WEB_PORT,
        "markdown live reload ws",
    )?);
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
        static_xml_css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_XML_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        static_xml_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MCLOCKS_STATIC_XML_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        css_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, HIGHLIGHT_CSS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, HIGHLIGHT_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
        ),
        mermaid_js_url: format!(
            "http://127.0.0.1:{}/{}?v={}",
            assets_port, MERMAID_JS_REL_PATH, MCLOCKS_ASSETS_VERSION
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
        markdown_live_reload_ws_port,
        editor_repos_dir,
        editor_include_host,
        editor_command,
        editor_args,
        local_preview_api_enabled: Arc::new(AtomicBool::new(enable_preview_api)),
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
    fn test_handle_web_request_root_with_index_shows_directory_listing() {
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
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        assert!(
            body.contains("index.html"),
            "Should include index.html as a regular file entry"
        );
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
            !body.contains("mclocks.web.content.mode")
                && !body.contains("data-active-mode=\"raw\"")
                && !body.contains("raw-switch"),
            "Directory listing should not include mode switch UI"
        );
    }

    #[test]
    fn test_handle_web_request_directory_with_index_shows_directory_listing() {
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
        let body = response.text().unwrap();
        assert!(body.contains("<ul>"), "Should show directory listing");
        assert!(
            body.contains("index.html"),
            "Should include index.html as a regular file entry"
        );
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
            body.contains("href=\"/?mode=source\""),
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
            body.contains("href=\"/subdir/.hidden.txt?mode=source\""),
            "Hidden file should be linked"
        );
        assert!(
            body.contains("href=\"/subdir/.hidden-dir/?mode=source\""),
            "Hidden directory should be linked"
        );
        assert!(
            body.contains("href=\"/subdir/visible.txt?mode=source\""),
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
    fn test_handle_web_request_xml_file_renders_html_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let xml_file = root_path.join("data.xml");
        fs::write(
            &xml_file,
            "<root><user enabled=\"true\"><name>alice</name></user><items><item>1</item><item>2</item></items></root>",
        )
        .expect("Failed to create data.xml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/data.xml?mode=source", port))
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
            "XML view response should be HTML, got: {}",
            content_type
        );

        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"summary-list\""),
            "XML view should include summary pane"
        );
        assert!(
            body.contains("id=\"outline-list\""),
            "XML view should include outline pane"
        );
        assert!(
            body.contains("data-mode=\"raw\"")
                && body.contains("href=\"/data.xml?mode=raw\"")
                && body.contains("data-mode=\"content\"")
                && body.contains("href=\"/data.xml\""),
            "XML view should include mode switch links"
        );
        assert!(
            body.contains("xml-key") && body.contains("xml-string"),
            "XML view should include colorized tokens"
        );
    }

    #[test]
    fn test_handle_web_request_xml_file_raw_query_returns_raw_xml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let xml_file = root_path.join("raw.xml");
        let xml_source = "<root><name>raw</name><count>1</count></root>";
        fs::write(&xml_file, xml_source).expect("Failed to create raw.xml");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/raw.xml?mode=raw", port))
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
            "Raw XML response should be text/plain, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert_eq!(body, xml_source);
    }

    #[test]
    fn test_handle_web_request_png_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let png_file = root_path.join("image.png");
        let png_bytes: Vec<u8> = vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, // signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
            b'I', b'H', b'D', b'R', // IHDR
            0x00, 0x00, 0x00, 0x02, // width = 2
            0x00, 0x00, 0x00, 0x03, // height = 3
            0x08, 0x02, 0x00, 0x00, 0x00, // bit depth / color type / etc.
            0x00, 0x00, 0x00, 0x00, // fake CRC (not validated by parser)
        ];
        fs::write(&png_file, png_bytes).expect("Failed to create image.png");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/image.png?mode=source", port))
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
            "PNG source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"sidebar-controls\"")
                && body.contains("id=\"summary-list\"")
                && body.contains("id=\"source-media-wrap\""),
            "PNG source view should be rendered in three-pane layout"
        );
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/image.png?mode=raw\""),
            "PNG source view should include image media element"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span>")
                && body.contains("<span class=\"label\">Height</span>"),
            "PNG source view summary should include Width/Height placeholders"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span><span class=\"value\">2px</span>")
                && body.contains(
                    "<span class=\"label\">Height</span><span class=\"value\">3px</span>"
                ),
            "PNG source view summary should include dimensions from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_mp3_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let mp3_file = root_path.join("sample.mp3");
        let mut mp3_bytes: Vec<u8> = vec![0xFF, 0xFB, 0x90, 0x64];
        mp3_bytes.resize(160_000, 0x00);
        fs::write(&mp3_file, mp3_bytes).expect("Failed to create sample.mp3");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.mp3?mode=source", port))
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
            "MP3 source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"sidebar-controls\"")
                && body.contains("id=\"summary-list\"")
                && body.contains("id=\"source-media-wrap\""),
            "MP3 source view should be rendered in three-pane layout"
        );
        assert!(
            body.contains("<audio id=\"source-media-audio\" controls preload=\"metadata\">")
                && body.contains("src=\"/sample.mp3?mode=raw\"")
                && body.contains("type=\"audio/mpeg\""),
            "MP3 source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span>"),
            "MP3 source view summary should include Duration placeholder"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:10</span>"),
            "MP3 source view summary should include estimated duration from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_mp4_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let mp4_file = root_path.join("sample.mp4");
        let mut mp4_bytes: Vec<u8> = vec![
            // ftyp box (24 bytes)
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0x00, 0x00,
            0x02, 0x00, b'i', b's', b'o', b'm', b'i', b's', b'o', b'2',
            // moov box (36 bytes)
            0x00, 0x00, 0x00, 0x24, b'm', b'o', b'o', b'v', // mvhd box (28 bytes)
            0x00, 0x00, 0x00, 0x1C, b'm', b'v', b'h', b'd', 0x00, 0x00, 0x00,
            0x00, // version + flags
            0x00, 0x00, 0x00, 0x00, // creation time
            0x00, 0x00, 0x00, 0x00, // modification time
            0x00, 0x00, 0x03, 0xE8, // timescale = 1000
            0x00, 0x00, 0x13, 0x88, // duration = 5000 (5 sec)
        ];
        mp4_bytes.resize(4096, 0x00);
        fs::write(&mp4_file, mp4_bytes).expect("Failed to create sample.mp4");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.mp4?mode=source", port))
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
            "MP4 source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("id=\"sidebar-controls\"")
                && body.contains("id=\"summary-list\"")
                && body.contains("id=\"source-media-wrap\""),
            "MP4 source view should be rendered in three-pane layout"
        );
        assert!(
            body.contains("<video id=\"source-media-video\" controls preload=\"metadata\">")
                && body.contains("src=\"/sample.mp4?mode=raw\"")
                && body.contains("type=\"video/mp4\""),
            "MP4 source view should include video media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span>"),
            "MP4 source view summary should include Duration placeholder"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:05</span>"),
            "MP4 source view summary should include parsed duration from mvhd"
        );
    }

    #[test]
    fn test_handle_web_request_jpg_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let jpg_file = root_path.join("image.jpg");
        let jpg_bytes: Vec<u8> = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, 0x00, 0x10, // APP0 (len=16)
            b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0xFF, 0xC0, 0x00, 0x11, // SOF0 (len=17)
            0x08, // precision
            0x00, 0x03, // height = 3
            0x00, 0x02, // width = 2
            0x03, // components
            0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00, 0xFF, 0xD9, // EOI
        ];
        fs::write(&jpg_file, jpg_bytes).expect("Failed to create image.jpg");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/image.jpg?mode=source", port))
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
            "JPG source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/image.jpg?mode=raw\""),
            "JPG source view should include image media element"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span><span class=\"value\">2px</span>")
                && body.contains(
                    "<span class=\"label\">Height</span><span class=\"value\">3px</span>"
                ),
            "JPG source view summary should include dimensions from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_gif_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let gif_file = root_path.join("image.gif");
        let gif_bytes: Vec<u8> = vec![
            b'G', b'I', b'F', b'8', b'9', b'a', // header
            0x02, 0x00, // width = 2 (LE)
            0x03, 0x00, // height = 3 (LE)
            0x80, 0x00, 0x00, // packed, bg, aspect
            0x00, 0x00, 0x00, // global color table #0
            0xFF, 0xFF, 0xFF, // global color table #1
            0x3B, // trailer
        ];
        fs::write(&gif_file, gif_bytes).expect("Failed to create image.gif");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/image.gif?mode=source", port))
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
            "GIF source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/image.gif?mode=raw\""),
            "GIF source view should include image media element"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span><span class=\"value\">2px</span>")
                && body.contains(
                    "<span class=\"label\">Height</span><span class=\"value\">3px</span>"
                ),
            "GIF source view summary should include dimensions from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_webp_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let webp_file = root_path.join("image.webp");
        let webp_bytes: Vec<u8> = vec![
            b'R', b'I', b'F', b'F', 0x16, 0x00, 0x00, 0x00, b'W', b'E', b'B',
            b'P', // RIFF WEBP
            b'V', b'P', b'8', b'X', 0x0A, 0x00, 0x00, 0x00, // VP8X chunk header
            0x00, 0x00, 0x00, 0x00, // flags + reserved
            0x01, 0x00, 0x00, // width-1 = 1 (width=2)
            0x02, 0x00, 0x00, // height-1 = 2 (height=3)
        ];
        fs::write(&webp_file, webp_bytes).expect("Failed to create image.webp");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/image.webp?mode=source", port))
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
            "WEBP source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/image.webp?mode=raw\""),
            "WEBP source view should include image media element"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span><span class=\"value\">2px</span>")
                && body.contains(
                    "<span class=\"label\">Height</span><span class=\"value\">3px</span>"
                ),
            "WEBP source view summary should include dimensions from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_bmp_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let bmp_file = root_path.join("image.bmp");
        let bmp_bytes: Vec<u8> = vec![
            // BITMAPFILEHEADER (14)
            b'B', b'M', 0x3A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00,
            0x00, // BITMAPINFOHEADER (40)
            0x28, 0x00, 0x00, 0x00, // DIB size
            0x02, 0x00, 0x00, 0x00, // width = 2
            0x03, 0x00, 0x00, 0x00, // height = 3
            0x01, 0x00, // planes
            0x18, 0x00, // bpp
            0x00, 0x00, 0x00, 0x00, // compression
            0x04, 0x00, 0x00, 0x00, // image size (dummy)
            0x13, 0x0B, 0x00, 0x00, // x ppm
            0x13, 0x0B, 0x00, 0x00, // y ppm
            0x00, 0x00, 0x00, 0x00, // colors used
            0x00, 0x00, 0x00, 0x00, // important colors
            // pixel bytes (dummy)
            0x00, 0x00, 0x00, 0x00,
        ];
        fs::write(&bmp_file, bmp_bytes).expect("Failed to create image.bmp");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/image.bmp?mode=source", port))
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
            "BMP source view response should be HTML, got: {}",
            content_type
        );
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/image.bmp?mode=raw\""),
            "BMP source view should include image media element"
        );
        assert!(
            body.contains("<span class=\"label\">Width</span><span class=\"value\">2px</span>")
                && body.contains(
                    "<span class=\"label\">Height</span><span class=\"value\">3px</span>"
                ),
            "BMP source view summary should include dimensions from file bytes"
        );
    }

    #[test]
    fn test_handle_web_request_invalid_png_source_hides_image_dimensions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let png_file = root_path.join("broken.png");
        // Valid PNG signature, but corrupted header (no IHDR at expected offset).
        let mut png_bytes: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png_bytes.resize(32, 0x00);
        fs::write(&png_file, png_bytes).expect("Failed to create broken.png");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/broken.png?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<img id=\"source-media-image\" src=\"/broken.png?mode=raw\""),
            "Broken PNG should still render media view"
        );
        assert!(
            !body.contains("<span class=\"label\">Width</span>")
                && !body.contains("<span class=\"label\">Height</span>"),
            "Width/Height labels should be hidden when dimensions are unavailable"
        );
    }

    #[test]
    fn test_handle_web_request_m4a_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let m4a_file = root_path.join("sample.m4a");
        let m4a_bytes: Vec<u8> = vec![
            // ftyp box (24 bytes)
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'M', b'4', b'A', b' ', 0x00, 0x00,
            0x00, 0x00, b'M', b'4', b'A', b' ', b'i', b's', b'o', b'm',
            // moov box (36 bytes)
            0x00, 0x00, 0x00, 0x24, b'm', b'o', b'o', b'v', // mvhd box (28 bytes)
            0x00, 0x00, 0x00, 0x1C, b'm', b'v', b'h', b'd', 0x00, 0x00, 0x00,
            0x00, // version + flags
            0x00, 0x00, 0x00, 0x00, // creation time
            0x00, 0x00, 0x00, 0x00, // modification time
            0x00, 0x00, 0x03, 0xE8, // timescale = 1000
            0x00, 0x00, 0x13, 0x88, // duration = 5000 (5 sec)
        ];
        fs::write(&m4a_file, m4a_bytes).expect("Failed to create sample.m4a");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.m4a?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("<audio id=\"source-media-audio\" controls preload=\"metadata\">")
                && body.contains("src=\"/sample.m4a?mode=raw\"")
                && body.contains("type=\"audio/mp4\""),
            "M4A source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:05</span>"),
            "M4A source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_wav_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let wav_file = root_path.join("sample.wav");
        let mut wav_bytes: Vec<u8> = vec![
            b'R', b'I', b'F', b'F', 0xA4, 0x3E, 0x00, 0x00, // RIFF size
            b'W', b'A', b'V', b'E', // WAVE
            b'f', b'm', b't', b' ', 0x10, 0x00, 0x00, 0x00, // fmt chunk
            0x01, 0x00, // PCM
            0x01, 0x00, // channels
            0x40, 0x1F, 0x00, 0x00, // sample rate 8000
            0x80, 0x3E, 0x00, 0x00, // byte rate 16000
            0x02, 0x00, // block align
            0x10, 0x00, // bits per sample
            b'd', b'a', b't', b'a', 0x80, 0x3E, 0x00, 0x00, // data chunk size 16000
        ];
        wav_bytes.resize(44 + 16_000, 0x00);
        fs::write(&wav_file, wav_bytes).expect("Failed to create sample.wav");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.wav?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.wav?mode=raw\"") && body.contains("type=\"audio/wav\""),
            "WAV source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:01</span>"),
            "WAV source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_ogg_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let ogg_file = root_path.join("sample.ogg");
        let mut ogg_bytes: Vec<u8> = Vec::new();
        // First page with OpusHead packet (19 bytes)
        ogg_bytes.extend_from_slice(b"OggS");
        ogg_bytes.push(0x00); // version
        ogg_bytes.push(0x02); // header type
        ogg_bytes.extend_from_slice(&0u64.to_le_bytes()); // granule
        ogg_bytes.extend_from_slice(&1u32.to_le_bytes()); // serial
        ogg_bytes.extend_from_slice(&0u32.to_le_bytes()); // seq
        ogg_bytes.extend_from_slice(&0u32.to_le_bytes()); // crc
        ogg_bytes.push(1); // segments
        ogg_bytes.push(19); // lacing
        ogg_bytes.extend_from_slice(b"OpusHead");
        ogg_bytes.push(1); // version
        ogg_bytes.push(2); // channels
        ogg_bytes.extend_from_slice(&0u16.to_le_bytes()); // pre-skip
        ogg_bytes.extend_from_slice(&48000u32.to_le_bytes()); // input sample rate
        ogg_bytes.extend_from_slice(&0u16.to_le_bytes()); // output gain
        ogg_bytes.push(0); // mapping family
        // Last page with granule = 48000 (1 sec)
        ogg_bytes.extend_from_slice(b"OggS");
        ogg_bytes.push(0x00); // version
        ogg_bytes.push(0x04); // eos
        ogg_bytes.extend_from_slice(&48000u64.to_le_bytes()); // granule
        ogg_bytes.extend_from_slice(&1u32.to_le_bytes()); // serial
        ogg_bytes.extend_from_slice(&1u32.to_le_bytes()); // seq
        ogg_bytes.extend_from_slice(&0u32.to_le_bytes()); // crc
        ogg_bytes.push(1); // segments
        ogg_bytes.push(1); // lacing
        ogg_bytes.push(0); // payload
        fs::write(&ogg_file, ogg_bytes).expect("Failed to create sample.ogg");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.ogg?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.ogg?mode=raw\"") && body.contains("type=\"audio/ogg\""),
            "OGG source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:01</span>"),
            "OGG source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_flac_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let flac_file = root_path.join("sample.flac");
        let sample_rate = 48000u64;
        let total_samples = 96_000u64; // 2 sec
        let channels_minus_one = 1u64;
        let bits_minus_one = 15u64;
        let packed = (sample_rate << 44)
            | (channels_minus_one << 41)
            | (bits_minus_one << 36)
            | total_samples;
        let mut flac_bytes: Vec<u8> = vec![
            b'f', b'L', b'a', b'C', // signature
            0x80, 0x00, 0x00, 0x22, // last-metadata + STREAMINFO length=34
            0x04, 0x00, // min block size
            0x04, 0x00, // max block size
            0x00, 0x00, 0x00, // min frame size
            0x00, 0x00, 0x00, // max frame size
        ];
        flac_bytes.extend_from_slice(&packed.to_be_bytes());
        flac_bytes.extend_from_slice(&[0u8; 16]); // md5
        fs::write(&flac_file, flac_bytes).expect("Failed to create sample.flac");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/sample.flac?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.flac?mode=raw\"") && body.contains("type=\"audio/flac\""),
            "FLAC source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:02</span>"),
            "FLAC source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_aac_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let aac_file = root_path.join("sample.aac");
        let mut aac_bytes: Vec<u8> = Vec::new();
        // 43 ADTS frames, each 20 bytes => about 1 second at 44.1kHz
        for _ in 0..43 {
            aac_bytes.extend_from_slice(&[0xFF, 0xF1, 0x50, 0x80, 0x02, 0x9F, 0xFC]);
            aac_bytes.extend_from_slice(&[0u8; 13]);
        }
        fs::write(&aac_file, aac_bytes).expect("Failed to create sample.aac");
        let port = find_available_port();

        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.aac?mode=source", port))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.aac?mode=raw\"") && body.contains("type=\"audio/aac\""),
            "AAC source view should include audio media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:01</span>"),
            "AAC source view summary should include estimated duration"
        );
    }

    #[test]
    fn test_handle_web_request_m4v_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let m4v_file = root_path.join("sample.m4v");
        let m4v_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0x00, 0x00,
            0x00, 0x00, b'i', b's', b'o', b'm', b'm', b'p', b'4', b'1', 0x00, 0x00, 0x00, 0x24,
            b'm', b'o', b'o', b'v', 0x00, 0x00, 0x00, 0x1C, b'm', b'v', b'h', b'd', 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE8,
            0x00, 0x00, 0x13, 0x88,
        ];
        fs::write(&m4v_file, m4v_bytes).expect("Failed to create sample.m4v");
        let port = find_available_port();
        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.m4v?mode=source", port))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.m4v?mode=raw\"") && body.contains("type=\"video/mp4\""),
            "M4V source view should include video media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:05</span>"),
            "M4V source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_mov_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let mov_file = root_path.join("sample.mov");
        let mov_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x14, b'f', b't', b'y', b'p', b'q', b't', b' ', b' ', 0x00, 0x00,
            0x00, 0x00, b'q', b't', b' ', b' ', 0x00, 0x00, 0x00, 0x24, b'm', b'o', b'o', b'v',
            0x00, 0x00, 0x00, 0x1C, b'm', b'v', b'h', b'd', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x13, 0x88,
        ];
        fs::write(&mov_file, mov_bytes).expect("Failed to create sample.mov");
        let port = find_available_port();
        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.mov?mode=source", port))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.mov?mode=raw\"")
                && body.contains("type=\"video/quicktime\""),
            "MOV source view should include video media element"
        );
        assert!(
            body.contains("<span class=\"label\">Duration</span><span class=\"value\">0:05</span>"),
            "MOV source view summary should include parsed duration"
        );
    }

    #[test]
    fn test_handle_web_request_webm_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let webm_file = root_path.join("sample.webm");
        let webm_bytes: Vec<u8> = vec![
            0x1A, 0x45, 0xDF, 0xA3, // EBML
            0x00, 0x00, 0x00, 0x00, b'w', b'e', b'b', b'm',
        ];
        fs::write(&webm_file, webm_bytes).expect("Failed to create sample.webm");
        let port = find_available_port();
        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "http://127.0.0.1:{}/sample.webm?mode=source",
                port
            ))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.webm?mode=raw\"") && body.contains("type=\"video/webm\""),
            "WEBM source view should include video media element"
        );
        assert!(
            !body.contains("<span class=\"label\">Duration</span>"),
            "WEBM source view summary should hide Duration when unavailable"
        );
    }

    #[test]
    fn test_handle_web_request_ogv_file_source_renders_media_view() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_path_buf();
        let ogv_file = root_path.join("sample.ogv");
        let ogv_bytes: Vec<u8> = vec![b'O', b'g', b'g', b'S', 0x00, 0x00, 0, 0, 0, 0, 0, 0];
        fs::write(&ogv_file, ogv_bytes).expect("Failed to create sample.ogv");
        let port = find_available_port();
        let _server_handle = start_test_server(root_path, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("http://127.0.0.1:{}/sample.ogv?mode=source", port))
            .send()
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().expect("Body should be readable");
        assert!(
            body.contains("src=\"/sample.ogv?mode=raw\"") && body.contains("type=\"video/ogg\""),
            "OGV source view should include video media element"
        );
        assert!(
            !body.contains("<span class=\"label\">Duration</span>"),
            "OGV source view summary should hide Duration when unavailable"
        );
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
    fn test_temp_share_works_when_root_is_unconfigured() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let missing_root = temp_dir.path().join("missing-web-root");
        let dropped_dir = temp_dir.path().join("dropped");
        fs::create_dir_all(&dropped_dir).expect("Failed to create dropped directory");
        fs::write(dropped_dir.join("hello.txt"), "hello").expect("Failed to create dropped file");
        let port = find_available_port();

        let _server_handle = start_test_server(missing_root, port, false, false, false);
        thread::sleep(std::time::Duration::from_millis(100));

        let client = reqwest::blocking::Client::new();

        let root_response = client
            .get(&format!("http://127.0.0.1:{}/", port))
            .send()
            .expect("Failed to send root request");
        assert_eq!(
            root_response.status(),
            404,
            "Regular path should return 404 when web root is not configured"
        );

        let hash = register_temp_root(dropped_dir.as_path())
            .expect("Failed to register temporary dropped directory");
        let tmp_response = client
            .get(&format!(
                "http://127.0.0.1:{}{}{}/?mode=source",
                port, TEMP_DIR_PREFIX, hash
            ))
            .send()
            .expect("Failed to send temp-share request");
        assert_eq!(
            tmp_response.status(),
            200,
            "Temp-share URL should still be accessible"
        );
        let body = tmp_response
            .text()
            .expect("Response body should be readable");
        assert!(
            body.contains("hello.txt"),
            "Temp-share directory listing should include dropped file"
        );
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
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;
        thread::spawn(move || {
            let server = match Server::http(format!("127.0.0.1:{}", port)) {
                Ok(s) => s,
                Err(_) => return,
            };
            let editor_args: Vec<String> = vec!["-g".to_string(), "{file}:{line}".to_string()];
            let editor_command = "code".to_string();
            let preview_off = Arc::new(AtomicBool::new(false));

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
                    None,
                    Some(&preview_off),
                    port,
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
    fn test_default_web_server_config_for_temp_share_only_mode() {
        let identifier = "test.app.defaultweb".to_string();
        let config =
            default_web_server_config(&identifier).expect("Should build default web server config");
        assert!(
            config.port >= MIN_WEB_PORT,
            "Default port should be in valid range"
        );
        assert_eq!(
            config.open_browser_at_start, false,
            "Default open browser should be disabled"
        );
        assert_eq!(config.dump, false, "Default dump should be disabled");
        assert_eq!(config.slow, false, "Default slow should be disabled");
        assert_eq!(config.status, false, "Default status should be disabled");
        assert!(
            config.root.contains("__mclocks_unconfigured_web_root__"),
            "Default root should point to unconfigured marker path"
        );
        assert!(
            config.assets_server.is_none(),
            "Assets server should be disabled in temp-share only mode"
        );
        assert!(
            config.markdown_highlight.is_none(),
            "Markdown highlight assets should be disabled in temp-share only mode"
        );
        let md_ws = config
            .markdown_live_reload_ws_port
            .expect("markdown live reload ws port should be set");
        assert!(
            md_ws < config.port,
            "markdown ws port should be below main web port"
        );
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
        let md_ws = config
            .markdown_live_reload_ws_port
            .expect("markdown live reload ws port");
        assert!(
            md_ws < assets_server.port,
            "markdown ws should be strictly below assets port (sequential downward bind)"
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
        let preview = config
            .content
            .as_ref()
            .and_then(|c| c.markdown.as_ref())
            .map(|m| m.enable_preview_api)
            .unwrap_or(false);
        assert_eq!(preview, false, "Default enable_preview_api should be false");
    }

    #[test]
    fn test_web_config_markdown_enable_preview_api() {
        let json = r#"{"root": "/x", "content": {"markdown": {"enablePreviewApi": true}}}"#;
        let config: WebConfig = serde_json::from_str(json).expect("deserialize");
        assert!(
            config
                .content
                .as_ref()
                .expect("content")
                .markdown
                .as_ref()
                .expect("markdown")
                .enable_preview_api
        );
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
