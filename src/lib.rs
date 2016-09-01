extern crate mio;
extern crate serial;

use std::ffi::OsStr;
use std::io;

use serial::SerialPort as _SerialPort;
pub use serial::PortSettings;

pub struct SerialPort(pub serial::SystemPort);

impl SerialPort {
    pub fn open<T: AsRef<OsStr> + ?Sized>(port_name: &T) -> io::Result<SerialPort> {
        let system_port = try!(serial::open(port_name));

        Ok(SerialPort::from(system_port))
    }

    pub fn open_with_settings<T: AsRef<OsStr> + ?Sized>(port_name: &T, settings: &PortSettings) -> io::Result<SerialPort> {
        let mut system_port = try!(serial::open(port_name));

        try!(system_port.configure(settings));

        Ok(SerialPort::from(system_port))
    }
}

impl From<serial::SystemPort> for SerialPort {
    fn from(port: serial::SystemPort) -> SerialPort {
        SerialPort(port)
    }
}

impl io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

use mio::{Evented, Selector, Token, EventSet, PollOpt};

#[cfg(unix)] use mio::unix::{EventedFd};
#[cfg(unix)] use std::os::unix::io::{AsRawFd};

#[cfg(unix)]
impl Evented for SerialPort {
    fn register(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let fd = self.0.as_raw_fd();
        let evented = EventedFd(&fd);
        evented.register(selector, token, interest, opts)
    }

    fn reregister(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let fd = self.0.as_raw_fd();
        let evented = EventedFd(&fd);
        evented.reregister(selector, token, interest, opts)
    }

    fn deregister(&self, selector: &mut Selector) -> io::Result<()> {
        let fd = self.0.as_raw_fd();
        let evented = EventedFd(&fd);
        evented.deregister(selector)
    }
}

#[cfg(windows)]
impl Evented for SerialPort {
    fn register(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let fd = self.0.as_raw_handle();
        let evented = EventedHandle(&handle);
        evented.register(selector, token, interest, opts)
    }

    fn reregister(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let fd = self.0.as_raw_handle();
        let evented = EventedHandle(&handle);
        evented.reregister(selector, token, interest, opts)
    }

    fn deregister(&self, selector: &mut Selector) -> io::Result<()> {
        let fd = self.0.as_raw_handle();
        let evented = EventedHandle(&handle);
        evented.deregister(selector)
    }
}

#[cfg(test)]
mod tests {
    extern crate serial;

    use std::io::{self, Read, Write};

    use serial::SerialPort as _SerialPort;

    use mio::{EventLoop, EventSet, PollOpt, Handler, Token};

    use super::{SerialPort, PortSettings};

    pub struct SerialPortHandler {
        port: SerialPort
    }

    impl Handler for SerialPortHandler {
        type Timeout = ();
        type Message = u32;

        fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
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
        try!(serial_port.reconfigure(&|settings| {
            try!(settings.set_baud_rate(serial::Baud115200));
            // try!(settings.set_baud_rate(serial::Baud9600));
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }));

        try!(serial_port.set_timeout(Duration::from_millis(10000)));

        Ok(())
    }

    #[test]
    fn serial_port() {
        let port_name = "/dev/tty.SLAB_USBtoUART";
        let port_name = "/dev/tty.usbserial";
        let port_name = "/dev/tty.usbmodem1A1211";

        {
            let _ = SerialPort::open_with_settings(port_name,
                &PortSettings {
                    baud_rate: serial::Baud115200,
                    char_size: serial::Bits8,
                    parity: serial::ParityNone,
                    stop_bits: serial::Stop1,
                    flow_control: serial::FlowNone
                }).unwrap();
        }

        let mut serial_port = SerialPort::open(port_name).unwrap();

        setup_serial_port(&mut serial_port.0).unwrap();

        let mut handler = SerialPortHandler { port: serial_port };

        let mut event_loop = EventLoop::new().unwrap();

        event_loop.register(&handler.port, Token(0), EventSet::writable(), PollOpt::level()).unwrap();

        let _ = event_loop.run(&mut handler);

        event_loop.reregister(&handler.port, Token(0), EventSet::readable(), PollOpt::level()).unwrap();

        let _ = event_loop.run(&mut handler);
    }
}
