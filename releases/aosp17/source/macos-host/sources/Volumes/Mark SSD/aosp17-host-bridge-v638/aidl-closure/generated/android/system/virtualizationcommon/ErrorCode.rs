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
  r#ErrorCode : [i32; 4] {
    r#UNKNOWN = 0,
    r#PAYLOAD_VERIFICATION_FAILED = 1,
    r#PAYLOAD_CHANGED = 2,
    r#PAYLOAD_INVALID_CONFIG = 3,
  }
}
pub(crate) mod mangled {
 pub use super::r#ErrorCode as _7_android_6_system_20_virtualizationcommon_9_ErrorCode;
}
