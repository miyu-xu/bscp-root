use libc::{c_char, c_int};

pub unsafe fn freecon(_con: *mut c_char) {}

pub unsafe fn fgetfilecon(_fd: c_int, _con: *mut *mut c_char) -> c_int {
    -1
}
