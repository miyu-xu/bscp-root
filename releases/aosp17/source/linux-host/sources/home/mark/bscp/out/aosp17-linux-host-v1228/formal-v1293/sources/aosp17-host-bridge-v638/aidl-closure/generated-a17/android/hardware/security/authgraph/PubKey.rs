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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum r#PubKey {
  PlainKey(crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_PlainPubKey),
  SignedKey(crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_SignedPubKey),
}
impl Default for r#PubKey {
  fn default() -> Self {
    Self::PlainKey(Default::default())
  }
}
impl binder::Parcelable for r#PubKey {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    match self {
      Self::PlainKey(v) => {
        parcel.write(&0i32)?;
        parcel.write(v)
      }
      Self::SignedKey(v) => {
        parcel.write(&1i32)?;
        parcel.write(v)
      }
    }
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    let tag: i32 = parcel.read()?;
    match tag {
      0 => {
        let value: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_PlainPubKey = parcel.read()?;
        *self = Self::PlainKey(value);
        Ok(())
      }
      1 => {
        let value: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_SignedPubKey = parcel.read()?;
        *self = Self::SignedKey(value);
        Ok(())
      }
      _ => {
        Err(binder::StatusCode::BAD_VALUE)
      }
    }
  }
}
binder::impl_serialize_for_parcelable!(r#PubKey);
binder::impl_deserialize_for_parcelable!(r#PubKey);
impl binder::binder_impl::ParcelableMetadata for r#PubKey {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.PubKey" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub mod r#Tag {
  #![allow(non_upper_case_globals)]
  use binder::declare_binder_enum;
  declare_binder_enum! {
    #[repr(align(4))]
    r#Tag : [i32; 2] {
      r#plainKey = 0,
      r#signedKey = 1,
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#PubKey as _7_android_8_hardware_8_security_9_authgraph_6_PubKey;
 pub use super::r#Tag::r#Tag as _7_android_8_hardware_8_security_9_authgraph_6_PubKey_3_Tag;
}
