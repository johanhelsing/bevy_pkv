use crate::{Location, StoreImpl};
use serde::{de::DeserializeOwned, Serialize};
use pickledb::{PickleDb,PickleDbDumpPolicy,SerializationMethod};
use std::fmt::{Debug,Formatter};
pub struct PickleDBStore {
    db: PickleDb,
}
impl Debug for PickleDBStore{
    fn fmt(&self,f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>{
        writeln!(f,"Pickle DB")?;
        Ok(())
    }
}
pub use PickleDBStore as InnerStore;

/// Errors that can occur during `PkvStore::get`
#[derive(thiserror::Error, Debug)]
pub enum GetError {
    /// An internal error from the rocksdb crate
    #[error("PickleDB error")]
    PickleDb(#[from] pickledb::error::Error),
    /// The value for the given key was not found
    #[error("No value found for the given key")]
    NotFound,
}

/// Errors that can occur during `PkvStore::set`
#[derive(thiserror::Error, Debug)]
pub enum SetError {
    /// An internal error from the rocksdb crate
    #[error("PickleDB error")]
    PickleDb(#[from] pickledb::error::Error),
}

impl PickleDBStore {
    pub(crate) fn new(location: Location) -> Self {
        let db_path = location.get_path().join("bevy_pickledb_pkv");
        let db=PickleDb::new(
            db_path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Bin,
        );
        Self { db }
    }
}

impl StoreImpl for PickleDBStore{
    type GetError = GetError;
    type SetError = SetError;

    /// Serialize and store the value
    fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::SetError> {
        self.db.set::<T>(key,value)?;
        Ok(())
    }

    /// More or less the same as set::<String>, but can take a &str
    fn set_string(&mut self, key: &str, value: &str) -> Result<(), Self::SetError> {
        self.db.set::<String>(key, &value.to_string())?;
        Ok(())
    }

    /// Get the value for the given key
    /// returns Err(GetError::NotFound) if the key does not exist in the key value store.
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Self::GetError> {
        let value= self.db.get::<T>(key).ok_or(Self::GetError::NotFound);
        value
    }

    /// Clear all keys and their values
    /// The PickleDB adapter uses an iterator to achieve this, unlike sled
    fn clear(&mut self) -> Result<(), Self::SetError> {
        for key in self.db.get_all() {
            self.db.rem(&key)?;
        }

        Ok(())
    }
}
