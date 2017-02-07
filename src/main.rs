extern crate irc;
extern crate shlex;
extern crate getopts;
extern crate redis;
#[macro_use]
extern crate lazy_static;

mod sh;

use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::collections::HashMap;
use std::env;
use irc::client::data::command::Command;
use irc::client::data::message::Message;
use irc::client::server::{IrcServer, Server};
use irc::client::server::utils::ServerExt;
use getopts::Options;
use redis::Commands;
use sh::{Sh, CmdFn};

lazy_static! {
    static ref REDIS: redis::Client = redis::Client::open(env::var("REDIS_URL").unwrap().as_str()).unwrap();
    static ref LEADER: String = REDIS.get::<_, String>("leader").unwrap_or("$".into());
}

fn join_start_channels<S>(server: &S) -> io::Result<()>
        where S: Server {
    let channels = BufReader::new(try!(File::open("channels.txt")));
    for line in channels.lines() {
        try!(server.send_join(&*try!(line)));
    }
    Ok(())
}

macro_rules! ignore {
    ($expression:expr) => (
        match $expression {
            Ok(o) => o,
            Err(..) => return,
        }
    )
}

macro_rules! try_or_send {
    ($expr:expr, $stderr:ident, $argv:ident) => (
        match $expr {
            Ok(m) => m,
            Err(f) => {
                ignore!($stderr.send(format!("{}: {}", $argv[0], f)));
                return
            },
        }
    )
}

static GET: &'static CmdFn = &|argv, _, stdout, stderr| {
    let conn = try_or_send!(REDIS.get_connection(), stderr, argv);
    ignore!(stdout.send(ignore!(conn.get(&argv[1]))))
};

static SET: &'static CmdFn = &|argv, _, stdout, stderr| {
    let conn = try_or_send!(REDIS.get_connection(), stderr, argv);
    ignore!(stdout.send(ignore!(conn.set(&argv[1], &argv[2]))))
};

static FLAGS: &'static CmdFn = &|argv, _, stdout, stderr| {
    let mut opts = Options::new();
    opts.optflag("t", "test", "test flag");
    let matches = try_or_send!(opts.parse(&argv[1..]), stderr, argv);
    ignore!(stdout.send(if matches.opt_present("t") {
        "test"
    } else {
        "not test"
    }.into()))
};

static ECHO: &'static CmdFn = &|mut argv, _, stdout, _| {
    ignore!(stdout.send(argv.split_off(1).join(" ")))
};

static CAT: &'static CmdFn = &|_, stdin, stdout, _| {
    for line in stdin.iter() {
        ignore!(stdout.send(line))
    }
};

static COUNT: &'static CmdFn = &|argv, stdin, stdout, _| {
    ignore!(stdout.send(format!("{}", if argv.len() == 1 {
        stdin.iter().count()
    } else {
        argv.len() - 1
    })))
};

fn find_or_spawn<'a, S>(server: &IrcServer,
                     user_senders: &'a mut HashMap<String, Sender<Message>>,
                     nick: S) -> &'a mut Sender<Message>
        where S: Into<String> {
    let nick = nick.into();
    user_senders.entry(nick.clone()).or_insert_with(|| {
        let (tx, rx) = channel::<Message>();
        let server = server.clone();
        thread::spawn(move || {
            let mut sh = Sh::new();
            sh.cmds.insert("get", GET);
            sh.cmds.insert("set", SET);
            sh.cmds.insert("flags", FLAGS);
            sh.cmds.insert("echo", ECHO);
            sh.cmds.insert("cat", CAT);
            sh.cmds.insert("count", COUNT);
            for message in rx.iter() {
                match message.command {
                    Command::PRIVMSG(ref target, ref msg) => {
                        let msg = msg.trim_left_matches(LEADER.as_str());
                        match sh.run_str(msg) {
                            Ok(rs) => {
                                server.send(Command::PRIVMSG(target.clone(),
                                    format!("{}$ {}", nick, rs.join(" | ")))).unwrap();
                            },
                            Err(e) => {
                                server.send(Command::PRIVMSG(target.clone(),
                                    format!("{}! {}", nick, e))).unwrap();
                            }
                        }
                    },
                    _ => (),
                }
            }
        });
        tx
    })
}

fn main() {
    let server = IrcServer::new("config.json").unwrap();
    server.identify().unwrap();
    join_start_channels(&server).unwrap();
    let mut user_senders = HashMap::new();

    for message in server.iter() {
        let message = message.unwrap();
        match message.command {
            Command::PRIVMSG(_, ref msg) => {
                if msg.starts_with(LEADER.as_str()) {
                    if let Some(chan) = message.source_nickname()
                            .map(|nick| find_or_spawn(&server,
                                                      &mut user_senders,
                                                      nick)) {
                        chan.send(message.clone()).unwrap();
                    }
                }
            },
            _ => (),
        }
    }
}
