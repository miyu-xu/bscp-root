use std::{env,path::PathBuf};
fn main() {
 let root=PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../native");
 let mut c=cc::Build::new(); c.include(&root);
 for f in ["fdt.c","fdt_ro.c","fdt_wip.c","fdt_sw.c","fdt_rw.c","fdt_strerror.c","fdt_empty_tree.c","fdt_addresses.c","fdt_overlay.c","fdt_check.c","acpi.c"] {
  c.file(root.join(f));
 }
 c.compile("fdt");
 bindgen::Builder::default().header(root.join("libfdt.h").to_str().unwrap())
  .clang_arg(format!("-I{}",root.display())).use_core().ctypes_prefix("core::ffi")
  .allowlist_type("fdt_.*").allowlist_function("fdt_.*").allowlist_var("FDT_.*")
  .generate().expect("libfdt bindings").write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();
 println!("cargo:rerun-if-changed={}",root.display());
}
