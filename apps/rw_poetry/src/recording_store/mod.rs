// Remaining store functions (list, get, delete) will be consumed in T09–T11.
#![allow(dead_code)]

use idb::{
    Database, DatabaseEvent, Factory, IndexParams, KeyPath, ObjectStoreParams, TransactionMode,
};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::JsValue;

const DB_NAME: &str = "rw_poetry_db";
const DB_VERSION: u32 = 1;
const STORE_RECORDINGS: &str = "recordings";
const STORE_BLOBS: &str = "audio_blobs";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum StoreError {
    NotFound,
    StorageFull,
    Unexpected(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "Recording not found."),
            StoreError::StorageFull => {
                write!(f, "Storage full. Delete some recordings to free space.")
            }
            StoreError::Unexpected(msg) => write!(f, "Unexpected storage error: {msg}"),
        }
    }
}

impl From<idb::Error> for StoreError {
    fn from(e: idb::Error) -> Self {
        let msg = format!("{e:?}");
        if msg.contains("QuotaExceededError") || msg.contains("QuotaExceeded") {
            StoreError::StorageFull
        } else {
            StoreError::Unexpected(msg)
        }
    }
}

// ---------------------------------------------------------------------------
// RecordingMetadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordingMetadata {
    pub recording_id: String,
    pub poem_id: String,
    pub poem_title: String,
    pub poem_author: String,
    /// ISO-8601 timestamp string
    pub recorded_at: String,
    pub duration_ms: Option<u64>,
    pub mime_type: String,
    pub audio_blob_key: String,
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

async fn open_db() -> Result<Database, StoreError> {
    let factory = Factory::new().map_err(|e| StoreError::Unexpected(format!("{e:?}")))?;

    let mut open_req = factory
        .open(DB_NAME, Some(DB_VERSION))
        .map_err(|e| StoreError::Unexpected(format!("{e:?}")))?;

    open_req.on_upgrade_needed(|event| {
        let db = event.database().expect("db handle in upgrade");

        // recordings store
        let mut rec_params = ObjectStoreParams::new();
        rec_params.key_path(Some(KeyPath::new_single("recording_id")));
        let rec_store = db
            .create_object_store(STORE_RECORDINGS, rec_params)
            .expect("create recordings store");

        let mut by_poem = IndexParams::new();
        by_poem.unique(false);
        rec_store
            .create_index("by_poem_id", KeyPath::new_single("poem_id"), Some(by_poem))
            .expect("create by_poem_id index");

        let mut by_date = IndexParams::new();
        by_date.unique(false);
        rec_store
            .create_index(
                "by_recorded_at",
                KeyPath::new_single("recorded_at"),
                Some(by_date),
            )
            .expect("create by_recorded_at index");

        // audio_blobs store
        let mut blob_params = ObjectStoreParams::new();
        blob_params.key_path(Some(KeyPath::new_single("blob_key")));
        db.create_object_store(STORE_BLOBS, blob_params)
            .expect("create audio_blobs store");
    });

    open_req
        .await
        .map_err(|e| StoreError::Unexpected(format!("{e:?}")))
}

fn metadata_to_jsvalue(meta: &RecordingMetadata) -> Result<JsValue, StoreError> {
    serde_wasm_bindgen::to_value(meta)
        .map_err(|e| StoreError::Unexpected(format!("serialize metadata: {e:?}")))
}

fn jsvalue_to_metadata(val: JsValue) -> Result<RecordingMetadata, StoreError> {
    serde_wasm_bindgen::from_value(val)
        .map_err(|e| StoreError::Unexpected(format!("deserialize metadata: {e:?}")))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Save a recording: writes blob first, then metadata. Cleans up on failure.
pub async fn save_recording(
    metadata: RecordingMetadata,
    audio_data: Vec<u8>,
) -> Result<(), StoreError> {
    let db = open_db().await?;

    // Write blob
    let blob_tx = db
        .transaction(&[STORE_BLOBS], TransactionMode::ReadWrite)
        .map_err(StoreError::from)?;
    let blob_store = blob_tx
        .object_store(STORE_BLOBS)
        .map_err(StoreError::from)?;

    let uint8 = Uint8Array::from(audio_data.as_slice());
    let blob_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &blob_obj,
        &JsValue::from_str("blob_key"),
        &JsValue::from_str(&metadata.audio_blob_key),
    )
    .map_err(|e| StoreError::Unexpected(format!("set blob_key: {e:?}")))?;
    js_sys::Reflect::set(&blob_obj, &JsValue::from_str("data"), &uint8)
        .map_err(|e| StoreError::Unexpected(format!("set data: {e:?}")))?;

    blob_store
        .add(&blob_obj, None)
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;
    blob_tx.await.map_err(StoreError::from)?;

    // Write metadata
    let meta_tx = db
        .transaction(&[STORE_RECORDINGS], TransactionMode::ReadWrite)
        .map_err(StoreError::from)?;
    let meta_store = meta_tx
        .object_store(STORE_RECORDINGS)
        .map_err(StoreError::from)?;

    let meta_val = metadata_to_jsvalue(&metadata)?;
    if let Err(e) = meta_store
        .add(&meta_val, None)
        .map_err(StoreError::from)?
        .await
    {
        // Metadata write failed — clean up orphaned blob
        let _ = delete_blob(&db, &metadata.audio_blob_key).await;
        return Err(StoreError::from(e));
    }
    meta_tx.await.map_err(StoreError::from)?;

    Ok(())
}

/// List all recordings sorted newest-first by `recorded_at`.
pub async fn list_recordings() -> Result<Vec<RecordingMetadata>, StoreError> {
    let db = open_db().await?;
    let tx = db
        .transaction(&[STORE_RECORDINGS], TransactionMode::ReadOnly)
        .map_err(StoreError::from)?;
    let store = tx
        .object_store(STORE_RECORDINGS)
        .map_err(StoreError::from)?;

    let all: Vec<JsValue> = store
        .get_all(None, None)
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;

    let mut recordings: Vec<RecordingMetadata> = all
        .into_iter()
        .map(jsvalue_to_metadata)
        .collect::<Result<Vec<_>, _>>()?;

    // Sort newest-first (ISO-8601 strings sort lexicographically)
    recordings.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
    Ok(recordings)
}

/// Fetch a single recording's metadata by ID.
pub async fn get_recording(recording_id: &str) -> Result<RecordingMetadata, StoreError> {
    let db = open_db().await?;
    let tx = db
        .transaction(&[STORE_RECORDINGS], TransactionMode::ReadOnly)
        .map_err(StoreError::from)?;
    let store = tx
        .object_store(STORE_RECORDINGS)
        .map_err(StoreError::from)?;

    let val = store
        .get(JsValue::from_str(recording_id))
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;

    match val {
        None => Err(StoreError::NotFound),
        Some(v) => jsvalue_to_metadata(v),
    }
}

/// Fetch raw audio bytes by blob key.
pub async fn get_audio_blob(blob_key: &str) -> Result<Vec<u8>, StoreError> {
    let db = open_db().await?;
    let tx = db
        .transaction(&[STORE_BLOBS], TransactionMode::ReadOnly)
        .map_err(StoreError::from)?;
    let store = tx.object_store(STORE_BLOBS).map_err(StoreError::from)?;

    let val = store
        .get(JsValue::from_str(blob_key))
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;

    match val {
        None => Err(StoreError::NotFound),
        Some(obj) => {
            let data = js_sys::Reflect::get(&obj, &JsValue::from_str("data"))
                .map_err(|e| StoreError::Unexpected(format!("read blob data: {e:?}")))?;
            let uint8 = Uint8Array::new(&data);
            Ok(uint8.to_vec())
        }
    }
}

/// Delete a recording: removes blob first, then metadata.
pub async fn delete_recording(recording_id: &str, blob_key: &str) -> Result<(), StoreError> {
    let db = open_db().await?;
    delete_blob(&db, blob_key).await?;

    let tx = db
        .transaction(&[STORE_RECORDINGS], TransactionMode::ReadWrite)
        .map_err(StoreError::from)?;
    let store = tx
        .object_store(STORE_RECORDINGS)
        .map_err(StoreError::from)?;
    store
        .delete(JsValue::from_str(recording_id))
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;
    tx.await.map_err(StoreError::from)?;
    Ok(())
}

async fn delete_blob(db: &Database, blob_key: &str) -> Result<(), StoreError> {
    let tx = db
        .transaction(&[STORE_BLOBS], TransactionMode::ReadWrite)
        .map_err(StoreError::from)?;
    let store = tx.object_store(STORE_BLOBS).map_err(StoreError::from)?;
    store
        .delete(JsValue::from_str(blob_key))
        .map_err(StoreError::from)?
        .await
        .map_err(StoreError::from)?;
    tx.await.map_err(StoreError::from)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata() -> RecordingMetadata {
        RecordingMetadata {
            recording_id: "rec-001".to_string(),
            poem_id: "emily-dickinson-hope".to_string(),
            poem_title: "Hope is the thing with feathers".to_string(),
            poem_author: "Emily Dickinson".to_string(),
            recorded_at: "2026-03-01T15:31:00Z".to_string(),
            duration_ms: Some(91342),
            mime_type: "audio/webm".to_string(),
            audio_blob_key: "blob-001".to_string(),
        }
    }

    #[test]
    fn recording_metadata_serde_round_trip() {
        let meta = sample_metadata();
        let json = serde_json::to_string(&meta).unwrap();
        let decoded: RecordingMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, decoded);
    }

    #[test]
    fn recording_metadata_null_duration() {
        let mut meta = sample_metadata();
        meta.duration_ms = None;
        let json = serde_json::to_string(&meta).unwrap();
        let decoded: RecordingMetadata = serde_json::from_str(&json).unwrap();
        assert!(decoded.duration_ms.is_none());
    }

    #[test]
    fn store_error_display_not_found() {
        assert_eq!(StoreError::NotFound.to_string(), "Recording not found.");
    }

    #[test]
    fn store_error_display_storage_full() {
        assert!(StoreError::StorageFull.to_string().contains("Storage full"));
    }

    #[test]
    fn store_error_display_unexpected() {
        let e = StoreError::Unexpected("boom".to_string());
        assert!(e.to_string().contains("boom"));
    }
}
