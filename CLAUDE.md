# pmacs_website agent instructions

The source of pmacs.io, a static site built by `pmacs-site`, the Rust
generator in this repository. Read this file, `docs/decisions.md` and
`docs/design.md` before changing anything. The reasoning behind the
design is `~/apocrypha/Passus/pmacs.io design (2026-10-27).md`; the study
the site was built from is kept beside it, in `~/apocrypha/Summarium/Images/`.
The site serves `[[pmacs]]`, whose planning lives in
`~/apocrypha/Negotia/pmacs roadmap.md`.

<!-- universum:begin -->
## The vault

**Never create, edit, move, or delete anything in `~/universum`.**
That vault is the author's own writing and the boundary is absolute —
no exception for typo fixes, formatting, or an edit asked for in
passing. (`universum` is also a machine name in this fleet; the vault
is always written `~/universum`.)

Write in **`~/apocrypha`** instead — same structure, agents' hand.
`~/bibliotheca` is the shared record store and is also writable.

`~/apocrypha/AGENTS.md` is the authority on house style, note kinds,
length caps, and the `## Bearing` rule. Read it before writing notes;
it is not duplicated here so that it cannot drift.

Useful from any terminal:

```bash
universum-embed find "<text>" --scope both   # semantic, over the vaults
universum-embed frontier --scope <project>   # what the readings agree on
universum-embed concordance <citekey>        # a paper across all stores
```
<!-- universum:end -->

## Always true

- The site is static. `cargo run -- build` renders `_site/`, and nothing runs
  on a server.
- pmacs's docs are rendered from pmacs at every release tag and never edited
  here (D3); a fix to a document goes to pmacs. This crate depends on the
  files a release carries, never on pmacs's crates.
- Packages are out until the owner says otherwise (D4): no package index.
- pmacs.io stays offline until the owner launches it (D6); there is no
  deploy target.
- Color only overhead; three ceilings, night the default; the wordmark is
  `pmacs`, lowercase, in Spectral roman (D1). The vault is drawn at build
  from `data/sky.toml`, so every page reads whole without JavaScript;
  `static/js/site.js` only switches the ceiling and places the ring.
- No tracking and no third-party requests: fonts are self-hosted, with no
  CDN and no analytics. A new font records its provenance and license in
  `THIRD-PARTY.md`.
- `#![forbid(unsafe_code)]`.
- Green is `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, then `cargo run -- build` and `cargo run -- check`; CI runs
  the same.
- Commit messages are `area: imperative summary` with a few tight lines of
  body, validation inside the last paragraph and never alone in it, written
  with `git commit -F <file>`. No trailers, and nothing naming an assistant
  or a session. Commits are SSH-signed; check with `git log --show-signature`.
  Never `git add .`, never `git stash`, and never `git checkout <file>` on a
  path with uncommitted edits; check `git status` for foreign work first.
- `origin` fetches from Forgejo and pushes to Forgejo and GitHub; `forgejo`
  and `github` name each alone.
