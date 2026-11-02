/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub struct r#AssignableDevice {
  pub r#node: String,
  pub r#dtbo_label: String,
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
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#node)?;
      subparcel.write(&self.r#dtbo_label)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
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
