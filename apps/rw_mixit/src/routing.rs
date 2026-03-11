/// Application route, determined by URL hash fragment.
#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Main,
    Settings,
    About,
    Help,
}

impl Route {
    /// Parse a URL hash string (e.g. `"#/settings"`) into a `Route`.
    /// Unrecognised hashes fall back to `Route::Main`.
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/settings" => Route::Settings,
            "#/about"    => Route::About,
            "#/help"     => Route::Help,
            _            => Route::Main,
        }
    }

    /// Return the canonical hash string for this route.
    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Main     => "#/",
            Route::Settings => "#/settings",
            Route::About    => "#/about",
            Route::Help     => "#/help",
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
    fn from_hash_settings() {
        assert_eq!(Route::from_hash("#/settings"), Route::Settings);
    }

    #[test]
    fn from_hash_about() {
        assert_eq!(Route::from_hash("#/about"), Route::About);
    }

    #[test]
    fn from_hash_help() {
        assert_eq!(Route::from_hash("#/help"), Route::Help);
    }

    #[test]
    fn round_trip_all_routes() {
        let routes = [Route::Main, Route::Settings, Route::About, Route::Help];
        for route in routes {
            assert_eq!(Route::from_hash(route.to_hash()), route,
                "round-trip failed for {route:?}");
        }
    }
}
