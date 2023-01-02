extern crate serde;
extern crate ws;

use pomodoro_core::{Command, CommandResult, Output, Pomodoro, Response, State};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

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
// It sends the commands along the the logic.
fn s(tx1: Sender<Command>, rx2: Receiver<CommandResult>) {
    websocket_server(tx1, rx2);
}

// This is the bare-minimum server that listens for commands.
struct Server<'a> {
    out: ws::Sender,
    tx1: &'a Sender<Command>,
    rx2: &'a Receiver<CommandResult>,
}

impl ws::Handler for Server<'_> // Use a generic lifetime
{
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Message received: {:?}", msg);
        match msg {
            ws::Message::Text(text) => {
                println!("Supported message type");

                let command = Command::from_str(&text);

                match command {
                    Ok(command) => {
                        println!("Command: {:?}", command);

                        match self.tx1.send(command) {
                            Ok(_result) => match self.rx2.recv() {
                                Ok(command_result) => match command_result {
                                    CommandResult::Success(pomodoro) => {
                                        let buf = serde_json::to_string(&Response {
                                            command: command,
                                            result: Output::Success,
                                            pomodoro: Some(pomodoro),
                                        })
                                        .unwrap();
                                        println!("{}", buf);

                                        let x = ws::Message::text(buf);

                                        self.out.send(x)
                                    }
                                    CommandResult::Failure => {
                                        let buf = serde_json::to_string(&Response {
                                            command: command,
                                            result: Output::Failure,
                                            pomodoro: None,
                                        })
                                        .unwrap();
                                        println!("{}", buf);

                                        let x = ws::Message::text(buf);

                                        self.out.send(x)
                                    }
                                },
                                Err(_err) => {
                                    let buf = serde_json::to_string(&Response {
                                        command: command,
                                        result: Output::Failure,
                                        pomodoro: None,
                                    })
                                    .unwrap();
                                    println!("{}", buf);

                                    let x = ws::Message::text(buf);

                                    self.out.send(x)
                                }
                            },
                            Err(_err) => {
                                let buf = serde_json::to_string(&Response {
                                    command: command,
                                    result: Output::Failure,
                                    pomodoro: None,
                                })
                                .unwrap();
                                println!("{}", buf);

                                let x = ws::Message::text(buf);

                                self.out.send(x)
                            }
                        }
                    }
                    Err(err) => {
                        println!("Invalid command: {:#?}", err);
                        self.out.send("Invalid command")
                    }
                }
            }
            ws::Message::Binary(_binary) => {
                println!("Unsupported message type");

                self.out.send("Unsupported message value")
            }
        }
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        println!("WebSocket closed. Code: {:?}, Reason: {:?}", code, reason);
    }
}

fn websocket_server(tx1: Sender<Command>, rx2: Receiver<CommandResult>) {
    ws::listen("127.0.0.1:3012", |out| Server {
        out: out,
        tx1: &tx1,
        rx2: &rx2,
    })
    .unwrap()
}
