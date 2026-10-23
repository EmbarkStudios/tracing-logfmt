//! Windows-specific OS utilities.
#![allow(unsafe_code, reason = "need to call system utilities in os code")]

#[link(name = "kernel32", kind = "raw-dylib")]
unsafe extern "system" {
    safe fn GetCurrentThreadId() -> u32;
}

/// Get the current system thread id.
#[inline]
pub(super) fn gettid() -> u64 {
    GetCurrentThreadId() as _
}
