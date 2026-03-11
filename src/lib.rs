pub mod timer {
    pub mod pomodoro;
    pub mod engine;
    pub mod input;
    
    pub use pomodoro::{Pomodoro, TimerState};
    pub use engine::PomodoroTimer;
    pub use input::{InputHandler, UserCommand};
}

#[cfg(test)]
mod tests {
    use crate::timer::pomodoro::Pomodoro;
    use crate::timer::engine::PomodoroTimer;
    use crate::timer::pomodoro::TimerState;

    #[test]
    fn test_pomodoro_creation_valid() {
        let result = Pomodoro::new(25 * 60, 5 * 60, 15 * 60, 4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pomodoro_creation_invalid_work_duration() {
        let result = Pomodoro::new(0, 5 * 60, 15 * 60, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_pomodoro_creation_invalid_cycles() {
        let result = Pomodoro::new(25 * 60, 5 * 60, 15 * 60, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_timer_initialization() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.remaining(), 5);
        assert_eq!(timer.state(), TimerState::Work);
        assert!(!timer.is_paused());
        assert_eq!(timer.cycles_completed(), 0);
    }

    #[test]
    fn test_timer_countdown() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // First tick should decrement remaining time
        timer.tick();
        assert_eq!(timer.remaining(), 4);
        assert_eq!(timer.state(), TimerState::Work);

        // Tick 3 more times
        timer.tick();
        timer.tick();
        timer.tick();
        assert_eq!(timer.remaining(), 1);

        // Final tick should transition to short break
        timer.tick();
        assert_eq!(timer.remaining(), 0);
        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_timer_state_transition_work_to_break() {
        let pomodoro = Pomodoro::new(2, 2, 3, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Exhaust work duration
        timer.tick(); // 1
        timer.tick(); // 0
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.cycles_completed(), 0);

        // Next tick should transition
        timer.tick();
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.cycles_completed(), 1);
        assert_eq!(timer.remaining(), 2);
    }

    #[test]
    fn test_pause_and_resume() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        timer.pause();
        assert!(timer.is_paused());

        // Pause prevents countdown
        timer.tick();
        assert_eq!(timer.remaining(), 5);

        timer.resume();
        assert!(!timer.is_paused());

        // After resume, countdown works again
        timer.tick();
        assert_eq!(timer.remaining(), 4);
    }

    #[test]
    fn test_toggle_pause() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert!(!timer.is_paused());
        timer.toggle_pause();
        assert!(timer.is_paused());
        timer.toggle_pause();
        assert!(!timer.is_paused());
    }

    #[test]
    fn test_skip_phase() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.state(), TimerState::Work);
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.remaining(), 3);
    }

    #[test]
    fn test_full_cycle_sequence() {
        let pomodoro = Pomodoro::new(1, 1, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Work 1
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.cycles_completed(), 0);
        timer.tick(); // remaining=0
        timer.tick(); // transition to ShortBreak
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.cycles_completed(), 1);

        // Short Break 1
        timer.tick(); // remaining=0
        timer.tick(); // transition to Work 2
        assert_eq!(timer.state(), TimerState::Work);

        // Work 2
        timer.tick(); // remaining=0
        timer.tick(); // transition to LongBreak (because 2 % 2 == 0)
        assert_eq!(timer.state(), TimerState::LongBreak);
        assert_eq!(timer.cycles_completed(), 2);
        assert_eq!(timer.remaining(), 2);

        // Long Break
        timer.tick(); // remaining=1
        timer.tick(); // remaining=0
        timer.tick(); // transition back to Work
        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_long_break_after_cycles() {
        let pomodoro = Pomodoro::new(1, 1, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Work 1: remaining goes from 1 -> 0
        timer.tick(); // remaining = 0
        
        // Transition to Short Break 1
        timer.tick(); // remaining = 1 (short break starts)
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.cycles_completed(), 1);
        
        // Short Break 1: remaining goes from 1 -> 0
        timer.tick(); // remaining = 0
        
        // Transition to Work 2
        timer.tick(); // remaining = 1 (work 2 starts)
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.cycles_completed(), 1);
        
        // Work 2: remaining goes from 1 -> 0
        timer.tick(); // remaining = 0
        
        // Transition to Long Break (because cycles_completed will be 2, and 2 % 2 == 0)
        timer.tick(); // should transition to LongBreak
        assert_eq!(timer.state(), TimerState::LongBreak);
        assert_eq!(timer.cycles_completed(), 2);
        assert_eq!(timer.remaining(), 2);
    }
}
