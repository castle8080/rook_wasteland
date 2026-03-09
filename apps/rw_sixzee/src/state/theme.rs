//! Theme enum — the 6 visual themes supported by rw_sixzee.
//!
//! Each theme overrides CSS custom properties via a `data-theme` attribute on
//! `<body>`. The active theme is stored in `localStorage` under `rw_sixzee.theme`.

/// The 6 visual themes available in rw_sixzee.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    /// Nordic Minimal — off-white, slate, moss/rust accents. The default.
    #[default]
    NordicMinimal,
    /// Abyssal Depths — midnight-blue bg, bioluminescent teal accent, Cinzel display font.
    AbyssalDepths,
    /// Borg — dark charcoal, cold cyan accent, monospace body font.
    Borg,
    /// Horror — deep black, sickly green accent, near-white text, serif font.
    Horror,
    /// Renaissance — warm parchment, burnished gold, deep ultramarine accent.
    Renaissance,
    /// Pacific Northwest — forest green bg, earthy ink-wash aesthetic.
    PacificNorthwest,
}

impl Theme {
    /// Returns the `data-theme` attribute string for this theme.
    ///
    /// This is the value written to `<body data-theme="...">` and stored in
    /// `localStorage` under `rw_sixzee.theme`.
    pub fn as_data_attr_value(self) -> &'static str {
        match self {
            Theme::NordicMinimal => "nordic_minimal",
            Theme::AbyssalDepths => "abyssal_depths",
            Theme::Borg => "borg",
            Theme::Horror => "horror",
            Theme::Renaissance => "renaissance",
            Theme::PacificNorthwest => "pacific_northwest",
        }
    }

    /// Returns the human-readable display name for this theme.
    pub fn label(self) -> &'static str {
        match self {
            Theme::NordicMinimal => "Nordic Minimal",
            Theme::AbyssalDepths => "Abyssal Depths",
            Theme::Borg => "Borg",
            Theme::Horror => "Horror",
            Theme::Renaissance => "Renaissance",
            Theme::PacificNorthwest => "Pacific NW",
        }
    }

    /// Returns all 6 theme variants in display order.
    pub fn all() -> &'static [Theme] {
        &[
            Theme::NordicMinimal,
            Theme::AbyssalDepths,
            Theme::Borg,
            Theme::Horror,
            Theme::Renaissance,
            Theme::PacificNorthwest,
        ]
    }

    /// Parses a `data-theme` attribute string back into a `Theme` variant.
    ///
    /// Returns `None` for unrecognised strings; callers should fall back to
    /// `Theme::default()`.
    pub fn from_data_attr(s: &str) -> Option<Theme> {
        match s {
            "nordic_minimal" => Some(Theme::NordicMinimal),
            "abyssal_depths" => Some(Theme::AbyssalDepths),
            "borg" => Some(Theme::Borg),
            "horror" => Some(Theme::Horror),
            "renaissance" => Some(Theme::Renaissance),
            "pacific_northwest" => Some(Theme::PacificNorthwest),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_nordic_minimal() {
        assert_eq!(Theme::default(), Theme::NordicMinimal);
    }

    #[test]
    fn data_attr_round_trip() {
        for &t in Theme::all() {
            let s = t.as_data_attr_value();
            assert_eq!(
                Theme::from_data_attr(s),
                Some(t),
                "round-trip failed for {s:?}"
            );
        }
    }

    #[test]
    fn from_data_attr_unknown_returns_none() {
        assert_eq!(Theme::from_data_attr(""), None);
        assert_eq!(Theme::from_data_attr("unknown_theme"), None);
    }

    #[test]
    fn all_has_six_variants() {
        assert_eq!(Theme::all().len(), 6);
    }

    #[test]
    fn all_variants_have_unique_data_attrs() {
        let attrs: Vec<_> = Theme::all().iter().map(|t| t.as_data_attr_value()).collect();
        let mut deduped = attrs.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(attrs.len(), deduped.len());
    }
}
