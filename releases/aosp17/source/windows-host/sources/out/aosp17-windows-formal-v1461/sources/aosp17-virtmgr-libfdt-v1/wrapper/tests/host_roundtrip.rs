use libfdt::Fdt;
#[repr(align(8))] struct Buf([u8;4096]);
#[test] fn mutable_node_roundtrip() {
 let mut b=Buf([0;4096]); let f=Fdt::create_empty_tree(&mut b.0).unwrap();
 f.root_mut().add_subnode(c"chosen").unwrap().setprop(c"test-value",&118u32.to_be_bytes()).unwrap();
 assert_eq!(f.chosen().unwrap().unwrap().getprop_u32(c"test-value").unwrap(),Some(118));
 f.chosen_mut().unwrap().unwrap().setprop(c"test-value",&119u32.to_be_bytes()).unwrap();
 f.pack().unwrap();
 let parsed=Fdt::from_slice(f.as_slice()).unwrap();
 assert_eq!(parsed.chosen().unwrap().unwrap().getprop_u32(c"test-value").unwrap(),Some(119));
}
#[test] fn invalid_header_rejected() {let b=Buf([0;4096]);assert!(Fdt::from_slice(&b.0).is_err());}
#[test] fn too_small_buffer_rejected() {let mut b=Buf([0;4096]);assert!(Fdt::create_empty_tree(&mut b.0[..8]).is_err());}
