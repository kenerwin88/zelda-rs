#![allow(non_snake_case)]

use super::*;

const POLY_ROTATION_SIN_COS: [i8; 320] = [
    0, 2, 3, 5, 6, 8, 9, 11, 12, 14, 16, 17, 19, 20, 22, 23, 24, 26, 27, 29, 30, 32, 33, 34, 36,
    37, 38, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 56, 57, 58, 59, 59,
    60, 60, 61, 61, 62, 62, 62, 63, 63, 63, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 63, 63, 63,
    62, 62, 62, 61, 61, 60, 60, 59, 59, 58, 57, 56, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45,
    44, 43, 42, 41, 39, 38, 37, 36, 34, 33, 32, 30, 29, 27, 26, 24, 23, 22, 20, 19, 17, 16, 14, 12,
    11, 9, 8, 6, 5, 3, 2, 0, -2, -3, -5, -6, -8, -9, -11, -12, -14, -16, -17, -19, -20, -22, -23,
    -24, -26, -27, -29, -30, -32, -33, -34, -36, -37, -38, -39, -41, -42, -43, -44, -45, -46, -47,
    -48, -49, -50, -51, -52, -53, -54, -55, -56, -56, -57, -58, -59, -59, -60, -60, -61, -61, -62,
    -62, -62, -63, -63, -63, -64, -64, -64, -64, -64, -64, -64, -64, -64, -64, -64, -63, -63, -63,
    -62, -62, -62, -61, -61, -60, -60, -59, -59, -58, -57, -56, -56, -55, -54, -53, -52, -51, -50,
    -49, -48, -47, -46, -45, -44, -43, -42, -41, -39, -38, -37, -36, -34, -33, -32, -30, -29, -27,
    -26, -24, -23, -22, -20, -19, -17, -16, -14, -12, -11, -9, -8, -6, -5, -3, -2, 0, 2, 3, 5, 6,
    8, 9, 11, 12, 14, 16, 17, 19, 20, 22, 23, 24, 26, 27, 29, 30, 32, 33, 34, 36, 37, 38, 39, 41,
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 56, 57, 58, 59, 59, 60, 60, 61, 61,
    62, 62, 62, 63, 63, 63, 64, 64, 64, 64, 64,
];

#[derive(Clone, Copy)]
struct Vertex3 {
    x: i8,
    y: i8,
    z: i8,
}

const POLY_MODEL_0_VERTICES: [Vertex3; 6] = [
    Vertex3 { x: 0, y: 65, z: 0 },
    Vertex3 { x: 0, y: -65, z: 0 },
    Vertex3 { x: 0, y: 0, z: -40 },
    Vertex3 { x: -40, y: 0, z: 0 },
    Vertex3 { x: 0, y: 0, z: 40 },
    Vertex3 { x: 40, y: 0, z: 0 },
];

const POLY_MODEL_0_FACE_INDICES: [u8; 40] = [
    3, 0, 5, 2, 4, 3, 0, 2, 3, 1, 3, 0, 3, 4, 2, 3, 0, 4, 5, 3, 3, 1, 2, 5, 4, 3, 1, 3, 2, 1, 3, 1,
    4, 3, 2, 3, 1, 5, 4, 3,
];

const POLY_MODEL_1_VERTICES: [Vertex3; 6] = [
    Vertex3 { x: 0, y: 40, z: 10 },
    Vertex3 {
        x: 40,
        y: -40,
        z: 10,
    },
    Vertex3 {
        x: -40,
        y: -40,
        z: 10,
    },
    Vertex3 {
        x: 0,
        y: 40,
        z: -10,
    },
    Vertex3 {
        x: -40,
        y: -40,
        z: -10,
    },
    Vertex3 {
        x: 40,
        y: -40,
        z: -10,
    },
];

const POLY_MODEL_1_FACE_INDICES: [u8; 28] = [
    3, 0, 1, 2, 7, 3, 3, 4, 5, 6, 4, 0, 3, 5, 1, 5, 4, 1, 5, 4, 2, 4, 4, 3, 0, 2, 4, 3,
];

struct PolyConfig {
    num_vtx: u8,
    num_poly: u8,
    vtx_val: u16,
    polys_val: u16,
    vertex: &'static [Vertex3],
    poly: &'static [u8],
}

const POLY_MODEL_CONFIGS: [PolyConfig; 2] = [
    PolyConfig {
        num_vtx: 6,
        num_poly: 8,
        vtx_val: 0xff98,
        polys_val: 0xffaa,
        vertex: &POLY_MODEL_0_VERTICES,
        poly: &POLY_MODEL_0_FACE_INDICES,
    },
    PolyConfig {
        num_vtx: 6,
        num_poly: 5,
        vtx_val: 0xffd2,
        polys_val: 0xffe4,
        vertex: &POLY_MODEL_1_VERTICES,
        poly: &POLY_MODEL_1_FACE_INDICES,
    },
];

const POLY_RASTER_COLORS: [u32; 16] = [
    0x00, 0xff, 0xff00, 0xffff, 0xff0000, 0xff00ff, 0xffff00, 0xffffff, 0xff000000, 0xff0000ff,
    0xff00ff00, 0xff00ffff, 0xffff0000, 0xffff00ff, 0xffffff00, 0xffffffff,
];
const POLY_LEFT_EDGE_MASKS: [u16; 8] = [
    0xffff, 0x7f7f, 0x3f3f, 0x1f1f, 0x0f0f, 0x0707, 0x0303, 0x0101,
];
const POLY_RIGHT_EDGE_MASKS: [u16; 8] = [
    0x8080, 0xc0c0, 0xe0e0, 0xf0f0, 0xf8f8, 0xfcfc, 0xfefe, 0xffff,
];

impl ZeldaState {
    pub(super) fn poly_run_frame(&mut self) {
        self.polyhedral_empty_bit_map_buffer();
        self.polyhedral_set_shape_pointer();
        self.polyhedral_set_rotation_matrix();
        self.polyhedral_operate_rotation();
        self.polyhedral_draw_polyhedron();
    }

    pub(super) fn polyhedral_empty_bit_map_buffer(&mut self) {
        self.poly_runtime_mut().clear_poly_buffer();
    }

    pub(super) fn polyhedral_set_shape_pointer(&mut self) {
        let shape_depth_bias = (self.poly_runtime().config1() as u16)
            .wrapping_mul(2)
            .wrapping_add(0x80);
        self.poly_runtime_mut()
            .set_shape_depth_bias(shape_depth_bias);
        let model_offset = (self.poly_runtime().model() as u16).wrapping_mul(2);
        self.poly_runtime_mut().set_tmp0_word(model_offset);

        let config = self.poly_config();
        self.poly_runtime_mut().set_num_vertices(config.num_vtx);
        self.poly_runtime_mut().set_num_polys(config.num_poly);
        self.poly_runtime_mut()
            .set_vertex_table_pointer(config.vtx_val);
        self.poly_runtime_mut()
            .set_face_table_pointer(config.polys_val);
    }

    pub(super) fn polyhedral_set_rotation_matrix(&mut self) {
        let angle_a = self.poly_runtime().angle_a() as usize;
        let sin_a = POLY_ROTATION_SIN_COS[angle_a];
        let cos_a = POLY_ROTATION_SIN_COS[angle_a + 64];
        let angle_b = self.poly_runtime().angle_b() as usize;
        let sin_b = POLY_ROTATION_SIN_COS[angle_b];
        let cos_b = POLY_ROTATION_SIN_COS[angle_b + 64];

        self.poly_runtime_mut().set_rotation_sin_a(sin_a as i16);
        self.poly_runtime_mut().set_rotation_cos_a(cos_a as i16);
        self.poly_runtime_mut().set_rotation_sin_b(sin_b as i16);
        self.poly_runtime_mut().set_rotation_cos_b(cos_b as i16);
        self.poly_runtime_mut().set_rotation_matrix_terms(
            (((sin_b as i32 * sin_a as i32) >> 8) << 2) as i16,
            (((cos_b as i32 * cos_a as i32) >> 8) << 2) as i16,
            (((cos_b as i32 * sin_a as i32) >> 8) << 2) as i16,
            (((sin_b as i32 * cos_a as i32) >> 8) << 2) as i16,
        );
    }

    pub(super) fn polyhedral_operate_rotation(&mut self) {
        let vertex = self.poly_config().vertex;
        let mut i = self.poly_runtime().num_vertices() as usize;
        while i != 0 {
            i -= 1;
            let src = vertex[i];
            self.poly_runtime_mut()
                .set_fromlut_position(src.z as u8, src.y as u8, src.x as u8);
            self.polyhedral_rotate_point();
            self.polyhedral_project_point();
            let f0 = self.poly_runtime().f0();
            let f1 = self.poly_runtime().f1();
            let x = self.poly_runtime().base_x().wrapping_add(f0);
            let y = self.poly_runtime().base_y().wrapping_sub(f1);
            self.poly_projected_vertex_mut().set_position(i, x, y);
        }
    }

    pub(super) fn polyhedral_rotate_point(&mut self) {
        let x = self.poly_runtime().fromlut_x() as i32;
        let y = self.poly_runtime().fromlut_y() as i32;
        let z = self.poly_runtime().fromlut_z() as i32;
        let poly_state = self.poly_runtime();
        let sin_a = poly_state.rotation_sin_a() as i32;
        let cos_a = poly_state.rotation_cos_a() as i32;
        let sin_b = poly_state.rotation_sin_b() as i32;
        let cos_b = poly_state.rotation_cos_b() as i32;
        let e0 = poly_state.rotation_e0() as i32;
        let e1 = poly_state.rotation_e1() as i32;
        let e2 = poly_state.rotation_e2() as i32;
        let e3 = poly_state.rotation_e3() as i32;
        let shape_depth_bias = self.poly_runtime().shape_depth_bias() as i32;

        self.poly_runtime_mut()
            .set_f0_word((cos_b * z - sin_b * x) as i16 as u16);
        self.poly_runtime_mut()
            .set_f1_word((e0 * z + cos_a * y + e2 * x) as i16 as u16);
        self.poly_runtime_mut().set_f2_word(
            (((e3 * z) >> 8) - ((sin_a * y) >> 8) + ((e1 * x) >> 8) + shape_depth_bias) as i16
                as u16,
        );
    }

    pub(super) fn polyhedral_project_point(&mut self) {
        let f0 = self.poly_divide(self.poly_runtime().f0_word(), self.poly_runtime().f2_word());
        let f1 = self.poly_divide(self.poly_runtime().f1_word(), self.poly_runtime().f2_word());
        self.poly_runtime_mut().set_f0_word(f0);
        self.poly_runtime_mut().set_f1_word(f1);
    }

    pub(super) fn poly_divide(&mut self, a: u16, b: u16) -> u16 {
        let mut tmp1 = if (a as i16) < 0 { a.wrapping_neg() } else { a };
        let mut tmp0 = b;
        self.poly_runtime_mut().set_tmp1_word(tmp1);
        self.poly_runtime_mut().set_tmp0_word(tmp0);
        while tmp0 >= 256 {
            tmp0 >>= 1;
            tmp1 >>= 1;
        }
        self.poly_runtime_mut().set_tmp1_word(tmp1);
        self.poly_runtime_mut().set_tmp0_word(tmp0);
        let q = tmp1 / tmp0;
        if (a as i16) < 0 {
            q.wrapping_neg()
        } else {
            q
        }
    }

    pub(super) fn polyhedral_draw_polyhedron(&mut self) {
        let poly = self.poly_config().poly;
        let mut src = 0usize;
        loop {
            let n = poly[src];
            src += 1;
            self.poly_runtime_mut().set_num_vertex_in_poly(n);
            self.poly_runtime_mut().set_tmp0(n);
            self.poly_face_coords_mut()
                .set_xy_coords_count(n.wrapping_mul(2));

            let mut i = 1usize;
            loop {
                let j = poly[src] as usize;
                src += 1;
                let projected = self.poly_projected_vertex();
                let x = projected.x(j);
                let y = projected.y(j);
                self.poly_face_coords_mut().set_coord(i, x);
                self.poly_face_coords_mut().set_coord(i + 1, y);
                i += 2;
                if self.poly_runtime_mut().decrement_tmp0() == 0 {
                    break;
                }
            }

            self.poly_runtime_mut().set_raster_color_config(poly[src]);
            src += 1;
            if self.polyhedral_calculate_cross_product() > 0 {
                self.polyhedral_set_foreground_color();
                self.polyhedral_draw_face();
            }

            if self.poly_runtime_mut().decrement_num_polys() == 0 {
                break;
            }
        }
    }

    pub(super) fn polyhedral_calculate_cross_product(&mut self) -> i16 {
        let face = self.poly_face_coords();
        let mut a = (face.coord(3) as i16).wrapping_sub(face.coord(1) as i16);
        let mut tmp0 = (a as i32)
            .wrapping_mul((face.coord(6).wrapping_sub(face.coord(4)) as i8) as i32)
            as u16;
        a = (face.coord(5) as i16).wrapping_sub(face.coord(3) as i16);
        tmp0 = tmp0.wrapping_sub(
            (a as i32).wrapping_mul((face.coord(4).wrapping_sub(face.coord(2)) as i8) as i32)
                as u16,
        );
        self.poly_runtime_mut().set_tmp0_word(tmp0);
        tmp0 as i16
    }

    pub(super) fn polyhedral_set_foreground_color(&mut self) {
        let poly_state = self.poly_runtime();
        let t = if poly_state.model() != 0 {
            poly_state.config1() >> 5
        } else {
            0
        };
        let a = (((self.poly_runtime().tmp0_word() as u32) << (t + 1)) >> 8) as u8;
        let color = if a <= 1 {
            1
        } else if a >= 7 {
            7
        } else {
            a as usize
        };
        self.polyhedral_set_color_mask(color);
    }

    pub(super) fn polyhedral_set_color_mask(&mut self, color: usize) {
        let value = POLY_RASTER_COLORS[color];
        self.poly_runtime_mut()
            .set_raster_colors(value as u16, (value >> 16) as u16);
    }

    pub(super) fn polyhedral_draw_face(&mut self) {
        let mut n = self.poly_face_coords().xy_coords_count() as usize;
        let mut min_y = self.poly_face_coords().coord(n);
        let mut min_idx = n;
        loop {
            n = n.wrapping_sub(2);
            if n == 0 {
                break;
            }
            let y = self.poly_face_coords().coord(n);
            if y < min_y {
                min_y = y;
                min_idx = n;
            }
        }

        let raster_dst_ptr = POLYHEDRAL_BUFFER as u16
            + (((min_y & 0x38) ^ if min_y & 0x20 != 0 { 0x24 } else { 0 }) as u16) * 64
            + ((min_y & 7) as u16) * 2;
        self.poly_runtime_mut().set_raster_dst_ptr(raster_dst_ptr);
        let total_steps = self.poly_face_coords().xy_coords_count() >> 1;
        self.poly_raster_edge_mut()
            .set_both_cur_vertex_idx(min_idx as u8);
        self.poly_raster_edge_mut().set_total_num_steps(total_steps);
        let min_x = self.poly_face_coords().coord(min_idx - 1);
        self.poly_raster_edge_mut().set_left_current(min_x, min_y);
        self.poly_raster_edge_mut().set_right_current(min_x, min_y);
        if self.polyhedral_set_left() || self.polyhedral_set_right() {
            return;
        }

        loop {
            self.polyhedral_fill_line();
            let mut raster_dst_ptr = self.poly_runtime().raster_dst_ptr();
            if raster_dst_ptr as u8 != 0x0e {
                raster_dst_ptr = raster_dst_ptr.wrapping_add(2);
            } else {
                let a = (raster_dst_ptr >> 8).wrapping_add(2) as u8;
                raster_dst_ptr = ((a ^ if a & 8 != 0 { 0 } else { 0x19 }) as u16) << 8;
            }
            self.poly_runtime_mut().set_raster_dst_ptr(raster_dst_ptr);

            let edge = self.poly_raster_edge();
            if edge.y0_cur() == edge.y0_trigger() {
                let x0_target = self.poly_raster_edge().x0_target();
                self.poly_raster_edge_mut().set_left_current_x(x0_target);
                if self.polyhedral_set_left() {
                    return;
                }
            }
            self.poly_raster_edge_mut().increment_y0_cur();
            let edge = self.poly_raster_edge();
            if edge.y1_cur() == edge.y1_trigger() {
                let x1_target = self.poly_raster_edge().x1_target();
                self.poly_raster_edge_mut().set_right_current_x(x1_target);
                if self.polyhedral_set_right() {
                    return;
                }
            }
            self.poly_raster_edge_mut().increment_y1_cur();
            self.poly_runtime_mut().add_x0_step_to_fraction();
            self.poly_runtime_mut().add_x1_step_to_fraction();
        }
    }

    pub(super) fn polyhedral_fill_line(&mut self) {
        let left = POLY_LEFT_EDGE_MASKS[((self.poly_runtime().x0_fraction() >> 8) & 7) as usize];
        let right = POLY_RIGHT_EDGE_MASKS[((self.poly_runtime().x1_fraction() >> 8) & 7) as usize];
        let tmp2 = ((self.poly_runtime().x0_fraction() >> 8) & 0x38) as u8;
        self.poly_runtime_mut().set_tmp2(tmp2);
        let mut d0 = ((self.poly_runtime().x1_fraction() >> 8) & 0x38) as i32;
        let mut ptr = self.poly_runtime().raster_dst_ptr() as usize + d0 as usize * 4;
        d0 -= self.poly_runtime().tmp2() as i32;
        if d0 == 0 {
            let mask = left & right;
            self.poly_runtime_mut().set_tmp1_word(mask);
            self.blend_poly_word(ptr, self.poly_runtime().raster_color0(), mask);
            self.blend_poly_word(ptr + 16, self.poly_runtime().raster_color1(), mask);
            return;
        }
        if d0 < 0 {
            return;
        }

        let mut n = d0 >> 3;
        self.blend_poly_word(ptr, self.poly_runtime().raster_color0(), right);
        self.blend_poly_word(ptr + 16, self.poly_runtime().raster_color1(), right);
        ptr = ptr.wrapping_sub(0x20);
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let color0 = self.poly_runtime().raster_color0();
            let color1 = self.poly_runtime().raster_color1();
            self.poly_runtime_mut().set_bitmap_word(ptr, color0);
            self.poly_runtime_mut().set_bitmap_word(ptr + 16, color1);
            ptr = ptr.wrapping_sub(0x20);
        }
        self.blend_poly_word(ptr, self.poly_runtime().raster_color0(), left);
        self.blend_poly_word(ptr + 16, self.poly_runtime().raster_color1(), left);
        self.poly_runtime_mut().set_tmp1_word(left);
        self.poly_runtime_mut().clear_raster_full_word_count();
    }

    pub(super) fn polyhedral_set_left(&mut self) -> bool {
        loop {
            if self.poly_raster_edge_mut().decrement_total_num_steps() < 0 {
                return true;
            }
            let mut i = self.poly_raster_edge().cur_vertex_idx0() as i32 - 2;
            if i == 0 {
                i = self.poly_face_coords().xy_coords_count() as i32;
            }
            let i = i as usize;
            let y = self.poly_face_coords().coord(i);
            let x = self.poly_face_coords().coord(i - 1);
            let y0_cur = self.poly_raster_edge().y0_cur();
            if y < y0_cur {
                return true;
            }
            if y != y0_cur {
                self.poly_raster_edge_mut().set_left_target(x, y);
                self.poly_raster_edge_mut().set_cur_vertex_idx0(i as u8);
                let edge = self.poly_raster_edge();
                let x0_cur = edge.x0_cur();
                let y0_cur = edge.y0_cur();
                let y0_trigger = edge.y0_trigger();
                let u = edge.x0_target() as i32 - x0_cur as i32;
                let t = (((u.abs() & 0xff) << 8) / y0_trigger.wrapping_sub(y0_cur) as i32) as u16;
                self.poly_runtime_mut()
                    .set_x0_fraction(((x0_cur as u16) << 8) | 0x80);
                self.poly_runtime_mut()
                    .set_x0_step(if u < 0 { t.wrapping_neg() } else { t });
                return false;
            }
            self.poly_raster_edge_mut().set_left_current_x(x);
            self.poly_raster_edge_mut().set_cur_vertex_idx0(i as u8);
        }
    }

    pub(super) fn polyhedral_set_right(&mut self) -> bool {
        loop {
            if self.poly_raster_edge_mut().decrement_total_num_steps() < 0 {
                return true;
            }
            let mut i = self.poly_raster_edge().cur_vertex_idx1() as usize;
            if i == self.poly_face_coords().xy_coords_count() as usize {
                i = 0;
            }
            i += 2;
            let y = self.poly_face_coords().coord(i);
            let x = self.poly_face_coords().coord(i - 1);
            let y1_cur = self.poly_raster_edge().y1_cur();
            if y < y1_cur {
                return true;
            }
            if y != y1_cur {
                self.poly_raster_edge_mut().set_right_target(x, y);
                self.poly_raster_edge_mut().set_cur_vertex_idx1(i as u8);
                let edge = self.poly_raster_edge();
                let x1_cur = edge.x1_cur();
                let y1_cur = edge.y1_cur();
                let y1_trigger = edge.y1_trigger();
                let u = edge.x1_target() as i32 - x1_cur as i32;
                let t = (((u.abs() & 0xff) << 8) / y1_trigger.wrapping_sub(y1_cur) as i32) as u16;
                self.poly_runtime_mut()
                    .set_x1_fraction(((x1_cur as u16) << 8) | 0x80);
                self.poly_runtime_mut()
                    .set_x1_step(if u < 0 { t.wrapping_neg() } else { t });
                return false;
            }
            self.poly_raster_edge_mut().set_right_current_x(x);
            self.poly_raster_edge_mut().set_cur_vertex_idx1(i as u8);
        }
    }

    fn blend_poly_word(&mut self, offset: usize, color: u16, mask: u16) {
        self.poly_runtime_mut()
            .blend_bitmap_word(offset, color, mask);
    }

    fn poly_config(&self) -> &'static PolyConfig {
        &POLY_MODEL_CONFIGS[self.poly_runtime().model() as usize]
    }
}
