use super::*;

pub(crate) struct PolyStateView<'a> {
    ram: &'a [u8],
}

impl<'a> PolyStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn config1(&self) -> u8 {
        byte(self.ram, POLY_CONFIG1)
    }

    pub(crate) fn color_mode(&self) -> u8 {
        byte(self.ram, POLY_CONFIG_COLOR_MODE)
    }

    pub(crate) fn model(&self) -> u8 {
        byte(self.ram, POLY_WHICH_MODEL)
    }

    pub(crate) fn angle_a(&self) -> u8 {
        byte(self.ram, POLY_A)
    }

    pub(crate) fn angle_b(&self) -> u8 {
        byte(self.ram, POLY_B)
    }

    pub(crate) fn base_x(&self) -> u8 {
        byte(self.ram, POLY_BASE_X)
    }

    pub(crate) fn base_y(&self) -> u8 {
        byte(self.ram, POLY_BASE_Y)
    }

    pub(crate) fn shape_depth_bias_low(&self) -> u8 {
        byte(self.ram, POLY_SHAPE_DEPTH_BIAS)
    }

    pub(crate) fn shape_depth_bias(&self) -> u16 {
        word(self.ram, POLY_SHAPE_DEPTH_BIAS)
    }

    pub(crate) fn num_vertices(&self) -> u8 {
        byte(self.ram, POLY_CONFIG_NUM_VERTEX)
    }

    pub(crate) fn num_polys(&self) -> u8 {
        byte(self.ram, POLY_CONFIG_NUM_POLYS)
    }

    pub(crate) fn fromlut_x(&self) -> i8 {
        byte(self.ram, POLY_FROMLUT_X) as i8
    }

    pub(crate) fn fromlut_y(&self) -> i8 {
        byte(self.ram, POLY_FROMLUT_Y) as i8
    }

    pub(crate) fn fromlut_z(&self) -> i8 {
        byte(self.ram, POLY_FROMLUT_Z) as i8
    }

    pub(crate) fn f0(&self) -> u8 {
        byte(self.ram, POLY_F0)
    }

    pub(crate) fn f1(&self) -> u8 {
        byte(self.ram, POLY_F1)
    }

    pub(crate) fn f0_word(&self) -> u16 {
        word(self.ram, POLY_F0)
    }

    pub(crate) fn f1_word(&self) -> u16 {
        word(self.ram, POLY_F1)
    }

    pub(crate) fn f2_word(&self) -> u16 {
        word(self.ram, POLY_F2)
    }

    pub(crate) fn rotation_sin_a(&self) -> i16 {
        word(self.ram, POLY_SIN_A) as i16
    }

    pub(crate) fn rotation_cos_a(&self) -> i16 {
        word(self.ram, POLY_COS_A) as i16
    }

    pub(crate) fn rotation_sin_b(&self) -> i16 {
        word(self.ram, POLY_SIN_B) as i16
    }

    pub(crate) fn rotation_cos_b(&self) -> i16 {
        word(self.ram, POLY_COS_B) as i16
    }

    pub(crate) fn rotation_e0(&self) -> i16 {
        word(self.ram, POLY_E0) as i16
    }

    pub(crate) fn rotation_e1(&self) -> i16 {
        word(self.ram, POLY_E1) as i16
    }

    pub(crate) fn rotation_e2(&self) -> i16 {
        word(self.ram, POLY_E2) as i16
    }

    pub(crate) fn rotation_e3(&self) -> i16 {
        word(self.ram, POLY_E3) as i16
    }

    pub(crate) fn num_vertex_in_poly(&self) -> u8 {
        byte(self.ram, POLY_NUM_VERTEX_IN_POLY)
    }

    pub(crate) fn raster_color_config(&self) -> u8 {
        byte(self.ram, POLY_RASTER_COLOR_CONFIG)
    }

    pub(crate) fn tmp0(&self) -> u8 {
        byte(self.ram, POLY_TMP0)
    }

    pub(crate) fn tmp0_word(&self) -> u16 {
        word(self.ram, POLY_TMP0)
    }

    pub(crate) fn tmp1_word(&self) -> u16 {
        word(self.ram, POLY_TMP1)
    }

    pub(crate) fn tmp2(&self) -> u8 {
        byte(self.ram, POLY_TMP2)
    }

    pub(crate) fn raster_color0(&self) -> u16 {
        word(self.ram, POLY_RASTER_COLOR0)
    }

    pub(crate) fn raster_color1(&self) -> u16 {
        word(self.ram, POLY_RASTER_COLOR1)
    }

    pub(crate) fn raster_dst_ptr(&self) -> u16 {
        word(self.ram, POLY_RASTER_DST_PTR)
    }

    pub(crate) fn x0_fraction(&self) -> u16 {
        word(self.ram, POLY_X0_FRAC)
    }

    pub(crate) fn x0_step(&self) -> u16 {
        word(self.ram, POLY_X0_STEP)
    }

    pub(crate) fn x1_fraction(&self) -> u16 {
        word(self.ram, POLY_X1_FRAC)
    }

    pub(crate) fn x1_step(&self) -> u16 {
        word(self.ram, POLY_X1_STEP)
    }

    pub(crate) fn bitmap_word(&self, offset: usize) -> u16 {
        word(self.ram, offset)
    }
}

pub(crate) struct PolyStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> PolyStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_config1(&mut self, value: u8) {
        self.ram[POLY_CONFIG1] = value;
    }

    pub(crate) fn clear_config1(&mut self) {
        self.ram[POLY_CONFIG1] = 0;
    }

    pub(crate) fn increment_config1(&mut self) -> u8 {
        self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_add(1);
        self.ram[POLY_CONFIG1]
    }

    pub(crate) fn subtract_config1(&mut self, value: u8) -> u8 {
        self.ram[POLY_CONFIG1] = self.ram[POLY_CONFIG1].wrapping_sub(value);
        self.ram[POLY_CONFIG1]
    }

    pub(crate) fn set_color_mode(&mut self, value: u8) {
        self.ram[POLY_CONFIG_COLOR_MODE] = value;
    }

    pub(crate) fn set_model(&mut self, value: u8) {
        self.ram[POLY_WHICH_MODEL] = value;
    }

    pub(crate) fn set_angle_a(&mut self, value: u8) {
        self.ram[POLY_A] = value;
    }

    pub(crate) fn set_angle_b(&mut self, value: u8) {
        self.ram[POLY_B] = value;
    }

    pub(crate) fn clear_angles(&mut self) {
        self.ram[POLY_A] = 0;
        self.ram[POLY_B] = 0;
    }

    pub(crate) fn add_angle_a(&mut self, value: u8) -> u8 {
        self.ram[POLY_A] = self.ram[POLY_A].wrapping_add(value);
        self.ram[POLY_A]
    }

    pub(crate) fn add_angle_b(&mut self, value: u8) -> u8 {
        self.ram[POLY_B] = self.ram[POLY_B].wrapping_add(value);
        self.ram[POLY_B]
    }

    pub(crate) fn add_angles(&mut self, angle_a: u8, angle_b: u8) {
        self.add_angle_a(angle_a);
        self.add_angle_b(angle_b);
    }

    pub(crate) fn set_base_x(&mut self, value: u8) {
        self.ram[POLY_BASE_X] = value;
    }

    pub(crate) fn set_base_y(&mut self, value: u8) {
        self.ram[POLY_BASE_Y] = value;
    }

    pub(crate) fn set_base_position(&mut self, x: u8, y: u8) {
        self.ram[POLY_BASE_X] = x;
        self.ram[POLY_BASE_Y] = y;
    }

    pub(crate) fn set_shape_depth_bias_low(&mut self, value: u8) {
        self.ram[POLY_SHAPE_DEPTH_BIAS] = value;
    }

    pub(crate) fn set_shape_depth_bias(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_SHAPE_DEPTH_BIAS, value);
    }

    pub(crate) fn set_num_vertices(&mut self, value: u8) {
        self.ram[POLY_CONFIG_NUM_VERTEX] = value;
    }

    pub(crate) fn set_num_polys(&mut self, value: u8) {
        self.ram[POLY_CONFIG_NUM_POLYS] = value;
    }

    pub(crate) fn decrement_num_polys(&mut self) -> u8 {
        self.ram[POLY_CONFIG_NUM_POLYS] = self.ram[POLY_CONFIG_NUM_POLYS].wrapping_sub(1);
        self.ram[POLY_CONFIG_NUM_POLYS]
    }

    pub(crate) fn set_fromlut_position(&mut self, x: u8, y: u8, z: u8) {
        self.ram[POLY_FROMLUT_X] = x;
        self.ram[POLY_FROMLUT_Y] = y;
        self.ram[POLY_FROMLUT_Z] = z;
    }

    pub(crate) fn set_vertex_table_pointer(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_FROMLUT_PTR2, value);
    }

    pub(crate) fn set_face_table_pointer(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_FROMLUT_PTR4, value);
    }

    pub(crate) fn set_rotation_sin_a(&mut self, value: i16) {
        write_le_u16(self.ram, POLY_SIN_A, value as u16);
    }

    pub(crate) fn set_rotation_cos_a(&mut self, value: i16) {
        write_le_u16(self.ram, POLY_COS_A, value as u16);
    }

    pub(crate) fn set_rotation_sin_b(&mut self, value: i16) {
        write_le_u16(self.ram, POLY_SIN_B, value as u16);
    }

    pub(crate) fn set_rotation_cos_b(&mut self, value: i16) {
        write_le_u16(self.ram, POLY_COS_B, value as u16);
    }

    pub(crate) fn set_rotation_matrix_terms(&mut self, e0: i16, e1: i16, e2: i16, e3: i16) {
        write_le_u16(self.ram, POLY_E0, e0 as u16);
        write_le_u16(self.ram, POLY_E1, e1 as u16);
        write_le_u16(self.ram, POLY_E2, e2 as u16);
        write_le_u16(self.ram, POLY_E3, e3 as u16);
    }

    pub(crate) fn set_f0_word(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_F0, value);
    }

    pub(crate) fn set_f1_word(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_F1, value);
    }

    pub(crate) fn set_f2_word(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_F2, value);
    }

    pub(crate) fn set_num_vertex_in_poly(&mut self, value: u8) {
        self.ram[POLY_NUM_VERTEX_IN_POLY] = value;
    }

    pub(crate) fn set_raster_color_config(&mut self, value: u8) {
        self.ram[POLY_RASTER_COLOR_CONFIG] = value;
    }

    pub(crate) fn set_tmp0(&mut self, value: u8) {
        self.ram[POLY_TMP0] = value;
    }

    pub(crate) fn set_tmp0_word(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_TMP0, value);
    }

    pub(crate) fn set_tmp1_word(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_TMP1, value);
    }

    pub(crate) fn decrement_tmp0(&mut self) -> u8 {
        self.ram[POLY_TMP0] = self.ram[POLY_TMP0].wrapping_sub(1);
        self.ram[POLY_TMP0]
    }

    pub(crate) fn set_tmp2(&mut self, value: u8) {
        self.ram[POLY_TMP2] = value;
    }

    pub(crate) fn set_raster_colors(&mut self, color0: u16, color1: u16) {
        write_le_u16(self.ram, POLY_RASTER_COLOR0, color0);
        write_le_u16(self.ram, POLY_RASTER_COLOR1, color1);
    }

    pub(crate) fn set_raster_dst_ptr(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_RASTER_DST_PTR, value);
    }

    pub(crate) fn set_x0_fraction(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_X0_FRAC, value);
    }

    pub(crate) fn add_x0_step_to_fraction(&mut self) {
        let value =
            read_le_u16(self.ram, POLY_X0_FRAC).wrapping_add(read_le_u16(self.ram, POLY_X0_STEP));
        write_le_u16(self.ram, POLY_X0_FRAC, value);
    }

    pub(crate) fn set_x0_step(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_X0_STEP, value);
    }

    pub(crate) fn set_x1_fraction(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_X1_FRAC, value);
    }

    pub(crate) fn add_x1_step_to_fraction(&mut self) {
        let value =
            read_le_u16(self.ram, POLY_X1_FRAC).wrapping_add(read_le_u16(self.ram, POLY_X1_STEP));
        write_le_u16(self.ram, POLY_X1_FRAC, value);
    }

    pub(crate) fn set_x1_step(&mut self, value: u16) {
        write_le_u16(self.ram, POLY_X1_STEP, value);
    }

    pub(crate) fn clear_raster_full_word_count(&mut self) {
        write_le_u16(self.ram, POLY_RASTER_NUMFULL, 0);
    }

    pub(crate) fn set_bitmap_word(&mut self, offset: usize, value: u16) {
        write_le_u16(self.ram, offset, value);
    }

    pub(crate) fn blend_bitmap_word(&mut self, offset: usize, color: u16, mask: u16) {
        let current = read_le_u16(self.ram, offset);
        write_le_u16(self.ram, offset, current ^ ((current ^ color) & mask));
    }

    pub(crate) fn clear_poly_buffer(&mut self) {
        self.ram[POLYHEDRAL_BUFFER..POLYHEDRAL_BUFFER + 0x800].fill(0);
    }
}

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
