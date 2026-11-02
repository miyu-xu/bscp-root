/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#Partition {
  pub r#label: String,
  pub r#image: Option<binder::ParcelFileDescriptor>,
  pub r#writable: bool,
  pub r#guid: Option<String>,
}
impl Default for r#Partition {
  fn default() -> Self {
    Self {
      r#label: Default::default(),
      r#image: Default::default(),
      r#writable: false,
      r#guid: Default::default(),
    }
  }
}
impl binder::Parcelable for r#Partition {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#label)?;
      let __field_ref = self.r#image.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
      subparcel.write(__field_ref)?;
      subparcel.write(&self.r#writable)?;
      subparcel.write(&self.r#guid)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#label = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#image = Some(subparcel.read()?);
      }
      if subparcel.has_more_data() {
        self.r#writable = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#guid = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#Partition);
binder::impl_deserialize_for_parcelable!(r#Partition);
impl binder::binder_impl::ParcelableMetadata for r#Partition {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.Partition" }
}
pub(crate) mod mangled {
 pub use super::r#Partition as _7_android_6_system_21_virtualizationservice_9_Partition;
}
