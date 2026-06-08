#![allow(non_snake_case)]

use super::*;

const K_POLY_SIN_COS: [i8; 320] = [
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

const K_POLY0_VTX: [Vertex3; 6] = [
    Vertex3 { x: 0, y: 65, z: 0 },
    Vertex3 { x: 0, y: -65, z: 0 },
    Vertex3 { x: 0, y: 0, z: -40 },
    Vertex3 { x: -40, y: 0, z: 0 },
    Vertex3 { x: 0, y: 0, z: 40 },
    Vertex3 { x: 40, y: 0, z: 0 },
];

const K_POLY0_POLYS: [u8; 40] = [
    3, 0, 5, 2, 4, 3, 0, 2, 3, 1, 3, 0, 3, 4, 2, 3, 0, 4, 5, 3, 3, 1, 2, 5, 4, 3, 1, 3, 2, 1, 3, 1,
    4, 3, 2, 3, 1, 5, 4, 3,
];

const K_POLY1_VTX: [Vertex3; 6] = [
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

const K_POLY1_POLYS: [u8; 28] = [
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

const K_POLY_CONFIGS: [PolyConfig; 2] = [
    PolyConfig {
        num_vtx: 6,
        num_poly: 8,
        vtx_val: 0xff98,
        polys_val: 0xffaa,
        vertex: &K_POLY0_VTX,
        poly: &K_POLY0_POLYS,
    },
    PolyConfig {
        num_vtx: 6,
        num_poly: 5,
        vtx_val: 0xffd2,
        polys_val: 0xffe4,
        vertex: &K_POLY1_VTX,
        poly: &K_POLY1_POLYS,
    },
];

const K_POLY_RASTER_COLORS: [u32; 16] = [
    0x00, 0xff, 0xff00, 0xffff, 0xff0000, 0xff00ff, 0xffff00, 0xffffff, 0xff000000, 0xff0000ff,
    0xff00ff00, 0xff00ffff, 0xffff0000, 0xffff00ff, 0xffffff00, 0xffffffff,
];
const K_POLY_LEFT_SIDE_MASK: [u16; 8] = [
    0xffff, 0x7f7f, 0x3f3f, 0x1f1f, 0x0f0f, 0x0707, 0x0303, 0x0101,
];
const K_POLY_RIGHT_SIDE_MASK: [u16; 8] = [
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
        self.ram[POLYHEDRAL_BUFFER..POLYHEDRAL_BUFFER + 0x800].fill(0);
    }

    pub(super) fn polyhedral_set_shape_pointer(&mut self) {
        self.write_u16_ram(
            POLY_VAR1,
            (self.ram[POLY_CONFIG1] as u16)
                .wrapping_mul(2)
                .wrapping_add(0x80),
        );
        self.write_u16_ram(
            POLY_TMP0,
            (self.ram[POLY_WHICH_MODEL] as u16).wrapping_mul(2),
        );

        let config = self.poly_config();
        self.ram[POLY_CONFIG_NUM_VERTEX] = config.num_vtx;
        self.ram[POLY_CONFIG_NUM_POLYS] = config.num_poly;
        self.write_u16_ram(POLY_FROMLUT_PTR2, config.vtx_val);
        self.write_u16_ram(POLY_FROMLUT_PTR4, config.polys_val);
    }

    pub(super) fn polyhedral_set_rotation_matrix(&mut self) {
        let sin_a = K_POLY_SIN_COS[self.ram[POLY_A] as usize];
        let cos_a = K_POLY_SIN_COS[self.ram[POLY_A] as usize + 64];
        let sin_b = K_POLY_SIN_COS[self.ram[POLY_B] as usize];
        let cos_b = K_POLY_SIN_COS[self.ram[POLY_B] as usize + 64];

        self.write_i16_ram(POLY_SIN_A, sin_a as i16);
        self.write_i16_ram(POLY_COS_A, cos_a as i16);
        self.write_i16_ram(POLY_SIN_B, sin_b as i16);
        self.write_i16_ram(POLY_COS_B, cos_b as i16);
        self.write_i16_ram(POLY_E0, (((sin_b as i32 * sin_a as i32) >> 8) << 2) as i16);
        self.write_i16_ram(POLY_E1, (((cos_b as i32 * cos_a as i32) >> 8) << 2) as i16);
        self.write_i16_ram(POLY_E2, (((cos_b as i32 * sin_a as i32) >> 8) << 2) as i16);
        self.write_i16_ram(POLY_E3, (((sin_b as i32 * cos_a as i32) >> 8) << 2) as i16);
    }

    pub(super) fn polyhedral_operate_rotation(&mut self) {
        let vertex = self.poly_config().vertex;
        let mut i = self.ram[POLY_CONFIG_NUM_VERTEX] as usize;
        while i != 0 {
            i -= 1;
            let src = vertex[i];
            self.ram[POLY_FROMLUT_X] = src.z as u8;
            self.ram[POLY_FROMLUT_Y] = src.y as u8;
            self.ram[POLY_FROMLUT_Z] = src.x as u8;
            self.polyhedral_rotate_point();
            self.polyhedral_project_point();
            self.ram[POLY_ARR_X + i] = self.ram[POLY_BASE_X].wrapping_add(self.ram[POLY_F0]);
            self.ram[POLY_ARR_Y + i] = self.ram[POLY_BASE_Y].wrapping_sub(self.ram[POLY_F1]);
        }
    }

    pub(super) fn polyhedral_rotate_point(&mut self) {
        let x = self.ram[POLY_FROMLUT_X] as i8 as i32;
        let y = self.ram[POLY_FROMLUT_Y] as i8 as i32;
        let z = self.ram[POLY_FROMLUT_Z] as i8 as i32;
        let sin_a = self.read_i16_ram(POLY_SIN_A) as i32;
        let cos_a = self.read_i16_ram(POLY_COS_A) as i32;
        let sin_b = self.read_i16_ram(POLY_SIN_B) as i32;
        let cos_b = self.read_i16_ram(POLY_COS_B) as i32;
        let e0 = self.read_i16_ram(POLY_E0) as i32;
        let e1 = self.read_i16_ram(POLY_E1) as i32;
        let e2 = self.read_i16_ram(POLY_E2) as i32;
        let e3 = self.read_i16_ram(POLY_E3) as i32;
        let var1 = self.read_u16_ram(POLY_VAR1) as i32;

        self.write_i16_ram(POLY_F0, (cos_b * z - sin_b * x) as i16);
        self.write_i16_ram(POLY_F1, (e0 * z + cos_a * y + e2 * x) as i16);
        self.write_i16_ram(
            POLY_F2,
            (((e3 * z) >> 8) - ((sin_a * y) >> 8) + ((e1 * x) >> 8) + var1) as i16,
        );
    }

    pub(super) fn polyhedral_project_point(&mut self) {
        let f0 = self.poly_divide(self.read_u16_ram(POLY_F0), self.read_u16_ram(POLY_F2));
        let f1 = self.poly_divide(self.read_u16_ram(POLY_F1), self.read_u16_ram(POLY_F2));
        self.write_u16_ram(POLY_F0, f0);
        self.write_u16_ram(POLY_F1, f1);
    }

    pub(super) fn poly_divide(&mut self, a: u16, b: u16) -> u16 {
        let mut tmp1 = if (a as i16) < 0 { a.wrapping_neg() } else { a };
        let mut tmp0 = b;
        self.write_u16_ram(POLY_TMP1, tmp1);
        self.write_u16_ram(POLY_TMP0, tmp0);
        while tmp0 >= 256 {
            tmp0 >>= 1;
            tmp1 >>= 1;
        }
        self.write_u16_ram(POLY_TMP1, tmp1);
        self.write_u16_ram(POLY_TMP0, tmp0);
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
            self.ram[POLY_NUM_VERTEX_IN_POLY] = poly[src];
            src += 1;
            self.ram[POLY_TMP0] = self.ram[POLY_NUM_VERTEX_IN_POLY];
            self.ram[POLY_XY_COORDS] = self.ram[POLY_NUM_VERTEX_IN_POLY].wrapping_mul(2);

            let mut i = 1usize;
            loop {
                let j = poly[src] as usize;
                src += 1;
                self.ram[POLY_XY_COORDS + i] = self.ram[POLY_ARR_X + j];
                self.ram[POLY_XY_COORDS + i + 1] = self.ram[POLY_ARR_Y + j];
                i += 2;
                self.ram[POLY_TMP0] = self.ram[POLY_TMP0].wrapping_sub(1);
                if self.ram[POLY_TMP0] == 0 {
                    break;
                }
            }

            self.ram[POLY_RASTER_COLOR_CONFIG] = poly[src];
            src += 1;
            if self.polyhedral_calculate_cross_product() > 0 {
                self.polyhedral_set_foreground_color();
                self.polyhedral_draw_face();
            }

            self.ram[POLY_CONFIG_NUM_POLYS] = self.ram[POLY_CONFIG_NUM_POLYS].wrapping_sub(1);
            if self.ram[POLY_CONFIG_NUM_POLYS] == 0 {
                break;
            }
        }
    }

    pub(super) fn polyhedral_calculate_cross_product(&mut self) -> i16 {
        let mut a =
            (self.ram[POLY_XY_COORDS + 3] as i16).wrapping_sub(self.ram[POLY_XY_COORDS + 1] as i16);
        let mut tmp0 = (a as i32).wrapping_mul(
            (self.ram[POLY_XY_COORDS + 6].wrapping_sub(self.ram[POLY_XY_COORDS + 4]) as i8) as i32,
        ) as u16;
        a = (self.ram[POLY_XY_COORDS + 5] as i16).wrapping_sub(self.ram[POLY_XY_COORDS + 3] as i16);
        tmp0 = tmp0.wrapping_sub((a as i32).wrapping_mul(
            (self.ram[POLY_XY_COORDS + 4].wrapping_sub(self.ram[POLY_XY_COORDS + 2]) as i8) as i32,
        ) as u16);
        self.write_u16_ram(POLY_TMP0, tmp0);
        tmp0 as i16
    }

    pub(super) fn polyhedral_set_foreground_color(&mut self) {
        let t = if self.ram[POLY_WHICH_MODEL] != 0 {
            self.ram[POLY_CONFIG1] >> 5
        } else {
            0
        };
        let a = (((self.read_u16_ram(POLY_TMP0) as u32) << (t + 1)) >> 8) as u8;
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
        let value = K_POLY_RASTER_COLORS[color];
        self.write_u16_ram(POLY_RASTER_COLOR0, value as u16);
        self.write_u16_ram(POLY_RASTER_COLOR1, (value >> 16) as u16);
    }

    pub(super) fn polyhedral_draw_face(&mut self) {
        let mut n = self.ram[POLY_XY_COORDS] as usize;
        let mut min_y = self.ram[POLY_XY_COORDS + n];
        let mut min_idx = n;
        loop {
            n = n.wrapping_sub(2);
            if n == 0 {
                break;
            }
            if self.ram[POLY_XY_COORDS + n] < min_y {
                min_y = self.ram[POLY_XY_COORDS + n];
                min_idx = n;
            }
        }

        let raster_dst_ptr = POLYHEDRAL_BUFFER as u16
            + (((min_y & 0x38) ^ if min_y & 0x20 != 0 { 0x24 } else { 0 }) as u16) * 64
            + ((min_y & 7) as u16) * 2;
        self.write_u16_ram(POLY_RASTER_DST_PTR, raster_dst_ptr);
        self.ram[POLY_CUR_VERTEX_IDX0] = min_idx as u8;
        self.ram[POLY_CUR_VERTEX_IDX1] = min_idx as u8;
        self.ram[POLY_TOTAL_NUM_STEPS] = self.ram[POLY_XY_COORDS] >> 1;
        self.ram[POLY_Y0_CUR] = self.ram[POLY_XY_COORDS + min_idx];
        self.ram[POLY_Y1_CUR] = self.ram[POLY_XY_COORDS + min_idx];
        self.ram[POLY_X0_CUR] = self.ram[POLY_XY_COORDS + min_idx - 1];
        self.ram[POLY_X1_CUR] = self.ram[POLY_XY_COORDS + min_idx - 1];
        if self.polyhedral_set_left() || self.polyhedral_set_right() {
            return;
        }

        loop {
            self.polyhedral_fill_line();
            let mut raster_dst_ptr = self.read_u16_ram(POLY_RASTER_DST_PTR);
            if raster_dst_ptr as u8 != 0x0e {
                raster_dst_ptr = raster_dst_ptr.wrapping_add(2);
            } else {
                let a = (raster_dst_ptr >> 8).wrapping_add(2) as u8;
                raster_dst_ptr = ((a ^ if a & 8 != 0 { 0 } else { 0x19 }) as u16) << 8;
            }
            self.write_u16_ram(POLY_RASTER_DST_PTR, raster_dst_ptr);

            if self.ram[POLY_Y0_CUR] == self.ram[POLY_Y0_TRIG] {
                self.ram[POLY_X0_CUR] = self.ram[POLY_X0_TARGET];
                if self.polyhedral_set_left() {
                    return;
                }
            }
            self.ram[POLY_Y0_CUR] = self.ram[POLY_Y0_CUR].wrapping_add(1);
            if self.ram[POLY_Y1_CUR] == self.ram[POLY_Y1_TRIG] {
                self.ram[POLY_X1_CUR] = self.ram[POLY_X1_TARGET];
                if self.polyhedral_set_right() {
                    return;
                }
            }
            self.ram[POLY_Y1_CUR] = self.ram[POLY_Y1_CUR].wrapping_add(1);
            self.write_u16_ram(
                POLY_X0_FRAC,
                self.read_u16_ram(POLY_X0_FRAC)
                    .wrapping_add(self.read_u16_ram(POLY_X0_STEP)),
            );
            self.write_u16_ram(
                POLY_X1_FRAC,
                self.read_u16_ram(POLY_X1_FRAC)
                    .wrapping_add(self.read_u16_ram(POLY_X1_STEP)),
            );
        }
    }

    pub(super) fn polyhedral_fill_line(&mut self) {
        let left = K_POLY_LEFT_SIDE_MASK[((self.read_u16_ram(POLY_X0_FRAC) >> 8) & 7) as usize];
        let right = K_POLY_RIGHT_SIDE_MASK[((self.read_u16_ram(POLY_X1_FRAC) >> 8) & 7) as usize];
        self.ram[POLY_TMP2] = ((self.read_u16_ram(POLY_X0_FRAC) >> 8) & 0x38) as u8;
        let mut d0 = ((self.read_u16_ram(POLY_X1_FRAC) >> 8) & 0x38) as i32;
        let mut ptr = self.read_u16_ram(POLY_RASTER_DST_PTR) as usize + d0 as usize * 4;
        d0 -= self.ram[POLY_TMP2] as i32;
        if d0 == 0 {
            let mask = left & right;
            self.write_u16_ram(POLY_TMP1, mask);
            self.blend_poly_word(ptr, self.read_u16_ram(POLY_RASTER_COLOR0), mask);
            self.blend_poly_word(ptr + 16, self.read_u16_ram(POLY_RASTER_COLOR1), mask);
            return;
        }
        if d0 < 0 {
            return;
        }

        let mut n = d0 >> 3;
        self.blend_poly_word(ptr, self.read_u16_ram(POLY_RASTER_COLOR0), right);
        self.blend_poly_word(ptr + 16, self.read_u16_ram(POLY_RASTER_COLOR1), right);
        ptr = ptr.wrapping_sub(0x20);
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            self.write_u16_ram(ptr, self.read_u16_ram(POLY_RASTER_COLOR0));
            self.write_u16_ram(ptr + 16, self.read_u16_ram(POLY_RASTER_COLOR1));
            ptr = ptr.wrapping_sub(0x20);
        }
        self.blend_poly_word(ptr, self.read_u16_ram(POLY_RASTER_COLOR0), left);
        self.blend_poly_word(ptr + 16, self.read_u16_ram(POLY_RASTER_COLOR1), left);
        self.write_u16_ram(POLY_TMP1, left);
        self.write_u16_ram(POLY_RASTER_NUMFULL, 0);
    }

    pub(super) fn polyhedral_set_left(&mut self) -> bool {
        loop {
            self.ram[POLY_TOTAL_NUM_STEPS] = self.ram[POLY_TOTAL_NUM_STEPS].wrapping_sub(1);
            if (self.ram[POLY_TOTAL_NUM_STEPS] as i8) < 0 {
                return true;
            }
            let mut i = self.ram[POLY_CUR_VERTEX_IDX0] as i32 - 2;
            if i == 0 {
                i = self.ram[POLY_XY_COORDS] as i32;
            }
            let i = i as usize;
            if self.ram[POLY_XY_COORDS + i] < self.ram[POLY_Y0_CUR] {
                return true;
            }
            if self.ram[POLY_XY_COORDS + i] != self.ram[POLY_Y0_CUR] {
                self.ram[POLY_Y0_TRIG] = self.ram[POLY_XY_COORDS + i];
                self.ram[POLY_X0_TARGET] = self.ram[POLY_XY_COORDS + i - 1];
                self.ram[POLY_CUR_VERTEX_IDX0] = i as u8;
                let u = self.ram[POLY_X0_TARGET] as i32 - self.ram[POLY_X0_CUR] as i32;
                let t = (((u.abs() & 0xff) << 8)
                    / self.ram[POLY_Y0_TRIG].wrapping_sub(self.ram[POLY_Y0_CUR]) as i32)
                    as u16;
                self.write_u16_ram(POLY_X0_FRAC, ((self.ram[POLY_X0_CUR] as u16) << 8) | 0x80);
                self.write_u16_ram(POLY_X0_STEP, if u < 0 { t.wrapping_neg() } else { t });
                return false;
            }
            self.ram[POLY_X0_CUR] = self.ram[POLY_XY_COORDS + i - 1];
            self.ram[POLY_CUR_VERTEX_IDX0] = i as u8;
        }
    }

    pub(super) fn polyhedral_set_right(&mut self) -> bool {
        loop {
            self.ram[POLY_TOTAL_NUM_STEPS] = self.ram[POLY_TOTAL_NUM_STEPS].wrapping_sub(1);
            if (self.ram[POLY_TOTAL_NUM_STEPS] as i8) < 0 {
                return true;
            }
            let mut i = self.ram[POLY_CUR_VERTEX_IDX1] as usize;
            if i == self.ram[POLY_XY_COORDS] as usize {
                i = 0;
            }
            i += 2;
            if self.ram[POLY_XY_COORDS + i] < self.ram[POLY_Y1_CUR] {
                return true;
            }
            if self.ram[POLY_XY_COORDS + i] != self.ram[POLY_Y1_CUR] {
                self.ram[POLY_Y1_TRIG] = self.ram[POLY_XY_COORDS + i];
                self.ram[POLY_X1_TARGET] = self.ram[POLY_XY_COORDS + i - 1];
                self.ram[POLY_CUR_VERTEX_IDX1] = i as u8;
                let u = self.ram[POLY_X1_TARGET] as i32 - self.ram[POLY_X1_CUR] as i32;
                let t = (((u.abs() & 0xff) << 8)
                    / self.ram[POLY_Y1_TRIG].wrapping_sub(self.ram[POLY_Y1_CUR]) as i32)
                    as u16;
                self.write_u16_ram(POLY_X1_FRAC, ((self.ram[POLY_X1_CUR] as u16) << 8) | 0x80);
                self.write_u16_ram(POLY_X1_STEP, if u < 0 { t.wrapping_neg() } else { t });
                return false;
            }
            self.ram[POLY_X1_CUR] = self.ram[POLY_XY_COORDS + i - 1];
            self.ram[POLY_CUR_VERTEX_IDX1] = i as u8;
        }
    }

    fn blend_poly_word(&mut self, offset: usize, color: u16, mask: u16) {
        let current = self.read_u16_ram(offset);
        self.write_u16_ram(offset, current ^ ((current ^ color) & mask));
    }

    fn poly_config(&self) -> &'static PolyConfig {
        &K_POLY_CONFIGS[self.ram[POLY_WHICH_MODEL] as usize]
    }
}
