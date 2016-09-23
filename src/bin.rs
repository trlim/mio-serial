extern crate mio;
extern crate mio_serial;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

use mio_serial::{SerialPort, PortSettings, BaudRate, CharSize, Parity, StopBits, FlowControl};
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
            baud_rate: BaudRate::Baud115200,
            char_size: CharSize::Bits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::Stop1,
            flow_control: FlowControl::FlowNone
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
