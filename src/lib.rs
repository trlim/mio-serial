extern crate mio;
extern crate serial;

use std::ffi::OsStr;
use std::io;

// import SerialPort trait for configure()
use serial::SerialPort as _SerialPort;

// Re-exports
pub use serial::PortSettings;
pub use serial::{BaudRate, CharSize, Parity, StopBits, FlowControl};

/// A serial port.
///
/// This type represents a serial port and is a simple wrapper over `SystemPort` of `serial-rs`.
///
/// `SerialPort` implements `Read`, `Write`, `Evented`, `AsRawFd`(or `AsRawHandle` on Windows) traits
/// for interoperating with other I/O code.
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
    pub fn open_with_settings<T: AsRef<OsStr> + ?Sized>(port_name: &T, settings: &PortSettings) -> io::Result<SerialPort> {
        let mut system_port = try!(serial::open(port_name));

        try!(system_port.configure(settings));

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
impl AsRawFd for SerialPort {
    fn as_raw_fd(&self) -> i32 {
        self.inner.as_raw_fd()
    }
}

use mio::{Evented, Poll, Token, Ready, PollOpt};

#[cfg(unix)] use mio::unix::{EventedFd};
#[cfg(unix)] use std::os::unix::io::{AsRawFd};

#[cfg(unix)]
impl Evented for SerialPort {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

#[cfg(windows)]
impl Evented for SerialPort {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedHandle(&self.as_raw_handle()).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedHandle(&self.as_raw_handle()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedHandle(&self.as_raw_handle()).deregister(poll)
    }
}

extern crate dotenv;

#[cfg(test)]
mod tests {
    extern crate serial;

    use dotenv::dotenv;
    use std::env;

    use std::io::{self, Read, Write};

    use serial::SerialPort as _SerialPort;

    use mio::{Ready, PollOpt, Token};
    use mio::deprecated::{EventLoop, Handler};
    use super::*;

    pub struct SerialPortHandler {
        port: SerialPort
    }

    impl Handler for SerialPortHandler {
        type Timeout = ();
        type Message = u32;

        fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: Ready) {
            println!("ready {:?} {:?}", token, events);

            match token {
                Token(0) => {
                    let mut buf: Vec<u8> = (0..255).collect();
                    if events.is_writable() {
                        self.port.write(&buf[..]).unwrap();
                    }
                    if events.is_readable() {
                        self.port.read(&mut buf[..]).unwrap();
                    }

                    event_loop.shutdown();
                }
                _ => {

                }
            }
        }
        fn notify(&mut self, event_loop: &mut EventLoop<SerialPortHandler>, _msg: Self::Message) {
            println!("{:?}", _msg);
            event_loop.shutdown();
        }
    }

    use std::time::Duration;

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
                flow_control: FlowControl::FlowNone
            }).unwrap();

        setup_serial_port(&mut serial_port.system_port()).unwrap();

        let mut handler = SerialPortHandler { port: serial_port };

        let mut event_loop = EventLoop::new().unwrap();

        event_loop.register(&handler.port, Token(0), Ready::writable(), PollOpt::level()).unwrap();

        let _ = event_loop.run(&mut handler);

        event_loop.reregister(&handler.port, Token(0), Ready::readable(), PollOpt::level()).unwrap();

        let _ = event_loop.run(&mut handler);
    }
}
