use crate::timer::pomodoro::{Pomodoro, TimerState};
use crate::timer::input::{InputHandler, UserCommand};
use std::time::Duration;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::time::interval;
use tokio::sync::mpsc;
use crossterm::{terminal, cursor, ExecutableCommand};

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
        self.transition_to_next_state();
    }

    /// Transition to the next state when current phase completes
    fn transition_to_next_state(&mut self) {
        match self.state {
            TimerState::Work => {
                self.cycles_completed += 1;

                // Check if we should take a long break (every N-th work
                // session)
                if self.cycles_completed % self.pomodoro.cycles == 0 {
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

    /// Skip the current phase and move to the next one
    pub fn skip_phase(&mut self) {
        self.remaining_time = 0;
        self.transition_to_next_state();
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        // Enable raw mode for keyboard input
        terminal::enable_raw_mode()?;

        let result = self.run_loop().await;

        // Disable raw mode and cleanup
        let _ = terminal::disable_raw_mode();

        result
    }

    async fn run_loop(&mut self) -> std::io::Result<()> {
        let mut stdout = io::stdout();

        // Create a channel for input commands
        let (tx, mut rx) = mpsc::channel(1);

        // Create a flag to signal the input task to stop
        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = Arc::clone(&should_stop);

        // Spawn a separate task to poll input continuously
        let input_task = tokio::spawn(async move {
            loop {
                // Check if we should stop
                if *should_stop_clone.lock().unwrap() {
                    break;
                }

                // Poll input with a short timeout so it doesn't block the timer
                if let Ok(cmd) = InputHandler::poll_input(Duration::from_millis(50)) {
                    if cmd != UserCommand::None {
                        // Only send non-None commands
                        let _ = tx.send(cmd).await;
                    }
                }
            }
        });

        // Display initial state immediately on startup
        self.display(&mut stdout)?;

        // Create a 1-second interval timer
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                // Timer tick: fires every 1000ms
                _ = ticker.tick() => {
                    self.tick();
                    self.display(&mut stdout)?;
                }

                // Input command arrives from the polling task
                Some(cmd) = rx.recv() => {
                    match cmd {
                        UserCommand::TogglePause => {
                            self.toggle_pause();
                            self.display(&mut stdout)?;
                        }
                        UserCommand::Skip => {
                            self.skip_phase();
                            self.display(&mut stdout)?;
                        }
                        UserCommand::Quit => {
                            // Signal the input task to stop
                            *should_stop.lock().unwrap() = true;
                            // Wait for the task to finish
                            let _ = input_task.await;
                            stdout.execute(cursor::MoveToColumn(0))?;
                            writeln!(stdout)?;
                            stdout.execute(cursor::MoveToColumn(0))?;
                            writeln!(stdout, "Timer stopped. Goodbye!")?;
                            stdout.execute(cursor::MoveToColumn(0))?;
                            writeln!(stdout)?;
                            stdout.flush()?;
                            return Ok(());
                        }
                        UserCommand::None => {
                            // Should never happen since we filter these out
                        }
                    }
                }
            }
        }
    }

    /// Display the current timer state
    fn display(&self, stdout: &mut io::Stdout) -> std::io::Result<()> {
        let formatted_time = self.format_time(self.remaining_time);
        let state_name = self.state_display();
        let current_cycle = if self.state == TimerState::Work {
            self.cycles_completed + 1
        } else {
            self.cycles_completed
        };

        let status = if self.on_pause { "PAUSED" } else { "RUNNING" };

        // Use carriage return to overwrite the current line
        write!(
            stdout,
            "\r[{}] {} | Cycle {}/{} | {} | (p)ause (s)kip (q)uit   ",
            state_name, formatted_time, current_cycle, self.pomodoro.cycles, status
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Formats seconds as MM:SS
    fn format_time(&self, seconds: u16) -> String {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }

    /// Get human-readable state name
    fn state_display(&self) -> &str {
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

    pub fn toggle_pause(&mut self) {
        self.on_pause = !self.on_pause;
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
