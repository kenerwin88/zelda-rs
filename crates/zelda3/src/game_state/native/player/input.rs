//! Filtered and historical player input.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct PlayerInputState {
    pub(super) button_mask_b_y: u8,
    pub(super) filtered_joypad_h: u8,
    pub(super) filtered_joypad_l: u8,
    pub(super) joypad1h_last: u8,
    pub(super) joypad1l_last: u8,
    pub(super) joypad1h_last2: u8,
    pub(super) joypad1l_last2: u8,
    pub(super) button_b_frames: u8,
}

impl PlayerInputState {
    pub(crate) fn button_mask_b_y(&self) -> u8 {
        self.button_mask_b_y
    }

    pub(crate) fn filtered_joypad_h(&self) -> u8 {
        self.filtered_joypad_h
    }

    pub(crate) fn filtered_joypad_l(&self) -> u8 {
        self.filtered_joypad_l
    }

    pub(crate) fn joypad1h_last(&self) -> u8 {
        self.joypad1h_last
    }

    pub(crate) fn joypad1l_last(&self) -> u8 {
        self.joypad1l_last
    }

    pub(crate) fn joypad1h_last2(&self) -> u8 {
        self.joypad1h_last2
    }

    pub(crate) fn joypad1l_last2(&self) -> u8 {
        self.joypad1l_last2
    }

    pub(crate) fn button_b_frames(&self) -> u8 {
        self.button_b_frames
    }

    pub(crate) fn button_b_frames_index(&self) -> usize {
        usize::from(self.button_b_frames())
    }

    pub(super) fn set_button_mask_b_y(&mut self, value: u8) {
        self.button_mask_b_y = value;
    }

    pub(super) fn add_button_mask_b_y_bits(&mut self, bits: u8) {
        self.button_mask_b_y |= bits;
    }

    pub(super) fn clear_button_mask_b_y_bits(&mut self, mask: u8) {
        self.button_mask_b_y &= !mask;
    }

    pub(super) fn set_filtered_joypad_h(&mut self, value: u8) {
        self.filtered_joypad_h = value;
    }

    pub(super) fn set_filtered_joypad_l(&mut self, value: u8) {
        self.filtered_joypad_l = value;
    }

    pub(super) fn clear_filtered_joypad_l_bits(&mut self, bits: u8) {
        self.filtered_joypad_l &= !bits;
    }

    pub(super) fn set_joypad1h_last(&mut self, value: u8) {
        self.joypad1h_last = value;
    }

    pub(super) fn set_joypad1l_last(&mut self, value: u8) {
        self.joypad1l_last = value;
    }

    pub(super) fn set_joypad1h_last2(&mut self, value: u8) {
        self.joypad1h_last2 = value;
    }

    pub(super) fn set_joypad1l_last2(&mut self, value: u8) {
        self.joypad1l_last2 = value;
    }

    pub(super) fn clear_button_b_frames(&mut self) {
        self.button_b_frames = 0;
    }

    pub(super) fn set_button_b_frames(&mut self, value: u8) {
        self.button_b_frames = value;
    }

    pub(super) fn increment_button_b_frames(&mut self) -> u8 {
        let value = self.button_b_frames().wrapping_add(1);
        self.set_button_b_frames(value);
        value
    }
}
