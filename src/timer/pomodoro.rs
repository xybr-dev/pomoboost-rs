#[derive(Debug, Clone, Copy)]
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
