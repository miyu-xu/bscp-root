/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#PublicKey {
  pub r#keyMaterial: Vec<u8>,
}
impl Default for r#PublicKey {
  fn default() -> Self {
    Self {
      r#keyMaterial: Default::default(),
    }
  }
}
impl binder::Parcelable for r#PublicKey {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#keyMaterial)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#keyMaterial = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#PublicKey);
binder::impl_deserialize_for_parcelable!(r#PublicKey);
impl binder::binder_impl::ParcelableMetadata for r#PublicKey {
  fn get_descriptor() -> &'static str { "android.hardware.security.secretkeeper.PublicKey" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#PublicKey as _7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey;
}
