extern crate alloc;
pub mod aidl {
pub mod android {
pub mod os {
#[path="@AIDL_ROOT@/android/os/IAccessor.rs"] pub mod IAccessor;
#[path="@AIDL_ROOT@/android/os/IRpcProvider.rs"] pub mod IRpcProvider;
}
pub mod system {
pub mod virtualizationcommon {
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/Atom.rs"] pub mod Atom;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/Certificate.rs"] pub mod Certificate;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/DeathReason.rs"] pub mod DeathReason;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/ErrorCode.rs"] pub mod ErrorCode;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/ICEStoreKEK.rs"] pub mod ICEStoreKEK;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/IEncryptedStoreKEK.rs"] pub mod IEncryptedStoreKEK;
#[path="@AIDL_ROOT@/android/system/virtualizationcommon/IGuestAgent.rs"] pub mod IGuestAgent;
}
pub mod virtualizationmaintenance {
#[path="@AIDL_ROOT@/android/system/virtualizationmaintenance/IVirtualizationMaintenance.rs"] pub mod IVirtualizationMaintenance;
#[path="@AIDL_ROOT@/android/system/virtualizationmaintenance/IVirtualizationReconciliationCallback.rs"] pub mod IVirtualizationReconciliationCallback;
}
pub mod virtualizationservice {
#[path="@AIDL_ROOT@/android/system/virtualizationservice/AssignableDevice.rs"] pub mod AssignableDevice;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/AssignedDevices.rs"] pub mod AssignedDevices;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/AudioConfig.rs"] pub mod AudioConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/CpuOptions.rs"] pub mod CpuOptions;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/CustomMemoryBackingFile.rs"] pub mod CustomMemoryBackingFile;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/DiskImage.rs"] pub mod DiskImage;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/DisplayConfig.rs"] pub mod DisplayConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/GpuConfig.rs"] pub mod GpuConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/InputDevice.rs"] pub mod InputDevice;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/IVirtualizationService.rs"] pub mod IVirtualizationService;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/IVirtualMachine.rs"] pub mod IVirtualMachine;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/IVirtualMachineCallback.rs"] pub mod IVirtualMachineCallback;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/Partition.rs"] pub mod Partition;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/PartitionType.rs"] pub mod PartitionType;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/SharedPath.rs"] pub mod SharedPath;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/SmbiosOptions.rs"] pub mod SmbiosOptions;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/UsbConfig.rs"] pub mod UsbConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineAppConfig.rs"] pub mod VirtualMachineAppConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineConfig.rs"] pub mod VirtualMachineConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineDebugInfo.rs"] pub mod VirtualMachineDebugInfo;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineDisplay.rs"] pub mod VirtualMachineDisplay;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachinePayloadConfig.rs"] pub mod VirtualMachinePayloadConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineRawConfig.rs"] pub mod VirtualMachineRawConfig;
#[path="@AIDL_ROOT@/android/system/virtualizationservice/VirtualMachineState.rs"] pub mod VirtualMachineState;
}
pub mod virtualizationservice_internal {
#[path="@AIDL_ROOT@/android/system/virtualizationservice_internal/IBoundDevice.rs"] pub mod IBoundDevice;
#[path="@AIDL_ROOT@/android/system/virtualizationservice_internal/IGlobalVmContext.rs"] pub mod IGlobalVmContext;
#[path="@AIDL_ROOT@/android/system/virtualizationservice_internal/IVfioHandler.rs"] pub mod IVfioHandler;
#[path="@AIDL_ROOT@/android/system/virtualizationservice_internal/IVirtualizationServiceInternal.rs"] pub mod IVirtualizationServiceInternal;
#[path="@AIDL_ROOT@/android/system/virtualizationservice_internal/IVmnic.rs"] pub mod IVmnic;
}
pub mod virtualmachineservice {
#[path="@AIDL_ROOT@/android/system/virtualmachineservice/IVirtualMachineService.rs"] pub mod IVirtualMachineService;
}
pub mod vmtethering {
#[path="@AIDL_ROOT@/android/system/vmtethering/IVmTethering.rs"] pub mod IVmTethering;
}
}
pub mod hardware {
pub mod security {
pub mod authgraph {
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/Arc.rs"] pub mod Arc;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/Error.rs"] pub mod Error;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/IAuthGraphKeyExchange.rs"] pub mod IAuthGraphKeyExchange;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/Identity.rs"] pub mod Identity;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/KeInitResult.rs"] pub mod KeInitResult;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/Key.rs"] pub mod Key;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/PlainPubKey.rs"] pub mod PlainPubKey;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/PubKey.rs"] pub mod PubKey;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/SessionIdSignature.rs"] pub mod SessionIdSignature;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/SessionInfo.rs"] pub mod SessionInfo;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/SessionInitiationInfo.rs"] pub mod SessionInitiationInfo;
#[path="@AIDL_ROOT@/android/hardware/security/authgraph/SignedPubKey.rs"] pub mod SignedPubKey;
}
pub mod secretkeeper {
#[path="@AIDL_ROOT@/android/hardware/security/secretkeeper/ISecretkeeper.rs"] pub mod ISecretkeeper;
#[path="@AIDL_ROOT@/android/hardware/security/secretkeeper/PublicKey.rs"] pub mod PublicKey;
#[path="@AIDL_ROOT@/android/hardware/security/secretkeeper/SecretId.rs"] pub mod SecretId;
}
}
}
}
}
pub(crate) mod mangled {
pub use crate::aidl::android::os::IAccessor::mangled::*;
pub use crate::aidl::android::os::IRpcProvider::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::Atom::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::Certificate::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::DeathReason::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::ErrorCode::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::ICEStoreKEK::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::IEncryptedStoreKEK::mangled::*;
pub use crate::aidl::android::system::virtualizationcommon::IGuestAgent::mangled::*;
pub use crate::aidl::android::system::virtualizationmaintenance::IVirtualizationMaintenance::mangled::*;
pub use crate::aidl::android::system::virtualizationmaintenance::IVirtualizationReconciliationCallback::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::AssignableDevice::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::AssignedDevices::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::AudioConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::CpuOptions::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::CustomMemoryBackingFile::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::DiskImage::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::DisplayConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::GpuConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::InputDevice::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::IVirtualizationService::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::IVirtualMachine::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::IVirtualMachineCallback::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::Partition::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::PartitionType::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::SharedPath::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::SmbiosOptions::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::UsbConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineAppConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineDebugInfo::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineDisplay::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachinePayloadConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineRawConfig::mangled::*;
pub use crate::aidl::android::system::virtualizationservice::VirtualMachineState::mangled::*;
pub use crate::aidl::android::system::virtualizationservice_internal::IBoundDevice::mangled::*;
pub use crate::aidl::android::system::virtualizationservice_internal::IGlobalVmContext::mangled::*;
pub use crate::aidl::android::system::virtualizationservice_internal::IVfioHandler::mangled::*;
pub use crate::aidl::android::system::virtualizationservice_internal::IVirtualizationServiceInternal::mangled::*;
pub use crate::aidl::android::system::virtualizationservice_internal::IVmnic::mangled::*;
pub use crate::aidl::android::system::virtualmachineservice::IVirtualMachineService::mangled::*;
pub use crate::aidl::android::system::vmtethering::IVmTethering::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::Arc::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::Error::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::IAuthGraphKeyExchange::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::Identity::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::KeInitResult::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::Key::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::PlainPubKey::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::PubKey::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::SessionIdSignature::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::SessionInfo::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::SessionInitiationInfo::mangled::*;
pub use crate::aidl::android::hardware::security::authgraph::SignedPubKey::mangled::*;
pub use crate::aidl::android::hardware::security::secretkeeper::ISecretkeeper::mangled::*;
pub use crate::aidl::android::hardware::security::secretkeeper::PublicKey::mangled::*;
pub use crate::aidl::android::hardware::security::secretkeeper::SecretId::mangled::*;
}

pub use binder;
