//! Stub AIDL surface for `android.content.pm.IPackageManagerNative` (host port).

pub mod aidl {
    pub mod android {
        pub mod content {
            pub mod pm {
                pub mod IPackageManagerNative {
                    use binder::Result;
                    use super::StagedApexInfo;

                    pub trait IPackageManagerNative: binder::Interface {
                        fn getStagedApexModuleNames(&self) -> Result<Vec<String>>;
                        fn getStagedApexInfo(&self, _name: &str) -> Result<Option<StagedApexInfo>>;
                    }
                }

                #[derive(Clone, Debug, Default)]
                pub struct StagedApexInfo {
                    pub moduleName: String,
                    pub versionCode: i64,
                    pub diskImagePath: String,
                    pub hasClassPathJars: bool,
                }
            }
        }
    }
}
