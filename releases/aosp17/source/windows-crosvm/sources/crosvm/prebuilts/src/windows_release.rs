// Verified offline Windows GNU release dependencies. See PREBUILTS-WINDOWS.json.
// No build script downloads executable content or trusts an unverified cache.
use std::{env,fs::{self,File,OpenOptions},io::{Read,Write},path::Path};
use anyhow::{anyhow,bail,Result};
use sha2::{Digest,Sha256};
fn pin(library:&str,version:u32,name:&str)->Result<(u64,&'static str)>{
 if version!=1{bail!("unlocked prebuilt version")}
 Ok(match (library,name){
 ("sandbox","libsandbox.lib")=>(95612356,"2a64a125b9ef49f5873ec13e9f8dfc6348d8e9596edb1a93d708f54f28ededce"),
 ("libslirp","libslirp.lib")=>(14714,"9a412b84c21481887159fe9521cc13cd286de3eb52152cc7b5baf49bda1bf922"),
 ("libslirp","libslirp-0.dll")=>(3281706,"592390eb3c6d8d15ecde18585e33bd0770b617221339910cd2fed5f041535cb7"),
 ("libslirp","libglib-2.0.dll.a")=>(1244358,"5c2b9f09f83c8ed144873a1342005de33ab9c283e25eca28d8c5ffb1e825de5e"),
 _=>bail!("unlocked Windows prebuilt {library}/{name}")})
}
fn verify(path:&Path,size:u64,expected:&str)->Result<()>{
 let meta=fs::symlink_metadata(path)?;if !meta.is_file()||meta.file_type().is_symlink()||meta.len()!=size{bail!("prebuilt type/size mismatch: {}",path.display())}
 let mut f=File::open(path)?;let mut hash=Sha256::new();let mut buf=[0;65536];loop{let n=f.read(&mut buf)?;if n==0{break}hash.update(&buf[..n]);}
 if format!("{:x}",hash.finalize())!=expected{bail!("prebuilt SHA256 mismatch: {}",path.display())}Ok(())
}
fn copy_verified(src:&Path,dst:&Path,size:u64,hash:&str)->Result<()>{
 if dst.try_exists()?{return verify(dst,size,hash)}
 let mut input=File::open(src)?;let mut output=OpenOptions::new().write(true).create_new(true).open(dst)?;std::io::copy(&mut input,&mut output)?;output.flush()?;output.sync_all()?;drop(output);verify(dst,size,hash)
}
pub(super) fn install(library:&str,version:u32,name:&str,dest:&Path,link:&Path)->Result<std::path::PathBuf>{
 println!("cargo:rerun-if-env-changed=CROSVM_PREBUILT_ROOT");
 if env::var("CARGO_CFG_TARGET_ARCH").as_deref()!=Ok("x86_64")||env::var("PROFILE").as_deref()!=Ok("release"){bail!("only locked x86_64 GNU release prebuilts are available")}
 let(size,hash)=pin(library,version,name)?;
 let root=std::path::PathBuf::from(env::var_os("CROSVM_PREBUILT_ROOT").ok_or_else(||anyhow!("set CROSVM_PREBUILT_ROOT to the verified offline dependency directory"))?);
 let src=root.join(library).join("release").join(version.to_string()).join(name);
 if !src.canonicalize()?.starts_with(root.canonicalize()?){bail!("prebuilt escapes configured root")}
 println!("cargo:rerun-if-changed={}",src.display());verify(&src,size,hash)?;
 let lock=named_lock::NamedLock::create("crosvm_prebuilts_download")?;let _guard=lock.lock()?;
 copy_verified(&src,dest,size,hash)?;copy_verified(dest,link,size,hash)?;Ok(dest.to_owned())
}
#[cfg(test)]mod tests{use super::*;#[test]fn exact_pins(){assert!(pin("sandbox",1,"libsandbox.lib").is_ok());assert!(pin("sandbox",2,"libsandbox.lib").is_err());assert!(pin("r8brain",1,"ucrtbased.dll").is_err());assert!(pin("libslirp",1,"../x").is_err());}}

#[cfg(test)]mod integrity_tests{use super::*;
#[test]fn tamper_and_existing_cache_fail_closed(){
 let root=std::env::temp_dir().join(format!("crosvm-prebuilt-integrity-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));fs::create_dir(&root).unwrap();let src=root.join("source");let dst=root.join("cache");fs::write(&src,b"abc").unwrap();let hash="ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
 verify(&src,3,hash).unwrap();copy_verified(&src,&dst,3,hash).unwrap();copy_verified(&src,&dst,3,hash).unwrap();fs::write(&dst,b"bad").unwrap();assert!(copy_verified(&src,&dst,3,hash).is_err());assert_eq!(fs::read(&dst).unwrap(),b"bad");assert!(verify(&src,4,hash).is_err());assert!(pin("libslirp",1,"ucrtbased.dll").is_err());
}}
