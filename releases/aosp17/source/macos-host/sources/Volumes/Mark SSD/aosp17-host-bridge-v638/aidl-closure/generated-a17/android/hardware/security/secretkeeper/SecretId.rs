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
pub struct r#SecretId {
  pub r#id: [u8; 64],
}
impl Default for r#SecretId {
  fn default() -> Self {
    Self {
      r#id: [Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default()],
    }
  }
}
impl binder::Parcelable for r#SecretId {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#id)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#id = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SecretId);
binder::impl_deserialize_for_parcelable!(r#SecretId);
impl binder::binder_impl::ParcelableMetadata for r#SecretId {
  fn get_descriptor() -> &'static str { "android.hardware.security.secretkeeper.SecretId" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#SecretId as _7_android_8_hardware_8_security_12_secretkeeper_8_SecretId;
}
