use core::str::FromStr;

use crate::error::Error;

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

#[derive(Debug, PartialEq)]
pub struct Volume(u8); //0..100

impl Volume {
    pub fn new(volume: u8) -> Result<Volume, Error> {
        let range = 0..=100;
        if range.contains(&volume) {
            Ok(Self(volume))
        } else {
            Err(Error::OutOfRange)
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn into_parameter_str<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let mut value = self.0;
        if value == 0 {
            //return b"0";
            buf[0] = b'0';
            return &buf[..1];
        }
        let mut i = 0;
        while value > 0 {
            buf[i] = (value % 10) as u8 + b'0';
            value /= 10;
            i += 1;
        }
        // let slice = &mut buf[..i];
        // slice.reverse();
        // &*slice
        buf[..i].reverse();
        // SAFETY: ??
        &buf[..i]
    }
}

impl FromStr for Volume {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let volume_value = s.parse::<u8>().map_err(|_| Error::InvalidString)?;

        let volume = Volume::new(volume_value)?;

        Ok(volume)
    }
}

#[derive(Debug, PartialEq)]
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

impl FromStr for Treble {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let treble_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let treble = Treble::new(treble_value)?;

        Ok(treble)
    }
}

#[derive(Debug, PartialEq)]
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

    // into_parameter_string - see https://doc.rust-lang.org/src/alloc/string.rs.html#2570
}

impl FromStr for Bass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bass_value = s.parse::<i8>().map_err(|_| Error::InvalidString)?;

        let bass = Bass::new(bass_value)?;

        Ok(bass)
    }
}

#[derive(Debug, PartialEq)]
pub struct PlayPreset(u8); // 0..10
impl PlayPreset {
    pub fn new(preset: u8) -> Result<Self, Error> {
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
        let preset_value = s.parse::<u8>().map_err(|_| Error::InvalidString)?;

        let preset = PlayPreset::new(preset_value)?;

        Ok(preset)
    }
}

///  A parameter that is used for on/off/toggle swiths in the UART API.
///  If the state is either On or Off it can be converted to a boolean (true for On).
#[derive(Debug, PartialEq)]
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

    pub fn into_parameter_str(&self, buf: &mut [u8]) -> &[u8] {
        let s = match self {
            Self::Off => "0",
            Self::On => "1",
            Self::Toggle => "T",
        };

        // Returned slice the same length as the parameter string
        &s.as_bytes()[0..1]
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_base_10_bytes() {
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
    fn volume_parameter_string() {
        let mut buf = [0; 3];
        assert_eq!(
            Volume::new(100).unwrap().into_parameter_str(&mut buf),
            b"100"
        );

        // assert_eq!(
        //     Volume::new(99).unwrap().into_parameter_str(),
        //     ArrayString::from("99").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(75).unwrap().into_parameter_str(),
        //     ArrayString::from("75").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(23).unwrap().into_parameter_str(),
        //     ArrayString::from("23").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(10).unwrap().into_parameter_str(),
        //     ArrayString::from("10").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(7).unwrap().into_parameter_str(),
        //     ArrayString::from("7").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(1).unwrap().into_parameter_str(),
        //     ArrayString::from("1").unwrap()
        // );

        // assert_eq!(
        //     Volume::new(0).unwrap().into_parameter_str(),
        //     ArrayString::from("0").unwrap()
        // );
    }

    #[test]
    fn volume_parameter_string2() {
        let test_input: [u8; 8] = [100, 99, 75, 23, 10, 7, 1, 0];

        let expected: [&str; 8] = ["100", "99", "75", "23", "10", "7", "1", "0"];

        let mut buf = [0; 3];
        for n in test_input.iter().enumerate() {
            let vol = Volume::new(*n.1).unwrap().into_parameter_str(&mut buf);
            //assert_eq!(vol, ArrayString::from(expected[n.0]).unwrap());
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

    // ---------------- preset TESTS
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

        assert_eq!(Switch::Off.into_parameter_str(&mut buf), b"0");
        assert_eq!(Switch::On.into_parameter_str(&mut buf), b"1");
        assert_eq!(Switch::Toggle.into_parameter_str(&mut buf), b"T");
    }
}
