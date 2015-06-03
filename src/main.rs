extern crate irc;

mod sh;
mod replace;

use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::collections::HashMap;
use irc::client::data::command::Command;
use irc::client::data::message::Message;
use irc::client::server::{IrcServer, NetIrcServer, Server};
use irc::client::server::utils::ServerExt;
use irc::client::data::kinds::{IrcRead, IrcWrite};
use sh::Lex;


fn join_start_channels<T, U>(server: &IrcServer<T, U>) -> io::Result<()>
        where T: IrcRead, U: IrcWrite {
    let channels = BufReader::new(try!(File::open("channels.txt")));
    for line in channels.lines() {
        try!(server.send_join(&*try!(line)));
    }
    Ok(())
}

fn find_or_spawn<'a, S>(arc_irc_server: &Arc<NetIrcServer>,
                     user_senders: &'a mut HashMap<String, Sender<Message>>,
                     nick: S) -> &'a mut Sender<Message>
        where S: Into<String> {
    user_senders.entry(nick.into()).or_insert_with(|| {
        let (tx, rx) = channel();
        let irc_server = arc_irc_server.clone();
        thread::spawn(move || {
            for message in rx.iter() {
                if let Ok(command) = Command::from_message(&message) {
                    match command {
                        Command::PRIVMSG(target, msg) => {
                            let lex = Lex::new(&*msg);
                            let resp = format!("{:?}", lex.collect::<Vec<_>>());
                            irc_server.send(Command::PRIVMSG(target, resp)).unwrap();
                        },
                        _ => (),
                    }
                }
            }
        });
        tx
    })
}

fn main() {
    let arc_irc_server = Arc::new(IrcServer::new("config.json").unwrap());
    let server = arc_irc_server.clone();
    server.identify().unwrap();
    join_start_channels(&server).unwrap();
    let mut user_senders = HashMap::new();

    for message in server.iter() {
        let message = message.unwrap();
        if let Ok(command) = Command::from_message(&message) {
            match command {
                Command::PRIVMSG(_, msg) => {
                    if msg.starts_with("#") {
                        if let Some(chan) = message.get_source_nickname()
                                .map(|nick| find_or_spawn(&arc_irc_server,
                                                          &mut user_senders,
                                                          nick)) {
                            chan.send(message).unwrap();
                        }
                    }
                },
                _ => (),
            }
        }
    }
}
