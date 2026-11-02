/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#DisplayConfig {
  pub r#width: i32,
  pub r#height: i32,
  pub r#horizontalDpi: i32,
  pub r#verticalDpi: i32,
  pub r#refreshRate: i32,
}
impl Default for r#DisplayConfig {
  fn default() -> Self {
    Self {
      r#width: 0,
      r#height: 0,
      r#horizontalDpi: 0,
      r#verticalDpi: 0,
      r#refreshRate: 0,
    }
  }
}
impl binder::Parcelable for r#DisplayConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#width)?;
      subparcel.write(&self.r#height)?;
      subparcel.write(&self.r#horizontalDpi)?;
      subparcel.write(&self.r#verticalDpi)?;
      subparcel.write(&self.r#refreshRate)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#width = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#height = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#horizontalDpi = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#verticalDpi = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#refreshRate = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#DisplayConfig);
binder::impl_deserialize_for_parcelable!(r#DisplayConfig);
impl binder::binder_impl::ParcelableMetadata for r#DisplayConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.DisplayConfig" }
}
pub(crate) mod mangled {
 pub use super::r#DisplayConfig as _7_android_6_system_21_virtualizationservice_13_DisplayConfig;
}
