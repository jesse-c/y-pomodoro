use serde::{Deserialize, Serialize};
use std::str;
use std::time::Duration;

const SET: u64 = 4;
const TIMER_LENGTH_IN_MINUTES: u64 = 25;
const SHORT_BREAK_LENGTH_IN_MINUTES: u64 = 5;
const LONG_BREAK_LENGTH_IN_MINUTES: u64 = 20;

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum Command {
    Pause,
    Resume,
    Show,
    Stop,
    Start,
    SkipBreak,
    Reset,
}

impl str::FromStr for Command {
    // Useless for now
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "pause" => Ok(Command::Pause),
            "resume" => Ok(Command::Resume),
            "stop" => Ok(Command::Stop),
            "show" => Ok(Command::Show),
            "start" => Ok(Command::Start),
            "skipbreak" => Ok(Command::SkipBreak),
            "reset" => Ok(Command::Reset),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum CommandResult {
    Success(Pomodoro),
    Failure,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum State {
    Paused(Duration),
    Stopped,
    Working(Duration),
    TakingShortBreak(Duration),
    TakingLongBreak(Duration),
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Pomodoro {
    pub state: State,
    pub completed_count: u64,
    pub break_count: u64,
}

pub fn do_command(cmd: Command, pomodoro: Pomodoro) -> Pomodoro {
    match cmd {
        Command::Pause => {
            // You can only pause when it's working
            if let State::Working(duration) = pomodoro.state {
                Pomodoro {
                    state: State::Paused(duration),
                    ..pomodoro
                }
            // Not working, so do nothing
            } else {
                pomodoro
            }
        }
        Command::Resume => {
            // You can only resume when it's working
            if let State::Paused(duration) = pomodoro.state {
                Pomodoro {
                    state: State::Working(duration),
                    ..pomodoro
                }
            // Not working, so do nothing
            } else {
                pomodoro
            }
        }
        Command::Stop => Pomodoro {
            state: State::Stopped,
            ..pomodoro
        },
        Command::Start => Pomodoro {
            state: State::Working(Duration::from_secs(0)),
            ..pomodoro
        },
        Command::SkipBreak => Pomodoro {
            state: State::Stopped,
            break_count: pomodoro.break_count + 1,
            ..pomodoro
        },
        Command::Reset => Pomodoro {
            state: State::Stopped,
            completed_count: 0,
            break_count: 0,
        },
        Command::Show => pomodoro,
    }
}

// If working or paused, tick
pub fn maybe_tick(pomodoro: Pomodoro) -> Pomodoro {
    match pomodoro.state {
        State::Working(duration) => Pomodoro {
            state: State::Working(duration + Duration::from_secs(1)),
            ..pomodoro
        },
        State::TakingShortBreak(duration) => Pomodoro {
            state: State::TakingShortBreak(duration + Duration::from_secs(1)),
            ..pomodoro
        },
        State::TakingLongBreak(duration) => Pomodoro {
            state: State::TakingLongBreak(duration + Duration::from_secs(1)),
            ..pomodoro
        },
        _ => pomodoro,
    }
}
pub fn do_next(pomodoro: Pomodoro) -> Pomodoro {
    match pomodoro.state {
        // Check if we've reached the end of the working timer
        State::Working(duration) => {
            if duration == Duration::from_secs(60 * TIMER_LENGTH_IN_MINUTES) {
                // We've reached the end of the timer
                let break_count = pomodoro.break_count + 1;
                let state: State = if break_count == SET {
                    State::TakingLongBreak(Duration::from_secs(0))
                } else {
                    State::TakingShortBreak(Duration::from_secs(0))
                };

                Pomodoro {
                    state,
                    break_count,
                    ..pomodoro
                }
            } else {
                pomodoro
            }
        }
        // Check if we've reached the end of the short break timer
        State::TakingShortBreak(duration) => {
            if duration == Duration::from_secs(60 * SHORT_BREAK_LENGTH_IN_MINUTES) {
                // Continue into the next working state
                let state: State = State::Working(Duration::from_secs(0));

                Pomodoro { state, ..pomodoro }
            } else {
                pomodoro
            }
        }
        // Check if we've reached the end of the long break timer
        State::TakingLongBreak(duration) => {
            if duration == Duration::from_secs(60 * LONG_BREAK_LENGTH_IN_MINUTES) {
                // Don't do anything
                let state: State = State::Stopped;

                Pomodoro { state, ..pomodoro }
            } else {
                pomodoro
            }
        }

        _ => pomodoro,
    }
}
