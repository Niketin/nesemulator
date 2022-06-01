
const MASK_STROBE: u8 = 1; 

pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

pub struct Controller {
    button_states: u8,
    shift_register: u8,
    strobe: bool,
}


impl Controller {
    pub fn new() -> Controller {
        Controller {
            button_states: 0,
            shift_register: 0x00,
            strobe: true,
        }
    }

    fn load_shift_register(&mut self) {
        self.shift_register = self.button_states;
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            self.load_shift_register();
        }
        let result = self.shift_register & 1;
        self.shift_register >>= 1;

        // This sets the MSB to 1 to ensure that each read after the first 8 outputs 1.
        self.shift_register |= 0b1000_0000;
        
        result
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = (value & MASK_STROBE) == 1;
        if self.strobe {
            self.load_shift_register()
        }
    }

    pub fn set_button_state(&mut self, button: Button, value: bool) {
        let index: u8 = match button {
            Button::A => 0,
            Button::B => 1,
            Button::Select => 2,
            Button::Start => 3,
            Button::Up => 4,
            Button::Down => 5,
            Button::Left => 6,
            Button::Right => 7,
        };
        self.button_states &= !(1 << index); // Reset bit at index
        self.button_states |= (value as u8) << index; // Set bit at index
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result::Result;

    #[test]
    fn test_controller_default_values() -> Result<(), std::io::Error> {
        let mut controller = Controller::new();

        for _ in 0..=1000 {
            assert!(controller.read() == 0);
        }

        Ok(())
    }

    #[test]
    fn test_controller_strobe_off() -> Result<(), std::io::Error> {
        let mut controller = Controller::new();
        controller.write(0);

        for i in 0..=7 {
            assert!(controller.read() == 0, "i = {}, controller.read() returned 1", i);
        }

        for i in 8..=1000 {
            assert!(controller.read() == 1, "i = {}, controller.read() returned 0", i);
        }

        Ok(())
    }

    #[test]
    fn test_controller_with_changing_state() -> Result<(), std::io::Error> {
        let mut controller = Controller::new();
        
        for i in 0..=7 {
            assert!(controller.read() == 0, "i = {}, controller.read() returned 1", i);
        }

        controller.set_button_state(Button::Left, true);
        controller.set_button_state(Button::Up, true);
        controller.set_button_state(Button::Select, true);
        
        controller.write(1);
        controller.write(0);

        // Set some states after 1 0 sequence. Should not affect the result.
        controller.set_button_state(Button::Select, false);
        controller.set_button_state(Button::A, true);

        assert!(controller.read() == 0); // A
        assert!(controller.read() == 0); // B
        assert!(controller.read() == 1); // Select
        assert!(controller.read() == 0); // Start
        assert!(controller.read() == 1); // Up
        assert!(controller.read() == 0); // Down
        assert!(controller.read() == 1); // Left
        assert!(controller.read() == 0); // Right

        for i in 8..=1000 {
            assert!(controller.read() == 1, "i = {}, controller.read() returned 0", i);
        }

        Ok(())
    }
}