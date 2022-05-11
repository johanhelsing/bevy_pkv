use serde::{de::DeserializeOwned, Serialize};

trait StoreImpl {
    type GetError;
    type SetError;

    fn set_string(&mut self, key: &str, value: &str) -> Result<(), Self::SetError> {
        self.set(key, &value.to_string())
    }
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Self::GetError>;
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::SetError>;
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
}

struct StoreConfig {
    qualifier: Option<String>,
    organization: String,
    application: String,
}

#[cfg(test)]
mod tests {
    use crate::PkvStore;

    // note: These tests use the real deal. Might be a good idea to clean the BevyPkv folder in .local/share
    // to get fresh tests.

    fn setup() {
        // When building for WASM, print panics to the browser console
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();
    }

    #[test]
    fn set_string_works() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "Tests");
        store.set_string("hello", "goodbye").unwrap();
        let ret = store.get::<String>("hello");
        assert_eq!(ret.unwrap(), "goodbye");
    }
}
