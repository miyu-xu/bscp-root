use microdroid_payload_config::{VmPayloadConfig,ExpectedAuthority};
#[test] fn defaults_remain_unprivileged() {
 let c:VmPayloadConfig=serde_json::from_str("{}").unwrap();
 assert!(!c.run_as_root); assert!(!c.enable_authfs); assert!(c.task.is_none()); assert!(c.tenants.is_empty());
}
#[test] fn legacy_launcher_config_roundtrips() {
 let c:VmPayloadConfig=serde_json::from_str(r#"{"task":{"type":"microdroid_launcher","command":"payload.so"},"extra_apks":[],"apexes":[]}"#).unwrap();
 let encoded=serde_json::to_string(&c).unwrap(); assert_eq!(serde_json::from_str::<VmPayloadConfig>(&encoded).unwrap(),c);
 assert!(!c.run_as_root);
}
#[test] fn invalid_field_types_fail() {
 assert!(serde_json::from_str::<VmPayloadConfig>(r#"{"run_as_root":"false"}"#).is_err());
 assert!(serde_json::from_str::<VmPayloadConfig>(r#"{"task":{"type":"shell","command":"x"}}"#).is_err());
}
#[test] fn host_authority_is_release() {
 let a=ExpectedAuthority{dev_key:"dev".into(),test_key:"test".into(),release_key:"release".into()};
 assert_eq!(a.resolve_authority(),"release");
}
