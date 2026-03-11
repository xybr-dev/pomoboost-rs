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

    /// Decrements the timer and transitions states when a phase ends.
    /// This is called once per second.
    pub fn tick(&mut self) {
        if self.on_pause {
            return;
        }

        // Only count down if time remains
        if self.remaining_time > 0 {
            self.remaining_time -= 1;
            return;
        }

        // Time has reached 0, transition to next state
        match self.state {
            TimerState::Work => {
                self.cycles_completed += 1;

                // After N work sessions, take a long break; otherwise short break
                if self.cycles_completed >= self.pomodoro.cycles {
                    // Completed all cycles, restart
                    self.cycles_completed = 0;
                    self.state = TimerState::Work;
                    self.remaining_time = self.pomodoro.work_duration;
                } else if self.cycles_completed % self.pomodoro.cycles == 0 {
                    // Every N-th work session is followed by a long break
                    self.state = TimerState::LongBreak;
                    self.remaining_time = self.pomodoro.long_break_duration;
                } else {
                    // Regular work session followed by short break
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

            // Display current state with formatted time
            let formatted_time = self.format_time(self.remaining_time);
            let state_name = self.state_display();
            let current_cycle = if self.state == TimerState::Work {
                self.cycles_completed + 1
            } else {
                self.cycles_completed
            };

            println!(
                "[{}] {} | Cycle {}/{} | Pause: {}",
                state_name,
                formatted_time,
                current_cycle,
                self.pomodoro.cycles,
                if self.on_pause { "YES" } else { "NO" }
            );

            sleep(Duration::from_secs(1)).await;
        }
    }

    /// Formats seconds as MM:SS
    fn format_time(&self, seconds: u16) -> String {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }

    /// Get human-readable state name
    fn state_display(&self) -> &'static str {
        match self.state {
            TimerState::Work => "WORK",
            TimerState::ShortBreak => "SHORT BREAK",
            TimerState::LongBreak => "LONG BREAK",
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

    pub fn cycles_completed(&self) -> u8 {
        self.cycles_completed
    }

    pub fn is_paused(&self) -> bool {
        self.on_pause
    }
}
