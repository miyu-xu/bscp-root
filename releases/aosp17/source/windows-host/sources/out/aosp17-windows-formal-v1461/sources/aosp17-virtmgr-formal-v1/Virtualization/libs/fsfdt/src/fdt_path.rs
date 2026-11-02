//! Host filesystem separators must never become FDT separators or lossy names.
use std::{io,path::{Path,Component}};
pub fn relative_bytes(path:&Path)->io::Result<Vec<u8>> {
 let mut bytes=Vec::new();
 for component in path.components() {
  let Component::Normal(name)=component else {return Err(io::Error::new(io::ErrorKind::InvalidInput,"FDT path must contain only relative normal components"));};
  let name=name.to_str().filter(|n|n.is_ascii() && !n.as_bytes().contains(&0)).ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"FDT name must be ASCII without NUL"))?;
  if !bytes.is_empty(){bytes.push(b'/');}bytes.extend_from_slice(name.as_bytes());
 }
 Ok(bytes)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn native_separator_to_fdt(){assert_eq!(relative_bytes(&Path::new("soc").join("uart@1000")).unwrap(),b"soc/uart@1000");}
 #[test] fn empty_root(){assert!(relative_bytes(Path::new("")).unwrap().is_empty());}
 #[test] fn no_parent_or_absolute(){assert!(relative_bytes(Path::new("../device")).is_err());assert!(relative_bytes(&std::env::current_dir().unwrap()).is_err());}
 #[test] fn no_lossy_names(){assert!(relative_bytes(Path::new("é")).is_err());assert!(relative_bytes(Path::new("a\0b")).is_err());}
}
