/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#Key {
  pub r#pubKey: Option<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey>,
  pub r#arcFromPBK: Option<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc>,
}
impl Default for r#Key {
  fn default() -> Self {
    Self {
      r#pubKey: Default::default(),
      r#arcFromPBK: Default::default(),
    }
  }
}
impl binder::Parcelable for r#Key {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#pubKey)?;
      subparcel.write(&self.r#arcFromPBK)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#pubKey = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#arcFromPBK = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#Key);
binder::impl_deserialize_for_parcelable!(r#Key);
impl binder::binder_impl::ParcelableMetadata for r#Key {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.Key" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#Key as _7_android_8_hardware_8_security_9_authgraph_3_Key;
}
