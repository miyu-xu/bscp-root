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
pub struct r#CustomMemoryBackingFile {
  pub r#file: Option<binder::ParcelFileDescriptor>,
  pub r#rangeStart: i64,
  pub r#size: i64,
}
impl Default for r#CustomMemoryBackingFile {
  fn default() -> Self {
    Self {
      r#file: Default::default(),
      r#rangeStart: 0,
      r#size: 0,
    }
  }
}
impl binder::Parcelable for r#CustomMemoryBackingFile {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      let __field_ref = self.r#file.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
      subparcel.write(__field_ref)?;
      subparcel.write(&self.r#rangeStart)?;
      subparcel.write(&self.r#size)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#file = Some(subparcel.read()?);
      }
      if subparcel.has_more_data() {
        self.r#rangeStart = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#size = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#CustomMemoryBackingFile);
binder::impl_deserialize_for_parcelable!(r#CustomMemoryBackingFile);
impl binder::binder_impl::ParcelableMetadata for r#CustomMemoryBackingFile {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.CustomMemoryBackingFile" }
}
pub(crate) mod mangled {
 pub use super::r#CustomMemoryBackingFile as _7_android_6_system_21_virtualizationservice_23_CustomMemoryBackingFile;
}
