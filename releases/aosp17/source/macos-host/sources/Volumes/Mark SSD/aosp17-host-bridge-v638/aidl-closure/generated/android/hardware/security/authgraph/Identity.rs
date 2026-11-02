/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#Identity {
  pub r#identity: Vec<u8>,
}
impl Default for r#Identity {
  fn default() -> Self {
    Self {
      r#identity: Default::default(),
    }
  }
}
impl binder::Parcelable for r#Identity {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#identity)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#identity = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#Identity);
binder::impl_deserialize_for_parcelable!(r#Identity);
impl binder::binder_impl::ParcelableMetadata for r#Identity {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.Identity" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#Identity as _7_android_8_hardware_8_security_9_authgraph_8_Identity;
}
