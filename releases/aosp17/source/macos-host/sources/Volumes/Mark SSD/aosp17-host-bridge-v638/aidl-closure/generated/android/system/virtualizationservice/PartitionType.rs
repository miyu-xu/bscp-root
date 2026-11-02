/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(non_upper_case_globals)]
use binder::declare_binder_enum;
declare_binder_enum! {
  #[repr(C, align(4))]
  r#PartitionType : [i32; 3] {
    r#RAW = 0,
    r#ANDROID_VM_INSTANCE = 1,
    r#ENCRYPTEDSTORE = 2,
  }
}
pub(crate) mod mangled {
 pub use super::r#PartitionType as _7_android_6_system_21_virtualizationservice_13_PartitionType;
}
