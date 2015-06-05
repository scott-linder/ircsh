use lex::{self, Lexer, Tok};
use std::{error, result};
use std::fmt;
use std::convert::From;

pub type Result<T> = result::Result<T, Error>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    LexError(lex::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::LexError(ref le) => le.description(),
        }
    }
}

impl From<lex::Error> for Error {
    fn from(e: lex::Error) -> Error {
        Error::LexError(e)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Cmd<'a>(pub Vec<&'a str>);

impl<'a> Cmd<'a> {
    pub fn new() -> Cmd<'a> {
        Cmd(vec![])
    }
}

pub fn parse<'a>(mut lexer: Lexer<'a>) -> Result<Vec<Cmd<'a>>> {
    let mut cmds = vec![Cmd::new()];
    while let Some(tok) = lexer.next() {
        match try!(tok) {
            Tok::String(s) => cmds.last_mut().expect("cmds is empty").0.push(s),
            Tok::Semicolon => cmds.push(Cmd::new()),
        }
    }
    Ok(cmds)
}

#[cfg(test)]
mod tests {
    use super::{Cmd, parse};
    use lex::Lexer;

    #[test]
    fn test_parse() {
        assert_eq!(parse(Lexer::new("foo")),
            Ok(vec![Cmd(vec!["foo"])]));
        assert_eq!(parse(Lexer::new("foo bar")),
            Ok(vec![Cmd(vec!["foo", "bar"])]));
        assert_eq!(parse(Lexer::new("foo bar; baz")),
            Ok(vec![Cmd(vec!["foo", "bar"]),
                    Cmd(vec!["baz"])]));
        assert_eq!(parse(Lexer::new("foo bar; baz;; qux")),
            Ok(vec![Cmd(vec!["foo", "bar"]),
                    Cmd(vec!["baz"]),
                    Cmd(vec![]),
                    Cmd(vec!["qux"])]));
    }
}
