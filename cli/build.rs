use std::path::PathBuf;

use cargo_toml::Manifest;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let manifest = Manifest::from_path(PathBuf::from_iter([ROOT, "Cargo.toml"])).unwrap();
    let version = manifest
        .dependencies
        .get("cambridge-asm")
        .unwrap()
        .detail()
        .unwrap()
        .version
        .as_ref()
        .unwrap();

    std::fs::write(
        PathBuf::from_iter([std::env::var("OUT_DIR").unwrap(), "LIBRARY_VERSION".into()]),
        version,
    )
    .unwrap();
}
