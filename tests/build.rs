//! The whole build, against a fixture pmacs repository made by the test.

use pmacs_site::docs::DocsSource;
use pmacs_site::{check, site};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pmacs-site-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.org",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgsign=false",
        ])
        .args(args)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

fn write(dir: &Path, path: &str, text: &str) {
    let p = dir.join(path);
    std::fs::create_dir_all(p.parent().expect("a parent")).expect("parent directory");
    std::fs::write(p, text).expect("fixture file");
}

/// Two releases and a release candidate between them.
fn fixture() -> PathBuf {
    let repo = scratch("fixture");
    git(&repo, &["init", "-q", "-b", "main"]);
    write(
        &repo,
        "README.md",
        "# fixture\n\nSee [the pins](docs/packages.md#pins) and [the source](src/lib.rs).\n",
    );
    write(
        &repo,
        "docs/packages.md",
        "# Packages\n\n## Pins\n\nBack to [the README](../README.md).\n",
    );
    write(
        &repo,
        "docs/releases/1.0.0.md",
        "# pmacs 1.0.0\n\nThe first release.\n",
    );
    write(&repo, "src/lib.rs", "");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "first"]);
    git(&repo, &["tag", "v1.0.0"]);
    write(
        &repo,
        "docs/releases/1.1.0.md",
        "# pmacs 1.1.0\n\nSee [packages](../packages.md).\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "second"]);
    git(&repo, &["tag", "v1.1.0-rc.1"]);
    git(&repo, &["tag", "v1.1.0"]);
    repo
}

#[test]
fn every_release_is_rendered_and_every_link_resolves() {
    let repo = fixture();
    let out = scratch("out").join("site");
    let report = site::build(&site::BuildOptions {
        root: root(),
        out: out.clone(),
        docs: DocsSource::Local(repo.clone()),
    })
    .expect("the site builds");
    assert_eq!(
        report.versions,
        ["1.1.0", "1.0.0"],
        "release candidates are not releases"
    );
    assert_eq!(report.release_notes, 2);

    let index = std::fs::read_to_string(out.join("docs/1.1.0/index.html"))
        .expect("the README is the index");
    assert!(
        index.contains(r#"href="/docs/1.1.0/packages.html#pins""#),
        "a published document links to its page"
    );
    assert!(
        index.contains(r#"href="https://github.com/levineuwirth/pmacs/blob/v1.1.0/src/lib.rs""#),
        "other files link to the forge at the tag"
    );
    let packages = std::fs::read_to_string(out.join("docs/1.0.0/packages.html"))
        .expect("each release has its own pages");
    assert!(packages.contains(r#"<h2 id="pins">"#));
    assert!(packages.contains(r#"href="/docs/1.0.0/index.html""#));
    let note =
        std::fs::read_to_string(out.join("history/1.1.0.html")).expect("release notes are pages");
    assert!(
        note.contains(r#"href="/docs/1.1.0/packages.html""#),
        "notes link into the newest release's docs"
    );
    assert!(!out.join("docs/1.1.0-rc.1").exists());

    let landing = std::fs::read_to_string(out.join("index.html")).expect("the landing");
    assert!(
        landing.contains(r#"class="release-num">1.1.0<"#),
        "the booth names the newest release"
    );
    assert!(
        landing.contains(r#"<svg class="sky""#),
        "the vault is drawn at build time"
    );

    check::run(&out).expect("every local link resolves");
    let _ = std::fs::remove_dir_all(&repo);
    let _ = std::fs::remove_dir_all(out.parent().expect("a parent"));
}

#[test]
fn a_build_without_docs_still_links_cleanly() {
    let out = scratch("nodocs").join("site");
    let report = site::build(&site::BuildOptions {
        root: root(),
        out: out.clone(),
        docs: DocsSource::None,
    })
    .expect("the site builds");
    assert!(report.versions.is_empty());
    assert!(
        !std::fs::read_to_string(out.join("index.html"))
            .expect("the landing")
            .contains("release-num")
    );
    check::run(&out).expect("every local link resolves");
    let _ = std::fs::remove_dir_all(out.parent().expect("a parent"));
}

#[test]
fn a_directory_it_did_not_make_is_never_cleared() {
    let out = scratch("foreign");
    std::fs::write(out.join("keep.txt"), "not the generator's").expect("a foreign file");
    let err = site::build(&site::BuildOptions {
        root: root(),
        out: out.clone(),
        docs: DocsSource::None,
    })
    .expect_err("the build refuses");
    assert!(err.to_string().contains("refusing to clear"), "{err}");
    assert!(out.join("keep.txt").exists());
    let _ = std::fs::remove_dir_all(&out);
}
