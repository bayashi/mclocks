use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContentMode {
    Content,
    Raw,
    Source,
}

impl ContentMode {
    pub fn as_query_value(self) -> Option<&'static str> {
        match self {
            ContentMode::Content => Some("content"),
            ContentMode::Raw => Some("raw"),
            ContentMode::Source => Some("source"),
        }
    }

    pub fn as_label(self) -> &'static str {
        match self {
            ContentMode::Content => "Content",
            ContentMode::Raw => "Raw",
            ContentMode::Source => "Source",
        }
    }
}

pub fn html_escape(s: &str) -> String {
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

pub fn build_mode_href(path: &str, query: &str, target_mode: ContentMode) -> String {
    let mut kept_pairs: Vec<String> = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let key = pair.splitn(2, '=').next().unwrap_or("");
        if key == "mode" || key == "raw" {
            continue;
        }
        kept_pairs.push(pair.to_string());
    }
    if target_mode == ContentMode::Content {
        // Keep content mode URL clean as implicit default.
    } else if let Some(mode_value) = target_mode.as_query_value() {
        kept_pairs.push(format!("mode={}", mode_value));
    }
    if kept_pairs.is_empty() {
        path.to_string()
    } else {
        format!("{}?{}", path, kept_pairs.join("&"))
    }
}

pub fn build_mode_switch_html(
    path: &str,
    query: &str,
    current_mode: ContentMode,
    id: &str,
) -> String {
    let mut html = String::new();
    let _ = write!(
        html,
        "<div class=\"{}\" role=\"group\" aria-label=\"Display mode\">",
        "mode-switch"
    );
    for mode in [ContentMode::Raw, ContentMode::Content] {
        let href = build_mode_href(path, query, mode);
        let active_class = if mode == current_mode {
            " is-active"
        } else {
            ""
        };
        let _ = write!(
            html,
            "<a id=\"{}-{}\" class=\"header-action-btn mode-btn{}\" href=\"{}\" data-mode=\"{}\">{}</a>",
            id,
            mode.as_query_value().unwrap_or("content"),
            active_class,
            html_escape(&href),
            mode.as_query_value().unwrap_or("content"),
            mode.as_label()
        );
    }
    html.push_str("</div>");
    html
}

pub fn render_main_header_html(
    absolute_path: &str,
    parent_directory_href: Option<&str>,
    mode_switch_html: Option<&str>,
) -> String {
    let directory_link_html = match parent_directory_href {
        Some(href) => format!(
            "<a id=\"directory-link\" href=\"{}\" title=\"Open directory\">📁</a>",
            html_escape(href)
        ),
        None => "".to_string(),
    };
    let mode_switch = mode_switch_html.unwrap_or("");
    format!(
        "<div id=\"main-header\"><div id=\"path-actions\">{}<div id=\"main-header-path\">{}</div><button id=\"path-copy-btn\" class=\"header-action-btn\" type=\"button\">Copy</button></div>{}</div>",
        directory_link_html,
        html_escape(absolute_path),
        mode_switch
    )
}
