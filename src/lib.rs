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

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct PkvStore {
    db: sled::Db,
}

/// Main resource for setting/getting values
///
/// Automatically inserted when adding `PkvPlugin`
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct PkvStore {}

#[allow(clippy::derivable_impls)]
impl Default for PkvStore {
    // todo: maybe consider not exposing this, so people are not tempted
    // to try to manually initialize stores?
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let db = sled::open("bevy_pkv.sled").expect("Failed to init key value store");
            Self { db }
        }
        #[cfg(target_arch = "wasm32")]
        Self {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(thiserror::Error, Debug)]
pub enum SetError {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),
}

#[cfg(target_arch = "wasm32")]
#[derive(thiserror::Error, Debug)]
pub enum SetError {
    #[error("JavaScript error from setItem")]
    SetItem(wasm_bindgen::JsValue),
    #[error("Error serializing as json")]
    Json(#[from] serde_json::Error),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(thiserror::Error, Debug)]
pub enum GetError {
    #[error("Sled error")]
    Sled(#[from] sled::Error),
    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),
    #[error("No value found for the given key")]
    NotFound,
}

#[cfg(target_arch = "wasm32")]
#[derive(thiserror::Error, Debug)]
pub enum GetError {
    #[error("No value found for the given key")]
    NotFound,
    #[error("JavaScript error from getItem")]
    GetItem(wasm_bindgen::JsValue),
}

#[cfg(target_arch = "wasm32")]
fn get_local_storage() -> web_sys::Storage {
    #[cfg(target_arch = "wasm32")]
    web_sys::window()
        .expect("No window")
        .local_storage()
        .expect("Failed to get local storage")
        .expect("No local storage")
}

impl PkvStore {
    /// Serialize and store the value
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), SetError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = bincode::serialize(value)?;
            self.db.insert(key, bytes)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let json = serde_json::to_string(value)?;
            let db = get_local_storage();
            db.set_item(key, &json).map_err(SetError::SetItem)?;
            Ok(())
        }
    }

    /// More or less the same as set::<String>, but can take a &str
    pub fn set_string(&mut self, key: &str, value: &str) -> Result<(), SetError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = bincode::serialize(value)?;
            self.db.insert(key, bytes)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let db = get_local_storage();
            db.set_item(key, value).map_err(SetError::SetItem)?;
            Ok(())
        }
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, GetError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = self.db.get(key)?.ok_or(GetError::NotFound)?;
            let value = bincode::deserialize(&bytes)?;
            Ok(value)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let db = get_local_storage();
            let entry = db.get_item(key).map_err(GetError::GetItem)?;
            let json = entry.as_ref().ok_or(GetError::NotFound)?;
            let value: T = serde_json::from_str(json).unwrap();
            Ok(value)
        }
    }
}
