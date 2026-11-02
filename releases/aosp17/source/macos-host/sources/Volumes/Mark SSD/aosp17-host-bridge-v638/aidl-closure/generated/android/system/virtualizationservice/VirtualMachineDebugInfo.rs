/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#VirtualMachineDebugInfo {
  pub r#name: String,
  pub r#cid: i32,
  pub r#temporaryDirectory: String,
  pub r#requesterUid: i32,
  pub r#requesterPid: i32,
  pub r#hostConsoleName: Option<String>,
}
impl Default for r#VirtualMachineDebugInfo {
  fn default() -> Self {
    Self {
      r#name: Default::default(),
      r#cid: 0,
      r#temporaryDirectory: Default::default(),
      r#requesterUid: 0,
      r#requesterPid: 0,
      r#hostConsoleName: Default::default(),
    }
  }
}
impl binder::Parcelable for r#VirtualMachineDebugInfo {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#name)?;
      subparcel.write(&self.r#cid)?;
      subparcel.write(&self.r#temporaryDirectory)?;
      subparcel.write(&self.r#requesterUid)?;
      subparcel.write(&self.r#requesterPid)?;
      subparcel.write(&self.r#hostConsoleName)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#name = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#cid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#temporaryDirectory = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#requesterUid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#requesterPid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hostConsoleName = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachineDebugInfo);
binder::impl_deserialize_for_parcelable!(r#VirtualMachineDebugInfo);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachineDebugInfo {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineDebugInfo" }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineDebugInfo as _7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo;
}
