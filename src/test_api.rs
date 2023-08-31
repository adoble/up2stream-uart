use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};

use super::*;

#[test]
fn send_command() {
    let msg = "CMD:on;".as_bytes();
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(msg),
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
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"CMD:on;"),
        //SerialTransaction::read_error(nb::Error::WouldBlock),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "on");

    serial.done();
}

#[test]
fn send_query_rx_with_noise_at_begining() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"CMD;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"42;\n\rMD:off;"), // Noise
        SerialTransaction::read_many(b"CMD:on;"),
        //SerialTransaction::read_many(b"\n\r"), // Noise
        //SerialTransaction::read_error(nb::Error::WouldBlock),
    ];
    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "on");

    serial.done();
}

#[test]
fn send_query_slow_response() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"CMD;"),
        SerialTransaction::flush(),
        SerialTransaction::read_error(nb::Error::WouldBlock),
        SerialTransaction::read_error(nb::Error::WouldBlock),
        SerialTransaction::read_error(nb::Error::WouldBlock),
        SerialTransaction::read_error(nb::Error::WouldBlock),
        SerialTransaction::read_many(b"42;\n\rMD:off;"), // Noise
        SerialTransaction::read_many(b"CMD:on;"),
        //SerialTransaction::read_many(b"\n\r"), // Noise
        //SerialTransaction::read_error(nb::Error::WouldBlock),
    ];
    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "on");

    serial.done();
}

#[test]
fn send_query_noise_at_end() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"CMD;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"CMD:on;"),
        SerialTransaction::read_many(b"\n\r"), // Noise
        SerialTransaction::read_error(nb::Error::WouldBlock),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "on");

    // Mop up the noise
    serial.read().unwrap();
    serial.read().unwrap();
    serial.read().unwrap_err();

    serial.done();
}

#[test]
fn send_query_rx_parameter_list() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"CMD;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"CMD:BT,1,456,PARA;"),
    ];
    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.send_query("CMD").unwrap();

    assert_eq!(response.as_str(), "BT,1,456,PARA");

    serial.done();
}

#[test]
fn firmware_version() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VER;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"VER:1234-13-42;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.firmware_version().unwrap();

    assert_eq!("1234-13-42", response);

    serial.done();
}

#[test]
fn device_status() -> Result<(), Error> {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"STA;"),
        SerialTransaction::flush(),
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
    assert_eq!(SystemControl::Reboot.to_parameter_str(&mut buf), b"REBOOT");
    buf = [0; 10];
    assert_eq!(
        SystemControl::Standby.to_parameter_str(&mut buf),
        b"STANDBY"
    );
    buf = [0; 10];
    assert_eq!(SystemControl::Reset.to_parameter_str(&mut buf), b"RESET");

    // Device API version 4 functionality
    // buf = [0; 10];
    // assert_eq!(
    //     SystemControl::Recover.to_parameter_str(&mut buf),
    //     b"RECOVER"
    // );
}

#[test]
fn execute_system_control() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SYS:RESET;"),
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
        SerialTransaction::flush(),
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
fn audio_out() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"AUD;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"AUD:1;"),
        //SerialTransaction::read_error(nb::Error::WouldBlock),
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
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"AUD:T;"),
        //SerialTransaction::read_error(nb::Error::WouldBlock),
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
        SerialTransaction::flush(),
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
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.select_input_source(Source::Coax);

    assert!(response.is_ok());

    serial.done();
}

// This is a runnable test of setting the volume one step lower. This forms the
// example in the library documentation
#[test]
fn volume_doc_code_test() -> Result<(), Error> {
    let initial_expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VOL:50;"),
        SerialTransaction::write_many(b"VOL;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"VOL:50;"),
        SerialTransaction::write_many(b"VOL:49;"),
        //SerialTransaction::flush(),
    ];

    let mut serial = SerialMock::new(&initial_expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    // Set the initial volume
    let initial_vol = Volume::new(50)?;
    up2stream_device.set_volume(initial_vol)?;

    // Get the volume
    let actual_volume = up2stream_device.volume()?.get();

    // Reduce the volume by 1 step
    if actual_volume > 0 {
        let new_volume = Volume::new(actual_volume - 1)?;
        up2stream_device.set_volume(new_volume)?;
    }

    serial.done();

    Ok(())
}

#[test]
fn volume() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"VOL;"),
        SerialTransaction::flush(),
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
        SerialTransaction::flush(),
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
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_mute(Switch::Toggle);

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn treble() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"TRE;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"TRE:-3;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.treble();

    assert!(response.is_ok());

    assert_eq!(response.unwrap(), Treble::new(-3).unwrap());

    serial.done();
}

#[test]
fn set_treble() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"TRE:-6;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.set_treble(Treble::new(-6).unwrap());

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn toogle_play_pause() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"POP;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.play_pause_toggle();

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn stop() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"SRC:NET;"),
        SerialTransaction::write_many(b"STP;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.stop();

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn stop_err() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"SRC:BT;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.stop();

    if let Err(e) = response {
        match e {
            Error::NotSupportedForDeviceSource => assert!(true),
            _ => assert!(false, "Incorrect error message"),
        }
    } else {
        assert!(false, "Error expected");
    };

    serial.done();
}

#[test]
fn next() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"SRC:BT;"),
        SerialTransaction::write_many(b"NXT;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.next();

    assert!(response.is_ok());

    serial.done();
}

#[test]
fn next_err() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"SRC:COAX;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.next();

    if let Err(e) = response {
        match e {
            Error::NotSupportedForDeviceSource => assert!(true),
            _ => assert!(false, "Incorrect error message"),
        }
    } else {
        assert!(false, "Error expected");
    };

    serial.done();
}

#[test]
fn previous() {
    let expectations = [
        SerialTransaction::write(b';'),
        SerialTransaction::write_many(b"SRC;"),
        SerialTransaction::flush(),
        SerialTransaction::read_many(b"SRC:BT;"),
        SerialTransaction::write_many(b"PRE;"),
    ];

    let mut serial = SerialMock::new(&expectations);

    let mut up2stream_device = Up2Stream::new(&mut serial);

    let response = up2stream_device.previous();

    assert!(response.is_ok());

    serial.done();
}
