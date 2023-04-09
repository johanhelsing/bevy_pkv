use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        rocksdb: { all(feature = "rocksdb", not(wasm)) },
        sled: { all(feature = "sled", not(wasm)) }
    }
}
