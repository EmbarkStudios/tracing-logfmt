//! Apple-specific OS utilities.
#![allow(unsafe_code, reason = "need to call system utilities in os code")]

/// Get the current system thread id.
#[inline]
pub(super) fn gettid() -> u64 {
    let mut id: u64 = 0;
    // SAFETY: id is stack-allocated and guaranteed to outlast the call.
    let res = unsafe { libc::pthread_threadid_np(0, &mut id) };

    (res == 0)
        .then_some(id)
        .expect("pthread_threadid_np only fails for NULL id or dead thread")
}
