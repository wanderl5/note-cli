use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub site: SiteConfig,
    #[serde(default)]
    pub path_map: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SiteConfig {
    pub title: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_docs_dir")]
    pub docs_dir: PathBuf,
    #[serde(default = "default_dist_dir")]
    pub dist_dir: PathBuf,
}

fn default_base_url() -> String {
    "/".into()
}
fn default_docs_dir() -> PathBuf {
    PathBuf::from("docs")
}
fn default_dist_dir() -> PathBuf {
    PathBuf::from("dist")
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;
        let mut cfg: Config = toml::from_str(&text)?;
        // Resolve relative paths against the config file's parent directory
        let base = path.parent().unwrap_or(Path::new("."));
        cfg.site.docs_dir = base.join(&cfg.site.docs_dir).canonicalize()
            .unwrap_or_else(|_| base.join(&cfg.site.docs_dir));
        cfg.site.dist_dir = base.join(&cfg.site.dist_dir);
        Ok(cfg)
    }
}

pub struct PathMapper {
    map: HashMap<String, String>,
}

impl PathMapper {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self { map }
    }

    /// Strip leading digits+dot prefix: "00.编程" → "编程"
    pub fn strip_prefix(s: &str) -> &str {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i > 0 && i < bytes.len() && bytes[i] == b'.' {
            &s[i + 1..]
        } else {
            s
        }
    }

    pub fn map_segment(&self, segment: &str) -> String {
        let stripped = Self::strip_prefix(segment);
        self.map
            .get(stripped)
            .cloned()
            .unwrap_or_else(|| stripped.to_string())
    }

    /// Convert a relative path from docs/ to a URL path.
    /// "00.编程/02.Java/Assert.md" → "/coding/java/Assert/"
    pub fn to_url(&self, rel: &Path) -> String {
        let parts: Vec<String> = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .map(|s| {
                let s = s.strip_suffix(".md").unwrap_or(s);
                self.map_segment(s)
            })
            .filter(|s| !s.is_empty() && s != "index")
            .collect();
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", parts.join("/"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapper() -> PathMapper {
        let mut m = HashMap::new();
        m.insert("编程".into(), "coding".into());
        m.insert("Java".into(), "java".into());
        PathMapper::new(m)
    }

    #[test]
    fn strip_prefix_digits() {
        assert_eq!(PathMapper::strip_prefix("00.编程"), "编程");
        assert_eq!(PathMapper::strip_prefix("10.IDE"), "IDE");
        assert_eq!(PathMapper::strip_prefix("Java"), "Java");
    }

    #[test]
    fn map_segment_known() {
        let m = mapper();
        assert_eq!(m.map_segment("00.编程"), "coding");
        assert_eq!(m.map_segment("02.Java"), "java");
    }

    #[test]
    fn map_segment_unknown_passthrough() {
        let m = mapper();
        assert_eq!(m.map_segment("03.Hibernate"), "Hibernate");
    }

    #[test]
    fn to_url_full_path() {
        let m = mapper();
        let p = Path::new("00.编程/02.Java/Assert.md");
        assert_eq!(m.to_url(p), "/coding/java/Assert/");
    }
}
