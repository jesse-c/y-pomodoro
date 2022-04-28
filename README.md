# Pomodoro

- Pomodoro time: 25 minutes
- Break time: 5 minutes
- Long break time: 15 minutes
- Long break each: 4 Pomodoros

## Design

The experiment is to separate the system from the client. That means that you can interact with the system through different locations. You could have a browser extension, a Raycast extension, a menu bar app, etc.

**What's next?**

- Add another communication channel that is easy to build a Swift client (with notifications) in.
- Add notifications

### Core

This contains the reusable logic around the daemon.

### Daemon

This is the background system to manage the timer. The daemon has `n` available comunication channels. At the moment it's just Unix sockets.

### Client

There's just 1 client at the moment. It communicates over the one available communication channel.

## Usage

Start the daemon with: `cargo run --bin pomodoro_daemon`. You then run commands with: `cargo run --bin pomodoro_client COMMAND`.

Available commands:

- `pause`: Pause the running timer, if any
- `resume`: Resume the paused timer, if any
- `start`: Start a new timer
- `stop`: Stop the timer
- `show`: Show the status of the current timer, if any
