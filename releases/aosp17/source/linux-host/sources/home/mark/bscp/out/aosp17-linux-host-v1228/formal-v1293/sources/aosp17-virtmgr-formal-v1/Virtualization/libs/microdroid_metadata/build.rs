use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let proto_root = manifest_dir.parent().expect("crate dir should have parent").join("libmicrodroid_payload_metadata");
    let proto = proto_root.join("metadata.proto");
    let generated = out_dir.join("metadata.rs");
    let sanitized = out_dir.join("metadata_sanitized.rs");

    println!("cargo:rerun-if-changed={}", proto.display());
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    protobuf_codegen::Codegen::new()
        .protoc()
        .protoc_path(&protoc)
        .out_dir(&out_dir)
        .inputs([proto.as_path()])
        .includes([proto_root.as_path()])
        .run_from_script();

    let raw = fs::read_to_string(&generated).expect("generated metadata.rs must exist");
    let filtered = raw
        .lines()
        .filter(|line| !line.starts_with("#!") && !line.starts_with("//!"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&sanitized, filtered).expect("failed to write sanitized metadata.rs");
}
