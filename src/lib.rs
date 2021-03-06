//! Binding for serial port and `mio`
//!
//! This crate provides bindings between `serial-rs` and `mio`.
//! The APIs and bindings in this crate are somewhat similar to the `mio-uds` crate.

#![deny(missing_docs)]

extern crate mio;
extern crate serial;

use std::ffi::OsStr;
use std::io;

// import SerialPort trait for configure()
use serial::SerialPort as _SerialPort;

// Re-exports
#[doc(no_inline)]
pub use serial::{PortSettings, BaudRate, CharSize, Parity, StopBits, FlowControl};

/// A serial port for `mio`.
///
/// This type represents a serial port and is a simple wrapper over `SystemPort` of `serial-rs`.
///
/// `SerialPort` implements `Read`, `Write`, `Evented`, `AsRawFd`(or `AsRawHandle` on Windows)
/// traits for interoperating with other I/O code.
#[derive(Debug)]
pub struct SerialPort {
    inner: serial::SystemPort,
}

impl SerialPort {
    /// open serial port named by port_name with default settings.
    ///
    pub fn open<T: AsRef<OsStr> + ?Sized>(port_name: &T) -> io::Result<SerialPort> {
        let system_port = try!(serial::open(port_name));

        Ok(SerialPort { inner: system_port })
    }

    /// open serial port named by port_name with custom settings.
    ///
    pub fn open_with_settings<T: AsRef<OsStr> + ?Sized>(port_name: &T,
                                                        settings: &PortSettings)
                                                        -> io::Result<SerialPort> {
        let mut system_port = try!(serial::open(port_name));

        try!(system_port.configure(settings));

        Ok(SerialPort { inner: system_port })
    }

    /// Creates a new independently owned handle to the underlying serial port.
    ///
    /// The returned `SerialPort` is a reference to the same state that this object references.
    /// Both handles will read and write.
    pub fn try_clone(&self) -> io::Result<SerialPort> {
        let system_port = try!(self.inner.duplicate());
        Ok(SerialPort { inner: system_port })
    }

    /// Will be removed later. Do NOT use!
    pub fn system_port(&mut self) -> &mut serial::SystemPort {
        &mut self.inner
    }
}

impl io::Read for SerialPort {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.inner.read(bytes)
    }
}

impl<'a> io::Read for &'a SerialPort {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        (&self.inner).read(bytes)
    }
}

impl io::Write for SerialPort {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.inner.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<'a> io::Write for &'a SerialPort {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        (&self.inner).write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.inner).flush()
    }
}

#[cfg(unix)]
mod sys {
    use super::SerialPort;

    use std::io;
    use mio::{Evented, Poll, Token, Ready, PollOpt};
    use std::os::unix::io::AsRawFd;
    use mio::unix::EventedFd;

    impl AsRawFd for SerialPort {
        fn as_raw_fd(&self) -> i32 {
            self.inner.as_raw_fd()
        }
    }

    impl Evented for SerialPort {
        fn register(&self,
                    poll: &Poll,
                    token: Token,
                    interest: Ready,
                    opts: PollOpt)
                    -> io::Result<()> {
            EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
        }

        fn reregister(&self,
                      poll: &Poll,
                      token: Token,
                      interest: Ready,
                      opts: PollOpt)
                      -> io::Result<()> {
            EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedFd(&self.as_raw_fd()).deregister(poll)
        }
    }
}

#[cfg(windows)]
mod sys {
    use super::SerialPort;

    use std::io;
    use std::os::windows::io::{AsRawHandle, RawHandle};
    use mio::{Evented, Poll, Token, Ready, PollOpt};

    impl AsRawHandle for SerialPort {
        fn as_raw_handle(&self) -> RawHandle {
            self.inner.as_raw_handle()
        }
    }

    impl Evented for SerialPort {
        fn register(&self,
                    poll: &Poll,
                    token: Token,
                    interest: Ready,
                    opts: PollOpt)
                    -> io::Result<()> {
            EventedHandle(&self.as_raw_handle()).register(poll, token, interest, opts)
        }

        fn reregister(&self,
                      poll: &Poll,
                      token: Token,
                      interest: Ready,
                      opts: PollOpt)
                      -> io::Result<()> {
            EventedHandle(&self.as_raw_handle()).reregister(poll, token, interest, opts)
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            EventedHandle(&self.as_raw_handle()).deregister(poll)
        }
    }
}
