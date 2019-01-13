use crate::Rule;
use pest::error::Error as PestError;
use std::num::ParseIntError;
use vcpu::ParseEnumError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    Pest(PestError<Rule>),
    ParseInt(ParseIntError),
    ParseEnum(ParseEnumError),
    CastInt,
}

impl From<PestError<Rule>> for ParseError {
    fn from(err: PestError<Rule>) -> ParseError {
        ParseError::Pest(err)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> ParseError {
        ParseError::ParseInt(err)
    }
}

impl From<ParseEnumError> for ParseError {
    fn from(err: ParseEnumError) -> ParseError {
        ParseError::ParseEnum(err)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssembleError {
    Misc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Parse(ParseError),
    Assemble(AssembleError),
}

impl<'i> From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::Parse(err)
    }
}

impl<'i> From<AssembleError> for Error {
    fn from(err: AssembleError) -> Error {
        Error::Assemble(err)
    }
}
