use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::MediaFile;

/// Shared in-memory cache of all media files.
/// Written atomically on each scan completion.
#[derive(Clone, Default)]
pub struct MediaCache(Arc<RwLock<Arc<Vec<MediaFile>>>>);

impl MediaCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Arc::new(vec![]))))
    }

    /// Snapshot the current cache — cheap, Arc clone.
    pub async fn get(&self) -> Arc<Vec<MediaFile>> {
        self.0.read().await.clone()
    }

    /// Atomically replace the cache contents.
    pub async fn set(&self, files: Vec<MediaFile>) {
        *self.0.write().await = Arc::new(files);
    }

    #[allow(dead_code)]
    pub async fn is_empty(&self) -> bool {
        self.0.read().await.is_empty()
    }
}
