use crate::web_server::WebMarkdownHighlightConfig;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use std::collections::HashMap;
use std::path::Path;
use tiny_http::{Header, Response, StatusCode};
use urlencoding::encode;

const MARKDOWN_RENDER_TEMPLATE: &str = r##"<!doctype html>
<html>
<head>
<meta charset="UTF-8" />
<title>__PAGE_TITLE__</title>
__MAIN_CSS_LINK__
__STATIC_MD_CSS_LINK__
__HIGHLIGHT_CSS_LINK__
</head>
<body class="mclocks-md">
<nav id="toc">
<h2>Index</h2>
<ul id="toc-list">__TOC_ITEMS__</ul>
<div id="toc-footer">
<a id="raw-toggle" href="__RAW_TOGGLE_HREF__">Raw</a>
</div>
</nav>
<div id="toc-resizer" aria-label="Resize TOC" title="Drag to resize"></div>
<div id="main">
<div id="content">__RENDERED_HTML__</div>
</div>
__HIGHLIGHT_JS_SCRIPT__
__MAIN_JS_SCRIPT__
__STATIC_MD_JS_SCRIPT__
</body>
</html>
"##;

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

fn get_markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn fnv1a_hash_hex8(input: &str) -> String {
    let mut hash: u32 = 0x811c9dc5;
    for b in input.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    format!("{:08x}", hash)
}

fn heading_id(text: &str) -> String {
    let source = encode(text.trim()).into_owned();
    fnv1a_hash_hex8(&source)
}

fn extract_markdown_headings(markdown: &str) -> Vec<(u8, String, String)> {
    let parser = Parser::new_ext(markdown, get_markdown_options());
    let mut headings = Vec::new();
    let mut used_ids: HashMap<String, usize> = HashMap::new();
    let mut in_heading = false;
    let mut current_level: u8 = 0;
    let mut current_text = String::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let level_num = heading_level_to_u8(level);
                if level_num <= 4 {
                    in_heading = true;
                    current_level = level_num;
                    current_text.clear();
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    let text = current_text.trim().to_string();
                    let base_id = heading_id(&text);
                    let count = used_ids.entry(base_id.clone()).or_insert(0);
                    *count += 1;
                    let id = if *count == 1 {
                        base_id
                    } else {
                        format!("{}-{}", base_id, *count)
                    };
                    headings.push((current_level, text, id));
                    in_heading = false;
                    current_level = 0;
                    current_text.clear();
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if in_heading {
                    current_text.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_heading {
                    current_text.push(' ');
                }
            }
            _ => {}
        }
    }
    headings
}

fn inject_heading_ids(rendered_html: &str, headings: &[(u8, String, String)]) -> String {
    let mut out = String::with_capacity(rendered_html.len() + headings.len() * 24);
    let mut cursor = 0usize;
    for (level, _, id) in headings {
        let marker = format!("<h{}>", level);
        if let Some(rel_pos) = rendered_html[cursor..].find(&marker) {
            let start = cursor + rel_pos;
            out.push_str(&rendered_html[cursor..start]);
            out.push_str(&format!("<h{} id=\"{}\">", level, html_escape(id)));
            cursor = start + marker.len();
        }
    }
    out.push_str(&rendered_html[cursor..]);
    out
}

fn build_toc_items_html(headings: &[(u8, String, String)]) -> String {
    let mut toc = String::new();
    for (level, text, id) in headings {
        toc.push_str("<li data-level=\"");
        toc.push_str(&level.to_string());
        toc.push_str("\"><a href=\"#");
        toc.push_str(&html_escape(id));
        toc.push_str("\">");
        toc.push_str(&html_escape(text));
        toc.push_str("</a></li>");
    }
    toc
}

fn render_markdown_html(markdown_source: &str, allow_html_in_md: bool) -> String {
    let parser = Parser::new_ext(markdown_source, get_markdown_options());
    let mut rendered_html = String::new();
    if allow_html_in_md {
        html::push_html(&mut rendered_html, parser);
    } else {
        let sanitized_events = parser.map(|event| match event {
            Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
            _ => event,
        });
        html::push_html(&mut rendered_html, sanitized_events);
    }
    rendered_html
}

pub fn is_markdown_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"),
        None => false,
    }
}

pub fn should_serve_raw_content(url: &str) -> bool {
    let query = match url.split('?').nth(1) {
        Some(q) => q.split('#').next().unwrap_or(q),
        None => return false,
    };
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");
        if key == "raw" && (value == "1" || value.eq_ignore_ascii_case("true")) {
            return true;
        }
    }
    false
}

pub fn build_raw_content_toggle_href(url: &str) -> String {
    let no_fragment = url.split('#').next().unwrap_or(url);
    let mut parts = no_fragment.splitn(2, '?');
    let path = parts.next().unwrap_or("/");
    let query = parts.next().unwrap_or("");
    let mut kept_pairs: Vec<String> = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let key = pair.splitn(2, '=').next().unwrap_or("");
        if key == "raw" {
            continue;
        }
        kept_pairs.push(pair.to_string());
    }
    kept_pairs.push("raw=1".to_string());
    format!("{}?{}", path, kept_pairs.join("&"))
}

pub fn create_markdown_response(
    file_path: &Path,
    markdown_source: &str,
    allow_html_in_md: bool,
    markdown_highlight: Option<&WebMarkdownHighlightConfig>,
    raw_toggle_href: &str,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let headings = extract_markdown_headings(markdown_source);
    let rendered_html = render_markdown_html(markdown_source, allow_html_in_md);
    let rendered_html = inject_heading_ids(&rendered_html, &headings);
    let toc_items_html = build_toc_items_html(&headings);
    let page_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(html_escape)
        .unwrap_or_else(|| "Markdown".to_string());
    let (
        main_css_link,
        static_md_css_link,
        main_js_script,
        static_md_js_script,
        highlight_css_link,
        highlight_js_script,
    ) = match markdown_highlight {
        Some(cfg) => (
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.main_css_url)
            ),
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.static_md_css_url)
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(&cfg.main_js_url)
            ),
            format!(
                "<script src=\"{}\"></script>",
                html_escape(&cfg.static_md_js_url)
            ),
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" />",
                html_escape(&cfg.css_url)
            ),
            format!("<script src=\"{}\"></script>", html_escape(&cfg.js_url)),
        ),
        None => (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ),
    };
    let html = MARKDOWN_RENDER_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__TOC_ITEMS__", &toc_items_html)
        .replace("__RAW_TOGGLE_HREF__", &html_escape(raw_toggle_href))
        .replace("__MAIN_CSS_LINK__", &main_css_link)
        .replace("__STATIC_MD_CSS_LINK__", &static_md_css_link)
        .replace("__MAIN_JS_SCRIPT__", &main_js_script)
        .replace("__STATIC_MD_JS_SCRIPT__", &static_md_js_script)
        .replace("__HIGHLIGHT_CSS_LINK__", &highlight_css_link)
        .replace("__HIGHLIGHT_JS_SCRIPT__", &highlight_js_script)
        .replace("__RENDERED_HTML__", &rendered_html);
    let content_type = "text/html; charset=utf-8";
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()) {
        Response::from_string(html)
            .with_header(header)
            .with_status_code(StatusCode(200))
    } else {
        Response::from_string(html).with_status_code(StatusCode(200))
    }
}
