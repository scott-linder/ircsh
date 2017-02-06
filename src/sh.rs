use std::{error, result};
use std::fmt;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use shlex::Shlex;

pub type Result<T> = result::Result<T, Error>;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Error {
    EmptyCommand,
    UnknownCommand(String),
    ParseError,
    CommandErrors(Vec<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Error::EmptyCommand
                | Error::ParseError => error::Error::description(self).into(),
            Error::UnknownCommand(ref s) => format!("Unknown command \"{}\".", s),
            Error::CommandErrors(ref es) => es.join(" | "),
        })
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EmptyCommand => "Empty command",
            Error::UnknownCommand(..) => "Unknown command",
            Error::ParseError => "Parse error",
            Error::CommandErrors(..) => "Command error",
        }
    }
}

pub type CmdFn = Fn(Vec<String>,
                    Receiver<String>,
                    Sender<String>,
                    Sender<String>) + Send + Sync + 'static;

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
        let (stderr_out, stderr_in) = channel::<String>();
        let (_, mut stdin) = channel::<String>();
        let (mut stdout, mut stdin_next) = channel::<String>();
        for argv in argvs {
            let cmd_fn = *try!(argv.first()
                                   .ok_or(Error::EmptyCommand)
                                   .and_then(|name|
                                        self.cmds.get(name.as_str())
                                                 .ok_or(Error::UnknownCommand(name.clone()))));
            let stderr = stderr_out.clone();
            thread::spawn(move || cmd_fn(argv, stdin, stdout, stderr));
            stdin = stdin_next;
            let pipe = channel();
            stdout = pipe.0;
            stdin_next = pipe.1;
        }
        drop(stderr_out);
        let stderrs = stderr_in.into_iter().collect::<Vec<_>>();
        if stderrs.len() != 0 {
            Err(Error::CommandErrors(stderrs))
        } else {
            Ok(stdin.into_iter().collect())
        }
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
