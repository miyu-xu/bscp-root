/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug)]
pub struct r#GpuConfig {
  pub r#backend: Option<String>,
  pub r#contextTypes: Option<Vec<Option<String>>>,
  pub r#pciAddress: Option<String>,
  pub r#rendererFeatures: Option<String>,
  pub r#rendererUseEgl: bool,
  pub r#rendererUseGles: bool,
  pub r#rendererUseGlx: bool,
  pub r#rendererUseSurfaceless: bool,
  pub r#rendererUseVulkan: bool,
}
impl Default for r#GpuConfig {
  fn default() -> Self {
    Self {
      r#backend: Default::default(),
      r#contextTypes: Default::default(),
      r#pciAddress: Default::default(),
      r#rendererFeatures: Default::default(),
      r#rendererUseEgl: false,
      r#rendererUseGles: false,
      r#rendererUseGlx: false,
      r#rendererUseSurfaceless: false,
      r#rendererUseVulkan: false,
    }
  }
}
impl binder::Parcelable for r#GpuConfig {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#backend)?;
      subparcel.write(&self.r#contextTypes)?;
      subparcel.write(&self.r#pciAddress)?;
      subparcel.write(&self.r#rendererFeatures)?;
      subparcel.write(&self.r#rendererUseEgl)?;
      subparcel.write(&self.r#rendererUseGles)?;
      subparcel.write(&self.r#rendererUseGlx)?;
      subparcel.write(&self.r#rendererUseSurfaceless)?;
      subparcel.write(&self.r#rendererUseVulkan)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#backend = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#contextTypes = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#pciAddress = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererFeatures = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererUseEgl = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererUseGles = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererUseGlx = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererUseSurfaceless = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#rendererUseVulkan = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#GpuConfig);
binder::impl_deserialize_for_parcelable!(r#GpuConfig);
impl binder::binder_impl::ParcelableMetadata for r#GpuConfig {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.GpuConfig" }
}
pub(crate) mod mangled {
 pub use super::r#GpuConfig as _7_android_6_system_21_virtualizationservice_9_GpuConfig;
}
