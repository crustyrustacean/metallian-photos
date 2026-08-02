// src/domain/gallery.rs

use serde::Serialize;
use uuid::Uuid;

/// A gallery is derived from the `band` field on photos — no separate table.
/// One gallery per unique band name.
#[derive(Debug, Serialize)]
pub struct Gallery {
    pub band: String,
    pub slug: String,
    pub photo_count: i64,
    pub cover_photo_id: Option<Uuid>,
}

/// Convert a band name into a URL-friendly slug.
///
/// "Iron Maiden" -> "iron-maiden"
/// "Motörhead" -> "motrhead" (non-ASCII chars get dropped)
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0' // mark for removal
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .trim_matches('-')
        .replace("--", "-")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Iron Maiden"), "iron-maiden");
    }

    #[test]
    fn slugify_handles_multiple_spaces() {
        assert_eq!(slugify("Black  Sabbath"), "black-sabbath");
    }

    #[test]
    fn slugify_handles_underscores_and_hyphens() {
        assert_eq!(slugify("Band_Name-Here"), "band-name-here");
    }

    #[test]
    fn slugify_trims_leading_trailing_dashes() {
        assert_eq!(slugify("  Iron Maiden  "), "iron-maiden");
    }

    #[test]
    fn slugify_drops_non_ascii() {
        assert_eq!(slugify("Mot\u{f6}rhead"), "motrhead");
    }
}