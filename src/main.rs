extern crate ctrlc;
extern crate rusqlite;

use notify_rust::Notification;
use rusqlite::Connection;
use std::fs;
use std::io::{BufRead, BufReader, Result, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/pomodoro.sock";
const DB_PATH: &str = "./pomodoro.db";

// Status:
//
// 0: Stopped
// 1: Paused
// 2: Started
//
// CREATE TABLE IF NOT EXISTS "pomodoro" ("id" integer NOT NULL,"remaining" integer NOT NULL,"status" integer, PRIMARY KEY (id));
//
// cid         name        type        notnull     dflt_value  pk
// ----------  ----------  ----------  ----------  ----------  ----------
// 0           id          integer     1                       1
// 1           remaining   integer     1                       0
// 2           status      integer     0                       0

fn handle_client(stream: UnixStream) -> Result<()> {
    let mut a = stream.try_clone().unwrap();

    let mut stream = BufReader::new(stream);
    let mut command = String::new();

    stream.read_line(&mut command).unwrap();

    println!("Received command: {:#?}", command);

    let conn = Connection::open(Path::new(DB_PATH)).unwrap();

    match command.as_str() {
        "start" => {
            let query = "UPDATE pomodoro SET remaining = (25 * 60), status = 2 WHERE id = 1;";
            println!("Query: {:#?}", query);

            match conn.execute(query, []) {
                Ok(result) => println!("Command execute result: {}", result),
                Err(err) => println!("Command execute error: {}", err),
            }
        }
        "show" => {
            let query = "SELECT remaining, status FROM pomodoro WHERE id = 1;";
            println!("Query: {:#?}", query);

            let (remaining, status) = conn
                .query_row::<(u32, u32), _, _>(query, [], |row| {
                    Ok((row.get_unwrap(0), row.get_unwrap(1)))
                })
                .unwrap();

            println!("Command execute remaining: {}", remaining);
            println!("Command execute status: {}", status);

            let status_ = match status {
                0 => "Stopped",
                1 => "Paused",
                2 => "Started",
                _ => panic!("unknown status"),
            };

            let msg = format!("Status: {} / Remaining: {}m", status_, remaining / 60);

            a.write_all(msg.as_bytes()).unwrap();
        }
        "pause" => {
            let query = "UPDATE pomodoro SET status = 1 WHERE id = 1;";
            println!("Query: {:#?}", query);

            match conn.execute(query, []) {
                Ok(result) => println!("Command execute result: {}", result),
                Err(err) => println!("Command execute error: {}", err),
            }
        }
        "resume" => {
            let query = "UPDATE pomodoro SET status = 2 WHERE id = 1;";
            println!("Query: {:#?}", query);

            match conn.execute(query, []) {
                Ok(result) => println!("Command execute result: {}", result),
                Err(err) => println!("Command execute error: {}", err),
            }
        }
        "stop" => {
            let query = "UPDATE pomodoro SET status = 0 WHERE id = 1;";
            println!("Query: {:#?}", query);

            match conn.execute(query, []) {
                Ok(result) => println!("Command execute result: {}", result),
                Err(err) => println!("Command execute error: {}", err),
            }
        }
        _ => (),
    };

    Ok(())
}

fn shutdown(socket: &'static Path) {
    println!("Ctrl-C handled");
    println!("Cleaning up socket");

    match fs::remove_file(&socket) {
        Ok(_) => process::exit(0),
        Err(_) => process::exit(1),
    }
}

fn setup() -> Result<&'static Path> {
    let socket = Path::new(SOCKET_PATH);

    if socket.exists() {
        fs::remove_file(&socket)?;
    }

    Ok(socket)
}

fn cont(duration: Duration) {
    loop {
        println!("Loop");

        let conn = Connection::open(Path::new(DB_PATH)).unwrap();

        let query = "SELECT remaining, status FROM pomodoro WHERE id = 1;";

        let (remaining, status) = conn
            .query_row::<(u32, u32), _, _>(query, [], |row| {
                Ok((row.get_unwrap(0), row.get_unwrap(1)))
            })
            .unwrap();

        println!("Command execute remaining: {}", remaining);
        println!("Command execute status: {}", status);

        if remaining == 0 && status == 2 {
            Notification::new()
                .summary("Timer finished")
                .show()
                .unwrap();

            let query = "UPDATE pomodoro SET status = 0 WHERE id = 1;";
            println!("Query: {:#?}", query);

            match conn.execute(query, []) {
                Ok(result) => println!("Loop execute result: {}", result),
                Err(err) => println!("Loop execute error: {}", err),
            }
        } else {
            let query = "
            UPDATE pomodoro
            SET remaining = remaining - 1
            WHERE id = 1
            AND status = 2;
        ";

            match conn.execute(query, []) {
                Ok(result) => println!("Loop execute result: {}", result),
                Err(err) => println!("Loop execute error: {}", err),
            }
        }

        thread::sleep(duration);
    }
}

fn main() -> Result<()> {
    let socket = setup()?;

    ctrlc::set_handler(move || shutdown(socket)).expect("Error setting Ctrl-C handler");

    let duration = Duration::from_secs(1);

    let _handler = thread::spawn(move || cont(duration));

    let listener = UnixListener::bind(&socket)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(_err) => {
                break;
            }
        }
    }

    Ok(())
}
