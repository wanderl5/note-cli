use crate::config::Config;
use crate::highlighter::Highlighter;
use crate::parser::parse_file;
use crate::scanner::{NavNode, PageEntry, ScanResult};
use anyhow::Result;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

#[derive(Serialize)]
struct HomeCategory {
    title: String,
    url: String,
    count: usize,
}

#[derive(Serialize)]
struct CatSection {
    title: String,
    url: String,
    is_dir: bool,
    count: usize,
    pages: Vec<CatPage>,
}

#[derive(Serialize)]
struct CatPage {
    title: String,
    url: String,
    description: String,
}

pub fn render_all(
    cfg: &Config,
    scan: &ScanResult,
    tera: &Tera,
    highlighter: &Highlighter,
    dist: &Path,
) -> Result<()> {
    let nav_json = serde_json::to_value(&scan.nav)?;

    let results: Vec<Result<()>> = scan
        .pages
        .par_iter()
        .map(|entry| render_page(entry, cfg, &nav_json, tera, highlighter, dist))
        .collect();

    for r in results {
        r?;
    }
    Ok(())
}

fn render_page(
    entry: &PageEntry,
    cfg: &Config,
    nav_json: &serde_json::Value,
    tera: &Tera,
    highlighter: &Highlighter,
    dist: &Path,
) -> Result<()> {
    let parsed = parse_file(&entry.abs_path, highlighter)?;

    let title = if parsed.meta.title.is_empty() {
        entry
            .rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    } else {
        parsed.meta.title.clone()
    };

    let mut ctx = Context::new();
    ctx.insert("site_title", &cfg.site.title);
    ctx.insert("page_title", &title);
    ctx.insert("description", &parsed.meta.description);
    ctx.insert("date", &parsed.meta.date);
    ctx.insert("tags", &parsed.meta.tags);
    ctx.insert("content", &parsed.html);
    ctx.insert("nav", nav_json);
    ctx.insert("current_url", &entry.url);

    let html = tera.render("page.html", &ctx)?;

    let out_dir = dist.join(entry.url.trim_start_matches('/').trim_end_matches('/'));
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join("index.html"), html)?;
    Ok(())
}

pub fn render_home(
    cfg: &Config,
    scan: &ScanResult,
    tera: &Tera,
    nav_json: &serde_json::Value,
    dist: &Path,
) -> Result<()> {
    let categories: Vec<HomeCategory> = scan
        .nav
        .iter()
        .map(|n| HomeCategory {
            title: n.title.clone(),
            url: n.url.clone().unwrap_or_else(|| "/".into()),
            count: count_leaves(n),
        })
        .collect();

    let mut ctx = Context::new();
    ctx.insert("site_title", &cfg.site.title);
    ctx.insert("current_url", "/");
    ctx.insert("nav", nav_json);
    ctx.insert("total_pages", &scan.pages.len());
    ctx.insert("categories", &categories);

    let html = tera.render("home.html", &ctx)?;
    fs::write(dist.join("index.html"), html)?;
    Ok(())
}

/// Render index.html for every directory node in the nav tree.
pub fn render_categories(
    cfg: &Config,
    nav: &[NavNode],
    tera: &Tera,
    nav_json: &serde_json::Value,
    dist: &Path,
) -> Result<()> {
    render_dir_nodes(cfg, nav, tera, nav_json, dist)
}

fn render_dir_nodes(
    cfg: &Config,
    nodes: &[NavNode],
    tera: &Tera,
    nav_json: &serde_json::Value,
    dist: &Path,
) -> Result<()> {
    for node in nodes {
        if node.children.is_empty() {
            continue; // leaf = article, already rendered
        }
        if let Some(url) = &node.url {
            render_one_category(cfg, node, url, tera, nav_json, dist)?;
        }
        // Recurse into sub-directories
        render_dir_nodes(cfg, &node.children, tera, nav_json, dist)?;
    }
    Ok(())
}

fn render_one_category(
    cfg: &Config,
    node: &NavNode,
    url: &str,
    tera: &Tera,
    nav_json: &serde_json::Value,
    dist: &Path,
) -> Result<()> {
    // Build sections: each direct child that is a dir becomes a section header
    // Direct page children go into a top-level "pages" section
    let mut top_pages: Vec<CatPage> = Vec::new();
    let mut sections: Vec<CatSection> = Vec::new();

    for child in &node.children {
        if child.children.is_empty() {
            // direct article child
            top_pages.push(CatPage {
                title: child.title.clone(),
                url: child.url.clone().unwrap_or_default(),
                description: String::new(),
            });
        } else {
            // sub-directory
            let pages: Vec<CatPage> = collect_direct_pages(child);
            sections.push(CatSection {
                title: child.title.clone(),
                url: child.url.clone().unwrap_or_default(),
                is_dir: true,
                count: count_leaves(child),
                pages,
            });
        }
    }

    // Prepend top-level pages as an un-titled section if any
    if !top_pages.is_empty() {
        sections.insert(
            0,
            CatSection {
                title: String::new(),
                url: String::new(),
                is_dir: false,
                count: top_pages.len(),
                pages: top_pages,
            },
        );
    }

    let total: usize = sections.iter().map(|s| s.count).sum();

    let mut ctx = Context::new();
    ctx.insert("site_title", &cfg.site.title);
    ctx.insert("cat_title", &node.title);
    ctx.insert("current_url", url);
    ctx.insert("nav", nav_json);
    ctx.insert("sections", &sections);
    ctx.insert("total", &total);

    let html = tera.render("category.html", &ctx)?;

    let out_dir = dist.join(url.trim_start_matches('/').trim_end_matches('/'));
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join("index.html"), html)?;
    Ok(())
}

/// Collect only the direct article children of a node (no recursion).
fn collect_direct_pages(node: &NavNode) -> Vec<CatPage> {
    node.children
        .iter()
        .filter(|c| c.children.is_empty())
        .map(|c| CatPage {
            title: c.title.clone(),
            url: c.url.clone().unwrap_or_default(),
            description: String::new(),
        })
        .collect()
}

fn count_leaves(node: &NavNode) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

/// Write bundled static assets to dist/static/.
pub fn copy_static(static_dir: &Path, dist: &Path) -> Result<()> {
    let dest = dist.join("static");
    fs::create_dir_all(&dest)?;

    let embedded: &[(&str, &[u8])] = &[
        ("style.css", include_bytes!("../static/style.css")),
        ("search.js", include_bytes!("../static/search.js")),
        ("lightbox.js", include_bytes!("../static/lightbox.js")),
        ("mermaid.min.js", include_bytes!("../static/mermaid.min.js")),
    ];
    for (name, data) in embedded {
        fs::write(dest.join(name), data)?;
    }

    if static_dir.exists() {
        for entry in fs::read_dir(static_dir)? {
            let entry = entry?;
            let dest_file = dest.join(entry.file_name());
            fs::copy(entry.path(), dest_file)?;
        }
    }
    Ok(())
}
