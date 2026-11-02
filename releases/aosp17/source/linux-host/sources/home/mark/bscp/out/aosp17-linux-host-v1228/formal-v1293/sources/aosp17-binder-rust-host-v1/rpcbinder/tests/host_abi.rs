use rpcbinder::{RpcSession, FileDescriptorTransportMode};
#[test]
fn native_session_create_configure_drop(){
 for _ in 0..32 {
  let session=RpcSession::new();
  session.set_file_descriptor_transport_mode(FileDescriptorTransportMode::Unix);
  session.set_max_incoming_threads(1);
  session.set_max_outgoing_connections(1);
  drop(session);
 }
}
