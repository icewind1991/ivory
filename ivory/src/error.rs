use crate::zend::ZValType;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct CastError {
    pub actual: ZValType,
}

#[derive(Debug)]
pub enum ArgError {
    CastError(CastError),
    NotEnoughArguments,
}

impl From<CastError> for ArgError {
    fn from(from: CastError) -> Self {
        ArgError::CastError(from)
    }
}

impl Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect variable type, got {}", self.actual)
    }
}

impl Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::CastError(err) => err.fmt(f),
            ArgError::NotEnoughArguments => write!(f, "Not enough arugments"),
        }
    }
}

impl Error for CastError {
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Error for ArgError {
    fn cause(&self) -> Option<&Error> {
        match self {
            ArgError::CastError(err) => Some(err),
            _ => None,
        }
    }
}
