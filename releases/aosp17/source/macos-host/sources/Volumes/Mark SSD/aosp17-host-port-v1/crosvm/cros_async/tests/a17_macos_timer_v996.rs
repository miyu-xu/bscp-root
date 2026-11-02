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
  let start=Instant::now();timer.wait().await.unwrap();assert!(start.elapsed()>=Duration::from_millis(10));
  timer.clear().unwrap();
  timer.reset_oneshot(Duration::from_secs(5)).unwrap();timer.clear().unwrap();
  let start=Instant::now();timer.reset_oneshot(delay).unwrap();timer.wait().await.unwrap();assert!(start.elapsed()>=delay);
  let mut timer2=Timer::new().unwrap();timer2.reset_oneshot(Duration::from_millis(60)).unwrap();
  let timer2=TimerAsync::new(timer2,&ex).unwrap();
  let start=Instant::now();TimerAsync::sleep(&ex,delay).await.unwrap();timer2.wait().await.unwrap();assert!(start.elapsed()>=Duration::from_millis(60));
 }).unwrap();
}
