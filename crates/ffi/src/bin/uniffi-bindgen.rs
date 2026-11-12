//! Generates the Swift bindings with the same `uniffi` version the library is built with
//! (`scripts/build-ffi.sh`): `cargo run -p planets-ffi --bin uniffi-bindgen -- generate --library <lib> ...`.

fn main() {
    uniffi::uniffi_bindgen_main();
}
