//! `site.toml`: the site's identity, navigation and docs source.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SiteConfig {
    pub title: String,
    pub tagline: String,
    pub description: String,
    pub url: String,
    pub source: String,
    pub nav: Vec<NavItem>,
    pub pmacs: PmacsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NavItem {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PmacsConfig {
    /// The git remote the docs are read from.
    pub repository: String,
    /// The browsable address, for source links and release pages.
    pub web: String,
    /// `git tag --list` pattern; only plain `x.y.z` versions count.
    pub release_pattern: String,
    /// The documents a release publishes, in index order.
    pub docs: Vec<String>,
    /// Where the release notes live, read from the newest release.
    pub releases_dir: String,
}

pub fn load(root: &Path) -> Result<SiteConfig> {
    let path = root.join("site.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: {e} (run from the repository root)", path.display()))?;
    Ok(toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?)
}
