/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#Certificate {
  pub r#encodedCertificate: Vec<u8>,
}
impl Default for r#Certificate {
  fn default() -> Self {
    Self {
      r#encodedCertificate: Default::default(),
    }
  }
}
impl binder::Parcelable for r#Certificate {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#encodedCertificate)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#encodedCertificate = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#Certificate);
binder::impl_deserialize_for_parcelable!(r#Certificate);
impl binder::binder_impl::ParcelableMetadata for r#Certificate {
  fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Certificate" }
}
pub(crate) mod mangled {
 pub use super::r#Certificate as _7_android_6_system_20_virtualizationcommon_11_Certificate;
}
