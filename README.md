# bevy_pkv

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)

Bevy pkv is a persistent key value store for bevy.

The end goal is to write something cross-platform (including web) that allows
storing things like settings, save games etc. It should just be a thin wrapper
around other crates.

It's currently using sled + bincode for storage, I'm not sure if that's the best
choice, but it will do for now.

It currently creates a single global key value store when the plugin is
initialized.

## TODO

- Wasm implementation based on localstorage api
- Different scopes?

## Usage

Add the plugin to your app

```rust
App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(PkvPlugin);
```

Use it in a system:

```rust
fn setup(mut pkv: ResMut<PkvStore>) {
    let has_run_before = pkv.get("has_run_before").is_ok();
    if has_run_before {
        if let Ok(username) = pkv.get("username") {
            info!("welcome back {username}");
        }
    } else {
        // <show tutorial>
        pkv.set("has_run_before", "true").expect("failed to set has_run_before");
    }
}
```


## Bevy version support

The `main` branch targets the latest bevy release.

I intend to support the `main` branch of Bevy in the `bevy-main` branch.

|bevy|bevy_pkv|
|---|---|
|0.7|0.2,0.3,main|
|0.6|0.1|

## License

MIT or Apache-2.0