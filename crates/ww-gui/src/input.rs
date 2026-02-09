//! Input abstraction for keyboard and mouse events.

use macroquad::prelude::*;

/// Collect all characters typed this frame.
pub fn typed_chars() -> Vec<char> {
    let mut chars = Vec::new();
    while let Some(ch) = get_char_pressed() {
        // Filter control characters but keep printable ones
        if (' '..='~').contains(&ch) {
            chars.push(ch);
        }
    }
    chars
}

/// Check if the backspace key was pressed this frame.
pub fn backspace_pressed() -> bool {
    is_key_pressed(KeyCode::Backspace)
}

/// Check if the Enter key was pressed this frame.
pub fn enter_pressed() -> bool {
    is_key_pressed(KeyCode::Enter)
}

/// Check if the Escape key was pressed this frame.
pub fn escape_pressed() -> bool {
    is_key_pressed(KeyCode::Escape)
}

/// Check if a navigation key was pressed (arrow up).
pub fn up_pressed() -> bool {
    is_key_pressed(KeyCode::Up)
}

/// Check if a navigation key was pressed (arrow down).
pub fn down_pressed() -> bool {
    is_key_pressed(KeyCode::Down)
}

/// Check if Tab was pressed.
pub fn tab_pressed() -> bool {
    is_key_pressed(KeyCode::Tab)
}

/// Get the mouse scroll wheel Y delta this frame.
///
/// Positive = scroll up, negative = scroll down.
pub fn scroll_y() -> f32 {
    mouse_wheel().1
}

/// Tracks a held key to fire repeating events after an initial delay.
///
/// Use one instance per repeatable key (e.g., Up, Down, PageUp, PageDown).
pub struct KeyRepeat {
    /// Time remaining before the next repeat fires.
    timer: f32,
    /// Whether we're past the initial delay.
    repeating: bool,
}

/// Initial delay before key repeat begins (seconds).
const REPEAT_DELAY: f32 = 0.35;
/// Interval between repeats once started (seconds).
const REPEAT_RATE: f32 = 0.06;

impl Default for KeyRepeat {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyRepeat {
    /// Create a new key repeat tracker.
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            repeating: false,
        }
    }

    /// Returns true on initial press and on repeat while the key is held.
    pub fn check(&mut self, key: KeyCode) -> bool {
        let dt = get_frame_time();

        if is_key_pressed(key) {
            self.timer = REPEAT_DELAY;
            self.repeating = false;
            return true;
        }

        if is_key_down(key) {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.repeating = true;
                self.timer = REPEAT_RATE;
                return true;
            }
        } else {
            self.timer = 0.0;
            self.repeating = false;
        }

        false
    }
}
