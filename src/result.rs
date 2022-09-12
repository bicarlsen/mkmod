use std::io;
use std::result::Result as StdResult;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Regex(regex::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Regex(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
