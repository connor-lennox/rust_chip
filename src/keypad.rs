pub struct Keypad {
    pub state: [bool; 16],
    pub last_released: usize,
    pub just_released: bool,
}


impl Keypad {
    pub fn new() -> Keypad {
        Keypad { state: [false; 16], last_released: 0, just_released: false }
    }

    pub fn set_state(&mut self, idx: Vec<usize>) {
        let mut new_state = [false; 16];
        for id in idx {
            new_state[id] = true;
        }

        // Calculate which key (if any) was released this cycle
        self.just_released = false;
        for old in 0..15 {
            if self.state[old] && !new_state[old] {
                self.last_released = old;
                self.just_released = true;
            }
        }

        self.state = new_state;
    }
}