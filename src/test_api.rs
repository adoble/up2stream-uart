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
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(msg),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let _result = up2stream_device
        .send_command("CMD", "on".as_bytes())
        .unwrap();

    serial.done();
}

#[test]
fn send_query() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"CMD;"),
        SerialTransaction::read_many(b"CMD:on;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "CMD:on");

    serial.done();
}

#[test]
fn firmware_version() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VER;"),
        SerialTransaction::read_many(b"VER:1234-13-42;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.firmware_version().unwrap();

    assert_eq!("VER:1234-13-42", response);

    serial.done();
}

#[test]
fn device_status() -> Result<(), Error> {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"STA;"),
        SerialTransaction::read_many(b"STA:BT,0,50,-4,4,1,1,1,0,0;"),
    ];

    let expected_device_status = DeviceStatus {
        source: Source::Bluetooth,
        mute: false,
        volume: Volume::new(50)?,
        treble: Treble::new(-4)?,
        bass: Bass::new(4)?,
        net: true,
        internet: true,
        playing: true,
        led: false,
        upgrading: false,
    };

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let device_status = up2stream_device.status().unwrap();

    assert_eq!(device_status, expected_device_status);

    serial.done();

    Ok(())
}

#[test]
fn system_control() {
    let mut buf: [u8; 10] = [0; 10];
    assert_eq!(SystemControl::Reboot.as_parameter_str(&mut buf), b"REBOOT");
    buf = [0; 10];
    assert_eq!(
        SystemControl::Standby.as_parameter_str(&mut buf),
        b"STANDBY"
    );
    buf = [0; 10];
    assert_eq!(
        SystemControl::Recover.as_parameter_str(&mut buf),
        b"RECOVER"
    );
    buf = [0; 10];
    assert_eq!(SystemControl::Reset.as_parameter_str(&mut buf), b"RESET");
}

#[test]
fn execute_system_control() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SYS:RESET;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.execute_system_control(SystemControl::Reset);

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn internet_connection() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"WWW;"),
        SerialTransaction::read_many(b"WWW:1;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.internet_connection();

    assert!(response.is_ok());

    assert_eq!(response.unwrap(), true);

    serial.done();
}

#[test]
fn internet_connection_err() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"WWW;"),
        SerialTransaction::read_many(b"WWWW:1;"), // Incorrect data
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.internet_connection();

    assert!(response.is_err());

    serial.done();
}

#[test]
fn audio_out() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"AUD;"),
        SerialTransaction::read_many(b"AUD:1;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.audio_out();

    assert!(response.is_ok());

    assert!(response.unwrap());

    serial.done();
}

#[test]
fn audio_out_err() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"AUD;"),
        SerialTransaction::read_many(b"AUD:T;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.audio_out();

    assert!(response.is_err());

    serial.done();
}
#[test]
fn set_audio_out() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"AUD:1;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_audio_out(true);

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn input_source() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::read_many(b"SRC:BT;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.input_source();

    assert!(response.is_ok());

    let source = response.unwrap();

    assert_eq!(source, Source::Bluetooth);

    serial.done();
}

#[test]
fn select_input_source() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC:COAX;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.select_input_source(Source::Coax);

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn volume() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VOL;"),
        SerialTransaction::read_many(b"VOL:50;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.volume();

    assert!(response.is_ok());

    assert_eq!(response.unwrap(), Volume::new(50).unwrap());

    serial.done();
}

#[test]
fn set_volume() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VOL:34;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_volume(Volume::new(34).unwrap());

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn mute_status() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"MUT;"),
        SerialTransaction::read_many(b"MUT:0;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.mute_status().unwrap();

    assert!(!response);

    serial.done();
}

#[test]
fn set_mute() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"MUT:1;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_mute(Switch::On);

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn toogle_mute() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"MUT:T;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_mute(Switch::Toggle);

    assert!(response.is_ok());

    serial.done();
}
