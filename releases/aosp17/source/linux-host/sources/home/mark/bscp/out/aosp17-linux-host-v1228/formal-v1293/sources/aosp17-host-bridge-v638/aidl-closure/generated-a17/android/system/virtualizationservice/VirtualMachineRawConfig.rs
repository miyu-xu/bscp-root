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
pub struct r#VirtualMachineRawConfig {
  pub r#name: alloc::string::String,
  pub r#instanceId: [u8; 64],
  pub r#osName: alloc::string::String,
  pub r#kernel: Option<binder::ParcelFileDescriptor>,
  pub r#initrd: Option<binder::ParcelFileDescriptor>,
  pub r#params: Option<alloc::string::String>,
  pub r#bootloader: Option<binder::ParcelFileDescriptor>,
  pub r#pflash: alloc::vec::Vec<binder::ParcelFileDescriptor>,
  pub r#disks: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_9_DiskImage>,
  pub r#sharedPaths: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_10_SharedPath>,
  pub r#protectedVm: bool,
  pub r#customPvmfw: Option<binder::ParcelFileDescriptor>,
  pub r#memoryMib: i32,
  pub r#swiotlbMib: i32,
  pub r#cpuOptions: crate::mangled::_7_android_6_system_21_virtualizationservice_10_CpuOptions,
  pub r#platformVersion: alloc::string::String,
  pub r#gdbPort: i32,
  pub r#hugePages: bool,
  pub r#devices: crate::mangled::_7_android_6_system_21_virtualizationservice_15_AssignedDevices,
  pub r#displayConfig: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig>,
  pub r#inputDevices: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_11_InputDevice>,
  pub r#networkSupported: bool,
  pub r#consoleInputDevice: Option<alloc::string::String>,
  pub r#boostUclamp: bool,
  pub r#gpuConfig: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_9_GpuConfig>,
  pub r#audioConfig: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_11_AudioConfig>,
  pub r#balloon: bool,
  pub r#allowVgicItsInPvm: bool,
  pub r#usbConfig: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_9_UsbConfig>,
  pub r#teeServices: alloc::vec::Vec<alloc::string::String>,
  pub r#enableHypervisorSpecificAuthMethod: bool,
  pub r#customMemoryBackingFiles: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_CustomMemoryBackingFile>,
  pub r#hostServices: alloc::vec::Vec<alloc::string::String>,
  pub r#smbiosOptions: Option<crate::mangled::_7_android_6_system_21_virtualizationservice_13_SmbiosOptions>,
  pub r#ramdump: bool,
}
impl Default for r#VirtualMachineRawConfig {
  fn default() -> Self {
    Self {
      r#name: Default::default(),
      r#instanceId: [Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default(), Default::default()],
      r#osName: Default::default(),
      r#kernel: Default::default(),
      r#initrd: Default::default(),
      r#params: Default::default(),
      r#bootloader: Default::default(),
      r#pflash: Default::default(),
      r#disks: Default::default(),
      r#sharedPaths: Default::default(),
      r#protectedVm: false,
      r#customPvmfw: Default::default(),
      r#memoryMib: 0,
      r#swiotlbMib: 0,
      r#cpuOptions: Default::default(),
      r#platformVersion: Default::default(),
      r#gdbPort: 0,
      r#hugePages: false,
      r#devices: Default::default(),
      r#displayConfig: Default::default(),
      r#inputDevices: Default::default(),
      r#networkSupported: false,
      r#consoleInputDevice: Default::default(),
      r#boostUclamp: false,
      r#gpuConfig: Default::default(),
      r#audioConfig: Default::default(),
      r#balloon: false,
      r#allowVgicItsInPvm: false,
      r#usbConfig: Default::default(),
      r#teeServices: Default::default(),
      r#enableHypervisorSpecificAuthMethod: false,
      r#customMemoryBackingFiles: Default::default(),
      r#hostServices: Default::default(),
      r#smbiosOptions: Default::default(),
      r#ramdump: true,
    }
  }
}
impl binder::Parcelable for r#VirtualMachineRawConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#name)?;
      subparcel.write(&self.r#instanceId)?;
      subparcel.write(&self.r#osName)?;
      subparcel.write(&self.r#kernel)?;
      subparcel.write(&self.r#initrd)?;
      subparcel.write(&self.r#params)?;
      subparcel.write(&self.r#bootloader)?;
      subparcel.write(&self.r#pflash)?;
      subparcel.write(&self.r#disks)?;
      subparcel.write(&self.r#sharedPaths)?;
      subparcel.write(&self.r#protectedVm)?;
      subparcel.write(&self.r#customPvmfw)?;
      subparcel.write(&self.r#memoryMib)?;
      subparcel.write(&self.r#swiotlbMib)?;
      subparcel.write(&self.r#cpuOptions)?;
      subparcel.write(&self.r#platformVersion)?;
      subparcel.write(&self.r#gdbPort)?;
      subparcel.write(&self.r#hugePages)?;
      subparcel.write(&self.r#devices)?;
      subparcel.write(&self.r#displayConfig)?;
      subparcel.write(&self.r#inputDevices)?;
      subparcel.write(&self.r#networkSupported)?;
      subparcel.write(&self.r#consoleInputDevice)?;
      subparcel.write(&self.r#boostUclamp)?;
      subparcel.write(&self.r#gpuConfig)?;
      subparcel.write(&self.r#audioConfig)?;
      subparcel.write(&self.r#balloon)?;
      subparcel.write(&self.r#allowVgicItsInPvm)?;
      subparcel.write(&self.r#usbConfig)?;
      subparcel.write(&self.r#teeServices)?;
      subparcel.write(&self.r#enableHypervisorSpecificAuthMethod)?;
      subparcel.write(&self.r#customMemoryBackingFiles)?;
      subparcel.write(&self.r#hostServices)?;
      subparcel.write(&self.r#smbiosOptions)?;
      subparcel.write(&self.r#ramdump)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#name = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#instanceId = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#osName = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#kernel = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#initrd = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#params = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#bootloader = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#pflash = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#disks = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#sharedPaths = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#protectedVm = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#customPvmfw = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#memoryMib = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#swiotlbMib = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#cpuOptions = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#platformVersion = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#gdbPort = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hugePages = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#devices = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#displayConfig = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#inputDevices = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#networkSupported = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#consoleInputDevice = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#boostUclamp = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#gpuConfig = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#audioConfig = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#balloon = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#allowVgicItsInPvm = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#usbConfig = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#teeServices = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#enableHypervisorSpecificAuthMethod = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#customMemoryBackingFiles = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hostServices = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#smbiosOptions = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#ramdump = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#VirtualMachineRawConfig);
binder::impl_deserialize_for_parcelable!(r#VirtualMachineRawConfig);
impl binder::binder_impl::ParcelableMetadata for r#VirtualMachineRawConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.VirtualMachineRawConfig" }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineRawConfig as _7_android_6_system_21_virtualizationservice_23_VirtualMachineRawConfig;
}
