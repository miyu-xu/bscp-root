/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 *
 * DO NOT CHECK THIS FILE INTO A CODE TREE (e.g. git, etc..).
 * ALWAYS GENERATE THIS FILE FROM UPDATED AIDL COMPILER
 * AS A BUILD INTERMEDIATE ONLY. THIS IS NOT SOURCE CODE.
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
use alloc::boxed::Box;
#[derive(Debug)]
pub enum r#InputDevice {
  SingleTouch(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_11_SingleTouch),
  EvDev(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_5_EvDev),
  Keyboard(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Keyboard),
  Mouse(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_5_Mouse),
  Switches(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Switches),
  Trackpad(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Trackpad),
  MultiTouch(crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_10_MultiTouch),
}
impl Default for r#InputDevice {
  fn default() -> Self {
    Self::SingleTouch(Default::default())
  }
}
impl binder::Parcelable for r#InputDevice {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    match self {
      Self::SingleTouch(v) => {
        parcel.write(&0i32)?;
        parcel.write(v)
      }
      Self::EvDev(v) => {
        parcel.write(&1i32)?;
        parcel.write(v)
      }
      Self::Keyboard(v) => {
        parcel.write(&2i32)?;
        parcel.write(v)
      }
      Self::Mouse(v) => {
        parcel.write(&3i32)?;
        parcel.write(v)
      }
      Self::Switches(v) => {
        parcel.write(&4i32)?;
        parcel.write(v)
      }
      Self::Trackpad(v) => {
        parcel.write(&5i32)?;
        parcel.write(v)
      }
      Self::MultiTouch(v) => {
        parcel.write(&6i32)?;
        parcel.write(v)
      }
    }
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    let tag: i32 = parcel.read()?;
    match tag {
      0 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_11_SingleTouch = parcel.read()?;
        *self = Self::SingleTouch(value);
        Ok(())
      }
      1 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_5_EvDev = parcel.read()?;
        *self = Self::EvDev(value);
        Ok(())
      }
      2 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Keyboard = parcel.read()?;
        *self = Self::Keyboard(value);
        Ok(())
      }
      3 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_5_Mouse = parcel.read()?;
        *self = Self::Mouse(value);
        Ok(())
      }
      4 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Switches = parcel.read()?;
        *self = Self::Switches(value);
        Ok(())
      }
      5 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_8_Trackpad = parcel.read()?;
        *self = Self::Trackpad(value);
        Ok(())
      }
      6 => {
        let value: crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice_10_MultiTouch = parcel.read()?;
        *self = Self::MultiTouch(value);
        Ok(())
      }
      _ => {
        Err(binder::StatusCode::BAD_VALUE)
      }
    }
  }
}
binder::impl_serialize_for_parcelable!(r#InputDevice);
binder::impl_deserialize_for_parcelable!(r#InputDevice);
impl binder::binder_impl::ParcelableMetadata for r#InputDevice {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice" }
}
pub mod r#SingleTouch {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#SingleTouch {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
    pub r#width: i32,
    pub r#height: i32,
    pub r#name: alloc::string::String,
  }
  impl Default for r#SingleTouch {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
        r#width: 1280,
        r#height: 1080,
        r#name: "".into(),
      }
    }
  }
  impl binder::Parcelable for r#SingleTouch {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        subparcel.write(&self.r#width)?;
        subparcel.write(&self.r#height)?;
        subparcel.write(&self.r#name)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        if subparcel.has_more_data() {
          self.r#width = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#height = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#name = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#SingleTouch);
  binder::impl_deserialize_for_parcelable!(r#SingleTouch);
  impl binder::binder_impl::ParcelableMetadata for r#SingleTouch {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.SingleTouch" }
  }
}
pub mod r#EvDev {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#EvDev {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
  }
  impl Default for r#EvDev {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#EvDev {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#EvDev);
  binder::impl_deserialize_for_parcelable!(r#EvDev);
  impl binder::binder_impl::ParcelableMetadata for r#EvDev {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.EvDev" }
  }
}
pub mod r#Keyboard {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#Keyboard {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
  }
  impl Default for r#Keyboard {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#Keyboard {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#Keyboard);
  binder::impl_deserialize_for_parcelable!(r#Keyboard);
  impl binder::binder_impl::ParcelableMetadata for r#Keyboard {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.Keyboard" }
  }
}
pub mod r#Mouse {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#Mouse {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
  }
  impl Default for r#Mouse {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#Mouse {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#Mouse);
  binder::impl_deserialize_for_parcelable!(r#Mouse);
  impl binder::binder_impl::ParcelableMetadata for r#Mouse {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.Mouse" }
  }
}
pub mod r#Switches {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#Switches {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
  }
  impl Default for r#Switches {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#Switches {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#Switches);
  binder::impl_deserialize_for_parcelable!(r#Switches);
  impl binder::binder_impl::ParcelableMetadata for r#Switches {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.Switches" }
  }
}
pub mod r#Trackpad {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#Trackpad {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
    pub r#width: i32,
    pub r#height: i32,
    pub r#name: alloc::string::String,
  }
  impl Default for r#Trackpad {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
        r#width: 1280,
        r#height: 1080,
        r#name: "".into(),
      }
    }
  }
  impl binder::Parcelable for r#Trackpad {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        subparcel.write(&self.r#width)?;
        subparcel.write(&self.r#height)?;
        subparcel.write(&self.r#name)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        if subparcel.has_more_data() {
          self.r#width = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#height = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#name = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#Trackpad);
  binder::impl_deserialize_for_parcelable!(r#Trackpad);
  impl binder::binder_impl::ParcelableMetadata for r#Trackpad {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.Trackpad" }
  }
}
pub mod r#MultiTouch {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#MultiTouch {
    pub r#pfd: Option<binder::ParcelFileDescriptor>,
    pub r#width: i32,
    pub r#height: i32,
    pub r#name: alloc::string::String,
  }
  impl Default for r#MultiTouch {
    fn default() -> Self {
      Self {
        r#pfd: Default::default(),
        r#width: 1280,
        r#height: 1080,
        r#name: "".into(),
      }
    }
  }
  impl binder::Parcelable for r#MultiTouch {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        let __field_ref = self.r#pfd.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
        subparcel.write(__field_ref)?;
        subparcel.write(&self.r#width)?;
        subparcel.write(&self.r#height)?;
        subparcel.write(&self.r#name)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#pfd = Some(subparcel.read()?);
        }
        if subparcel.has_more_data() {
          self.r#width = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#height = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#name = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#MultiTouch);
  binder::impl_deserialize_for_parcelable!(r#MultiTouch);
  impl binder::binder_impl::ParcelableMetadata for r#MultiTouch {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.InputDevice.MultiTouch" }
  }
}
pub mod r#Tag {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(align(4))]
    r#Tag : [i32; 7] {
      r#singleTouch = 0,
      r#evDev = 1,
      r#keyboard = 2,
      r#mouse = 3,
      r#switches = 4,
      r#trackpad = 5,
      r#multiTouch = 6,
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#InputDevice as _7_android_6_system_21_virtualizationservice_11_InputDevice;
 pub use super::r#SingleTouch::r#SingleTouch as _7_android_6_system_21_virtualizationservice_11_InputDevice_11_SingleTouch;
 pub use super::r#EvDev::r#EvDev as _7_android_6_system_21_virtualizationservice_11_InputDevice_5_EvDev;
 pub use super::r#Keyboard::r#Keyboard as _7_android_6_system_21_virtualizationservice_11_InputDevice_8_Keyboard;
 pub use super::r#Mouse::r#Mouse as _7_android_6_system_21_virtualizationservice_11_InputDevice_5_Mouse;
 pub use super::r#Switches::r#Switches as _7_android_6_system_21_virtualizationservice_11_InputDevice_8_Switches;
 pub use super::r#Trackpad::r#Trackpad as _7_android_6_system_21_virtualizationservice_11_InputDevice_8_Trackpad;
 pub use super::r#MultiTouch::r#MultiTouch as _7_android_6_system_21_virtualizationservice_11_InputDevice_10_MultiTouch;
 pub use super::r#Tag::r#Tag as _7_android_6_system_21_virtualizationservice_11_InputDevice_3_Tag;
}
