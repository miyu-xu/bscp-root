use vmconfig::VmConfig;
fn parse(s:&str)->VmConfig { serde_json::from_str(s).unwrap() }
#[test] fn kernel_default_unprotected() {
 let c=parse(r#"{"kernel":"kernel","platform_version":"~1.0"}"#);
 assert!(c.validate().is_ok()); assert!(!c.protected); assert!(c.memory_mib.is_none());
}
#[test] fn boot_sources_are_exclusive() {
 assert!(parse(r#"{"platform_version":"~1.0"}"#).validate().is_err());
 assert!(parse(r#"{"kernel":"k","bootloader":"b","platform_version":"~1.0"}"#).validate().is_err());
 assert!(parse(r#"{"initrd":"i","bootloader":"b","platform_version":"~1.0"}"#).validate().is_err());
}
#[test] fn disk_shape_is_checked() {
 assert!(parse(r#"{"kernel":"k","platform_version":"~1.0","disks":[{"writable":false}]}"#).validate().is_err());
 assert!(parse(r#"{"kernel":"k","platform_version":"~1.0","disks":[{"writable":false,"image":"disk"}]}"#).validate().is_ok());
}
#[test] fn zero_memory_and_invalid_types_fail() {
 assert!(serde_json::from_str::<VmConfig>(r#"{"kernel":"k","platform_version":"~1.0","memory_mib":0}"#).is_err());
 assert!(serde_json::from_str::<VmConfig>(r#"{"kernel":"k","platform_version":"~1.0","protected":"false"}"#).is_err());
}
