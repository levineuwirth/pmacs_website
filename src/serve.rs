//! A static file server for previewing the output on localhost. It serves
//! one request at a time and is not meant for anything else.

use crate::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

pub fn serve(out: &Path, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("binding 127.0.0.1:{port}: {e}"))?;
    println!(
        "serving {} at http://127.0.0.1:{port}/ (ctrl-c stops)",
        out.display()
    );
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if let Err(e) = respond(out, &mut stream) {
            eprintln!("serve: {e}");
        }
    }
    Ok(())
}

fn respond(out: &Path, stream: &mut TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header.trim().is_empty() {
            break;
        }
    }
    let mut parts = request.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    if method != "GET" && method != "HEAD" {
        return reply(
            stream,
            "405 Method Not Allowed",
            "text/plain",
            b"method not allowed\n",
            false,
        );
    }
    let path = target.split(['?', '#']).next().unwrap_or("/");
    let (status, file) = match locate(out, path) {
        Some(file) => ("200 OK", file),
        None => ("404 Not Found", out.join("404.html")),
    };
    let body = std::fs::read(&file).unwrap_or_default();
    reply(stream, status, mime(&file), &body, method == "HEAD")
}

fn locate(out: &Path, path: &str) -> Option<PathBuf> {
    if path.split('/').any(|s| s == "..") {
        return None;
    }
    let p = out.join(path.trim_start_matches('/'));
    if p.is_file() {
        Some(p)
    } else if p.join("index.html").is_file() {
        Some(p.join("index.html"))
    } else {
        None
    }
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        Some("wasm") => "application/wasm",
        Some("png") => "image/png",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn reply(
    stream: &mut TcpStream,
    status: &str,
    kind: &str,
    body: &[u8],
    head: bool,
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if !head {
        stream.write_all(body)?;
    }
    stream.flush()
}
