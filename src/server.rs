use anyhow::Result;
use std::path::{Path, PathBuf};
use tiny_http::{Response, Server};

pub fn serve(dist: &Path, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
    println!("Serving at http://localhost:{}/  (Ctrl+C to stop)", port);

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let url_path = url.split('?').next().unwrap_or("/");
        let file_path = resolve_path(dist, url_path);

        match std::fs::read(&file_path) {
            Ok(data) => {
                let mime = guess_mime(&file_path);
                let response = Response::from_data(data).with_header(
                    tiny_http::Header::from_bytes(b"Content-Type", mime.as_bytes()).unwrap(),
                );
                let _ = request.respond(response);
            }
            Err(_) => {
                let not_found = b"<h1>404 Not Found</h1>";
                let _ = request
                    .respond(Response::from_data(not_found.to_vec()).with_status_code(404));
            }
        }
    }
    Ok(())
}

/// Decode percent-encoded URL path: "%E5%9F%BA" → "基"
fn url_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

fn resolve_path(dist: &Path, url_path: &str) -> PathBuf {
    let decoded = url_decode(url_path);
    let clean = decoded.trim_start_matches('/');
    let candidate = dist.join(clean);
    if candidate.is_dir() {
        candidate.join("index.html")
    } else if candidate.exists() {
        candidate
    } else {
        dist.join("index.html")
    }
}

fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
