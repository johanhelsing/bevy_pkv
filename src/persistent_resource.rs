//! Plugin for automatically persisting resources when they change

use serde::{de::DeserializeOwned, Serialize};

use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;

use crate::PkvStore;

/// A plugin that automatically persists a resource when it changes using a [`PkvStore`]
///
/// This plugin will:
/// - Load the resource from persistent storage on startup (if it exists)
/// - Save the resource to persistent storage whenever it changes (runs in [`PostUpdate`])
/// - Use the type name as the storage key automatically
///
/// The save system runs in [`PostUpdate`] to ensure it captures all changes made during
/// the frame by `PreUpdate`, `Update`, and other systems.
///
/// # Convenience Methods
///
/// For easier usage, consider using the [`PersistentResourceAppExtensions`] trait methods instead:
/// - [`init_persistent_resource()`](PersistentResourceAppExtensions::init_persistent_resource) for types with `Default`
/// - [`init_persistent_resource_with()`](PersistentResourceAppExtensions::init_persistent_resource_with) for custom factory functions
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_pkv::{PkvStore, PersistentResourcePlugin};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Resource, Serialize, Deserialize, Default)]
/// struct GameSettings {
///     volume: f32,
///     difficulty: u8,
/// }
///
/// // For types that implement Default, you can use new():
/// App::new()
///     .insert_resource(PkvStore::new("MyGame", "Settings"))
///     .add_plugins(PersistentResourcePlugin::<GameSettings>::new())
///     .run();
///
/// // Or with a custom factory function (works for any type, with or without Default):
/// App::new()
///     .insert_resource(PkvStore::new("MyGame", "Settings"))
///     .add_plugins(PersistentResourcePlugin::<GameSettings>::with_default(|| GameSettings {
///         volume: 0.8,
///         difficulty: 2,
///     }))
///     .run();
///
/// // For types without Default, you must use with_default():
/// #[derive(Resource, Serialize, Deserialize)]
/// struct CustomSettings {
///     name: String,
/// }
///
/// App::new()
///     .insert_resource(PkvStore::new("MyGame", "Settings"))
///     .add_plugins(PersistentResourcePlugin::<CustomSettings>::with_default(|| CustomSettings {
///         name: "Player".to_string(),
///     }))
///     .run();
/// ```
pub struct PersistentResourcePlugin<T> {
    _phantom: std::marker::PhantomData<T>,
    factory: Option<Box<dyn Fn() -> T + Send + Sync + 'static>>,
}

impl<T> PersistentResourcePlugin<T>
where
    T: Resource + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
{
    /// Create a new [`PersistentResourcePlugin`] that uses the type name as the storage key
    /// and `T::default()` for creating default values
    ///
    /// # Convenience Alternative
    ///
    /// Consider using [`App::init_persistent_resource()`](PersistentResourceAppExtensions::init_persistent_resource)
    /// for a more concise API.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            factory: Some(Box::new(|| T::default())),
        }
    }
}

impl<T> PersistentResourcePlugin<T>
where
    T: Resource + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Create a new [`PersistentResourcePlugin`] with a custom factory function for default values
    ///
    /// # Convenience Alternative
    ///
    /// Consider using [`App::init_persistent_resource_with()`](PersistentResourceAppExtensions::init_persistent_resource_with)
    /// for a more concise API.
    pub fn with_default<F>(factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            _phantom: std::marker::PhantomData,
            factory: Some(Box::new(factory)),
        }
    }
}

impl<T> Default for PersistentResourcePlugin<T>
where
    T: Resource + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Plugin for PersistentResourcePlugin<T>
where
    T: Resource + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        let key = std::any::type_name::<T>();
        let pkv = app.world_mut().resource_mut::<PkvStore>();
        let resource = pkv.get::<T>(key).unwrap_or_else(|_| {
            // We always have a factory function now (either from with_default or new)
            self.factory
                .as_ref()
                .expect("PersistentResourcePlugin should always have a factory function")(
            )
        });
        app.insert_resource(resource);
        app.add_systems(PostUpdate, save_resource::<T>.run_if(resource_changed::<T>));
    }
}

fn save_resource<T>(resource: Res<T>, mut pkv: ResMut<PkvStore>)
where
    T: Resource + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let key = std::any::type_name::<T>();
    if let Err(e) = pkv.set(key, &*resource) {
        eprintln!("Failed to persist resource: {:?}", e);
    }
}

/// Extension trait for [`App`] to provide convenient methods for initializing persistent resources
///
/// These methods provide a more ergonomic API compared to manually adding [`PersistentResourcePlugin`].
///
/// # See Also
///
/// - [`PersistentResourcePlugin`] - The underlying plugin these methods use
pub trait PersistentResourceAppExtensions {
    /// Initialize a persistent resource that implements [`Default`]
    ///
    /// This is a convenience method equivalent to:
    /// ```rust,ignore
    /// app.add_plugins(PersistentResourcePlugin::<T>::new())
    /// ```
    ///
    /// # Example
    /// ```rust,ignore
    /// use bevy::prelude::*;
    /// use bevy_pkv::{PkvStore, PersistentResourceAppExtensions};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Resource, Serialize, Deserialize, Default)]
    /// struct GameSettings {
    ///     volume: f32,
    ///     difficulty: u8,
    /// }
    ///
    /// App::new()
    ///     .insert_resource(PkvStore::new("MyGame", "Settings"))
    ///     .init_persistent_resource::<GameSettings>()
    ///     .run();
    /// ```
    ///
    /// # See Also
    ///
    /// - [`PersistentResourcePlugin::new()`] - The equivalent plugin method
    fn init_persistent_resource<T>(&mut self) -> &mut Self
    where
        T: Resource + Serialize + DeserializeOwned + Default + Send + Sync + 'static;

    /// Initialize a persistent resource with a custom default factory function
    ///
    /// This is a convenience method equivalent to:
    /// ```rust,ignore
    /// app.add_plugins(PersistentResourcePlugin::<T>::with_default(factory))
    /// ```
    ///
    /// # Example
    /// ```rust,ignore
    /// use bevy::prelude::*;
    /// use bevy_pkv::{PkvStore, PersistentResourceAppExtensions};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Resource, Serialize, Deserialize)]
    /// struct GameSettings {
    ///     volume: f32,
    ///     difficulty: u8,
    /// }
    ///
    /// App::new()
    ///     .insert_resource(PkvStore::new("MyGame", "Settings"))
    ///     .init_persistent_resource_with(|| GameSettings { volume: 0.8, difficulty: 2 })
    ///     .run();
    /// ```
    ///
    /// # See Also
    ///
    /// - [`PersistentResourcePlugin::with_default()`] - The equivalent plugin method
    fn init_persistent_resource_with<T, F>(&mut self, factory: F) -> &mut Self
    where
        T: Resource + Serialize + DeserializeOwned + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static;
}

impl PersistentResourceAppExtensions for App {
    fn init_persistent_resource<T>(&mut self) -> &mut Self
    where
        T: Resource + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
    {
        self.add_plugins(PersistentResourcePlugin::<T>::new())
    }

    fn init_persistent_resource_with<T, F>(&mut self, factory: F) -> &mut Self
    where
        T: Resource + Serialize + DeserializeOwned + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.add_plugins(PersistentResourcePlugin::<T>::with_default(factory))
    }
}
