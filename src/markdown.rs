//! Markdown to HTML, as GitHub renders pmacs's documents: tables,
//! footnotes, task lists, and heading anchors with GitHub's slugs, so a
//! link to `packages.md#pin-kinds` keeps working once rendered here.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};
use std::collections::HashMap;

pub struct Rendered {
    pub html: String,
    /// The text of the first top-level heading.
    pub title: Option<String>,
}

/// Renders `src`, passing every link and image destination through
/// `rewrite(destination, is_image)`.
pub fn render(src: &str, rewrite: &dyn Fn(&str, bool) -> String) -> Rendered {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut events: Vec<Event> = Parser::new_ext(src, options).collect();

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut title = None;
    let mut i = 0;
    while i < events.len() {
        let level = match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => *level as u8,
            _ => {
                i += 1;
                continue;
            }
        };
        let mut text = String::new();
        let mut j = i + 1;
        while j < events.len() && !matches!(events[j], Event::End(TagEnd::Heading(_))) {
            if let Event::Text(t) | Event::Code(t) = &events[j] {
                text.push_str(t);
            }
            j += 1;
        }
        if let Event::Start(Tag::Heading { id, .. }) = &mut events[i]
            && id.is_none()
        {
            *id = Some(CowStr::from(unique(&mut seen, slug(&text))));
        }
        if level == 1 && title.is_none() {
            title = Some(text);
        }
        i = j + 1;
    }

    for event in &mut events {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                *dest_url = CowStr::from(rewrite(dest_url, false))
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                *dest_url = CowStr::from(rewrite(dest_url, true))
            }
            _ => {}
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    Rendered { html: out, title }
}

/// GitHub's heading slug: lowercase, spaces to hyphens, punctuation other
/// than `-` and `_` dropped.
pub fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn unique(seen: &mut HashMap<String, usize>, base: String) -> String {
    let n = seen.entry(base.clone()).or_insert(0);
    let id = if *n == 0 { base } else { format!("{base}-{n}") };
    *n += 1;
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(src: &str) -> Rendered {
        render(src, &|d, _| d.to_string())
    }

    #[test]
    fn slugs_follow_github() {
        assert_eq!(
            slug("Pin kinds: `version`, `branch`, `commit`"),
            "pin-kinds-version-branch-commit"
        );
        assert_eq!(
            slug("`pmacs.packages.install { ... }` — user-config scope"),
            "pmacspackagesinstall-----user-config-scope"
        );
        assert_eq!(slug("1. Package layout"), "1-package-layout");
    }

    #[test]
    fn repeated_headings_are_numbered_and_the_first_h1_is_the_title() {
        let r = plain("# pmacs\n\n## Notes\n\n## Notes\n");
        assert_eq!(r.title.as_deref(), Some("pmacs"));
        assert!(r.html.contains(r#"<h2 id="notes">"#));
        assert!(r.html.contains(r#"<h2 id="notes-1">"#));
    }

    #[test]
    fn links_and_images_pass_through_the_rewriter() {
        let r = render("[a](packages.md#x) ![b](shot.png)", &|d, image| {
            format!("{}{d}", if image { "img:" } else { "link:" })
        });
        assert!(r.html.contains(r#"href="link:packages.md#x""#));
        assert!(r.html.contains(r#"src="img:shot.png""#));
    }
}
