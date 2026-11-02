/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 *
 * DO NOT CHECK THIS FILE INTO A CODE TREE (e.g. git, etc..).
 * ALWAYS GENERATE THIS FILE FROM UPDATED AIDL COMPILER
 * AS A BUILD INTERMEDIATE ONLY. THIS IS NOT SOURCE CODE.
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
use alloc::boxed::Box;
#[derive(Debug)]
pub struct r#DiskImage {
  pub r#image: Option<binder::ParcelFileDescriptor>,
  pub r#writable: bool,
  pub r#partitions: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_9_Partition>,
  pub r#id: Option<alloc::string::String>,
}
impl Default for r#DiskImage {
  fn default() -> Self {
    Self {
      r#image: Default::default(),
      r#writable: false,
      r#partitions: Default::default(),
      r#id: Default::default(),
    }
  }
}
impl binder::Parcelable for r#DiskImage {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#image)?;
      subparcel.write(&self.r#writable)?;
      subparcel.write(&self.r#partitions)?;
      subparcel.write(&self.r#id)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#image = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#writable = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#partitions = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#id = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#DiskImage);
binder::impl_deserialize_for_parcelable!(r#DiskImage);
impl binder::binder_impl::ParcelableMetadata for r#DiskImage {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.DiskImage" }
}
pub(crate) mod mangled {
 pub use super::r#DiskImage as _7_android_6_system_21_virtualizationservice_9_DiskImage;
}
