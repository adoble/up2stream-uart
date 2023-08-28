use core::str::FromStr;

use crate::error::Error;

// // Takes an integer value and converts to a set of ascii (u8) bytes
// // representing the number.
// fn base_10_bytes(mut value: u64, buf: &mut [u8]) -> &[u8] {
//     if value == 0 {
//         return b"0";
//     }
//     let mut i = 0;
//     while value > 0 {
//         buf[i] = (value % 10) as u8 + b'0';
//         value /= 10;
//         i += 1;
//     }
//     let slice = &mut buf[..i];
//     slice.reverse();
//     &*slice
// }

pub trait ScalarParameter<T> {
    fn get(&self) -> i8;
    //fn set(&self, value: T) -> &mut T;

    fn to_parameter_str<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let mut value = self.get(); //self.0.into();
        if value == 0 {
            //return b"0";
            buf[0] = b'0';
            return &buf[..1];
        }

        let negative: bool;
        if value < 0 {
            negative = true;
            value = -value;
        } else {
            negative = false;
        }

        let mut i = 0;
        while value > 0 {
            let digit: u32 = value as u32 % 10;

            let chr = char::from_digit(digit, 10);
            if let Some(c) = chr {
                buf[i] = u8::try_from(c).unwrap_or(b'?');
            }

            value /= 10;
            i += 1;
        }
        if negative {
            buf[i] = b'-';
            i += 1;
        };

        // Transmission order
        buf[..i].reverse();

        // Only return the slice worked on
        &buf[..i]
    }
}

/// Represents a volume from 0 to 100.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Volume(i8);

impl Volume {
    pub fn new(volume: i8) -> Result<Volume, Error> {
        let range = 0..=100;
        if range.contains(&volume) {
            Ok(Self(volume))
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl ScalarParameter<u8> for Volume {
    /// Get the volume as value
    fn get(&self) -> i8 {
        self.0
    }
}

impl FromStr for Volume {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let volume_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let volume = Volume::new(volume_value)?;

        Ok(volume)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Treble(i8); //-10..10
impl Treble {
    pub fn new(treble: i8) -> Result<Self, Error> {
        let range = -10..=10;
        if range.contains(&treble) {
            Ok(Self(treble))
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl ScalarParameter<i8> for Treble {
    /// Get the treble settign as value
    fn get(&self) -> i8 {
        self.0
    }
}

impl FromStr for Treble {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let treble_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let treble = Treble::new(treble_value)?;

        Ok(treble)
    }
}

/// Represents a bass setting.
/// Bass settings can be from -10 to 10.
///
/// # Examples
/// ```ignore
/// use  up2stream_uart::Bass;
///
/// let bass = Bass::new(5).unwrap();
///
/// assert_eq!(5, bass.get());
///
/// ```
///
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bass(i8); //-10..10
impl Bass {
    pub fn new(bass: i8) -> Result<Self, Error> {
        let range = -10..=10;
        if range.contains(&bass) {
            Ok(Self(bass))
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl ScalarParameter<i8> for Bass {
    fn get(&self) -> i8 {
        self.0
    }
}

impl FromStr for Bass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bass_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let bass = Bass::new(bass_value)?;

        Ok(bass)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PlayPreset(i8); // 0..10
impl PlayPreset {
    pub fn new(preset: i8) -> Result<Self, Error> {
        let range = 0..=10;
        if range.contains(&preset) {
            Ok(Self(preset))
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl FromStr for PlayPreset {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let preset_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let preset = PlayPreset::new(preset_value)?;

        Ok(preset)
    }
}

///  A parameter that is used for on/off/toggle swiths in the UART API.
///  If the state is either On or Off it can be converted to a boolean (true for On).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Switch {
    On,
    Off,
    Toggle,
}

impl Switch {
    pub fn to_bool(&self) -> Result<bool, Error> {
        match self {
            Self::On => Ok(true),
            Self::Off => Ok(false),
            Self::Toggle => Err(Error::CannotConvert),
        }
    }

    pub fn to_parameter_str<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let s = match self {
            Self::Off => "0",
            Self::On => "1",
            Self::Toggle => "T",
        };

        buf[0] = s.as_bytes()[0];

        // Returned slice the same length as the parameter string
        &buf[0..1]
    }
}

impl From<bool> for Switch {
    fn from(value: bool) -> Self {
        if value {
            Switch::On
        } else {
            Switch::Off
        }
    }
}

impl FromStr for Switch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Switch::Off),
            "1" => Ok(Switch::On),
            "T" => Ok(Switch::Toggle),
            _ => Err(Error::InvalidString),
        }
    }
}

pub enum SystemControl {
    Reboot,
    Standby,
    Reset,
    Recover,
}

impl SystemControl {
    //TODO use a standard trait?
    pub fn to_parameter_str<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let parameter = match self {
            Self::Reboot => "REBOOT",
            Self::Standby => "STANDBY",
            Self::Reset => "RESET",
            Self::Recover => "RECOVER",
        };

        buf[..parameter.len()].clone_from_slice(&parameter.as_bytes()[..parameter.len()]);

        // Return the slice that has the same number of characters as
        // the parameter
        &buf[..parameter.len()]

        //&parameter.as_bytes()[..parameter.len()]
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
    Hdmi,
}
impl Source {
    pub fn to_parameter_str<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let parameter = match self {
            Self::Net => "NET",
            Self::Usb => "USB",
            Self::UsbDac => "USBDAC",
            Self::LineIn => "LINE-IN",
            Self::LineIn2 => "LINE-IN2",
            Self::Bluetooth => "BT",
            Self::Optical => "OPT",
            Self::Coax => "COAX",
            Self::I2S => "I2S",
            Self::Hdmi => "HDMI",
        };

        // Returned slice the same length as the parameter string
        buf[..parameter.len()].clone_from_slice(&parameter.as_bytes()[..parameter.len()]);

        // Return the slice that has the same number of characters as
        // the parameter
        &buf[..parameter.len()]
    }
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
            "HDMI" => Ok(Source::Hdmi),
            _ => Err(Error::SourceNotKnown), // "USB", "USBDAC", "LINE-IN", "LINE-IN2", "BT", "OPT", "COAX", "I2S", "HDMI",
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DeviceStatus {
    pub source: Source,
    pub mute: bool,
    pub volume: Volume,
    pub treble: Treble,
    pub bass: Bass,
    pub net: bool,
    pub internet: bool,
    pub playing: bool,
    pub led: bool,
    pub upgrading: bool,
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
mod test {

    use arrayvec::ArrayVec;

    use super::*;

    // Test utility to take an integer value and converts to a set of
    // ascii (u8) bytes representing a number.
    fn base_10_bytes(mut value: u64, buf: &mut [u8]) -> &[u8] {
        if value == 0 {
            return b"0";
        }
        let mut i = 0;
        while value > 0 {
            buf[i] = (value % 10) as u8 + b'0';
            value /= 10;
            i += 1;
        }
        let slice = &mut buf[..i];
        slice.reverse();
        &*slice
    }

    #[test]
    fn test_base_10_bytes() {
        // This only tests
        let mut buf: [u8; 3] = [0; 3];
        assert_eq!(base_10_bytes(34, &mut buf), b"34");
        assert_eq!(base_10_bytes(255, &mut buf), b"255");
        assert_eq!(base_10_bytes(105, &mut buf), b"105");
        assert_eq!(base_10_bytes(100, &mut buf), b"100");
        assert_eq!(base_10_bytes(99, &mut buf), b"99");
        assert_eq!(base_10_bytes(45, &mut buf), b"45");
        assert_eq!(base_10_bytes(10, &mut buf), b"10");
        assert_eq!(base_10_bytes(5, &mut buf), b"5");
        assert_eq!(base_10_bytes(1, &mut buf), b"1");
        assert_eq!(base_10_bytes(0, &mut buf), b"0");
    }

    #[test]
    fn new_volume() -> Result<(), Error> {
        let v1 = Volume::new(34)?;
        let v2 = Volume::new(34)?;

        assert_eq!(v1, v2);

        let v3 = Volume::new(44)?;
        assert_ne!(v1, v3);

        Ok(())
    }

    #[test]
    fn new_volume_limits() {
        let mut vol = Volume::new(101);
        assert!(vol.is_err());

        vol = Volume::new(100);
        assert!(vol.is_ok());

        vol = Volume::new(0);
        assert!(vol.is_ok());
    }

    #[test]
    fn volume_from_str() {
        let mut expected_vol = Volume::new(10).unwrap();

        let vol = Volume::from_str("10").unwrap();
        assert_eq!(vol, expected_vol);

        expected_vol = Volume::new(100).unwrap();
        let vol = Volume::from_str("100").unwrap();
        assert_eq!(vol, expected_vol);

        expected_vol = Volume::new(0).unwrap();
        let vol = Volume::from_str("0").unwrap();
        assert_eq!(vol, expected_vol);

        let vol = Volume::from_str("-10");
        assert!(vol.is_err());

        let vol = Volume::from_str("101");
        assert!(vol.is_err());

        let vol = Volume::from_str("XXX");
        assert!(vol.is_err());
    }

    #[test]
    fn volume_get() {
        let vol = Volume::new(23).unwrap();

        assert_eq!(23, vol.get());
    }

    #[test]
    fn volume_parameter_string() {
        let mut buf = [0; 3];
        assert_eq!(Volume::new(100).unwrap().to_parameter_str(&mut buf), b"100");
        assert_eq!(Volume::new(99).unwrap().to_parameter_str(&mut buf), b"99");
        assert_eq!(Volume::new(75).unwrap().to_parameter_str(&mut buf), b"75");
        assert_eq!(Volume::new(23).unwrap().to_parameter_str(&mut buf), b"23");
        assert_eq!(Volume::new(10).unwrap().to_parameter_str(&mut buf), b"10");
        assert_eq!(Volume::new(7).unwrap().to_parameter_str(&mut buf), b"7");
        assert_eq!(Volume::new(1).unwrap().to_parameter_str(&mut buf), b"1");
        assert_eq!(Volume::new(0).unwrap().to_parameter_str(&mut buf), b"0");
    }

    #[test]
    fn volume_parameter_string2() {
        let test_input: [i8; 8] = [100, 99, 75, 23, 10, 7, 1, 0];

        let expected: [&str; 8] = ["100", "99", "75", "23", "10", "7", "1", "0"];

        let mut buf = [0; 3];
        for n in test_input.iter().enumerate() {
            let vol = Volume::new(*n.1).unwrap().to_parameter_str(&mut buf);
            assert_eq!(vol, expected[n.0].as_bytes());
        }
    }

    // ---------------- TREBLE TESTS
    #[test]
    fn new_treble() -> Result<(), Error> {
        let t1 = Treble::new(5)?;
        let t2 = Treble::new(5)?;

        assert_eq!(t1, t2);

        let t3 = Treble::new(-2)?;
        assert_ne!(t1, t3);

        Ok(())
    }

    #[test]
    fn new_treble_limits() {
        let mut treble = Treble::new(11);
        assert!(treble.is_err());

        treble = Treble::new(10);
        assert!(treble.is_ok());

        treble = Treble::new(0);
        assert!(treble.is_ok());

        treble = Treble::new(-10);
        assert!(treble.is_ok());

        treble = Treble::new(-11);
        assert!(treble.is_err());
    }

    #[test]
    fn treble_get() {
        let treble = Treble::new(-3).unwrap();

        assert_eq!(-3, treble.get());
    }

    #[test]
    fn treble_from_str() {
        let mut expected_treble = Treble::new(10).unwrap();

        let treble = Treble::from_str("10").unwrap();
        assert_eq!(treble, expected_treble);

        expected_treble = Treble::new(-10).unwrap();
        let treble = Treble::from_str("-10").unwrap();
        assert_eq!(treble, expected_treble);

        expected_treble = Treble::new(0).unwrap();
        let treble = Treble::from_str("0").unwrap();
        assert_eq!(treble, expected_treble);

        let treble = Treble::from_str("-11");
        assert!(treble.is_err());

        let treble = Treble::from_str("101");
        assert!(treble.is_err());

        let treble = Treble::from_str("XXX");
        assert!(treble.is_err());
    }

    #[test]
    fn new_bass() -> Result<(), Error> {
        let b1 = Bass::new(5)?;
        let b2 = Bass::new(5)?;

        assert_eq!(b1, b2);

        let b3 = Bass::new(-2)?;
        assert_ne!(b1, b3);

        Ok(())
    }

    #[test]
    fn new_bass_limits() {
        let mut bass = Bass::new(11);
        assert!(bass.is_err());

        bass = Bass::new(10);
        assert!(bass.is_ok());

        bass = Bass::new(0);
        assert!(bass.is_ok());

        bass = Bass::new(-10);
        assert!(bass.is_ok());

        bass = Bass::new(-11);
        assert!(bass.is_err());
    }

    #[test]
    fn bass_get() {
        let bass = Bass::new(5).unwrap();

        assert_eq!(5, bass.get());
    }

    #[test]
    fn bass_from_str() {
        let mut expected_bass = Bass::new(10).unwrap();

        let bass = Bass::from_str("10").unwrap();
        assert_eq!(bass, expected_bass);

        expected_bass = Bass::new(-10).unwrap();
        let bass = Bass::from_str("-10").unwrap();
        assert_eq!(bass, expected_bass);

        expected_bass = Bass::new(0).unwrap();
        let bass = Bass::from_str("0").unwrap();
        assert_eq!(bass, expected_bass);

        let mut bass = Bass::from_str("-11");
        assert!(bass.is_err());

        bass = Bass::from_str("101");
        assert!(bass.is_err());

        bass = Bass::from_str("XXX");
        assert!(bass.is_err());
    }

    #[test]
    fn bass_parameter_string() {
        let test_input: [i8; 5] = [10, 5, 0, -4, -10];

        let expected: [&str; 5] = ["10", "5", "0", "-4", "-10"];

        let mut buf = [0; 3];
        for n in test_input.iter().enumerate() {
            let bass_parameter = Bass::new(*n.1).unwrap().to_parameter_str(&mut buf);
            assert_eq!(bass_parameter, expected[n.0].as_bytes());
        }
    }

    // ---------------- PLAYPRESET TESTS
    #[test]
    fn new_preset() -> Result<(), Error> {
        let p1 = PlayPreset::new(5)?;
        let p2 = PlayPreset::new(5)?;

        assert_eq!(p1, p2);

        let p3 = PlayPreset::new(3)?;
        assert_ne!(p1, p3);

        Ok(())
    }

    #[test]
    fn new_preset_limits() {
        let mut preset = PlayPreset::new(11);
        assert!(preset.is_err());

        preset = PlayPreset::new(10);
        assert!(preset.is_ok());

        preset = PlayPreset::new(0);
        assert!(preset.is_ok());

        preset = PlayPreset::new(11);
        assert!(preset.is_err());
    }

    #[test]
    fn preset_from_str() {
        let mut expected_preset = PlayPreset::new(10).unwrap();

        let mut preset = PlayPreset::from_str("10").unwrap();
        assert_eq!(preset, expected_preset);

        expected_preset = PlayPreset::new(0).unwrap();
        preset = PlayPreset::from_str("0").unwrap();
        assert_eq!(preset, expected_preset);

        let mut preset = PlayPreset::from_str("11");
        assert!(preset.is_err());

        preset = PlayPreset::from_str("101");
        assert!(preset.is_err());

        preset = PlayPreset::from_str("XXX");
        assert!(preset.is_err());
    }

    #[test]
    fn switch_from() {
        let mut switch: Switch = Switch::from(true);

        assert_eq!(switch, Switch::On);

        switch = Switch::from(false);

        assert_eq!(switch, Switch::Off);
    }

    #[test]
    fn switch_into() {
        let mut switch = Switch::On;

        let state: bool = switch.to_bool().unwrap();

        assert!(state);

        switch = Switch::Off;

        assert!(!switch.to_bool().unwrap());

        switch = Switch::Toggle;

        assert!(switch.to_bool().is_err());
    }

    #[test]
    fn switch_from_string() {
        assert_eq!(Switch::from_str("0").unwrap(), Switch::Off);
        assert_eq!(Switch::from_str("1").unwrap(), Switch::On);
        assert_eq!(Switch::from_str("T").unwrap(), Switch::Toggle);
        assert!(Switch::from_str("X").is_err());
    }

    #[test]
    fn switch_to_string() {
        let mut buf = [0; 1];

        assert_eq!(Switch::Off.to_parameter_str(&mut buf), b"0");
        assert_eq!(Switch::On.to_parameter_str(&mut buf), b"1");
        assert_eq!(Switch::Toggle.to_parameter_str(&mut buf), b"T");
    }

    #[test]
    fn system_control_to_parameter_str() {
        let mut buf: [u8; 7] = [0; 7];
        assert_eq!(SystemControl::Reboot.to_parameter_str(&mut buf), b"REBOOT");
        assert_eq!(
            SystemControl::Standby.to_parameter_str(&mut buf),
            b"STANDBY"
        );
        assert_eq!(SystemControl::Reset.to_parameter_str(&mut buf), b"RESET");
        assert_eq!(
            SystemControl::Recover.to_parameter_str(&mut buf),
            b"RECOVER"
        );
    }

    #[test]
    fn source_from_string() {
        const NUMBER_SOURCES: usize = 10;
        let source_strings: [&str; NUMBER_SOURCES] = [
            "NET", "USB", "USBDAC", "LINE-IN", "LINE-IN2", "BT", "OPT", "COAX", "I2S", "HDMI",
        ];
        use Source::*;
        let expected_sources = ArrayVec::from([
            Net, Usb, UsbDac, LineIn, LineIn2, Bluetooth, Optical, Coax, I2S, Hdmi,
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
}
