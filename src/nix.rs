use std::io::Error;


pub fn check_err<T: Ord + Default>(num: T) -> std::io::Result<T> {
    if num < T::default() {
        Err(Error::last_os_error())
    } else {
        Ok(num)
    }
}

pub fn fork() -> std::io::Result<u32> {
    check_err(unsafe { libc::fork() }).map(|pid| pid as u32)
}