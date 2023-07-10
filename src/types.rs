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
        let volume = s.parse::<u8>().map_err(|_| Error::InvalidString)?;
        Ok(Self(volume))
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
}
