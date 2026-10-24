//! Machine-wide renderer serialization, shared by oracle tools and logical replay.
use std::{fs, path::Path, process};

pub(crate) fn acquire_snes9x_compare_lock_mode(exclusive: bool) -> fs::File {
    use std::os::unix::io::AsRawFd;
    let path = Path::new("/tmp/zelda3-snes9x-compare.lock");
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|error| {
            eprintln!("failed to open {}: {error}", path.display());
            process::exit(2);
        });
    if !exclusive {
        return file;
    }
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        eprintln!(
            "another rendering Snes9x comparison is already running (lock: {}); GPU comparisons must run serially (renderless runs take no lock)",
            path.display()
        );
        process::exit(2);
    }
    file
}
