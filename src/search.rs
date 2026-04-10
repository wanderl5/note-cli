use crate::highlighter::Highlighter;
use crate::parser::parse_file;
use crate::scanner::PageEntry;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct SearchEntry {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub url: String,
    pub body: String,
}

/// Remove HTML tags and decode common HTML entities.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;

    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
        // Skip <script>…</script> blocks entirely
        if !in_tag && html[i..].starts_with("<script") {
            in_script = true;
        }
        if in_script {
            if html[i..].starts_with("</script>") {
                in_script = false;
                i += 9;
            } else {
                i += 1;
            }
            continue;
        }

        match bytes[i] {
            b'<' => { in_tag = true; i += 1; }
            b'>' => { in_tag = false; i += 1; }
            b'&' if !in_tag => {
                // Decode common HTML entities
                let rest = &html[i..];
                if rest.starts_with("&lt;")   { out.push('<'); i += 4; }
                else if rest.starts_with("&gt;")   { out.push('>'); i += 4; }
                else if rest.starts_with("&amp;")  { out.push('&'); i += 5; }
                else if rest.starts_with("&quot;") { out.push('"'); i += 6; }
                else if rest.starts_with("&#39;")  { out.push('\''); i += 5; }
                else if rest.starts_with("&nbsp;") { out.push(' '); i += 6; }
                else { out.push('&'); i += 1; }
            }
            _ if !in_tag => {
                // Safe to index by byte here only for ASCII; use char boundary
                let ch = html[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            _ => { i += 1; }
        }
    }
    out
}

pub fn build_index(pages: &[PageEntry], highlighter: &Highlighter, dist: &Path) -> Result<()> {
    let entries: Vec<SearchEntry> = pages
        .iter()
        .filter_map(|entry| {
            let parsed = parse_file(&entry.abs_path, highlighter).ok()?;
            let plain = strip_html(&parsed.html);
            // Take up to 500 chars of plain text (sufficient for Chinese content)
            let plain_trimmed = plain.trim();
            let body: String = if plain_trimmed.chars().count() > 500 {
                plain_trimmed.chars().take(500).collect()
            } else {
                plain_trimmed.to_string()
            };
            let title = if parsed.meta.title.is_empty() {
                entry
                    .rel_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                parsed.meta.title
            };
            Some(SearchEntry {
                title,
                description: parsed.meta.description,
                tags: parsed.meta.tags,
                url: entry.url.clone(),
                body,
            })
        })
        .collect();

    let json = serde_json::to_string(&entries)?;
    std::fs::write(dist.join("search.json"), json)?;
    println!("Search index: {} entries → search.json", entries.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags() {
        let html = "<h1>Hello</h1><p>World <strong>!</strong></p>";
        let stripped = strip_html(html);
        assert!(stripped.contains("Hello"));
        assert!(stripped.contains("World"));
        assert!(!stripped.contains('<'));
    }

    #[test]
    fn strip_html_decodes_entities() {
        let html = "<p>if x &lt; 10 &amp;&amp; y &gt; 0</p>";
        let stripped = strip_html(html);
        assert!(stripped.contains("if x < 10 && y > 0"), "got: {}", stripped);
        assert!(!stripped.contains("&lt;"));
        assert!(!stripped.contains("&amp;"));
    }
}
