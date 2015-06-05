use std::{error, result};
use std::fmt;
use std::collections::HashMap;
use lex::{Lexer};
use parse::{self, Cmd};

pub type Result<T> = result::Result<T, Error>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    EmptyCommand,
    UnknownCommand,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EmptyCommand => "Empty command",
            Error::UnknownCommand => "Unknown command",
        }
    }
}

/// A shell.
pub struct Sh {
    pub cmds: HashMap<String, Box<Fn(&[&str]) -> String>>,
}

impl Sh {
    /// Create a new shell.
    pub fn new() -> Sh {
        Sh {
            cmds: HashMap::new()
        }
    }

    /// Run a command.
    pub fn run_cmd(&self, cmd: &Cmd) -> Result<String> {
        if let Some(name) = cmd.0.first() {
            if let Some(cmd_fn) = self.cmds.get(*name) {
                Ok(cmd_fn(&cmd.0[1..]))
            } else {
                Err(Error::UnknownCommand)
            }
        } else {
            Err(Error::EmptyCommand)
        }
    }

    /// Parse and run a source string.
    pub fn run_str(&self, source: &str) -> parse::Result<Vec<Result<String>>> {
        let cmds = try!(parse::parse(Lexer::new(source)));
        Ok(cmds.iter().map(|cmd| { self.run_cmd(cmd) }).collect())
    }
}
