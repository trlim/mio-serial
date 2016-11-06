extern crate mio;
extern crate mio_serial;
extern crate dotenv;

use self::dotenv::dotenv;
use std::env;
use std::io::{self, Read, Write};
use std::time::Duration;

use mio::{Events, Poll, Ready, PollOpt, Token};
use mio_serial::{SerialPort, PortSettings};
use mio_serial::{BaudRate, CharSize, Parity, StopBits, FlowControl};

extern crate serial;
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
