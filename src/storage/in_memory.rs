// src/storage/in_memory.rs

use crate::storage::{StorageBackend, StorageError};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct InMemoryStorageBackend {
    store: Mutex<HashMap<Uuid, Vec<u8>>>,
}

impl Default for InMemoryStorageBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorageBackend {
    pub fn new() -> Self {
        let store = Mutex::new(HashMap::new());

        Self { store }
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorageBackend {
    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let mut locked_store = self
            .store
            .lock()
            .map_err(|e| StorageError::Operation(anyhow::anyhow!("Mutex poisoned: {e}")))?;
        let _ = locked_store.remove(&id);

        Ok(())
    }

    async fn find(&self, id: Uuid) -> Result<Bytes, StorageError> {
        let locked_store = self
            .store
            .lock()
            .map_err(|e| StorageError::Operation(anyhow::anyhow!("Mutex poisoned: {e}")))?;
        let result = locked_store.get(&id);

        match result.ok_or_else(|| StorageError::NotFound("No value with that key.".to_string())) {
            Ok(b) => Ok(Bytes::copy_from_slice(b.as_slice())),
            Err(e) => Err(e),
        }
    }

    async fn save(&self, id: Uuid, bytes: Bytes) -> Result<(), StorageError> {
        let mut locked_store = self
            .store
            .lock()
            .map_err(|e| StorageError::Operation(anyhow::anyhow!("Mutex poisoned: {e}")))?;
        let _ = locked_store.insert(id, bytes.to_vec());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- save / find ---

    #[tokio::test]
    async fn save_then_find_round_trips_bytes() {
        let store = InMemoryStorageBackend::new();
        let id = Uuid::new_v4();
        let bytes = Bytes::from_static(&[1, 2, 3, 4]);

        store.save(id, bytes.clone()).await.unwrap();
        let found = store.find(id).await.unwrap();

        assert_eq!(found, bytes);
    }

    #[tokio::test]
    async fn find_on_missing_id_returns_not_found() {
        let store = InMemoryStorageBackend::new();
        let result = store.find(Uuid::new_v4()).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    // --- delete ---

    #[tokio::test]
    async fn delete_removes_stored_bytes() {
        let store = InMemoryStorageBackend::new();
        let id = Uuid::new_v4();
        store.save(id, Bytes::from_static(&[1])).await.unwrap();

        store.delete(id).await.unwrap();

        // after delete, the bytes are gone
        let result = store.find(id).await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn delete_on_missing_id_is_idempotent() {
        // deleting an id that was never saved succeeds — matches the real
        // (OpenDAL) backend, which is also idempotent.
        let store = InMemoryStorageBackend::new();
        let result = store.delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    // NOTE on `save`: there is no meaningful unhappy path for the in-memory
    // backend. `HashMap::insert` cannot fail (barring allocation), and the
    // only `Err` case — a poisoned mutex — requires a panic mid-mutation,
    // which can't be triggered without deliberately corrupting state. The
    // OpenDAL backend exercises real failure modes (network, auth); this
    // backend's value is determinism, not failure coverage.
}
