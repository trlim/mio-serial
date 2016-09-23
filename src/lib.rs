extern crate mio;
extern crate serial;

use std::ffi::OsStr;
use std::io;

// import SerialPort trait for configure()
use serial::SerialPort as _SerialPort;

// Re-exports
pub use serial::PortSettings;
pub use serial::{BaudRate, CharSize, Parity, StopBits, FlowControl};

/// A serial port for `mio`.
///
/// This type represents a serial port and is a simple wrapper over `SystemPort` of `serial-rs`.
///
/// `SerialPort` implements `Read`, `Write`, `Evented`, `AsRawFd`(or `AsRawHandle` on Windows)
/// traits for interoperating with other I/O code.
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
    use mio::{Evented, Poll, Token, Ready, PollOpt};

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

#[cfg(test)]
mod tests {
    extern crate dotenv;
    extern crate serial;

    use self::dotenv::dotenv;
    use std::env;
    use std::io::{self, Read, Write};

    use mio::{Events, Poll, Ready, PollOpt, Token};
    use super::*;

    use std::time::Duration;
    use serial::SerialPort as _SerialPort;

    fn setup_serial_port(serial_port: &mut serial::SystemPort) -> io::Result<()> {
        try!(serial_port.set_timeout(Duration::from_millis(10000)));

        Ok(())
    }

    #[test]
    fn serial_port() {
        dotenv().ok();

        let port_name = env::var("SERIAL_PORT")
            .expect("Environment variable SERIAL_PORT must be specified");

        let mut serial_port = SerialPort::open_with_settings(port_name.as_str(),
                                                             &PortSettings {
                                                                 baud_rate: BaudRate::Baud115200,
                                                                 char_size: CharSize::Bits8,
                                                                 parity: Parity::ParityNone,
                                                                 stop_bits: StopBits::Stop1,
                                                                 flow_control:
                                                                     FlowControl::FlowNone,
                                                             })
            .unwrap();

        setup_serial_port(&mut serial_port.system_port()).unwrap();

        let mut events = Events::with_capacity(256);
        let mut buf = [0; 256];

        let poll = Poll::new().unwrap();

        poll.register(&serial_port, Token(0), Ready::writable(), PollOpt::level()).unwrap();

        assert!(poll.poll(&mut events, None).is_ok());
        assert!(events.len() > 0);

        for event in &events {
            assert_eq!(event.token(), Token(0));
            assert!(event.kind().is_writable());
            assert!(serial_port.write(&buf[..]).is_ok());
        }

        poll.register(&serial_port, Token(0), Ready::readable(), PollOpt::level()).unwrap();

        assert!(poll.poll(&mut events, None).is_ok());
        assert!(events.len() > 0);

        for event in &events {
            assert_eq!(event.token(), Token(0));
            assert!(event.kind().is_readable());
            assert!(serial_port.read(&mut buf[..]).is_ok());
        }

        let clone = serial_port.try_clone();
        assert!(clone.is_ok());
        let mut clone_port = clone.unwrap();

        poll.register(&clone_port, Token(0), Ready::readable(), PollOpt::level()).unwrap();

        assert!(poll.poll(&mut events, None).is_ok());
        assert!(events.len() > 0);

        for event in &events {
            assert_eq!(event.token(), Token(0));
            assert!(event.kind().is_readable());
            assert!(clone_port.read(&mut buf[..]).is_ok());
        }
    }
}
