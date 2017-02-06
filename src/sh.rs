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
    pub fn run_cmds(&self, argvs: Vec<Vec<String>>) -> Result<Vec<String>> {
        let (_, mut rx1) = channel::<String>();
        let (mut tx2, mut rx2) = channel::<String>();
        for argv in argvs {
            let cmd_fn = *try!(argv.first()
                                   .ok_or(Error::EmptyCommand)
                                   .and_then(|name|
                                        self.cmds.get(name.as_str())
                                                 .ok_or(Error::UnknownCommand)));
            thread::spawn(move || cmd_fn(argv, rx1, tx2));
            rx1 = rx2;
            let pipe = channel();
            tx2 = pipe.0;
            rx2 = pipe.1;
        }
        Ok(rx1.iter().collect())
    }

    /// Parse and run a source string.
    pub fn run_str(&self, source: &str) -> Result<Vec<String>> {
        let mut shlex = Shlex::new(source);
        let mut cmds = Vec::new();
        let mut cmd = Vec::new();
        for token in shlex.by_ref() {
            if token == "|" {
                cmds.push(cmd);
                cmd = Vec::new();
            } else {
                cmd.push(token);
            }
        }
        cmds.push(cmd);
        if shlex.had_error {
            Err(Error::ParseError)
        } else {
            self.run_cmds(cmds)
        }
    }
}
