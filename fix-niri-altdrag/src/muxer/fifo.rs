use std::{ffi::CString, fs::{self, File, OpenOptions}, io::{self, Read, Write}, marker::PhantomData, os::fd::AsRawFd, path::Path, str::FromStr};

use libc::{mkfifo, poll};

/// A _very_ silly trait that lets us interpret types as raw bytes for use in `FifoQueue`.
/// This is not the case under _so many_ circumstances, but we are going to pretend it is.
pub unsafe trait TrustMeBroThisIsSafe {}

/// A wrapper over Unix FIFOs. No more than one consumer should read from the queue at a time,
/// but multiple producers can write to it.
pub struct FifoQueue<T: TrustMeBroThisIsSafe> {
    fifo: File,
    _marker: PhantomData<T>,
}

unsafe impl TrustMeBroThisIsSafe for u8 {}

impl<T: TrustMeBroThisIsSafe> FifoQueue<T> {
    pub fn new(path: &str) -> io::Result<Self> {
        let path = Path::new(path);

        // Ensure the parent directory exists
        fs::create_dir_all(path.parent().unwrap())?;

        // Atomically attempt to create the FIFO, ignore EEXIST
        unsafe {
            let path_c_str = CString::from_str(
                path.to_str().expect("Failed to convert path to CString")
            ).expect("Failed to create CString from path");
            let ret = mkfifo(path_c_str.as_ptr(), 0o644);
            if ret != 0 {
                let err = io::Error::last_os_error();
                if err.raw_os_error() != Some(libc::EEXIST) {
                    eprintln!("Failed to create FIFO at {}: {}", path.display(), err);
                    return Err(err);
                }
            }
        }

        eprintln!("Opening FIFO at {}", path.display());

        // Open the FIFO for reading and writing
        let fifo = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;

        Ok(FifoQueue {
            fifo,
            _marker: PhantomData,
        })
    }

    pub fn read(&mut self, timeout_ms: i32) -> io::Result<T> {
        if timeout_ms > 0 {
            // Poll the fd with a timeout to check if data is available
            unsafe {
                let mut fds = libc::pollfd {
                    fd: self.fifo.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                if poll(&mut fds as *mut libc::pollfd, 1, timeout_ms as i32) != 1 || (fds.revents & libc::POLLIN) == 0 {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, "No data available"));
                }
            }
        }

        // Read the data from the FIFO
        let mut buffer = vec![0u8; std::mem::size_of::<T>()];
        self.fifo.read_exact(&mut buffer)?;
        // Safety: We assume the data is valid for T
        // "Unsafety" is probably a better term for this...
        Ok(unsafe { std::ptr::read(buffer.as_ptr() as *const T) })
    }

    pub fn has_available(&mut self) -> bool {
        // Poll the fd with a timeout of 0 to check if data is available
        unsafe {
            let mut fds = libc::pollfd {
                fd: self.fifo.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            return poll(&mut fds as *mut libc::pollfd, 1, 0) == 1 && (fds.revents & libc::POLLIN) != 0;
        }
    }

    pub fn write(&mut self, item: &T) -> io::Result<()> {
        let bytes = unsafe {
            // Safety: Length is correct
            std::slice::from_raw_parts(item as *const T as *const u8, std::mem::size_of::<T>())
        };
        self.fifo.write_all(bytes)
    }
}