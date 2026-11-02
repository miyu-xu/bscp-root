/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub struct r#CpuOptions {
  pub r#cpuTopology: crate::mangled::_7_android_6_system_21_virtualizationservice_10_CpuOptions_11_CpuTopology,
}
impl Default for r#CpuOptions {
  fn default() -> Self {
    Self {
      r#cpuTopology: Default::default(),
    }
  }
}
impl binder::Parcelable for r#CpuOptions {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#cpuTopology)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#cpuTopology = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#CpuOptions);
binder::impl_deserialize_for_parcelable!(r#CpuOptions);
impl binder::binder_impl::ParcelableMetadata for r#CpuOptions {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.CpuOptions" }
}
pub mod r#CpuTopology {
  #[derive(Debug, Clone, PartialEq)]
  pub enum r#CpuTopology {
    CpuCount(i32),
    MatchHost(bool),
  }
  impl Default for r#CpuTopology {
    fn default() -> Self {
      Self::CpuCount(1)
    }
  }
  impl binder::Parcelable for r#CpuTopology {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      match self {
        Self::CpuCount(v) => {
          parcel.write(&0i32)?;
          parcel.write(v)
        }
        Self::MatchHost(v) => {
          parcel.write(&1i32)?;
          parcel.write(v)
        }
      }
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      let tag: i32 = parcel.read()?;
      match tag {
        0 => {
          let value: i32 = parcel.read()?;
          *self = Self::CpuCount(value);
          Ok(())
        }
        1 => {
          let value: bool = parcel.read()?;
          *self = Self::MatchHost(value);
          Ok(())
        }
        _ => {
          Err(binder::StatusCode::BAD_VALUE)
        }
      }
    }
  }
  binder::impl_serialize_for_parcelable!(r#CpuTopology);
  binder::impl_deserialize_for_parcelable!(r#CpuTopology);
  impl binder::binder_impl::ParcelableMetadata for r#CpuTopology {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.CpuOptions.CpuTopology" }
  }
  pub mod r#Tag {
    #![allow(non_upper_case_globals)]
    use binder::declare_binder_enum;
    declare_binder_enum! {
      #[repr(C, align(4))]
      r#Tag : [i32; 2] {
        r#cpuCount = 0,
        r#matchHost = 1,
      }
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#CpuOptions as _7_android_6_system_21_virtualizationservice_10_CpuOptions;
 pub use super::r#CpuTopology::r#CpuTopology as _7_android_6_system_21_virtualizationservice_10_CpuOptions_11_CpuTopology;
 pub use super::r#CpuTopology::r#Tag::r#Tag as _7_android_6_system_21_virtualizationservice_10_CpuOptions_11_CpuTopology_3_Tag;
}
