// Do not include this when testing in std environment
#[cfg(not(test))]
use defmt::Format;

#[derive(Debug)]
#[cfg_attr(not(test), derive(Format))] // Only used when running on target hardware
pub enum Error {
    NotSupportedForDeviceSource,
    ReadingQueryReponse,
    NonUTF8,
    SendCommand,
    SourceNotKnown,
    BooleanParse,
    OutOfRange,
    InvalidString,
    IllFormedReponse,
    CannotConvert,
    Timeout,
    Read,
    Unimplemented,
}
