// Copyright 2021, The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Struct for VM configuration with JSON (de)serialization and AIDL parcelables

use android_system_virtualizationservice::{
    aidl::android::system::virtualizationservice::{
        AssignedDevices::AssignedDevices, CpuOptions::CpuOptions,
        CpuOptions::CpuTopology::CpuTopology, DiskImage::DiskImage as AidlDiskImage,
        Partition::Partition as AidlPartition, UsbConfig::UsbConfig as AidlUsbConfig,
        VirtualMachineAppConfig::DebugLevel::DebugLevel,
        VirtualMachineConfig::VirtualMachineConfig,
        VirtualMachineRawConfig::VirtualMachineRawConfig,
    },
    binder::ParcelFileDescriptor,
};

use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const INSTANCE_ID_SIZE: usize = 64;

/// Configuration for a particular VM to be started.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VmConfig {
    /// The name of VM.
    pub name: Option<String>,
    /// The filename of the kernel image, if any.
    pub kernel: Option<PathBuf>,
    /// The filename of the initial ramdisk for the kernel, if any.
    pub initrd: Option<PathBuf>,
    /// Parameters to pass to the kernel. As far as the VMM and boot protocol are concerned this is
    /// just a string, but typically it will contain multiple parameters separated by spaces.
    pub params: Option<String>,
    /// The bootloader to use. If this is supplied then the kernel and initrd must not be supplied;
    /// the bootloader is instead responsibly for loading the kernel from one of the disks.
    pub bootloader: Option<PathBuf>,
    /// The pflash images to use.
    #[serde(default)]
    pub pflash: Vec<PathBuf>,
    /// Disk images to be made available to the VM.
    #[serde(default)]
    pub disks: Vec<DiskImage>,
    /// Whether the VM should be a protected VM.
    #[serde(default)]
    pub protected: bool,
    /// The amount of RAM to give the VM, in MiB.
    #[serde(default)]
    pub memory_mib: Option<NonZeroU32>,
    /// The CPU topology: either "one_cpu"(default) or "match_host"
    pub cpu_topology: Option<String>,
    /// Version or range of versions of the virtual platform that this config is compatible with.
    /// The format follows SemVer (https://semver.org).
    pub platform_version: VersionReq,
    /// SysFS paths of devices assigned to the VM.
    #[serde(default)]
    pub devices: Vec<PathBuf>,
    /// The serial device for VM console input.
    pub console_input_device: Option<String>,
    /// The USB config of the VM.
    pub usb_config: Option<UsbConfig>,
    /// VM Instance ID
    pub instance_id: Option<PathBuf>,
    /// SMBIOS configuration
    pub smbios: Option<SmbiosConfig>,
    /// Explicit desktop Microdroid GPU mode, mapped to the upstream AIDL GPU contract.
    #[serde(default)]
    pub gpu: Option<DesktopGpuConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopGpuConfig { pub enabled: bool }
impl DesktopGpuConfig {
    fn to_parcelable(&self) -> Option<android_system_virtualizationservice::aidl::android::system::virtualizationservice::GpuConfig::GpuConfig> {
        self.enabled.then(|| android_system_virtualizationservice::aidl::android::system::virtualizationservice::GpuConfig::GpuConfig {
            backend: Some("gfxstream".into()),
            contextTypes: Some(vec![Some("gfxstream-vulkan".into())]),
            rendererUseVulkan: true,
            rendererUseSurfaceless: true,
            ..Default::default()
        })
    }
}

impl VmConfig {
    /// Ensure that the configuration has a valid combination of fields set, or return an error if
    /// not.
    pub fn validate(&self) -> Result<(), Error> {
        if self.bootloader.is_none() && self.kernel.is_none() {
            bail!("VM must have either a bootloader or a kernel image.");
        }
        if self.bootloader.is_some() && (self.kernel.is_some() || self.initrd.is_some()) {
            bail!("Can't have both bootloader and kernel/initrd image.");
        }
        for disk in &self.disks {
            if disk.image.is_none() == disk.partitions.is_empty() {
                bail!("Exactly one of image and partitions must be specified. (Was {:?}.)", disk);
            }
        }
        Ok(())
    }

    /// Load the configuration for a VM from the given JSON file, and check that it is valid.
    pub fn load(file: &File) -> Result<VmConfig, Error> {
        let buffered = BufReader::new(file);
        let config: VmConfig = serde_json::from_reader(buffered)?;
        config.validate()?;
        Ok(config)
    }

    /// Convert the `VmConfig` to a [`VirtualMachineConfig`] which can be passed to the Virt
    /// Manager.
    pub fn to_parcelable(&self) -> Result<VirtualMachineRawConfig, Error> {
        let memory_mib = if let Some(memory_mib) = self.memory_mib {
            memory_mib.get().try_into().context("Invalid memory_mib")?
        } else {
            0
        };
        let cpu_topology = match self.cpu_topology.as_deref() {
            None => CpuTopology::CpuCount(1),
            Some("one_cpu") => CpuTopology::CpuCount(1),
            Some("match_host") => CpuTopology::MatchHost(true),
            Some(cpu_topology) => bail!("Invalid cpu topology {}", cpu_topology),
        };
        let cpu_options = CpuOptions { cpuTopology: cpu_topology };
        let usb_config = self.usb_config.clone().map(|x| x.to_parcelable()).transpose()?;
        let instance_id = if let Some(path) = self.instance_id.as_ref() {
            load_instance_id(path)?
        } else {
            [0u8; INSTANCE_ID_SIZE]
        };
        Ok(VirtualMachineRawConfig {
            kernel: maybe_open_parcel_file(self.kernel.as_ref(), false)?,
            initrd: maybe_open_parcel_file(self.initrd.as_ref(), false)?,
            params: self.params.clone(),
            bootloader: maybe_open_parcel_file(self.bootloader.as_ref(), false)?,
            pflash: self
                .pflash
                .iter()
                .map(|p| open_parcel_file(p, true))
                .collect::<Result<Vec<_>, Error>>()?,
            disks: self.disks.iter().map(DiskImage::to_parcelable).collect::<Result<_, Error>>()?,
            protectedVm: self.protected,
            memoryMib: memory_mib,
            cpuOptions: cpu_options,
            platformVersion: self.platform_version.to_string(),
            devices: AssignedDevices::Devices(
                self.devices
                    .iter()
                    .map(|x| {
                        x.to_str()
                            .map(String::from)
                            .ok_or(anyhow!("Failed to convert {x:?} to String"))
                    })
                    .collect::<Result<_>>()?,
            ),
            consoleInputDevice: self.console_input_device.clone(),
            usbConfig: usb_config,
            balloon: true,
            instanceId: instance_id,
            smbiosOptions: self.smbios.as_ref().map(|x| x.to_parcelable()),
            gpuConfig: self.gpu.as_ref().and_then(DesktopGpuConfig::to_parcelable),
            ..Default::default()
        })
    }
}

fn load_instance_id(path: &Path) -> anyhow::Result<[u8; INSTANCE_ID_SIZE]> {
    let mut file =
        File::open(path).with_context(|| format!("open VM Instance ID file: {:?}", path))?;

    let metadata = file
        .metadata()
        .with_context(|| format!("get metadata for VM Instance ID file: {:?}", path))?;

    ensure!(
        metadata.len() == INSTANCE_ID_SIZE as u64,
        "VM Instance ID file {:?} has incorrect size. Expected {}, Got {}",
        path,
        INSTANCE_ID_SIZE,
        metadata.len()
    );

    let mut buffer = [0u8; INSTANCE_ID_SIZE];
    file.read_exact(&mut buffer)
        .with_context(|| format!("read VM Instance ID file: {:?}", path))?;
    Ok(buffer)
}

/// Returns the debug level of the VM from its configuration.
pub fn get_debug_level(config: &VirtualMachineConfig) -> Option<DebugLevel> {
    match config {
        VirtualMachineConfig::AppConfig(config) => Some(config.debugLevel),
        VirtualMachineConfig::RawConfig(_) => None,
    }
}

/// A disk image to be made available to the VM.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiskImage {
    /// The filename of the disk image, if it already exists. Exactly one of this and `partitions`
    /// must be specified.
    #[serde(default)]
    pub image: Option<PathBuf>,
    /// A set of partitions to be assembled into a composite image.
    #[serde(default)]
    pub partitions: Vec<Partition>,
    /// Whether this disk should be writable by the VM.
    pub writable: bool,
    /// Identifier for the disk device.
    pub id: Option<String>,
}

impl DiskImage {
    fn to_parcelable(&self) -> Result<AidlDiskImage, Error> {
        let partitions =
            self.partitions.iter().map(Partition::to_parcelable).collect::<Result<_>>()?;
        Ok(AidlDiskImage {
            image: maybe_open_parcel_file(self.image.as_ref(), self.writable)?,
            writable: self.writable,
            partitions,
            id: self.id.clone(),
        })
    }
}

/// A partition to be assembled into a composite image.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Partition {
    /// A label for the partition.
    pub label: String,
    /// The filename of the partition image.
    pub path: PathBuf,
    /// Whether the partition should be writable.
    #[serde(default)]
    pub writable: bool,
    /// GUID of this partition.
    #[serde(default)]
    pub guid: Option<Uuid>,
}

impl Partition {
    fn to_parcelable(&self) -> Result<AidlPartition> {
        Ok(AidlPartition {
            image: Some(open_parcel_file(&self.path, self.writable)?),
            writable: self.writable,
            label: self.label.to_owned(),
            guid: None,
        })
    }
}

/// USB controller and available USB devices
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsbConfig {
    /// Enable USB controller
    pub controller: bool,
}

impl UsbConfig {
    fn to_parcelable(&self) -> Result<AidlUsbConfig> {
        Ok(AidlUsbConfig { controller: self.controller })
    }
}

/// SMBIOS configuration
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SmbiosConfig {
    /// BIOS vendor name.
    pub bios_vendor: Option<String>,
    /// BIOS version number (free-form string).
    pub bios_version: Option<String>,
    /// System manufacturer name.
    pub manufacturer: Option<String>,
    /// System product name.
    pub product_name: Option<String>,
    /// System serial number.
    pub serial_number: Option<String>,
    /// System UUID.
    pub uuid: Option<String>,
    /// Free-form OEM strings (SMBIOS type 11).
    #[serde(default)]
    pub oem_strings: Vec<String>,
}

impl SmbiosConfig {
    fn to_parcelable(
        &self,
    ) -> android_system_virtualizationservice::aidl::android::system::virtualizationservice::SmbiosOptions::SmbiosOptions
    {
        android_system_virtualizationservice::aidl::android::system::virtualizationservice::SmbiosOptions::SmbiosOptions {
            biosVendor: self.bios_vendor.clone(),
            biosVersion: self.bios_version.clone(),
            manufacturer: self.manufacturer.clone(),
            productName: self.product_name.clone(),
            serialNumber: self.serial_number.clone(),
            uuid: self.uuid.clone(),
            oemStrings: self.oem_strings.clone(),
        }
    }
}

/// Try to open the given file and wrap it in a [`ParcelFileDescriptor`].
pub fn open_parcel_file(filename: &Path, writable: bool) -> Result<ParcelFileDescriptor> {
    let filename = resolve_host_path(filename)?;
    Ok(ParcelFileDescriptor::new(
        OpenOptions::new()
            .read(true)
            .write(writable)
            .open(&filename)
            .with_context(|| format!("Failed to open {filename:?}"))?,
    ))
}

/// If the given filename is `Some`, try to open it and wrap it in a [`ParcelFileDescriptor`].
fn maybe_open_parcel_file(
    filename: Option<&PathBuf>,
    writable: bool,
) -> Result<Option<ParcelFileDescriptor>> {
    filename.map(|filename| open_parcel_file(filename, writable)).transpose()
}

/// Desktop artifact roots use the same namespace-root contract as the launchers.
/// Android retains its own namespace. No fallback to unrelated host artifacts.
pub fn resolve_host_path(path: &Path) -> Result<PathBuf> {
    #[cfg(target_os="android")]
    { Ok(path.to_owned()) }
    #[cfg(not(target_os="android"))]
    { map_host_path(path, |key| std::env::var_os(key).map(PathBuf::from)) }
}
#[cfg(not(target_os="android"))]
fn map_host_path(path:&Path, roots:impl Fn(&str)->Option<PathBuf>) -> Result<PathBuf> {
    if path.components().any(|c|matches!(c,std::path::Component::ParentDir)) {
        anyhow::bail!("Parent traversal in artifact path");
    }
    for (prefix,key) in [("/apex","VIRTMGR_APEX_ROOT"),("/system_ext","VIRTMGR_SYSTEM_EXT_ROOT"),("/system","VIRTMGR_SYSTEM_ROOT")] {
        if let Ok(rest)=path.strip_prefix(prefix) {
            if let Some(root)=roots(key) {
                if !root.is_absolute(){anyhow::bail!("{key} must be absolute");}
                return Ok(root.join(rest));
            }
        }
    }
    Ok(path.to_owned())
}
#[cfg(all(test,not(target_os="android")))]
mod desktop_path_tests {
 use super::*;
 #[test] fn exact_module_and_sibling_mapping(){
  let roots=|k:&str|if k=="VIRTMGR_APEX_ROOT"{Some(PathBuf::from("/product/apex"))}else{None};
  assert_eq!(map_host_path(Path::new("/apex/com.android.virt/etc/x"),roots).unwrap(),PathBuf::from("/product/apex/com.android.virt/etc/x"));
  assert_eq!(map_host_path(Path::new("/apex/apex-info-list.xml"),roots).unwrap(),PathBuf::from("/product/apex/apex-info-list.xml"));
 }
 #[test] fn bad_root_and_traversal_fail(){
  assert!(map_host_path(Path::new("/apex/com.android.virt/x"), |_|Some(PathBuf::from("relative"))).is_err());
  assert!(map_host_path(Path::new("/apex/com.android.virt/../x"), |_|None).is_err());
 }
 #[test] fn component_prefix_and_unmapped_paths(){
  let roots=|_:&str|Some(PathBuf::from("/mapped"));
  assert_eq!(map_host_path(Path::new("/systematic/x"),roots).unwrap(),PathBuf::from("/systematic/x"));
  assert_eq!(map_host_path(Path::new("/tmp/private"),roots).unwrap(),PathBuf::from("/tmp/private"));
 }
}

#[cfg(test)]
mod desktop_gpu_contract_tests {
    use super::*;
    fn config(gpu: &str) -> VmConfig {
        serde_json::from_str(&format!(r#"{{"kernel":"/dev/null","platform_version":"~1.0"{gpu}}}"#)).unwrap()
    }
    #[test] fn absent_and_off_have_no_gpu() {
        assert!(config("").to_parcelable().unwrap().gpuConfig.is_none());
        assert!(config(r#", "gpu":{"enabled":false}"#).to_parcelable().unwrap().gpuConfig.is_none());
    }
    #[test] fn enabled_maps_to_real_upstream_gpu_fields() {
        let g=config(r#", "gpu":{"enabled":true}"#).to_parcelable().unwrap().gpuConfig.unwrap();
        assert_eq!(g.backend.as_deref(),Some("gfxstream"));
        assert_eq!(g.contextTypes,Some(vec![Some("gfxstream-vulkan".into())]));
        assert!(g.rendererUseVulkan && g.rendererUseSurfaceless);
        assert!(!g.rendererUseEgl && !g.rendererUseGles && !g.rendererUseGlx);
    }
    #[test] fn invalid_gpu_contract_is_rejected() {
        for s in [r#"{}"#,r#"{"enabled":1}"#,r#"{"enabled":true,"unknown":1}"#] {
            assert!(serde_json::from_str::<DesktopGpuConfig>(s).is_err());
        }
    }
}
