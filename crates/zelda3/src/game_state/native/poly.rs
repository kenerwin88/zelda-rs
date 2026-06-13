use crate::game_state::constants::{
    POLY_CUR_VERTEX_IDX0, POLY_CUR_VERTEX_IDX1, POLY_FACE_COORDS, POLY_PROJECTED_X,
    POLY_PROJECTED_Y, POLY_TOTAL_NUM_STEPS, POLY_X0_CUR, POLY_X0_TARGET, POLY_X1_CUR,
    POLY_X1_TARGET, POLY_Y0_CUR, POLY_Y0_TRIG, POLY_Y1_CUR, POLY_Y1_TRIG,
};

const POLY_PROJECTED_VERTEX_COUNT: usize = 40;
const POLY_FACE_COORD_COUNT: usize = 0x20;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolyState {
    pub(crate) projected_vertices: PolyProjectedVerticesState,
    pub(crate) face_coords: PolyFaceCoordsState,
    pub(crate) raster_edge: PolyRasterEdgeState,
}

impl PolyState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            projected_vertices: PolyProjectedVerticesState::load_from_ram(ram),
            face_coords: PolyFaceCoordsState::load_from_ram(ram),
            raster_edge: PolyRasterEdgeState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.projected_vertices.write_to_ram(ram);
        self.face_coords.write_to_ram(ram);
        self.raster_edge.write_to_ram(ram);
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
        *state = PolyProjectedVerticesState::load_from_ram(ram);
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
        *state = PolyFaceCoordsState::load_from_ram(ram);
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
        *state = PolyRasterEdgeState::load_from_ram(ram);
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
