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

use core::str::FromStr;

use embedded_hal::serial::{Read, Write};

use arrayvec::{ArrayString, ArrayVec};

// TODO consider this for error type conversion: https://doc.rust-lang.org/std/convert/trait.From.html
mod error;
mod parameter_types;

use crate::error::Error;
use crate::parameter_types::{Bass, DeviceStatus, Source, Switch, SystemControl, Treble, Volume};

const MAX_SIZE_RESPONSE: usize = 1024;

// Commands
const COMMAND_VER: &str = "VER";
const COMMAND_STATUS: &str = "STA";
const COMMAND_SYSTEM_CONTROL: &str = "SYS";
const COMMAND_WWW: &str = "WWW";
const COMMAND_AUD: &str = "AUD";
const COMMAND_SRC: &str = "SRC";
const COMMAND_VOL: &str = "VOL";
const COMMAND_MUT: &str = "MUT";
const COMMAND_BAS: &str = "BAS";
const COMMAND_TRE: &str = "TRE";
const COMMAND_POP: &str = "POP";
const COMMAND_STP: &str = "STP";
const COMMAND_NXT: &str = "NXT";
const COMMAND_PRE: &str = "PRE";
const COMMAND_BTC: &str = "BTC";
const COMMAND_PLA: &str = "PLA";
const COMMAND_CHN: &str = "CHN";
const COMMAND_MRM: &str = "MRM";
const COMMAND_LED: &str = "LED";
const COMMAND_BEP: &str = "BEP";
const COMMAND_PST: &str = "PST";
const COMMAND_VBS: &str = "VBS";
const COMMAND_WRS: &str = "WRS";
const COMMAND_LPM: &str = "LPM";
const COMMAND_NAM: &str = "NAM";
const COMMAND_ETH: &str = "ETH";
const COMMAND_WIF: &str = "WIF";

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
            mute: Switch::from_str(status_fields[2])?.to_bool()?,
            volume: Volume::from_str(status_fields[3])?,
            treble: Treble::from_str(status_fields[4])?,
            bass: Bass::from_str(status_fields[5])?,
            net: Switch::from_str(status_fields[6])?.to_bool()?,
            internet: Switch::from_str(status_fields[7])?.to_bool()?,
            playing: Switch::from_str(status_fields[8])?.to_bool()?,
            led: Switch::from_str(status_fields[9])?.to_bool()?,
            upgrading: Switch::from_str(status_fields[10])?.to_bool()?,
        };

        Ok(device_status)
    }

    pub fn execute_system_control(&mut self, control: SystemControl) -> Result<(), Error> {
        let mut buf = [0; 64];

        self.send_command(COMMAND_SYSTEM_CONTROL, control.as_parameter_str(&mut buf))?;

        Ok(())
    }

    pub fn internet_connection(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_WWW)?;

        // Response is in the form WWW:{status}\n", so first extract the status
        let start = COMMAND_WWW.len() + 1;
        let end = start + 1;
        if let Some(s) = response.get(start..end) {
            Switch::from_str(s)?.to_bool()
        } else {
            Err(Error::IllFormedReponse)
        }
    }

    /// Get if audio out has been enabled.
    pub fn audio_out(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_AUD)?;

        // Response is in the form AUD:{0/1}\n", so first extract the status
        let start = COMMAND_AUD.len() + 1;
        let end = start + 1;
        if let Some(s) = response.get(start..end) {
            Switch::from_str(s)?.to_bool()
        } else {
            Err(Error::IllFormedReponse)
        }
    }

    pub fn set_audio_out(&mut self, enable: bool) -> Result<(), Error> {
        let switch = Switch::from(enable);

        let mut buf = [0; 1];

        self.send_command(COMMAND_AUD, switch.as_parameter_str(&mut buf))
    }

    /// Get the current input source.
    /// #Example
    /// ```ignore
    /// let source: Source = driver.input_source().unwrap();
    /// match source {
    /// Source::Bluetooth => todo!(),
    /// _ => todo!(),
    /// }
    /// ```
    pub fn input_source(&mut self) -> Result<Source, Error> {
        let response = self.send_query(COMMAND_SRC)?;

        // A response is in the form
        // SRC:{source string}
        let parts: ArrayVec<&str, 2> = response.split(':').collect();
        let source = Source::from_str(parts[1])?;
        Ok(source)
    }

    /// Select the input source.
    /// The source is constrained by the values in Source.
    /// #Example:
    /// ```ignore
    /// driver.select_input_source(Source::Bluetooth).unwrap();
    /// ```
    pub fn select_input_source(&mut self, source: Source) -> Result<(), Error> {
        let mut buf = [0; 20];
        self.send_command(COMMAND_SRC, source.as_parameter_str(&mut buf))
    }

    /// Get the current volume, e.g:
    /// ```ignore
    /// let volume: Volume = driver.volume().unwrap();
    /// let vol_value: u8 = volume.get();
    ///
    /// ```
    pub fn volume(&mut self) -> Result<Volume, Error> {
        let response = self.send_query(COMMAND_VOL)?;

        // Response is in the form VOL:{vol}", so first extract the volume
        let parts: ArrayVec<&str, 2> = response.split(':').collect();

        let volume = Volume::from_str(parts[1])?;

        Ok(volume)
    }

    /// Set the volume, e.g,
    /// ```ignore
    /// let volume = Volume::new(52).unwrap();
    /// driver.set_volume(volume).unwrap();
    /// ```
    pub fn set_volume(&mut self, volume: Volume) -> Result<(), Error> {
        let mut buf = [0; 3];
        self.send_command(COMMAND_VOL, volume.as_parameter_str(&mut buf))
    }

    /// Get if the audio is muted or not. .
    pub fn mute_status(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_MUT)?;

        // Response is in the form MUT:{0/1}\n", so first extract the status
        let parts: ArrayVec<&str, 2> = response.split(':').collect();

        let status = Switch::from_str(parts[1])?.to_bool()?;

        Ok(status)
    }

    /// Mute or unmute the audio.
    /// # Examples
    /// To mute:
    /// ```ignore
    /// device.set_mute(Switch::On).unwrap();
    /// ```
    /// To toggle the mute status:
    /// ```ignore
    /// device.set_mute(Switch::Toggle).unwrap();
    /// ```
    pub fn set_mute(&mut self, switch: Switch) -> Result<(), Error> {
        let mut buf = [0; 1];
        self.send_command(COMMAND_MUT, switch.as_parameter_str(&mut buf))
    }

    /// Get the bass value, e.g.;
    /// ```ignore
    /// let bass: Bass = driver.bass().unwrap();
    /// ```
    pub fn bass(&mut self) -> Result<Bass, Error> {
        todo!();
    }
    /*




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

    fn send_command(&mut self, command: &str, parameter: &[u8]) -> Result<(), Error> {
        for c in command.chars() {
            self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
        }

        if !parameter.is_empty() {
            self.uart
                .write(COMMAND_PARAMETER_START as u8)
                .map_err(|_| Error::SendCommand)?;
            for c in parameter {
                self.uart.write(*c).map_err(|_| Error::SendCommand)?;
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

#[cfg(test)]
mod test_api;
