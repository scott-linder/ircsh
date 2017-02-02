use std::{error, result};
use std::fmt;
use std::collections::HashMap;
use shlex::Shlex;

pub type Result<T> = result::Result<T, Error>;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    EmptyCommand,
    UnknownCommand,
    ParseError,
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
            Error::ParseError => "Parse error",
        }
    }
}

/// A shell.
pub struct Sh {
    pub cmds: HashMap<String, Box<Fn(&[String]) -> String>>,
}

impl Sh {
    /// Create a new shell.
    pub fn new() -> Sh {
        Sh {
            cmds: HashMap::new()
        }
    }

    /// Run a command.
    pub fn run_cmd(&self, cmd: Vec<String>) -> Result<String> {
        if let Some(name) = cmd.first() {
            if let Some(cmd_fn) = self.cmds.get(name) {
                Ok(cmd_fn(&cmd[1..]))
            } else {
                Err(Error::UnknownCommand)
            }
        } else {
            Err(Error::EmptyCommand)
        }
    }

    /// Parse and run a source string.
    pub fn run_str(&self, source: &str) -> Result<String> {
        let mut shlex = Shlex::new(source);
        let cmd = shlex.by_ref().collect::<Vec<_>>();
        if shlex.had_error {
            Err(Error::ParseError)
        } else {
            self.run_cmd(cmd)
        }
    }
}
