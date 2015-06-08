# ircsh #

A shell for IRC.

# Usage #

Create a `config.json` file with the IRC configuration for the bot:

    {
        "nickname": "sh",
        "username": "sh",
        "realname": "sh",
        "server": "localhost",
        "port": 6667
    }

Create a `channels.txt` file containing a list of channels, one per line, to
join when the bot connects to the server:

    #test

Start the bot:

    $ cargo run --release

## Shell Language ##

The shell language is presently rudimentary.

One begins a command to the bot with the leader `$`, followed by a command-line
consisting of commands separated (but not terminated) by `;` where each command
is a list of (optionally quoted) strings. String escape sequences are not yet
supported.

Planned features include piping, redirection, flow control, and some form of
control characters (most likely support for at least restarting the shell
instance for a user).

## Shell Semantics ##

The main bot thread decodes the user's nick and uses it to route the message to
a thread unique to that nick. Thus, each nick has its own independent instance
of the shell which does not block other instances.

Each shell instance parses the command-line into its constituent commands and
runs each in turn, providing one line of output (prefixed with the user's nick)
for each command.

## Rationale ##

It's fun to write a shell, but hard to match the utility of any existing OS
shell. IRC seems to be a clean slate. Additionally, bots tend naturally towards
improved programmability to ease the load of the initial bot writer anyway,
so why not provide a rich programming environment directly from the IRC
channel?
