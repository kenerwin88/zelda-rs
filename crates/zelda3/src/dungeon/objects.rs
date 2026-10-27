//! Room object subtype dispatch and object parameter tables.

use super::*;
use crate::game_state::ObjectRecord;

impl ZeldaState {

    pub(super) fn LoadType1ObjectSubtype1(&mut self, idx: u8, width: u8, height: u8, dsto: u16) {
        let Some(src) = object_subtype1_param(idx) else {
            panic!("LoadType1ObjectSubtype1 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 | 0xb8 | 0xb9 => {
                let count = size_1to15_or(width, height, 32);
                if self.lag_free {
                    // $01:8B89/8A92/8C37/8B79: wrapper setup and actual loop
                    // passes; size helper $01:B0AC/B0BE/B0CC includes its RTS.
                    // The drawing child retains its existing independent charge.
                    let size_master = if width == 0 && height == 0 { 206 } else { 188 };
                    crate::cycle_ledger::charge(46 + size_master + 116 * u64::from(count)
                        + 6 * u64::from(count.saturating_sub(1)) + 42);
                }
                for i in 0..count {
                    self.room_draw_rightwards2x2(src, dsto + i * 2);
                }
            }
            0x01 | 0x02 | 0xb6 | 0xb7 => {
                let count = size_1to15_or(width, height, 26);
                if self.lag_free {
                    // $01:8B89/8A92/8C37/8B79: wrapper setup and actual loop
                    // passes; size helper $01:B0AC/B0BE/B0CC includes its RTS.
                    // The drawing child retains its existing independent charge.
                    let size_master = if width == 0 && height == 0 { 206 } else { 188 };
                    crate::cycle_ledger::charge(78 + size_master + 172 * u64::from(count)
                        + 6 * u64::from(count.saturating_sub(1)) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(2, src, dsto + i * 2);
                }
            }
            0x03 | 0x04 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8B0D..8B73: register setup, size child $B0AC,
                    // eight load/two-background-store triples per pass, RTS.
                    // The native both-background tile helper has no price.
                    crate::cycle_ledger::charge(88 + 286 + 1262 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_data_draw_object_nx4_both_bgs(src, dsto + i * 2, 2);
                }
            }
            0x05 | 0x06 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8B89/8A92/8C37/8B79: wrapper setup and actual loop
                    // passes; size helper $01:B0AC/B0BE/B0CC includes its RTS.
                    // The drawing child retains its existing independent charge.
                    let size_master = 286;
                    crate::cycle_ledger::charge(78 + size_master + 238 * u64::from(count)
                        + 6 * u64::from(count.saturating_sub(1)) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(2, src, dsto + i * 6);
                }
            }
            0x07 | 0x08 | 0x53 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8B89/8A92/8C37/8B79: wrapper setup and actual loop
                    // passes; size helper $01:B0AC/B0BE/B0CC includes its RTS.
                    // The drawing child retains its existing independent charge.
                    let size_master = 286;
                    crate::cycle_ledger::charge(68 + size_master + 116 * u64::from(count)
                        + 6 * u64::from(count.saturating_sub(1)) + 42);
                }
                for i in 0..count {
                    self.room_draw_rightwards2x2(src, dsto + i * 2);
                }
            }
            0x09 | 0x0c | 0x0d | 0x10 | 0x11 | 0x14 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                if self.lag_free {
                    // $01:8C58..8C60 -> B2AA; B2A1..B2AE. The initial
                    // DEC/BNE enters the loop; $B279 owns its five stores.
                    // $B0AF's size body is 262 (without $B0AC's LDA).
                    crate::cycle_ledger::charge(94 + 262 + 70 + 6
                        + 182 * u64::from(count) + 6 * u64::from(count - 1) + 42);
                }
                let mut dst = dsto;
                for _ in 0..count {
                    self.room_draw_draw_object2x2and1(src, dst);
                    dst = dst.wrapping_sub(63);
                }
            }
            0x0a | 0x0b | 0x0e | 0x0f | 0x12 | 0x13 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                if self.lag_free {
                    // $01:8C61..8C69 -> B29C; B293..B2A0, the opposite
                    // diagonal's identical control work; $B279 prices stores.
                    crate::cycle_ledger::charge(94 + 262 + 70 + 6
                        + 182 * u64::from(count) + 6 * u64::from(count - 1) + 42);
                }
                let mut dst = dsto;
                for _ in 0..count {
                    self.room_draw_draw_object2x2and1(src, dst);
                    dst = dst.wrapping_add(65);
                }
            }
            0x15 | 0x18 | 0x19 | 0x1c | 0x1d | 0x20 => {
                let count = size_a_to_a_plus_15(width, height, 6);
                if self.lag_free {
                    // $01:8C6A..8CB8: setup/size, five pairs of direct
                    // background stores and the actual diagonal steps.
                    crate::cycle_ledger::charge(168 + 262 + 854 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg1(dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_sub(63);
                }
            }
            0x16 | 0x17 | 0x1a | 0x1b | 0x1e | 0x1f => {
                let count = size_a_to_a_plus_15(width, height, 6);
                if self.lag_free {
                    // $01:8CB9..8CC6 -> 8C76..8CB8: opposite diagonal
                    // setup (158 plus shared STA 32), size and direct stores.
                    crate::cycle_ledger::charge(190 + 262 + 854 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                let mut dst = dsto;
                for _ in 0..count {
                    for y in 0..5 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(src, y as usize));
                        self.room_write_bg1(dst + y * 64, self.tile_word(src, y as usize));
                    }
                    dst = dst.wrapping_add(65);
                }
            }
            0x21 => {
                let mut count = (((width as u16) << 2) | height as u16) * 2 + 1;
                if self.lag_free {
                    // $01:8D5D..8D7D: two cap calls and actual repeated middles;
                    // final LDA falls into the separately priced 1x3 drawer.
                    crate::cycle_ledger::charge(222 + 206 * u64::from(count)
                        + 6 * u64::from(count.saturating_sub(1)) + 24);
                }
                let mut dst = dsto;
                self.room_draw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count != 0 {
                    self.room_draw_1x3_rightwards(src + 6, dst, 1);
                    dst += 1;
                    count -= 1;
                }
                self.room_draw_1x3_rightwards(src + 12, dst, 1);
            }
            0x22 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                let preserve_corner = self.room_read_current(dsto) & 0x03ff == 0x00e2;
                if self.lag_free {
                    // $01:8EF0..8F0B: size/corner calls, actual corner
                    // branch, line call and tail; child bodies excluded.
                    crate::cycle_ledger::charge(156 + if preserve_corner { 6 } else { 102 } + 190);
                    self.price_room_draw_size_ato_aplus15();
                    self.price_room_draw_small_rail_corner(preserve_corner);
                }
                if !preserve_corner {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                self.room_object_fill_row_advance(count, src + 2, dsto + 1);
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x23..=0x2e | 0x3f..=0x46 | 0xb3 | 0xb4 => {
                // $01:893C JMP ($0E), subtype-1 table $8200: IDs $23..$2E
                // select $8F62. The size, corner and row-fill JSRs are children.
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x018f62));
                let b = crate::cycle_ledger::blocks(self.lag_free);
                b.block(0x018f62);
                let count = size_1to16(width, height);
                self.price_room_draw_size_1to16();
                b.block(0x018f65);
                let tile = self.room_read_current(dsto) & 0x03ff;
                self.price_room_draw_small_rail_corner(tile == 0x01db);
                b.block(0x018f6b);
                if tile == 0x01db { b.taken(0x018f6b); }
                else {
                    b.block(0x018f6d);
                    if tile == 0x01a6 { b.taken(0x018f70); }
                    else {
                        b.block(0x018f72);
                        if tile == 0x01dd { b.taken(0x018f75); }
                        else {
                            b.block(0x018f77);
                            if tile == 0x01fc { b.taken(0x018f7a); }
                            else {
                                b.block(0x018f7c);
                                self.room_write_current(dsto, self.tile_word(src, 0));
                            }
                        }
                    }
                }
                b.block(0x018f81);
                self.room_object_fill_row_advance(count, src + 2, dsto + 1);
                b.block(0x018f84);
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x2f => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 1, self.tile_word(src, 2));
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, fill);
                    dst += 1;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 1, self.tile_word(src, 5));
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x30 => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 64) & 0x03ff != 0x00e2 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 2;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, self.tile_word(src, 3));
                    dst += 1;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 64, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x33 | 0xb2 | 0xba => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9111..911F: size/parameter setup, JSR $97ED,
                    // DEC/BNE and RTS; the 4x4 child keeps its own price.
                    crate::cycle_ledger::charge(78 + 286 + 148 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_4x4(src, dsto + i * 4);
                }
            }
            0x34 => {
                let count = size_a_to_a_plus_15(width, height, 4);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i, tile);
                }
            }
            0x36 | 0x37 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:92FB..930D: size/parameter setup and four extra
                    // INYs after each independently priced $97ED call.
                    crate::cycle_ledger::charge(78 + 286 + 204 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_4x4(src, dsto + i * 6);
                }
            }
            0x38 => {
                let statue_src = 0x0e26;
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9323..9337: size, statue source/width, JSR $8D80
                    // and horizontal spacing; child stores stay separate.
                    crate::cycle_ledger::charge(46 + 286 + 220 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_1x3_rightwards(statue_src, dsto + i * 4, 2);
                }
            }
            0x39 | 0x3d => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:936F..9386: size, saved source, width/call,
                    // spacing and loop; $97F0 prices only the tile body.
                    crate::cycle_ledger::charge(78 + 286 + 238 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(2, src, dsto + i * 6);
                }
            }
            0x3a | 0x3b => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9387..939E: width-four $8D80 calls plus actual
                    // spacing/loop control, excluding the priced child.
                    crate::cycle_ledger::charge(78 + 286 + 238 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_1x3_rightwards(src, dsto + i * 8, 4);
                }
            }
            0x3c => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:93B7..93DB: size call, two JSR $B2AF calls,
                    // pointer steps and actual loop branches; children excluded.
                    crate::cycle_ledger::charge(46 + 420 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    let dst = dsto + i * 4;
                    self.room_draw_downwards2x2(src, dst);
                    self.room_draw_downwards2x2(src + 8, dst + 6 * 64);
                }
            }
            0x3e | 0x4b => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9456..9465: size call, JSR $B2AF, spacing,
                    // actual DEC/BNE and RTS; child bodies excluded.
                    crate::cycle_ledger::charge(46 + 168 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    self.room_draw_downwards2x2(src, dsto + i * 14);
                }
            }
            0x47 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.room_draw_draw_object2x2and1(src, dsto) + 1;
                for _ in 0..count {
                    self.room_draw_draw_object2x2and1(src + 10, dst);
                    dst += 1;
                }
                self.room_draw_draw_object2x2and1(src + 20, dst);
            }
            0x48 => {
                let count = size_1to16(width, height) * 2;
                let mut dst = dsto;
                self.room_draw_1x3_rightwards(src, dst, 1);
                dst += 1;
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 64, self.tile_word(src, 4));
                    self.room_write_current(dst + 128, self.tile_word(src, 5));
                    dst += 1;
                }
                self.room_draw_1x3_rightwards(src + 12, dst, 1);
            }
            0x49 | 0x4a => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:94B4: JSR $B0AC, spacing and JMP $B220.
                    // The real size-helper child retains its own call frame.
                    crate::cycle_ledger::charge(94);
                    self.price_room_draw_size_1to16();
                }
                self.room_draw_downwards4x2_variable_spacing(4, src, dsto, count);
            }
            0x4c => {
                let count = size_1to16(width, height) * 2;
                let mut dst = self.room_draw_rightward_bar_segment(src, dsto) + 1;
                for _ in 0..count {
                    dst = self.room_draw_rightward_bar_segment(src + 6, dst) + 1;
                }
                self.room_draw_rightward_bar_segment(src + 12, dst);
            }
            0x4d..=0x4f => {
                let count = size_1to16(width, height);
                let mut dst = dsto;
                self.room_draw_object_nx4(1, src, dst);
                dst += 1;
                for _ in 0..count {
                    self.room_draw_object_nx4(2, src + 8, dst);
                    dst += 2;
                }
                self.room_draw_rightward_shelf_end(src + 24, dst);
            }
            0x50 => {
                let count = size_a_to_a_plus_15(width, height, 2);
                self.object_fill_nx1(count, src, dsto);
            }
            0x51 | 0x52 | 0x5b | 0x5c => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.room_draw_1x3_rightwards(src, dst, 2);
                dst += 2;
                while count > 1 {
                    self.room_draw_1x3_rightwards(src + 12, dst, 2);
                    dst += 2;
                    count -= 1;
                }
                self.room_draw_1x3_rightwards(src + 24, dst, 2);
            }
            0x55 | 0x56 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:96F9: JSR $B0AC, spacing and JMP $B220.
                    // The real size-helper child retains its own call frame.
                    crate::cycle_ledger::charge(94);
                    self.price_room_draw_size_1to16();
                }
                self.room_draw_downwards4x2_variable_spacing(12, src, dsto, count);
            }
            0x5d => {
                let count = size_1to16(width, height) + 1;
                let mut dst = dsto;
                self.room_draw_1x3_rightwards(src, dst, 2);
                dst += 2;
                for _ in 0..count {
                    self.room_draw_rightward_bar_segment(src + 12, dst);
                    dst += 1;
                }
                self.room_draw_1x3_rightwards(src + 18, dst, 2);
            }
            0x5e | 0xbb => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9338..9346: size, JSR $9895, four INYs,
                    // DEC/BNE and RTS; $9895 retains its four-store price.
                    crate::cycle_ledger::charge(46 + 286 + 172 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_rightwards2x2(src, dsto + i * 4);
                }
            }
            0x5f => {
                let count = size_a_to_a_plus_15(width, height, 21);
                let preserve_corner = self.room_read_current(dsto) & 0x03ff == 0x00e2;
                if self.lag_free {
                    // $01:8EEB -> 8EF3..8F0B: alternate size entry and
                    // actual corner branch; children own their bodies.
                    crate::cycle_ledger::charge(178 + if preserve_corner { 6 } else { 102 } + 190);
                    self.price_room_draw_size_ato_aplus15();
                    self.price_room_draw_small_rail_corner(preserve_corner);
                }
                if !preserve_corner {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                self.room_object_fill_row_advance(count, src + 2, dsto + 1);
                self.room_write_current(dsto + count + 1, self.tile_word(src, 2));
            }
            0x60 | 0x92 | 0x93 => {
                let count = size_1to15_or(width, height, 32);
                if self.lag_free {
                    // $01:8B7E..8B88: size call, JSR $B2AF, actual
                    // DEC/BNE and RTS; child bodies excluded.
                    crate::cycle_ledger::charge(46 + 116 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    crate::cycle_ledger::charge_routine(0x01_b0cc,
                        if width == 0 && height == 0 { 206 } else { 188 });
                }
                for i in 0..count {
                    self.room_draw_downwards2x2(src, dsto + i * 64 * 2);
                }
            }
            0x61 | 0x62 | 0x90 | 0x91 => {
                let count = size_1to15_or(width, height, 26);
                if self.lag_free {
                    // $01:8A89..8A91: JSR size, spacing and JMP $B220.
                    // $01:B0BE owns its actual size-helper branch and RTS.
                    crate::cycle_ledger::charge(94);
                    crate::cycle_ledger::charge_routine(0x01_b0be,
                        if width == 0 && height == 0 { 206 } else { 188 });
                }
                self.room_draw_downwards4x2_variable_spacing(2 * 64, src, dsto, count);
            }
            0x63 | 0x64 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8AA4..8B0C: register setup, eight direct dual-BG
                    // triples per block, vertical step, actual loop and RTS.
                    crate::cycle_ledger::charge(88 + 1272 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    self.object_draw_4x2_both_bgs(src, dsto + i * 2 * 64);
                }
            }
            0x65 | 0x66 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8C4F: JSR $B0AC, spacing and JMP $B220.
                    // The real size-helper child retains its own call frame.
                    crate::cycle_ledger::charge(94);
                    self.price_room_draw_size_1to16();
                }
                self.room_draw_downwards4x2_variable_spacing(6 * 64, src, dsto, count);
            }
            0x67 | 0x68 | 0x7d => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8B74..8B88: size call/BRA, JSR $B2AF,
                    // actual DEC/BNE and RTS; child bodies excluded.
                    crate::cycle_ledger::charge(68 + 116 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    self.room_draw_downwards2x2(src, dsto + i * 2 * 64);
                }
            }
            0x7c => {
                let count = size_1to16(width, height) + 1;
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x7f | 0x80 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9702..9719: width-two $97F0 call and vertical
                    // spacing, size child $B0AC, loop and RTS.
                    crate::cycle_ledger::charge(78 + 286 + 238 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(2, src, dsto + i * 12 * 64);
                }
            }
            0x81..=0x84 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:971B..9732: width-three $97F0 call and vertical
                    // spacing; tile child independently prices its stores.
                    crate::cycle_ledger::charge(78 + 286 + 238 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(3, src, dsto + i * 6 * 64);
                }
            }
            0x85 | 0x86 => {
                let mut count = size_1to16(width, height);
                let mut dst = dsto;
                self.object_draw_3x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                while {
                    count -= 1;
                    count != 0
                } {
                    self.object_draw_3x2(src + 12, dst);
                    dst = dst.wrapping_add(2 * 64);
                }
                self.object_draw_3x2(src + 24, dst);
            }
            0x69 | 0x71 => {
                let count = size_a_to_a_plus_15(width, height, if idx == 0x69 { 2 } else { 4 });
                if idx == 0x69 {
                    let preserve_corner = self.room_read_current(dsto) & 0x03ff == 0x00e3;
                    if self.lag_free {
                        // $01:8EC3..8EEA: size/corner calls, actual corner
                        // branch, vertical stores/control and final tile/RTS.
                        crate::cycle_ledger::charge(156 + if preserve_corner { 6 } else { 102 }
                            + 238 * u64::from(count) + 6 * u64::from(count - 1) + 144);
                        self.price_room_draw_size_ato_aplus15();
                        self.price_room_draw_small_rail_corner(preserve_corner);
                    }
                    if !preserve_corner {
                        self.room_write_current(dsto, self.tile_word(src, 0));
                    }
                    let tile = self.tile_word(src, 1);
                    for i in 1..=count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                    self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
                } else {
                    let tile = self.tile_word(src, 0);
                    for i in 0..count {
                        self.room_write_current(dsto + i * 64, tile);
                    }
                }
            }
            0x6a | 0x6b | 0x79 | 0x7a | 0x8d | 0x8e => {
                let count = size_1to16(width, height);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x6c => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, self.tile_word(src, 1));
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 1, fill);
                    self.room_write_current(dst + 65, fill);
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, self.tile_word(src, 3));
                    self.room_write_current(dst + 1, fill);
                    dst += 64;
                }
                self.room_write_current(dst, self.tile_word(src, 4));
                self.room_write_current(dst + 64, self.tile_word(src, 5));
                self.room_write_current(dst + 1, fill);
                self.room_write_current(dst + 65, fill);
            }
            0x6d => {
                let count = size_a_to_a_plus_15(width, height, 10);
                let fill = self.tile_word(src, 0);
                let mut dst = dsto;
                if self.room_read_current(dst + 1) & 0x03ff != 0x00e3 {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 64, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    self.room_write_current(dst + 65, self.tile_word(src, 2));
                    dst += 128;
                }
                for _ in 0..count {
                    self.room_write_current(dst, fill);
                    self.room_write_current(dst + 1, self.tile_word(src, 3));
                    dst += 64;
                }
                self.room_write_current(dst, fill);
                self.room_write_current(dst + 64, fill);
                self.room_write_current(dst + 1, self.tile_word(src, 4));
                self.room_write_current(dst + 65, self.tile_word(src, 5));
            }
            0x70 | 0x94 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.room_draw_4x4(src, dsto + i * 4 * 64);
                }
            }
            0x73 | 0x74 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.room_draw_4x4(src, dsto + i * 6 * 64);
                }
            }
            0x75 | 0x87 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9357..936E, the other IDs sharing this ROM tail.
                    crate::cycle_ledger::charge(78 + 286 + 238 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                }
                for i in 0..count {
                    self.room_draw_object_nx4(2, src, dsto + i * 6 * 64);
                }
            }
            0x76 | 0x77 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.room_draw_object_nx4(3, src, dsto + i * 8 * 64);
                }
            }
            0x78 | 0x7b => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9446..9455: size call, JSR $B2AF, spacing,
                    // actual DEC/BNE and RTS; child bodies excluded.
                    crate::cycle_ledger::charge(46 + 168 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    self.room_draw_downwards2x2(src, dsto + i * 14 * 64);
                }
            }
            0x89 => {
                let count = size_1to16(width, height);
                if self.lag_free {
                    // $01:9347..9356: size call, JSR $B2AF, spacing,
                    // actual DEC/BNE and RTS; child bodies excluded.
                    crate::cycle_ledger::charge(46 + 168 * u64::from(count)
                        + 6 * u64::from(count - 1) + 42);
                    self.price_room_draw_size_1to16();
                }
                for i in 0..count {
                    self.room_draw_downwards2x2(src, dsto + i * 4 * 64);
                }
            }
            0x8a => {
                let count = size_a_to_a_plus_15(width, height, 21);
                let preserve_corner = self.room_read_current(dsto) & 0x03ff == 0x00e3;
                if self.lag_free {
                    // $01:8EBE -> 8EC6..8EEA: alternate size entry,
                    // actual corner branch, vertical stores/control and RTS.
                    crate::cycle_ledger::charge(178 + if preserve_corner { 6 } else { 102 }
                        + 238 * u64::from(count) + 6 * u64::from(count - 1) + 144);
                    self.price_room_draw_size_ato_aplus15();
                    self.price_room_draw_small_rail_corner(preserve_corner);
                }
                if !preserve_corner {
                    self.room_write_current(dsto, self.tile_word(src, 0));
                }
                let tile = self.tile_word(src, 1);
                for i in 1..=count {
                    self.room_write_current(dsto + i * 64, tile);
                }
                self.room_write_current(dsto + (count + 1) * 64, self.tile_word(src, 2));
            }
            0x88 => {
                let mut count = size_1to16(width, height);
                if self.lag_free {
                    // $01:8F0C..8F2F: size and $B2AF calls, direct paired
                    // stores and loop, tail JMP $8D80; child bodies excluded.
                    crate::cycle_ledger::charge(158 + 340 * u64::from(count)
                        + 6 * u64::from(count - 1) + 104);
                    self.price_room_draw_size_1to16();
                }
                let mut dst = dsto;
                self.room_draw_downwards2x2(src, dst);
                dst = dst.wrapping_add(2 * 64);
                let src = src + 8;
                while count != 0 {
                    self.room_write_current(dst, self.tile_word(src, 0));
                    self.room_write_current(dst + 1, self.tile_word(src, 1));
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
                self.room_draw_1x3_rightwards(src + 4, dst, 2);
            }
            0x8b | 0x8c => {
                let count = size_a_to_a_plus_15(width, height, 8);
                let tile = self.tile_word(src, 0);
                for i in 0..count {
                    self.room_write_current(dsto + i * 64, tile);
                }
            }
            0x8f => {
                let count = size_a_to_a_plus_15(width, height, 2) * 2;
                let mut dst = dsto;
                self.room_write_current(dst, self.tile_word(src, 0));
                self.room_write_current(dst + 1, self.tile_word(src, 1));
                for _ in 0..count {
                    self.room_write_current(dst + 64, self.tile_word(src, 2));
                    self.room_write_current(dst + 65, self.tile_word(src, 3));
                    dst = dst.wrapping_add(64);
                }
            }
            0x95 => {
                let count = size_1to16(width, height);
                let mut pos = dsto;
                for _ in 0..count {
                    self.room_draw_single_pot(src, pos);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0x96 => {
                let count = size_1to16(width, height);
                let mut pos = dsto;
                for _ in 0..count {
                    self.room_draw_hammer_peg_single(src, pos);
                    pos = pos.wrapping_add(2 * 64);
                }
            }
            0xa0 | 0xa5 | 0xa9 => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_object_fill_row_width(count, src, dst);
                    dst = dst.wrapping_add(64);
                    count -= 1;
                }
            }
            0xa1 | 0xa6 | 0xaa => {
                let count = size_a_to_a_plus_15(width, height, 4);
                for y in 0..count {
                    self.room_object_fill_row_a(y + 1, src, dsto + y * 64);
                }
            }
            0xa2 | 0xa7 | 0xab => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_object_fill_row_width(count, src, dst);
                    dst = dst.wrapping_add(65);
                    count -= 1;
                }
            }
            0xa3 | 0xa8 | 0xac => {
                let mut count = size_a_to_a_plus_15(width, height, 4);
                let mut dst = dsto;
                while count != 0 {
                    self.room_object_fill_row_width(count, src, dst);
                    dst = dst.wrapping_sub(63);
                    count -= 1;
                }
            }
            0xa4 => self.object_hole(src, dsto, width, height),
            0xb0 | 0xb1 => {
                let count = size_a_to_a_plus_15(width, height, 8);
                self.object_fill_nx1(count, src, dsto);
            }
            0xb5 => {
                let count = size_1to16(width, height);
                for i in 0..count {
                    self.room_draw_object_nx4(2, 0x0b16, dsto + i * 2);
                }
            }
            0xbc => {
                let count = size_1to16(width, height);
                let mut pos = dsto;
                for _ in 0..count {
                    self.room_draw_single_pot(src, pos);
                    pos = pos.wrapping_add(2);
                }
            }
            0xbd => {
                let count = size_1to16(width, height);
                let mut pos = dsto;
                for _ in 0..count {
                    self.room_draw_hammer_peg_single(src, pos);
                    pos = pos.wrapping_add(2);
                }
            }
            0xc5..=0xca | 0xd1 | 0xd2 | 0xd9 | 0xdf..=0xe8 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                if self.lag_free {
                    // $01:8FA5..8FBB: dimension increments, one real
                    // JSR $8A44 per row, pointer/loop work and RTS.
                    // The existing $8A44 child owns all block stores.
                    crate::cycle_ledger::charge(108 + 264 * u64::from(count_y)
                        + 6 * u64::from(count_y - 1) + 42);
                }
                for y in 0..count_y {
                    // Object_Draw4x4_Size1to4 $018FA9: one JSR RoomDraw_A_Many32x32Blocks
                    // per row with A = the block count (the chained n = 1 calls wrote
                    // the same blocks in the same order).
                    self.room_draw_a_many32x32_blocks(count_x as i32, src, dsto + y * 4 * 64);
                }
            }
            0xc3 | 0xd7 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                if self.lag_free {
                    // $01:8D9E..8DDB: dimensions, per-row setup, nine
                    // direct stores per 3x3 block, actual branches and RTS.
                    // room_fill_rect has no child charge.
                    let columns = u64::from(count_x);
                    let rows = u64::from(count_y);
                    crate::cycle_ledger::charge(108
                        + rows * (64 + 798 * columns + 6 * (columns - 1) + 186)
                        + 6 * (rows - 1) + 42);
                }
                let tile = self.tile_word(src, 0);
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.room_fill_rect(dsto + y * 3 * 64 + x * 3, 3, 3, tile);
                    }
                }
            }
            0xd8 => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                self.water_hdma_window_mut().set_window_x_radius(count_x << 4);
                self.water_hdma_window_mut().set_window_y_radius(count_y << 4);
                self.water_hdma_window_mut().set_window_y_target((count_y << 4).wrapping_sub(24));
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v());
                self.water_hdma_window_mut()
                    .set_window_position(hdma0, hdma1);
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0 {
                    self.dungeon_header_mut().clear_header_tag(1);
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    self.dungeon_stair_lists_mut()
                        .promote_water_stairs_to_active();
                    let water_dsto = dsto
                        .wrapping_add((count_x - 1) << 1)
                        .wrapping_add((count_y - 1) << 7);
                    self.DrawWaterThingBg(0x4000, water_dsto, 0x1438);
                } else {
                    let mut dst = dsto;
                    for _ in 0..count_y {
                        self.room_draw_a_many32x32_blocks(count_x as i32, 0x0110, dst);
                        dst = dst.wrapping_add(4 * 64);
                    }
                }
            }
            0xda => {
                let count_x = width as u16 + 2;
                let count_y = height as u16 + 2;
                self.water_hdma_window_mut().set_window_x_radius((count_x << 4).wrapping_sub(24));
                self.water_hdma_window_mut().set_window_y_target((count_y << 4).wrapping_sub(8));
                self.water_hdma_window_mut().set_window_y_radius((count_y << 4).wrapping_sub(32));
                self.water_hdma_window_mut().set_window_y_radius_alt(0);
                let hdma0 = ((dsto & 0x003f) << 3)
                    .wrapping_add(count_x << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h());
                let hdma1 = ((dsto & 0x0fc0) >> 3)
                    .wrapping_add(count_y << 4)
                    .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v())
                    .wrapping_sub(8);
                self.water_hdma_window_mut()
                    .set_window_position(hdma0, hdma1);
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0 {
                    self.dungeon_header_mut().clear_header_tag(1);
                } else {
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    self.dungeon_stair_lists_mut()
                        .promote_water_stairs_to_active();
                }
                let mut dst = dsto;
                for _ in 0..(count_y * 2 - 1) {
                    let row = dst;
                    for _ in 0..count_x {
                        for y in 0..2 {
                            for x in 0..4 {
                                self.room_write_current(
                                    dst + y * 64 + x,
                                    self.tile_word(0x0110, (y * 4 + x) as usize),
                                );
                            }
                        }
                        dst = dst.wrapping_add(4);
                    }
                    dst = row.wrapping_add(2 * 64);
                }
            }
            0xc4 => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = self
                    .game_state
                    .dungeon
                    .room_parser
                    .floor_2_filler_tile_source();
                for y in 0..count_y {
                    self.room_draw_a_many32x32_blocks(count_x as i32, src, dsto + y * 4 * 64);
                }
            }
            0xdb => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                let src = self
                    .game_state
                    .dungeon
                    .room_parser
                    .floor_1_filler_tile_source();
                for y in 0..count_y {
                    self.room_draw_a_many32x32_blocks(count_x as i32, src, dsto + y * 4 * 64);
                }
            }
            0xc0 | 0xc2 => {
                if self.lag_free {
                    // $01:8B94..8BDF: INC dimensions, sixteen direct stores
                    // per 4x4 block, actual column/row branches and RTS.
                    // room_fill_rect is unpriced; there is no child charge.
                    let columns = u64::from(width) + 1;
                    let rows = u64::from(height) + 1;
                    crate::cycle_ledger::charge(108
                        + rows * (64 + columns * 1190 + (columns - 1) * 6 + 186)
                        + (rows - 1) * 6 + 42);
                }
                let tile = self.tile_word(src, 0);
                for y in 0..=height as u16 {
                    for x in 0..=width as u16 {
                        self.room_fill_rect(dsto + y * 4 * 64 + x * 4, 4, 4, tile);
                    }
                }
            }
            0xc1 => {
                let width = width as u16 + 4;
                let height = height as u16 + 1;
                let mut src = src;
                let mut dst = dsto;
                self.room_draw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.room_draw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.room_draw_1x3_rightwards(src + 12, dst, 3);
                src += 30;

                dst = dsto + 3 * 64;
                for _ in 0..height {
                    let mut row = dst;
                    self.object_draw_3x2(src, row);
                    row += 3;
                    for _ in 0..width {
                        self.room_draw_rightwards2x2(src + 12, row);
                        row += 2;
                    }
                    self.object_draw_3x2(src + 20, row);
                    dst += 2 * 64;
                }

                let bottom_start = dst;
                src += 32;
                self.room_draw_1x3_rightwards(src, dst, 3);
                src += 18;
                dst += 3;
                for _ in 0..width {
                    self.room_draw_1x3_rightwards(src, dst, 2);
                    dst += 2;
                }
                self.room_draw_1x3_rightwards(src + 12, dst, 3);

                self.room_draw_rightwards2x2(
                    0x0590,
                    bottom_start
                        .wrapping_add(width + 2)
                        .wrapping_sub((height + 1) * 64),
                );
            }
            0xcd => self.room_draw_moving_wall_right(width, height, dsto),
            0xce => self.room_draw_moving_wall_left(width, height, dsto),
            0x31
            | 0x32
            | 0x35
            | 0x54
            | 0x57..=0x5a
            | 0x6e
            | 0x6f
            | 0x72
            | 0x7e
            | 0x97..=0x9f
            | 0xad..=0xaf
            | 0xbe
            | 0xbf => {}
            0xdd => {
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 2;
                self.object_table_helper(src, dsto, width);
                for y in 1..height {
                    self.object_table_helper(src + 8, dsto + y * 64, width);
                }
                self.object_table_helper(src + 16, dsto + height * 64, width);
                self.object_table_helper(src + 24, dsto + (height + 1) * 64, width);
            }
            0xdc => {
                let mut dst = dsto
                    | if self
                        .game_state
                        .dungeon
                        .room_tilemaps
                        .first_line_pointer_row0()
                        == 0x4000
                    {
                        0x1000
                    } else {
                        0
                    };
                let width = width as u16 + 1;
                let height = height as u16 * 2 + 5;
                for _ in 0..height {
                    self.room_draw_chest_platform_row(0x0ab4, dst, width);
                    dst += 64;
                }
                self.room_draw_chest_platform_row(0x0ab4 + 2, dst, width);
                dst += 64;
                self.room_draw_chest_platform_row(0x0ab4 + 4, dst, width);
            }
            0xde => {
                let count_x = width as u16 + 1;
                let count_y = height as u16 + 1;
                for y in 0..count_y {
                    for x in 0..count_x {
                        self.room_draw_rightwards2x2(src, dsto + y * 2 * 64 + x * 2);
                    }
                }
            }
            _ => panic!("LoadType1ObjectSubtype1 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype3(&mut self, idx: u8, dsto: u16) {
        let Some(src) = object_subtype3_param(idx) else {
            panic!("LoadType1ObjectSubtype3 invalid object id {idx:#04x}");
        };
        match idx {
            0x00 => {
                if self.game_state.dungeon.header.header_tag(1) == 27 {
                    let room = self.game_state.world.location.dungeon_room() as usize;
                    if self.saved_room_flags(room) & 0x0100 != 0 {
                        self.room_draw_water_holding_object(5, 0x162c, dsto);
                        return;
                    }
                } else if self.game_state.dungeon.header.header_tag(1) == 25
                    && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 != 0
                {
                    self.room_draw_water_holding_object(5, 0x162c, dsto);
                    return;
                }
                self.dungeon_room_items_mut()
                    .set_replacement_tile_source_x2(dsto * 2);
                self.room_draw_water_holding_object(3, src, dsto);
            }
            0x01 => self.room_draw_water_holding_object(5, 0x162c, dsto),
            0x02 => self.room_draw_water_holding_object(7, src, dsto),
            0x03 | 0x0e => {
                self.dungeon_environment_mut()
                    .increment_somaria_block_bg_check_flag();
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x04..=0x0c | 0x0f => {
                self.room_write_current(dsto, self.tile_word(src, 0));
            }
            0x0d | 0x17 => self.room_draw_prison_cell(dsto),
            0x10
            | 0x11
            | 0x13
            | 0x1a
            | 0x22..=0x25
            | 0x3e..=0x46
            | 0x49
            | 0x4a
            | 0x4f..=0x53
            | 0x56..=0x59
            | 0x5e
            | 0x5f
            | 0x63..=0x65
            | 0x75
            | 0x7c..=0x7e => {
                self.room_draw_rightwards2x2(src, dsto);
            }
            0x12 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x1000 == 0 {
                    let dst = dsto | self.room_plane_offset();
                    let src = 0x1dd6;
                    for i in 0..3 {
                        let col = i * 2;
                        self.room_write_bg2(dst + col, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 3 * 64, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 6 * 64, self.tile_word(src, 0));
                        self.room_write_bg2(dst + col + 64, self.tile_word(src, 1));
                        self.room_write_bg2(dst + col + 4 * 64, self.tile_word(src, 1));
                        self.room_write_bg2(dst + col + 7 * 64, self.tile_word(src, 1));
                    }
                }
            }
            0x14 | 0x4e | 0x67 | 0x68 | 0x6c | 0x6d | 0x79 => {
                self.room_draw_1x3_rightwards(src, dsto, 4);
            }
            0x15 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
                    self.room_draw_some_big_decors(10, 0x1dfa, dsto);
                }
            }
            0x16 => {
                self.room_draw_hammer_peg_single(src, dsto);
            }
            0x18 => self.room_draw_cell_lock(dsto),
            0x19 => self.room_draw_chest(dsto),
            0x1b => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::Stairs1, dsto);
                self.object_draw_nx4_both_bgs(4, src, dsto);
            }
            0x1c => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::Stairs2, dsto);
                self.object_draw_nx4_both_bgs(4, src, dsto);
            }
            0x1d => {
                self.dungeon_stair_lists_mut()
                    .append_stair_table_position(DungeonStairList::WetStairs, dsto);
                // RoomDraw_AutoStairs_South_MultiLayer_C $01A37D: JMP RoomDraw_4x4.
                self.room_draw_4x4_by_jump(src, dsto);
            }
            0x1e => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpNorthStraight, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomUpNorthStraight,
                        DungeonStairList::InterRoomUpSouthStraight,
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_object_nx4_tiles(src, dsto, 4); // the ROM inlines its own 4-column loop at $01A634 (no Nx4 entry)
            }
            0x1f => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomDownNorthStraight, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_object_nx4_tiles(src, dsto, 4); // the ROM inlines its own 4-column loop at $01A634 (no Nx4 entry)
            }
            0x20 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpSouthStraight, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomUpSouthStraight,
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_object_nx4_tiles(src, dsto, 4); // the ROM inlines its own 4-column loop at $01A634 (no Nx4 entry)
            }
            0x21 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomDownSouthStraight, dsto);
                self.dungeon_stair_lists_mut()
                    .set_stair_list_count(DungeonStairList::InterRoomDownSouthStraight, next);
                self.room_draw_object_nx4_tiles(src, dsto, 4); // the ROM inlines its own 4-column loop at $01A634 (no Nx4 entry)
            }
            0x26 => self.room_draw_lower_door_stairs_up(src, dsto, true),
            0x27 => self.room_draw_lower_door_stairs_up(src, dsto, false),
            0x28 => self.room_draw_lower_door_stairs_down(src, dsto, true),
            0x29 => self.room_draw_lower_door_stairs_down(src, dsto, false),
            0x2a => {
                self.room_draw_single_lamp_cone(0x0514, 0x16dc);
                self.room_draw_single_lamp_cone(0x0554, 0x17f6);
                self.room_draw_single_lamp_cone(0x1514, 0x1914);
                self.room_draw_single_lamp_cone(0x1554, 0x1a2a);
            }
            0x2b => {
                self.DrawBigGraySegment(ObjectRecord::liftable(0), src, dsto);
            }
            0x2c => {
                self.DrawBigGraySegment(ObjectRecord::big_rock_segment(0), 0x0e62, dsto);
                self.DrawBigGraySegment(
                    ObjectRecord::big_rock_segment(1),
                    0x0e6a,
                    dsto + xy(2, 0) as u16,
                );
                self.DrawBigGraySegment(
                    ObjectRecord::big_rock_segment(2),
                    0x0e72,
                    dsto + xy(0, 2) as u16,
                );
                self.DrawBigGraySegment(
                    ObjectRecord::big_rock_segment(3),
                    0x0e7a,
                    dsto + xy(2, 2) as u16,
                );
            }
            0x2d => self.room_draw_agahnim_altar(dsto),
            0x2e => self.room_draw_agahnims_windows(dsto),
            0x2f => {
                self.room_draw_single_pot(0x0e82, dsto);
            }
            0x30 => {
                self.DrawBigGraySegment(ObjectRecord::liftable(2), src, dsto);
            }
            0x31 => {
                let loc = (dsto * 2) | 0x8000 | self.room_plane_tilemap_bit();
                let chest = self
                    .dungeon_room_items_mut()
                    .append_chest_location_and_sync_big_key_count(loc);
                if chest < DUNGEON_CHEST_OPEN_MASKS.len()
                    && self.game_state.dungeon.savegame_state.savegame_state_bits()
                        & DUNGEON_CHEST_OPEN_MASKS[chest]
                        != 0
                {
                    self.dungeon_room_items_mut().clear_chest_location(chest);
                    self.room_draw_1x3_rightwards(0x14c4, dsto, 4);
                } else {
                    self.room_draw_1x3_rightwards(0x14ac, dsto, 4);
                }
            }
            0x32 => self.room_draw_1x3_rightwards(src, dsto, 4),
            0x33 => {
                if self.game_state.dungeon.header.header_tag(1) == 27 {
                    let room = self.game_state.world.location.dungeon_room() as usize;
                    if self.saved_room_flags(room) & 0x0100 == 0 {
                        self.dungeon_room_load_mut().clear_bg2_properties();
                        self.dungeon_stair_lists_mut()
                            .append_stair_table_position(DungeonStairList::WetStairs, dsto);
                    } else {
                        self.set_color_window_selection(2);
                        self.set_color_math_control(0x62);
                        self.dungeon_stair_lists_mut().append_stair_table_position(
                            DungeonStairList::InRoomUpSouthWater,
                            dsto,
                        );
                    }
                } else {
                    self.dungeon_stair_lists_mut()
                        .append_stair_table_position(DungeonStairList::InRoomUpSouthWater, dsto);
                }
                // RoomDraw_AutoStairs_South_MergedLayer $01A3AB: JMP RoomDraw_4x4.
                self.room_draw_4x4_by_jump(src, dsto);
            }
            0x3a | 0x3b => {
                self.room_draw_1x3_rightwards(src, dsto, 4);
                self.room_draw_1x3_rightwards(src + 24, dsto + 3 * 64, 4);
            }
            // RoomDraw_HorizontalTurtleRockPipe $019AA6: LDA #6, JMP RoomDraw_Object_Nx4
            // (the body charges into this frame; no Nx4 scope).
            0x3c | 0x3d | 0x5c => self.room_data_draw_object_nx4(src, dsto, 6),
            0x47 => {
                self.room_draw_bombable_floor(dsto);
            }
            // The subtype-3 table points straight at RoomDraw_4x4 $0197ED, so the
            // `JMP ($0E)` dispatch enters it without a JSR.
            0x48 | 0x66 | 0x6b | 0x7a => self.room_draw_4x4_by_jump(src, dsto),
            0x4b | 0x76 | 0x77 => self.room_draw_1x3_rightwards(src, dsto, 8),
            0x4c => self.room_draw_some_big_decors(6, 0x1f92, dsto),
            0x4d | 0x5d => self.room_draw_1x3_rightwards(src, dsto, 6),
            0x54 => self.room_draw_fortune_teller_room(dsto),
            0x55 | 0x5b => {
                for x in 0..3 {
                    self.room_write_current(dsto + x, self.tile_word(src, x as usize));
                }
                for y in 1..=3 {
                    for x in 0..3 {
                        self.room_write_current(
                            dsto + y * 64 + x,
                            self.tile_word(src, (3 + x) as usize),
                        );
                    }
                }
                for x in 0..3 {
                    self.room_write_current(
                        dsto + x + 4 * 64,
                        self.tile_word(src, (6 + x) as usize),
                    );
                }
            }
            0x5a => self.room_draw_water_holding_object(2, src, dsto),
            0x60 | 0x61 => {
                self.room_draw_1x3_rightwards(src, dsto, 3);
                self.room_draw_1x3_rightwards(src + 18, dsto + 3 * 64, 3);
            }
            0x62 => {
                let mut dst = dsto;
                let mut s = 0x20f6;
                for _ in 0..22 {
                    for y in 0..11 {
                        self.room_write_bg1(dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 22;
                }
                dst -= 22;
                s = 0x22da;
                for i in 0..3 {
                    self.room_write_bg1(dst + 9 + 11 * 64, self.tile_word(s, i));
                    self.room_write_bg1(dst + 9 + 12 * 64, self.tile_word(s, i + 3));
                    dst += 1;
                }
            }
            // RoomDraw_SolidWallDecor3x4 $0199EF: LDA #3, JMP RoomDraw_Object_Nx4.
            0x69 | 0x6a | 0x6e | 0x6f => self.room_data_draw_object_nx4(src, dsto, 3),
            0x70 => {
                self.room_draw_4x4(src, dsto);
                self.room_draw_4x4(0x2376, dsto + 2 * 64);
                // RoomDraw_LightBeamOnFloor $01A7D0: the third 4x4 is a JMP.
                self.room_draw_4x4_by_jump(0x2396, dsto + 6 * 64);
            }
            0x71 => {
                if self.saved_room_flags(101) & 0x0100 != 0 {
                    self.object_draw8x8(src, dsto);
                }
            }
            0x72 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
                    self.room_draw_some_big_decors(10, src, dsto);
                }
            }
            // RoomDraw_BG2MaskFull $01A25A: JMP RoomDraw_FloorChunks (no scope).
            0x73 => self.room_draw_floor_chunks_body(
                self.game_state
                    .dungeon
                    .room_tilemaps
                    .first_line_pointer_row0() as usize,
                0x00e0,
            ),
            0x74 => {
                self.object_draw8x8(src, dsto);
            }
            0x78 => {
                self.room_draw_4x4(src, dsto);
                self.room_draw_4x4(src + 32, dsto.wrapping_sub(2).wrapping_add(4 * 64));
                // RoomDraw_GanonTriforceFloorDecor $01A806: the third 4x4 is a JMP.
                self.room_draw_4x4_by_jump(src + 32, dsto + 2 + 4 * 64);
            }
            0x7b => {
                // RoomDraw_VitreousGooDamage $01A809: LDA #5, JSR
                // RoomDraw_A_Many32x32Blocks (one scoped call of five blocks),
                // then LDA #5, JMP RoomDraw_A_Many32x32Blocks (the body only).
                self.room_draw_a_many32x32_blocks(5, src, dsto);
                self.room_draw_a_many_32x32_blocks_body(5, src, dsto + 4 * 64);
            }
            _ => panic!("LoadType1ObjectSubtype3 unhandled object id {idx:#04x}"),
        }
    }

    pub(super) fn LoadType1ObjectSubtype2(&mut self, idx: u8, dsto: u16) {
        let Some(src) = object_subtype2_param(idx) else {
            panic!("LoadType1ObjectSubtype2 invalid object id {idx:#04x}");
        };
        match idx {
            0x00..=0x07 | 0x1c | 0x24 | 0x25 | 0x29 => {
                // The subtype-2 table points straight at RoomDraw_4x4 $0197ED, so
                // the `JMP ($0E)` dispatch enters it without a JSR.
                self.room_draw_4x4_by_jump(src, dsto);
            }
            0x10..=0x13 => self.room_data_draw_object_nx4_both_bgs(src, dsto, 3),
            0x14..=0x17 => self.object_draw_nx3_both_bgs(4, src, dsto),
            0x18..=0x1b | 0x27 | 0x2b | 0x34 => self.room_draw_rightwards2x2(src, dsto),
            0x1d | 0x21 | 0x26 => self.room_draw_1x3_rightwards(src, dsto, 2),
            0x1e => self.room_draw_rightwards2x2(src, dsto),
            0x1f => {
                let plane = self.room_plane_offset();
                self.dungeon_room_parser_mut()
                    .append_star_switch_tile(dsto | plane);
                self.room_draw_rightwards2x2(src, dsto);
            }
            0x20 => {
                self.dungeon_torch_mut().increment_lit_torches();
                self.room_draw_rightwards2x2(src, dsto);
            }
            0x22 | 0x28 => self.object_draw_5x4(src, dsto),
            0x23 => self.room_draw_1x3_rightwards(src, dsto, 4),
            0x2a => {
                self.dungeon_room_load_mut().set_draw_width_indicator(1);
                self.room_draw_downwards4x2_variable_spacing(1, src, dsto, 1);
            }
            0x2c => self.room_draw_1x3_rightwards(src, dsto, 6),
            0x2d => {
                let plane = self.room_plane_offset();
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomUpNorth, dsto | plane);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomUpNorth,
                        DungeonStairList::WallUpNorthSpiral,
                        DungeonStairList::WallUpNorthSpiralBg1,
                        DungeonStairList::InterRoomUpNorthStraight,
                        DungeonStairList::InterRoomUpSouthStraight,
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                // RoomDraw_InterRoomFatStairsUp $01A455: JMP RoomDraw_4x4.
                self.room_draw_4x4_by_jump(0x1088, dsto);
            }
            0x2e | 0x2f => {
                let plane = self.room_plane_offset();
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_interroom_staircase(DungeonStairList::InterRoomSouthDown, dsto | plane);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                // RoomDraw_InterRoomFatStairsDown_A/_B $01A483/$01A4B1: JMP RoomDraw_4x4.
                self.room_draw_4x4_by_jump(0x10a8, dsto);
            }
            0x08..=0x0f => {
                if self.lag_free {
                    // $01:9813..9853: register/count setup, four dual-BG
                    // columns (732 each), three taken branches and RTS.
                    // Direct dispatch/JMP stays in the $893C caller frame.
                    crate::cycle_ledger::charge(98 + 4 * 732 + 3 * 6 + 42);
                }
                self.room_data_draw_object_nx4_both_bgs(src, dsto, 4);
            }
            0x31 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::InRoomSouthDown, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WaterLadders,
                        DungeonStairList::WaterSideStepSwitch,
                    ],
                    next,
                );
                self.room_data_draw_object_nx4_both_bgs(src, dsto, 4);
            }
            0x32 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::InterPseudoUpNorth, dsto);
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WaterLadders,
                        DungeonStairList::WaterSideStepSwitch,
                    ],
                    next,
                );
                self.room_draw_4x4_by_jump(src, dsto); // RoomDraw_AutoStairs_North_MergedLayer_A $01A2DC: JMP RoomDraw_4x4
            }
            0x33 => {
                let room = self.game_state.world.location.dungeon_room() as usize;
                if self.game_state.dungeon.header.header_tag(1) == 27
                    && self.saved_room_flags(room) & 0x0100 == 0
                {
                    self.dungeon_room_load_mut().clear_bg2_properties();
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::InterPseudoUpNorth,
                            dsto,
                        );
                    self.dungeon_stair_lists_mut().sync_stair_list_counts(
                        &[
                            DungeonStairList::WaterLadders,
                            DungeonStairList::WaterSideStepSwitch,
                        ],
                        next,
                    );
                    // RoomDraw_AutoStairs_North_MergedLayer_B $01A309: JMP RoomDraw_4x4.
                    self.room_draw_4x4_by_jump(0x10c8, dsto);
                } else {
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::InRoomUpNorthWater,
                            dsto,
                        );
                    self.dungeon_stair_lists_mut()
                        .set_stair_list_count(DungeonStairList::ActivatedWaterLadders, next);
                    // RoomDraw_AutoStairs_North_MergedLayer_B $01A309: JMP RoomDraw_4x4.
                    self.room_draw_4x4_by_jump(0x10c8, dsto);
                }
            }
            0x35 => {
                let room = self.game_state.world.location.dungeon_room() as usize;
                if self.game_state.dungeon.header.header_tag(1) == 27
                    && self.saved_room_flags(room) & 0x0100 == 0
                {
                    let next = self
                        .dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(DungeonStairList::WaterLadders, dsto);
                    self.dungeon_stair_lists_mut()
                        .set_stair_list_count(DungeonStairList::WaterSideStepSwitch, next);
                    self.object_draw_4x2_both_bgs(0x1108, dsto);
                } else {
                    self.dungeon_stair_lists_mut()
                        .append_bg1_stair_table_position(
                            DungeonStairList::ActivatedWaterLadders,
                            dsto,
                        );
                    self.dungeon_room_load_mut().set_draw_width_indicator(1);
                    self.room_draw_downwards4x2_variable_spacing(1, 0x1108, dsto, 1);
                }
            }
            0x36 => {
                let next = self
                    .dungeon_stair_lists_mut()
                    .append_bg1_stair_table_position(DungeonStairList::WaterLadders, dsto);
                self.dungeon_stair_lists_mut()
                    .set_stair_list_count(DungeonStairList::WaterSideStepSwitch, next);
                self.object_draw_4x2_both_bgs(0x1108, dsto);
            }
            0x37 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x0800 == 0 {
                    self.room_draw_object_nx4(10, src, dsto);
                    self.water_hdma_window_mut().set_watergate_pointer(0x0f);
                    self.water_hdma_window_mut()
                        .set_watergate_tilemap_pos_x2(dsto * 2);
                } else {
                    self.room_draw_object_nx4(10, 0x13e8, dsto);
                    let load_ptr = self.game_state.dungeon.door_setup.active_room_load_ptr();
                    let load_ptr_offs = self.game_state.dungeon.room_load.load_ptr_offset();
                    let load_ptr_bank = self
                        .game_state
                        .dungeon
                        .door_setup
                        .active_room_load_ptr_bank();
                    self.room_tag_operate_water_flooring();
                    self.dungeon_room_doors_mut()
                        .set_active_room_load_ptr_bank(load_ptr_bank);
                    self.dungeon_room_load_mut()
                        .set_load_ptr_offset(load_ptr_offs);
                    self.dungeon_room_doors_mut()
                        .set_active_room_load_ptr(load_ptr);
                }
            }
            0x38 => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallUpNorthSpiral,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallUpNorthSpiral,
                        DungeonStairList::WallUpNorthSpiralBg1,
                        DungeonStairList::InterRoomUpNorthStraight,
                        DungeonStairList::InterRoomUpSouthStraight,
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_1x3_rightwards(0x1148, dsto, 4);
                let left = self.room_read_bg2(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg2(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg2(dsto + 4) | 0x2000;
                self.room_write_bg2(dsto + 4, right);
            }
            0x39 => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallDownNorthSpiral,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_1x3_rightwards(0x1160, dsto, 4);
                let left = self.room_read_bg2(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg2(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg2(dsto + 4) | 0x2000;
                self.room_write_bg2(dsto + 4, right);
            }
            0x3a => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallUpNorthSpiralBg1,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallUpNorthSpiralBg1,
                        DungeonStairList::InterRoomUpNorthStraight,
                        DungeonStairList::InterRoomUpSouthStraight,
                        DungeonStairList::InterRoomSouthDown,
                        DungeonStairList::WallDownNorthSpiral,
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_1x3_rightwards(0x1178, dsto, 4);
                let left = self.room_read_bg1(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg1(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg1(dsto + 4) | 0x2000;
                self.room_write_bg1(dsto + 4, right);
            }
            0x3b => {
                let plane = self.room_plane_offset();
                let next = self.dungeon_stair_lists_mut().append_interroom_staircase(
                    DungeonStairList::WallDownNorthSpiralBg1,
                    dsto.wrapping_sub(0x40) | plane,
                );
                self.dungeon_stair_lists_mut().sync_stair_list_counts(
                    &[
                        DungeonStairList::WallDownNorthSpiralBg1,
                        DungeonStairList::InterRoomDownNorthStraight,
                        DungeonStairList::InterRoomDownSouthStraight,
                    ],
                    next,
                );
                self.room_draw_1x3_rightwards(0x1190, dsto, 4);
                let left = self.room_read_bg1(dsto.wrapping_sub(1)) | 0x2000;
                self.room_write_bg1(dsto.wrapping_sub(1), left);
                let right = self.room_read_bg1(dsto + 4) | 0x2000;
                self.room_write_bg1(dsto + 4, right);
            }
            0x3c => {
                let mut dst = dsto;
                let mut s = src;
                for _ in 0..6 {
                    let tile0 = self.tile_word(s, 0);
                    let tile6 = self.tile_word(s, 6);
                    for x in [0, 4, 8, 14, 18, 22] {
                        self.room_write_bg2(dst + x, tile0);
                    }
                    for x in [1, 5, 9, 15, 19, 23] {
                        self.room_write_bg2(dst + x, tile0 | 0x4000);
                    }
                    for x in [2, 6, 16, 20] {
                        self.room_write_bg2(dst + x, tile6);
                    }
                    for x in [3, 7, 17, 21] {
                        self.room_write_bg2(dst + x, tile6 | 0x4000);
                    }
                    dst += 64;
                    s += 2;
                }
                self.room_draw_1x3_rightwards(src + 24, dsto + 10, 4);
            }
            0x3e => self.room_draw_1x3_rightwards(src, dsto, 6),
            0x3f => {
                let mut dst = dsto | self.room_plane_offset();
                let mut s = src;
                for _ in 0..8 {
                    for y in 0..7 {
                        self.room_write_bg2(dst + y * 64, self.tile_word(s, y as usize));
                    }
                    dst += 1;
                    s += 14;
                }
            }
            _ => panic!("LoadType1ObjectSubtype2 unhandled object id {idx:#04x}"),
        }
    }
}

pub(super) fn object_subtype1_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 256] = [
        0x03d8, 0x02e8, 0x02f8, 0x0328, 0x0338, 0x0400, 0x0410, 0x0388, 0x0390, 0x0420, 0x042a,
        0x0434, 0x043e, 0x0448, 0x0452, 0x045c, 0x0466, 0x0470, 0x047a, 0x0484, 0x048e, 0x0498,
        0x04a2, 0x04ac, 0x04b6, 0x04c0, 0x04ca, 0x04d4, 0x04de, 0x04e8, 0x04f2, 0x04fc, 0x0506,
        0x0598, 0x0600, 0x063c, 0x063c, 0x063c, 0x063c, 0x063c, 0x0642, 0x064c, 0x0652, 0x0658,
        0x065e, 0x0664, 0x066a, 0x0688, 0x0694, 0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x078a,
        0x07aa, 0x0e26, 0x084a, 0x086a, 0x0882, 0x08ca, 0x085a, 0x08fa, 0x091a, 0x0920, 0x092a,
        0x0930, 0x0936, 0x093c, 0x0942, 0x0948, 0x094e, 0x096c, 0x097e, 0x098e, 0x0902, 0x099e,
        0x09d8, 0x09d8, 0x09d8, 0x09fa, 0x156c, 0x1590, 0x1d86, 0x0000, 0x0a14, 0x0a24, 0x0a54,
        0x0a54, 0x0a84, 0x0a84, 0x14dc, 0x1500, 0x061e, 0x0e52, 0x0600, 0x03d8, 0x02c8, 0x02d8,
        0x0308, 0x0318, 0x03e0, 0x03f0, 0x0378, 0x0380, 0x05fa, 0x0648, 0x064a, 0x0670, 0x067c,
        0x06a8, 0x06a8, 0x06a8, 0x06c8, 0x0000, 0x07aa, 0x07ca, 0x084a, 0x089a, 0x08b2, 0x090a,
        0x0926, 0x0928, 0x0912, 0x09f8, 0x1d7e, 0x0000, 0x0a34, 0x0a44, 0x0a54, 0x0a6c, 0x0a84,
        0x0a9c, 0x1524, 0x1548, 0x085a, 0x0606, 0x0e52, 0x05fa, 0x06a0, 0x06a2, 0x0b12, 0x0b14,
        0x09b0, 0x0b46, 0x0b56, 0x1f52, 0x1f5a, 0x0288, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x03d8, 0x03d8, 0x03d8, 0x03d8, 0x05aa,
        0x05b2, 0x05b2, 0x05b2, 0x05b2, 0x00e0, 0x00e0, 0x00e0, 0x00e0, 0x0110, 0x0000, 0x0000,
        0x06a4, 0x06a6, 0x0ae6, 0x0b06, 0x0b0c, 0x0b16, 0x0b26, 0x0b36, 0x1f52, 0x1f5a, 0x0288,
        0x0eba, 0x0e82, 0x1df2, 0x0000, 0x0000, 0x03d8, 0x0510, 0x05aa, 0x05aa, 0x0000, 0x0168,
        0x00e0, 0x0158, 0x0100, 0x0110, 0x0178, 0x072a, 0x072a, 0x072a, 0x075a, 0x0670, 0x0670,
        0x0130, 0x0148, 0x072a, 0x072a, 0x072a, 0x075a, 0x00e0, 0x0110, 0x00f0, 0x0110, 0x0000,
        0x0ab4, 0x08da, 0x0ade, 0x0188, 0x01a0, 0x01b0, 0x01c0, 0x01d0, 0x01e0, 0x01f0, 0x0200,
        0x0120, 0x02a8, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000,
    ];
    Some(PARAMS[idx as usize])
}

pub(super) fn object_subtype2_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 64] = [
        0x0b66, 0x0b86, 0x0ba6, 0x0bc6, 0x0c66, 0x0c86, 0x0ca6, 0x0cc6, 0x0be6, 0x0c06, 0x0c26,
        0x0c46, 0x0ce6, 0x0d06, 0x0d26, 0x0d46, 0x0d66, 0x0d7e, 0x0d96, 0x0dae, 0x0dc6, 0x0dde,
        0x0df6, 0x0e0e, 0x0398, 0x03a0, 0x03a8, 0x03b0, 0x0e32, 0x0e26, 0x0ea2, 0x0e9a, 0x0eca,
        0x0ed2, 0x0ede, 0x0ede, 0x0f1e, 0x0f3e, 0x0f5e, 0x0f6a, 0x0ef6, 0x0f72, 0x0f92, 0x0fa2,
        0x0fa2, 0x1088, 0x10a8, 0x10a8, 0x10c8, 0x10c8, 0x10c8, 0x10c8, 0x0e52, 0x1108, 0x1108,
        0x12a8, 0x1148, 0x1160, 0x1178, 0x1190, 0x1458, 0x1488, 0x2062, 0x2086,
    ];
    PARAMS.get(idx as usize).copied()
}

pub(super) fn object_subtype3_param(idx: u8) -> Option<usize> {
    const PARAMS: [usize; 128] = [
        0x1614, 0x162c, 0x1654, 0x0a0e, 0x0a0c, 0x09fc, 0x09fe, 0x0a00, 0x0a02, 0x0a04, 0x0a06,
        0x0a08, 0x0a0a, 0x0000, 0x0a10, 0x0a12, 0x1dda, 0x1de2, 0x1dd6, 0x1dea, 0x15fc, 0x1dfa,
        0x1df2, 0x1488, 0x1494, 0x149c, 0x14a4, 0x10e8, 0x10e8, 0x10e8, 0x11a8, 0x11c8, 0x11e8,
        0x1208, 0x03b8, 0x03c0, 0x03c8, 0x03d0, 0x1228, 0x1248, 0x1268, 0x1288, 0x0000, 0x0e5a,
        0x0e62, 0x0000, 0x0000, 0x0e82, 0x0e8a, 0x14ac, 0x14c4, 0x10e8, 0x1614, 0x1614, 0x1614,
        0x1614, 0x1614, 0x1614, 0x1cbe, 0x1cee, 0x1d1e, 0x1d4e, 0x1d8e, 0x1d96, 0x1d9e, 0x1da6,
        0x1dae, 0x1db6, 0x1dbe, 0x1dc6, 0x1dce, 0x0220, 0x0260, 0x0280, 0x1f3a, 0x1f62, 0x1f92,
        0x1ff2, 0x2016, 0x1f42, 0x0eaa, 0x1f4a, 0x1f52, 0x1f5a, 0x202e, 0x2062, 0x09b8, 0x09c0,
        0x09c8, 0x09d0, 0x0fa2, 0x0fb2, 0x0fc4, 0x0ff4, 0x1018, 0x1020, 0x15b4, 0x15d8, 0x20f6,
        0x0eba, 0x22e6, 0x22ee, 0x05da, 0x281e, 0x2ae0, 0x2d2a, 0x2f2a, 0x22f6, 0x2316, 0x232e,
        0x2346, 0x235e, 0x2376, 0x23b6, 0x1e9a, 0x0000, 0x2436, 0x149c, 0x24b6, 0x24e6, 0x2516,
        0x1028, 0x1040, 0x1060, 0x1070, 0x1078, 0x1080, 0x0000,
    ];
    PARAMS.get(idx as usize).copied()
}

