#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(all(rocksdb_backend, sled_backend, redb_backend))]
compile_error!(
    "the \"rocksdb\", \"redb\" and \"sled\" features may not be enabled at the same time"
);

#[cfg(not(any(rocksdb_backend, sled_backend, redb_backend, wasm)))]
compile_error!("either the \"rocksdb\", \"redb\" or \"sled\" feature must be enabled on native");

use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "bevy")]
mod persistent_resource;

#[cfg(feature = "bevy")]
pub use persistent_resource::{PersistentResourceAppExtensions, PersistentResourcePlugin};

pub mod prelude;

trait StoreImpl {
    type GetError;
    type SetError;
    type RemoveError;

    fn set_string(&mut self, key: &str, value: &str) -> Result<(), Self::SetError> {
        self.set(key, &value.to_string())
    }
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Self::GetError>;
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::SetError>;
    fn remove(&mut self, key: &str) -> Result<(), Self::RemoveError>;
    fn remove_and_get<T: DeserializeOwned>(
        &mut self,
        key: &str,
    ) -> Result<Option<T>, Self::RemoveError>;
    fn clear(&mut self) -> Result<(), Self::SetError>;
}

#[cfg(wasm)]
mod local_storage_store;

#[cfg(wasm)]
use local_storage_store::{self as backend};

#[cfg(sled_backend)]
mod sled_store;

#[cfg(sled_backend)]
use sled_store::{self as backend};

#[cfg(rocksdb_backend)]
mod rocksdb_store;

#[cfg(rocksdb_backend)]
use rocksdb_store::{self as backend};

// todo: Look into unifying these types?
pub use backend::{GetError, RemoveError, SetError};

enum Location<'a> {
    PlatformDefault(&'a PlatformDefault),
    #[cfg(any(sled_backend, rocksdb_backend, redb_backend))]
    CustomPath(&'a std::path::Path),
}

#[cfg(redb_backend)]
mod redb_store;

#[cfg(redb_backend)]
use redb_store::{self as backend};

#[cfg(any(sled_backend, rocksdb_backend, redb_backend))]
mod path;

/// Main resource for setting/getting values
#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::prelude::Resource))]
pub struct PkvStore {
    inner: backend::InnerStore,
}

#[allow(clippy::result_large_err)]
impl PkvStore {
    /// Creates or opens a persistent key value store
    ///
    /// The given `organization` and `application` are used to create a backing file
    /// in a corresponding location on the users device. Usually within the home or user folder
    pub fn new(organization: &str, application: &str) -> Self {
        let config = PlatformDefault {
            qualifier: None,
            organization: organization.to_string(),
            application: application.to_string(),
        };
        Self::new_in_location(&config)
    }

    /// Creates or opens a persistent key value store
    ///
    /// Like [`PkvStore::new`], but also provide a qualifier.
    /// Some operating systems use the qualifier as part of the path to the store.
    /// The qualifier is usually "com", "org" etc.
    pub fn new_with_qualifier(qualifier: &str, organization: &str, application: &str) -> Self {
        let config = PlatformDefault {
            qualifier: Some(qualifier.to_string()),
            organization: organization.to_string(),
            application: application.to_string(),
        };
        Self::new_in_location(&config)
    }

    /// Creates or opens a persistent key value store
    ///
    /// Like [`PkvStore::new`], but requires a direct path.
    /// The `path` is used to create a backing file
    /// in a corresponding location on the users device.
    #[cfg(any(sled_backend, rocksdb_backend, redb_backend))]
    pub fn new_in_dir<P: AsRef<std::path::Path>>(path: P) -> Self {
        let inner = backend::InnerStore::new(Location::CustomPath(path.as_ref()));
        Self { inner }
    }

    fn new_in_location(config: &PlatformDefault) -> Self {
        let inner = backend::InnerStore::new(Location::PlatformDefault(config));
        Self { inner }
    }

    /// Serialize and store the value
    pub fn set<T: Serialize>(&mut self, key: impl AsRef<str>, value: &T) -> Result<(), SetError> {
        self.inner.set(key.as_ref(), value)
    }

    /// More or less the same as set::<String>, but can take a &str
    pub fn set_string(&mut self, key: impl AsRef<str>, value: &str) -> Result<(), SetError> {
        self.inner.set_string(key.as_ref(), value)
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    pub fn get<T: DeserializeOwned>(&self, key: impl AsRef<str>) -> Result<T, GetError> {
        self.inner.get(key.as_ref())
    }
    /// Remove the value from the store for the given key
    /// returns the removed value if one existed
    pub fn remove_and_get<T: DeserializeOwned>(
        &mut self,
        key: impl AsRef<str>,
    ) -> Result<Option<T>, RemoveError> {
        self.inner.remove_and_get(key.as_ref())
    }

    /// Remove the value from the store for the given key
    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), RemoveError> {
        self.inner.remove(key.as_ref())
    }

    /// Clear all key values data
    /// returns Err(SetError) if clear error
    pub fn clear(&mut self) -> Result<(), SetError> {
        self.inner.clear()
    }
}

struct PlatformDefault {
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

    #[cfg(any(sled_backend, rocksdb_backend, redb_backend))]
    #[test]
    fn new_in_dir() {
        setup();

        let dirs = directories::ProjectDirs::from("", "BevyPkv", "test_new_in_dir");
        let parent_dir = match dirs.as_ref() {
            Some(dirs) => dirs.data_dir(),
            None => std::path::Path::new("."), // todo: maybe warn?
        };

        let mut store = PkvStore::new_in_dir(parent_dir);

        store
            .set_string("hello_custom_path", "goodbye_custom_path")
            .unwrap();
        let ret = store.get::<String>("hello_custom_path");
        assert_eq!(ret.unwrap(), "goodbye_custom_path");
    }

    #[cfg(any(sled_backend, rocksdb_backend, redb_backend))]
    #[test]
    fn empty_db_not_found() {
        use crate::GetError;

        setup();

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let store = PkvStore::new_in_dir(dir.path());

        let err = store.get::<String>("not_there").unwrap_err();

        // todo: Use assert_matches! when stable
        assert!(matches!(err, GetError::NotFound));
    }

    #[test]
    fn clear() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_clear");

        // More than 1 key-value pair was added to the test because the
        // RocksDB adapter uses an iterator in order to implement .clear()
        store.set_string("key1", "goodbye").unwrap();
        store.set_string("key2", "see yeah!").unwrap();

        let ret = store.get::<String>("key1").unwrap();
        let ret2 = store.get::<String>("key2").unwrap();

        assert_eq!(ret, "goodbye");
        assert_eq!(ret2, "see yeah!");

        store.clear().unwrap();
        let ret = store.get::<String>("key1").ok();
        let ret2 = store.get::<String>("key2").ok();
        assert_eq!((ret, ret2), (None, None))
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

    #[test]
    fn remove() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_remove");
        let user = User {
            name: "alice".to_string(),
            age: 32,
        };
        store.set("user", &user).unwrap();
        store.remove("user").unwrap();
        assert_eq!(store.get::<User>("user").ok(), None);
    }

    #[test]
    fn remove_and_get() {
        setup();
        let mut store = PkvStore::new("BevyPkv", "test_remove_and_get");
        let user = User {
            name: "alice".to_string(),
            age: 32,
        };
        store.set("user", &user).unwrap();
        let removed_user = store.remove_and_get::<User>("user").unwrap().unwrap();
        assert_eq!(user, removed_user);
        assert_eq!(store.get::<User>("user").ok(), None);
    }
}
