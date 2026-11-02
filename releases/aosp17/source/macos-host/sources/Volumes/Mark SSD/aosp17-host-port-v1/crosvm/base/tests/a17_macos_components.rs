#![cfg(target_os = "macos")]
use base::{Event, EventWaitResult, Tube};
use std::time::Duration;

#[test]
fn event_timeout_signal_reset() {
    let e=Event::new().unwrap();
    assert_eq!(e.wait_timeout(Duration::from_millis(10)).unwrap(), EventWaitResult::TimedOut);
    let clone=e.try_clone().unwrap(); clone.signal().unwrap();
    assert_eq!(e.wait_timeout(Duration::from_secs(1)).unwrap(), EventWaitResult::Signaled);
    e.signal().unwrap(); e.reset().unwrap();
    assert_eq!(e.wait_timeout(Duration::from_millis(10)).unwrap(), EventWaitResult::TimedOut);
}

#[test]
fn framed_tube_roundtrip() {
    let (a,b)=Tube::pair().unwrap();
    a.set_send_timeout(Some(Duration::from_secs(1))).unwrap();
    b.set_recv_timeout(Some(Duration::from_secs(1))).unwrap();
    let reader=std::thread::spawn(move || {
        for n in [0, 1, 8192] {
            let got:Vec<u32>=b.recv().unwrap(); assert_eq!(vec![17u32;n],got);
        }
    });
    for n in [0, 1, 8192] { a.send(&vec![17u32;n]).unwrap(); }
    reader.join().unwrap();
}

#[test]
fn tube_event_descriptor_transfer() {
    let (a,b)=Tube::pair().unwrap();
    b.set_recv_timeout(Some(Duration::from_secs(1))).unwrap();
    let e=Event::new().unwrap(); a.send(&e).unwrap();
    let received:Event=b.recv().unwrap(); received.signal().unwrap();
    assert_eq!(e.wait_timeout(Duration::from_secs(1)).unwrap(), EventWaitResult::Signaled);
}

#[test]
fn event_repeated_immediate_reset() {
    let e=Event::new().unwrap();
    for _ in 0..5000 {
        e.signal().unwrap(); e.reset().unwrap();
        assert_eq!(e.wait_timeout(Duration::ZERO).unwrap(), EventWaitResult::TimedOut);
    }
}

#[test]
fn event_saturated_queue_coalesces_without_late_signal() {
    let e=Event::new().unwrap();
    for _ in 0..10000 { e.signal().unwrap(); }
    assert_eq!(e.wait_timeout(Duration::from_secs(1)).unwrap(), EventWaitResult::Signaled);
    assert_eq!(e.wait_timeout(Duration::ZERO).unwrap(), EventWaitResult::TimedOut);
}

#[test]
fn event_transfer_reset_keeps_one_descriptor_contract() {
    use base::{IntoRawDescriptor,FromRawDescriptor};
    let e=Event::new().unwrap();
    let raw=e.into_raw_descriptor();
    let e=unsafe { Event::from_raw_descriptor(raw) };
    let (a,b)=Tube::pair().unwrap();
    b.set_recv_timeout(Some(Duration::from_secs(1))).unwrap();
    a.send(&e).unwrap(); let remote:Event=b.recv().unwrap();
    for _ in 0..1000 {
        remote.signal().unwrap(); e.reset().unwrap();
        assert_eq!(e.wait_timeout(Duration::ZERO).unwrap(),EventWaitResult::TimedOut);
    }
}

#[test]
fn reset_finishes_with_continuous_producer() {
    use std::sync::{Arc,Barrier,atomic::{AtomicBool,Ordering}};
    use std::time::Instant;
    let e=Event::new().unwrap(); let writer=e.try_clone().unwrap();
    let done=Arc::new(AtomicBool::new(false)); let stop=done.clone();
    let barrier=Arc::new(Barrier::new(2)); let ready=barrier.clone();
    let worker=std::thread::spawn(move || {
        writer.signal().unwrap(); ready.wait(); let end=Instant::now()+Duration::from_secs(2);
        while !stop.load(Ordering::Relaxed) && Instant::now()<end {
            if let Err(err)=writer.signal() {
                assert!(matches!(err.errno(),libc::ENOBUFS|libc::EAGAIN));
            }
        }
    });
    barrier.wait(); let start=Instant::now(); let result=e.reset();
    let elapsed=start.elapsed(); done.store(true,Ordering::Relaxed); worker.join().unwrap();
    result.unwrap(); assert!(elapsed<Duration::from_secs(1),"reset: {:?}",elapsed);
    e.reset().unwrap(); assert_eq!(e.wait_timeout(Duration::ZERO).unwrap(),EventWaitResult::TimedOut);
}

#[test]
fn interrupted_wait_keeps_original_deadline() {
    use std::time::Instant;
    extern "C" fn ignore(_:libc::c_int) {}
    unsafe {
        let mut action:libc::sigaction=std::mem::zeroed();
        action.sa_sigaction=ignore as *const () as libc::sighandler_t;
        libc::sigemptyset(&mut action.sa_mask);
        let mut old:libc::sigaction=std::mem::zeroed();
        assert_eq!(libc::sigaction(libc::SIGUSR1,&action,&mut old),0);
        let target=libc::pthread_self();
        let sender=std::thread::spawn(move || {
            for _ in 0..8 { std::thread::sleep(Duration::from_millis(15)); assert_eq!(libc::pthread_kill(target,libc::SIGUSR1),0); }
        });
        let e=Event::new().unwrap(); let start=Instant::now();
        let result=e.wait_timeout(Duration::from_millis(50)); let elapsed=start.elapsed();
        sender.join().unwrap(); assert_eq!(libc::sigaction(libc::SIGUSR1,&old,std::ptr::null_mut()),0);
        assert_eq!(result.unwrap(),EventWaitResult::TimedOut);
        assert!(elapsed<Duration::from_millis(110),"wait: {:?}",elapsed);
    }
}

#[test]
fn fractional_millisecond_wait_is_not_truncated() {
    let e=Event::new().unwrap(); let duration=Duration::from_micros(1950);
    let start=std::time::Instant::now();
    assert_eq!(e.wait_timeout(duration).unwrap(),EventWaitResult::TimedOut);
    assert!(start.elapsed()>=duration);
}
