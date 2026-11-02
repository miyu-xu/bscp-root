#![cfg(target_os = "macos")]
use std::time::{Duration,Instant};
use base::{Timer,TimerTrait};
use cros_async::{Executor,TimerAsync};
#[test]
fn native_timer_async_deadlines_repeat_cancel_rearm() {
 std::thread::spawn(|| {std::thread::sleep(Duration::from_secs(20));std::process::exit(124);});
 let ex=Executor::new().unwrap();
 ex.run_until(async {
  let delay=Duration::from_millis(25);
  let start=Instant::now();TimerAsync::sleep(&ex,delay).await.unwrap();assert!(start.elapsed()>=delay);
  let mut timer=TimerAsync::new(Timer::new().unwrap(),&ex).unwrap();
  timer.reset_repeating(delay).unwrap();
  let start=Instant::now();timer.wait().await.unwrap();assert!(start.elapsed()>=delay);
  timer.wait().await.unwrap();assert!(start.elapsed()>=delay*2);
  timer.clear().unwrap();
  timer.reset_oneshot(Duration::from_secs(5)).unwrap();timer.clear().unwrap();
  let start=Instant::now();timer.reset_oneshot(delay).unwrap();timer.wait().await.unwrap();assert!(start.elapsed()>=delay);
  let mut timer2=Timer::new().unwrap();timer2.reset_oneshot(Duration::from_millis(60)).unwrap();
  let timer2=TimerAsync::new(timer2,&ex).unwrap();
  let start=Instant::now();TimerAsync::sleep(&ex,delay).await.unwrap();timer2.wait().await.unwrap();assert!(start.elapsed()>=Duration::from_millis(60));
 }).unwrap();
}

#[test]
fn pending_cancel_rearm_does_not_return_stale_expiration() {
 std::thread::spawn(|| {std::thread::sleep(Duration::from_secs(20));std::process::exit(124);});
 let ex=Executor::new().unwrap(); let mut timer=Timer::new().unwrap();
 timer.reset_oneshot(Duration::from_millis(1)).unwrap();
 std::thread::sleep(Duration::from_millis(10));timer.clear().unwrap();
 timer.reset_oneshot(Duration::from_millis(25)).unwrap();
 let timer=TimerAsync::new(timer,&ex).unwrap();let start=Instant::now();
 ex.run_until(timer.wait()).unwrap().unwrap();assert!(start.elapsed()>=Duration::from_millis(20));
}
#[test]
fn non_timer_event_is_error_not_expiration() {
 use base::AsRawDescriptor;
 std::thread::spawn(|| {std::thread::sleep(Duration::from_secs(20));std::process::exit(124);});
 let ex=Executor::new().unwrap(); let timer=Timer::new().unwrap();
 let event=libc::kevent64_s {ident:123,filter:libc::EVFILT_USER,flags:libc::EV_ADD|libc::EV_CLEAR,fflags:libc::NOTE_TRIGGER,data:0,udata:0,ext:[0,0]};
 let zero=libc::timespec {tv_sec:0,tv_nsec:0};
 assert_eq!(unsafe {libc::kevent64(timer.as_raw_descriptor(),&event,1,std::ptr::null_mut(),0,0,&zero)},0);
 let timer=TimerAsync::new(timer,&ex).unwrap();
 assert!(ex.run_until(timer.wait()).unwrap().is_err());
}
