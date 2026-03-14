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
