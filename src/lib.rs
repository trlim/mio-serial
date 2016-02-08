extern crate mio;
extern crate serial;

// use serial::prelude::*;
use serial::SerialPort as Port;

use mio::{Evented, Poll};
#[cfg(unix)] use mio::unix::{EventedFd};
#[cfg(unix)] use std::os::unix::io::{RawFd, AsRawFd};

struct SerialPort {
    port: serial::SystemPort,

    #[cfg(unix)]
    fd: RawFd,

    #[cfg(windows)]
    handle: RawHandle,
}

// impl SerialPort {
// }

impl From<serial::SystemPort> for SerialPort {
    #[cfg(unix)]
    fn from(port: serial::SystemPort) -> SerialPort {
        let fd = port.as_raw_fd();
        SerialPort {
            port: port,
            fd: fd
        }
    }

    #[cfg(windows)]
    fn from(port: serial::SystemPort) -> SerialPort {
        let handle = port.as_raw_handle();
        SerialPort {
            port: port,
            handle: handle
        }
    }
}

#[cfg(unix)]
impl Evented for SerialPort {
    fn register(&self, poll: &mut Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let evented = EventedFd(&self.fd);
        evented.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &mut Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let evented = EventedFd(&self.fd);
        evented.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mut Poll) -> io::Result<()> {
        let evented = EventedFd(&self.fd);
        evented.deregister(poll)
    }
}

#[cfg(windows)]
impl Evented for SerialPort {
    fn register(&self, poll: &mut Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let evented = EventedHandle(&self.handle);
        evented.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &mut Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        let evented = EventedHandle(&self.handle);
        evented.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mut Poll) -> io::Result<()> {
        let evented = EventedHandle(&self.handle);
        evented.deregister(poll)
    }
}

use std::io;
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

use mio::{EventLoop, EventSet, PollOpt, Handler, Token};

struct SerialPortHandler;

impl Handler for SerialPortHandler {
    type Timeout = ();
    type Message = u32;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        println!("ready {:?} {:?}", token, events);

        match token {
            Token(0) => {
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

#[test]
fn it_works() {
    let port_name = "/dev/tty.SLAB_USBtoUART";
    let port_name = "/dev/tty.usbserial";

    let mut serial_port = serial::open(port_name).unwrap();

    setup_serial_port(&mut serial_port).unwrap();

    let port = SerialPort::from(serial_port);

    let mut event_loop = EventLoop::new().unwrap();

    event_loop.register(&port, Token(0), EventSet::readable(), PollOpt::level()).unwrap();

    let _ = event_loop.run(&mut SerialPortHandler);
}
