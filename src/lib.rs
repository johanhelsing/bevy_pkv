use bevy::prelude::*;

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
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), SetError> {
        let value = bincode::serialize(value)?;
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String, GetError> {
        let value = self.db.get(key)?.ok_or(GetError::NotFound)?;
        let value = bincode::deserialize(&value)?;
        Ok(value)
    }
}
