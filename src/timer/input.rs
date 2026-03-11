use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Represents user input commands
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserCommand {
    /// Toggle pause/resume
    TogglePause,
    /// Skip to the next phase
    Skip,
    /// Quit the application
    Quit,
    /// No command
    None,
}

/// Handles keyboard input in a non-blocking way
pub struct InputHandler;

impl InputHandler {
    /// Check for user input with a timeout
    /// Returns UserCommand if a key was pressed, None otherwise
    pub fn poll_input(timeout: Duration) -> std::io::Result<UserCommand> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => Ok(Self::handle_key_event(key_event)),
                Event::Resize(_, _) => Ok(UserCommand::None),
                _ => Ok(UserCommand::None),
            }
        } else {
            Ok(UserCommand::None)
        }
    }

    /// Handle a single key event
    fn handle_key_event(key_event: KeyEvent) -> UserCommand {
        match key_event.code {
            // Toggle pause/resume with 'p'
            KeyCode::Char('p') | KeyCode::Char('P') => UserCommand::TogglePause,
            // Skip with 's'
            KeyCode::Char('s') | KeyCode::Char('S') => UserCommand::Skip,
            // Quit with 'q' or 'Q'
            KeyCode::Char('q') | KeyCode::Char('Q') => UserCommand::Quit,
            // Quit with Ctrl+C
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                UserCommand::Quit
            }
            _ => UserCommand::None,
        }
    }
}
