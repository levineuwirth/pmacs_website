//! The site's own pages: `content/*.md`, each with TOML front matter
//! between `+++` lines. `index.md` is the landing; every other file is a
//! passage, an informational page under a strip of the vault.

use crate::Result;
use crate::markdown;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Front {
    title: String,
    description: Option<String>,
    /// The constellation the page's strip picks out.
    sign: Option<String>,
    /// The small line above the title.
    eyebrow: Option<String>,
    /// A margin note.
    note: Option<String>,
    /// The landing's release booth: what runs where, in Markdown.
    booth: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub sign: Option<String>,
    pub eyebrow: Option<String>,
    pub note: Option<String>,
    pub booth_html: Option<String>,
    pub html: String,
}

pub fn load(root: &Path) -> Result<Vec<Page>> {
    let dir = root.join("content");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    let mut pages = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)?;
        let (front, body) =
            split(&text).ok_or_else(|| format!("{}: no `+++` front matter", path.display()))?;
        let front: Front = toml::from_str(front).map_err(|e| format!("{}: {e}", path.display()))?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("a content file name is not UTF-8")?
            .to_string();
        let plain = |src: &str| markdown::render(src, &|d, _| d.to_string()).html;
        pages.push(Page {
            slug,
            title: front.title,
            description: front.description,
            sign: front.sign,
            eyebrow: front.eyebrow,
            note: front.note,
            booth_html: front.booth.as_deref().map(plain),
            html: plain(body),
        });
    }
    Ok(pages)
}

fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("+++\n")?;
    let end = rest.find("\n+++\n")?;
    Some((&rest[..end], &rest[end + 5..]))
}
