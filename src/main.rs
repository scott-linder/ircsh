extern crate irc;
extern crate shlex;

mod sh;

use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::collections::HashMap;
use irc::client::data::command::Command;
use irc::client::data::message::Message;
use irc::client::server::{IrcServer, Server};
use irc::client::server::utils::ServerExt;
use sh::{Sh, CmdFn};

const LEADER: char = '~';

fn join_start_channels<S>(server: &S) -> io::Result<()>
        where S: Server {
    let channels = BufReader::new(try!(File::open("channels.txt")));
    for line in channels.lines() {
        try!(server.send_join(&*try!(line)));
    }
    Ok(())
}

static ECHO: &'static CmdFn = &|argv, _, tx| {
    for arg in argv {
        tx.send(arg).unwrap();
    }
};

static CAT: &'static CmdFn = &|_, rx, tx| {
    for line in rx.iter() {
        tx.send(line).unwrap();
    }
};

static COUNT: &'static CmdFn = &|argv, rx, tx| {
    tx.send(format!("{}", if argv.len() == 1 {
        rx.iter().count()
    } else {
        argv.len()
    })).unwrap()
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
            sh.cmds.insert("echo", ECHO);
            sh.cmds.insert("cat", CAT);
            sh.cmds.insert("count", COUNT);
            for message in rx.iter() {
                match message.command {
                    Command::PRIVMSG(ref target, ref msg) => {
                        let msg = msg.trim_left_matches(LEADER);
                        match sh.run_str(msg) {
                            Ok(rs) => for r in rs {
                                server.send(Command::PRIVMSG(target.clone(),
                                    r)).unwrap();
                            },
                            Err(e) => {
                                server.send(Command::PRIVMSG(target.clone(),
                                    format!("error: {}", e))).unwrap();
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
                if msg.starts_with(LEADER) {
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
