#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Game,
    History,
    HistoryDetail { id: String },
    Settings,
}

pub fn parse_hash(hash: &str) -> Route {
    match hash.trim_start_matches('#').trim_start_matches('/') {
        "" | "game" => Route::Game,
        "history" => Route::History,
        s if s.starts_with("history/") => Route::HistoryDetail {
            id: s["history/".len()..].to_owned(),
        },
        "settings" => Route::Settings,
        _ => Route::Game,
    }
}

/// Update `window.location.hash`, which triggers the `hashchange` event and
/// causes the `App` listener to sync the active `Route` signal.
#[cfg(target_arch = "wasm32")]
pub fn navigate(route: &Route) {
    let hash = match route {
        Route::Game => "#/game".to_string(),
        Route::History => "#/history".to_string(),
        Route::HistoryDetail { id } => format!("#/history/{id}"),
        Route::Settings => "#/settings".to_string(),
    };
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(&hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_hash_is_game() {
        assert_eq!(parse_hash(""), Route::Game);
    }

    #[test]
    fn test_hash_slash_is_game() {
        assert_eq!(parse_hash("#/"), Route::Game);
        assert_eq!(parse_hash("#"), Route::Game);
    }

    #[test]
    fn test_hash_game() {
        assert_eq!(parse_hash("#/game"), Route::Game);
        assert_eq!(parse_hash("game"), Route::Game);
    }

    #[test]
    fn test_hash_history() {
        assert_eq!(parse_hash("#/history"), Route::History);
        assert_eq!(parse_hash("history"), Route::History);
    }

    #[test]
    fn test_hash_history_detail() {
        assert_eq!(
            parse_hash("#/history/abc-123"),
            Route::HistoryDetail {
                id: "abc-123".to_string()
            }
        );
        assert_eq!(
            parse_hash("history/some-uuid-here"),
            Route::HistoryDetail {
                id: "some-uuid-here".to_string()
            }
        );
    }

    #[test]
    fn test_hash_settings() {
        assert_eq!(parse_hash("#/settings"), Route::Settings);
        assert_eq!(parse_hash("settings"), Route::Settings);
    }

    #[test]
    fn test_unknown_hash_falls_back_to_game() {
        assert_eq!(parse_hash("#/unknown"), Route::Game);
        assert_eq!(parse_hash("anything_else"), Route::Game);
        assert_eq!(parse_hash("#/GAME"), Route::Game); // case-sensitive
    }
}
