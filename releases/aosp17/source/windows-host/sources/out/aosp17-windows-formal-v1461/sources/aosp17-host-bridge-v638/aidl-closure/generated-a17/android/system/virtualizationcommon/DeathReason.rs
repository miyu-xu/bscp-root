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
#![allow(non_upper_case_globals)]
use binder::declare_binder_enum;
declare_binder_enum! {
  #[repr(align(4))]
  r#DeathReason : [i32; 17] {
    r#INFRASTRUCTURE_ERROR = 0,
    r#KILLED = 1,
    r#UNKNOWN = 2,
    r#SHUTDOWN = 3,
    r#START_FAILED = 4,
    r#REBOOT = 5,
    r#CRASH = 6,
    r#PVM_FIRMWARE_PUBLIC_KEY_MISMATCH = 7,
    r#PVM_FIRMWARE_INSTANCE_IMAGE_CHANGED = 8,
    r#MICRODROID_FAILED_TO_CONNECT_TO_VIRTUALIZATION_SERVICE = 11,
    r#MICRODROID_PAYLOAD_HAS_CHANGED = 12,
    r#MICRODROID_PAYLOAD_VERIFICATION_FAILED = 13,
    r#MICRODROID_INVALID_PAYLOAD_CONFIG = 14,
    r#MICRODROID_UNKNOWN_RUNTIME_ERROR = 15,
    r#HANGUP = 16,
    r#WATCHDOG_REBOOT = 17,
    r#HOST_SHUTDOWN = 18,
  }
}
pub(crate) mod mangled {
 pub use super::r#DeathReason as _7_android_6_system_20_virtualizationcommon_11_DeathReason;
}
