/// Current keyboard/input state, updated by JS event listeners.
#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub fire_pressed: bool,
    pub pause_pressed: bool,
    pub start_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_keydown(&mut self, key: &str) {
        match key {
            "ArrowLeft" | "a" | "A" => self.left_pressed = true,
            "ArrowRight" | "d" | "D" => self.right_pressed = true,
            " " | "ArrowUp" | "w" | "W" => self.fire_pressed = true,
            "Escape" | "p" | "P" => self.pause_pressed = true,
            "Enter" => self.start_pressed = true,
            _ => {}
        }
    }

    pub fn handle_keyup(&mut self, key: &str) {
        match key {
            "ArrowLeft" | "a" | "A" => self.left_pressed = false,
            "ArrowRight" | "d" | "D" => self.right_pressed = false,
            " " | "ArrowUp" | "w" | "W" => self.fire_pressed = false,
            "Escape" | "p" | "P" => self.pause_pressed = false,
            "Enter" => self.start_pressed = false,
            _ => {}
        }
    }

    /// Consume the pause press (one-shot).
    pub fn consume_pause(&mut self) -> bool {
        let v = self.pause_pressed;
        self.pause_pressed = false;
        v
    }

    /// Consume the start press (one-shot).
    pub fn consume_start(&mut self) -> bool {
        let v = self.start_pressed;
        self.start_pressed = false;
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keydown_left() {
        let mut input = InputState::new();
        input.handle_keydown("ArrowLeft");
        assert!(input.left_pressed);
        input.handle_keyup("ArrowLeft");
        assert!(!input.left_pressed);
    }

    #[test]
    fn test_keydown_wasd() {
        let mut input = InputState::new();
        input.handle_keydown("a");
        assert!(input.left_pressed);
        input.handle_keydown("d");
        assert!(input.right_pressed);
        input.handle_keydown("w");
        assert!(input.fire_pressed);
    }

    #[test]
    fn test_consume_pause() {
        let mut input = InputState::new();
        input.handle_keydown("Escape");
        assert!(input.consume_pause());
        assert!(!input.consume_pause()); // consumed, no longer active
    }
}
