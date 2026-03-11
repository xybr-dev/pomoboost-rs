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

    // ============================================================================
    // POMODORO CONFIGURATION TESTS
    // ============================================================================

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
    fn test_pomodoro_creation_invalid_short_break() {
        let result = Pomodoro::new(25 * 60, 0, 15 * 60, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_pomodoro_creation_invalid_long_break() {
        let result = Pomodoro::new(25 * 60, 5 * 60, 0, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_pomodoro_creation_invalid_cycles() {
        let result = Pomodoro::new(25 * 60, 5 * 60, 15 * 60, 0);
        assert!(result.is_err());
    }

    // ============================================================================
    // TIMER INITIALIZATION TESTS
    // ============================================================================

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
    fn test_timer_starts_in_work_state() {
        let pomodoro = Pomodoro::new(10, 5, 10, 2).unwrap();
        let timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.remaining(), 10);
    }

    #[test]
    fn test_timer_initial_pause_state_is_running() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let timer = PomodoroTimer::new(pomodoro);

        assert!(!timer.is_paused());
    }

    // ============================================================================
    // BASIC COUNTDOWN TESTS
    // ============================================================================

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

        // One more tick to reach 0
        timer.tick();
        assert_eq!(timer.remaining(), 0);
        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_timer_countdown_single_second() {
        let pomodoro = Pomodoro::new(1, 1, 1, 1).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.remaining(), 1);
        timer.tick();
        assert_eq!(timer.remaining(), 0);
    }

    #[test]
    fn test_timer_countdown_respects_work_duration() {
        let pomodoro = Pomodoro::new(3, 2, 2, 1).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.remaining(), 3);
        timer.tick(); // 2
        timer.tick(); // 1
        timer.tick(); // 0 (still in Work state before transition)
        assert_eq!(timer.remaining(), 0);
        assert_eq!(timer.state(), TimerState::Work);
    }

    // ============================================================================
    // STATE TRANSITION TESTS
    // ============================================================================

    #[test]
    fn test_timer_state_transition_work_to_short_break() {
        let pomodoro = Pomodoro::new(2, 2, 3, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Exhaust work duration
        timer.tick(); // remaining=1
        timer.tick(); // remaining=0
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.cycles_completed(), 0);

        // Next tick should transition to ShortBreak
        timer.tick();
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.cycles_completed(), 1);
        assert_eq!(timer.remaining(), 2);
    }

    #[test]
    fn test_timer_state_transition_short_break_to_work() {
        let pomodoro = Pomodoro::new(1, 1, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Move through work phase
        timer.tick(); // remaining=0
        timer.tick(); // transition to ShortBreak, remaining=1
        assert_eq!(timer.state(), TimerState::ShortBreak);

        // Complete short break
        timer.tick(); // remaining=0
        timer.tick(); // transition to Work
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.remaining(), 1);
    }

    #[test]
    fn test_timer_state_transition_to_long_break() {
        let pomodoro = Pomodoro::new(1, 1, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // First cycle: Work 1 -> Short Break 1 -> Work 2
        timer.tick(); // Work: 1 -> 0
        timer.tick(); // Short Break: 1 -> cycles=1
        timer.tick(); // Short Break: 1 -> 0
        timer.tick(); // Work 2: 1 -> cycles=1

        // Complete second work session
        timer.tick(); // Work 2: remaining -> 0

        // Next tick should transition to LongBreak (because cycles_completed==2, and 2%2==0)
        timer.tick();
        assert_eq!(timer.state(), TimerState::LongBreak);
        assert_eq!(timer.cycles_completed(), 2);
        assert_eq!(timer.remaining(), 2);
    }

    #[test]
    fn test_timer_state_transition_long_break_to_work() {
        let pomodoro = Pomodoro::new(1, 1, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Navigate to long break state
        for _ in 0..6 {
            timer.tick();
        }
        assert_eq!(timer.state(), TimerState::LongBreak);

        // Complete long break
        timer.tick(); // remaining=1
        timer.tick(); // remaining=0
        timer.tick(); // transition to Work
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.remaining(), 1);
    }

    // ============================================================================
    // PAUSE/RESUME TESTS
    // ============================================================================

    #[test]
    fn test_pause_prevents_countdown() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        timer.pause();
        assert!(timer.is_paused());

        // Pause prevents countdown
        timer.tick();
        assert_eq!(timer.remaining(), 5);
        assert_eq!(timer.state(), TimerState::Work);
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
    fn test_toggle_pause_on() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert!(!timer.is_paused());
        timer.toggle_pause();
        assert!(timer.is_paused());
    }

    #[test]
    fn test_toggle_pause_off() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        timer.pause();
        assert!(timer.is_paused());
        timer.toggle_pause();
        assert!(!timer.is_paused());
    }

    #[test]
    fn test_toggle_pause_multiple_times() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert!(!timer.is_paused());
        timer.toggle_pause();
        assert!(timer.is_paused());
        timer.toggle_pause();
        assert!(!timer.is_paused());
        timer.toggle_pause();
        assert!(timer.is_paused());
    }

    #[test]
    fn test_pause_during_break() {
        let pomodoro = Pomodoro::new(1, 2, 3, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Move to short break
        timer.tick(); // remaining=0
        timer.tick(); // transition to ShortBreak, remaining=2

        assert_eq!(timer.state(), TimerState::ShortBreak);

        // Pause during break
        timer.pause();
        timer.tick();
        assert_eq!(timer.remaining(), 2); // Should not decrement

        // Resume and continue
        timer.resume();
        timer.tick();
        assert_eq!(timer.remaining(), 1);
    }

    #[test]
    fn test_multiple_pause_resume_cycles() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Pause, resume, pause, resume
        timer.pause();
        timer.tick();
        assert_eq!(timer.remaining(), 5);

        timer.resume();
        timer.tick();
        assert_eq!(timer.remaining(), 4);

        timer.pause();
        timer.tick();
        assert_eq!(timer.remaining(), 4);

        timer.resume();
        timer.tick();
        assert_eq!(timer.remaining(), 3);
    }

    // ============================================================================
    // SKIP PHASE TESTS
    // ============================================================================

    #[test]
    fn test_skip_phase_from_work() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.state(), TimerState::Work);
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.remaining(), 3);
    }

    #[test]
    fn test_skip_phase_from_short_break() {
        let pomodoro = Pomodoro::new(1, 3, 7, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Move to short break
        timer.tick();
        timer.tick();
        assert_eq!(timer.state(), TimerState::ShortBreak);

        // Skip short break
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.remaining(), 1);
    }

    #[test]
    fn test_skip_phase_from_long_break() {
        let pomodoro = Pomodoro::new(1, 1, 3, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Navigate to long break
        for _ in 0..6 {
            timer.tick();
        }
        assert_eq!(timer.state(), TimerState::LongBreak);

        // Skip long break
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::Work);
        assert_eq!(timer.remaining(), 1);
    }

    #[test]
    fn test_skip_phase_during_pause() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        timer.pause();
        timer.skip_phase();

        // Skip should work even when paused
        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.remaining(), 3);
    }

    #[test]
    fn test_skip_phase_multiple_times() {
        let pomodoro = Pomodoro::new(1, 1, 1, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.state(), TimerState::Work);
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::ShortBreak);

        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::Work);

        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::LongBreak);

        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::Work);
    }

    // ============================================================================
    // FULL CYCLE SEQUENCE TESTS
    // ============================================================================

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

    #[test]
    fn test_multiple_cycles_with_short_breaks_only() {
        let pomodoro = Pomodoro::new(1, 1, 2, 3).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Cycle 1: Work 1 -> ShortBreak 1 -> Work 2
        timer.tick(); // Work: 0
        timer.tick(); // ShortBreak: cycles=1
        timer.tick(); // ShortBreak: 0
        timer.tick(); // Work: cycles=1

        // Cycle 2: Work 2 -> ShortBreak 2 -> Work 3
        timer.tick(); // Work: 0
        timer.tick(); // ShortBreak: cycles=2
        assert_eq!(timer.state(), TimerState::ShortBreak);
        timer.tick(); // ShortBreak: 0
        timer.tick(); // Work: cycles=2

        // Cycle 3: Work 3 -> LongBreak
        timer.tick(); // Work: 0
        timer.tick(); // LongBreak: cycles=3 (3 % 3 == 0)
        assert_eq!(timer.state(), TimerState::LongBreak);
    }

    // ============================================================================
    // EDGE CASE COMBINATION TESTS
    // ============================================================================

    #[test]
    fn test_pause_skip_resume_sequence() {
        let pomodoro = Pomodoro::new(5, 3, 7, 4).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        timer.pause();
        timer.skip_phase();
        timer.resume();

        assert_eq!(timer.state(), TimerState::ShortBreak);
        assert_eq!(timer.remaining(), 3);
        assert!(!timer.is_paused());
    }

    #[test]
    fn test_skip_during_pause_then_resume() {
        let pomodoro = Pomodoro::new(2, 2, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Pause and skip to short break
        timer.pause();
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::ShortBreak);

        // Resume and tick
        timer.resume();
        timer.tick();
        assert_eq!(timer.remaining(), 1);
    }

    #[test]
    fn test_alternating_pause_skip_operations() {
        let pomodoro = Pomodoro::new(1, 1, 1, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Pause, skip, resume, tick
        timer.pause();
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::ShortBreak);

        timer.resume();
        timer.tick();
        assert_eq!(timer.remaining(), 0);

        // Pause, skip
        timer.pause();
        timer.skip_phase();
        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_rapid_state_changes_via_skip() {
        let pomodoro = Pomodoro::new(1, 1, 1, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Rapidly skip through all states
        assert_eq!(timer.state(), TimerState::Work);
        timer.skip_phase();

        assert_eq!(timer.state(), TimerState::ShortBreak);
        timer.skip_phase();

        assert_eq!(timer.state(), TimerState::Work);
        timer.skip_phase();

        assert_eq!(timer.state(), TimerState::LongBreak);
        timer.skip_phase();

        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_pause_resume_does_not_affect_state() {
        let pomodoro = Pomodoro::new(3, 2, 2, 1).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        assert_eq!(timer.state(), TimerState::Work);
        timer.pause();
        assert_eq!(timer.state(), TimerState::Work);
        timer.resume();
        assert_eq!(timer.state(), TimerState::Work);
    }

    #[test]
    fn test_countdown_to_zero_does_not_auto_transition() {
        let pomodoro = Pomodoro::new(2, 2, 2, 2).unwrap();
        let mut timer = PomodoroTimer::new(pomodoro);

        // Count down to 0
        timer.tick();
        timer.tick();

        // At 0, still in same state
        assert_eq!(timer.remaining(), 0);
        assert_eq!(timer.state(), TimerState::Work);

        // Next tick triggers transition
        timer.tick();
        assert_eq!(timer.state(), TimerState::ShortBreak);
    }
}
