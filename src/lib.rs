//! A driver for the Arylic Up2Steam board. This only uses the UART interface.
//!

//#![no_std]
// DO not include the standard library, except when testing.
#![cfg_attr(not(test), no_std)]
//#![no_main]
#![allow(dead_code)]

// See API description:
// https://docs.google.com/spreadsheets/d/1gOb4VBruyJgaBZJHClV6dEkoiwf5hkzE/edit?pli=1#gid=425762154

// According to this https://github.com/Resinchem/Arylic-Amp-MQTT/blob/main/src/arylic_amp.ino
// the commands are terminated with ';' and query responses are terminated with '\n'
// This is confirmed here https://forum.arylic.com/t/latest-api-documents-and-uart-protocols/534/8

// Baud is 115200,8,N,1, no flow control.
// Source https://forum.arylic.com/t/latest-api-documents-and-uart-protocols/534/5

//use nb::block;

use core::str::FromStr;

use embedded_hal::serial::{Read, Write};

use arrayvec::{ArrayString, ArrayVec};

// TODO consider this for error type conversion: https://doc.rust-lang.org/std/convert/trait.From.html
mod error;
mod parameter_types;

use crate::error::Error;
use crate::parameter_types::{
    AudioChannel, Bass, DeviceStatus, Led, LoopMode, MultiroomState, PlayPreset, Playback, Source,
    Switch, SystemControl, Treble, Volume,
};

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

const TERMINATOR: u8 = b';';
const PARAMETER_START: u8 = b':';
const PARAMETER_DELIMITER: u8 = b',';

pub struct Up2Stream<'a, UART: Read<u8> + Write<u8>> {
    uart: &'a mut UART,

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
            //.send_query(COMMAND_VER)
            //.map_err(|_| Error::SendCommand)?;
            .send_query(COMMAND_VER)?;

        let s = self.response.as_str();

        Ok(s)
    }

    pub fn status(&mut self) -> Result<DeviceStatus, Error> {
        // Response is local to this function as return a device status and not a string slice
        let response = self
            .send_query(COMMAND_STATUS)
            .map_err(|_| Error::SendCommand)?;

        //let status_fields: ArrayVec<&str, 20> = response.split(&[':', ',']).collect();
        let status_fields: ArrayVec<&str, 20> = response.split(&[',']).collect();

        let device_status = DeviceStatus {
            source: Source::from_str(status_fields[0])?,
            mute: Switch::from_str(status_fields[1])?.to_bool()?,
            volume: Volume::from_str(status_fields[2])?,
            treble: Treble::from_str(status_fields[3])?,
            bass: Bass::from_str(status_fields[4])?,
            net: Switch::from_str(status_fields[5])?.to_bool()?,
            internet: Switch::from_str(status_fields[6])?.to_bool()?,
            playing: Switch::from_str(status_fields[7])?.to_bool()?,
            led: Switch::from_str(status_fields[8])?.to_bool()?,
            upgrading: Switch::from_str(status_fields[9])?.to_bool()?,
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

        if response.len() != 1 {
            return Err(Error::IllFormedReponse);
        }

        let internet_connection_status = Switch::from_str(response.as_str())?;

        internet_connection_status.to_bool()
    }

    /// Get if audio out has been enabled.
    pub fn audio_out(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_AUD)?;

        if response.len() > 1 {
            return Err(Error::IllFormedReponse);
        }

        Switch::from_str(response.as_str())?.to_bool()
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

        let source = Source::from_str(response.as_str())?;
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

        let volume = Volume::from_str(response.as_str())?;

        // Response is in the form VOL:{vol}", so first extract the volume
        // let parts: ArrayVec<&str, 2> = response.split(':').collect();

        // let volume = Volume::from_str(parts[1])?;

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

        let mute_status = Switch::from_str(response.as_str())?;

        mute_status.to_bool()
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
    pub fn set_bass(&mut self, _bass: Bass) -> Result<(), Error> {
        todo!();
    }
    pub fn treble(&mut self) -> Result<Treble, Error> {
        todo!();
    }

    pub fn set_treble(&mut self, _treble: Treble) -> Result<(), Error> {
        todo!();
    }

    pub fn play_pause_toggle(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn stop(&self) -> Result<(), Error> {
        todo!()
    }
    pub fn next(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn previous(&self) -> Result<(), Error> {
        todo!()
    }
    pub fn bluetooth_connected(&self) -> Result<bool, Error> {
        todo!()
    }
    pub fn connect_bluetooth(&self) -> Result<(), Error> {
        todo!()
    }
    pub fn disconnect_bluetooth(&self) -> Result<(), Error> {
        todo!()
    }
    pub fn playback_status(&self) -> Result<Playback, Error> {
        todo!()
    }
    pub fn audio_channel(&self) -> Result<AudioChannel, Error> {
        todo!()
    }
    pub fn multiroom_state(&self) -> Result<MultiroomState, Error> {
        todo!()
    }
    pub fn set_multiroom_state(&mut self, _state: MultiroomState) -> Result<(), Error> {
        todo!();
    }
    pub fn led(&self) -> Result<Led, Error> {
        todo!()
    }
    pub fn set_led(&mut self, _led_status: Led) -> Result<(), Error> {
        todo!();
    }
    pub fn beep(&self) -> Result<bool, Error> {
        todo!();
    }

    pub fn set_beep(&self, _beep: bool) -> Result<(), Error> {
        todo!()
    }

    pub fn set_play_preset(&self, _preset: PlayPreset) -> Result<(), Error> {
        todo!();
    }
    pub fn virtual_bass(&self) -> Result<bool, Error> {
        todo!();
    }

    pub fn enable_virtual_bass(&self) -> Result<(), Error> {
        todo!();
    }

    pub fn disable_virtual_bass(&self) -> Result<(), Error> {
        todo!();
    }

    pub fn toggle_virtual_bass(&self) -> Result<(), Error> {
        todo!();
    }
    pub fn reset_wifi(&self) -> Result<(), Error> {
        todo!();
    }
    pub fn loop_mode(&self) -> Result<LoopMode, Error> {
        todo!();
    }
    pub fn set_loop_mode(&self, _loop_mode: LoopMode) -> Result<(), Error> {
        todo!();
    }
    pub fn device_name(&self) -> Result<&str, Error> {
        todo!();
    }

    pub fn set_device_name(&self, _device_name: &str) -> Result<LoopMode, Error> {
        todo!();
    }
    pub fn enternet_connection(&self) -> Result<bool, Error> {
        todo!();
    }

    pub fn wifi_connection(&self) -> Result<bool, Error> {
        todo!();
    }

    //    ******* TODO more commands for version 4 here https://docs.google.com/spreadsheets/d/1LT6nsaCmg2B6vV0M2iOusxZ-hIqgDeqB0SLPTtZokCo/edit#gid=1444188925

    // Send a command with any specified parameters.
    // Commands are send as bytes with the following syntax (BNF)
    //
    //    <COMMAND> = <COMMAND_NAME> ";" | <COMMAND_NAME> ":" <PARAMETER> ";"
    fn send_command(&mut self, command: &str, parameter: &[u8]) -> Result<(), Error> {
        // First write a terminator character. This resets the channel.
        // self.uart
        //     .write(TERMINATOR)
        //     .map_err(|_| Error::SendCommand)?;

        // Now send the command characters
        for c in command.chars() {
            self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
        }

        // Send parameters if availble
        if !parameter.is_empty() {
            self.uart
                .write(PARAMETER_START)
                .map_err(|_| Error::SendCommand)?;
            for c in parameter {
                self.uart.write(*c).map_err(|_| Error::SendCommand)?;
            }
        }

        // Send termination character
        self.uart
            .write(TERMINATOR)
            .map_err(|_| Error::SendCommand)?;

        self.uart.flush().map_err(|_| Error::SendCommand)?;

        Ok(())
    }

    // Send a query and read the response. Queries are sent with the following syntax (BNF):
    //   <query> ::= <command> <terminator>
    //   <command> ::= <alphanumeric> | <command>
    //   <terminator> ::= ";"
    //
    // The response has the following syntax:
    //  <response> ::= <command> <parameter_start> <parameter_list> <terminator>
    //  <parameter_list> ::= <parameter> <parameter_delimiter> <parameter_list> | <parameter>
    //  <parameter_start> = ":"
    //  <parameter_delimiter> ::= ","
    //
    // In reality, a response  can be other characters due to framing issues, start up messages etc. so the "real"
    // response grammmer looks like:
    //  <response> ::= <noise> <command> <parameter_start> <parameter_list> <terminator> <noise> <EOR>
    //  <parameter_list> ::= <parameter> <parameter_delimiter> <parameter_list> | <parameter>
    //  <noise >::= <control_character> | <character> | <noise>
    //  <control_char> ::=   "\n" | "\r"
    //  <character> is any printable character
    //  <EOR> is the end of record as indicated with a blocking read.
    //
    fn send_query(&mut self, command: &str) -> Result<ArrayString<MAX_SIZE_RESPONSE>, Error> {
        const MAX_NUMBER_RESENDS: u8 = 3;

        let mut query_response = ArrayString::<MAX_SIZE_RESPONSE>::new();

        // First write a terminator character. This resets the channel.
        // TODO do we still need to do this?
        // self.uart
        //     .write(TERMINATOR)
        //     .map_err(|_| Error::SendCommand)?;

        let mut number_resends: u8 = 0;
        let mut resend_command = false;

        loop {
            // Send (or resend) the command characters
            for c in command.chars() {
                self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
            }

            self.uart
                .write(TERMINATOR)
                .map_err(|_| Error::SendCommand)?;

            //self.uart.flush().map_err(|_| Error::SendCommand)?;

            #[cfg_attr(not(test), derive(defmt::Format))] // Only used when running on target hardware
            enum TokenType {
                Character(u8),
                EOR,
                ControlCharacter,
                Terminator,
                ParameterStart,
                ParameterDelimiter,
            }

            #[cfg_attr(not(test), derive(defmt::Format))] // Only used when running on target hardware
            #[derive(Clone, Copy)]
            enum ParseState {
                Command,
                ValidatedCommand,
                Parameter,
                ValidatedParameter,
            }

            let mut state = ParseState::Command;
            let mut command_string_index = 0;

            // Read and parse the response
            loop {
                let token = match self.uart.read() {
                    Ok(c) if c.is_ascii_alphanumeric() => Ok(TokenType::Character(c)),
                    Ok(c) if c == b'-' => Ok(TokenType::Character(c)), // Occurs in the version number and negative numbers
                    Ok(c) if c.is_ascii_control() => Ok(TokenType::ControlCharacter),
                    Ok(c) if c == TERMINATOR => Ok(TokenType::Terminator),
                    Ok(c) if c == PARAMETER_START => Ok(TokenType::ParameterStart),
                    Ok(c) if c == PARAMETER_DELIMITER => Ok(TokenType::ParameterDelimiter),
                    // Other characters should not occur
                    Ok(_) => Err(Error::Read),
                    // Assuming that Err(WouldBlock) is an end of record.
                    Err(nb::Error::WouldBlock) => Ok(TokenType::EOR),
                    // Read error condition
                    Err(nb::Error::Other(_e)) => Err(Error::Read),
                }?;

                match (state, token) {
                    (ParseState::Command, TokenType::Character(c)) => {
                        if c == command.as_bytes()[command_string_index] {
                            command_string_index += 1;
                            if command_string_index != command.len() {
                                state = ParseState::Command;
                            } else {
                                state = ParseState::ValidatedCommand;
                            };
                        };
                    }
                    (ParseState::Command, TokenType::EOR) => {
                        resend_command = true;
                        break;
                    }
                    (ParseState::Command, _) => {
                        command_string_index = 0;
                        state = ParseState::Command;
                    }
                    (ParseState::ValidatedCommand, TokenType::ParameterStart) => {
                        state = ParseState::Parameter
                    }
                    (ParseState::ValidatedCommand, TokenType::EOR) => {
                        resend_command = true;
                        break;
                    }
                    (ParseState::ValidatedCommand, _) => return Err(Error::ParseResponseError),
                    (ParseState::Parameter, TokenType::Character(c)) => {
                        query_response.push(c as char)
                    }

                    // Currently not seperating parameters and just treating them all as a string.
                    (ParseState::Parameter, TokenType::ParameterDelimiter) => {
                        query_response.push(PARAMETER_DELIMITER as char)
                    }
                    (ParseState::Parameter, TokenType::Terminator) => {
                        state = ParseState::ValidatedParameter
                    }
                    (ParseState::Parameter, TokenType::EOR) => {
                        resend_command = true;
                        break;
                    }
                    (ParseState::Parameter, _) => return Err(Error::ParseResponseError),
                    (ParseState::ValidatedParameter, TokenType::EOR) => break, // Stop reading
                    (ParseState::ValidatedParameter, _) => continue, // Mop up any noise at the end
                }

                // match state {
                //     ParseState::Command => match token {
                //         TokenType::Character(c)
                //             if c == command.as_bytes()[command_string_index] =>
                //         {
                //             command_string_index += 1;
                //             if command_string_index != command.len() {
                //                 state = ParseState::Command;
                //             } else {
                //                 state = ParseState::ValidatedCommand;
                //             };
                //         }
                //         TokenType::EOR => {
                //             resend_command = true;
                //             break;
                //         }
                //         _ => {
                //             command_string_index = 0;
                //             state = ParseState::Command;
                //         }
                //     },
                //     ParseState::ValidatedCommand => match token {
                //         TokenType::ParameterStart => state = ParseState::Parameter,
                //         TokenType::EOR => {
                //             resend_command = true;
                //             break;
                //         } // Retry
                //         _ => return Err(Error::ParseResponseError),
                //     },
                //     ParseState::Parameter => match token {
                //         TokenType::Character(c) => query_response.push(c as char),
                //         // Currently not seperating parameters and just treating them all as a string.
                //         TokenType::ParameterDelimiter => {
                //             query_response.push(PARAMETER_DELIMITER as char)
                //         }
                //         TokenType::Terminator => state = ParseState::ValidatedParameter,
                //         TokenType::EOR => {
                //             resend_command = true;
                //             break;
                //         } // Retry
                //         _ => return Err(Error::ParseResponseError),
                //     },
                //     ParseState::ValidatedParameter => match token {
                //         TokenType::EOR => break, // Stop reading
                //         _ => continue,           // Mop up any noise at the end
                //     },
                // }
            }

            if resend_command {
                if number_resends < MAX_NUMBER_RESENDS {
                    number_resends += 1;
                    continue;
                } else {
                    return Err(Error::Timeout);
                }
            } else {
                break;
            }
        }

        Ok(query_response)
    }
}

#[cfg(test)]
mod test_api;
