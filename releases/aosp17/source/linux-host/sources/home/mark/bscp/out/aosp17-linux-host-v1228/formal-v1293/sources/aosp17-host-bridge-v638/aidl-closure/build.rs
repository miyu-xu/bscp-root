use std::{env,fs,path::PathBuf};
fn main(){
 let root=PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
 let source=root.join("generated-a17");
 let template=fs::read_to_string(root.join("modules.template.rs")).unwrap();
 println!("cargo:rerun-if-changed=modules.template.rs");
 println!("cargo:rerun-if-changed=generated-a17");
 let path=source.to_str().expect("UTF-8 generated module path").replace('\\',"/");
 assert!(!path.contains('"') && !path.contains('\n'),"invalid module path");
 let output=template.replace("@AIDL_ROOT@",&path);
 fs::write(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("modules.rs"),output).unwrap();
}
