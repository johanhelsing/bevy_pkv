use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Default)]
pub struct PkvPlugin;

impl Plugin for PkvPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PkvStore>();
    }
}

#[derive(Debug)]
pub struct PkvStore {
    db: sled::Db,
}

impl Default for PkvStore {
    fn default() -> Self {
        let db = sled::open("bevy_pkv.sled").expect("Failed to init key value store");
        Self { db }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SetError {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GetError {
    // todo: convert sled errors to our own types of errors
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),
    #[error("No value found for the given key")]
    NotFound,
}

impl PkvStore {
    /// Serialize and store the value
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), SetError> {
        let bytes = bincode::serialize(value)?;
        self.db.insert(key, bytes)?;
        Ok(())
    }

    /// More or less the same as set::<String>, but can take a &str
    pub fn set_string(&mut self, key: &str, value: &str) -> Result<(), SetError> {
        let bytes = bincode::serialize(value)?;
        self.db.insert(key, bytes)?;
        Ok(())
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, GetError> {
        let bytes = self.db.get(key)?.ok_or(GetError::NotFound)?;
        let value = bincode::deserialize(&bytes)?;
        Ok(value)
    }
}
