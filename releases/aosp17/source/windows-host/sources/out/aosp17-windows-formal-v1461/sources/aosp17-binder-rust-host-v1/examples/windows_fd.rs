#[cfg(windows)]
fn main()->std::io::Result<()> {
    use binder::platform_fd::{AsFd,AsRawFd,FromRawFd,IntoRawFd,OwnedFd};
    use std::{fs::OpenOptions,io::{Read,Seek,SeekFrom,Write}};
    let path=std::env::temp_dir().join(format!("a17-rust-fd-{}.bin",std::process::id()));
    let mut file=OpenOptions::new().read(true).write(true).create_new(true).open(&path)?;
    file.write_all(b"A17")?;file.seek(SeekFrom::Start(0))?;
    let original=OwnedFd::try_from_handle(&file)?;
    let cloned=original.try_clone()?;drop(file);drop(original);
    let mut exported=std::fs::File::from(cloned.try_clone_to_handle()?);
    let mut data=String::new();exported.read_to_string(&mut data)?;
    assert_eq!(data,"A17");drop(exported);
    let borrowed=cloned.as_fd();assert_eq!(borrowed.as_raw_fd(),cloned.as_raw_fd());
    let fd=cloned.into_raw_fd();let owned=unsafe{OwnedFd::from_raw_fd(fd)};
    drop(owned);
    let file=OpenOptions::new().read(true).open(&path)?;
    let parcel=binder::ParcelFileDescriptor::try_from_file(file)?;
    let mut again=parcel.try_clone_to_file()?;drop(parcel);
    let mut text=String::new();again.read_to_string(&mut text)?;assert_eq!(text,"A17");drop(again);
    std::fs::remove_file(path)?;
    println!("A17_RUST_NATIVE_FD_DUPLICATE_OWNERSHIP_ROUNDTRIP_PASS");Ok(())
}
#[cfg(not(windows))]
fn main(){panic!("Windows-only actual API test");}
