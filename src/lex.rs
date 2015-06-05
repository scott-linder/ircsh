use std::str::CharIndices;
use std::fmt;
use std::{error, result};
use replace::ReplaceOne;

pub type Result<T> = result::Result<T, Error>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    UnterminatedString,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnterminatedString => "Quoted string left unclosed.",
        }
    }
}

static WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];
static SYMBOLS: [char; 2] = [';', '"'];

fn terminates_unquoted_string(c: char) -> bool {
    WHITESPACE.contains(&c) || SYMBOLS.contains(&c)
}

/// Language token.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Tok<'a> {
    String(&'a str),
    Semicolon,
}

/// Lexical analyzer.
///
/// Implemented as an iterator which yields a result. After an error is
/// encountered and returned (e.g. `Some(Err(..))`) the iterator will
/// yield `None` forever.
pub struct Lexer<'a> {
    source: &'a str,
    chars: ReplaceOne<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer.
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer {
            source: source,
            chars: ReplaceOne::new(source.char_indices()),
        }
    }

    /// Update underylying iterator to point to end of source.
    fn consume(&mut self) {
        while let Some(..) = self.chars.next() {}
    }

    /// Update underlying iterator to point to next non-whitespace character.
    fn consume_whitespace(&mut self) {
        while let Some((i, c)) = self.chars.next() {
            if !WHITESPACE.contains(&c) {
                self.chars.replace((i, c));
                break;
            }
        }
    }

    /// Expect an unquoted string, starting from index `start` into the source.
    fn unquoted_string(&mut self, start: usize) -> Result<Tok<'a>> {
        while let Some((i, c)) = self.chars.next() {
            if terminates_unquoted_string(c) {
                self.chars.replace((i, c));
                return Ok(Tok::String(&self.source[start..i]));
            }
        }
        Ok(Tok::String(&self.source[start..]))
    }

    /// Expect a quoted string.
    fn quoted_string(&mut self) -> Result<Tok<'a>> {
        let start = match self.chars.next() {
            Some((i, '"')) => return Ok(Tok::String(&self.source[i..i])),
            Some((i, _)) => i,
            None => return Err(Error::UnterminatedString),
        };
        while let Some((i, c)) = self.chars.next() {
            if c == '"' {
                return Ok(Tok::String(&self.source[start..i]));
            }
        }
        return Err(Error::UnterminatedString);
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Tok<'a>>;

    fn next(&mut self) -> Option<Result<Tok<'a>>> {
        self.consume_whitespace();
        if let Some((i, c)) = self.chars.next() {
            let item = match c {
                ';' => Ok(Tok::Semicolon),
                '"' => self.quoted_string(),
                _ => self.unquoted_string(i),
            };
            if let Err(..) = item {
                self.consume();
            }
            return Some(item);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Tok, Lexer};

    #[test]
    fn empty() {
        let mut lex = Lexer::new("");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn string() {
        let mut lex = Lexer::new("string");
        assert_eq!(lex.next(), Some(Ok(Tok::String("string"))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn two_string() {
        let mut lex = Lexer::new("one two");
        assert_eq!(lex.next(), Some(Ok(Tok::String("one"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String("two"))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn string_semicolon_string() {
        let mut lex = Lexer::new("one; two");
        assert_eq!(lex.next(), Some(Ok(Tok::String("one"))));
        assert_eq!(lex.next(), Some(Ok(Tok::Semicolon)));
        assert_eq!(lex.next(), Some(Ok(Tok::String("two"))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn quoted_string() {
        let mut lex = Lexer::new(r#""foo bar""#);
        assert_eq!(lex.next(), Some(Ok(Tok::String("foo bar"))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn empty_quoted_string() {
        let mut lex = Lexer::new(r#""""#);
        assert_eq!(lex.next(), Some(Ok(Tok::String(""))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn quoted_strings_and_semicolons() {
        let mut lex = Lexer::new(r#"foo "bar baz" qux; one two "three" """#);
        assert_eq!(lex.next(), Some(Ok(Tok::String("foo"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String("bar baz"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String("qux"))));
        assert_eq!(lex.next(), Some(Ok(Tok::Semicolon)));
        assert_eq!(lex.next(), Some(Ok(Tok::String("one"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String("two"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String("three"))));
        assert_eq!(lex.next(), Some(Ok(Tok::String(""))));
        assert_eq!(lex.next(), None);
    }
}
