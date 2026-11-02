/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#VirtualMachineAppConfig {
  pub r#name: String,
  pub r#instanceId: [u8; 64],
  pub r#apk: Option<binder::ParcelFileDescriptor>,
  pub r#idsig: Option<binder::ParcelFileDescriptor>,
  pub r#extraIdsigs: Vec<binder::ParcelFileDescriptor>,
  pub r#tenantApks: Vec<binder::ParcelFileDescriptor>,
  pub r#tenantIdsigs: Vec<binder::ParcelFileDescriptor>,
  pub r#instanceImage: Option<binder::ParcelFileDescriptor>,
  pub r#encryptedStorageImage: Option<binder::ParcelFileDescriptor>,
  pub r#payload: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_7_Payload,
  pub r#osName: String,
  pub r#debugLevel: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_10_DebugLevel,
  pub r#protectedVm: bool,
  pub r#memoryMib: i32,
  pub r#cpuOptions: crate::mangled::_7_android_6_system_21_virtualizationservice_10_CpuOptions,
  pub r#customConfig: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_12_CustomConfig>,
  pub r#hugePages: bool,
  pub r#boostUclamp: bool,
  pub r#hostServices: Vec<String>,
  pub r#shouldDelayEncryptedStoreSetup: bool,
  pub r#encryptedStoreKEK: Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>,
  pub r#autoStartAdbd: bool,
  pub r#ramdump: bool,
}
impl Default for r#VirtualMachineAppConfig {
  fn default() -> Self {
    Self {
      r#name: Default::default(),
      r#instanceId: [Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default()],
      r#apk: Default::default(),
      r#idsig: Default::default(),
      r#extraIdsigs: Default::default(),
      r#tenantApks: Default::default(),
      r#tenantIdsigs: Default::default(),
      r#instanceImage: Default::default(),
      r#encryptedStorageImage: Default::default(),
      r#payload: Default::default(),
      r#osName: "microdroid".into(),
      r#debugLevel: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_10_DebugLevel::NONE,
      r#protectedVm: false,
      r#memoryMib: 0,
      r#cpuOptions: Default::default(),
      r#customConfig: Default::default(),
      r#hugePages: false,
      r#boostUclamp: false,
      r#hostServices: Default::default(),
      r#shouldDelayEncryptedStoreSetup: false,
      r#encryptedStoreKEK: Default::default(),
      r#autoStartAdbd: false,
      r#ramdump: true,
    }
  }
}
impl binder::Parcelable for r#VirtualMachineAppConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#name)?;
      subparcel.write(&self.r#instanceId)?;
      let __field_ref = self.r#apk.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
      subparcel.write(__field_ref)?;
      let __field_ref = self.r#idsig.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
      subparcel.write(__field_ref)?;
      subparcel.write(&self.r#extraIdsigs)?;
      subparcel.write(&self.r#tenantApks)?;
      subparcel.write(&self.r#tenantIdsigs)?;
      let __field_ref = self.r#instanceImage.as_ref().ok_or(binder::StatusCode::UNEXPECTED_NULL)?;
      subparcel.write(__field_ref)?;
      subparcel.write(&self.r#encryptedStorageImage)?;
      subparcel.write(&self.r#payload)?;
      subparcel.write(&self.r#osName)?;
      subparcel.write(&self.r#debugLevel)?;
      subparcel.write(&self.r#protectedVm)?;
      subparcel.write(&self.r#memoryMib)?;
      subparcel.write(&self.r#cpuOptions)?;
      subparcel.write(&self.r#customConfig)?;
      subparcel.write(&self.r#hugePages)?;
      subparcel.write(&self.r#boostUclamp)?;
      subparcel.write(&self.r#hostServices)?;
      subparcel.write(&self.r#shouldDelayEncryptedStoreSetup)?;
      subparcel.write(&self.r#encryptedStoreKEK)?;
      subparcel.write(&self.r#autoStartAdbd)?;
      subparcel.write(&self.r#ramdump)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#name = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#instanceId = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#apk = Some(subparcel.read()?);
      }
      if subparcel.has_more_data() {
        self.r#idsig = Some(subparcel.read()?);
      }
      if subparcel.has_more_data() {
        self.r#extraIdsigs = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#tenantApks = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#tenantIdsigs = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#instanceImage = Some(subparcel.read()?);
      }
      if subparcel.has_more_data() {
        self.r#encryptedStorageImage = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#payload = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#osName = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#debugLevel = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#protectedVm = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#memoryMib = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#cpuOptions = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#customConfig = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hugePages = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#boostUclamp = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hostServices = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#shouldDelayEncryptedStoreSetup = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#encryptedStoreKEK = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#autoStartAdbd = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#ramdump = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachineAppConfig);
binder::impl_deserialize_for_parcelable!(r#VirtualMachineAppConfig);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachineAppConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineAppConfig" }
}
pub mod r#Payload {
  #[derive(Debug)]
  pub enum r#Payload {
    ConfigPath(String),
    PayloadConfig(crate::mangled::_7_android_6_system_21_virtualizationservice_27_VirtualMachinePayloadConfig),
  }
  impl Default for r#Payload {
    fn default() -> Self {
      Self::ConfigPath(Default::default())
    }
  }
  impl binder::Parcelable for r#Payload {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      match self {
        Self::ConfigPath(v) => {
          parcel.write(&0i32)?;
          parcel.write(v)
        }
        Self::PayloadConfig(v) => {
          parcel.write(&1i32)?;
          parcel.write(v)
        }
      }
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      let tag: i32 = parcel.read()?;
      match tag {
        0 => {
          let value: String = parcel.read()?;
          *self = Self::ConfigPath(value);
          Ok(())
        }
        1 => {
          let value: crate::mangled::_7_android_6_system_21_virtualizationservice_27_VirtualMachinePayloadConfig = parcel.read()?;
          *self = Self::PayloadConfig(value);
          Ok(())
        }
        _ => {
          Err(binder::StatusCode::BAD_VALUE)
        }
      }
    }
  }
  binder::impl_serialize_for_parcelable!(r#Payload);
  binder::impl_deserialize_for_parcelable!(r#Payload);
  impl binder::binder_impl::ParcelableMetadata for r#Payload {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineAppConfig.Payload" }
  }
  pub mod r#Tag {
    #![allow(non_upper_case_globals)]
    use binder::declare_binder_enum;
    declare_binder_enum! {
      #[repr(C, align(4))]
      r#Tag : [i32; 2] {
        r#configPath = 0,
        r#payloadConfig = 1,
      }
    }
  }
}
pub mod r#DebugLevel {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(C, align(1))]
    r#DebugLevel : [i8; 2] {
      r#NONE = 0,
      r#FULL = 1,
    }
  }
}
pub mod r#CustomConfig {
  #[derive(Debug)]
  pub struct r#CustomConfig {
    pub r#customKernelImage: Option<binder::ParcelFileDescriptor>,
    pub r#gdbPort: i32,
    pub r#vendorImage: Option<binder::ParcelFileDescriptor>,
    pub r#devices: Vec<String>,
    pub r#wantUpdatable: bool,
    pub r#networkSupported: bool,
    pub r#extraKernelCmdlineParams: Vec<String>,
    pub r#teeServices: Vec<String>,
  }
  impl Default for r#CustomConfig {
    fn default() -> Self {
      Self {
        r#customKernelImage: Default::default(),
        r#gdbPort: 0,
        r#vendorImage: Default::default(),
        r#devices: Default::default(),
        r#wantUpdatable: true,
        r#networkSupported: false,
        r#extraKernelCmdlineParams: Default::default(),
        r#teeServices: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#CustomConfig {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#customKernelImage)?;
        subparcel.write(&self.r#gdbPort)?;
        subparcel.write(&self.r#vendorImage)?;
        subparcel.write(&self.r#devices)?;
        subparcel.write(&self.r#wantUpdatable)?;
        subparcel.write(&self.r#networkSupported)?;
        subparcel.write(&self.r#extraKernelCmdlineParams)?;
        subparcel.write(&self.r#teeServices)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#customKernelImage = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#gdbPort = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#vendorImage = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#devices = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#wantUpdatable = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#networkSupported = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#extraKernelCmdlineParams = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#teeServices = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#CustomConfig);
  binder::impl_deserialize_for_parcelable!(r#CustomConfig);
  impl binder::binder_impl::ParcelableMetadata for r#CustomConfig {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineAppConfig.CustomConfig" }
  }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineAppConfig as _7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig;
 pub use super::r#Payload::r#Payload as _7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_7_Payload;
 pub use super::r#Payload::r#Tag::r#Tag as _7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_7_Payload_3_Tag;
 pub use super::r#DebugLevel::r#DebugLevel as _7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_10_DebugLevel;
 pub use super::r#CustomConfig::r#CustomConfig as _7_android_6_system_21_virtualizationservice_23_VirtualMachineAppConfig_12_CustomConfig;
}
