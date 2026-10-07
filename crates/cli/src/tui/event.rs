use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Polls for a terminal event with a short timeout.
/// Returns Some(KeyEvent) if a key was pressed, None if no event or non-key event.
pub fn poll_key_event(timeout_ms: u64) -> io::Result<Option<KeyEvent>> {
    if event::poll(std::time::Duration::from_millis(timeout_ms))? {
        if let Event::Key(key) = event::read()? {
            // Ignore key release events on platforms that emit them
            if key.kind == crossterm::event::KeyEventKind::Press {
                return Ok(Some(key));
            }
        }
    }
    Ok(None)
}

/// Helper to check for Ctrl+C
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}
