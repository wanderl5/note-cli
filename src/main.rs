mod config;
mod highlighter;
mod parser;
mod renderer;
mod scanner;
mod search;
mod server;

use clap::{Parser, Subcommand};
use config::{Config, PathMapper};
use std::path::PathBuf;
use tera::Tera;

#[derive(Parser)]
#[command(name = "note-cli", about = "Static site generator for Markdown notes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Path to note.toml config file
    #[arg(short, long, default_value = "note.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Build static site into dist/
    Build,
    /// Build and serve locally
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Generate note.toml template in current directory
    Init,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config;

    match cli.command {
        Commands::Build => {
            let cfg = Config::load(&config_path)?;
            build(&cfg)?;
            Ok(())
        }
        Commands::Serve { port } => {
            let cfg = Config::load(&config_path)?;
            build(&cfg)?;
            let dist_dir = cfg.site.dist_dir.clone();
            server::serve(&dist_dir, port)?;
            Ok(())
        }
        Commands::Init => {
            let template = include_str!("../note.toml.template");
            std::fs::write("note.toml", template)?;
            println!("Created note.toml — edit path_map as needed");
            Ok(())
        }
    }
}

/// Build Tera instance with templates embedded in the binary.
fn make_tera() -> anyhow::Result<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_template("base.html", include_str!("../templates/base.html"))?;
    tera.add_raw_template("page.html", include_str!("../templates/page.html"))?;
    tera.add_raw_template("home.html", include_str!("../templates/home.html"))?;
    tera.add_raw_template("category.html", include_str!("../templates/category.html"))?;
    Ok(tera)
}

fn build(cfg: &Config) -> anyhow::Result<()> {
    let mapper = PathMapper::new(cfg.path_map.clone());
    let docs_dir = &cfg.site.docs_dir;
    let dist_dir = &cfg.site.dist_dir;

    println!("Scanning {}...", docs_dir.display());
    let scan = scanner::scan(docs_dir, &mapper);
    println!("Found {} pages", scan.pages.len());

    let tera = make_tera()?;
    let highlighter = highlighter::Highlighter::new();

    std::fs::create_dir_all(dist_dir)?;

    println!("Rendering pages...");
    renderer::render_all(cfg, &scan, &tera, &highlighter, dist_dir)?;

    let nav_json = serde_json::to_value(&scan.nav)?;
    renderer::render_home(cfg, &scan, &tera, &nav_json, dist_dir)?;
    renderer::render_categories(cfg, &scan.nav, &tera, &nav_json, dist_dir)?;

    // Copy static assets from binary's adjacent static/ dir if present,
    // otherwise embed the bundled ones
    let static_src = PathBuf::from("static");
    renderer::copy_static(&static_src, dist_dir)?;

    println!("Building search index...");
    search::build_index(&scan.pages, &highlighter, dist_dir)?;

    println!("Done → {}", dist_dir.display());
    Ok(())
}
