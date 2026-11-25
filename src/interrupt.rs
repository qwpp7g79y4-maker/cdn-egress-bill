//! SIGINT ends the process with exit 130 and one stderr line.
//!
//! The handler only calls `write` and `_exit`, which are async-signal-safe.
//! The report is written after all the work is done, so a signal before that
//! leaves nothing on stdout. The process never unwinds from the handler.

#[cfg(unix)]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};

    static HOLD: AtomicBool = AtomicBool::new(false);
    static PENDING: AtomicBool = AtomicBool::new(false);
    const SIGINT: i32 = 2;
    const MESSAGE: &[u8] = b"cdn-egress-bill: interrupted by SIGINT\n";

    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
        fn write(fd: i32, buf: *const std::ffi::c_void, len: usize) -> isize;
        fn _exit(code: i32) -> !;
    }

    extern "C" fn on_sigint(_sig: i32) {
        if HOLD.load(Ordering::SeqCst) {
            PENDING.store(true, Ordering::SeqCst);
            return;
        }
        // SAFETY: `write` and `_exit` are async-signal-safe, and MESSAGE is a static slice.
        unsafe {
            let _ = write(2, MESSAGE.as_ptr().cast(), MESSAGE.len());
            _exit(130)
        }
    }

    pub fn hold() {
        HOLD.store(true, Ordering::SeqCst);
    }

    pub fn release() -> bool {
        HOLD.store(false, Ordering::SeqCst);
        PENDING.load(Ordering::SeqCst)
    }

    pub fn install() {
        // SAFETY: `on_sigint` has the C signature `signal` expects and only calls
        // async-signal-safe functions.
        unsafe {
            signal(SIGINT, on_sigint as extern "C" fn(i32) as usize);
        }
    }
}

#[cfg(not(unix))]
mod imp {
    pub fn install() {}
    pub fn hold() {}
    pub fn release() -> bool {
        false
    }
}

/// Install the SIGINT handler. Other platforms keep their default behaviour.
pub fn install() {
    imp::install();
}

/// While held, a SIGINT is remembered instead of ending the process. Used around
/// the publishing of an output file, so that no temporary file is left behind.
pub fn hold() {
    imp::hold();
}

/// End the hold. True when a SIGINT arrived during it: the caller then stops with exit 130.
pub fn release() -> bool {
    imp::release()
}

/// The exit code and stderr line for a SIGINT that was held back.
pub const EXIT_INTERRUPTED: u8 = 130;
pub const INTERRUPTED_LINE: &str = "cdn-egress-bill: interrupted by SIGINT";
