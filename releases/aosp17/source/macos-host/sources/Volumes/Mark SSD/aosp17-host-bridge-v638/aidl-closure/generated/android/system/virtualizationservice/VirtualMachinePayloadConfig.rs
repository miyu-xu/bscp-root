/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#VirtualMachinePayloadConfig {
  pub r#payloadBinaryName: String,
  pub r#extraApks: Vec<binder::ParcelFileDescriptor>,
}
impl Default for r#VirtualMachinePayloadConfig {
  fn default() -> Self {
    Self {
      r#payloadBinaryName: Default::default(),
      r#extraApks: Default::default(),
    }
  }
}
impl binder::Parcelable for r#VirtualMachinePayloadConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#payloadBinaryName)?;
      subparcel.write(&self.r#extraApks)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#payloadBinaryName = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#extraApks = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachinePayloadConfig);
binder::impl_deserialize_for_parcelable!(r#VirtualMachinePayloadConfig);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachinePayloadConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachinePayloadConfig" }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachinePayloadConfig as _7_android_6_system_21_virtualizationservice_27_VirtualMachinePayloadConfig;
}
