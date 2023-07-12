//! A driver for the Arylic Up2Steam board. This only uses the UART interface.
//!

//#![no_std]
// DO not include the standard library, except when testing.
#![cfg_attr(not(test), no_std)]
//#![no_main]
#![allow(dead_code)]

// See API description:
// https://docs.google.com/spreadsheets/d/1gOb4VBruyJgaBZJHClV6dEkoiwf5hkzE/edit?pli=1#gid=425762154

// Accoring to this https://github.com/Resinchem/Arylic-Amp-MQTT/blob/main/src/arylic_amp.ino
// the commands are terminated with ';' and query responses are terminated with '\n'
// This is confirmed here https://forum.arylic.com/t/latest-api-documents-and-uart-protocols/534/8

// Baud is 115200,8,N,1, no flow control.
// Source https://forum.arylic.com/t/latest-api-documents-and-uart-protocols/534/5

//use core::str;

use core::str::FromStr;

use embedded_hal::serial::{Read, Write};

use arrayvec::{ArrayString, ArrayVec};

mod error;
mod parameter_types;

use crate::error::Error;
use crate::parameter_types::{Bass, Treble, Volume};

const MAX_SIZE_RESPONSE: usize = 1024;

// Commands
const COMMAND_VER: &str = "VER";
const COMMAND_STATUS: &str = "STA";
const COMMAND_SYSTEM_CONTROL: &str = "SYS";
const COMMAND_WWW: &str = "WWW";

const COMMAND_DELIMITER: char = ';';
const COMMAND_PARAMETER_START: char = ':';
const TERMINATOR: char = '\n';

pub struct Up2Stream<'a, UART: Read<u8> + Write<u8>> {
    uart: &'a mut UART,
    //rx_buffer: &'r [u8; MAX_SIZE_RESPONSE],
    response: ArrayString<MAX_SIZE_RESPONSE>,
}

impl<'a, UART> Up2Stream<'a, UART>
where
    UART: Write<u8> + Read<u8>,
{
    /// Create a new Up2Stream driver from an object that inplements the read and write traits.
    //pub fn new(uart_reader: R, uart_writer: W) -> Up2Stream<R, W> {
    //pub fn new(uart: impl Read<u8> + Write<u8>) -> Up2Stream<UART> {
    pub fn new(uart: &mut UART) -> Up2Stream<UART> {
        Up2Stream {
            uart,
            response: ArrayString::<MAX_SIZE_RESPONSE>::new(),
            //rx_buffer: &mut [0; MAX_SIZE_RESPONSE],
        }
    }

    /// Get the device firmware version as a string in the form
    /// {firmware}-{commit}-{api}
    pub fn firmware_version(&mut self) -> Result<&str, Error> {
        self.response = self
            .send_query(COMMAND_VER)
            .map_err(|_| Error::SendCommand)?;

        let s = self.response.as_str();

        Ok(s)
    }

    pub fn status(&mut self) -> Result<DeviceStatus, Error> {
        // Response is local to this function as return a device status and not a string slice
        let response = self
            .send_query(COMMAND_STATUS)
            .map_err(|_| Error::SendCommand)?;

        let status_fields: ArrayVec<&str, 20> = response.split(&[':', ',']).collect();
        // The first field [0] contains the command

        let device_status = DeviceStatus {
            source: Source::from_str(status_fields[1])?,
            mute: boolean_from_str(status_fields[2])?,
            volume: Volume::from_str(status_fields[3])?,
            treble: Treble::from_str(status_fields[4])?,
            bass: Bass::from_str(status_fields[5])?,
            net: boolean_from_str(status_fields[6])?,
            internet: boolean_from_str(status_fields[7])?,
            playing: boolean_from_str(status_fields[8])?,
            led: boolean_from_str(status_fields[9])?,
            upgrading: boolean_from_str(status_fields[10])?,
        };

        Ok(device_status)
    }

    pub fn execute_system_control(&mut self, control: SystemControl) -> Result<(), Error> {
        let parameter = control.into_parameter_str();

        self.send_command(COMMAND_SYSTEM_CONTROL, parameter.as_str())?;

        Ok(())
    }

    pub fn internet_connection(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_WWW)?;

        // Response is in the form WWW:{status}\n", so first extract the status
        let start = COMMAND_WWW.len() + 1;
        let end = start + 1;
        if let Some(s) = response.get(start..end) {
            Ok(boolean_from_str(s)?)
        } else {
            Err(Error::IllFormedReponse)
        }
    }
    /*


        pub fn audio_out_enabled(&self) -> bool {}

        pub fn enable_audio_out(&self, enable; bool) {}

        pub fn input_source(&self) -> Source {}

        pub fn select_input_source(&self, source: Source) {}

        pub fn volume(&self) -> Volume {}

        pub fn set_volume(&self, volume: Volume) {}

        pub fn mute_status(&self) {}

        pub fn set_mute(&self, on_off: bool) {}

        pub fn bass(&self) -> Bass {}

        pub fn set_bass(&self, bass: Bass) {}

        pub fn treble(&self) -> Treble {}

        pub fn set_treble(&self, treble: Treble) {}

        //TODO could not invalid states be implemented using the type state pattern?
        pub fn play_pause_toggle(&self) -> Result<(), Error> {


            use Source::*;
            match self.input_source() {
                Bluetooth | Net |Usb => Ok(()) , // TODO Send the command,

                _ =>  Err(Error::NotSupportedForDeviceSource),
            }

        }

        pub fn stop(&self) {}

        pub fn next(&self) {}

        pub fn previous(&self) {}

        pub fn bluetooth_connected(&self) -> bool {}

        pub fn connect_bluetooth(&self) -> Result<(), Error> {}

        pub fn disconnect_bluetooth(&self) -> Result<(), Error> {}

        pub fn playback_status(&self) -> Playback {}

        pub fn audio_channel(&self) -> AudioChannel {}

        pub fn multiroom_state(&self) -> MultiroomState {}

        pub fn set_multiroom_state(&self, state: MultiroomState) {}

        pub fn led(&self)  -> Led {}

        pub fn set_led(&self, status: Led) {}

        pub fn beep(&self) -> bool {}

        pub fn set_beep(&self, beep: bool) {}

        pub fn set_play_preset(&self, preset: PlayPreset) {}

        pub fn virtual_bass(&self) -> bool {}

        pub fn enable_virtual_bass(&self) {}

        pub fn disable_virtual_bass(&self) {}

        pub fn toggle_virtual_bass(&self) {}

        pub fn reset_wifi(&self) {}

        pub fn loop_mode(&self) -> LoopMode {}

        pub fn set_loop_mode(&self, loop_mode: LoopMode) {}

        pub fn device_name(&self) -> &str {}

        pub fn set_device_name(&self, device_name: &str) {}

        pub fn enternet_connection(&self) -> bool {}

        pub fn wifi_connection(&self) -> bool {}
    */

    //TODO more commands for version 4 here https://docs.google.com/spreadsheets/d/1LT6nsaCmg2B6vV0M2iOusxZ-hIqgDeqB0SLPTtZokCo/edit#gid=1444188925

    fn send_command(&mut self, command: &str, parameter: &str) -> Result<(), Error> {
        for c in command.chars() {
            self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
        }

        if !parameter.is_empty() {
            self.uart
                .write(COMMAND_PARAMETER_START as u8)
                .map_err(|_| Error::SendCommand)?;
            for c in parameter.chars() {
                self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
            }
        }

        self.uart
            .write(COMMAND_DELIMITER as u8)
            .map_err(|_| Error::SendCommand)?;

        self.uart
            .write(TERMINATOR as u8)
            .map_err(|_| Error::SendCommand)?;

        self.uart.flush().map_err(|_| Error::SendCommand)?;

        Ok(())
    }

    fn send_query(&mut self, command: &str) -> Result<ArrayString<MAX_SIZE_RESPONSE>, Error> {
        //self.send_command(command, "")?;

        for c in command.chars() {
            self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
        }
        self.uart.write(b'\n').map_err(|_| Error::SendCommand)?;

        self.uart.flush().map_err(|_| Error::SendCommand)?;

        let mut response = ArrayString::<MAX_SIZE_RESPONSE>::new();

        //let c: char = ' ';
        let mut terminator: [u8; 1] = [0; 1];
        TERMINATOR.encode_utf8(&mut terminator);
        let mut read_byte;
        loop {
            read_byte = self.uart.read().map_err(|_| Error::ReadingQueryReponse)?;
            if read_byte == terminator[0] {
                break;
            }
            response.push(read_byte as char);
        }

        Ok(response)
    }
}

// #[derive(Debug)]
// pub enum Error {
//     NotSupportedForDeviceSource,
//     ReadingQueryReponse,
//     NonUTF8,
//     SendCommand,
//     SourceNotKnown,
//     BooleanParse,
//     OutOfRange,
//     InvalidString,
//     Unimplemented,
// }

#[derive(Debug, PartialEq)]
pub struct DeviceStatus {
    source: Source,
    mute: bool,
    volume: Volume,
    treble: Treble,
    bass: Bass,
    net: bool,
    internet: bool,
    playing: bool,
    led: bool,
    upgrading: bool,
}

// #[derive(Debug, PartialEq)]
// struct Treble(i8); //-10..10
// impl Treble {
//     fn new(treble: i8) -> Result<Self, Error> {
//         let range = -10..10;
//         if range.contains(&treble) {
//             Ok(Self(treble))
//         } else {
//             Err(Error::OutOfRange)
//         }
//     }
// }

// impl FromStr for Treble {
//     type Err = Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let treble = s.parse::<i8>().map_err(|_| Error::InvalidString)?;
//         Ok(Self(treble))
//     }
// }

// #[derive(Debug, PartialEq)]
// struct Bass(i8); //-10..10
// impl Bass {
//     fn new(bass: i8) -> Result<Self, Error> {
//         let range = -10..10;
//         if range.contains(&bass) {
//             Ok(Self(bass))
//         } else {
//             Err(Error::OutOfRange)
//         }
//     }
// }

// impl FromStr for Bass {
//     type Err = Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let bass = s.parse::<i8>().map_err(|_| Error::InvalidString)?;
//         Ok(Self(bass))
//     }
// }

// struct PlayPreset(u8); // 0..10
// impl PlayPreset {
//     fn new(preset: u8) -> Result<Self, Error> {
//         let range = 0..10;
//         if range.contains(&preset) {
//             Ok(Self(preset))
//         } else {
//             Err(Error::OutOfRange)
//         }
//     }
// }

// impl FromStr for PlayPreset {
//     type Err = Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let preset = s.parse::<u8>().map_err(|_| Error::InvalidString)?;
//         Ok(Self(preset))
//     }
// }

pub enum SystemControl {
    Reboot,
    Standby,
    Reset,
    Recover,
}

impl SystemControl {
    pub fn into_parameter_str(&self) -> ArrayString<8> {
        let parameter = match self {
            Self::Reboot => "REBOOT",
            Self::Standby => "STANDBYE",
            Self::Reset => "RESET",
            Self::Recover => "RECOVER",
        };

        // This should infallible as the string input
        // is constrained to the above
        ArrayString::from_str(parameter).unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub enum Source {
    Net,
    Usb,
    UsbDac,
    LineIn,
    LineIn2,
    Bluetooth,
    Optical,
    Coax,
    I2S,
    HDMI,
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(source_str: &str) -> Result<Source, Error> {
        match source_str {
            "NET" => Ok(Source::Net),
            "USB" => Ok(Source::Usb),
            "USBDAC" => Ok(Source::UsbDac),
            "LINE-IN" => Ok(Source::LineIn),
            "LINE-IN2" => Ok(Source::LineIn2),
            "BT" => Ok(Source::Bluetooth),
            "OPT" => Ok(Source::Optical),
            "COAX" => Ok(Source::Coax),
            "I2S" => Ok(Source::I2S),
            "HDMI" => Ok(Source::HDMI),
            _ => Err(Error::SourceNotKnown), // "USB", "USBDAC", "LINE-IN", "LINE-IN2", "BT", "OPT", "COAX", "I2S", "HDMI",
        }
    }
}

pub enum Playback {
    Playing,
    NotPlaying,
}

pub enum AudioChannel {
    Left,
    Right,
    Silent, // ???
}

pub enum MultiroomState {
    Slave,
    Master,
    None,
}

pub enum Led {
    On,
    Off,
    Toogle,
}

pub enum LoopMode {
    RepeatAll,
    RepeatOne,
    RepeatShuffle,
    Shuffle,
    Sequence,
}

fn boolean_from_str(s: &str) -> Result<bool, Error> {
    match s {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(Error::BooleanParse),
    }
}

#[cfg(test)]
mod test_api;
