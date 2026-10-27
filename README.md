# pmacs_website

The source of [pmacs.io](https://pmacs.io), the home of
[pmacs](https://github.com/levineuwirth/pmacs). The site is not live yet.

It is a static site, built by `pmacs-site`, a small Rust generator in this
repository. The landing is drawn under a painted vault after the ceiling of
Grand Central Terminal. `docs/design.md` has the design, and
`docs/decisions.md` the rulings behind it.

## Building

The toolchain is pinned in `rust-toolchain.toml`, at the version pmacs pins.
Run from the repository root:

```sh
cargo run -- build                          # render into _site/
cargo run -- build --pmacs ../pmacs         # read pmacs's docs from a local repository
cargo run -- build --no-docs                # the landing and pages only
cargo run -- check                          # every local link in _site/ must resolve
cargo run -- serve                          # build, then serve on http://127.0.0.1:8000/
```

Without `--pmacs`, a build clones pmacs bare and without blobs into
`.cache/`, and fetches new tags on later builds. A build clears `_site/`
first, and refuses to clear a directory it did not make.

What CI runs, on every push and pull request:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --release -- build
cargo run --release -- check
```

## Where things are

| Path | Holds |
|---|---|
| `site.toml` | the site's identity, navigation, and which pmacs documents each release publishes |
| `content/` | the site's own pages: `index.md` is the landing, every other file a page at `/<name>/` |
| `data/sky.toml` | the vault's stars, constellation lines and projection |
| `templates/` | the page templates (minijinja) |
| `static/` | CSS, the one script, fonts and the icon, copied as they are |
| `src/sky.rs` | draws the vault as SVG at build time |
| `src/docs.rs` | reads pmacs at each release tag and renders its documents |
| `src/check.rs` | the link check |

## Pages

A page in `content/` starts with TOML front matter between `+++` lines:

```toml
+++
title = "Two frontends, one editor"
description = "One line for search results."
eyebrow = "About"          # the small line above the title
sign = "Gemini"            # the constellation the page's strip picks out
note = "A margin note."
+++
```

`sign` names a constellation in `data/sky.toml`.

## Docs

Every pmacs release, meaning a tag `vX.Y.Z` (release candidates are left
out), publishes the documents `site.toml` lists. They appear under
`/docs/X.Y.Z/`, with `/docs/latest/` pointing at the newest. A link
between two published documents becomes a link between their pages; any
other relative link points at the file on GitHub, at the same tag. The
release notes in `docs/releases/` of the newest release become `/history/`.

The documents are pmacs's, and this repository never edits them. A fix to
one goes to pmacs.

## Contributing

Fixes are welcome. The site's design is the owner's. Documentation
contributions belong in [pmacs](https://github.com/levineuwirth/pmacs).

## License

[MIT](LICENSE), as levineuwirth.org's code is. That covers everything in
this repository except the fonts, which are under the SIL Open Font
License (`THIRD-PARTY.md` has their provenance). The pmacs documents the
build renders are pmacs's, under pmacs's license.
