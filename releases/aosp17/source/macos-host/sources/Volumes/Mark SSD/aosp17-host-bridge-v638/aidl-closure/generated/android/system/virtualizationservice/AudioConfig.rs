/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#AudioConfig {
  pub r#useMicrophone: bool,
  pub r#useSpeaker: bool,
}
impl Default for r#AudioConfig {
  fn default() -> Self {
    Self {
      r#useMicrophone: false,
      r#useSpeaker: false,
    }
  }
}
impl binder::Parcelable for r#AudioConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#useMicrophone)?;
      subparcel.write(&self.r#useSpeaker)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#useMicrophone = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#useSpeaker = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#AudioConfig);
binder::impl_deserialize_for_parcelable!(r#AudioConfig);
impl binder::binder_impl::ParcelableMetadata for r#AudioConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.AudioConfig" }
}
pub(crate) mod mangled {
 pub use super::r#AudioConfig as _7_android_6_system_21_virtualizationservice_11_AudioConfig;
}
