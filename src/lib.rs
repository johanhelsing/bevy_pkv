use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "bevy")] use bevy_ecs::prelude::Resource;


trait StoreImpl {
    type GetError;
    type SetError;

    fn set_string(&mut self, key: &str, value: &str) -> Result<(), Self::SetError> {
        self.set(key, &value.to_string())
    }
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Self::GetError>;
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::SetError>;
    fn clear(&mut self) -> Result<(), Self::SetError>;
}

#[cfg(target_arch = "wasm32")]
mod local_storage_store;

#[cfg(target_arch = "wasm32")]
use local_storage_store::{self as backend};

#[cfg(not(target_arch = "wasm32"))]
mod sled_store;

#[cfg(not(target_arch = "wasm32"))]
use sled_store::{self as backend};

// todo: Look into unifying these types?
pub use backend::{GetError, SetError};

/// Main resource for setting/getting values
///
/// Automatically inserted when adding `PkvPlugin`
#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct PkvStore {
    inner: backend::InnerStore,
}

impl PkvStore {
    pub fn new(organization: &str, application: &str) -> Self {
        let config = StoreConfig {
            qualifier: None,
            organization: organization.to_string(),
            application: application.to_string(),
        };
        Self::new_from_config(&config)
    }

    pub fn new_with_qualifier(qualifier: &str, organization: &str, application: &str) -> Self {
        let config = StoreConfig {
            qualifier: Some(qualifier.to_string()),
            organization: organization.to_string(),
            application: application.to_string(),
        };
        Self::new_from_config(&config)
    }

    fn new_from_config(config: &StoreConfig) -> Self {
        let inner = backend::InnerStore::new(config);
        Self { inner }
    }

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

    /// Clear all key values data
    /// returns Err(SetError) if clear error
    pub fn clear(&mut self) -> Result<(), SetError> {
        self.inner.clear()
    }
}

struct StoreConfig {
    qualifier: Option<String>,
    organization: String,
    application: String,
}

#[cfg(test)]
mod tests {
    use crate::PkvStore;
    use serde::{Deserialize, Serialize};

    // note: These tests use the real deal. Might be a good idea to clean the BevyPkv folder in .local/share
    // to get fresh tests.

    fn setup() {
        // When building for WASM, print panics to the browser console
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
    }

    #[test]
    fn set_string() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_set_string");
        store.set_string("hello", "goodbye").unwrap();
        let ret = store.get::<String>("hello");
        assert_eq!(ret.unwrap(), "goodbye");
    }

    #[test]
    fn clear() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_clear");
        store.set_string("key1", "goodbye").unwrap();
        let ret = store.get::<String>("key1").unwrap();
        assert_eq!(ret, "goodbye");
        store.clear().unwrap();
        let ret = store.get::<String>("key1").ok();
        assert_eq!(ret, None)
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct User {
        name: String,
        age: u8,
    }

    #[test]
    fn set() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_set");
        let user = User {
            name: "alice".to_string(),
            age: 32,
        };
        store.set("user", &user).unwrap();
        assert_eq!(store.get::<User>("user").unwrap(), user);
    }
}
