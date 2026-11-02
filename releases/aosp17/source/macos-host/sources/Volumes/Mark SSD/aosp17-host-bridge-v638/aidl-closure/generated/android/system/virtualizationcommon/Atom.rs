/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub enum r#Atom {
  CgroupMemoryBreachReported(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_26_CgroupMemoryBreachReported),
  FsckFailedReported(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_18_FsckFailedReported),
  GetOrCreateSkSecretFailedReported(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_33_GetOrCreateSkSecretFailedReported),
  PsiMonitorFailedReported(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_24_PsiMonitorFailedReported),
  StaleEncryptedstoreDetected(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_27_StaleEncryptedstoreDetected),
  VmBooted(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_8_VmBooted),
  VmCreationRequested(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_19_VmCreationRequested),
  VmExited(crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_8_VmExited),
}
impl Default for r#Atom {
  fn default() -> Self {
    Self::CgroupMemoryBreachReported(Default::default())
  }
}
impl binder::Parcelable for r#Atom {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    match self {
      Self::CgroupMemoryBreachReported(v) => {
        parcel.write(&0i32)?;
        parcel.write(v)
      }
      Self::FsckFailedReported(v) => {
        parcel.write(&1i32)?;
        parcel.write(v)
      }
      Self::GetOrCreateSkSecretFailedReported(v) => {
        parcel.write(&2i32)?;
        parcel.write(v)
      }
      Self::PsiMonitorFailedReported(v) => {
        parcel.write(&3i32)?;
        parcel.write(v)
      }
      Self::StaleEncryptedstoreDetected(v) => {
        parcel.write(&4i32)?;
        parcel.write(v)
      }
      Self::VmBooted(v) => {
        parcel.write(&5i32)?;
        parcel.write(v)
      }
      Self::VmCreationRequested(v) => {
        parcel.write(&6i32)?;
        parcel.write(v)
      }
      Self::VmExited(v) => {
        parcel.write(&7i32)?;
        parcel.write(v)
      }
    }
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    let tag: i32 = parcel.read()?;
    match tag {
      0 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_26_CgroupMemoryBreachReported = parcel.read()?;
        *self = Self::CgroupMemoryBreachReported(value);
        Ok(())
      }
      1 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_18_FsckFailedReported = parcel.read()?;
        *self = Self::FsckFailedReported(value);
        Ok(())
      }
      2 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_33_GetOrCreateSkSecretFailedReported = parcel.read()?;
        *self = Self::GetOrCreateSkSecretFailedReported(value);
        Ok(())
      }
      3 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_24_PsiMonitorFailedReported = parcel.read()?;
        *self = Self::PsiMonitorFailedReported(value);
        Ok(())
      }
      4 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_27_StaleEncryptedstoreDetected = parcel.read()?;
        *self = Self::StaleEncryptedstoreDetected(value);
        Ok(())
      }
      5 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_8_VmBooted = parcel.read()?;
        *self = Self::VmBooted(value);
        Ok(())
      }
      6 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_19_VmCreationRequested = parcel.read()?;
        *self = Self::VmCreationRequested(value);
        Ok(())
      }
      7 => {
        let value: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom_8_VmExited = parcel.read()?;
        *self = Self::VmExited(value);
        Ok(())
      }
      _ => {
        Err(binder::StatusCode::BAD_VALUE)
      }
    }
  }
}
binder::impl_serialize_for_parcelable!(r#Atom);
binder::impl_deserialize_for_parcelable!(r#Atom);
impl binder::binder_impl::ParcelableMetadata for r#Atom {
  fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom" }
}
pub mod r#CgroupMemoryBreachReported {
  #[derive(Debug)]
  pub struct r#CgroupMemoryBreachReported {
    pub r#highBreachCount: i64,
    pub r#highMemoryPeakMb: i64,
  }
  impl Default for r#CgroupMemoryBreachReported {
    fn default() -> Self {
      Self {
        r#highBreachCount: 0,
        r#highMemoryPeakMb: 0,
      }
    }
  }
  impl binder::Parcelable for r#CgroupMemoryBreachReported {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#highBreachCount)?;
        subparcel.write(&self.r#highMemoryPeakMb)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#highBreachCount = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#highMemoryPeakMb = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#CgroupMemoryBreachReported);
  binder::impl_deserialize_for_parcelable!(r#CgroupMemoryBreachReported);
  impl binder::binder_impl::ParcelableMetadata for r#CgroupMemoryBreachReported {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.CgroupMemoryBreachReported" }
  }
}
pub mod r#FsckFailedReported {
  #[derive(Debug)]
  pub struct r#FsckFailedReported {
    pub r#exitCode: i32,
  }
  impl Default for r#FsckFailedReported {
    fn default() -> Self {
      Self {
        r#exitCode: 0,
      }
    }
  }
  impl binder::Parcelable for r#FsckFailedReported {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#exitCode)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#exitCode = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#FsckFailedReported);
  binder::impl_deserialize_for_parcelable!(r#FsckFailedReported);
  impl binder::binder_impl::ParcelableMetadata for r#FsckFailedReported {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.FsckFailedReported" }
  }
}
pub mod r#GetOrCreateSkSecretFailedReported {
  #[derive(Debug)]
  pub struct r#GetOrCreateSkSecretFailedReported {
    pub r#retryCount: i32,
  }
  impl Default for r#GetOrCreateSkSecretFailedReported {
    fn default() -> Self {
      Self {
        r#retryCount: 0,
      }
    }
  }
  impl binder::Parcelable for r#GetOrCreateSkSecretFailedReported {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#retryCount)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#retryCount = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#GetOrCreateSkSecretFailedReported);
  binder::impl_deserialize_for_parcelable!(r#GetOrCreateSkSecretFailedReported);
  impl binder::binder_impl::ParcelableMetadata for r#GetOrCreateSkSecretFailedReported {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.GetOrCreateSkSecretFailedReported" }
  }
}
pub mod r#PsiMonitorFailedReported {
  #[derive(Debug)]
  pub struct r#PsiMonitorFailedReported {
    pub r#exponentialBackoffSeconds: i64,
  }
  impl Default for r#PsiMonitorFailedReported {
    fn default() -> Self {
      Self {
        r#exponentialBackoffSeconds: 0,
      }
    }
  }
  impl binder::Parcelable for r#PsiMonitorFailedReported {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#exponentialBackoffSeconds)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#exponentialBackoffSeconds = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#PsiMonitorFailedReported);
  binder::impl_deserialize_for_parcelable!(r#PsiMonitorFailedReported);
  impl binder::binder_impl::ParcelableMetadata for r#PsiMonitorFailedReported {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.PsiMonitorFailedReported" }
  }
}
pub mod r#StaleEncryptedstoreDetected {
  #[derive(Debug)]
  pub struct r#StaleEncryptedstoreDetected {
  }
  impl Default for r#StaleEncryptedstoreDetected {
    fn default() -> Self {
      Self {
      }
    }
  }
  impl binder::Parcelable for r#StaleEncryptedstoreDetected {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#StaleEncryptedstoreDetected);
  binder::impl_deserialize_for_parcelable!(r#StaleEncryptedstoreDetected);
  impl binder::binder_impl::ParcelableMetadata for r#StaleEncryptedstoreDetected {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.StaleEncryptedstoreDetected" }
  }
}
pub mod r#VmBooted {
  #[derive(Debug)]
  pub struct r#VmBooted {
    pub r#elapsedTimeMillis: i64,
  }
  impl Default for r#VmBooted {
    fn default() -> Self {
      Self {
        r#elapsedTimeMillis: 0,
      }
    }
  }
  impl binder::Parcelable for r#VmBooted {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#elapsedTimeMillis)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#elapsedTimeMillis = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#VmBooted);
  binder::impl_deserialize_for_parcelable!(r#VmBooted);
  impl binder::binder_impl::ParcelableMetadata for r#VmBooted {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.VmBooted" }
  }
}
pub mod r#VmCreationRequested {
  #[derive(Debug)]
  pub struct r#VmCreationRequested {
    pub r#isProtected: bool,
    pub r#creationSucceeded: bool,
    pub r#binderExceptionCode: i32,
    pub r#configType: i32,
    pub r#numCpus: i32,
    pub r#memoryMib: i32,
    pub r#apexes: String,
  }
  impl Default for r#VmCreationRequested {
    fn default() -> Self {
      Self {
        r#isProtected: false,
        r#creationSucceeded: false,
        r#binderExceptionCode: 0,
        r#configType: 0,
        r#numCpus: 0,
        r#memoryMib: 0,
        r#apexes: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#VmCreationRequested {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#isProtected)?;
        subparcel.write(&self.r#creationSucceeded)?;
        subparcel.write(&self.r#binderExceptionCode)?;
        subparcel.write(&self.r#configType)?;
        subparcel.write(&self.r#numCpus)?;
        subparcel.write(&self.r#memoryMib)?;
        subparcel.write(&self.r#apexes)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#isProtected = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#creationSucceeded = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#binderExceptionCode = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#configType = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#numCpus = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#memoryMib = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#apexes = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#VmCreationRequested);
  binder::impl_deserialize_for_parcelable!(r#VmCreationRequested);
  impl binder::binder_impl::ParcelableMetadata for r#VmCreationRequested {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.VmCreationRequested" }
  }
}
pub mod r#VmExited {
  #[derive(Debug)]
  pub struct r#VmExited {
    pub r#deathReason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason,
    pub r#exitSignal: i32,
    pub r#elapsedTimeMillis: i64,
    pub r#guestTimeMillis: i64,
    pub r#rssKb: i64,
  }
  impl Default for r#VmExited {
    fn default() -> Self {
      Self {
        r#deathReason: Default::default(),
        r#exitSignal: 0,
        r#elapsedTimeMillis: 0,
        r#guestTimeMillis: 0,
        r#rssKb: 0,
      }
    }
  }
  impl binder::Parcelable for r#VmExited {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#deathReason)?;
        subparcel.write(&self.r#exitSignal)?;
        subparcel.write(&self.r#elapsedTimeMillis)?;
        subparcel.write(&self.r#guestTimeMillis)?;
        subparcel.write(&self.r#rssKb)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#deathReason = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#exitSignal = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#elapsedTimeMillis = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#guestTimeMillis = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#rssKb = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#VmExited);
  binder::impl_deserialize_for_parcelable!(r#VmExited);
  impl binder::binder_impl::ParcelableMetadata for r#VmExited {
    fn get_descriptor() -> &'static str { "android.system.virtualizationcommon.Atom.VmExited" }
  }
}
pub mod r#Tag {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(C, align(4))]
    r#Tag : [i32; 8] {
      r#cgroupMemoryBreachReported = 0,
      r#fsckFailedReported = 1,
      r#getOrCreateSkSecretFailedReported = 2,
      r#psiMonitorFailedReported = 3,
      r#staleEncryptedstoreDetected = 4,
      r#vmBooted = 5,
      r#vmCreationRequested = 6,
      r#vmExited = 7,
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#Atom as _7_android_6_system_20_virtualizationcommon_4_Atom;
 pub use super::r#CgroupMemoryBreachReported::r#CgroupMemoryBreachReported as _7_android_6_system_20_virtualizationcommon_4_Atom_26_CgroupMemoryBreachReported;
 pub use super::r#FsckFailedReported::r#FsckFailedReported as _7_android_6_system_20_virtualizationcommon_4_Atom_18_FsckFailedReported;
 pub use super::r#GetOrCreateSkSecretFailedReported::r#GetOrCreateSkSecretFailedReported as _7_android_6_system_20_virtualizationcommon_4_Atom_33_GetOrCreateSkSecretFailedReported;
 pub use super::r#PsiMonitorFailedReported::r#PsiMonitorFailedReported as _7_android_6_system_20_virtualizationcommon_4_Atom_24_PsiMonitorFailedReported;
 pub use super::r#StaleEncryptedstoreDetected::r#StaleEncryptedstoreDetected as _7_android_6_system_20_virtualizationcommon_4_Atom_27_StaleEncryptedstoreDetected;
 pub use super::r#VmBooted::r#VmBooted as _7_android_6_system_20_virtualizationcommon_4_Atom_8_VmBooted;
 pub use super::r#VmCreationRequested::r#VmCreationRequested as _7_android_6_system_20_virtualizationcommon_4_Atom_19_VmCreationRequested;
 pub use super::r#VmExited::r#VmExited as _7_android_6_system_20_virtualizationcommon_4_Atom_8_VmExited;
 pub use super::r#Tag::r#Tag as _7_android_6_system_20_virtualizationcommon_4_Atom_3_Tag;
}
