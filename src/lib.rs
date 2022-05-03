use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

/// Main plugin for the bevy_pkv crate
#[derive(Default)]
pub struct PkvPlugin;

impl Plugin for PkvPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PkvStore>();
    }
}

trait StoreImpl<GetError, SetError> {
    fn set_string(&mut self, key: &str, value: &str) -> Result<(), SetError> {
        self.set(key, &value.to_string())
    }
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, GetError>;
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), SetError>;
}

#[cfg(target_arch = "wasm32")]
mod local_storage_store;

#[cfg(target_arch = "wasm32")]
pub use local_storage_store::{GetError, SetError};

#[cfg(not(target_arch = "wasm32"))]
mod sled_store;

#[cfg(not(target_arch = "wasm32"))]
pub use sled_store::{GetError, SetError};

/// Main resource for setting/getting values
///
/// Automatically inserted when adding `PkvPlugin`
#[derive(Debug, Default)]
#[cfg(target_arch = "wasm32")]
pub struct PkvStore {
    inner: local_storage_store::LocalStorageStore,
}

#[derive(Debug, Default)]
#[cfg(not(target_arch = "wasm32"))]
pub struct PkvStore {
    inner: sled_store::SledStore,
}

impl PkvStore {
    /// Serialize and store the value
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), SetError> {
        self.inner.set(key, value)
    }

    /// More or less the same as set::<String>, but can take a &str
    pub fn set_string(&mut self, key: &str, value: &str) -> Result<(), SetError> {
        self.inner.set_string(key, value)
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, GetError> {
        self.inner.get(key)
    }
}
