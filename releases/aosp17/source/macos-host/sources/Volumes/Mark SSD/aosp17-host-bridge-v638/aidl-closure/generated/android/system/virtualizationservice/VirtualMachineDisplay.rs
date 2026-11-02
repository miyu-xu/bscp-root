/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#VirtualMachineDisplay {
  pub r#id: i32,
  pub r#config: crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig,
}
impl Default for r#VirtualMachineDisplay {
  fn default() -> Self {
    Self {
      r#id: 0,
      r#config: Default::default(),
    }
  }
}
impl binder::Parcelable for r#VirtualMachineDisplay {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#id)?;
      subparcel.write(&self.r#config)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#id = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#config = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachineDisplay);
binder::impl_deserialize_for_parcelable!(r#VirtualMachineDisplay);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachineDisplay {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineDisplay" }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineDisplay as _7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay;
}
