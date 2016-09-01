extern crate mio;
extern crate serial;
extern crate serial_mio;

use serial_mio::{SerialPort, PortSettings};
use mio::{EventLoop, EventSet, PollOpt, Handler, Token};
use std::io::{Read/*, Write*/};

use std::str;

struct SerialPortHandler {
    port: SerialPort
}

impl Handler for SerialPortHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        match token {
            Token(0) => {
                let mut buf = [0; 256];
                if events.is_readable() {
                    while let Ok(n) = self.port.read(&mut buf) {
                        if let Ok(s) = str::from_utf8(&buf[..n]) {
                            print!("{}", s);
                        }
                    }
                }
            }
            _ => {
                println!("ready {:?} {:?}", token, events);
            }
        }
    }
}

pub fn main() {
    let port_name = "/dev/tty.usbserial";

    let serial_port = SerialPort::open_with_settings(port_name,
        &PortSettings {
            baud_rate: serial::Baud115200,
            char_size: serial::Bits8,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
            flow_control: serial::FlowNone
        }).unwrap();

    let mut handler = SerialPortHandler { port: serial_port };

    if let Ok(mut event_loop) = EventLoop::new() {
        if let Ok(_) = event_loop.register(&handler.port, Token(0), EventSet::readable(), PollOpt::level()) {
            let _ = event_loop.run(&mut handler);
        }
    }
}
