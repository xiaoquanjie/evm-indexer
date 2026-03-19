use crate::models;
use crate::models::SyncState;
use std::sync::RwLock;

struct Inner {
    sync_state: Option<models::SyncState>,
}

pub struct Cache {
    inner: RwLock<Inner>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            inner: RwLock::new(Inner { sync_state: None }),
        }
    }

    pub fn update_sync_state(&self, block_number: u64, block_hash: &str) {
        let mut inner = self.inner.write().unwrap();
        if let Some(ref mut ss) = inner.sync_state {
            ss.last_block = block_number as i64;
            ss.last_block_hash = block_hash.to_string();
        }
    }

    pub fn set_sync_state(&self, sync_state: SyncState) {
        let mut inner = self.inner.write().unwrap();
        inner.sync_state = Some(sync_state);
    }

    pub fn get_sync_state(&self) -> Option<SyncState> {
        let inner = self.inner.read().unwrap();
        inner.sync_state.clone()
    }
}
