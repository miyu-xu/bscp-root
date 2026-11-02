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
  r#Error : [i32; 12] {
    r#OK = 0,
    r#INVALID_PEER_NONCE = -1,
    r#INVALID_PEER_KE_KEY = -2,
    r#INVALID_IDENTITY = -3,
    r#INVALID_CERT_CHAIN = -4,
    r#INVALID_SIGNATURE = -5,
    r#INVALID_KE_KEY = -6,
    r#INVALID_PUB_KEY_IN_KEY = -7,
    r#INVALID_PRIV_KEY_ARC_IN_KEY = -8,
    r#INVALID_SHARED_KEY_ARCS = -9,
    r#MEMORY_ALLOCATION_FAILED = -10,
    r#INCOMPATIBLE_PROTOCOL_VERSION = -11,
  }
}
pub(crate) mod mangled {
 pub use super::r#Error as _7_android_8_hardware_8_security_9_authgraph_5_Error;
}
