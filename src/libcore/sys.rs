#[doc = "Misc low level stuff"];

export last_os_error;
export size_of;
export align_of;
export refcount;
export log_str;
export set_exit_status;

#[abi = "cdecl"]
native mod rustrt {
    // Explicitly re-export native stuff we want to be made
    // available outside this crate. Otherwise it's
    // visible-in-crate, but not re-exported.
    fn last_os_error() -> str;
    fn refcount<T>(t: @T) -> libc::intptr_t;
    fn unsupervise();
    fn shape_log_str<T>(t: *refl::type_desc, data: T) -> str;
    fn rust_set_exit_status(code: libc::intptr_t);
}

#[abi = "rust-intrinsic"]
native mod rusti {
    fn size_of<T>() -> uint;
    fn align_of<T>() -> uint;
}

#[doc = "Get a string representing the platform-dependent last error"]
fn last_os_error() -> str {
    rustrt::last_os_error()
}

#[doc = "Returns the size of a type"]
fn size_of<T>() -> uint unsafe {
    rusti::size_of::<T>()
}

#[doc = "Returns the alignment of a type"]
fn align_of<T>() -> uint unsafe {
    rusti::align_of::<T>()
}

#[doc = "Returns the refcount of a shared box"]
fn refcount<T>(t: @T) -> uint {
    ret rustrt::refcount::<T>(t) as uint;
}

fn log_str<T>(t: T) -> str {
    rustrt::shape_log_str(refl::get_type_desc::<T>(), t)
}

#[doc = "
Sets the process exit code

Sets the exit code returned by the process if all supervised tasks terminate
successfully (without failing). If the current root task fails and is
supervised by the scheduler then any user-specified exit status is ignored and
the process exits with the default failure status
"]
fn set_exit_status(code: int) {
    rustrt::rust_set_exit_status(code as libc::intptr_t);
}

#[cfg(test)]
mod tests {

    #[test]
    fn last_os_error() {
        log(debug, last_os_error());
    }

    #[test]
    fn size_of_basic() {
        assert size_of::<u8>() == 1u;
        assert size_of::<u16>() == 2u;
        assert size_of::<u32>() == 4u;
        assert size_of::<u64>() == 8u;
    }

    #[test]
    #[cfg(target_arch = "x86")]
    #[cfg(target_arch = "arm")]
    fn size_of_32() {
        assert size_of::<uint>() == 4u;
        assert size_of::<*uint>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn size_of_64() {
        assert size_of::<uint>() == 8u;
        assert size_of::<*uint>() == 8u;
    }

    #[test]
    fn align_of_basic() {
        assert align_of::<u8>() == 1u;
        assert align_of::<u16>() == 2u;
        assert align_of::<u32>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86")]
    #[cfg(target_arch = "arm")]
    fn align_of_32() {
        assert align_of::<uint>() == 4u;
        assert align_of::<*uint>() == 4u;
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn align_of_64() {
        assert align_of::<uint>() == 8u;
        assert align_of::<*uint>() == 8u;
    }
}

// Local Variables:
// mode: rust;
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
