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
<style>
:root {
	--toc-width: 240px;
}
* {
	box-sizing: border-box;
}
body {
	color: #aaa;
	background: #000;
	margin: 0;
	display: flex;
	font-family: "Segoe UI", "Yu Gothic UI", "Meiryo", "Hiragino Kaku Gothic ProN", sans-serif;
	line-height: 1.6;
}
#toc {
	position: fixed;
	top: 0;
	left: 0;
	width: var(--toc-width);
	height: 100vh;
	overflow-y: auto;
	background: #0a0a0a;
	border-right: 1px solid #222;
	padding: 16px 12px;
	font-size: 13px;
	scrollbar-width: thin;
	scrollbar-color: #333 transparent;
}
#toc h2 {
	color: #666;
	font-size: 11px;
	letter-spacing: 0.1em;
	text-transform: uppercase;
	margin: 0 0 12px 4px;
}
#toc ul {
	list-style: none;
	margin: 0;
	padding: 0;
}
#toc li a {
	display: block;
	color: #555;
	text-decoration: none;
	padding: 3px 4px;
	border-radius: 3px;
	line-height: 1.4;
	transition: color 0.15s, background 0.15s;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}
#toc li a:hover {
	color: #ccc;
	background: #1a1a1a;
}
#toc li a.active {
	color: #fff;
	background: #1e1e1e;
}
#toc li[data-level="1"] a {
	padding-left: 4px;
	font-weight: 600;
	color: #777;
}
#toc li[data-level="2"] a {
	padding-left: 12px;
}
#toc li[data-level="3"] a {
	padding-left: 24px;
	font-size: 12px;
}
#toc li[data-level="4"] a {
	padding-left: 36px;
	font-size: 11px;
}
#toc-footer {
	padding-top: 12px;
	margin-top: 12px;
	border-top: 1px solid #222;
}
#raw-toggle {
	display: inline-block;
	color: #ccc;
	text-decoration: none;
	font-size: 12px;
	padding: 4px 8px;
	border: 1px solid #444;
	border-radius: 2px;
}
#raw-toggle:hover {
	background: #1a1a1a;
	color: #fff;
}
#toc-resizer {
	position: fixed;
	top: 0;
	left: calc(var(--toc-width) - 3px);
	width: 6px;
	height: 100vh;
	cursor: col-resize;
	background: transparent;
	z-index: 20;
}
#toc-resizer:hover {
	background: #222;
}
#toc-resizer.is-dragging {
	background: #333;
}
#main {
	margin-left: var(--toc-width);
	padding: 16px 24px;
	max-width: none;
	width: calc(100vw - var(--toc-width));
	overflow-wrap: anywhere;
	word-break: break-word;
}
pre {
	font-family: "Consolas", "Cascadia Code", "SFMono-Regular", "Menlo", "Monaco", "Courier New", monospace;
	color: #fff;
	padding-left: 16px;
	margin-bottom: 0;
	white-space: pre-wrap;
	overflow-wrap: anywhere;
	word-break: break-word;
}
code {
	font-family: "Consolas", "Cascadia Code", "SFMono-Regular", "Menlo", "Monaco", "Courier New", monospace;
	overflow-wrap: anywhere;
	word-break: break-word;
}
.copy-btn {
	display: block;
	margin-top: 6px;
	margin-left: 16px;
	padding: 4px 8px;
	background: #333;
	color: #fff;
	border: 1px solid #555;
	border-radius: 2px;
	cursor: pointer;
	font-size: 10px;
	width: fit-content;
}
.copy-btn:hover {
	background: #ccc;
}
</style>
</head>
<body>
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
<script>
const root = document.documentElement;
const resizer = document.getElementById("toc-resizer");
const MIN_TOC_WIDTH = 160;
const MAX_TOC_WIDTH = 560;
const TOC_WIDTH_STORAGE_KEY = "mclocks-md-toc-width";

const storedWidthRaw = localStorage.getItem(TOC_WIDTH_STORAGE_KEY);
if (storedWidthRaw !== null) {
	const storedWidth = Number(storedWidthRaw);
	if (Number.isFinite(storedWidth)) {
		const clamped = Math.max(MIN_TOC_WIDTH, Math.min(MAX_TOC_WIDTH, storedWidth));
		root.style.setProperty("--toc-width", `${clamped}px`);
	}
}

let isResizing = false;
const setTocWidth = (rawWidth) => {
	const clamped = Math.max(MIN_TOC_WIDTH, Math.min(MAX_TOC_WIDTH, rawWidth));
	root.style.setProperty("--toc-width", `${clamped}px`);
	localStorage.setItem(TOC_WIDTH_STORAGE_KEY, String(clamped));
};

resizer.addEventListener("mousedown", (e) => {
	e.preventDefault();
	isResizing = true;
	resizer.classList.add("is-dragging");
});

window.addEventListener("mousemove", (e) => {
	if (!isResizing) {
		return;
	}
	setTocWidth(e.clientX);
});

window.addEventListener("mouseup", () => {
	if (!isResizing) {
		return;
	}
	isResizing = false;
	resizer.classList.remove("is-dragging");
});

document.querySelectorAll("pre code").forEach((code) => {
	const pre = code.parentElement;
	if (pre.nextElementSibling?.classList.contains("copy-btn")) {
		return;
	}
	const btn = document.createElement("button");
	btn.textContent = "Copy";
	btn.className = "copy-btn";
	btn.onclick = () => {
		navigator.clipboard.writeText(code.textContent);
		btn.textContent = "Copied!";
		setTimeout(() => (btn.textContent = "Copy"), 2000);
	};
	pre.parentNode.insertBefore(btn, pre.nextSibling);
});

const tocList = document.getElementById("toc-list");
const links = tocList.querySelectorAll("a");
const headings = document.querySelectorAll("#content h1, #content h2, #content h3, #content h4");
const observer = new IntersectionObserver(
	(entries) => {
		entries.forEach((entry) => {
			if (entry.isIntersecting) {
				const id = entry.target.id;
				links.forEach((a) => {
					a.classList.toggle("active", a.getAttribute("href") === `#${id}`);
				});
			}
		});
	},
	{ rootMargin: "0px 0px -80% 0px", threshold: 0 }
);
headings.forEach((h) => observer.observe(h));
</script>
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
        Some(ext) => ext.eq_ignore_ascii_case("md"),
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
    let html = MARKDOWN_RENDER_TEMPLATE
        .replace("__PAGE_TITLE__", &page_title)
        .replace("__TOC_ITEMS__", &toc_items_html)
        .replace("__RAW_TOGGLE_HREF__", &html_escape(raw_toggle_href))
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
