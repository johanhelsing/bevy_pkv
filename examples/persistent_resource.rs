use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use bevy_pkv::{PersistentResourceAppExtensions, PkvStore};
use serde::{Deserialize, Serialize};

// Example 1: Resource with Default implementation
// This will be automatically persisted and restored
#[derive(Resource, Serialize, Deserialize, Default, Debug)]
struct GameSettings {
    volume: f32,
    difficulty: u8,
}

// Example 2: Resource using a factory function with arguments
// Useful when you need custom initialization logic or external data
#[derive(Resource, Serialize, Deserialize, Debug)]
struct PlayerProfile {
    name: String,
    play_count: u32,
    created_at: u64,
}

impl PlayerProfile {
    fn new(name: String) -> Self {
        Self {
            name,
            play_count: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

fn setup(settings: Res<GameSettings>, profile: Res<PlayerProfile>) {
    info!("=== Persistent Resource Example ===");
    info!("Game Settings: {:?}", *settings);
    info!("Player Profile: {:?}", *profile);
    info!("Resources update every 3 seconds. Stop and restart to see persistence!");
}

fn update_settings(mut settings: ResMut<GameSettings>) {
    // Simulate user changing settings
    settings.volume = (settings.volume + 0.1).min(1.0);
    settings.difficulty = (settings.difficulty + 1).min(3);
    info!(
        "Updated settings: volume={:.1}, difficulty={}",
        settings.volume, settings.difficulty
    );
}

fn update_profile(mut profile: ResMut<PlayerProfile>) {
    // Simulate a play session - increment play count
    profile.play_count += 1;
    info!("Updated profile: play count is now {}", profile.play_count);
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(PkvStore::new("BevyPkv", "PersistentResourceExample"))
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Method 1: Use Default implementation - automatically persisted
        .init_persistent_resource::<GameSettings>()
        // Method 2: Use factory function with arguments - useful for custom initialization
        .init_persistent_resource_with(|| {
            let user_name = std::env::var("USERNAME") // Windows
                .or_else(|_| std::env::var("USER")) // Unix/Linux/macOS
                .unwrap_or_else(|_| "Player".to_string()); // Fallback
            PlayerProfile::new(user_name)
        })
        .add_systems(PostStartup, setup)
        .add_systems(
            Update,
            // Update resources every 3 seconds to show persistence
            (update_settings, update_profile).run_if(on_timer(Duration::from_secs(3))),
        )
        .run();
}
