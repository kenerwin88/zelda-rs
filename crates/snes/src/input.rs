//! Controller input. Port of `zelda3/snes/input.c`.
//!
//! Mimics the SNES serial-shift protocol on $4016/$4017: latching the
//! state, then shifting it out one bit at a time.

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct InputState {
    pub kind: u8,
    pub latch_line: bool,
    pub current_state: u16,
    pub latched_state: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            // Matches `input_init` which sets type = 1 (joypad).
            kind: 1,
            latch_line: false,
            current_state: 0,
            latched_state: 0,
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.latch_line = false;
        self.latched_state = 0;
    }

    /// `input_cycle` — copies `current_state` into the shift register
    /// while the latch line is held high.
    pub fn cycle(&mut self) {
        if self.latch_line {
            self.latched_state = self.current_state;
        }
    }

    /// `input_read` — shifts one bit out, MSB-filled with 1 (so the
    /// idle state on the line stays high once the 16-bit packet is
    /// drained).
    pub fn read(&mut self) -> u8 {
        let bit = (self.latched_state & 1) as u8;
        self.latched_state = (self.latched_state >> 1) | 0x8000;
        bit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latches_on_cycle_only() {
        let mut input = InputState::new();
        input.current_state = 0x1234;
        // No cycle → no latch
        assert_eq!(input.read(), 0);
        // Holding latch line + cycle copies state.
        input.latch_line = true;
        input.cycle();
        // Bit 0 of 0x1234 is 0 → first read returns 0
        assert_eq!(input.read(), 0);
        // Bit 1 → 0
        assert_eq!(input.read(), 0);
        // Bit 2 → 1
        assert_eq!(input.read(), 1);
    }

    #[test]
    fn shift_fills_with_ones() {
        let mut input = InputState::new();
        input.current_state = 0;
        input.latch_line = true;
        input.cycle();
        for _ in 0..16 {
            let _ = input.read();
        }
        // After 16 shifts, register should be all ones.
        assert_eq!(input.latched_state, 0xffff);
        assert_eq!(input.read(), 1);
    }
}
