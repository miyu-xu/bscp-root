/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct r#SmbiosOptions {
  pub r#biosVendor: Option<String>,
  pub r#biosVersion: Option<String>,
  pub r#manufacturer: Option<String>,
  pub r#productName: Option<String>,
  pub r#serialNumber: Option<String>,
  pub r#uuid: Option<String>,
  pub r#oemStrings: Vec<String>,
}
impl Default for r#SmbiosOptions {
  fn default() -> Self {
    Self {
      r#biosVendor: Default::default(),
      r#biosVersion: Default::default(),
      r#manufacturer: Default::default(),
      r#productName: Default::default(),
      r#serialNumber: Default::default(),
      r#uuid: Default::default(),
      r#oemStrings: Default::default(),
    }
  }
}
impl binder::Parcelable for r#SmbiosOptions {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#biosVendor)?;
      subparcel.write(&self.r#biosVersion)?;
      subparcel.write(&self.r#manufacturer)?;
      subparcel.write(&self.r#productName)?;
      subparcel.write(&self.r#serialNumber)?;
      subparcel.write(&self.r#uuid)?;
      subparcel.write(&self.r#oemStrings)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#biosVendor = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#biosVersion = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#manufacturer = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#productName = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#serialNumber = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#uuid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#oemStrings = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SmbiosOptions);
binder::impl_deserialize_for_parcelable!(r#SmbiosOptions);
impl binder::binder_impl::ParcelableMetadata for r#SmbiosOptions {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.SmbiosOptions" }
}
pub(crate) mod mangled {
 pub use super::r#SmbiosOptions as _7_android_6_system_21_virtualizationservice_13_SmbiosOptions;
}
