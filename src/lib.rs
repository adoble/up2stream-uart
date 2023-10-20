//! This crate provides a UART driver for the **Arylic Up2Steam Pro** board.
//!
//! It provides a subset of the functionality provided by the UART interface to the board.
//!
//! The public API is available as functions exposed by the [Up2Stream] struct.
//!
//! The main driver is created using `up2stream_uart::Up2Stream::new` which accepts
//! an UART peripheral that implements the `embedded_hal::serial::{Read, Write}` traits. Tne UART
//! connection is configured as `115200,8,N,1` with no flow control
//!
//! Values are not set directly, but through the use of either enums or scalar types (such as [Volume] or [Bass])
//! that provide range constraints on the values (for instance `Bass` can only accept values between -10 and +10 inclusive).
//! If a value is out of range an error is returned. As such, type safety is ensured.
//!
//! # Example
//!
//! Gets the current volume and sets it to a lower level.
//! ```
//! # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
//! use up2stream_uart::{Up2Stream, Volume, ScalarParameter, Error};
//! # fn main() -> Result<(), up2stream_uart::Error> {
//! # let initial_expectations = [
//! # SerialTransaction::write(b';'),
//! # SerialTransaction::write_many(b"VOL:50;"),
//! # SerialTransaction::write_many(b"VOL;"),
//! # SerialTransaction::flush(),
//! # SerialTransaction::read_many(b"VOL:50;"),
//! # SerialTransaction::write_many(b"VOL:49;"),
//! # ];
//!
//! // Initialise a serial peripheral on your MCU that implements the traits
//! // `embedded_hal::serial::{Read, Write}`. This is assigned
//! // the variable `serial` in the code example
//! # let mut serial = SerialMock::new(&initial_expectations);
//!
//! // Initialise the drive using the previously setup serial peripheral
//! let mut up2stream_device = Up2Stream::new(&mut serial);
//!
//! // Set the initial volume
//! let initial_vol = Volume::new(50)?;
//! up2stream_device.set_volume(initial_vol)?;
//!
//! // Do some more processing ...
//!
//! // Get the volume from the device
//! let actual_volume: i8 = up2stream_device.volume()?.get();
//!
//! // Reduce the volume by 1 step
//! if actual_volume > 0 {
//!     let new_volume = Volume::new(actual_volume - 1)?;
//!     up2stream_device.set_volume(new_volume)?;
//! }
//!
//! # serial.done();
//! # Ok(())
//! # }
//! ```
//! # Restrictions
//! Currently only covers a subset of the UART API version 3.
//!
//!
//! # API description for the UART interface to the Up2Stream Pro.
//! The Arylic API for the UART  can be downloaded [here](https://developer.arylic.com/download/api-info-4.xlsx).
//!

// TODO Complete the functions marked todo()!
// TODO Provide some configuration for differences between firmware version 3 and 4

//#![no_std]
// DO not include the standard library, except when testing.
#![cfg_attr(not(test), no_std)]
//#![no_main]
#![allow(dead_code)]

use nb::block;

use core::str::FromStr;

use embedded_hal::serial::{Read, Write};

use arrayvec::{ArrayString, ArrayVec};

// TODO consider this for error type conversion: https://doc.rust-lang.org/std/convert/trait.From.html
mod error;
mod parameter_types;

pub use crate::error::Error;

/// Re-exports of parameter types
pub use crate::parameter_types::{
    AudioChannel, Bass, DeviceStatus, Led, LoopMode, MultiroomState, PlayPreset, Playback,
    ScalarParameter, Source, Switch, SystemControl, Treble, Volume,
};

// #[cfg(doctest)]
// use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};

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

/// The UART driver for the **Arylic Up2Stream Pro** board.
//pub struct Up2Stream<'a, UART: Read<u8> + Write<u8>> {
pub struct Up2Stream<UART: Read<u8> + Write<u8>> {
    uart: UART,

    response: ArrayString<MAX_SIZE_RESPONSE>,
}

//impl<'a, UART> Up2Stream<'a, UART>
impl<UART> Up2Stream<UART>
where
    UART: Write<u8> + Read<u8>,
{
    /// Create a new Up2Stream driver from an UART object that implements the `Read` and `Write` traits.
    pub fn new(mut uart: UART) -> Up2Stream<UART> {
        // This seems to be required by the device before usage.
        // It can fail, but the uart channel is then usable
        block!(uart.write(TERMINATOR)).ok();

        Up2Stream {
            uart,
            response: ArrayString::<MAX_SIZE_RESPONSE>::new(),
        }
    }

    /// Get the device firmware version as a string in the form
    /// {firmware}-{commit}-{api}.
    ///
    /// ```no_run
    /// use up2stream_uart::Up2Stream;
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let firmware : &str = up2stream_driver.firmware_version().unwrap();
    ///
    /// ```
    pub fn firmware_version(&mut self) -> Result<&str, Error> {
        self.response = self
            //.send_query(COMMAND_VER)
            //.map_err(|_| Error::SendCommand)?;
            .send_query(COMMAND_VER)?;

        let s = self.response.as_str();

        Ok(s)
    }

    /// Get the device status as a [DeviceStatus] struct.
    ///
    /// For example:
    ///
    /// ```no_run
    /// use up2stream_uart::Up2Stream;
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let device_status = up2stream_driver.status().unwrap();
    /// let source = device_status.source;
    ///
    /// ```
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

    /// Reset, reboot or put into standby the device.
    /// Use the parameter type [SystemControl] to decide what happens. For example, to reboot
    /// the device:
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, SystemControl};
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let device_status = up2stream_driver.execute_system_control(SystemControl::Reboot).unwrap();
    ///
    /// ```
    pub fn execute_system_control(&mut self, control: SystemControl) -> Result<(), Error> {
        let mut buf = [0; 64];

        self.send_command(COMMAND_SYSTEM_CONTROL, control.to_parameter_str(&mut buf))?;

        Ok(())
    }

    /// Get the status of the internet connection
    pub fn internet_connection(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_WWW)?;

        if response.len() != 1 {
            return Err(Error::IllFormedReponse);
        }

        let internet_connection_status = Switch::from_str(response.as_str())?;

        internet_connection_status.to_bool()
    }

    /// Get if audio output has been enabled.
    pub fn audio_out(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_AUD)?;

        if response.len() > 1 {
            return Err(Error::IllFormedReponse);
        }

        Switch::from_str(response.as_str())?.to_bool()
    }

    /// Enable or disable audio output. For instance:
    ///
    /// ```no_run
    /// use up2stream_uart::Up2Stream;
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// // Enable audio output
    /// up2stream_driver.set_audio_out(true).unwrap();
    ///
    /// ```
    pub fn set_audio_out(&mut self, enable: bool) -> Result<(), Error> {
        let switch = Switch::from(enable);

        let mut buf = [0; 1];

        self.send_command(COMMAND_AUD, switch.to_parameter_str(&mut buf))
    }

    /// Get the current input source.
    ///
    /// # Example
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Source};
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    ///
    /// let source: Source = up2stream_driver.input_source().unwrap();
    /// match source {
    ///   Source::Bluetooth => todo!(),
    ///    _ => todo!(),
    /// }
    /// ```
    pub fn input_source(&mut self) -> Result<Source, Error> {
        let response = self.send_query(COMMAND_SRC)?;

        let source = Source::from_str(response.as_str())?;
        Ok(source)
    }

    /// Select the input source.
    /// The source is constrained by the values in Source.
    ///
    /// #Example:
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Source};
    /// # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    /// # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    /// # let mut up2stream_driver =Up2Stream::new(&mut uart);
    ///
    /// up2stream_driver.select_input_source(Source::Bluetooth).unwrap();
    /// ```
    pub fn select_input_source(&mut self, source: Source) -> Result<(), Error> {
        let mut buf = [0; 20];
        self.send_command(COMMAND_SRC, source.to_parameter_str(&mut buf))
    }

    /// Get the current volume
    ///
    /// # Example
    /// ```no_run
    ///  use up2stream_uart::{Up2Stream, Volume, ScalarParameter};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let volume = up2stream_driver.volume().unwrap();
    /// // Now get the value of the volume
    /// let volume_value: i8 = volume.get();
    ///
    /// ```
    pub fn volume(&mut self) -> Result<Volume, Error> {
        let response = self.send_query(COMMAND_VOL)?;

        let volume = Volume::from_str(response.as_str())?;

        Ok(volume)
    }

    /// Set the volume.
    ///
    /// This uses the parameter type [Volume].
    ///
    /// # Example
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Volume, ScalarParameter};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let volume = Volume::new(52).unwrap();
    /// up2stream_driver.set_volume(volume).unwrap();
    /// ```
    pub fn set_volume(&mut self, volume: Volume) -> Result<(), Error> {
        let mut buf = [0; 3];
        self.send_command(COMMAND_VOL, volume.to_parameter_str(&mut buf))
    }

    /// Get if the audio is muted or not.
    pub fn mute_status(&mut self) -> Result<bool, Error> {
        let response = self.send_query(COMMAND_MUT)?;

        let mute_status = Switch::from_str(response.as_str())?;

        mute_status.to_bool()
    }

    /// Mute or unmute the audio.
    ///
    /// # Examples
    ///
    /// To mute:
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Switch};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// up2stream_driver.set_mute(Switch::On).unwrap();
    /// ```
    /// To toggle the mute status:
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Switch};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// up2stream_driver.set_mute(Switch::Toggle).unwrap();
    /// ```
    pub fn set_mute(&mut self, switch: Switch) -> Result<(), Error> {
        let mut buf = [0; 1];
        self.send_command(COMMAND_MUT, switch.to_parameter_str(&mut buf))
    }

    /// Get the bass value, e.g.;
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Bass};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let bass: Bass = up2stream_driver.bass().unwrap();
    ///
    /// ```
    pub fn bass(&mut self) -> Result<Bass, Error> {
        let response = self.send_query(COMMAND_BAS)?;

        let bass = Bass::from_str(response.as_str())?;

        Ok(bass)
    }

    /// Set the bass value. This uses the parameter type [Bass].
    ///
    /// # Example
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Bass};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let bass = Bass::new(-6).unwrap();
    /// up2stream_driver.set_bass(bass).unwrap();
    ///
    /// ```
    pub fn set_bass(&mut self, bass: Bass) -> Result<(), Error> {
        let mut buf = [0; 3];
        self.send_command(COMMAND_BAS, bass.to_parameter_str(&mut buf))
    }

    /// Get the treble value.
    ///
    /// # Example
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Treble, ScalarParameter};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    ///
    /// let treble = up2stream_driver.treble().unwrap();
    /// let treble_value : i8 = treble.get();
    ///
    /// ```
    pub fn treble(&mut self) -> Result<Treble, Error> {
        let response = self.send_query(COMMAND_TRE)?;

        let treble = Treble::from_str(response.as_str())?;

        Ok(treble)
    }

    /// Set the treble value. This uses the parameter type [Treble].
    ///
    /// # Example
    /// ```no_run
    /// use up2stream_uart::{Up2Stream, Treble};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    /// let treble = Treble::new(-6).unwrap();
    /// up2stream_driver.set_treble(treble).unwrap();
    ///
    /// ```
    pub fn set_treble(&mut self, treble: Treble) -> Result<(), Error> {
        let mut buf = [0; 3];
        self.send_command(COMMAND_TRE, treble.to_parameter_str(&mut buf))
    }

    /// Toggle between play and pause.
    ///
    /// # Example
    /// ```no_run
    ///  # use up2stream_uart::{Up2Stream};
    ///  # use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    ///  # let mut uart =   SerialMock::new(&[SerialTransaction::read(b';')]);
    ///  # let mut up2stream_driver =Up2Stream::new(&mut uart);
    ///
    ///  up2stream_driver.play_pause_toggle().unwrap();
    ///
    /// ```
    pub fn play_pause_toggle(&mut self) -> Result<(), Error> {
        self.send_command(COMMAND_POP, b"")
    }

    /// Stop playing.
    ///
    /// This is only available for Wifi or USB sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn stop(&mut self) -> Result<(), Error> {
        let source = self.input_source()?;

        match source {
            Source::Net | Source::Usb => self.send_command(COMMAND_STP, b""),
            _ => Err(Error::NotSupportedForDeviceSource),
        }
    }

    /// Play the next track.
    ///
    /// This is only available for Bluetooth, Wifi or USB sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn next_track(&mut self) -> Result<(), Error> {
        let source = self.input_source()?;

        match source {
            Source::Bluetooth | Source::Net | Source::Usb => self.send_command(COMMAND_NXT, b""),
            _ => Err(Error::NotSupportedForDeviceSource),
        }
    }

    /// Play the previous track.
    ///
    /// If the track has been playing for some time then this will replay the same track.
    ///
    /// This is only available for Bluetooth, Wifi or USB sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn previous_track(&mut self) -> Result<(), Error> {
        let source = self.input_source()?;

        match source {
            Source::Bluetooth | Source::Net | Source::Usb => self.send_command(COMMAND_PRE, b""),
            _ => Err(Error::NotSupportedForDeviceSource),
        }
    }

    /// Get current bluetooth connection state
    ///
    /// This is only available for Bluetooth sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn bluetooth_connected(&mut self) -> Result<bool, Error> {
        let source = self.input_source()?;

        if source != Source::Bluetooth {
            return Err(Error::NotSupportedForDeviceSource);
        };

        let response = self.send_query(COMMAND_BTC)?;

        let status = Switch::from_str(response.as_str())?;

        status.to_bool()
    }

    /// Reconnect the current bluetooth device
    ///
    /// This is only available for Bluetooth sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn connect_bluetooth(&mut self) -> Result<(), Error> {
        let source = self.input_source()?;

        if source != Source::Bluetooth {
            return Err(Error::NotSupportedForDeviceSource);
        };

        let reconnect = Switch::On;

        let mut buf = [0; 1];
        self.send_command(COMMAND_BTC, reconnect.to_parameter_str(&mut buf))
    }

    /// Disconnect the current bluetooth device
    ///
    /// This is only available for Bluetooth sources. If the source
    /// has been set to something different then this will return the
    /// error `Error::NotSupportedForDeviceSource`.
    pub fn disconnect_bluetooth(&mut self) -> Result<(), Error> {
        let source = self.input_source()?;

        if source != Source::Bluetooth {
            return Err(Error::NotSupportedForDeviceSource);
        };

        let disconnect = Switch::Off;

        let mut buf = [0; 1];
        self.send_command(COMMAND_BTC, disconnect.to_parameter_str(&mut buf))
    }

    #[doc(hidden)]
    pub fn playback_status(&mut self) -> Result<Playback, Error> {
        todo!()
    }
    #[doc(hidden)]
    pub fn audio_channel(&mut self) -> Result<AudioChannel, Error> {
        todo!()
    }
    #[doc(hidden)]
    pub fn multiroom_state(&mut self) -> Result<MultiroomState, Error> {
        todo!()
    }
    #[doc(hidden)]
    pub fn set_multiroom_state(&mut self, _state: MultiroomState) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn led(&mut self) -> Result<Led, Error> {
        todo!()
    }
    #[doc(hidden)]
    pub fn set_led(&mut self, _led_status: Led) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn beep(&mut self) -> Result<bool, Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn set_beep(&mut self, _beep: bool) -> Result<(), Error> {
        todo!()
    }

    #[doc(hidden)]
    pub fn set_play_preset(&mut self, _preset: PlayPreset) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn virtual_bass(&mut self) -> Result<bool, Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn enable_virtual_bass(&mut self) -> Result<(), Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn disable_virtual_bass(&mut self) -> Result<(), Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn toggle_virtual_bass(&mut self) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn reset_wifi(&mut self) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn loop_mode(&mut self) -> Result<LoopMode, Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn set_loop_mode(&mut self, _loop_mode: LoopMode) -> Result<(), Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn device_name(&mut self) -> Result<&str, Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn set_device_name(&mut self, _device_name: &str) -> Result<LoopMode, Error> {
        todo!();
    }
    #[doc(hidden)]
    pub fn enternet_connection(&mut self) -> Result<bool, Error> {
        todo!();
    }

    #[doc(hidden)]
    pub fn wifi_connection(&mut self) -> Result<bool, Error> {
        todo!();
    }

    //    ******* TODO more commands for version 4 here https://docs.google.com/spreadsheets/d/1LT6nsaCmg2B6vV0M2iOusxZ-hIqgDeqB0SLPTtZokCo/edit#gid=1444188925

    // Send a command with any specified parameters.
    // Commands are send as bytes with the following syntax (BNF)
    //
    //    <command> = <command_name> ";" | <command_name> ":" <parameter> <terminator>
    //    <command> ::= <alphanumeric> | <command>
    //    <terminator> ::= ";"
    fn send_command(&mut self, command: &str, parameter: &[u8]) -> Result<(), Error> {
        // Now send the command characters
        for c in command.chars() {
            self.uart.write(c as u8).map_err(|_| Error::SendCommand)?;
        }

        // Send parameters if available
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

        Ok(())
    }

    // Send a query and read the response. Queries are sent with the following syntax (BNF):
    //   <query> ::= <command_name> <terminator>
    //   <command_name> ::= <alphanumeric> | <command>
    //   <terminator> ::= ";"
    //
    // The response has the following syntax:
    //  <response> ::= <command_name> <parameter_start> <parameter_list> <terminator>
    //  <parameter_list> ::= <parameter> <parameter_delimiter> <parameter_list> | <parameter>
    //  <parameter_start> = ":"
    //  <parameter_delimiter> ::= ","
    //
    // In reality, a response  can be other characters due to framing issues, start up messages etc. so the "real"
    // response grammmer looks like:
    //  <response> ::= <noise> <command_name> <parameter_start> <parameter_list> <terminator>
    //  <parameter_list> ::= <parameter> <parameter_delimiter> <parameter_list> | <parameter>
    //  <noise >::= <control_character> | <character> | <noise>
    //  <control_char> ::=   "\n" | "\r"
    //  <character> is any printable character
    //
    fn send_query(&mut self, command: &str) -> Result<ArrayString<MAX_SIZE_RESPONSE>, Error> {
        const MAX_NUMBER_RESENDS: u8 = 3;

        let mut query_response = ArrayString::<MAX_SIZE_RESPONSE>::new();

        // Send  the command characters
        for c in command.chars() {
            block!(self.uart.write(c as u8)).map_err(|_| Error::SendCommand)?;
        }

        block!(self.uart.write(TERMINATOR)).map_err(|_| Error::SendCommand)?;

        block!(self.uart.flush()).map_err(|_| Error::SendCommand)?;

        //#[cfg_attr(not(test), derive(defmt::Format))] // Only used when running on target hardware
        enum Symbol {
            Character(u8),
            Block,
            ControlCharacter(u8),
            Terminator(u8),
            ParameterStart(u8),
            ParameterDelimiter(u8),
        }

        impl Symbol {
            pub fn as_char(&self) -> char {
                match self {
                    Self::Character(c) => *c as char,
                    Self::ControlCharacter(c) => *c as char,
                    Self::ParameterStart(c) => *c as char,
                    Self::ParameterDelimiter(c) => *c as char,
                    Self::Terminator(c) => *c as char,
                    Self::Block => '|',
                }
            }
        }

        //#[cfg_attr(not(test), derive(defmt::Format))] // Only used when running on target hardware
        #[derive(Clone, Copy)]
        enum ParseState {
            Command,
            ValidatedCommand,
            Parameter,
        }

        let mut state = ParseState::Command;
        let mut command_string_index = 0;

        // Read and parse the response
        loop {
            let symbol = match self.uart.read() {
                Ok(c) if c.is_ascii_alphanumeric() => Ok(Symbol::Character(c)),
                Ok(c) if c == b'-' => Ok(Symbol::Character(c)), // Occurs in the version number and negative numbers
                Ok(c) if c == b'+' => Ok(Symbol::Character(c)), // Occurs in certain commands
                Ok(c) if c.is_ascii_control() => Ok(Symbol::ControlCharacter(c)),
                Ok(c) if c == TERMINATOR => Ok(Symbol::Terminator(c)),
                Ok(c) if c == PARAMETER_START => Ok(Symbol::ParameterStart(c)),
                Ok(c) if c == PARAMETER_DELIMITER => Ok(Symbol::ParameterDelimiter(c)),
                // Other characters should not occur
                Ok(_) => Err(Error::Read),
                // Assuming that Err(WouldBlock) is an end of record.
                Err(nb::Error::WouldBlock) => Ok(Symbol::Block),
                // Read error condition
                Err(nb::Error::Other(_e)) => return Err(Error::Read),
            }?;

            match (state, symbol) {
                (ParseState::Command, Symbol::Character(c)) => {
                    if c == command.as_bytes()[command_string_index] {
                        command_string_index += 1;
                        if command_string_index != command.len() {
                            state = ParseState::Command;
                        } else {
                            state = ParseState::ValidatedCommand;
                        };
                    };
                }
                (ParseState::Command, Symbol::Block) => state = ParseState::Command,
                (ParseState::Command, _) => {
                    command_string_index = 0;
                    state = ParseState::Command;
                }
                (ParseState::ValidatedCommand, Symbol::ParameterStart(_)) => {
                    state = ParseState::Parameter
                }
                (ParseState::ValidatedCommand, Symbol::Block) => {
                    state = ParseState::ValidatedCommand
                }
                (ParseState::ValidatedCommand, _) => return Err(Error::ParseResponse),
                (ParseState::Parameter, Symbol::Character(c)) => query_response.push(c as char),

                // Currently not seperating parameters and just treating them all as a string.
                (ParseState::Parameter, Symbol::ParameterDelimiter(_)) => {
                    query_response.push(PARAMETER_DELIMITER as char)
                }

                (ParseState::Parameter, Symbol::Terminator(_)) => break, // Finished parsing
                (ParseState::Parameter, Symbol::Block) => state = ParseState::Parameter,

                (ParseState::Parameter, _) => return Err(Error::IllFormedReponse),
            }
        }

        Ok(query_response)
    }
}

#[cfg(test)]
mod test_api;
