use std::{
    num::NonZeroUsize,
    sync::{Mutex, MutexGuard, PoisonError},
};

use dashmap::DashMap;
use lru::LruCache;

use crate::utils::database::ServerConfig;

pub fn config_cache() -> &'static ConfigCache {
    crate::CONFIG_CACHE
        .get()
        .expect("CONFIG_CACHE not initialized")
}

#[derive(Debug)]
pub struct ConfigCache {
    cache: DashMap<u64, ServerConfig>,
    lru: Mutex<LruCache<u64, ()>>,
}

impl ConfigCache {
    pub fn new() -> Self {
        let cache_capacity =
            NonZeroUsize::new(1000).expect("Capacity must be > 0, please check source code.");
        Self {
            cache: DashMap::new(),
            lru: Mutex::new(LruCache::new(cache_capacity)),
        }
    }

    // Gets config from cache, else retrieves it from db and adds to cache.
    pub async fn get_or_fetch(&self, guild_id: u64) -> anyhow::Result<ServerConfig> {
        let Some(config) = self.cache.get(&guild_id) else {
            tracing::debug!("Could not find guild in cache, retrieving from database.");
            let config = ServerConfig::get_or_default(guild_id).await?;
            self.try_insert(guild_id, config.clone());

            return Ok(config);
        };
        // Marks the guild as recently used.
        if let Ok(mut lru) = self.lru.lock() {
            tracing::debug!("Updated LRU");
            lru.promote(&guild_id);
        }
        tracing::debug!("Found Server Config in cache");

        Ok(config.clone())
    }

    // Update the server config in the database and cache.
    pub async fn update_config(&self, guild_id: u64, config: ServerConfig) -> anyhow::Result<()> {
        // Updates the database with change.
        config.save().await?;
        // Updates the cache
        self.insert(guild_id, config);

        Ok(())
    }

    fn try_insert(&self, guild_id: u64, config: ServerConfig) {
        if self.cache.contains_key(&guild_id) {
            if let Ok(mut lru) = self.lru.lock() {
                lru.promote(&guild_id);
            }
            return;
        }

        self.insert(guild_id, config);
    }

    // Inserts config into cache and updates the LRU queue, and handles filled queue.
    fn insert(&self, guild_id: u64, config: ServerConfig) {
        tracing::debug!("Attempting to insert guild config");
        // Updates the LRU queue.
        match self.lru.lock() {
            Ok(mut lru) => {
                // If the LRU queue is full, the guild_id that needs to be evicted is returned.
                if let Some((evicted_id, _)) = lru.push(guild_id, ()) {
                    self.cache.remove(&evicted_id);
                    tracing::debug!("Evicted guild_id {} from config cache", evicted_id);
                }
            }
            Err(e) => Self::handle_poison(e, guild_id),
        }

        self.cache.insert(guild_id, config);
    }

    fn handle_poison(e: PoisonError<MutexGuard<'_, LruCache<u64, ()>>>, guild_id: u64) {
        tracing::error!("LRU lock poisoned, attempting recovery...");

        // Get the guard despite poisoning and clear the LRU
        let mut lru = e.into_inner();
        lru.clear();

        // Re-insert this entry as the first one
        lru.push(guild_id, ());

        tracing::warn!("LRU cache cleared and reset. Cache state restored.");
    }
}
