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
  r#VirtualMachineState : [i32; 6] {
    r#NOT_STARTED = 0,
    r#STARTING = 1,
    r#STARTED = 2,
    r#READY = 3,
    r#FINISHED = 4,
    r#DEAD = 6,
  }
}
pub(crate) mod mangled {
 pub use super::r#VirtualMachineState as _7_android_6_system_21_virtualizationservice_19_VirtualMachineState;
}
