use std::{error, result};
use std::fmt;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
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

pub type CmdFn = Fn(Vec<String>, Receiver<String>, Sender<String>) + Send + Sync + 'static;

/// A shell.
#[derive(Default)]
pub struct Sh<'a> {
    pub cmds: HashMap<&'a str, &'static CmdFn>,
}

impl<'a> Sh<'a> {
    /// Create a new shell.
    pub fn new() -> Sh<'a> {
        Default::default()
    }

    /// Run a command.
    pub fn run_cmd(&self, argv: Vec<String>) -> Result<Vec<String>> {
        let name = try!(argv.first().ok_or(Error::EmptyCommand));
        let cmd_fn = *try!(self.cmds.get(&name[..])
                           .ok_or(Error::UnknownCommand));
        let argv = argv.clone();
        let (tx1, rx1) = channel::<String>();
        let (tx2, rx2) = channel::<String>();
        thread::spawn(move || cmd_fn(argv, rx1, tx2));
        drop(tx1);
        Ok(rx2.iter().collect())
    }

    /// Parse and run a source string.
    pub fn run_str(&self, source: &str) -> Result<Vec<String>> {
        let mut shlex = Shlex::new(source);
        let cmd = shlex.by_ref().collect::<Vec<_>>();
        if shlex.had_error {
            Err(Error::ParseError)
        } else {
            self.run_cmd(cmd)
        }
    }
}
