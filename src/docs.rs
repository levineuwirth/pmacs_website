//! pmacs's docs, read from its repository at every release tag.
//!
//! The site renders the files a release carries and never edits them: the
//! documents live in pmacs, where its tests keep them true. Reads go
//! through `git` and are read-only, so `--pmacs` may point at a working
//! checkout without touching it.

use crate::Result;
use crate::config::PmacsConfig;
use crate::markdown;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum DocsSource {
    /// No docs: the landing and passages only.
    None,
    /// A local repository, read in place.
    Local(PathBuf),
    /// The configured remote, cloned bare and without blobs into `.cache/`.
    Remote,
}

pub struct Repo {
    dir: PathBuf,
    web: String,
}

/// A release: a tag whose version is plain `major.minor.patch`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub version: String,
    key: (u64, u64, u64),
}

impl Release {
    /// `v2.0.0` is a release; `v2.0.0-rc.1` and `nightly` are not.
    pub fn parse(tag: &str) -> Option<Self> {
        let version = tag.strip_prefix('v').unwrap_or(tag);
        let mut parts = version.split('.').map(|p| p.parse::<u64>().ok());
        let key = (parts.next()??, parts.next()??, parts.next()??);
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            tag: tag.to_string(),
            version: version.to_string(),
            key,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct VersionDocs {
    pub version: String,
    pub tag: String,
    pub date: String,
    pub pages: Vec<DocPage>,
}

#[derive(Debug, Serialize)]
pub struct DocPage {
    /// The file's path in pmacs, as `docs/packages.md`.
    pub path: String,
    /// Its route under `/docs/<version>/`, as `packages.html`.
    pub route: String,
    pub title: String,
    pub html: String,
    /// The file at its tag, on the forge.
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct ReleaseNote {
    pub version: String,
    pub title: String,
    pub date: String,
    pub html: String,
    /// The forge's release page, where the archives are, if the tag exists.
    pub release_page: Option<String>,
}

impl Repo {
    pub fn open(source: &DocsSource, root: &Path, cfg: &PmacsConfig) -> Result<Option<Self>> {
        let dir = match source {
            DocsSource::None => return Ok(None),
            DocsSource::Local(path) => path.clone(),
            DocsSource::Remote => {
                let dir = root.join(".cache/pmacs.git");
                if dir.exists() {
                    git(&dir, &["fetch", "--quiet", "--force", "--tags", "origin"])?;
                } else {
                    std::fs::create_dir_all(root.join(".cache"))?;
                    let target = dir.to_str().ok_or("the cache path is not UTF-8")?;
                    git(
                        root,
                        &[
                            "clone",
                            "--quiet",
                            "--bare",
                            "--filter=blob:none",
                            &cfg.repository,
                            target,
                        ],
                    )?;
                }
                dir
            }
        };
        git(&dir, &["rev-parse", "--git-dir"])
            .map_err(|e| format!("{} is not a git repository: {e}", dir.display()))?;
        Ok(Some(Self {
            dir,
            web: cfg.web.trim_end_matches('/').to_string(),
        }))
    }

    /// Every release, newest first.
    pub fn releases(&self, pattern: &str) -> Result<Vec<Release>> {
        let mut releases: Vec<Release> = git(&self.dir, &["tag", "--list", pattern])?
            .lines()
            .filter_map(Release::parse)
            .collect();
        releases.sort_by_key(|r| std::cmp::Reverse(r.key));
        Ok(releases)
    }

    pub fn files(&self, tag: &str) -> Result<BTreeSet<String>> {
        Ok(git(&self.dir, &["ls-tree", "-r", "--name-only", tag])?
            .lines()
            .map(str::to_string)
            .collect())
    }

    pub fn read(&self, tag: &str, path: &str) -> Result<String> {
        git(&self.dir, &["show", &format!("{tag}:{path}")])
    }

    /// The date of the commit a tag names, `YYYY-MM-DD`.
    pub fn date(&self, tag: &str) -> Result<String> {
        Ok(git(&self.dir, &["log", "-1", "--format=%cs", tag])?
            .trim()
            .to_string())
    }
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("running git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The route of a pmacs file under `/docs/<version>/`: the README is the
/// index, `docs/` is dropped, names are lowercased.
pub fn route(path: &str) -> String {
    if path == "README.md" {
        return "index.html".into();
    }
    let p = path.strip_prefix("docs/").unwrap_or(path);
    format!("{}.html", p.strip_suffix(".md").unwrap_or(p).to_lowercase())
}

/// What a link in one document can reach.
pub struct LinkContext<'a> {
    pub web: &'a str,
    pub tag: &'a str,
    pub version: &'a str,
    /// The document the link is in.
    pub from: &'a str,
    /// The documents this release publishes.
    pub published: &'a BTreeSet<String>,
    /// Every file at the tag.
    pub files: &'a BTreeSet<String>,
}

/// A link to a published document becomes its page here; any other
/// relative link points at the file on the forge, at the same tag.
pub fn rewrite(dest: &str, image: bool, cx: &LinkContext) -> String {
    if dest.is_empty() || dest.starts_with('#') || dest.starts_with("//") || has_scheme(dest) {
        return dest.to_string();
    }
    let (path, fragment) = match dest.find('#') {
        Some(i) => (&dest[..i], &dest[i..]),
        None => (dest, ""),
    };
    let path = path.split('?').next().unwrap_or(path);
    let base = if path.starts_with('/') {
        ""
    } else {
        parent(cx.from)
    };
    let Some(target) = normalize(base, path.trim_start_matches('/')) else {
        return dest.to_string();
    };
    if image {
        return format!("{}/raw/{}/{target}", cx.web, cx.tag);
    }
    if cx.published.contains(&target) {
        return format!("/docs/{}/{}{fragment}", cx.version, route(&target));
    }
    let kind = if cx.files.contains(&target) {
        "blob"
    } else {
        "tree"
    };
    format!("{}/{kind}/{}/{target}{fragment}", cx.web, cx.tag)
}

fn has_scheme(dest: &str) -> bool {
    match dest.find(':') {
        Some(i) => {
            let scheme = &dest[..i];
            scheme
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        }
        None => false,
    }
}

fn parent(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..i])
}

/// Joins `rel` onto `base` and resolves `.` and `..`; `None` if it climbs
/// above the repository root.
fn normalize(base: &str, rel: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
    for p in rel.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            p => parts.push(p),
        }
    }
    Some(parts.join("/"))
}

/// The docs of every release, newest first.
pub fn collect(repo: &Repo, cfg: &PmacsConfig) -> Result<Vec<VersionDocs>> {
    let mut all = Vec::new();
    for release in repo.releases(&cfg.release_pattern)? {
        let files = repo.files(&release.tag)?;
        let published: BTreeSet<String> = cfg
            .docs
            .iter()
            .filter(|p| files.contains(*p))
            .cloned()
            .collect();
        let mut pages = Vec::new();
        for path in cfg.docs.iter().filter(|p| published.contains(*p)) {
            let cx = LinkContext {
                web: &repo.web,
                tag: &release.tag,
                version: &release.version,
                from: path,
                published: &published,
                files: &files,
            };
            let rendered = markdown::render(&repo.read(&release.tag, path)?, &|d, image| {
                rewrite(d, image, &cx)
            });
            pages.push(DocPage {
                path: path.clone(),
                route: route(path),
                title: rendered.title.unwrap_or_else(|| path.clone()),
                html: rendered.html,
                source: format!("{}/blob/{}/{path}", repo.web, release.tag),
            });
        }
        let date = repo.date(&release.tag)?;
        all.push(VersionDocs {
            version: release.version,
            tag: release.tag,
            date,
            pages,
        });
    }
    Ok(all)
}

/// The release notes the newest release carries, newest first, their links
/// resolved against that release's docs.
pub fn history(repo: &Repo, cfg: &PmacsConfig, latest: &VersionDocs) -> Result<Vec<ReleaseNote>> {
    let files = repo.files(&latest.tag)?;
    let published: BTreeSet<String> = latest.pages.iter().map(|p| p.path.clone()).collect();
    let releases = repo.releases(&cfg.release_pattern)?;
    let prefix = format!("{}/", cfg.releases_dir.trim_end_matches('/'));
    let mut notes: Vec<(Option<Release>, ReleaseNote)> = Vec::new();
    for path in files
        .iter()
        .filter(|f| f.starts_with(&prefix) && f.ends_with(".md"))
    {
        let stem = path[prefix.len()..].trim_end_matches(".md");
        if stem.contains('/') {
            continue;
        }
        let cx = LinkContext {
            web: &repo.web,
            tag: &latest.tag,
            version: &latest.version,
            from: path,
            published: &published,
            files: &files,
        };
        let rendered = markdown::render(&repo.read(&latest.tag, path)?, &|d, image| {
            rewrite(d, image, &cx)
        });
        let tagged = releases.iter().find(|r| r.version == stem).cloned();
        let date = match &tagged {
            Some(r) => repo.date(&r.tag)?,
            None => String::new(),
        };
        let release_page = tagged
            .as_ref()
            .map(|r| format!("{}/releases/tag/{}", repo.web, r.tag));
        notes.push((
            Release::parse(stem),
            ReleaseNote {
                version: stem.to_string(),
                title: rendered.title.unwrap_or_else(|| stem.to_string()),
                date,
                html: rendered.html,
                release_page,
            },
        ));
    }
    notes.sort_by_key(|(release, _)| std::cmp::Reverse(release.as_ref().map(|r| r.key)));
    Ok(notes.into_iter().map(|(_, n)| n).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_versions_are_releases() {
        assert_eq!(
            Release::parse("v2.0.0").map(|r| r.version),
            Some("2.0.0".into())
        );
        assert!(Release::parse("v2.0.0-rc.1").is_none());
        assert!(Release::parse("v1.1").is_none());
        assert!(Release::parse("nightly").is_none());
    }

    #[test]
    fn routes_drop_docs_and_make_the_readme_the_index() {
        assert_eq!(route("README.md"), "index.html");
        assert_eq!(route("docs/packages.md"), "packages.html");
        assert_eq!(route("CONTRIBUTING.md"), "contributing.html");
    }

    #[test]
    fn links_resolve_to_pages_or_to_the_forge_at_the_same_tag() {
        let published: BTreeSet<String> =
            ["README.md", "docs/packages.md"].map(String::from).into();
        let files: BTreeSet<String> = [
            "README.md",
            "docs/packages.md",
            "src/main.rs",
            "audit/rules.scm",
        ]
        .map(String::from)
        .into();
        let cx = LinkContext {
            web: "https://forge/p",
            tag: "v2.0.0",
            version: "2.0.0",
            from: "docs/packages.md",
            published: &published,
            files: &files,
        };
        assert_eq!(
            rewrite("../README.md#install", false, &cx),
            "/docs/2.0.0/index.html#install"
        );
        assert_eq!(
            rewrite("packages.md", false, &cx),
            "/docs/2.0.0/packages.html"
        );
        assert_eq!(
            rewrite("../src/main.rs", false, &cx),
            "https://forge/p/blob/v2.0.0/src/main.rs"
        );
        assert_eq!(
            rewrite("../audit/", false, &cx),
            "https://forge/p/tree/v2.0.0/audit"
        );
        assert_eq!(
            rewrite("https://example.org/x.md", false, &cx),
            "https://example.org/x.md"
        );
        assert_eq!(rewrite("#pin-kinds", false, &cx), "#pin-kinds");
        assert_eq!(rewrite("../../outside.md", false, &cx), "../../outside.md");
        assert_eq!(
            rewrite("shot.png", true, &cx),
            "https://forge/p/raw/v2.0.0/docs/shot.png"
        );
    }
}
