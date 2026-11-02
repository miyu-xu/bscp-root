#![cfg(target_os = "macos")]
use base::Event;
fn count_open() -> usize { (0..256).filter(|fd| unsafe { libc::fcntl(*fd, libc::F_GETFD) } >= 0).count() }
#[test]
fn event_original_and_clone_drop_release_exact_descriptors() {
 let before=count_open();
 for _ in 0..128 {
  let events: Vec<_>=(0..64).map(|_| Event::new().unwrap()).collect();
  assert_eq!(count_open(),before+64);
  let clones: Vec<_>=events.iter().map(|e| e.try_clone().unwrap()).collect();
  assert_eq!(count_open(),before+128);
  drop(events); assert_eq!(count_open(),before+64);
  drop(clones); assert_eq!(count_open(),before);
 }
 let mut events=Vec::new();
 loop { match Event::new() { Ok(e)=>events.push(e), Err(e)=>{assert_eq!(e.errno(),libc::EMFILE);break;} } }
 assert!(!events.is_empty()); drop(events); assert_eq!(count_open(),before);
 let e=Event::new().unwrap(); drop(e); assert_eq!(count_open(),before);
}
