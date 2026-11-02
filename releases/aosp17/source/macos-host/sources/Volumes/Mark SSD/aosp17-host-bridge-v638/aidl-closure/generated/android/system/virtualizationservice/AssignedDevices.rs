/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub enum r#AssignedDevices {
  Devices(Vec<String>),
  Dtbo(Option<binder::ParcelFileDescriptor>),
}
impl Default for r#AssignedDevices {
  fn default() -> Self {
    Self::Devices(vec![])
  }
}
impl binder::Parcelable for r#AssignedDevices {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    match self {
      Self::Devices(v) => {
        parcel.write(&0i32)?;
        parcel.write(v)
      }
      Self::Dtbo(v) => {
        parcel.write(&1i32)?;
        let __field_ref = v.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        parcel.write(__field_ref)
      }
    }
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    let tag: i32 = parcel.read()?;
    match tag {
      0 => {
        let value: Vec<String> = parcel.read()?;
        *self = Self::Devices(value);
        Ok(())
      }
      1 => {
        let value: Option<binder::ParcelFileDescriptor> = Some(parcel.read()?);
        *self = Self::Dtbo(value);
        Ok(())
      }
      _ => {
        Err(binder::StatusCode::BAD_VALUE)
      }
    }
  }
}
binder::impl_serialize_for_parcelable!(r#AssignedDevices);
binder::impl_deserialize_for_parcelable!(r#AssignedDevices);
impl binder::binder_impl::ParcelableMetadata for r#AssignedDevices {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.AssignedDevices" }
}
pub mod r#Tag {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(C, align(4))]
    r#Tag : [i32; 2] {
      r#devices = 0,
      r#dtbo = 1,
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#AssignedDevices as _7_android_6_system_21_virtualizationservice_15_AssignedDevices;
 pub use super::r#Tag::r#Tag as _7_android_6_system_21_virtualizationservice_15_AssignedDevices_3_Tag;
}
