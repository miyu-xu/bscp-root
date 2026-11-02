use a17_virtmgr_aidl_closure::aidl::android::system::{virtualizationcommon::DeathReason::DeathReason, virtualizationservice::CpuOptions::{CpuOptions,CpuTopology::CpuTopology}};
use binder::binder_impl::Parcel;
#[test] fn enum_layout_and_wire_roundtrip(){
 assert_eq!(std::mem::size_of::<DeathReason>(),4);
 assert_eq!(std::mem::align_of::<DeathReason>(),4);
 let mut p=Parcel::new();p.write(&DeathReason::HOST_SHUTDOWN).unwrap();
 assert_eq!(p.get_data_size(),4);unsafe{p.set_data_position(0).unwrap();}
 assert_eq!(p.read::<DeathReason>().unwrap(),DeathReason::HOST_SHUTDOWN);
}
#[test] fn cpu_options_roundtrip(){
 let mut p=Parcel::new();let input=CpuOptions{cpuTopology:CpuTopology::CpuCount(2)};
 p.write(&input).unwrap();unsafe{p.set_data_position(0).unwrap();}
 let output=p.read::<CpuOptions>().unwrap();assert_eq!(output.cpuTopology,CpuTopology::CpuCount(2));
}
