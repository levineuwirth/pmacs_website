# Design

pmacs.io is a terminal in both senses: Grand Central's, where the lines end
and the city begins, and the one pmacs runs in. Its ceiling is the site's
one colored element, the thing a visitor looks up at on arrival.

## One rule

Color only overhead. The vault's turquoise and gold are the only color on
the site. Everything under them is levineuwirth.org's stone and ink, and
color never encodes anything. The rule carries over levineuwirth.org's own
argument, that color does work typography should do.

## Three registers

- **Concourse.** The landing, under the full vault. The release booth below
  it holds the latest release, where it runs, and the two ways in.
- **Passages.** Informational pages (`content/*.md`). A strip of the same sky
  carries the page's sign, one constellation per page, the way
  levineuwirth.org's pages carry printer's marks. Below it is the reading
  register: a drop cap, a small-capital first line, margin notes.
- **Platforms.** The docs and the history. These are generated, plain and
  mostly monospaced, and there is no sky down here.

## Three ceilings

Night is the default: the stars light in turn, then twinkle. Day is the
painted turquoise by daylight, with smaller halos. Before the cleaning is the
ceiling under its tar and nicotine; there are no lights, because there were
none from 1945 until the restoration. A visitor who asks for reduced motion
sees the lights steady. The choice is remembered per browser, and a visitor
without JavaScript stays at night.

## Kept marks

These are details the ceiling keeps, kept here too.

- **The mirror.** The sky runs as a celestial globe shows it, right ascension
  increasing left to right, with Orion alone the right way round. That is the
  ceiling's error: a commuter noticed it within a month of the 1913 opening,
  and it was never corrected.
- **The patch.** The 1990s restoration left a dark patch near Cancer
  uncleaned, as a record. Here it leads to the history.
- **The boards.** The faint grid is the 4-by-8-foot boards of 1944, which
  replaced the water-damaged original.
- **The ring.** A ring on the ecliptic marks the point due south at local
  midnight tonight, worked out from the date alone.

The sky is the ceiling's winter zodiac, Aquarius to Cancer, with Pegasus,
Triangulum and Orion (`data/sky.toml`; J2000 positions, approximate). It
draws only constellation lines: the names were removed, and Helleu's figures
are not reproduced.

## Type

- **Spectral** for reading, as on levineuwirth.org, with its small capitals
  and oldstyle figures.
- **Fira Sans** capitals for wayfinding, the station's signage voice.
- **JetBrains Mono** for code. It is the face pmacs-gpu bundles, so code here
  is in the editor's own type.

The name is lowercase everywhere, which rules out Beaux-Arts inscriptional
capitals for the wordmark.

## Palette

| Token | Night | Day | Before the cleaning |
|---|---|---|---|
| `--vault` | `#0b2e33` | `#2d777d` | `#2b2117` |
| `--gold` | `#d8b660` | `#dcbb6a` | `#a08452` |
| `--stone` | `#101413` | `#faf8f4` | `#1b1510` |
| `--ink` | `#d6d1c7` | `#1a1a1a` | `#e3d0a8` |
| `--grime` | `#2a1e14` | `#3b2b1d` | `#0d0906` |

`static/css/site.css` holds the full set.
