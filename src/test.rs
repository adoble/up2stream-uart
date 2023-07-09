//#[allow(warnings)]
//use embedded_hal::blocking::serial::Write;
use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
//use embedded_hal_mock::serial::{Read, Write};
//use embedded_hal_mock::MockError;

use super::*;

#[test]
fn send_command() {
    let msg = "CMD:on;".as_bytes();
    let expectations = [
        SerialTransaction::write_many(msg),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let _result = up2stream_device.send_command("CMD", "on").unwrap();

    serial.done();
}
