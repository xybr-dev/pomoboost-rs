use crate::timer::pomodoro::Pomodoro;
use crate::timer::engine::PomodoroTimer;

#[tokio::main]
async fn main() {
    // Create Pomodoro config with validation
    // Using short durations for testing (normally 25*60, 5*60, 15*60)
    let pomodoro = Pomodoro::new(5, 3, 7, 4).expect("Failed to create Pomodoro timer");

    let mut timer = PomodoroTimer::new(pomodoro);
    timer.run().await;
}

mod timer {
    pub mod pomodoro;
    pub mod engine;
}
