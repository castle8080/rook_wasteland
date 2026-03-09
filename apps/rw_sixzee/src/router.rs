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

/// Returns `true` when the opening-quote overlay should be shown in place of
/// the current route content.
///
/// The overlay is only relevant on the [`Route::Game`] screen — it is a
/// "welcome to your new game" affordance.  Gating on `Route::Game` ensures
/// that navigating directly to Settings or History (via a hash URL or the tab
/// bar) is never blocked by the overlay.
pub fn opening_quote_visible(show: bool, bank_ready: bool, route: &Route) -> bool {
    show && bank_ready && matches!(route, Route::Game)
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

    // ── opening_quote_visible ────────────────────────────────────────────

    #[test]
    fn opening_quote_visible_false_on_settings_route() {
        // Bug 004: overlay must NOT block non-Game routes even when
        // show=true and bank_ready=true.
        assert!(
            !opening_quote_visible(true, true, &Route::Settings),
            "opening-quote overlay must not block the Settings route"
        );
    }

    #[test]
    fn opening_quote_visible_false_on_history_detail_route() {
        let detail_route = Route::HistoryDetail { id: "123".to_string() };
        assert!(
            !opening_quote_visible(true, true, &detail_route),
            "opening-quote overlay must not block the HistoryDetail route"
        );
    }

    #[test]
    fn opening_quote_visible_false_on_history_route() {
        assert!(
            !opening_quote_visible(true, true, &Route::History),
            "opening-quote overlay must not block the History route"
        );
    }

    #[test]
    fn opening_quote_visible_true_on_game_route_when_conditions_met() {
        assert!(
            opening_quote_visible(true, true, &Route::Game),
            "opening-quote overlay should show on the Game route when both flags are set"
        );
    }

    #[test]
    fn opening_quote_visible_false_when_show_is_false() {
        assert!(!opening_quote_visible(false, true, &Route::Game));
    }

    #[test]
    fn opening_quote_visible_false_when_bank_not_ready() {
        assert!(!opening_quote_visible(true, false, &Route::Game));
    }
}
