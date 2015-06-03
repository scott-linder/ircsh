use std::str::CharIndices;
use replace::ReplaceOne;

static WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];
static SYMBOLS: [char; 1] = [';'];

fn terminates_string(c: char) -> bool {
    WHITESPACE.contains(&c) || SYMBOLS.contains(&c)
}

/// Language token.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Tok<'a> {
    String(&'a str),
    Semicolon,
}

/// Lexical analyzer.
pub struct Lex<'a> {
    source: &'a str,
    chars: ReplaceOne<CharIndices<'a>>,
}

impl<'a> Lex<'a> {
    /// Create a new lexer.
    pub fn new(source: &'a str) -> Lex<'a> {
        Lex {
            source: source,
            chars: ReplaceOne::new(source.char_indices()),
        }
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

    /// Expect a string, starting from index `start` into the source.
    fn string(&mut self, start: usize) -> Option<Tok<'a>> {
        loop {
            if let Some((i, c)) = self.chars.next() {
                if terminates_string(c) {
                    self.chars.replace((i, c));
                    return Some(Tok::String(&self.source[start..i]));
                }
            } else {
                break;
            }
        }
        Some(Tok::String(&self.source[start..]))
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = Tok<'a>;

    fn next(&mut self) -> Option<Tok<'a>> {
        self.consume_whitespace();
        if let Some((i, c)) = self.chars.next() {
            return match c {
                ';' => Some(Tok::Semicolon),
                _ => self.string(i),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Tok, Lex};

    #[test]
    fn empty() {
        let mut lex = Lex::new("");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn string() {
        let mut lex = Lex::new("string");
        assert_eq!(lex.next(), Some(Tok::String("string")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn two_string() {
        let mut lex = Lex::new("one two");
        assert_eq!(lex.next(), Some(Tok::String("one")));
        assert_eq!(lex.next(), Some(Tok::String("two")));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn string_semicolon_string() {
        let mut lex = Lex::new("one; two");
        assert_eq!(lex.next(), Some(Tok::String("one")));
        assert_eq!(lex.next(), Some(Tok::Semicolon));
        assert_eq!(lex.next(), Some(Tok::String("two")));
        assert_eq!(lex.next(), None);
    }
}
