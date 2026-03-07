/// Application route, determined by URL hash fragment.
///
/// `rw_teleidoscope` is a single-view app. The `Main` variant is the only
/// active route; the enum exists so future routes can be added without
/// restructuring app.rs.
#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    /// Default view — the kaleidoscope canvas with controls.
    Main,
}

impl Route {
    /// Parse a URL hash string (e.g. `"#/"`) into a `Route`.
    /// All hashes map to `Route::Main` for now.
    pub fn from_hash(_hash: &str) -> Self {
        Route::Main
    }

    /// Return the canonical hash string for this route.
    #[allow(dead_code)] // used by navigation code added in later milestones
    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Main => "#/",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hash_defaults_to_main() {
        assert_eq!(Route::from_hash(""), Route::Main);
        assert_eq!(Route::from_hash("#/"), Route::Main);
        assert_eq!(Route::from_hash("#/unknown"), Route::Main);
    }

    #[test]
    fn round_trip_main() {
        assert_eq!(Route::from_hash(Route::Main.to_hash()), Route::Main);
    }
}
