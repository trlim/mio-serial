extern crate mio;
extern crate serial_mio;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

use serial_mio::{SerialPort, PortSettings};
use mio::{Ready, PollOpt, Token};
use mio::deprecated::{EventLoop, Handler};
use std::io::{Read/*, Write*/};

use std::str;

struct SerialPortHandler {
    port: SerialPort
}

impl Handler for SerialPortHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, _event_loop: &mut EventLoop<Self>, token: Token, events: Ready) {
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
    dotenv().ok();

    let port_name = env::var("SERIAL_PORT")
        .expect("serial port name must be specified");

    let serial_port = SerialPort::open_with_settings(port_name.as_str(),
        &PortSettings {
            baud_rate: serial_mio::Baud115200,
            char_size: serial_mio::Bits8,
            parity: serial_mio::ParityNone,
            stop_bits: serial_mio::Stop1,
            flow_control: serial_mio::FlowNone
        }).expect("Can't open serial port");

    let mut handler = SerialPortHandler { port: serial_port };

    if let Ok(mut event_loop) = EventLoop::new() {
        if let Ok(_) = event_loop.register(&handler.port, Token(0), Ready::readable(), PollOpt::level()) {
            let _ = event_loop.run(&mut handler);
        }
    }
}
