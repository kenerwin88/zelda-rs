use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const POLY_PROJECTED_VERTEX_COUNT: usize = 40;
const POLY_FACE_COORD_COUNT: usize = 0x20;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyState {
    pub(crate) runtime: PolyRuntimeState,
    pub(crate) projected_vertices: PolyProjectedVerticesState,
    pub(crate) face_coords: PolyFaceCoordsState,
    pub(crate) raster_edge: PolyRasterEdgeState,
}

impl PolyState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            runtime: PolyRuntimeState::load_from_ram(ram),
            projected_vertices: PolyProjectedVerticesState::load_from_ram(ram),
            face_coords: PolyFaceCoordsState::load_from_ram(ram),
            raster_edge: PolyRasterEdgeState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.runtime.write_to_ram(ram);
        self.projected_vertices.write_to_ram(ram);
        self.face_coords.write_to_ram(ram);
        self.raster_edge.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyRuntimeState {
    config1: u8,
    color_mode: u8,
    model: u8,
    angle_a: u8,
    angle_b: u8,
    base_x: u8,
    base_y: u8,
    shape_depth_bias: u16,
    num_vertices: u8,
    num_polys: u8,
    fromlut_x: u8,
    fromlut_y: u8,
    fromlut_z: u8,
    vertex_table_pointer: u16,
    face_table_pointer: u16,
    f0: u16,
    f1: u16,
    f2: u16,
    rotation_sin_a: i16,
    rotation_cos_a: i16,
    rotation_sin_b: i16,
    rotation_cos_b: i16,
    rotation_e0: i16,
    rotation_e1: i16,
    rotation_e2: i16,
    rotation_e3: i16,
    num_vertex_in_poly: u8,
    raster_color_config: u8,
    tmp0: u16,
    tmp1: u16,
    tmp2: u8,
    raster_color0: u16,
    raster_color1: u16,
    raster_dst_ptr: u16,
    x0_fraction: u16,
    x0_step: u16,
    x1_fraction: u16,
    x1_step: u16,
    raster_full_word_count: u16,
}

impl PolyRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            config1: ram.get(POLY_CONFIG1).copied().unwrap_or(0),
            color_mode: ram.get(POLY_CONFIG_COLOR_MODE).copied().unwrap_or(0),
            model: ram.get(POLY_WHICH_MODEL).copied().unwrap_or(0),
            angle_a: ram.get(POLY_A).copied().unwrap_or(0),
            angle_b: ram.get(POLY_B).copied().unwrap_or(0),
            base_x: ram.get(POLY_BASE_X).copied().unwrap_or(0),
            base_y: ram.get(POLY_BASE_Y).copied().unwrap_or(0),
            shape_depth_bias: read_le_u16(ram, POLY_SHAPE_DEPTH_BIAS),
            num_vertices: ram.get(POLY_CONFIG_NUM_VERTEX).copied().unwrap_or(0),
            num_polys: ram.get(POLY_CONFIG_NUM_POLYS).copied().unwrap_or(0),
            fromlut_x: ram.get(POLY_FROMLUT_X).copied().unwrap_or(0),
            fromlut_y: ram.get(POLY_FROMLUT_Y).copied().unwrap_or(0),
            fromlut_z: ram.get(POLY_FROMLUT_Z).copied().unwrap_or(0),
            vertex_table_pointer: read_le_u16(ram, POLY_FROMLUT_PTR2),
            face_table_pointer: read_le_u16(ram, POLY_FROMLUT_PTR4),
            f0: read_le_u16(ram, POLY_F0),
            f1: read_le_u16(ram, POLY_F1),
            f2: read_le_u16(ram, POLY_F2),
            rotation_sin_a: read_le_u16(ram, POLY_SIN_A) as i16,
            rotation_cos_a: read_le_u16(ram, POLY_COS_A) as i16,
            rotation_sin_b: read_le_u16(ram, POLY_SIN_B) as i16,
            rotation_cos_b: read_le_u16(ram, POLY_COS_B) as i16,
            rotation_e0: read_le_u16(ram, POLY_E0) as i16,
            rotation_e1: read_le_u16(ram, POLY_E1) as i16,
            rotation_e2: read_le_u16(ram, POLY_E2) as i16,
            rotation_e3: read_le_u16(ram, POLY_E3) as i16,
            num_vertex_in_poly: ram.get(POLY_NUM_VERTEX_IN_POLY).copied().unwrap_or(0),
            raster_color_config: ram.get(POLY_RASTER_COLOR_CONFIG).copied().unwrap_or(0),
            tmp0: read_le_u16(ram, POLY_TMP0),
            tmp1: read_le_u16(ram, POLY_TMP1),
            tmp2: ram.get(POLY_TMP2).copied().unwrap_or(0),
            raster_color0: read_le_u16(ram, POLY_RASTER_COLOR0),
            raster_color1: read_le_u16(ram, POLY_RASTER_COLOR1),
            raster_dst_ptr: read_le_u16(ram, POLY_RASTER_DST_PTR),
            x0_fraction: read_le_u16(ram, POLY_X0_FRAC),
            x0_step: read_le_u16(ram, POLY_X0_STEP),
            x1_fraction: read_le_u16(ram, POLY_X1_FRAC),
            x1_step: read_le_u16(ram, POLY_X1_STEP),
            raster_full_word_count: read_le_u16(ram, POLY_RASTER_NUMFULL),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[POLY_CONFIG1] = self.config1;
        ram[POLY_CONFIG_COLOR_MODE] = self.color_mode;
        ram[POLY_WHICH_MODEL] = self.model;
        ram[POLY_A] = self.angle_a;
        ram[POLY_B] = self.angle_b;
        ram[POLY_BASE_X] = self.base_x;
        ram[POLY_BASE_Y] = self.base_y;
        write_le_u16(ram, POLY_SHAPE_DEPTH_BIAS, self.shape_depth_bias);
        ram[POLY_CONFIG_NUM_VERTEX] = self.num_vertices;
        ram[POLY_CONFIG_NUM_POLYS] = self.num_polys;
        ram[POLY_FROMLUT_X] = self.fromlut_x;
        ram[POLY_FROMLUT_Y] = self.fromlut_y;
        ram[POLY_FROMLUT_Z] = self.fromlut_z;
        write_le_u16(ram, POLY_FROMLUT_PTR2, self.vertex_table_pointer);
        write_le_u16(ram, POLY_FROMLUT_PTR4, self.face_table_pointer);
        write_le_u16(ram, POLY_F0, self.f0);
        write_le_u16(ram, POLY_F1, self.f1);
        write_le_u16(ram, POLY_F2, self.f2);
        write_le_u16(ram, POLY_SIN_A, self.rotation_sin_a as u16);
        write_le_u16(ram, POLY_COS_A, self.rotation_cos_a as u16);
        write_le_u16(ram, POLY_SIN_B, self.rotation_sin_b as u16);
        write_le_u16(ram, POLY_COS_B, self.rotation_cos_b as u16);
        write_le_u16(ram, POLY_E0, self.rotation_e0 as u16);
        write_le_u16(ram, POLY_E1, self.rotation_e1 as u16);
        write_le_u16(ram, POLY_E2, self.rotation_e2 as u16);
        write_le_u16(ram, POLY_E3, self.rotation_e3 as u16);
        ram[POLY_NUM_VERTEX_IN_POLY] = self.num_vertex_in_poly;
        ram[POLY_RASTER_COLOR_CONFIG] = self.raster_color_config;
        write_le_u16(ram, POLY_TMP0, self.tmp0);
        write_le_u16(ram, POLY_TMP1, self.tmp1);
        ram[POLY_TMP2] = self.tmp2;
        write_le_u16(ram, POLY_RASTER_COLOR0, self.raster_color0);
        write_le_u16(ram, POLY_RASTER_COLOR1, self.raster_color1);
        write_le_u16(ram, POLY_RASTER_DST_PTR, self.raster_dst_ptr);
        write_le_u16(ram, POLY_X0_FRAC, self.x0_fraction);
        write_le_u16(ram, POLY_X0_STEP, self.x0_step);
        write_le_u16(ram, POLY_X1_FRAC, self.x1_fraction);
        write_le_u16(ram, POLY_X1_STEP, self.x1_step);
        write_le_u16(ram, POLY_RASTER_NUMFULL, self.raster_full_word_count);
    }

    pub(crate) fn config1(&self) -> u8 {
        self.config1
    }

    pub(crate) fn color_mode(&self) -> u8 {
        self.color_mode
    }

    pub(crate) fn model(&self) -> u8 {
        self.model
    }

    pub(crate) fn angle_a(&self) -> u8 {
        self.angle_a
    }

    pub(crate) fn angle_b(&self) -> u8 {
        self.angle_b
    }

    pub(crate) fn base_x(&self) -> u8 {
        self.base_x
    }

    pub(crate) fn base_y(&self) -> u8 {
        self.base_y
    }

    pub(crate) fn shape_depth_bias_low(&self) -> u8 {
        self.shape_depth_bias as u8
    }

    pub(crate) fn shape_depth_bias(&self) -> u16 {
        self.shape_depth_bias
    }

    pub(crate) fn num_vertices(&self) -> u8 {
        self.num_vertices
    }

    pub(crate) fn num_polys(&self) -> u8 {
        self.num_polys
    }

    pub(crate) fn fromlut_x(&self) -> i8 {
        self.fromlut_x as i8
    }

    pub(crate) fn fromlut_y(&self) -> i8 {
        self.fromlut_y as i8
    }

    pub(crate) fn fromlut_z(&self) -> i8 {
        self.fromlut_z as i8
    }

    pub(crate) fn f0(&self) -> u8 {
        self.f0 as u8
    }

    pub(crate) fn f1(&self) -> u8 {
        self.f1 as u8
    }

    pub(crate) fn f0_word(&self) -> u16 {
        self.f0
    }

    pub(crate) fn f1_word(&self) -> u16 {
        self.f1
    }

    pub(crate) fn f2_word(&self) -> u16 {
        self.f2
    }

    pub(crate) fn rotation_sin_a(&self) -> i16 {
        self.rotation_sin_a
    }

    pub(crate) fn rotation_cos_a(&self) -> i16 {
        self.rotation_cos_a
    }

    pub(crate) fn rotation_sin_b(&self) -> i16 {
        self.rotation_sin_b
    }

    pub(crate) fn rotation_cos_b(&self) -> i16 {
        self.rotation_cos_b
    }

    pub(crate) fn rotation_e0(&self) -> i16 {
        self.rotation_e0
    }

    pub(crate) fn rotation_e1(&self) -> i16 {
        self.rotation_e1
    }

    pub(crate) fn rotation_e2(&self) -> i16 {
        self.rotation_e2
    }

    pub(crate) fn rotation_e3(&self) -> i16 {
        self.rotation_e3
    }

    pub(crate) fn num_vertex_in_poly(&self) -> u8 {
        self.num_vertex_in_poly
    }

    pub(crate) fn raster_color_config(&self) -> u8 {
        self.raster_color_config
    }

    pub(crate) fn tmp0(&self) -> u8 {
        self.tmp0 as u8
    }

    pub(crate) fn tmp0_word(&self) -> u16 {
        self.tmp0
    }

    pub(crate) fn tmp1_word(&self) -> u16 {
        self.tmp1
    }

    pub(crate) fn tmp2(&self) -> u8 {
        self.tmp2
    }

    pub(crate) fn raster_color0(&self) -> u16 {
        self.raster_color0
    }

    pub(crate) fn raster_color1(&self) -> u16 {
        self.raster_color1
    }

    pub(crate) fn raster_dst_ptr(&self) -> u16 {
        self.raster_dst_ptr
    }

    pub(crate) fn x0_fraction(&self) -> u16 {
        self.x0_fraction
    }

    pub(crate) fn x0_step(&self) -> u16 {
        self.x0_step
    }

    pub(crate) fn x1_fraction(&self) -> u16 {
        self.x1_fraction
    }

    pub(crate) fn x1_step(&self) -> u16 {
        self.x1_step
    }
}

pub(crate) struct NativePolyRuntimeBridgeMut<'a> {
    runtime: &'a mut PolyRuntimeState,
    ram: &'a mut [u8],
}

impl<'a> NativePolyRuntimeBridgeMut<'a> {
    pub(crate) fn new(runtime: &'a mut PolyRuntimeState, ram: &'a mut [u8]) -> Self {
        Self { runtime, ram }
    }

    fn sync(&mut self) {
        self.runtime.write_to_ram(self.ram);
        debug_assert_eq!(*self.runtime, PolyRuntimeState::load_from_ram(self.ram));
    }

    pub(crate) fn set_config1(&mut self, value: u8) {
        self.runtime.config1 = value;
        self.sync();
    }

    pub(crate) fn clear_config1(&mut self) {
        self.set_config1(0);
    }

    pub(crate) fn increment_config1(&mut self) -> u8 {
        self.runtime.config1 = self.runtime.config1.wrapping_add(1);
        let value = self.runtime.config1;
        self.sync();
        value
    }

    pub(crate) fn subtract_config1(&mut self, value: u8) -> u8 {
        self.runtime.config1 = self.runtime.config1.wrapping_sub(value);
        let next = self.runtime.config1;
        self.sync();
        next
    }

    pub(crate) fn set_color_mode(&mut self, value: u8) {
        self.runtime.color_mode = value;
        self.sync();
    }

    pub(crate) fn set_model(&mut self, value: u8) {
        self.runtime.model = value;
        self.sync();
    }

    pub(crate) fn set_angle_a(&mut self, value: u8) {
        self.runtime.angle_a = value;
        self.sync();
    }

    pub(crate) fn set_angle_b(&mut self, value: u8) {
        self.runtime.angle_b = value;
        self.sync();
    }

    pub(crate) fn clear_angles(&mut self) {
        self.runtime.angle_a = 0;
        self.runtime.angle_b = 0;
        self.sync();
    }

    pub(crate) fn add_angle_a(&mut self, value: u8) -> u8 {
        self.runtime.angle_a = self.runtime.angle_a.wrapping_add(value);
        let next = self.runtime.angle_a;
        self.sync();
        next
    }

    pub(crate) fn add_angle_b(&mut self, value: u8) -> u8 {
        self.runtime.angle_b = self.runtime.angle_b.wrapping_add(value);
        let next = self.runtime.angle_b;
        self.sync();
        next
    }

    pub(crate) fn add_angles(&mut self, angle_a: u8, angle_b: u8) {
        self.runtime.angle_a = self.runtime.angle_a.wrapping_add(angle_a);
        self.runtime.angle_b = self.runtime.angle_b.wrapping_add(angle_b);
        self.sync();
    }

    pub(crate) fn set_base_x(&mut self, value: u8) {
        self.runtime.base_x = value;
        self.sync();
    }

    pub(crate) fn set_base_y(&mut self, value: u8) {
        self.runtime.base_y = value;
        self.sync();
    }

    pub(crate) fn set_base_position(&mut self, x: u8, y: u8) {
        self.runtime.base_x = x;
        self.runtime.base_y = y;
        self.sync();
    }

    pub(crate) fn set_shape_depth_bias_low(&mut self, value: u8) {
        self.runtime.shape_depth_bias = (self.runtime.shape_depth_bias & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_shape_depth_bias(&mut self, value: u16) {
        self.runtime.shape_depth_bias = value;
        self.sync();
    }

    pub(crate) fn set_num_vertices(&mut self, value: u8) {
        self.runtime.num_vertices = value;
        self.sync();
    }

    pub(crate) fn set_num_polys(&mut self, value: u8) {
        self.runtime.num_polys = value;
        self.sync();
    }

    pub(crate) fn decrement_num_polys(&mut self) -> u8 {
        self.runtime.num_polys = self.runtime.num_polys.wrapping_sub(1);
        let next = self.runtime.num_polys;
        self.sync();
        next
    }

    pub(crate) fn set_fromlut_position(&mut self, x: u8, y: u8, z: u8) {
        self.runtime.fromlut_x = x;
        self.runtime.fromlut_y = y;
        self.runtime.fromlut_z = z;
        self.sync();
    }

    pub(crate) fn set_vertex_table_pointer(&mut self, value: u16) {
        self.runtime.vertex_table_pointer = value;
        self.sync();
    }

    pub(crate) fn set_face_table_pointer(&mut self, value: u16) {
        self.runtime.face_table_pointer = value;
        self.sync();
    }

    pub(crate) fn set_rotation_sin_a(&mut self, value: i16) {
        self.runtime.rotation_sin_a = value;
        self.sync();
    }

    pub(crate) fn set_rotation_cos_a(&mut self, value: i16) {
        self.runtime.rotation_cos_a = value;
        self.sync();
    }

    pub(crate) fn set_rotation_sin_b(&mut self, value: i16) {
        self.runtime.rotation_sin_b = value;
        self.sync();
    }

    pub(crate) fn set_rotation_cos_b(&mut self, value: i16) {
        self.runtime.rotation_cos_b = value;
        self.sync();
    }

    pub(crate) fn set_rotation_matrix_terms(&mut self, e0: i16, e1: i16, e2: i16, e3: i16) {
        self.runtime.rotation_e0 = e0;
        self.runtime.rotation_e1 = e1;
        self.runtime.rotation_e2 = e2;
        self.runtime.rotation_e3 = e3;
        self.sync();
    }

    pub(crate) fn set_f0_word(&mut self, value: u16) {
        self.runtime.f0 = value;
        self.sync();
    }

    pub(crate) fn set_f1_word(&mut self, value: u16) {
        self.runtime.f1 = value;
        self.sync();
    }

    pub(crate) fn set_f2_word(&mut self, value: u16) {
        self.runtime.f2 = value;
        self.sync();
    }

    pub(crate) fn set_num_vertex_in_poly(&mut self, value: u8) {
        self.runtime.num_vertex_in_poly = value;
        self.sync();
    }

    pub(crate) fn set_raster_color_config(&mut self, value: u8) {
        self.runtime.raster_color_config = value;
        self.sync();
    }

    pub(crate) fn set_tmp0(&mut self, value: u8) {
        self.runtime.tmp0 = (self.runtime.tmp0 & 0xff00) | u16::from(value);
        self.sync();
    }

    pub(crate) fn set_tmp0_word(&mut self, value: u16) {
        self.runtime.tmp0 = value;
        self.sync();
    }

    pub(crate) fn set_tmp1_word(&mut self, value: u16) {
        self.runtime.tmp1 = value;
        self.sync();
    }

    pub(crate) fn decrement_tmp0(&mut self) -> u8 {
        let low = (self.runtime.tmp0 as u8).wrapping_sub(1);
        self.runtime.tmp0 = (self.runtime.tmp0 & 0xff00) | u16::from(low);
        self.sync();
        low
    }

    pub(crate) fn set_tmp2(&mut self, value: u8) {
        self.runtime.tmp2 = value;
        self.sync();
    }

    pub(crate) fn set_raster_colors(&mut self, color0: u16, color1: u16) {
        self.runtime.raster_color0 = color0;
        self.runtime.raster_color1 = color1;
        self.sync();
    }

    pub(crate) fn set_raster_dst_ptr(&mut self, value: u16) {
        self.runtime.raster_dst_ptr = value;
        self.sync();
    }

    pub(crate) fn set_x0_fraction(&mut self, value: u16) {
        self.runtime.x0_fraction = value;
        self.sync();
    }

    pub(crate) fn add_x0_step_to_fraction(&mut self) {
        self.runtime.x0_fraction = self.runtime.x0_fraction.wrapping_add(self.runtime.x0_step);
        self.sync();
    }

    pub(crate) fn set_x0_step(&mut self, value: u16) {
        self.runtime.x0_step = value;
        self.sync();
    }

    pub(crate) fn set_x1_fraction(&mut self, value: u16) {
        self.runtime.x1_fraction = value;
        self.sync();
    }

    pub(crate) fn add_x1_step_to_fraction(&mut self) {
        self.runtime.x1_fraction = self.runtime.x1_fraction.wrapping_add(self.runtime.x1_step);
        self.sync();
    }

    pub(crate) fn set_x1_step(&mut self, value: u16) {
        self.runtime.x1_step = value;
        self.sync();
    }

    pub(crate) fn clear_raster_full_word_count(&mut self) {
        self.runtime.raster_full_word_count = 0;
        self.sync();
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyProjectedVerticesState {
    x: Vec<u8>,
    y: Vec<u8>,
}

impl Default for PolyProjectedVerticesState {
    fn default() -> Self {
        Self {
            x: vec![0; POLY_PROJECTED_VERTEX_COUNT],
            y: vec![0; POLY_PROJECTED_VERTEX_COUNT],
        }
    }
}

impl PolyProjectedVerticesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for vertex in 0..POLY_PROJECTED_VERTEX_COUNT {
            state.x[vertex] = ram.get(POLY_PROJECTED_X + vertex).copied().unwrap_or(0);
            state.y[vertex] = ram.get(POLY_PROJECTED_Y + vertex).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for vertex in 0..POLY_PROJECTED_VERTEX_COUNT {
            ram[POLY_PROJECTED_X + vertex] = self.x.get(vertex).copied().unwrap_or(0);
            ram[POLY_PROJECTED_Y + vertex] = self.y.get(vertex).copied().unwrap_or(0);
        }
    }

    pub(crate) fn x(&self, vertex: usize) -> u8 {
        self.x.get(vertex).copied().unwrap_or(0)
    }

    pub(crate) fn y(&self, vertex: usize) -> u8 {
        self.y.get(vertex).copied().unwrap_or(0)
    }

    pub(crate) fn set_position(&mut self, vertex: usize, x: u8, y: u8) {
        if vertex < POLY_PROJECTED_VERTEX_COUNT {
            self.x[vertex] = x;
            self.y[vertex] = y;
        }
    }
}

pub(crate) struct NativePolyProjectedVerticesBridgeMut<'a> {
    state: &'a mut PolyProjectedVerticesState,
    ram: &'a mut [u8],
}

impl<'a> NativePolyProjectedVerticesBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PolyProjectedVerticesState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        debug_assert_eq!(
            *self.state,
            PolyProjectedVerticesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_position(&mut self, vertex: usize, x: u8, y: u8) {
        self.state.set_position(vertex, x, y);
        self.sync();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyFaceCoordsState {
    coords: [u8; POLY_FACE_COORD_COUNT],
}

impl Default for PolyFaceCoordsState {
    fn default() -> Self {
        Self {
            coords: [0; POLY_FACE_COORD_COUNT],
        }
    }
}

impl PolyFaceCoordsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut coords = [0; POLY_FACE_COORD_COUNT];
        for (offset, coord) in coords.iter_mut().enumerate() {
            *coord = ram.get(POLY_FACE_COORDS + offset).copied().unwrap_or(0);
        }
        Self { coords }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[POLY_FACE_COORDS..POLY_FACE_COORDS + POLY_FACE_COORD_COUNT]
            .copy_from_slice(&self.coords);
    }

    pub(crate) fn coord(&self, offset: usize) -> u8 {
        self.coords.get(offset).copied().unwrap_or(0)
    }

    pub(crate) fn xy_coords_count(&self) -> u8 {
        self.coord(0)
    }

    pub(crate) fn set_coord(&mut self, offset: usize, value: u8) {
        if let Some(coord) = self.coords.get_mut(offset) {
            *coord = value;
        }
    }

    pub(crate) fn set_xy_coords_count(&mut self, value: u8) {
        self.coords[0] = value;
    }
}

pub(crate) struct NativePolyFaceCoordsBridgeMut<'a> {
    state: &'a mut PolyFaceCoordsState,
    ram: &'a mut [u8],
}

impl<'a> NativePolyFaceCoordsBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PolyFaceCoordsState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        debug_assert_eq!(*self.state, PolyFaceCoordsState::load_from_ram(self.ram));
    }

    pub(crate) fn set_coord(&mut self, offset: usize, value: u8) {
        self.state.set_coord(offset, value);
        self.sync();
    }

    pub(crate) fn set_xy_coords_count(&mut self, value: u8) {
        self.state.set_xy_coords_count(value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyRasterEdgeState {
    total_num_steps: u8,
    x0_current: u8,
    y0_current: u8,
    x0_target: u8,
    y0_trigger: u8,
    x1_current: u8,
    y1_current: u8,
    x1_target: u8,
    y1_trigger: u8,
    current_vertex_index0: u8,
    current_vertex_index1: u8,
}

impl PolyRasterEdgeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            total_num_steps: ram.get(POLY_TOTAL_NUM_STEPS).copied().unwrap_or(0),
            x0_current: ram.get(POLY_X0_CUR).copied().unwrap_or(0),
            y0_current: ram.get(POLY_Y0_CUR).copied().unwrap_or(0),
            x0_target: ram.get(POLY_X0_TARGET).copied().unwrap_or(0),
            y0_trigger: ram.get(POLY_Y0_TRIG).copied().unwrap_or(0),
            x1_current: ram.get(POLY_X1_CUR).copied().unwrap_or(0),
            y1_current: ram.get(POLY_Y1_CUR).copied().unwrap_or(0),
            x1_target: ram.get(POLY_X1_TARGET).copied().unwrap_or(0),
            y1_trigger: ram.get(POLY_Y1_TRIG).copied().unwrap_or(0),
            current_vertex_index0: ram.get(POLY_CUR_VERTEX_IDX0).copied().unwrap_or(0),
            current_vertex_index1: ram.get(POLY_CUR_VERTEX_IDX1).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[POLY_TOTAL_NUM_STEPS] = self.total_num_steps;
        ram[POLY_X0_CUR] = self.x0_current;
        ram[POLY_Y0_CUR] = self.y0_current;
        ram[POLY_X0_TARGET] = self.x0_target;
        ram[POLY_Y0_TRIG] = self.y0_trigger;
        ram[POLY_X1_CUR] = self.x1_current;
        ram[POLY_Y1_CUR] = self.y1_current;
        ram[POLY_X1_TARGET] = self.x1_target;
        ram[POLY_Y1_TRIG] = self.y1_trigger;
        ram[POLY_CUR_VERTEX_IDX0] = self.current_vertex_index0;
        ram[POLY_CUR_VERTEX_IDX1] = self.current_vertex_index1;
    }

    pub(crate) fn x0_cur(&self) -> u8 {
        self.x0_current
    }

    pub(crate) fn y0_cur(&self) -> u8 {
        self.y0_current
    }

    pub(crate) fn x1_cur(&self) -> u8 {
        self.x1_current
    }

    pub(crate) fn y1_cur(&self) -> u8 {
        self.y1_current
    }

    pub(crate) fn x0_target(&self) -> u8 {
        self.x0_target
    }

    pub(crate) fn y0_trigger(&self) -> u8 {
        self.y0_trigger
    }

    pub(crate) fn x1_target(&self) -> u8 {
        self.x1_target
    }

    pub(crate) fn y1_trigger(&self) -> u8 {
        self.y1_trigger
    }

    pub(crate) fn total_num_steps(&self) -> u8 {
        self.total_num_steps
    }

    pub(crate) fn total_num_steps_signed(&self) -> i8 {
        self.total_num_steps as i8
    }

    pub(crate) fn cur_vertex_idx0(&self) -> u8 {
        self.current_vertex_index0
    }

    pub(crate) fn cur_vertex_idx1(&self) -> u8 {
        self.current_vertex_index1
    }

    pub(crate) fn set_left_current(&mut self, x: u8, y: u8) {
        self.x0_current = x;
        self.y0_current = y;
    }

    pub(crate) fn set_right_current(&mut self, x: u8, y: u8) {
        self.x1_current = x;
        self.y1_current = y;
    }

    pub(crate) fn set_left_target(&mut self, x: u8, y: u8) {
        self.x0_target = x;
        self.y0_trigger = y;
    }

    pub(crate) fn set_right_target(&mut self, x: u8, y: u8) {
        self.x1_target = x;
        self.y1_trigger = y;
    }

    pub(crate) fn set_left_current_x(&mut self, x: u8) {
        self.x0_current = x;
    }

    pub(crate) fn set_right_current_x(&mut self, x: u8) {
        self.x1_current = x;
    }

    pub(crate) fn set_total_num_steps(&mut self, value: u8) {
        self.total_num_steps = value;
    }

    pub(crate) fn decrement_total_num_steps(&mut self) -> i8 {
        self.total_num_steps = self.total_num_steps.wrapping_sub(1);
        self.total_num_steps as i8
    }

    pub(crate) fn set_both_cur_vertex_idx(&mut self, value: u8) {
        self.current_vertex_index0 = value;
        self.current_vertex_index1 = value;
    }

    pub(crate) fn set_cur_vertex_idx0(&mut self, value: u8) {
        self.current_vertex_index0 = value;
    }

    pub(crate) fn set_cur_vertex_idx1(&mut self, value: u8) {
        self.current_vertex_index1 = value;
    }

    pub(crate) fn increment_y0_cur(&mut self) {
        self.y0_current = self.y0_current.wrapping_add(1);
    }

    pub(crate) fn increment_y1_cur(&mut self) {
        self.y1_current = self.y1_current.wrapping_add(1);
    }
}

pub(crate) struct NativePolyRasterEdgeBridgeMut<'a> {
    state: &'a mut PolyRasterEdgeState,
    ram: &'a mut [u8],
}

impl<'a> NativePolyRasterEdgeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PolyRasterEdgeState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        debug_assert_eq!(*self.state, PolyRasterEdgeState::load_from_ram(self.ram));
    }

    pub(crate) fn set_left_current(&mut self, x: u8, y: u8) {
        self.state.set_left_current(x, y);
        self.sync();
    }

    pub(crate) fn set_right_current(&mut self, x: u8, y: u8) {
        self.state.set_right_current(x, y);
        self.sync();
    }

    pub(crate) fn set_left_target(&mut self, x: u8, y: u8) {
        self.state.set_left_target(x, y);
        self.sync();
    }

    pub(crate) fn set_right_target(&mut self, x: u8, y: u8) {
        self.state.set_right_target(x, y);
        self.sync();
    }

    pub(crate) fn set_left_current_x(&mut self, x: u8) {
        self.state.set_left_current_x(x);
        self.sync();
    }

    pub(crate) fn set_right_current_x(&mut self, x: u8) {
        self.state.set_right_current_x(x);
        self.sync();
    }

    pub(crate) fn set_total_num_steps(&mut self, value: u8) {
        self.state.set_total_num_steps(value);
        self.sync();
    }

    pub(crate) fn decrement_total_num_steps(&mut self) -> i8 {
        let value = self.state.decrement_total_num_steps();
        self.sync();
        value
    }

    pub(crate) fn set_both_cur_vertex_idx(&mut self, value: u8) {
        self.state.set_both_cur_vertex_idx(value);
        self.sync();
    }

    pub(crate) fn set_cur_vertex_idx0(&mut self, value: u8) {
        self.state.set_cur_vertex_idx0(value);
        self.sync();
    }

    pub(crate) fn set_cur_vertex_idx1(&mut self, value: u8) {
        self.state.set_cur_vertex_idx1(value);
        self.sync();
    }

    pub(crate) fn increment_y0_cur(&mut self) {
        self.state.increment_y0_cur();
        self.sync();
    }

    pub(crate) fn increment_y1_cur(&mut self) {
        self.state.increment_y1_cur();
        self.sync();
    }
}
