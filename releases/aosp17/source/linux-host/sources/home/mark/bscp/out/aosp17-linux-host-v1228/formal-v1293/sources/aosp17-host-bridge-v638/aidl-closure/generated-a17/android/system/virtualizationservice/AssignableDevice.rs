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
#[derive(Debug, Clone)]
pub struct r#AssignableDevice {
  pub r#node: alloc::string::String,
  pub r#dtbo_label: alloc::string::String,
}
impl Default for r#AssignableDevice {
  fn default() -> Self {
    Self {
      r#node: Default::default(),
      r#dtbo_label: Default::default(),
    }
  }
}
impl binder::Parcelable for r#AssignableDevice {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#node)?;
      subparcel.write(&self.r#dtbo_label)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#node = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#dtbo_label = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#AssignableDevice);
binder::impl_deserialize_for_parcelable!(r#AssignableDevice);
impl binder::binder_impl::ParcelableMetadata for r#AssignableDevice {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.AssignableDevice" }
}
pub(crate) mod mangled {
 pub use super::r#AssignableDevice as _7_android_6_system_21_virtualizationservice_16_AssignableDevice;
}
