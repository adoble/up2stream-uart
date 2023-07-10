//#[allow(warnings)]
//use embedded_hal::blocking::serial::Write;
use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
//use embedded_hal_mock::serial::{Read, Write};
//use embedded_hal_mock::MockError;

use super::*;

#[test]
fn send_command() {
    let msg = "CMD:on;\n".as_bytes();
    let expectations = [
        SerialTransaction::write_many(msg),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let _result = up2stream_device.send_command("CMD", "on").unwrap();

    serial.done();
}

#[test]
fn send_query() {
    let expectations = [
        SerialTransaction::write_many(b"CMD\n"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"CMD:on\n"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "CMD:on");

    serial.done();
}

#[test]
fn boolean_from_string() {
    assert!(boolean_from_str("1").unwrap());
    assert!(!boolean_from_str("0").unwrap());
    assert!(boolean_from_str("T").is_err());
}

#[test]
fn source_from_string() {
    const NUMBER_SOURCES: usize = 10;
    let source_strings: [&str; NUMBER_SOURCES] = [
        "NET", "USB", "USBDAC", "LINE-IN", "LINE-IN2", "BT", "OPT", "COAX", "I2S", "HDMI",
    ];
    use Source::*;
    let expected_sources = ArrayVec::from([
        Net, Usb, UsbDac, LineIn, LineIn2, Bluetooth, Optical, Coax, I2S, HDMI,
    ]);
    let mut actual_sources = ArrayVec::<Source, NUMBER_SOURCES>::new();

    // let mut source: Source;
    // for (index, source_str)  in source_strings.iter().enumerate() {
    //     let source = expected_sources
    // }

    for s in source_strings {
        let source = Source::from_str(s).unwrap();
        actual_sources.push(source);
    }

    assert_eq!(actual_sources, expected_sources);

    let source: Result<Source, Error> = Source::from_str("UNKNOWN");
    assert!(source.is_err());
}

#[test]
fn firmware_version_test() {
    let expectations = [
        SerialTransaction::write_many(b"VER\n"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"VER:1234-13-42\n"),
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
        SerialTransaction::write_many(b"STA\n"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"STA:BT,0,50,-4,4,1,1,1,0,0\n"),
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
