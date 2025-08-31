//! Convenient re-exports for common bevy_pkv functionality

pub use crate::PkvStore;

#[cfg(feature = "bevy")]
pub use crate::{PersistentResourceAppExtensions, PersistentResourcePlugin};
