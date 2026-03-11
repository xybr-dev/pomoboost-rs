#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    Work,
    ShortBreak,
    LongBreak,
}

/// Represents a Pomodoro timer with work,
/// short break, and long break durations.
/// Represented in seconds.
#[derive(Debug, Clone, Copy)]
pub struct Pomodoro {
    pub work_duration: u16,
    pub short_break_duration: u16,
    pub long_break_duration: u16,
    pub cycles: u8,
}

impl Pomodoro {
    /// Creates a new Pomodoro configuration with validation.
    pub fn new(
        work_duration: u16,
        short_break_duration: u16,
        long_break_duration: u16,
        cycles: u8,
    ) -> Result<Self, String> {
        if work_duration == 0 {
            return Err("work_duration must be greater than 0".to_string());
        }
        if short_break_duration == 0 {
            return Err("short_break_duration must be greater than 0".to_string());
        }
        if long_break_duration == 0 {
            return Err("long_break_duration must be greater than 0".to_string());
        }
        if cycles == 0 {
            return Err("cycles must be greater than 0".to_string());
        }

        Ok(Self {
            work_duration,
            short_break_duration,
            long_break_duration,
            cycles,
        })
    }

    /// Get the total number of pomodoros (work + break cycles)
    pub fn total_pomodoros(&self) -> u8 {
        self.cycles
    }

    /// Get duration for a given timer state
    pub fn duration_for_state(&self, state: TimerState) -> u16 {
        match state {
            TimerState::Work => self.work_duration,
            TimerState::ShortBreak => self.short_break_duration,
            TimerState::LongBreak => self.long_break_duration,
        }
    }
}
