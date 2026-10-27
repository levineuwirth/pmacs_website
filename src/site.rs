//! The build: every page of the site, written into the output directory.

use crate::config::{self, SiteConfig};
use crate::docs::{self, DocsSource, ReleaseNote, VersionDocs};
use crate::{Result, pages, sky};
use minijinja::{Environment, Value, context};
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};

pub struct BuildOptions {
    /// The repository root, holding `site.toml`.
    pub root: PathBuf,
    pub out: PathBuf,
    pub docs: DocsSource,
}

#[derive(Debug, Default)]
pub struct Report {
    pub pages: usize,
    pub versions: Vec<String>,
    pub release_notes: usize,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "built {} pages", self.pages)?;
        if self.versions.is_empty() {
            write!(f, "; no release docs")
        } else {
            write!(
                f,
                "; docs for {}; {} release notes",
                self.versions.join(", "),
                self.release_notes
            )
        }
    }
}

/// Written into the output directory, so a later build may clear it.
const MARKER: &str = ".pmacs-site";

const VAULT_LABEL: &str = "A painted night sky after the Grand Central Terminal ceiling: the winter zodiac from Aquarius to Cancer with Pegasus, Triangulum and Orion, in gold on turquoise, crossed by the ecliptic and the celestial equator, its brightest stars lit.";

#[derive(Serialize)]
struct SiteContext<'a> {
    title: &'a str,
    tagline: &'a str,
    description: &'a str,
    url: &'a str,
    source: &'a str,
    nav: Vec<NavContext<'a>>,
}

#[derive(Serialize)]
struct NavContext<'a> {
    label: &'a str,
    href: &'a str,
    current: bool,
}

#[derive(Serialize)]
struct Latest {
    version: String,
    date: String,
    download: String,
    docs: String,
}

#[derive(Serialize)]
struct DocLink<'a> {
    title: &'a str,
    route: &'a str,
}

#[derive(Serialize)]
struct OtherVersion<'a> {
    version: &'a str,
    href: String,
}

pub fn build(o: &BuildOptions) -> Result<Report> {
    let cfg = config::load(&o.root)?;
    let catalog = sky::load(&o.root)?;
    let env = environment(&o.root);
    prepare(&o.out)?;
    copy_tree(&o.root.join("static"), &o.out)?;
    let mut out = Output {
        dir: &o.out,
        pages: 0,
    };

    let (versions, notes) = match docs::Repo::open(&o.docs, &o.root, &cfg.pmacs)? {
        Some(repo) => {
            let versions = docs::collect(&repo, &cfg.pmacs)?;
            let notes = match versions.first() {
                Some(latest) => docs::history(&repo, &cfg.pmacs, latest)?,
                None => Vec::new(),
            };
            (versions, notes)
        }
        None => (Vec::new(), Vec::new()),
    };
    let latest = versions.first().map(|v| Latest {
        version: v.version.clone(),
        date: v.date.clone(),
        download: format!(
            "{}/releases/tag/{}",
            cfg.pmacs.web.trim_end_matches('/'),
            v.tag
        ),
        docs: format!("/docs/{}/", v.version),
    });

    let pages = pages::load(&o.root)?;
    let landing = pages
        .iter()
        .find(|p| p.slug == "index")
        .ok_or("content/index.md is missing")?;
    let vault = sky::render(
        &catalog,
        &sky::Frame {
            id: "vault",
            dec_top: catalog.view.dec_top,
            dec_bottom: catalog.view.dec_bottom,
            highlight: None,
            tick_labels: true,
            clearing: true,
            patch: Some("/history/"),
            ring: true,
            label: VAULT_LABEL,
        },
    );
    out.write(
        "index.html",
        env.get_template("landing.html")?.render(context! {
            site => site_context(&cfg, "/"), page => landing, sky => vault, latest => latest,
        })?,
    )?;

    for page in pages.iter().filter(|p| p.slug != "index") {
        let path = format!("/{}/", page.slug);
        let label = match &page.sign {
            Some(sign) => {
                format!("A strip of the vault's painted sky with {sign} picked out in gold.")
            }
            None => "A strip of the vault's painted sky.".to_string(),
        };
        // The strip is framed on the page's sign: centered on it in
        // declination, and on narrow screens shifted toward it.
        let (focus, center) = page
            .sign
            .as_deref()
            .and_then(|s| sky::locate(&catalog, s))
            .unwrap_or((0.5, 21.0));
        let dec_top = (center + 16.0).min(catalog.view.dec_top);
        let dec_bottom = (center - 16.0).max(catalog.view.dec_bottom);
        let aspect = catalog.view.width / (catalog.view.y(dec_bottom) - catalog.view.y(dec_top));
        let strip = sky::render(
            &catalog,
            &sky::Frame {
                id: "strip",
                dec_top,
                dec_bottom,
                highlight: page.sign.as_deref(),
                tick_labels: false,
                clearing: false,
                patch: None,
                ring: false,
                label: &label,
            },
        );
        out.write(
            &format!("{}/index.html", page.slug),
            env.get_template("passage.html")?.render(context! {
                site => site_context(&cfg, &path), page => page, sky => strip,
                focus => format!("{:.1}", focus * 100.0), aspect => format!("{aspect:.3}"),
            })?,
        )?;
    }

    write_docs(&env, &cfg, &versions, &mut out)?;
    write_history(&env, &cfg, &notes, &mut out)?;
    out.write(
        "404.html",
        env.get_template("404.html")?
            .render(context! { site => site_context(&cfg, "/404") })?,
    )?;

    Ok(Report {
        pages: out.pages,
        versions: versions.iter().map(|v| v.version.clone()).collect(),
        release_notes: notes.len(),
    })
}

fn write_docs(
    env: &Environment,
    cfg: &SiteConfig,
    versions: &[VersionDocs],
    out: &mut Output,
) -> Result<()> {
    let site = site_context(cfg, "/docs/");
    out.write(
        "docs/index.html",
        env.get_template("docs-index.html")?
            .render(context! { site => &site, versions => versions })?,
    )?;
    if let Some(latest) = versions.first() {
        let target = format!("/docs/{}/", latest.version);
        out.write(
            "docs/latest/index.html",
            env.get_template("redirect.html")?
                .render(context! { site => &site, target => target })?,
        )?;
    }
    for version in versions {
        let index: Vec<DocLink> = version
            .pages
            .iter()
            .map(|p| DocLink {
                title: &p.title,
                route: &p.route,
            })
            .collect();
        for page in &version.pages {
            let others: Vec<OtherVersion> = versions
                .iter()
                .filter(|v| v.version != version.version)
                .map(|v| OtherVersion {
                    version: &v.version,
                    href: if v.pages.iter().any(|p| p.route == page.route) {
                        format!("/docs/{}/{}", v.version, page.route)
                    } else {
                        format!("/docs/{}/", v.version)
                    },
                })
                .collect();
            out.write(
                &format!("docs/{}/{}", version.version, page.route),
                env.get_template("docs.html")?.render(context! {
                    site => &site, version => version.version, tag => version.tag, index => &index, doc => page, others => others,
                })?,
            )?;
        }
    }
    Ok(())
}

fn write_history(
    env: &Environment,
    cfg: &SiteConfig,
    notes: &[ReleaseNote],
    out: &mut Output,
) -> Result<()> {
    let site = site_context(cfg, "/history/");
    out.write(
        "history/index.html",
        env.get_template("history.html")?
            .render(context! { site => &site, notes => notes })?,
    )?;
    for note in notes {
        out.write(
            &format!("history/{}.html", note.version),
            env.get_template("release.html")?
                .render(context! { site => &site, note => note })?,
        )?;
    }
    Ok(())
}

fn site_context<'a>(cfg: &'a SiteConfig, path: &str) -> SiteContext<'a> {
    SiteContext {
        title: &cfg.title,
        tagline: &cfg.tagline,
        description: &cfg.description,
        url: &cfg.url,
        source: &cfg.source,
        nav: cfg
            .nav
            .iter()
            .map(|n| NavContext {
                label: &n.label,
                href: &n.href,
                current: n.href.starts_with('/') && path.starts_with(&n.href),
            })
            .collect(),
    }
}

fn environment(root: &Path) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(root.join("templates")));
    env.add_filter("keep_hyphenated", keep_hyphenated);
    env
}

/// Escapes `text` and keeps each hyphenated word on one line, so the
/// tagline never breaks inside "Lua-scripted".
fn keep_hyphenated(text: String) -> Value {
    let words: Vec<String> = text
        .split(' ')
        .map(|w| {
            let e = w
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            if w.contains('-') {
                format!(r#"<span class="nw">{e}</span>"#)
            } else {
                e
            }
        })
        .collect();
    Value::from_safe_string(words.join(" "))
}

struct Output<'a> {
    dir: &'a Path,
    pages: usize,
}

impl Output<'_> {
    fn write(&mut self, rel: &str, html: String) -> Result<()> {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, html).map_err(|e| format!("{}: {e}", path.display()))?;
        self.pages += 1;
        Ok(())
    }
}

/// Empties the output directory, refusing one this generator did not make.
fn prepare(out: &Path) -> Result<()> {
    if out.exists() {
        let empty = std::fs::read_dir(out)?.next().is_none();
        if !empty && !out.join(MARKER).exists() {
            return Err(format!(
                "refusing to clear {}: it is not a pmacs-site output directory",
                out.display()
            )
            .into());
        }
        std::fs::remove_dir_all(out)?;
    }
    std::fs::create_dir_all(out)?;
    std::fs::write(
        out.join(MARKER),
        "Built by pmacs-site; the next build clears this directory.\n",
    )?;
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
