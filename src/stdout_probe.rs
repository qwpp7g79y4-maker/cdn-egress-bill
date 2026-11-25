//! Het enige unsafe-stuk van het project.
//!
//! Rust vervangt een gesloten fd 0, 1 of 2 vóór `main` door `/dev/null`
//! (lees-schrijf). Daarna is een echt gesloten stdout niet meer te
//! onderscheiden van `1<>/dev/null`. Een constructor in `.init_array` draait
//! eerder dan die vervanging en legt vast of fd 1 toen al dicht was.

use std::sync::atomic::{AtomicBool, Ordering};

static FD1_WAS_CLOSED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
mod probe {
    use super::{AtomicBool, Ordering, FD1_WAS_CLOSED};

    extern "C" {
        fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }
    const F_GETFD: i32 = 1;

    extern "C" fn record() {
        // SAFETY: `fcntl(1, F_GETFD)` leest alleen een vlag en wijzigt niets.
        let closed = unsafe { fcntl(1, F_GETFD) } == -1;
        let flag: &AtomicBool = &FD1_WAS_CLOSED;
        flag.store(closed, Ordering::Relaxed);
    }

    #[used]
    #[link_section = ".init_array"]
    static INIT: extern "C" fn() = record;
}

/// Was fd 1 al dicht toen het proces startte? Alleen betrouwbaar op Linux.
pub fn stdout_was_closed_at_start() -> bool {
    FD1_WAS_CLOSED.load(Ordering::Relaxed)
}
