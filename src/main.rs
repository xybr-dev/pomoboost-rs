use tokio::time::sleep;
use std::time::Duration;
use crate::timer::pomodoro::{Pomodoro, TimerState};
use crate::timer::engine::PomodoroTimer;

#[tokio::main]
async fn main() {
    let pomodoro = Pomodoro {
        work_duration: 5,
        short_break_duration: 5,
        long_break_duration: 5,
        cycles: 4,
    };

    let mut timer = PomodoroTimer::new(pomodoro);
    timer.run().await;
}

mod timer {
    pub mod pomodoro;
    pub mod engine;
}
