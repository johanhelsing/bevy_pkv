//! this example just shows how you can use serde aliases if you rename fields

use bevy::prelude::*;
use bevy_pkv::{PkvPlugin, PkvStore};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct UserV1 {
    nick: String,
    favorite_color: Color,
}

#[derive(Serialize, Deserialize)]
struct UserV2 {
    #[serde(alias = "nick")]
    name: String,
    // notice we no longer care about favorite colors
}

fn setup(mut pkv: ResMut<PkvStore>) {
    let user_v1 = UserV1 {
        nick: "old bob".to_string(),
        favorite_color: Color::BEIGE,
    };
    pkv.set("user", &user_v1)
        .expect("failed to store User struct");

    let user_v2 = pkv.get::<UserV2>("user").unwrap();
    info!("Welcome back {}", user_v2.name);
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugin(PkvPlugin)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}
