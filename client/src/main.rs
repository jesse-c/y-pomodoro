extern crate rmp_serde as rmps;

use pomodoro_core::{Command, CommandResult};
use rmps::decode;
use rmps::Serializer;
use serde::Serialize;
use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::str::FromStr;

const SOCKET_PATH: &str = "/tmp/pomodoro.sock";

fn main() {
    // Get the command, if any
    let command = env::args().nth(1).expect("no command given");

    // Parse the command
    let command = Command::from_str(&command).expect("invalid command given");

    let mut socket = match UnixStream::connect(SOCKET_PATH) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {:?}", e);
            return;
        }
    };

    let mut buf = Vec::new();
    command.serialize(&mut Serializer::new(&mut buf)).unwrap();

    socket.write_all(&buf).unwrap();

    let result: Result<CommandResult, decode::Error> = decode::from_read(socket);
    println!("Result: {:?}", result);
}
