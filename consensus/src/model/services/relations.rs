use crate::model::stores::relations::RelationsStoreReader;
use parking_lot::RwLock;
use std::sync::Arc;
use vecno_consensus_core::BlockHashSet;
use vecno_database::prelude::{ReadLock, StoreError, StoreResult};
use vecno_hashes::Hash;

/// Multi-threaded block-relations service imp
#[derive(Clone)]
pub struct MTRelationsService<T: RelationsStoreReader> {
    store: Arc<RwLock<Vec<T>>>,
    level: usize,
}

impl<T: RelationsStoreReader> MTRelationsService<T> {
    pub fn new(store: Arc<RwLock<Vec<T>>>, level: u8) -> Self {
        Self { store, level: level as usize }
    }
}

impl<T: RelationsStoreReader> RelationsStoreReader for MTRelationsService<T> {
    fn get_parents(&self, hash: Hash) -> Result<vecno_consensus_core::blockhash::BlockHashes, StoreError> {
        self.store.read()[self.level].get_parents(hash)
    }

    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        self.store.read()[self.level].get_children(hash)
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        self.store.read()[self.level].has(hash)
    }

    fn counts(&self) -> Result<(usize, usize), StoreError> {
        self.store.read()[self.level].counts()
    }
}
