use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

pub struct Highlighter {
    ss: SyntaxSet,
    ts: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            ss: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    /// Highlight `code` for the given language. Returns HTML string.
    /// Falls back to plain text if language unknown.
    pub fn highlight(&self, code: &str, lang: &str) -> String {
        let syntax = self
            .ss
            .find_syntax_by_token(lang)
            .or_else(|| self.ss.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());

        let theme = &self.ts.themes["InspiredGitHub"];
        let mut h = HighlightLines::new(syntax, theme);

        let mut output = String::from("<pre class=\"code-block\"><code>");
        for line in syntect::util::LinesWithEndings::from(code) {
            match h.highlight_line(line, &self.ss) {
                Ok(regions) => {
                    let html =
                        styled_line_to_highlighted_html(&regions, IncludeBackground::No)
                            .unwrap_or_else(|_| line.to_string());
                    output.push_str(&html);
                }
                Err(_) => output.push_str(line),
            }
        }
        output.push_str("</code></pre>");
        output
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_rust_code() {
        let h = Highlighter::new();
        let out = h.highlight("let x = 1;", "rust");
        assert!(out.contains("<pre"));
        assert!(out.contains("let"));
    }

    #[test]
    fn highlight_unknown_lang_plain() {
        let h = Highlighter::new();
        let out = h.highlight("hello world", "xyz_unknown");
        assert!(out.contains("hello world"));
    }
}
