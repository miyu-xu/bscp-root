//! Fixed per-user allocation leases, not disk recovery claims.
//! Security implementation is shared with the client; no ACL repair.
use std::{fs::File,io,path::PathBuf};
pub(crate) use desktop_host::windows_private_directory::ExistingPrivateDirectoryGuard;
pub fn root_path()->io::Result<PathBuf>{Ok(crate::desktop_authority::process_profile()?.join(".bscp-microdroid").join("leases"))}
pub fn acquire(key:&str)->io::Result<File>{
 desktop_host::windows_private_directory::acquire_in_existing_profile(&crate::desktop_authority::process_profile()?,key)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn invalid_keys_do_not_touch_files(){for s in ["","../x","x/y","x.y","CON:"]{assert!(acquire(s).is_err());}}
 #[test]fn fixed_profile_namespace(){let p=root_path().unwrap();assert!(p.ends_with(".bscp-microdroid/leases"));}
}
