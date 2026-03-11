use crate::timer::pomodoro::{Pomodoro, TimerState};
use std::time::Duration;
use tokio::time::sleep;

pub struct PomodoroTimer {
    pomodoro: Pomodoro,
    remaining_time: u16,
    cycles_completed: u8,
    state: TimerState,
    on_pause: bool,
}

impl PomodoroTimer {
    pub fn new(pomodoro: Pomodoro) -> Self {
        Self {
            remaining_time: pomodoro.work_duration,
            pomodoro,
            cycles_completed: 0,
            state: TimerState::Work,
            on_pause: false,
        }
    }

    pub fn tick(&mut self) {
        if self.on_pause {
            return;
        }

        if self.remaining_time > 0 {
            self.remaining_time -= 1;
            return;
        }

        match self.state {
            TimerState::Work => {
                self.cycles_completed += 1;

                if self.cycles_completed % self.pomodoro.cycles == 0 {
                    self.state = TimerState::LongBreak;
                    self.remaining_time = self.pomodoro.long_break_duration;
                } else {
                    self.state = TimerState::ShortBreak;
                    self.remaining_time = self.pomodoro.short_break_duration;
                }
            }

            TimerState::ShortBreak => {
                self.state = TimerState::Work;
                self.remaining_time = self.pomodoro.work_duration;
            }

            TimerState::LongBreak => {
                self.state = TimerState::Work;
                self.remaining_time = self.pomodoro.work_duration;
            }
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.tick();

            // Display current state
            println!(
                "State: {:?}, Remaining: {}s, Cycles: {}",
                self.state,
                self.remaining_time,
                self.cycles_completed,
            );

            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn pause(&mut self) {
        self.on_pause = true;
    }

    pub fn resume(&mut self) {
        self.on_pause = false;
    }

    // Getters
    pub fn remaining(&self) -> u16 {
        self.remaining_time
    }

    pub fn state(&self) -> TimerState {
        self.state
    }
}
