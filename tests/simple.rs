extern crate mio;
extern crate mio_serial;
extern crate dotenv;

use self::dotenv::dotenv;
use std::env;
use std::io::{Read, Write};
use std::time::Duration;

use mio::{Events, Poll, Ready, PollOpt, Token};
use mio::timer::Timer;
use mio_serial::{SerialPort, PortSettings};
use mio_serial::{BaudRate, CharSize, Parity, StopBits, FlowControl};

#[test]
fn poll_write() {
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

    let mut events = Events::with_capacity(256);
    let buf = [0; 256];

    let poll = Poll::new().unwrap();

    poll.register(&serial_port, Token(0), Ready::writable(), PollOpt::level()).unwrap();

    assert!(poll.poll(&mut events, None).is_ok());
    assert!(events.len() > 0);

    for event in &events {
        assert_eq!(event.token(), Token(0));
        assert!(event.kind().is_writable());
        assert!(serial_port.write(&buf[..]).is_ok());
    }
}

#[test]
fn poll_read() {
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

    let mut events = Events::with_capacity(256);
    let mut buf = [0; 256];

    let poll = Poll::new().unwrap();
    let mut timer = Timer::default();

    timer.set_timeout(Duration::from_millis(5000), "timeout").unwrap();

    poll.register(&serial_port, Token(0), Ready::readable(), PollOpt::level()).unwrap();
    poll.register(&timer, Token(1), Ready::readable(), PollOpt::edge()).unwrap();

    assert!(poll.poll(&mut events, None).is_ok());
    assert!(events.len() > 0);

    for event in &events {
        match event.token() {
            Token(0) => {
                assert!(event.kind().is_readable());
                assert!(serial_port.read(&mut buf[..]).is_ok());
            }
            Token(1) => {
                assert_eq!("timeout", timer.poll().unwrap());
                assert_eq!(None, timer.poll());
            }
            _ => assert!(false)
        }
    }

    let clone = serial_port.try_clone();
    assert!(clone.is_ok());
    let clone_port = clone.unwrap();

    timer.set_timeout(Duration::from_millis(5000), "timeout").unwrap();

    poll.register(&clone_port, Token(0), Ready::readable(), PollOpt::level()).unwrap();

    assert!(poll.poll(&mut events, None).is_ok());
    assert!(events.len() > 0);

    for event in &events {
        match event.token() {
            Token(0) => {
                assert!(event.kind().is_readable());
                assert!(serial_port.read(&mut buf[..]).is_ok());
            }
            Token(1) => {
                assert_eq!("timeout", timer.poll().unwrap());
                assert_eq!(None, timer.poll());
            }
            _ => assert!(false)
        }
    }
}
