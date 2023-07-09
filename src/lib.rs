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

use embedded_hal::serial::{Read, Write};

use arrayvec::ArrayString;

const MAX_SIZE_RESPONSE: usize = 1024;

// Commands
const COMMAND_VER: &str = "VER";

const COMMAND_TERMINATOR: char = ';';
const COMMAND_PARAMETER_START: char = ':';
const RESPONSE_TERMINATOR: char = '\n';

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

        let s = self.response.as_str().clone();

        Ok(s)
    }

    /*
        pub fn status(&self) -> DeviceStatus {}

        pub fn execute_system_control(&self, control: SystemControl) {}

        pub fn internet_connection(&self) -> bool {}

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

        if parameter.len() > 0 {
            self.uart
                .write(COMMAND_PARAMETER_START as u8)
                .map_err(|_| Error::SendCommand)?;
            for c in parameter.chars() {
                self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
            }
        }

        self.uart
            .write(COMMAND_TERMINATOR as u8)
            .map_err(|_| Error::SendCommand)?;

        self.uart.flush().map_err(|_| Error::SendCommand)?;

        Ok(())
    }

    fn send_query(&mut self, command: &str) -> Result<ArrayString<MAX_SIZE_RESPONSE>, Error> {
        self.send_command(command, "")?;

        // let mut rx_buffer: [u8; MAX_SIZE_RESPONSE] = [0; MAX_SIZE_RESPONSE];
        // let mut rx_count: usize = 0;
        // let c: char = ' ';

        // while c != COMMAND_TERMINATOR {
        //     let c = self
        //         .uart_reader
        //         .read()
        //         .map_err(|e| Error::ReadingQueryReponse)?;
        //     rx_buf[rx_count] = c;
        //     rx_count += 1;
        // }

        // Ok(rx_count)

        let mut response = ArrayString::<MAX_SIZE_RESPONSE>::new();

        let c: char = ' ';
        while c != RESPONSE_TERMINATOR {
            let c = self.uart.read().map_err(|_| Error::ReadingQueryReponse)?;
            response.push(c as char);
        }

        Ok(response)
    }
}

#[derive(Debug)]
pub enum Error {
    NotSupportedForDeviceSource,
    ReadingQueryReponse,
    NonUTF8,
    SendCommand,
}

pub struct DeviceStatus<'d> {
    source: &'d str,
    mute: bool,
    volume: Volume,
    treble: Treble,
    bass: Bass,
    net: &'d str,
    internet: &'d str,
    led: bool,
    upgrading: bool,
}

struct Volume(u8); //0..100
struct Treble(i8); //-10..10
struct Bass(i8); //-10..10

struct PlayPreset(u8); // 0..10

pub enum SystemControl {
    Reboot,
    Standby,
    Reset,
}

pub enum Source {
    Net,
    Usb,
    UsbDac,
    Linein,
    Bluetooth,
    Optical,
    Coax,
    I2S,
    HDMI,
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

#[cfg(test)]
mod test;
