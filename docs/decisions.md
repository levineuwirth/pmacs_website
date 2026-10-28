# Decisions

The owner's rulings on the site, dated. A later decision that changes an
earlier one says so and leaves the earlier one as written.

**D1 (2026-10-27). The design is the vault.** pmacs.io is presented under a
painted vault after the ceiling of Grand Central Terminal's Main Concourse,
over the reading register of levineuwirth.org. Color appears only overhead.
There are three ceilings, night (the default), day, and before the cleaning,
and the lit stars twinkle. The wordmark is `pmacs`, lowercase, in Spectral
roman. `docs/design.md` has the rules.

**D2 (2026-10-27). The site is static, built by a small Rust generator in
this repository.** One language with pmacs, so whoever maintains pmacs can
maintain the site, and the vault and each page's constellation are drawn at
build time. Hakyll, which builds levineuwirth.org, and Zola were considered.

**D3 (2026-10-27). The docs of every release are published, rendered from
pmacs and never edited here.** The documents live in pmacs, where its tests
keep them true. The site reads them at each release tag (plain `x.y.z`, so
release candidates are left out) as files, and depends on nothing in pmacs
but those files: never on its crates. A fix to a document goes to pmacs.

**D4 (2026-10-27). Packages are out until after launch.** There is no
package index. pmacs's own documents about packages are docs and are
published under D3. This holds until the owner says otherwise.

**D5 (2026-10-27). The repository is public on GitHub and on Forgejo.**
`origin` fetches from Forgejo and pushes to both, as levineuwirth.org's does.

**D6 (2026-10-27). pmacs.io stays offline until the owner launches it**, as
pmacs's own D41 rules. There is no deploy target yet.

**D7 (2026-10-27). The site is the owner's authored work.** It takes fixes;
its design is the owner's. Documentation contributions go to pmacs. Package
authors document their packages in their own repositories.

**D8 (2026-10-27). The browser frontend comes later, on its own route.** pmacs
will have a WASM phase that runs the editor in a browser, after pmacs.io goes
live. The frontend is built in pmacs, and the site serves the release's
build. If it uses threads, browsers require cross-origin isolation (the
`Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers),
which limit what a page may embed. So it gets a route of its own with its
own headers, and the rest of the site is untouched.

**D9 (2026-10-27). The license is MIT, as levineuwirth.org's code is.** It
covers everything in this repository: code, templates, styles, the star
catalog, the pages and these documents. The exceptions are the third-party
fonts, under the SIL Open Font License, and the pmacs documents the build
renders, which stay under pmacs's license.

**D10 (2026-10-28). pmacs.io is served from the owner's VPS, beside
levineuwirth.org.** It is a static nginx virtual host of its own, so the
browser frontend's route (D8) can carry its own headers. Deploys go through a
user that can write only the site's directory, and release archives stay on
GitHub Releases. Netlify and Cloudflare Pages were considered, and GitHub
Pages was ruled out because it cannot set D8's headers. The site moves to a
server of its own if pmacs gains maintainers who need access to it, or a
package index that needs a running service.
