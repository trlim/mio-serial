extern crate mio;
extern crate serial_mio;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

use serial_mio::{SerialPort, PortSettings};
use mio::{Events, Poll, Ready, PollOpt, Token};
use std::io::{Read/*, Write*/};

use std::str;

pub fn main() {
    dotenv().ok();

    let port_name = env::args().nth(1)
        .or(env::var("SERIAL_PORT").ok())
        .expect("serial port name must be specified");

    println!("port {:?}", port_name);

    let mut serial_port = SerialPort::open_with_settings(port_name.as_str(),
        &PortSettings {
            baud_rate: serial_mio::Baud115200,
            char_size: serial_mio::Bits8,
            parity: serial_mio::ParityNone,
            stop_bits: serial_mio::Stop1,
            flow_control: serial_mio::FlowNone
        }).expect("Can't open serial port");

    if let Ok(poll) = Poll::new() {
        if let Ok(_) = poll.register(&serial_port, Token(0), Ready::readable(), PollOpt::level()) {
            let mut events = Events::with_capacity(256);
            let mut buf = [0; 256];

            loop {
                match poll.poll(&mut events, None) {
                    Ok(_) => {
                        for event in &events {
                            match event.token() {
                                Token(0) => {
                                    if event.kind().is_readable() {
                                        while let Ok(n) = serial_port.read(&mut buf) {
                                            if let Ok(s) = str::from_utf8(&buf[..n]) {
                                                print!("{}", s);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!("ready {:?}", event.token());
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
