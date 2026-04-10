use crate::config::PathMapper;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct NavNode {
    pub title: String,
    pub url: Option<String>,
    pub children: Vec<NavNode>,
}

#[derive(Debug, Clone)]
pub struct PageEntry {
    pub abs_path: PathBuf,
    pub rel_path: PathBuf,
    pub url: String,
}

pub struct ScanResult {
    pub nav: Vec<NavNode>,
    pub pages: Vec<PageEntry>,
}

fn dir_title(name: &str) -> String {
    PathMapper::strip_prefix(name).to_string()
}

fn file_title(name: &str) -> String {
    let s = name.strip_suffix(".md").unwrap_or(name);
    PathMapper::strip_prefix(s).to_string()
}

pub fn scan(docs_dir: &Path, mapper: &PathMapper) -> ScanResult {
    let mut pages = Vec::new();
    let mut nav = Vec::new();
    scan_dir(docs_dir, docs_dir, mapper, &mut nav, &mut pages);
    ScanResult { nav, pages }
}

fn scan_dir(
    docs_root: &Path,
    current: &Path,
    mapper: &PathMapper,
    nav_out: &mut Vec<NavNode>,
    pages_out: &mut Vec<PageEntry>,
) {
    let mut entries: Vec<_> = match std::fs::read_dir(current) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap_or_default();

        if name.starts_with('.') || name.starts_with('@') {
            continue;
        }
        if name == "index.md" || name == "索引.md" || name == "superpowers" {
            continue;
        }

        if path.is_dir() {
            let mut children = Vec::new();
            scan_dir(docs_root, &path, mapper, &mut children, pages_out);
            if !children.is_empty() {
                let rel = path.strip_prefix(docs_root).unwrap();
                let url = mapper.to_url(rel);
                nav_out.push(NavNode {
                    title: dir_title(&name),
                    url: Some(url),
                    children,
                });
            }
        } else if name.ends_with(".md") {
            let rel = path.strip_prefix(docs_root).unwrap().to_path_buf();
            let url = mapper.to_url(&rel);
            let title = file_title(&name);

            nav_out.push(NavNode {
                title,
                url: Some(url.clone()),
                children: vec![],
            });

            pages_out.push(PageEntry {
                abs_path: path,
                rel_path: rel,
                url,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn make_mapper() -> PathMapper {
        let mut m = HashMap::new();
        m.insert("编程".into(), "coding".into());
        PathMapper::new(m)
    }

    #[test]
    fn scan_finds_md_files() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        let sub = docs.join("00.编程");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("01.Hello.md"), "# Hello").unwrap();

        let mapper = make_mapper();
        let result = scan(&docs, &mapper);

        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.pages[0].url, "/coding/Hello/");
    }

    #[test]
    fn scan_skips_index_and_special() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "home").unwrap();
        fs::write(docs.join("01.Real.md"), "# Real").unwrap();

        let mapper = make_mapper();
        let result = scan(&docs, &mapper);

        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.pages[0].url, "/Real/");
    }
}
