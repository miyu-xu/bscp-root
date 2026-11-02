#![cfg(feature = "composite-disk")]
use protobuf::Message;
#[test] fn composite_disk_wire_roundtrip() {
 let mut disk=protos::cdisk_spec::CompositeDisk::new(); disk.version=2;disk.length=4096;
 let mut part=protos::cdisk_spec::ComponentDisk::new();part.file_path="/nonexistent/test-only".into();part.offset=512;disk.component_disks.push(part);
 let bytes=disk.write_to_bytes().unwrap();let decoded=protos::cdisk_spec::CompositeDisk::parse_from_bytes(&bytes).unwrap();assert_eq!(disk,decoded);
}
#[test] fn malformed_composite_disk_rejected() {
 assert!(protos::cdisk_spec::CompositeDisk::parse_from_bytes(&[0x12,0xff]).is_err());
}
