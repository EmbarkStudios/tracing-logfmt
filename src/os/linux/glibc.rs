//! libc's `gettid`, looked up at runtime since glibc < 2.30 does not export it.

type GetTidFn = extern "C" fn() -> libc::pid_t;

/// Looks up libc's `gettid`, if it exports one.
///
/// The lookup runs once per process.
fn gettid_fn() -> Option<GetTidFn> {
    static GETTID: std::sync::OnceLock<Option<GetTidFn>> = std::sync::OnceLock::new();

    *GETTID.get_or_init(|| {
        // SAFETY: the name is NUL-terminated and `RTLD_DEFAULT` is a valid handle.
        let res: *mut libc::c_void = unsafe { libc::dlsym(libc::RTLD_DEFAULT, c"gettid".as_ptr()) };

        if res.is_null() {
            return None;
        }

        // SAFETY: libc's `gettid` is `pid_t gettid(void)`, which is what
        // `GetTidFn` declares, and `res` is not null.
        Some(unsafe { std::mem::transmute::<*mut libc::c_void, GetTidFn>(res) })
    })
}

/// Gets the calling thread's id from libc's `gettid`, if libc exports one.
pub(super) fn gettid() -> Option<u64> {
    gettid_fn().map(|gettid| gettid() as u64)
}
