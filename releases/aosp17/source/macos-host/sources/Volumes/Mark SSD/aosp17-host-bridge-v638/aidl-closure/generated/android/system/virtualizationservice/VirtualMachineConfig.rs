/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub enum r#VirtualMachineConfig {
  AppConfig(crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig),
  RawConfig(crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineRawConfig),
}
impl Default for r#VirtualMachineConfig {
  fn default() -> Self {
    Self::AppConfig(Default::default())
  }
}
impl binder::Parcelable for r#VirtualMachineConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    match self {
      Self::AppConfig(v) => {
        parcel.write(&0i32)?;
        parcel.write(v)
      }
      Self::RawConfig(v) => {
        parcel.write(&1i32)?;
        parcel.write(v)
      }
    }
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    let tag: i32 = parcel.read()?;
    match tag {
      0 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig = parcel.read()?;
        *self = Self::AppConfig(value);
        Ok(())
      }
      1 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineRawConfig = parcel.read()?;
        *self = Self::RawConfig(value);
        Ok(())
      }
      _ => {
        Err(binder::StatusCode::BAD_VALUE)
      }
    }
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachineConfig);
binder::impl_deserialize_for_parcelable!(r#VirtualMachineConfig);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachineConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineConfig" }
}
pub mod r#Tag {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(C, align(4))]
    r#Tag : [i32; 2] {
      r#appConfig = 0,
      r#rawConfig = 1,
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineConfig as _7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig;
 pub use super::r#Tag::r#Tag as _7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig_3_Tag;
}
