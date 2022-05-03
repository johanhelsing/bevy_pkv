# bevy_pkv

[![crates.io](https://img.shields.io/crates/v/bevy_pkv.svg)](https://crates.io/crates/bevy_pkv)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![docs.rs](https://img.shields.io/docsrs/bevy_pkv)](https://docs.rs/bevy_pkv)
[![ci](https://github.com/johanhelsing/bevy_pkv/actions/workflows/ci.yml/badge.svg)](https://github.com/johanhelsing/bevy_pkv/actions/workflows/ci.yml)

Bevy pkv is a persistent key value store for bevy.

The end goal is to write something cross-platform (including web) that allows
storing things like settings, save games etc. It should just be a thin wrapper
around other crates.

It's currently using sled + bincode for storage, I'm not sure if that's the best
choice, but it will do for now.

It currently creates a single global key value store when the plugin is
initialized.

## Usage

Add the plugin to your app

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(PkvPlugin)
    .run();
```

Use it in a system:

```rust
fn setup(mut pkv: ResMut<PkvStore>) {
    if let Ok(username) = pkv.get::<String>("username") {
        info!("Welcome back {username}");
    } else {
        pkv.set_string("username", "alice")
            .expect("failed to store username");

        // alternatively, using the slightly less efficient generic api:
        pkv.set("username", &"alice".to_string())
            .expect("failed to store username");
    }
}
```

Using your own types implementing `serde::Serialize` and `Deserialize`:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

fn setup(mut pkv: ResMut<PkvStore>) {
    if let Ok(user) = pkv.get::<User>("user") {
        info!("Welcome back {}", user.name);
    } else {
        let user = User {
            name: "bob".to_string(),
        };
        pkv.set("user", &user).expect("failed to store user");
    }
}
```

See the [examples](./examples) for further usage

## Bevy version support

The `main` branch targets the latest bevy release.

I intend to support the `main` branch of Bevy in the `bevy-main` branch.

|bevy|bevy_pkv|
|---|---|
|0.7|0.2,0.3,0.4,main|
|0.6|0.1|

## License

MIT or Apache-2.0