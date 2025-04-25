use bevy::{log::LogPlugin, prelude::*};
use bevy_pkv::PkvStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

fn setup(mut pkv: ResMut<PkvStore>) {
    // strings
    if let Ok(username) = pkv.get::<String>("username") {
        info!("Welcome back {username}");
    } else {
        info!("First time user, setting username to 'alice'");
        pkv.set_string("username", "alice")
            .expect("failed to store username");

        // alternatively, using the slightly less efficient generic api:
        pkv.set("username", &"alice".to_string())
            .expect("failed to store username");
    }

    // serde types
    if let Ok(user) = pkv.get::<User>("user") {
        info!("Welcome back {}", user.name);
    } else {
        info!("First time user, setting user to 'bob'");
        let user = User {
            name: "bob".to_string(),
        };
        pkv.set("user", &user).expect("failed to store User struct");
    }
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(PkvStore::new("BevyPkv", "BasicExample"))
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_systems(Startup, setup)
        .run();
}
