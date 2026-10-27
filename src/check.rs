//! Every local `href` and `src` in the output must resolve to a file.

use crate::Result;
use std::path::{Path, PathBuf};

pub fn run(out: &Path) -> Result<()> {
    if !out.is_dir() {
        return Err(format!("{} does not exist; run `build` first", out.display()).into());
    }
    let mut pages = Vec::new();
    collect_html(out, &mut pages)?;
    let (mut links, mut broken) = (0, Vec::new());
    for page in &pages {
        let text = std::fs::read_to_string(page)?;
        for target in attribute_values(&text) {
            if is_external(&target) {
                continue;
            }
            links += 1;
            if !resolves(out, page, &target) {
                broken.push(format!(
                    "{}: {target}",
                    page.strip_prefix(out).unwrap_or(page).display()
                ));
            }
        }
    }
    if broken.is_empty() {
        println!(
            "checked {links} local links in {} pages: all resolve",
            pages.len()
        );
        Ok(())
    } else {
        for b in &broken {
            eprintln!("broken link in {b}");
        }
        Err(format!("{} of {links} local links do not resolve", broken.len()).into())
    }
}

fn collect_html(dir: &Path, into: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_html(&path, into)?;
        } else if path.extension().is_some_and(|x| x == "html") {
            into.push(path);
        }
    }
    into.sort();
    Ok(())
}

/// The values of every `href="…"` and `src="…"` attribute.
fn attribute_values(html: &str) -> Vec<String> {
    let mut values = Vec::new();
    for attr in [" href=\"", " src=\""] {
        let mut rest = html;
        while let Some(i) = rest.find(attr) {
            rest = &rest[i + attr.len()..];
            if let Some(end) = rest.find('"') {
                values.push(decode(&rest[..end]));
                rest = &rest[end..];
            }
        }
    }
    values
}

/// Decodes the character references an attribute value may hold, as a
/// browser does: the template engine writes `/` as `&#x2f;`.
fn decode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(end) = rest.find(';') else { break };
        let entity = &rest[1..end];
        let c = match entity {
            "amp" => Some('&'),
            "quot" => Some('"'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "apos" => Some('\''),
            _ => entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match c {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn is_external(target: &str) -> bool {
    target.is_empty()
        || target.starts_with('#')
        || target.starts_with("//")
        || ["http:", "https:", "mailto:", "data:"]
            .iter()
            .any(|s| target.starts_with(s))
}

fn resolves(out: &Path, page: &Path, target: &str) -> bool {
    let path = target.split(['#', '?']).next().unwrap_or(target);
    let base = if path.starts_with('/') {
        out.to_path_buf()
    } else {
        page.parent().unwrap_or(out).to_path_buf()
    };
    let candidate = base.join(path.trim_start_matches('/'));
    candidate.is_file() || candidate.join("index.html").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_both_attributes_and_skips_the_outside_world() {
        let values = attribute_values(
            r#"<a href="/docs/">d</a><img src="x.svg"><a href="https://e.org/?a=1&amp;b=2">e</a>"#,
        );
        assert_eq!(values, ["/docs/", "https://e.org/?a=1&b=2", "x.svg"]);
        assert!(is_external("https://e.org/"));
        assert!(is_external("#top"));
        assert!(!is_external("/history/"));
    }

    #[test]
    fn character_references_decode_as_a_browser_would() {
        assert_eq!(decode("&#x2f;docs&#x2f;2.0.0&#x2f;"), "/docs/2.0.0/");
        assert_eq!(decode("&#47;a&amp;b&#39;"), "/a&b'");
        assert_eq!(decode("a & b"), "a & b");
    }
}
