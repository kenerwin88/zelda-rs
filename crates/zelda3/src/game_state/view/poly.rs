use super::*;

pub(crate) struct IntroActorView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> IntroActorView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        byte(self.ram, INTRO_X_LO + self.slot) as u16
            | ((byte(self.ram, INTRO_X_HI + self.slot) as u16) << 8)
    }

    pub(crate) fn y(&self) -> u16 {
        byte(self.ram, INTRO_Y_LO + self.slot) as u16
            | ((byte(self.ram, INTRO_Y_HI + self.slot) as u16) << 8)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, INTRO_X_LO + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, INTRO_Y_LO + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, INTRO_X_VEL + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, INTRO_Y_VEL + self.slot)
    }

    pub(crate) fn init_phase(&self) -> u8 {
        byte(self.ram, INTRO_SPRITE_IS_INITED + self.slot)
    }

    pub(crate) fn subtype(&self) -> u8 {
        byte(self.ram, INTRO_SPRITE_SUBTYPE + self.slot)
    }

    pub(crate) fn state(&self) -> u8 {
        byte(self.ram, INTRO_SPRITE_STATE + self.slot)
    }
}

pub(crate) struct IntroActorViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> IntroActorViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_x(&mut self, value: i16) {
        self.ram[INTRO_X_LO + self.slot] = value as u8;
        self.ram[INTRO_X_HI + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_y(&mut self, value: i16) {
        self.ram[INTRO_Y_LO + self.slot] = value as u8;
        self.ram[INTRO_Y_HI + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[INTRO_X_LO + self.slot] = value;
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[INTRO_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[INTRO_X_VEL + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[INTRO_Y_VEL + self.slot] = value;
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) {
        self.ram[INTRO_X_VEL + self.slot] = self.ram[INTRO_X_VEL + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.ram[INTRO_Y_VEL + self.slot] = self.ram[INTRO_Y_VEL + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_init_phase(&mut self, value: u8) {
        self.ram[INTRO_SPRITE_IS_INITED + self.slot] = value;
    }

    pub(crate) fn increment_init_phase(&mut self) {
        self.ram[INTRO_SPRITE_IS_INITED + self.slot] =
            self.ram[INTRO_SPRITE_IS_INITED + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_subtype(&mut self, value: u8) {
        self.ram[INTRO_SPRITE_SUBTYPE + self.slot] = value;
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.ram[INTRO_SPRITE_STATE + self.slot] = value;
    }

    pub(crate) fn increment_state(&mut self) {
        self.ram[INTRO_SPRITE_STATE + self.slot] =
            self.ram[INTRO_SPRITE_STATE + self.slot].wrapping_add(1);
    }

    pub(crate) fn move_x(&mut self) {
        move_axis24(
            self.ram,
            INTRO_X_SUBPIXEL + self.slot,
            INTRO_X_LO + self.slot,
            INTRO_X_HI + self.slot,
            INTRO_X_VEL + self.slot,
        );
    }

    pub(crate) fn move_y(&mut self) {
        move_axis24(
            self.ram,
            INTRO_Y_SUBPIXEL + self.slot,
            INTRO_Y_LO + self.slot,
            INTRO_Y_HI + self.slot,
            INTRO_Y_VEL + self.slot,
        );
    }
}
