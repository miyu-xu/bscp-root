//! Stub — host port; real checks use `check_permission` paths on Android.

pub mod aidl {
    pub mod android {
        pub mod os {
            pub mod IPermissionController {
                use binder::Interface;

                pub trait IPermissionController: Interface {
                    fn checkPermission(
                        &self,
                        _perm: &str,
                        _pid: i32,
                        _uid: i32,
                    ) -> binder::Result<bool>;
                }
            }
        }
    }
}
