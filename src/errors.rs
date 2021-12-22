use crate::*;

use mmrbi::*;

use std::fmt::Display;
use std::io;



pub(crate) fn unexpected_eof(loc: &Location, expected: impl Display) -> io::Error {
    error!(at: &loc.path, line: loc.line_no_or_0(), "unexpected EOF, expected {}", expected);
    io::Error::new(io::ErrorKind::UnexpectedEof, format!("unexpected EOF at `{}`, expected {}", loc, expected))
}

pub(crate) fn warn_expected(loc: &Location, expected: impl Display) -> io::Error {
    warning!(at: &loc.path, line: loc.line_no_or_0(), "expected {}", expected);
    io::Error::new(io::ErrorKind::InvalidData, format!("expected {} at `{}`", expected, loc))
}

pub(crate) fn err_expected(loc: &Location, expected: impl Display) -> io::Error {
    error!(at: &loc.path, line: loc.line_no_or_0(), "expected {}", expected);
    io::Error::new(io::ErrorKind::InvalidData, format!("expected {} at `{}`", expected, loc))
}
