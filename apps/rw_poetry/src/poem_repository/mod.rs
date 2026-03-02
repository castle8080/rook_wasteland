use gloo_net::http::Request;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PoemIndexEntry {
    pub id: String,
    pub path: String,
    pub title: String,
    pub author: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PoemIndex {
    pub version: u32,
    pub poems: Vec<PoemIndexEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PoemDetail {
    pub id: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub date: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Fetch functions
// ---------------------------------------------------------------------------

/// Fetch and parse the poem index from `/poems/poems_index.json`.
pub async fn fetch_index() -> Result<PoemIndex, String> {
    let response = Request::get("/poems/poems_index.json")
        .send()
        .await
        .map_err(|e| format!("Network error fetching index: {e}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to load poem index (HTTP {})",
            response.status()
        ));
    }

    response
        .json::<PoemIndex>()
        .await
        .map_err(|e| format!("Failed to parse poem index: {e}"))
}

/// Fetch and parse a single poem JSON file by its path from the index.
pub async fn fetch_poem(path: &str) -> Result<PoemDetail, String> {
    let response = Request::get(path)
        .send()
        .await
        .map_err(|e| format!("Network error fetching poem: {e}"))?;

    if !response.ok() {
        return Err(format!(
            "Failed to load poem '{}' (HTTP {})",
            path,
            response.status()
        ));
    }

    response
        .json::<PoemDetail>()
        .await
        .map_err(|e| format!("Failed to parse poem JSON: {e}"))
}

// ---------------------------------------------------------------------------
// Random selection
// ---------------------------------------------------------------------------

/// Pick a random entry from the index, optionally excluding the currently
/// displayed poem by ID to avoid immediate repeats.
pub fn pick_random<'a>(
    index: &'a PoemIndex,
    exclude_id: Option<&str>,
) -> Option<&'a PoemIndexEntry> {
    let candidates: Vec<&PoemIndexEntry> = index
        .poems
        .iter()
        .filter(|p| exclude_id.is_none_or(|id| p.id != id))
        .collect();

    if candidates.is_empty() {
        return index.poems.first();
    }

    // Use browser crypto via js_sys — works in WASM, no rand crate needed.
    let idx = (js_sys::Math::random() * candidates.len() as f64) as usize;
    pick_at_index(&candidates, idx)
}

/// Select from a pre-filtered candidate slice by a pre-computed index.
/// Clamps `idx` to the last valid position defensively.
/// Extracted from `pick_random` to allow deterministic unit testing without WASM.
fn pick_at_index<'a>(
    candidates: &[&'a PoemIndexEntry],
    idx: usize,
) -> Option<&'a PoemIndexEntry> {
    if candidates.is_empty() {
        return None;
    }
    candidates.get(idx.min(candidates.len() - 1)).copied()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(ids: &[&str]) -> PoemIndex {
        PoemIndex {
            version: 1,
            poems: ids
                .iter()
                .map(|id| PoemIndexEntry {
                    id: id.to_string(),
                    path: format!("/poems/{id}.json"),
                    title: format!("Title {id}"),
                    author: "Test Author".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn pick_random_returns_none_for_empty_index() {
        let index = make_index(&[]);
        assert!(pick_random(&index, None).is_none());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn pick_random_single_entry_no_exclude() {
        let index = make_index(&["a"]);
        let picked = pick_random(&index, None);
        assert_eq!(picked.map(|p| p.id.as_str()), Some("a"));
    }

    #[test]
    fn pick_random_fallback_when_only_entry_is_excluded() {
        // With only one entry and it excluded, should fall back to that entry.
        let index = make_index(&["a"]);
        let picked = pick_random(&index, Some("a"));
        assert_eq!(picked.map(|p| p.id.as_str()), Some("a"));
    }

    // -- pick_at_index tests (native, no WASM required) -----------------------

    #[test]
    fn pick_at_index_empty_returns_none() {
        let candidates: Vec<&PoemIndexEntry> = vec![];
        assert!(pick_at_index(&candidates, 0).is_none());
    }

    #[test]
    fn pick_at_index_selects_correct_entry() {
        let index = make_index(&["a", "b", "c"]);
        let candidates: Vec<&PoemIndexEntry> = index.poems.iter().collect();
        assert_eq!(pick_at_index(&candidates, 0).map(|e| e.id.as_str()), Some("a"));
        assert_eq!(pick_at_index(&candidates, 1).map(|e| e.id.as_str()), Some("b"));
        assert_eq!(pick_at_index(&candidates, 2).map(|e| e.id.as_str()), Some("c"));
    }

    #[test]
    fn pick_at_index_clamps_out_of_bounds() {
        // idx beyond the last valid position should clamp to the last entry.
        let index = make_index(&["a", "b"]);
        let candidates: Vec<&PoemIndexEntry> = index.poems.iter().collect();
        assert_eq!(pick_at_index(&candidates, 99).map(|e| e.id.as_str()), Some("b"));
    }

    #[test]
    fn pick_at_index_exclude_filters_correctly() {
        // Simulates what pick_random does: filter candidates, then select by index.
        // With "a" excluded from [a, b, c], candidates = [b, c].
        // Index 0 must return "b", index 1 must return "c".
        // This verifies the exclude logic produces the correct candidate set and
        // that pick_at_index selects from it deterministically — the core of the
        // selection algorithm exercised without needing js_sys::Math::random().
        let index = make_index(&["a", "b", "c"]);
        let candidates: Vec<&PoemIndexEntry> = index.poems.iter().filter(|e| e.id != "a").collect();
        assert_eq!(candidates.len(), 2);
        assert_eq!(pick_at_index(&candidates, 0).map(|e| e.id.as_str()), Some("b"));
        assert_eq!(pick_at_index(&candidates, 1).map(|e| e.id.as_str()), Some("c"));
    }

    #[test]
    fn pick_at_index_single_candidate_always_returns_it() {
        // When exclude reduces candidates to one entry, any idx returns that entry.
        // This is the typical "New Poem" case on a two-poem index.
        let index = make_index(&["a", "b"]);
        let candidates: Vec<&PoemIndexEntry> = index.poems.iter().filter(|e| e.id != "a").collect();
        assert_eq!(candidates.len(), 1);
        assert_eq!(pick_at_index(&candidates, 0).map(|e| e.id.as_str()), Some("b"));
    }

    #[test]
    fn deserialize_poem_index() {
        let json = r#"{"version":1,"poems":[{"id":"test-id","path":"/poems/test.json","title":"Test","author":"Auth"}]}"#;
        let index: PoemIndex = serde_json::from_str(json).unwrap();
        assert_eq!(index.version, 1);
        assert_eq!(index.poems.len(), 1);
        assert_eq!(index.poems[0].id, "test-id");
    }

    #[test]
    fn deserialize_poem_detail_required_fields() {
        let json = r#"{"id":"x","title":"X","author":"Y","content":"line one\nline two"}"#;
        let detail: PoemDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.id, "x");
        assert_eq!(detail.content, "line one\nline two");
        assert!(detail.date.is_none());
        assert!(detail.tags.is_none());
    }

    #[test]
    fn deserialize_poem_detail_optional_fields() {
        let json = r#"{"id":"x","title":"T","author":"A","content":"c","date":"1861","source":"PD","tags":["nature"]}"#;
        let detail: PoemDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.date.as_deref(), Some("1861"));
        assert_eq!(detail.source.as_deref(), Some("PD"));
        assert_eq!(detail.tags.as_deref(), Some(&["nature".to_string()][..]));
    }
}
