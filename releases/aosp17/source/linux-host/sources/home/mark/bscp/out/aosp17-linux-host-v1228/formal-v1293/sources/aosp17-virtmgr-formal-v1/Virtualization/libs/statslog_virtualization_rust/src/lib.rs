pub mod vm_creation_requested {
    #[derive(Clone, Copy, Debug)]
    pub enum ConfigType {
        VirtualMachineAppConfig,
        VirtualMachineRawConfig,
    }
}
