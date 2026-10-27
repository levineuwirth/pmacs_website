//! The vault: the winter sky of the Grand Central Terminal ceiling, drawn as
//! SVG at build time from `data/sky.toml`.
//!
//! Everything is drawn here except the ring, which marks where the visitor's
//! own night falls and so is placed by `static/js/site.js`; without the
//! script the sky is whole and the ring is simply absent. Colors come from
//! the page's CSS tokens through class names, so one drawing serves all
//! three ceilings, and the lit stars twinkle by CSS animation.

use crate::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::Path;

/// Obliquity of the ecliptic, J2000, in degrees.
const OBLIQUITY: f64 = 23.4393;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub patch: [f64; 2],
    pub view: View,
    pub constellation: Vec<Constellation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct View {
    pub ra0: f64,
    pub span: f64,
    pub dec_top: f64,
    pub dec_bottom: f64,
    pub width: f64,
    pub height: f64,
    pub field_seed: u32,
    pub field_count: usize,
    pub light_seed: u32,
    pub lit_magnitude: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constellation {
    pub name: String,
    #[serde(default)]
    pub mirror: bool,
    /// `[name, right ascension (h), declination (deg), magnitude]`.
    pub stars: Vec<(String, f64, f64, f64)>,
    pub lines: Vec<(String, String)>,
}

pub fn load(root: &Path) -> Result<Catalog> {
    let path = root.join("data/sky.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let catalog: Catalog = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    for c in &catalog.constellation {
        for (a, b) in &c.lines {
            for end in [a, b] {
                if !c.stars.iter().any(|s| &s.0 == end) {
                    return Err(format!(
                        "{}: {} draws a line to `{end}`, which it does not list",
                        path.display(),
                        c.name
                    )
                    .into());
                }
            }
        }
    }
    Ok(catalog)
}

impl View {
    /// Degrees of right ascension from the left edge. A star past the right
    /// edge wraps to the left only beyond 300 degrees.
    pub fn u(&self, ra_hours: f64) -> f64 {
        let d = (ra_hours - self.ra0).rem_euclid(24.0) * 15.0;
        if d > 300.0 { d - 360.0 } else { d }
    }

    /// viewBox units per degree of declination.
    pub fn pyd(&self) -> f64 {
        self.height / (self.dec_top - self.dec_bottom)
    }

    pub fn x(&self, u: f64) -> f64 {
        u * self.width / self.span
    }

    pub fn y(&self, dec: f64) -> f64 {
        (self.dec_top - dec) * self.pyd()
    }
}

/// Ecliptic longitude in degrees to right ascension (hours) and
/// declination (degrees).
pub fn ecliptic_to_equatorial(lambda: f64) -> (f64, f64) {
    let (l, e) = (lambda.to_radians(), OBLIQUITY.to_radians());
    let ra = (l.sin() * e.cos()).atan2(l.cos()).to_degrees() / 15.0;
    (ra.rem_euclid(24.0), (e.sin() * l.sin()).asin().to_degrees())
}

/// mulberry32, the generator the design study drew its sky with, so a seed
/// draws the same field here as there.
pub struct Mulberry32(u32);

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let a = self.0;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

/// Where a constellation sits: the fraction of the view's width at its
/// center, and the declination of its center.
pub fn locate(cat: &Catalog, name: &str) -> Option<(f64, f64)> {
    let v = &cat.view;
    let c = cat
        .constellation
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))?;
    let n = c.stars.len() as f64;
    let u = c.stars.iter().map(|s| v.u(s.1)).sum::<f64>() / n;
    let (lo, hi) = c.stars.iter().fold((f64::MAX, f64::MIN), |(lo, hi), s| {
        (lo.min(s.2), hi.max(s.2))
    });
    Some((u / v.span, (lo + hi) / 2.0))
}

/// What one drawing of the sky shows.
pub struct Frame<'a> {
    /// Prefix for the ids in the drawing's `<defs>`, unique on its page.
    pub id: &'a str,
    /// The window, in declination, within the catalog's view.
    pub dec_top: f64,
    pub dec_bottom: f64,
    /// A constellation picked out, the others dimmed: a page's sign.
    pub highlight: Option<&'a str>,
    /// Hour marks on the equator and degree marks on the ecliptic.
    pub tick_labels: bool,
    /// Darkened ground behind the inscription.
    pub clearing: bool,
    /// Where the patch near Cancer leads, if it is drawn.
    pub patch: Option<&'a str>,
    /// The midnight ring, placed by script.
    pub ring: bool,
    pub label: &'a str,
}

/// Half the width of the two gold bands, in viewBox units.
const BAND: f64 = 4.5;

pub fn render(cat: &Catalog, f: &Frame) -> String {
    let v = &cat.view;
    let (w, y0, y1) = (v.width, v.y(f.dec_top), v.y(f.dec_bottom));
    let id = f.id;
    let mut s = String::with_capacity(200_000);
    let _ = write!(
        s,
        r#"<svg class="sky" viewBox="0 {} {} {}" preserveAspectRatio="xMidYMid slice" role="img" aria-label="{}" data-ra0="{}" data-span="{}" data-width="{}" data-dec-top="{}" data-pyd="{}">"#,
        num(y0),
        num(w),
        num(y1 - y0),
        esc(f.label),
        v.ra0,
        v.span,
        v.width,
        v.dec_top,
        v.pyd()
    );
    let _ = write!(
        s,
        concat!(
            r#"<defs><linearGradient id="{id}-ground" x1="0" y1="0" x2="0" y2="1">"#,
            r#"<stop offset="0" class="stop-edge"/><stop offset=".5" class="stop-crown"/><stop offset="1" class="stop-edge"/></linearGradient>"#,
            r#"<radialGradient id="{id}-halo"><stop offset="0" class="stop-halo"/><stop offset="1" class="stop-halo-out"/></radialGradient>"#,
            r#"<radialGradient id="{id}-clear"><stop offset="0" class="stop-clear"/><stop offset="1" class="stop-clear-out"/></radialGradient>"#,
            r#"<filter id="{id}-soot" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="6"/></filter></defs>"#
        ),
        id = id
    );
    let _ = write!(
        s,
        r#"<rect x="0" y="{}" width="{}" height="{}" fill="url(#{id}-ground)"/>"#,
        num(y0),
        num(w),
        num(y1 - y0)
    );

    // The 4-by-8-foot boards of 1944, whose outlines still show.
    let bh = (v.height / 8.5).max(34.0);
    let mut d = String::new();
    let mut y = bh;
    while y < v.height {
        if y > y0 && y < y1 {
            let _ = write!(d, "M0 {}H{}", num(y), num(w));
        }
        y += bh;
    }
    let mut x = (w / 2.0) % (2.0 * bh);
    while x < w {
        let _ = write!(d, "M{} {}V{}", num(x), num(y0), num(y1));
        x += 2.0 * bh;
    }
    let _ = write!(s, r#"<path class="boards" d="{d}"/>"#);

    // The field, grouped by brightness to keep the markup small.
    let mut rng = Mulberry32::new(v.field_seed);
    let mut buckets: BTreeMap<u32, String> = BTreeMap::new();
    for _ in 0..v.field_count {
        let (u, dec, size, glow) = (
            -100.0 + rng.next_f64() * 400.0,
            -45.0 + rng.next_f64() * 110.0,
            rng.next_f64(),
            rng.next_f64(),
        );
        if !(0.0..=v.span).contains(&u) || dec > f.dec_top + 0.5 || dec < f.dec_bottom - 0.5 {
            continue;
        }
        let r = 1.25 * (0.32 + 1.25 * size.powi(9));
        let opacity = ((0.38 + 0.5 * glow) * 20.0).round() as u32 * 5;
        let _ = write!(
            buckets.entry(opacity).or_default(),
            r#"<circle cx="{}" cy="{}" r="{r:.2}"/>"#,
            num(v.x(u)),
            num(v.y(dec))
        );
    }
    s.push_str(r#"<g class="field">"#);
    for (opacity, circles) in &buckets {
        let _ = write!(
            s,
            r#"<g opacity="{}">{circles}</g>"#,
            f64::from(*opacity) / 100.0
        );
    }
    s.push_str("</g>");

    if f.clearing {
        let _ = write!(
            s,
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="url(#{id}-clear)"/>"#,
            num(w / 2.0),
            num(y0 + (y1 - y0) * 0.78),
            num(w * 0.34),
            num(w * 0.34 * 0.42)
        );
    }

    s.push_str(&bands(v, f, w));

    let mut lights = Mulberry32::new(v.light_seed);
    let (mut figures, mut stars, mut lit) = (String::new(), String::new(), String::new());
    for c in &cat.constellation {
        let picked = f.highlight.is_none_or(|h| h.eq_ignore_ascii_case(&c.name));
        let mean = c.stars.iter().map(|s| v.u(s.1)).sum::<f64>() / c.stars.len() as f64;
        let place = |ra: f64, dec: f64| {
            let u = if c.mirror {
                2.0 * mean - v.u(ra)
            } else {
                v.u(ra)
            };
            (v.x(u), v.y(dec))
        };
        let at: HashMap<&str, (f64, f64)> = c
            .stars
            .iter()
            .map(|(k, ra, dec, _)| (k.as_str(), place(*ra, *dec)))
            .collect();
        let class = match f.highlight {
            None => "figure",
            Some(_) if picked => "figure picked",
            Some(_) => "figure dim",
        };
        let mut d = String::new();
        for (a, b) in &c.lines {
            let (p, q) = (at[a.as_str()], at[b.as_str()]);
            let _ = write!(d, "M{} {}L{} {}", num(p.0), num(p.1), num(q.0), num(q.1));
        }
        let _ = write!(figures, r#"<path class="{class}" d="{d}"/>"#);

        for (k, _, _, m) in &c.stars {
            // Every star draws its light, in catalog order, whether or not
            // this frame shows it, so a star twinkles alike on every page.
            let (on, period, flicker, phase, depth) = (
                lights.next_f64() * 0.85,
                1.0 / (0.22 + lights.next_f64() * 0.45),
                1.6 / (0.8 + lights.next_f64() * 1.1),
                lights.next_f64(),
                0.22 + lights.next_f64() * 0.22,
            );
            let (x, y) = at[k.as_str()];
            if y < y0 - 10.0 || y > y1 + 10.0 {
                continue;
            }
            let r = 1.25 * (3.2 - 0.7 * m).max(0.95) * if picked { 1.0 } else { 0.8 };
            let dim = if picked { "" } else { r#" opacity=".6""# };
            if *m <= v.lit_magnitude {
                let _ = write!(
                    lit,
                    r#"<g class="light"{dim} style="--t:{on:.2};--d:{period:.2}s;--e:{flicker:.2}s;--p:-{:.2}s;--lo:{:.2}"><circle class="halo" cx="{cx}" cy="{cy}" r="{}" fill="url(#{id}-halo)"/><circle class="core" cx="{cx}" cy="{cy}" r="{r:.2}"/></g>"#,
                    phase * period,
                    1.0 - depth,
                    num(r * 6.0),
                    cx = num(x),
                    cy = num(y)
                );
            } else {
                let _ = write!(
                    stars,
                    r#"<circle cx="{}" cy="{}" r="{r:.2}"{dim}/>"#,
                    num(x),
                    num(y)
                );
            }
        }
    }
    let _ = write!(
        s,
        r#"<g class="figures">{figures}</g><g class="stars">{stars}</g><g class="lights">{lit}</g>"#
    );

    if let Some(href) = f.patch {
        let (x, y) = (v.x(v.u(cat.patch[0])), v.y(cat.patch[1]));
        if y > y0 && y < y1 {
            let _ = write!(
                s,
                concat!(
                    r#"<a class="patch" href="{href}" aria-label="The patch near Cancer, left uncleaned as a record: the history">"#,
                    r#"<title>Left uncleaned in the 1990s restoration, near Cancer, as a record of the grime. The history.</title>"#,
                    r#"<rect class="soot" x="{x}" y="{y}" width="44" height="28" rx="3" filter="url(#{id}-soot)"/>"#,
                    r#"<rect class="patch-focus" x="{fx}" y="{fy}" width="60" height="44" rx="4"/></a>"#
                ),
                href = esc(href),
                x = num(x - 22.0),
                y = num(y - 14.0),
                fx = num(x - 30.0),
                fy = num(y - 22.0),
                id = id
            );
        }
    }
    if f.ring {
        s.push_str(r#"<g class="ring" style="display:none"><circle r="8.5"/><circle class="ring-dot" r="2"/></g>"#);
    }
    s.push_str("</svg>");
    s
}

/// The celestial equator and the ecliptic, each a gold band between two
/// hairlines, with hour marks on the one and ten-degree marks on the other.
fn bands(v: &View, f: &Frame, w: f64) -> String {
    let mut s = String::from(r#"<g class="bands">"#);
    let equator = [(0.0, v.y(0.0)), (w, v.y(0.0))];
    band(&mut s, &equator);

    let mut runs: Vec<Vec<(f64, f64)>> = vec![Vec::new()];
    let mut previous: Option<f64> = None;
    let mut lambda = 250.0_f64;
    while lambda <= 610.0 {
        let (ra, dec) = ecliptic_to_equatorial(lambda % 360.0);
        let u = v.u(ra);
        let jumped = previous.is_some_and(|p| (u - p).abs() > 30.0);
        let inside = (-5.0..=v.span + 5.0).contains(&u);
        if jumped || !inside {
            runs.push(Vec::new());
        }
        if inside {
            runs.last_mut()
                .expect("runs is never empty")
                .push((v.x(u), v.y(dec)));
        }
        previous = Some(u);
        lambda += 0.5;
    }
    for run in runs.iter().filter(|r| r.len() > 1) {
        band(&mut s, run);
    }

    let (mut ticks, mut labels) = (String::new(), String::new());
    let ye = v.y(0.0);
    for hour in 0..24 {
        let x = v.x(v.u(f64::from(hour)));
        if !(0.0..=w).contains(&x) {
            continue;
        }
        let _ = write!(ticks, "M{} {}V{}", num(x), num(ye - BAND), num(ye + BAND));
        if f.tick_labels && hour % 3 == 0 {
            let _ = write!(
                labels,
                r#"<text x="{}" y="{}">{hour}h</text>"#,
                num(x),
                num(ye - BAND - 10.0)
            );
        }
    }
    for lambda in (0..360).step_by(10) {
        let (ra, dec) = ecliptic_to_equatorial(f64::from(lambda));
        let (x, y) = (v.x(v.u(ra)), v.y(dec));
        if !(0.0..=w).contains(&x) {
            continue;
        }
        let reach = if lambda % 30 == 0 { BAND + 3.5 } else { BAND };
        let _ = write!(ticks, "M{} {}V{}", num(x), num(y - reach), num(y + reach));
        if f.tick_labels && lambda % 30 == 0 {
            let _ = write!(
                labels,
                r#"<text x="{}" y="{}">{lambda}°</text>"#,
                num(x),
                num(y + BAND + 16.0)
            );
        }
    }
    let _ = write!(s, r#"<path class="ticks" d="{ticks}"/>"#);
    if f.tick_labels {
        let _ = write!(s, r#"<g class="tick-labels">{labels}</g>"#);
    }
    s.push_str("</g>");
    s
}

fn band(s: &mut String, pts: &[(f64, f64)]) {
    let path = |offset: f64| {
        let mut d = String::new();
        for (i, (x, y)) in pts.iter().enumerate() {
            let _ = write!(
                d,
                "{}{} {}",
                if i == 0 { 'M' } else { 'L' },
                num(*x),
                num(y + offset)
            );
        }
        d
    };
    let _ = write!(
        s,
        r#"<path class="band-fill" d="{}"/><path class="band-line" d="{}{}"/>"#,
        path(0.0),
        path(-BAND),
        path(BAND)
    );
}

/// A coordinate to one decimal, without a trailing `.0`.
fn num(v: f64) -> String {
    let s = format!("{v:.1}");
    match s.strip_suffix(".0") {
        Some("-0") => "0".into(),
        Some(t) => t.into(),
        None => s,
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        load(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data/sky.toml loads")
    }

    #[test]
    fn the_left_edge_is_ra0_and_the_view_wraps_past_300_degrees() {
        let v = catalog().view;
        assert_eq!(v.u(v.ra0), 0.0);
        assert_eq!(v.u(v.ra0 - 1.0), -15.0);
        assert!((v.u(8.975) - 188.625).abs() < 1e-9);
    }

    #[test]
    fn the_ecliptic_reaches_the_solstice_at_six_hours() {
        let (ra, dec) = ecliptic_to_equatorial(90.0);
        assert!((ra - 6.0).abs() < 1e-9);
        assert!((dec - OBLIQUITY).abs() < 1e-9);
    }

    #[test]
    fn orion_alone_is_mirrored() {
        let cat = catalog();
        let mirrored: Vec<&str> = cat
            .constellation
            .iter()
            .filter(|c| c.mirror)
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(mirrored, ["Orion"]);
    }

    #[test]
    fn the_hero_draws_about_2500_field_stars_and_lights_the_bright_ones() {
        let cat = catalog();
        let svg = render(
            &cat,
            &Frame {
                id: "t",
                dec_top: cat.view.dec_top,
                dec_bottom: cat.view.dec_bottom,
                highlight: None,
                tick_labels: true,
                clearing: true,
                patch: Some("/history/"),
                ring: true,
                label: "sky",
            },
        );
        let field = svg
            .split(r#"<g class="field">"#)
            .nth(1)
            .and_then(|r| r.split("</g></g>").next())
            .unwrap_or("");
        let count = field.matches("<circle").count();
        assert!((2_300..=2_900).contains(&count), "field stars: {count}");
        let bright = cat
            .constellation
            .iter()
            .flat_map(|c| &c.stars)
            .filter(|s| s.3 <= cat.view.lit_magnitude)
            .count();
        assert_eq!(svg.matches(r#"class="light""#).count(), bright);
        assert!(svg.contains(r#"href="/history/""#));
        assert!(svg.contains(r#"class="ring""#));
    }

    #[test]
    fn a_strip_picks_out_its_sign() {
        let cat = catalog();
        let svg = render(
            &cat,
            &Frame {
                id: "s",
                dec_top: 37.0,
                dec_bottom: 5.0,
                highlight: Some("gemini"),
                tick_labels: false,
                clearing: false,
                patch: None,
                ring: false,
                label: "sky",
            },
        );
        assert_eq!(svg.matches("figure picked").count(), 1);
        assert!(!svg.contains("<text"));
        assert!(!svg.contains("patch"));
    }
}
