//! Linux-specific OS utilities.
#![allow(unsafe_code, reason = "need to call system utilities in os code")]

mod glibc;

/// Get the current thread's system ID.
#[inline]
pub(super) fn gettid() -> u64 {
    fn syscall_gettid() -> u64 {
        // SAFETY: the gettid syscall takes no arguments and touches no memory.
        unsafe { libc::syscall(libc::SYS_gettid) as u64 }
    }

    glibc::gettid().unwrap_or_else(syscall_gettid)
}
