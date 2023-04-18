#[cfg(wasm)]
pub mod local_storage_store;

/// A store implemented using [`sled`]
#[cfg(sled)]
pub mod sled_store;

/// A store implemented using [`rocksdb`]
#[cfg(rocksdb)]
pub mod rocksdb_store;

/// A store implemented using the LocalStorage through `wasm-bindgen`
#[cfg(wasm)]
pub use local_storage_store::{self as default_store, LocalStorageStore as DefaultStore};

#[cfg(sled)]
pub use sled_store::{self as default_store, SledStore as DefaultStore};

#[cfg(all(rocksdb, not(sled)))]
pub use rocksdb_store::{self as default_store, RocksDbStore as DefaultStore};
