use crate::highlighter::Highlighter;
use anyhow::{Context, Result};
use pulldown_cmark::{html, Options, Parser as MdParser};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Default, Deserialize)]
pub struct Frontmatter {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct ParsedPage {
    pub meta: Frontmatter,
    pub html: String,
}

/// Split "---\n<yaml>\n---\n<body>" into (yaml_str, body_str).
pub fn split_frontmatter(content: &str) -> (&str, &str) {
    if !content.starts_with("---") {
        return ("", content);
    }
    let rest = &content[3..];
    let rest = if rest.starts_with('\n') { &rest[1..] } else { rest };
    if let Some(end) = rest.find("\n---") {
        let yaml = &rest[..end];
        let body = &rest[end + 4..];
        let body = if body.starts_with('\n') { &body[1..] } else { body };
        (yaml, body)
    } else {
        ("", content)
    }
}

pub fn parse_frontmatter(yaml: &str) -> Frontmatter {
    if yaml.is_empty() {
        return Frontmatter::default();
    }
    serde_yaml::from_str(yaml).unwrap_or_default()
}

/// Convert Markdown body to HTML string.
pub fn markdown_to_html(body: &str) -> String {
    let opts = Options::all();
    let parser = MdParser::new_ext(body, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

static CODE_BLOCK_RE: OnceLock<Regex> = OnceLock::new();

fn code_block_re() -> &'static Regex {
    CODE_BLOCK_RE.get_or_init(|| {
        Regex::new(r#"<pre><code(?:\s+class="language-([^"]*)")?>"#).unwrap()
    })
}

/// Post-process HTML: replace <code class="language-XXX"> blocks with
/// syntect-highlighted HTML.
pub fn apply_highlighting(html: &str, highlighter: &Highlighter) -> String {
    let re = code_block_re();
    let mut result = String::with_capacity(html.len() + html.len() / 4);
    let mut pos = 0;

    for cap in re.captures_iter(html) {
        let full_match = cap.get(0).unwrap();
        let lang = cap.get(1).map(|m| m.as_str()).unwrap_or("text");

        result.push_str(&html[pos..full_match.start()]);

        let code_start = full_match.end();
        if let Some(end_offset) = html[code_start..].find("</code></pre>") {
            let raw_code = &html[code_start..code_start + end_offset];
            let code = raw_code
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&#39;", "'");

            result.push_str(&highlighter.highlight(&code, lang));
            pos = code_start + end_offset + "</code></pre>".len();
        } else {
            result.push_str(full_match.as_str());
            pos = full_match.end();
        }
    }
    result.push_str(&html[pos..]);
    result
}

pub fn parse_file(path: &Path, highlighter: &Highlighter) -> Result<ParsedPage> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let (yaml, body) = split_frontmatter(&content);
    let meta = parse_frontmatter(yaml);
    let raw_html = markdown_to_html(body);
    let html = apply_highlighting(&raw_html, highlighter);
    Ok(ParsedPage { meta, html })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_with_frontmatter() {
        let content = "---\ntitle: Hello\n---\n# Body";
        let (yaml, body) = split_frontmatter(content);
        assert_eq!(yaml, "title: Hello");
        assert_eq!(body, "# Body");
    }

    #[test]
    fn split_without_frontmatter() {
        let content = "# Just body";
        let (yaml, body) = split_frontmatter(content);
        assert_eq!(yaml, "");
        assert_eq!(body, "# Just body");
    }

    #[test]
    fn parse_meta_fields() {
        let yaml = "title: Assert\ndescription: test desc\ntags:\n  - Java\n  - 编程";
        let meta = parse_frontmatter(yaml);
        assert_eq!(meta.title, "Assert");
        assert_eq!(meta.tags, vec!["Java", "编程"]);
    }

    #[test]
    fn markdown_renders_heading() {
        let html = markdown_to_html("# Hello World");
        assert!(html.contains("<h1>Hello World</h1>"));
    }

    #[test]
    fn markdown_renders_code_block() {
        let html = markdown_to_html("```rust\nlet x = 1;\n```");
        assert!(html.contains("<code"));
        assert!(html.contains("let x = 1;"));
    }
}
