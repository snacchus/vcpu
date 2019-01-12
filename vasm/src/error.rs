use crate::Rule;
use vcpu::ParseEnumError;
use std::num::ParseIntError;
use pest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError<'i> {
    Pest(pest::Error<'i, Rule>),
    ParseInt(ParseIntError),
    ParseEnum(ParseEnumError),
    CastInt,
}

impl<'i> From<pest::Error<'i, Rule>> for ParseError<'i> {
    fn from(err: pest::Error<'i, Rule>) -> ParseError<'i> {
        ParseError::Pest(err)
    }
}

impl<'i> From<ParseIntError> for ParseError<'i> {
    fn from(err: ParseIntError) -> ParseError<'i> {
        ParseError::ParseInt(err)
    }
}

impl<'i> From<ParseEnumError> for ParseError<'i> {
    fn from(err: ParseEnumError) -> ParseError<'i> {
        ParseError::ParseEnum(err)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssembleError {
    Misc
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error<'i> {
    Parse(ParseError<'i>),
    Assemble(AssembleError)
}

impl<'i> From<ParseError<'i>> for Error<'i> {
    fn from(err: ParseError<'i>) -> Error<'i> {
        Error::Parse(err)
    }
}

impl<'i> From<AssembleError> for Error<'i> {
    fn from(err: AssembleError) -> Error<'i> {
        Error::Assemble(err)
    }
}
