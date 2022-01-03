extern crate rmp_serde as rmps;
extern crate serde;
extern crate serde_derive;

use pomodoro_core::{Command, CommandResult, Pomodoro, State};
use rmps::decode;
use rmps::Serializer;
use serde::Serialize;
use std::fs;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/pomodoro.sock";

fn main() {
    let (tx1, rx1): (Sender<Command>, Receiver<Command>) = mpsc::channel();
    let (tx2, rx2): (Sender<CommandResult>, Receiver<CommandResult>) = mpsc::channel();

    let logic = thread::spawn(|| l(rx1, tx2));
    let server = thread::spawn(|| s(tx1, rx2));

    logic.join().unwrap();
    server.join().unwrap();
}

// # Logic
//
// This is the "brain" of the daemon. It processes and acts
// on commands.
fn l(rx1: Receiver<Command>, tx2: Sender<CommandResult>) {
    // Init Pomodoro
    // For now, we're keeping the state in the daemon's memory
    let mut pomodoro: Pomodoro = Pomodoro {
        state: State::Stopped,
        completed_count: 0,
        break_count: 0,
    };

    loop {
        // Give precedence to any command before possibly ticking
        let received = rx1.try_recv();

        pomodoro = match received {
            Ok(cmd) => pomodoro_core::do_command(cmd, pomodoro),
            // Nothing to do
            Err(_err) => pomodoro,
        };
        pomodoro = pomodoro_core::maybe_tick(pomodoro);
        pomodoro = pomodoro_core::do_next(pomodoro);

        // Only send a result if there was a command
        if let Ok(_cmd) = received {
            tx2.send(CommandResult::Success(pomodoro)).unwrap();
        };

        println!("Loop: {:?}", pomodoro);

        thread::sleep(Duration::from_secs(1));
    }
}

// # Server
//
// This is the bare-minimum server that listens for commands.
// It sends the commands along th the logic.
fn s(tx1: Sender<Command>, rx2: Receiver<CommandResult>) {
    // We're going to use this more than once
    let socket = Path::new(SOCKET_PATH);

    // Cleanup any previous leftover socket
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }

    let listener = UnixListener::bind(&socket).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut a = stream.try_clone().unwrap();
                let stream = BufReader::new(stream);

                let command: Result<Command, decode::Error> = decode::from_read(stream);

                match command {
                    Ok(command) => {
                        println!("Command: {:?}", command);
                        match tx1.send(command) {
                            Ok(_result) => match rx2.recv() {
                                Ok(command_result) => {
                                    let mut buf = Vec::new();
                                    command_result
                                        .serialize(&mut Serializer::new(&mut buf))
                                        .unwrap();

                                    a.write_all(&buf).unwrap();
                                }

                                Err(_err) => {
                                    let mut buf = Vec::new();
                                    CommandResult::Failure
                                        .serialize(&mut Serializer::new(&mut buf))
                                        .unwrap();

                                    a.write_all(&buf).unwrap();
                                }
                            },
                            Err(_err) => {
                                let mut buf = Vec::new();
                                CommandResult::Failure
                                    .serialize(&mut Serializer::new(&mut buf))
                                    .unwrap();

                                a.write_all(&buf).unwrap();
                            }
                        }
                    }
                    Err(err) => println!("Invalid command: {}", err),
                }

                continue;
            }
            Err(_err) => {
                continue;
            }
        }
    }
}
