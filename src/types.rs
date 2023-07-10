use core::str::FromStr;

use crate::error::Error;

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
struct Treble(i8); //-10..10
impl Treble {
    fn new(treble: i8) -> Result<Self, Error> {
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
        let treble = s.parse::<i8>().map_err(|_| Error::InvalidString)?;
        Ok(Self(treble))
    }
}

#[derive(Debug, PartialEq)]
struct Bass(i8); //-10..10
impl Bass {
    fn new(bass: i8) -> Result<Self, Error> {
        let range = -10..=10;
        if range.contains(&bass) {
            Ok(Self(bass))
        } else {
            Err(Error::OutOfRange)
        }
    }
}

impl FromStr for Bass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bass = s.parse::<i8>().map_err(|_| Error::InvalidString)?;
        Ok(Self(bass))
    }
}

struct PlayPreset(u8); // 0..10
impl PlayPreset {
    fn new(preset: u8) -> Result<Self, Error> {
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
        let preset = s.parse::<u8>().map_err(|_| Error::InvalidString)?;
        Ok(Self(preset))
    }
}

#[cfg(test)]
mod test {

    use super::*;

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
}
