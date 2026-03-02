/// All navigable views in the app.
#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Reader { poem_id: Option<String> },
    RecordingsList,
    RecordingDetail(String),
}

/// Parse the browser's `window.location.hash` string into a `Route`.
/// The hash includes the leading `#`, e.g. `"#/readings/abc-123"`.
pub fn parse_hash(hash: &str) -> Route {
    let fragment = hash.trim_start_matches('#');

    // Split path from query string
    let (path, query) = match fragment.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (fragment, None),
    };

    match path {
        "" | "/" => {
            let poem_id = query.and_then(|q| {
                q.split('&').find_map(|pair| {
                    let (k, v) = pair.split_once('=')?;
                    if k == "poem_id" {
                        Some(v.to_string())
                    } else {
                        None
                    }
                })
            });
            Route::Reader { poem_id }
        }
        "/readings" => Route::RecordingsList,
        p if p.starts_with("/readings/") => {
            Route::RecordingDetail(p["/readings/".len()..].to_string())
        }
        _ => Route::Reader { poem_id: None },
    }
}

/// Convert a `Route` back to a hash string (including `#`) suitable for
/// setting `window.location.hash` or using as an `href` attribute value.
pub fn route_to_hash(route: &Route) -> String {
    match route {
        Route::Reader { poem_id: None } => "#/".to_string(),
        Route::Reader {
            poem_id: Some(id),
        } => format!("#/?poem_id={id}"),
        Route::RecordingsList => "#/readings".to_string(),
        Route::RecordingDetail(id) => format!("#/readings/{id}"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_hash() {
        assert_eq!(parse_hash(""), Route::Reader { poem_id: None });
    }

    #[test]
    fn parse_hash_root() {
        assert_eq!(parse_hash("#/"), Route::Reader { poem_id: None });
    }

    #[test]
    fn parse_hash_with_poem_id() {
        assert_eq!(
            parse_hash("#/?poem_id=abc-123"),
            Route::Reader {
                poem_id: Some("abc-123".to_string())
            }
        );
    }

    #[test]
    fn parse_hash_readings_list() {
        assert_eq!(parse_hash("#/readings"), Route::RecordingsList);
    }

    #[test]
    fn parse_hash_recording_detail() {
        assert_eq!(
            parse_hash("#/readings/my-recording-id"),
            Route::RecordingDetail("my-recording-id".to_string())
        );
    }

    #[test]
    fn parse_unknown_path_falls_back_to_reader() {
        assert_eq!(
            parse_hash("#/unknown/path"),
            Route::Reader { poem_id: None }
        );
    }

    #[test]
    fn round_trip_reader() {
        let route = Route::Reader { poem_id: None };
        assert_eq!(parse_hash(&route_to_hash(&route)), route);
    }

    #[test]
    fn round_trip_reader_with_poem_id() {
        let route = Route::Reader {
            poem_id: Some("test-id".to_string()),
        };
        assert_eq!(parse_hash(&route_to_hash(&route)), route);
    }

    #[test]
    fn round_trip_recordings_list() {
        let route = Route::RecordingsList;
        assert_eq!(parse_hash(&route_to_hash(&route)), route);
    }

    #[test]
    fn round_trip_recording_detail() {
        let route = Route::RecordingDetail("abc-123".to_string());
        assert_eq!(parse_hash(&route_to_hash(&route)), route);
    }
}
