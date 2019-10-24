// Inspired by the log crate
use std::{
    ffi::CStr,
    sync::atomic::{AtomicUsize, Ordering},
};

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static STATE: AtomicUsize = AtomicUsize::new(UNINITIALIZED);
static mut LOGGER: &'static dyn Fn(&str) = &|_| {};

/// Set the function the native cubism core calls for logging.
/// Once set calling this function will do nothing.
pub fn set_core_logger<F>(logger: F)
where
    F: Fn(&str) + 'static,
{
    match STATE.compare_and_swap(UNINITIALIZED, INITIALIZING, Ordering::SeqCst) {
        UNINITIALIZED => unsafe {
            LOGGER = Box::leak(Box::new(logger));
            ffi::csmSetLogFunction(Some(core_logger));
            STATE.store(INITIALIZED, Ordering::SeqCst);
        },
        INITIALIZING => while STATE.load(Ordering::SeqCst) == INITIALIZING {},
        _ => (),
    }
}

unsafe extern "C" fn core_logger(message: *const std::os::raw::c_char) {
    if let Ok(s) = CStr::from_ptr(message).to_str() {
        (LOGGER)(s.trim_end())
    }
}
