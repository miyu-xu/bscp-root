/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#SessionIdSignature {
  pub r#signature: Vec<u8>,
}
impl Default for r#SessionIdSignature {
  fn default() -> Self {
    Self {
      r#signature: Default::default(),
    }
  }
}
impl binder::Parcelable for r#SessionIdSignature {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#signature)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#signature = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SessionIdSignature);
binder::impl_deserialize_for_parcelable!(r#SessionIdSignature);
impl binder::binder_impl::ParcelableMetadata for r#SessionIdSignature {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.SessionIdSignature" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#SessionIdSignature as _7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature;
}
