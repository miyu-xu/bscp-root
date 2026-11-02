/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#UsbConfig {
  pub r#controller: bool,
}
impl Default for r#UsbConfig {
  fn default() -> Self {
    Self {
      r#controller: false,
    }
  }
}
impl binder::Parcelable for r#UsbConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#controller)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#controller = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#UsbConfig);
binder::impl_deserialize_for_parcelable!(r#UsbConfig);
impl binder::binder_impl::ParcelableMetadata for r#UsbConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.UsbConfig" }
}
pub(crate) mod mangled {
 pub use super::r#UsbConfig as _7_android_6_system_21_virtualizationservice_9_UsbConfig;
}
