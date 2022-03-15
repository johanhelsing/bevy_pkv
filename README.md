# bevy_pkv

Bevy pkv is a persistent key value store for bevy.

The end goal is to write something cross-platform (including web) that allows
storing things like settings, save games etc. It should just be a thin wrapper
around other crates.

It's currently using sled + bincode for storage, I'm not sure if that's the best
choice, but it will do for now.

## TODO

- Wasm implementation based on localstorage api
- Different scopes?