#![allow(unused_imports)]

// TODO reinstate the conditional compilation, but this time
// use falgs to th compiler. Just handling the condition
// not(test) does not include conditions such as doctest.
// Need to provide docu on this in the readme.

// Do not include this when testing in std environment
// #[cfg(not(test))]
// use defmt::Format;

#[derive(Debug)]
//#[cfg_attr(not(test), derive(defmt::Format))] // Only used when running on target hardware
pub enum Error {
    NotSupportedForDeviceSource,
    ReadingQueryResponse,
    ParseResponse,
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
    Write,
    Unimplemented,
}
