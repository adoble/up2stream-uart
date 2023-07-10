#[derive(Debug)]
pub enum Error {
    NotSupportedForDeviceSource,
    ReadingQueryReponse,
    NonUTF8,
    SendCommand,
    SourceNotKnown,
    BooleanParse,
    OutOfRange,
    InvalidString,
    Unimplemented,
}
