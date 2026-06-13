use super::*;

pub(crate) struct RamSpriteSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> RamSpriteSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn slot(&self) -> u8 {
        self.slot as u8
    }

    pub(crate) fn sprite_type(&self) -> u8 {
        byte(self.ram, SPRITE_TYPE + self.slot)
    }

    pub(crate) fn state(&self) -> u8 {
        byte(self.ram, SPRITE_STATE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.sprite_type() != 0 || self.state() != 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, SPRITE_X_LO + self.slot, SPRITE_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, SPRITE_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, SPRITE_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, SPRITE_Y_LO + self.slot, SPRITE_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, SPRITE_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, SPRITE_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_Y_VELOCITY + self.slot)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_Z_VELOCITY + self.slot)
    }

    pub(crate) fn x_recoil(&self) -> u8 {
        byte(self.ram, SPRITE_X_RECOIL + self.slot)
    }

    pub(crate) fn y_recoil(&self) -> u8 {
        byte(self.ram, SPRITE_Y_RECOIL + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn z(&self) -> u8 {
        byte(self.ram, SPRITE_Z + self.slot)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_Z_SUBPIXEL + self.slot)
    }

    pub(crate) fn ai_state(&self) -> u8 {
        byte(self.ram, SPRITE_AI_STATE + self.slot)
    }

    pub(crate) fn a(&self) -> u8 {
        byte(self.ram, SPRITE_A + self.slot)
    }

    pub(crate) fn c(&self) -> u8 {
        byte(self.ram, SPRITE_C + self.slot)
    }

    pub(crate) fn b(&self) -> u8 {
        byte(self.ram, SPRITE_B + self.slot)
    }

    pub(crate) fn e(&self) -> u8 {
        byte(self.ram, SPRITE_E + self.slot)
    }

    pub(crate) fn f(&self) -> u8 {
        byte(self.ram, SPRITE_F + self.slot)
    }

    pub(crate) fn g(&self) -> u8 {
        byte(self.ram, SPRITE_G + self.slot)
    }

    pub(crate) fn graphics(&self) -> u8 {
        byte(self.ram, SPRITE_GRAPHICS + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, SPRITE_D + self.slot)
    }

    pub(crate) fn subtype(&self) -> u8 {
        byte(self.ram, SPRITE_SUBTYPE + self.slot)
    }

    pub(crate) fn delay_main(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_MAIN + self.slot)
    }

    pub(crate) fn delay_aux1(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX1 + self.slot)
    }

    pub(crate) fn delay_aux4(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX4 + self.slot)
    }

    pub(crate) fn delay_aux2(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX2 + self.slot)
    }

    pub(crate) fn flags2(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS2 + self.slot)
    }

    pub(crate) fn flags(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS + self.slot)
    }

    pub(crate) fn flags3(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS3 + self.slot)
    }

    pub(crate) fn wall_collision(&self) -> u8 {
        byte(self.ram, SPRITE_WALL_COLLISION + self.slot)
    }

    pub(crate) fn anim_clock(&self) -> u8 {
        byte(self.ram, SPRITE_ANIM_CLOCK + self.slot)
    }

    pub(crate) fn delay_aux3(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX3 + self.slot)
    }

    pub(crate) fn flags4(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS4 + self.slot)
    }

    pub(crate) fn flags5(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS5 + self.slot)
    }

    pub(crate) fn health(&self) -> u8 {
        byte(self.ram, SPRITE_HEALTH + self.slot)
    }

    pub(crate) fn hit_timer(&self) -> u8 {
        byte(self.ram, SPRITE_HIT_TIMER + self.slot)
    }

    pub(crate) fn pause(&self) -> u8 {
        byte(self.ram, SPRITE_PAUSE + self.slot)
    }

    pub(crate) fn stunned(&self) -> u8 {
        byte(self.ram, SPRITE_STUNNED + self.slot)
    }

    pub(crate) fn ignore_projectile(&self) -> u8 {
        byte(self.ram, SPRITE_IGNORE_PROJECTILE + self.slot)
    }

    pub(crate) fn draw_work_byte_2(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_2 + self.slot)
    }

    pub(crate) fn n(&self) -> u8 {
        byte(self.ram, SPRITE_N + self.slot)
    }

    pub(crate) fn n_word(&self) -> u16 {
        read_le_u16(self.ram, SPRITE_N + self.slot * 2)
    }

    pub(crate) fn deflection_bits(&self) -> u8 {
        byte(self.ram, SPRITE_DEFL_BITS + self.slot)
    }

    pub(crate) fn bump_damage(&self) -> u8 {
        byte(self.ram, SPRITE_BUMP_DAMAGE + self.slot)
    }

    pub(crate) fn incoming_damage(&self) -> u8 {
        byte(self.ram, SPRITE_INCOMING_DAMAGE + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, SPRITE_FLOOR + self.slot)
    }

    pub(crate) fn room(&self) -> u8 {
        byte(self.ram, SPRITE_ROOM + self.slot)
    }

    pub(crate) fn die_action(&self) -> u8 {
        byte(self.ram, SPRITE_DIE_ACTION + self.slot)
    }

    pub(crate) fn draw_i(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_I + self.slot)
    }

    pub(crate) fn draw_work_byte_3(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_3 + self.slot)
    }

    pub(crate) fn draw_work_byte_4(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_4 + self.slot)
    }

    pub(crate) fn draw_work_byte_5(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_5 + self.slot)
    }

    pub(crate) fn draw_work_byte_1(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_1 + self.slot)
    }

    pub(crate) fn head_direction(&self) -> u8 {
        byte(self.ram, SPRITE_HEAD_DIR + self.slot)
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        byte(self.ram, SPRITE_OAM_FLAGS + self.slot)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        byte(self.ram, SPRITE_OBJ_PRIO + self.slot)
    }

    pub(crate) fn subtype2(&self) -> u8 {
        byte(self.ram, SPRITE_SUBTYPE2 + self.slot)
    }
}

pub(crate) struct RamSpriteSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> RamSpriteSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_sprite_type(&mut self, value: u8) {
        self.ram[SPRITE_TYPE + self.slot] = value;
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.ram[SPRITE_STATE + self.slot] = value;
    }

    pub(crate) fn increment_state(&mut self) {
        self.ram[SPRITE_STATE + self.slot] = self.ram[SPRITE_STATE + self.slot].wrapping_add(1);
    }

    pub(crate) fn clear(&mut self) {
        self.set_state(0);
    }

    pub(crate) fn clear_prep_runtime_state(&mut self) {
        for base in [
            SPRITE_PAUSE,
            SPRITE_E,
            SPRITE_X_VELOCITY,
            SPRITE_Y_VELOCITY,
            SPRITE_Z_VELOCITY,
            SPRITE_X_SUBPIXEL,
            SPRITE_Y_SUBPIXEL,
            SPRITE_Z_SUBPIXEL,
            SPRITE_AI_STATE,
            SPRITE_GRAPHICS,
            SPRITE_D,
            SPRITE_DELAY_MAIN,
            SPRITE_DELAY_AUX1,
            SPRITE_DELAY_AUX2,
            SPRITE_DELAY_AUX4,
            SPRITE_HEAD_DIR,
            SPRITE_ANIM_CLOCK,
            SPRITE_G,
            SPRITE_HIT_TIMER,
            SPRITE_WALL_COLLISION,
            SPRITE_Z,
            SPRITE_HEALTH,
            SPRITE_F,
            SPRITE_X_RECOIL,
            SPRITE_Y_RECOIL,
            SPRITE_A,
            SPRITE_B,
            SPRITE_C,
            SPRITE_DRAW_WORK_BYTE_2,
            SPRITE_SUBTYPE2,
            SPRITE_IGNORE_PROJECTILE,
            SPRITE_OBJ_PRIO,
            SPRITE_OAM_FLAGS,
            SPRITE_STUNNED,
            SPRITE_INCOMING_DAMAGE,
            SPRITE_DRAW_WORK_BYTE_3,
            SPRITE_DRAW_WORK_BYTE_4,
            SPRITE_DRAW_WORK_BYTE_5,
            SPRITE_DRAW_WORK_BYTE_1,
            SPRITE_DRAW_I,
        ] {
            self.ram[base + self.slot] = 0;
        }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            SPRITE_X_LO + self.slot,
            SPRITE_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[SPRITE_X_HI + self.slot] = value;
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = self.ram[SPRITE_X_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = self.ram[SPRITE_X_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            SPRITE_Y_LO + self.slot,
            SPRITE_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[SPRITE_Y_HI + self.slot] = value;
    }

    pub(crate) fn add_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = self.ram[SPRITE_Y_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = self.ram[SPRITE_Y_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] = value;
    }

    pub(crate) fn decrement_z_velocity(&mut self) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_delay_aux4(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX4 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux1(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX1 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux2(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX2 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux3(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX3 + self.slot] = value;
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.ram[SPRITE_X_VELOCITY + self.slot]
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.ram[SPRITE_Y_VELOCITY + self.slot]
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn xor_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] ^= value;
    }

    pub(crate) fn xor_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] ^= value;
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn negate_z_velocity(&mut self) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn and_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] &= value;
    }

    pub(crate) fn and_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] &= value;
    }

    pub(crate) fn shift_x_velocity_left(&mut self, amount: u32) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_shl(amount);
    }

    pub(crate) fn shift_y_velocity_left(&mut self, amount: u32) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_shl(amount);
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn subtract_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn halve_x_velocity(&mut self) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            ((self.ram[SPRITE_X_VELOCITY + self.slot] as i8) >> 1) as u8;
    }

    pub(crate) fn halve_y_velocity(&mut self) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            ((self.ram[SPRITE_Y_VELOCITY + self.slot] as i8) >> 1) as u8;
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.ram[SPRITE_Z + self.slot] = value;
    }

    pub(crate) fn add_z(&mut self, value: u8) {
        self.ram[SPRITE_Z + self.slot] = self.ram[SPRITE_Z + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn negate_z_subpixel(&mut self) {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_neg();
    }

    pub(crate) fn increment_z_subpixel(&mut self) -> u8 {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_add(1);
        self.ram[SPRITE_Z_SUBPIXEL + self.slot]
    }

    pub(crate) fn decrement_z_subpixel(&mut self) -> u8 {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_sub(1);
        self.ram[SPRITE_Z_SUBPIXEL + self.slot]
    }

    pub(crate) fn set_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] = value;
    }

    pub(crate) fn increment_ai_state(&mut self) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(1);
    }

    pub(crate) fn decrement_ai_state(&mut self) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(1);
    }

    pub(crate) fn add_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = value;
    }

    pub(crate) fn increment_a(&mut self) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(1);
    }

    pub(crate) fn xor_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] ^= value;
    }

    pub(crate) fn add_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_a(&mut self) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(1);
    }

    pub(crate) fn subtract_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = value;
    }

    pub(crate) fn increment_b(&mut self) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_sub(value);
    }

    pub(crate) fn xor_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] ^= value;
    }

    pub(crate) fn set_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] = value;
    }

    pub(crate) fn increment_c(&mut self) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(value);
    }

    pub(crate) fn xor_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] ^= value;
    }

    pub(crate) fn decrement_c(&mut self) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_e(&mut self, value: u8) {
        self.ram[SPRITE_E + self.slot] = value;
    }

    pub(crate) fn increment_e(&mut self) -> u8 {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(1);
        self.ram[SPRITE_E + self.slot]
    }

    pub(crate) fn add_e(&mut self, value: u8) {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_e(&mut self) -> u8 {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_sub(1);
        self.ram[SPRITE_E + self.slot]
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] = value;
    }

    pub(crate) fn add_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(value);
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] &= value;
    }

    pub(crate) fn increment_direction(&mut self) {
        self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(1);
    }

    pub(crate) fn xor_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] ^= value;
    }

    pub(crate) fn set_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] = value;
    }

    pub(crate) fn increment_delay_main(&mut self) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_sub(value);
    }

    pub(crate) fn halve_delay_main(&mut self) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] >>= 1;
    }

    pub(crate) fn set_head_direction(&mut self, value: u8) {
        self.ram[SPRITE_HEAD_DIR + self.slot] = value;
    }

    pub(crate) fn increment_head_direction(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_head_direction(&mut self, value: u8) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_head_direction(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_sub(1);
    }

    pub(crate) fn increment_head_direction_mod16(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1) & 15;
    }

    pub(crate) fn set_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] = value;
    }

    pub(crate) fn increment_graphics(&mut self) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_graphics(&mut self) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_sub(1);
    }

    pub(crate) fn xor_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] ^= value;
    }

    pub(crate) fn set_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] = value;
    }

    pub(crate) fn add_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] =
            self.ram[SPRITE_FLAGS2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] =
            self.ram[SPRITE_FLAGS2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn and_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] &= value;
    }

    pub(crate) fn or_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] |= value;
    }

    pub(crate) fn masked_or_flags2(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] = (self.ram[SPRITE_FLAGS2 + self.slot] & mask) | value;
    }

    pub(crate) fn set_flags(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS + self.slot] = value;
    }

    pub(crate) fn masked_or_flags(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS + self.slot] = (self.ram[SPRITE_FLAGS + self.slot] & mask) | value;
    }

    pub(crate) fn set_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] = value;
    }

    pub(crate) fn and_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] &= value;
    }

    pub(crate) fn clear_flags3_bits(&mut self, mask: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] &= !mask;
    }

    pub(crate) fn set_flags4(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS4 + self.slot] = value;
    }

    pub(crate) fn increment_flags4(&mut self) {
        self.ram[SPRITE_FLAGS4 + self.slot] = self.ram[SPRITE_FLAGS4 + self.slot].wrapping_add(1);
    }

    pub(crate) fn or_flags4(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS4 + self.slot] |= value;
    }

    pub(crate) fn set_flags5(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS5 + self.slot] = value;
    }

    pub(crate) fn and_flags5(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS5 + self.slot] &= value;
    }

    pub(crate) fn or_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] |= value;
    }

    pub(crate) fn xor_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] ^= value;
    }

    pub(crate) fn masked_or_flags3(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] = (self.ram[SPRITE_FLAGS3 + self.slot] & mask) | value;
    }

    pub(crate) fn set_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] = value;
    }

    pub(crate) fn increment_subtype2(&mut self) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_subtype2(&mut self) -> u8 {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(1);
        self.ram[SPRITE_SUBTYPE2 + self.slot]
    }

    pub(crate) fn subtract_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] = value;
    }

    pub(crate) fn increment_subtype(&mut self) {
        self.ram[SPRITE_SUBTYPE + self.slot] = self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] =
            self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(value);
    }

    pub(crate) fn and_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] &= value;
    }

    pub(crate) fn decrement_subtype(&mut self) {
        self.ram[SPRITE_SUBTYPE + self.slot] = self.ram[SPRITE_SUBTYPE + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_f(&mut self, value: u8) {
        self.ram[SPRITE_F + self.slot] = value;
    }

    pub(crate) fn subtract_f(&mut self, value: u8) {
        self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(value);
    }

    pub(crate) fn decrement_f(&mut self) {
        self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = value;
    }

    pub(crate) fn increment_g(&mut self) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(value);
    }

    pub(crate) fn decrement_g(&mut self) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[SPRITE_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_room(&mut self, value: u8) {
        self.ram[SPRITE_ROOM + self.slot] = value;
    }

    pub(crate) fn set_x_recoil(&mut self, value: u8) {
        self.ram[SPRITE_X_RECOIL + self.slot] = value;
    }

    pub(crate) fn set_y_recoil(&mut self, value: u8) {
        self.ram[SPRITE_Y_RECOIL + self.slot] = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] = value;
    }

    pub(crate) fn and_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] &= value;
    }

    pub(crate) fn xor_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] ^= value;
    }

    pub(crate) fn or_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] |= value;
    }

    pub(crate) fn masked_or_oam_flags(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] =
            (self.ram[SPRITE_OAM_FLAGS + self.slot] & mask) | value;
    }

    pub(crate) fn set_stunned(&mut self, value: u8) {
        self.ram[SPRITE_STUNNED + self.slot] = value;
    }

    pub(crate) fn decrement_stunned(&mut self) {
        self.ram[SPRITE_STUNNED + self.slot] = self.ram[SPRITE_STUNNED + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_pause(&mut self, value: u8) {
        self.ram[SPRITE_PAUSE + self.slot] = value;
    }

    pub(crate) fn set_health(&mut self, value: u8) {
        self.ram[SPRITE_HEALTH + self.slot] = value;
    }

    pub(crate) fn decrement_health(&mut self) {
        self.ram[SPRITE_HEALTH + self.slot] = self.ram[SPRITE_HEALTH + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_ignore_projectile(&mut self, value: u8) {
        self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_2(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_2 + self.slot] = value;
    }

    pub(crate) fn set_draw_i(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_I + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_3(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] = value;
    }

    pub(crate) fn increment_draw_work_byte_3(&mut self) {
        self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] =
            self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_draw_work_byte_4(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_4 + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_5(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_5 + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_1(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_1 + self.slot] = value;
    }

    pub(crate) fn increment_ignore_projectile(&mut self) {
        self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] =
            self.ram[SPRITE_IGNORE_PROJECTILE + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] = value;
    }

    pub(crate) fn or_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] |= value;
    }

    pub(crate) fn clear_deflection_bits(&mut self, mask: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] &= !mask;
    }

    pub(crate) fn and_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] &= value;
    }

    pub(crate) fn set_bump_damage(&mut self, value: u8) {
        self.ram[SPRITE_BUMP_DAMAGE + self.slot] = value;
    }

    pub(crate) fn and_bump_damage(&mut self, value: u8) {
        self.ram[SPRITE_BUMP_DAMAGE + self.slot] &= value;
    }

    pub(crate) fn set_incoming_damage(&mut self, value: u8) {
        self.ram[SPRITE_INCOMING_DAMAGE + self.slot] = value;
    }

    pub(crate) fn set_n(&mut self, value: u8) {
        self.ram[SPRITE_N + self.slot] = value;
    }

    pub(crate) fn set_n_word(&mut self, value: u16) {
        write_le_u16(self.ram, SPRITE_N + self.slot * 2, value);
    }

    pub(crate) fn set_die_action(&mut self, value: u8) {
        self.ram[SPRITE_DIE_ACTION + self.slot] = value;
    }

    pub(crate) fn increment_die_action(&mut self) {
        self.ram[SPRITE_DIE_ACTION + self.slot] =
            self.ram[SPRITE_DIE_ACTION + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] = value;
    }

    pub(crate) fn clear_object_priority_bits(&mut self, mask: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] &= !mask;
    }

    pub(crate) fn or_object_priority_bits(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
    }

    pub(crate) fn or_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
    }

    pub(crate) fn and_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] &= value;
    }

    pub(crate) fn set_hit_timer(&mut self, value: u8) {
        self.ram[SPRITE_HIT_TIMER + self.slot] = value;
    }

    pub(crate) fn or_hit_timer(&mut self, value: u8) {
        self.ram[SPRITE_HIT_TIMER + self.slot] |= value;
    }

    pub(crate) fn set_anim_clock(&mut self, value: u8) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] = value;
    }

    pub(crate) fn add_anim_clock(&mut self, value: u8) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(value);
    }

    pub(crate) fn increment_anim_clock(&mut self) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(1);
    }

    pub(crate) fn decrement_anim_clock(&mut self) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_wall_collision(&mut self, value: u8) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] = value;
    }

    pub(crate) fn or_wall_collision(&mut self, value: u8) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] |= value;
    }

    pub(crate) fn decrement_wall_collision(&mut self) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] =
            self.ram[SPRITE_WALL_COLLISION + self.slot].wrapping_sub(1);
    }

    pub(crate) fn move_x(&mut self) {
        if self.ram[SPRITE_X_VELOCITY + self.slot] == 0 {
            return;
        }
        move_axis24(
            self.ram,
            SPRITE_X_SUBPIXEL + self.slot,
            SPRITE_X_LO + self.slot,
            SPRITE_X_HI + self.slot,
            SPRITE_X_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_y(&mut self) {
        if self.ram[SPRITE_Y_VELOCITY + self.slot] == 0 {
            return;
        }
        move_axis24(
            self.ram,
            SPRITE_Y_SUBPIXEL + self.slot,
            SPRITE_Y_LO + self.slot,
            SPRITE_Y_HI + self.slot,
            SPRITE_Y_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_z(&mut self) {
        move_axis16(
            self.ram,
            SPRITE_Z_SUBPIXEL + self.slot,
            SPRITE_Z + self.slot,
            SPRITE_Z_VELOCITY + self.slot,
        );
    }
}

pub(crate) struct RamAncillaSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> RamAncillaSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn slot(&self) -> u8 {
        self.slot as u8
    }

    pub(crate) fn ancilla_type(&self) -> u8 {
        byte(self.ram, ANCILLA_TYPE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.ancilla_type() != 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, ANCILLA_X_LO + self.slot, ANCILLA_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, ANCILLA_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, ANCILLA_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, ANCILLA_Y_LO + self.slot, ANCILLA_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_VELOCITY + self.slot)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_Z_VELOCITY + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn z(&self) -> u8 {
        byte(self.ram, ANCILLA_Z + self.slot)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_Z_SUBPIXEL_PLAYER + self.slot)
    }

    pub(crate) fn item_to_link(&self) -> u8 {
        byte(self.ram, ANCILLA_ITEM_TO_LINK + self.slot)
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, ANCILLA_TIMER + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, ANCILLA_FLOOR + self.slot)
    }

    pub(crate) fn floor2(&self) -> u8 {
        byte(self.ram, ANCILLA_FLOOR2 + self.slot)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        byte(self.ram, ANCILLA_OBJPRIO + self.slot)
    }

    pub(crate) fn u(&self) -> u8 {
        byte(self.ram, ANCILLA_U + self.slot)
    }

    pub(crate) fn num_sprites(&self) -> u8 {
        byte(self.ram, ANCILLA_NUMSPR + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, ANCILLA_DIRECTION + self.slot)
    }

    pub(crate) fn tile_attribute(&self) -> u8 {
        byte(self.ram, ANCILLA_TILE_ATTRIBUTE + self.slot)
    }

    pub(crate) fn step(&self) -> u8 {
        byte(self.ram, ANCILLA_STEP + self.slot)
    }

    pub(crate) fn aux_timer(&self) -> u8 {
        byte(self.ram, ANCILLA_AUX_TIMER + self.slot)
    }

    pub(crate) fn work_byte_3(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_3 + self.slot)
    }

    pub(crate) fn work_byte_1(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_1 + self.slot)
    }

    pub(crate) fn s_player(&self) -> u8 {
        byte(self.ram, ANCILLA_S_PLAYER + self.slot)
    }

    pub(crate) fn t_player(&self) -> u8 {
        byte(self.ram, ANCILLA_T_PLAYER + self.slot)
    }

    pub(crate) fn a(&self) -> u8 {
        byte(self.ram, ANCILLA_A + self.slot)
    }

    pub(crate) fn b(&self) -> u8 {
        byte(self.ram, ANCILLA_B + self.slot)
    }

    pub(crate) fn ab_word(&self) -> u16 {
        u16::from(self.a()) | (u16::from(self.b()) << 8)
    }

    /// Word read at `a`; the high byte aliases the next slot's `a` byte,
    /// matching the original WORD(ancilla_A(k)) access.
    pub(crate) fn a_word(&self) -> u16 {
        word(self.ram, ANCILLA_A + self.slot)
    }

    pub(crate) fn l(&self) -> u8 {
        byte(self.ram, ANCILLA_L + self.slot)
    }

    pub(crate) fn h(&self) -> u8 {
        byte(self.ram, ANCILLA_H + self.slot)
    }

    pub(crate) fn k(&self) -> u8 {
        byte(self.ram, ANCILLA_K + self.slot)
    }

    pub(crate) fn g(&self) -> u8 {
        byte(self.ram, ANCILLA_G + self.slot)
    }

    pub(crate) fn r(&self) -> u8 {
        byte(self.ram, ANCILLA_R + self.slot)
    }

    pub(crate) fn work_byte_22(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_22 + self.slot)
    }

    pub(crate) fn work_byte_23(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_23 + self.slot)
    }

    pub(crate) fn work_byte_24(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_24 + self.slot)
    }

    pub(crate) fn work_byte_4(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_4 + self.slot)
    }

    pub(crate) fn work_byte_25(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_25 + self.slot)
    }

    pub(crate) fn work_byte_26(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_26 + self.slot)
    }
}

pub(crate) struct RamAncillaSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> RamAncillaSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_ancilla_type(&mut self, value: u8) {
        self.ram[ANCILLA_TYPE + self.slot] = value;
    }

    pub(crate) fn increment_ancilla_type(&mut self) -> u8 {
        self.ram[ANCILLA_TYPE + self.slot] = self.ram[ANCILLA_TYPE + self.slot].wrapping_add(1);
        self.ram[ANCILLA_TYPE + self.slot]
    }

    pub(crate) fn clear(&mut self) {
        self.set_ancilla_type(0);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            ANCILLA_X_LO + self.slot,
            ANCILLA_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[ANCILLA_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[ANCILLA_X_HI + self.slot] = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            ANCILLA_Y_LO + self.slot,
            ANCILLA_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[ANCILLA_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[ANCILLA_Y_HI + self.slot] = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.ram[ANCILLA_X_VELOCITY + self.slot]
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.ram[ANCILLA_Y_VELOCITY + self.slot]
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) -> u8 {
        let velocity = self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_add(value);
        self.set_x_velocity(velocity);
        velocity
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) -> u8 {
        let velocity = self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_add(value);
        self.set_y_velocity(velocity);
        velocity
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] =
            self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        self.ram[ANCILLA_X_VELOCITY + self.slot] =
            self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] =
            self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Z_VELOCITY + self.slot] = value;
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Z_VELOCITY + self.slot] =
            self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_z_velocity(&mut self) -> u8 {
        let value = self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_sub(1);
        self.set_z_velocity(value);
        value
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.ram[ANCILLA_Z + self.slot] = value;
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + self.slot] = value;
    }

    pub(crate) fn move_x(&mut self) {
        move_axis24(
            self.ram,
            ANCILLA_X_SUBPIXEL + self.slot,
            ANCILLA_X_LO + self.slot,
            ANCILLA_X_HI + self.slot,
            ANCILLA_X_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_y(&mut self) {
        move_axis24(
            self.ram,
            ANCILLA_Y_SUBPIXEL + self.slot,
            ANCILLA_Y_LO + self.slot,
            ANCILLA_Y_HI + self.slot,
            ANCILLA_Y_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_z(&mut self) {
        move_axis16(
            self.ram,
            ANCILLA_Z_SUBPIXEL_PLAYER + self.slot,
            ANCILLA_Z + self.slot,
            ANCILLA_Z_VELOCITY + self.slot,
        );
    }

    pub(crate) fn set_item_to_link(&mut self, value: u8) {
        self.ram[ANCILLA_ITEM_TO_LINK + self.slot] = value;
    }

    pub(crate) fn advance_item_to_link(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(1);
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn add_item_to_link(&mut self, value: u8) {
        self.ram[ANCILLA_ITEM_TO_LINK + self.slot] =
            self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_item_to_link(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_sub(1);
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn toggle_item_to_link_bit0(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot] ^ 1;
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[ANCILLA_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_TIMER + self.slot].wrapping_sub(1);
        self.set_timer(value);
        value
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[ANCILLA_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_floor2(&mut self, value: u8) {
        self.ram[ANCILLA_FLOOR2 + self.slot] = value;
    }

    pub(crate) fn set_oam_index(&mut self, value: u8) {
        self.ram[ANCILLA_OAM_IDX + self.slot] = value;
    }

    pub(crate) fn set_num_sprites(&mut self, value: u8) {
        self.ram[ANCILLA_NUMSPR + self.slot] = value;
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.ram[ANCILLA_OBJPRIO + self.slot] = value;
    }

    pub(crate) fn xor_object_priority(&mut self, value: u8) {
        self.ram[ANCILLA_OBJPRIO + self.slot] ^= value;
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] = value;
    }

    pub(crate) fn or_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] |= value;
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] &= value;
    }

    pub(crate) fn add_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] =
            self.ram[ANCILLA_DIRECTION + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_tile_attribute(&mut self, value: u8) {
        self.ram[ANCILLA_TILE_ATTRIBUTE + self.slot] = value;
    }

    pub(crate) fn set_step(&mut self, value: u8) {
        self.ram[ANCILLA_STEP + self.slot] = value;
    }

    pub(crate) fn advance_step(&mut self) -> u8 {
        let value = self.ram[ANCILLA_STEP + self.slot].wrapping_add(1);
        self.set_step(value);
        value
    }

    pub(crate) fn add_step(&mut self, value: u8) {
        self.ram[ANCILLA_STEP + self.slot] = self.ram[ANCILLA_STEP + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_step(&mut self) -> u8 {
        let value = self.ram[ANCILLA_STEP + self.slot].wrapping_sub(1);
        self.set_step(value);
        value
    }

    pub(crate) fn set_aux_timer(&mut self, value: u8) {
        self.ram[ANCILLA_AUX_TIMER + self.slot] = value;
    }

    pub(crate) fn advance_aux_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(1);
        self.set_aux_timer(value);
        value
    }

    pub(crate) fn add_aux_timer(&mut self, value: u8) {
        self.ram[ANCILLA_AUX_TIMER + self.slot] =
            self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_aux_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_sub(1);
        self.set_aux_timer(value);
        value
    }

    pub(crate) fn set_work_byte_3(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_3 + self.slot] = value;
    }

    pub(crate) fn add_work_byte_3(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_3 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_1(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_1 + self.slot] = value;
    }

    pub(crate) fn subtract_work_byte_1(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_1 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.ram[ANCILLA_A + self.slot] = value;
    }

    pub(crate) fn advance_a(&mut self) -> u8 {
        let value = self.ram[ANCILLA_A + self.slot].wrapping_add(1);
        self.set_a(value);
        value
    }

    /// Word write at `a`; the high byte aliases the next slot's `a` byte,
    /// matching the original WORD(ancilla_A(k)) access.
    pub(crate) fn set_a_word(&mut self, value: u16) {
        write_le_u16(self.ram, ANCILLA_A + self.slot, value);
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.ram[ANCILLA_B + self.slot] = value;
    }

    pub(crate) fn set_l(&mut self, value: u8) {
        self.ram[ANCILLA_L + self.slot] = value;
    }

    pub(crate) fn advance_l(&mut self) -> u8 {
        let value = self.ram[ANCILLA_L + self.slot].wrapping_add(1);
        self.set_l(value);
        value
    }

    pub(crate) fn add_l(&mut self, value: u8) {
        self.ram[ANCILLA_L + self.slot] = self.ram[ANCILLA_L + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_l(&mut self) -> u8 {
        let value = self.ram[ANCILLA_L + self.slot].wrapping_sub(1);
        self.set_l(value);
        value
    }

    pub(crate) fn set_h(&mut self, value: u8) {
        self.ram[ANCILLA_H + self.slot] = value;
    }

    pub(crate) fn set_k(&mut self, value: u8) {
        self.ram[ANCILLA_K + self.slot] = value;
    }

    pub(crate) fn toggle_k_bit0(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot] ^ 1;
        self.set_k(value);
        value
    }

    pub(crate) fn advance_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_add(1);
        self.set_k(value);
        value
    }

    pub(crate) fn add_k(&mut self, value: u8) {
        self.ram[ANCILLA_K + self.slot] = self.ram[ANCILLA_K + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
        self.set_k(value);
        value
    }

    pub(crate) fn tick_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
        self.set_k(value);
        value
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.ram[ANCILLA_G + self.slot] = value;
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.ram[ANCILLA_G + self.slot] = self.ram[ANCILLA_G + self.slot].wrapping_sub(value);
    }

    pub(crate) fn tick_g(&mut self) -> u8 {
        let value = self.ram[ANCILLA_G + self.slot].wrapping_sub(1);
        self.set_g(value);
        value
    }

    pub(crate) fn set_s_player(&mut self, value: u8) {
        self.ram[ANCILLA_S_PLAYER + self.slot] = value;
    }

    pub(crate) fn set_t_player(&mut self, value: u8) {
        self.ram[ANCILLA_T_PLAYER + self.slot] = value;
    }

    pub(crate) fn set_r(&mut self, value: u8) {
        self.ram[ANCILLA_R + self.slot] = value;
    }

    pub(crate) fn advance_r(&mut self) -> u8 {
        let value = self.ram[ANCILLA_R + self.slot].wrapping_add(1);
        self.set_r(value);
        value
    }

    pub(crate) fn add_r(&mut self, value: u8) {
        self.ram[ANCILLA_R + self.slot] = self.ram[ANCILLA_R + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_s_player(&mut self) -> u8 {
        let value = self.ram[ANCILLA_S_PLAYER + self.slot].wrapping_sub(1);
        self.set_s_player(value);
        value
    }

    pub(crate) fn set_u(&mut self, value: u8) {
        self.ram[ANCILLA_U + self.slot] = value;
    }

    pub(crate) fn subtract_u(&mut self, value: u8) {
        self.ram[ANCILLA_U + self.slot] = self.ram[ANCILLA_U + self.slot].wrapping_sub(value);
    }

    pub(crate) fn advance_work_byte_1_mod4(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(1) & 3;
        self.set_work_byte_1(value);
        value
    }

    pub(crate) fn add_work_byte_1_mod4(&mut self, value: u8) -> u8 {
        let next = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(value) & 3;
        self.set_work_byte_1(next);
        next
    }

    pub(crate) fn tick_work_byte_3(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_sub(1);
        self.set_work_byte_3(value);
        value
    }

    pub(crate) fn advance_work_byte_3(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(1);
        self.set_work_byte_3(value);
        value
    }

    pub(crate) fn set_work_byte_4(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_4 + self.slot] = value;
    }

    pub(crate) fn subtract_work_byte_4(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_4 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_work_byte_22(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_22 + self.slot] = value;
    }

    pub(crate) fn tick_work_byte_22(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(1);
        self.set_work_byte_22(value);
        value
    }

    pub(crate) fn subtract_work_byte_22(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_22 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_work_byte_23(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_23 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_23(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(1);
        self.set_work_byte_23(value);
        value
    }

    pub(crate) fn add_work_byte_23(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_23 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_24(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_24 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_24(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(1);
        self.set_work_byte_24(value);
        value
    }

    pub(crate) fn add_work_byte_24(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_24 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_25(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_25 + self.slot] = value;
    }

    pub(crate) fn set_work_byte_26(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_26 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_25(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_add(1);
        self.set_work_byte_25(value);
        value
    }

    pub(crate) fn retreat_work_byte_25(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_sub(1);
        self.set_work_byte_25(value);
        value
    }

    pub(crate) fn advance_work_byte_4(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_add(1);
        self.set_work_byte_4(value);
        value
    }
}

pub(crate) struct RamOverlordSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> RamOverlordSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_X_LO + self.slot,
            OVERLORD_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_Y_LO + self.slot,
            OVERLORD_Y_HI + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn adjacent_x_low_word(&self) -> u16 {
        word(self.ram, OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, OVERLORD_X_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_HI + self.slot)
    }

    pub(crate) fn overlord_type(&self) -> u8 {
        byte(self.ram, OVERLORD_TYPE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.overlord_type() != 0
    }

    pub(crate) fn gen1(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN1 + self.slot)
    }

    pub(crate) fn gen1_word(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN1 + self.slot,
            OVERLORD_GEN1 + self.slot + 1,
        )
    }

    pub(crate) fn gen2(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN2 + self.slot)
    }

    pub(crate) fn gen2_word(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN2 + self.slot,
            OVERLORD_GEN2 + self.slot + 1,
        )
    }

    pub(crate) fn gen3(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN3 + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, OVERLORD_FLOOR + self.slot)
    }

    pub(crate) fn spawned_area(&self) -> u8 {
        byte(self.ram, OVERLORD_SPAWNED_AREA + self.slot)
    }

    pub(crate) fn sprite_block_pos(&self) -> u16 {
        word(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2)
    }
}

pub(crate) struct RamOverlordSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> RamOverlordSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            OVERLORD_X_LO + self.slot,
            OVERLORD_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] = value;
    }

    pub(crate) fn set_adjacent_x_low_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERLORD_X_LO + self.slot, value);
    }

    pub(crate) fn subtract_adjacent_x_low_word(&mut self, value: u16) -> u16 {
        let updated = word(self.ram, OVERLORD_X_LO + self.slot).wrapping_sub(value);
        self.set_adjacent_x_low_word(updated);
        updated
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.ram[OVERLORD_X_LO + self.slot]
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[OVERLORD_X_HI + self.slot] = value;
    }

    pub(crate) fn increment_x_high(&mut self) {
        self.ram[OVERLORD_X_HI + self.slot] = self.ram[OVERLORD_X_HI + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] =
            self.ram[OVERLORD_X_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_circle_x(&mut self, value: u16) {
        self.ram[OVERLORD_X_HI + self.slot] = value as u8;
        self.ram[OVERLORD_Y_HI + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_circle_y(&mut self, value: u16) {
        self.ram[OVERLORD_GEN2 + self.slot] = value as u8;
        self.ram[OVERLORD_FLOOR + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            OVERLORD_Y_LO + self.slot,
            OVERLORD_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[OVERLORD_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[OVERLORD_Y_HI + self.slot] = value;
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] =
            self.ram[OVERLORD_X_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_overlord_type(&mut self, value: u8) {
        self.ram[OVERLORD_TYPE + self.slot] = value;
    }

    pub(crate) fn clear(&mut self) {
        self.set_overlord_type(0);
    }

    pub(crate) fn set_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] = value;
    }

    pub(crate) fn add_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] =
            self.ram[OVERLORD_GEN1 + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_gen1_word(&mut self, value: u16) {
        let next = read_le_u16(self.ram, OVERLORD_GEN1 + self.slot).wrapping_add(value);
        write_le_u16(self.ram, OVERLORD_GEN1 + self.slot, next);
    }

    pub(crate) fn subtract_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] =
            self.ram[OVERLORD_GEN1 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] = value;
    }

    pub(crate) fn add_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] =
            self.ram[OVERLORD_GEN2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_gen2_word(&mut self, value: u16) {
        let next = read_le_u16(self.ram, OVERLORD_GEN2 + self.slot).wrapping_add(value);
        write_le_u16(self.ram, OVERLORD_GEN2 + self.slot, next);
    }

    pub(crate) fn subtract_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] =
            self.ram[OVERLORD_GEN2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_gen3(&mut self, value: u8) {
        self.ram[OVERLORD_GEN3 + self.slot] = value;
    }

    pub(crate) fn add_gen3(&mut self, value: u8) {
        self.ram[OVERLORD_GEN3 + self.slot] =
            self.ram[OVERLORD_GEN3 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[OVERLORD_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_sprite_block_pos(&mut self, value: u16) {
        write_le_u16(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2, value);
    }

    pub(crate) fn set_spawned_area(&mut self, value: u8) {
        self.ram[OVERLORD_SPAWNED_AREA + self.slot] = value;
    }
}

pub(crate) struct RamGarnishSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> RamGarnishSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn garnish_type(&self) -> u8 {
        byte(self.ram, GARNISH_TYPE + self.slot)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.garnish_type() == 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, GARNISH_X_LO + self.slot, GARNISH_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, GARNISH_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, GARNISH_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, GARNISH_Y_LO + self.slot, GARNISH_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, GARNISH_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, GARNISH_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, GARNISH_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, GARNISH_Y_VELOCITY + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, GARNISH_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, GARNISH_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn countdown(&self) -> u8 {
        byte(self.ram, GARNISH_COUNTDOWN + self.slot)
    }

    pub(crate) fn sprite(&self) -> u8 {
        byte(self.ram, GARNISH_SPRITE + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, GARNISH_FLOOR + self.slot)
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        byte(self.ram, GARNISH_OAM_FLAGS + self.slot)
    }
}

pub(crate) struct RamGarnishSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> RamGarnishSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_garnish_type(&mut self, value: u8) {
        self.ram[GARNISH_TYPE + self.slot] = value;
    }

    pub(crate) fn clear(&mut self) {
        self.set_garnish_type(0);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            GARNISH_X_LO + self.slot,
            GARNISH_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[GARNISH_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[GARNISH_X_HI + self.slot] = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            GARNISH_Y_LO + self.slot,
            GARNISH_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = value;
    }

    pub(crate) fn add_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = self.ram[GARNISH_Y_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = self.ram[GARNISH_Y_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[GARNISH_Y_HI + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[GARNISH_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[GARNISH_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.ram[GARNISH_Y_VELOCITY + self.slot] =
            self.ram[GARNISH_Y_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[GARNISH_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[GARNISH_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.ram[GARNISH_COUNTDOWN + self.slot] = value;
    }

    pub(crate) fn subtract_countdown(&mut self, value: u8) {
        self.ram[GARNISH_COUNTDOWN + self.slot] =
            self.ram[GARNISH_COUNTDOWN + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_sprite(&mut self, value: u8) {
        self.ram[GARNISH_SPRITE + self.slot] = value;
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[GARNISH_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.ram[GARNISH_OAM_FLAGS + self.slot] = value;
    }
}
