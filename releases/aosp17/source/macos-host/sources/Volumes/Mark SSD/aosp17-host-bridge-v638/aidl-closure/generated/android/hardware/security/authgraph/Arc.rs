/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#Arc {
  pub r#arc: Vec<u8>,
}
impl Default for r#Arc {
  fn default() -> Self {
    Self {
      r#arc: Default::default(),
    }
  }
}
impl binder::Parcelable for r#Arc {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#arc)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#arc = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#Arc);
binder::impl_deserialize_for_parcelable!(r#Arc);
impl binder::binder_impl::ParcelableMetadata for r#Arc {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.Arc" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#Arc as _7_android_8_hardware_8_security_9_authgraph_3_Arc;
}
