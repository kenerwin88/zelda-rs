use super::*;

#[test]
fn initial_scroll_resumes_without_replaying_stripes_or_coordinate_tails() {
    let path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(path) else { return };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for index in 0..0x3000 {
        write_le_u16(&mut base.ram, 0x2000 + index * 2, ((index * 7) & 0xff) as u16);
    }
    for small in [false, true] {
        let screen = (0..64).find(|&screen| (base.asset_u8(107, screen) != 0) == small).unwrap();
        for direction in [1, 2, 4, 8] {
            let mut initial = base.clone();
            initial.set_overworld_screen_word(screen as u16);
            initial.set_screen_transition_direction_bits(direction);
            initial.set_overworld_map16_src_off(0x0440);
            initial.set_overworld_map16_dst_off(15);
            initial.set_overworld_map16_y_unit(17);
            let mut expected = initial.clone();
            expected.reference_CreateInitialNewScreenMapToScroll();
            let mut probe = initial.clone();
            let total = probe.begin_initial_scroll_cursor().total_steps();
            for cut in [0, 1, 31, 32, 33, 34, 97, 98, 161, 162, 163, 164, 165, 166, 167, total - 2, total - 1, total] {
                let mut resumed = initial.clone();
                let mut cursor = resumed.begin_initial_scroll_cursor();
                let pending = resumed.ram[0x1102..0x1950].to_vec();
                resumed.advance_initial_scroll_cursor(&mut cursor, cut);
                if cut <= 32 {
                    assert_eq!(&resumed.ram[0x1102..0x1950], &pending,
                        "stripe source stores must not publish map8 output");
                }
                let once = resumed.ram.clone();
                resumed.advance_initial_scroll_cursor(&mut cursor, cut);
                assert_eq!(resumed.ram, once);
                resumed.native_initial_scroll = Some(cursor);
                resumed.CreateInitialNewScreenMapToScroll();
                assert!(resumed.native_initial_scroll.is_none());
                assert_eq!(resumed.ram, expected.ram, "small={small} dir={direction} cut={cut}");
                assert_eq!(resumed.game_state, expected.game_state);
            }
            let mut completed = initial.clone();
            let mut cursor = completed.begin_initial_scroll_cursor();
            completed.advance_initial_scroll_cursor(&mut cursor, total);
            completed.set_overworld_map16_stripe_word(0, 0x1234);
            completed.write_overworld_vram_word(1, 0x9876);
            completed.set_overworld_map16_src_off(0x4321);
            let once = completed.ram.clone();
            completed.native_initial_scroll = Some(cursor);
            completed.CreateInitialNewScreenMapToScroll();
            assert_eq!(completed.ram, once);
            for horizontal in [true, false] {
                let mut actual = initial.clone();
                let mut frozen = initial.clone();
                let (a, b) = if horizontal {
                    (actual.BufferAndBuildMap16Stripes_X(7), frozen.reference_BufferAndBuildMap16Stripes_X(7))
                } else {
                    (actual.BufferAndBuildMap16Stripes_Y(7), frozen.reference_BufferAndBuildMap16Stripes_Y(7))
                };
                assert_eq!(a, b);
                assert_eq!(actual.ram, frozen.ram);
                assert_eq!(actual.game_state, frozen.game_state);
            }
        }
    }
}

#[test]
fn quadrant_publication_resumes_each_word_without_redrawing_completed_tiles() {
    let path = std::env::var_os("ZELDA3_ASSETS").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat"));
    let Ok(assets) = std::fs::read(path) else { return };
    let mut base = ZeldaState::new();
    base.set_assets(&assets).unwrap();
    for screen in [0, 0x18, 0x40] {
        base.set_overworld_screen_word(screen);
        for offset in (0..0x2000).step_by(2) {
            base.dungeon_room_tilemaps_mut().set_bg2_tile_by_byte_pos(offset, 0xeeee);
        }
        let mut expected = base.clone();
        // The independent bulk path predates the resumable four-quadrant caller.
        for (dst, area) in [(0x2000, 0), (0x2040, 1), (0x3000, 8), (0x3040, 9)] {
            expected.Overworld_DecompressAndDrawOneQuadrant(dst, i32::from(screen) + area);
        }
        for cut in [0, 1, 2, 3, 4, 63, 1023, 1024, 1025, 2047, 3073, 4095, 4096] {
            let mut resumed = base.clone();
            let mut cursor = resumed.begin_overworld_quadrant_cursor();
            resumed.advance_overworld_quadrant_cursor(&mut cursor, cut);
            let mut prefix = base.ram[0x2000..0x4000].to_vec();
            for store in 0..usize::from(cut) {
                let quadrant = store / 1024;
                let tile = (store % 1024) / 4;
                let pos = [0, 0x40, 0x1000, 0x1040][quadrant]
                    + (tile / 16) * 256 + (tile % 16) * 4 + [0, 128, 2, 130][store % 4];
                prefix[pos..pos + 2].copy_from_slice(&expected.ram[0x2000 + pos..0x2002 + pos]);
            }
            assert_eq!(&resumed.ram[0x2000..0x4000], &prefix, "screen={screen} cut={cut}");
            let once = resumed.ram.clone();
            resumed.advance_overworld_quadrant_cursor(&mut cursor, cut);
            assert_eq!(resumed.ram, once);
            resumed.native_overworld_quadrants = Some(cursor);
            resumed.Overworld_DecompressAndDrawAllQuadrants();
            assert!(resumed.native_overworld_quadrants.is_none());
            assert_eq!(resumed.ram, expected.ram, "screen={screen} cut={cut}");
            assert_eq!(resumed.game_state, expected.game_state);
        }
        let mut cursor = expected.begin_overworld_quadrant_cursor();
        expected.advance_overworld_quadrant_cursor(&mut cursor, 4096);
        expected.dungeon_room_tilemaps_mut().set_bg2_tile_by_byte_pos(0, 0x1234);
        expected.ram[0x14400] = 0x79;
        let completed = expected.ram.clone();
        expected.native_overworld_quadrants = Some(cursor);
        expected.Overworld_DecompressAndDrawAllQuadrants();
        assert_eq!(expected.ram, completed);
    }
}

#[test]
fn pre_overworld_music_selection_preserves_f2_goto_setsong_path() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0xf2, 2, 0, 0x40, 1);

    assert_eq!(xt, 0xf3);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn pre_overworld_music_selection_applies_darkworld_override_without_f2() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0x09, 2, 0, 0x40, 1);

    assert_eq!(xt, 9);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn turtle_rock_vram_common_terminates_nmi_upload_data() {
    let mut state = ZeldaState::new();
    state.write_vram_upload_buffer_byte(6, 0);
    state.ram[UVRAM_DATA_OVERWORLD + 6] = 0;

    state.turtle_rock_vram_common(0x10);

    assert_eq!(state.vram_upload_buffer_byte(6), 0xff);
    assert_eq!(state.ram[UVRAM_DATA_OVERWORLD + 6], 0);
}

#[test]
fn map8_conversion_resumes_at_stripe_header_plane_and_row_tail_boundaries() {
    use crate::zelda_rtl::overworld::OVERWORLD_MAP8_TOTAL_STEPS;
    let mut base = ZeldaState::new();
    let mut ranges = vec![(0, 0); 71];
    let table: Vec<u8> = (0..0x4000u16).flat_map(|i| (i ^ 0x4321).to_le_bytes()).collect();
    ranges[70] = (0, table.len());
    base.assets = Some(AssetPack::from_data_ranges(table.clone(), ranges));
    for i in 0..0x2000 {
        write_le_u16(&mut base.ram, 0x2000 + i * 2, ((i * 7) & 0xfff) as u16);
    }
    base.ram[0x12000..0x14080].fill(0xa5);
    for (page, page_base, upload_base) in [
        (OverworldMap16SourcePage::Main, 0x2000, 0),
        (OverworldMap16SourcePage::Overlay, 0x4000, 0x1000),
    ] {
        for (source, destination, y_unit) in [(0x003eu16, 31u16, 15u16), (0x1f80, 0, 31), (0xfff0, 63, 0)] {
            let mut initial = base.clone();
            initial.set_overworld_map16_src_off(source);
            initial.set_overworld_map16_dst_off(destination);
            initial.set_overworld_map16_y_unit(y_unit);
            // Frozen bulk specification: derive each stripe from the input
            // page and expand all four quarters without using the new cursor.
            let mut expected = initial.clone();
            let mut src = source.wrapping_add(0x1000);
            let mut y = y_unit;
            for row in 0..32 {
                for i in 0..32 {
                    let offset = src.wrapping_sub(0x410).wrapping_add(i * 2) & 0x1fff;
                    let value = read_le_u16(&initial.ram, page_base + usize::from(offset));
                    expected.set_overworld_map16_stripe_word(usize::from((destination + i) & 31), value);
                }
                for half in 0..2 {
                    let address = ((y & 15) * 64 + if y >= 16 { 0x800 } else { 0 }) + half * 0x400;
                    expected.write_vram_upload_absolute_word(0x14000 + row * 4 + usize::from(half) * 2, address | upload_base);
                    for tile in 0..16 {
                        let k = expected.game_state.world.transient.overworld_map16_stripe_word(usize::from(half) * 16 + tile);
                        for (quarter, offset) in [(0, 0), (1, 2), (2, 64), (3, 66)] {
                            let value = read_le_u16(&table, usize::from(k) * 8 + quarter * 2);
                            expected.dungeon_bg2_attributes_mut().import_aliased_map8_word(
                                row * 256 + usize::from(half) * 128 + tile * 4 + offset, value,
                            );
                        }
                    }
                }
                src = src.wrapping_sub(0x80);
                y = y.wrapping_sub(1) & 31;
            }
            expected.set_overworld_map16_src_off(src);
            expected.set_overworld_map16_y_unit(y);
            for cut in [0, 1, 31, 32, 33, 34, 96, 97, 98, 161, 162, 163, 164, 5000, OVERWORLD_MAP8_TOTAL_STEPS] {
                let mut resumed = initial.clone();
                let mut cursor = resumed.begin_overworld_map8_cursor(page, upload_base);
                resumed.advance_overworld_map8_cursor(&mut cursor, cut);
                if cut <= 32 {
                    assert_eq!(&resumed.ram[0x12000..0x14080], &initial.ram[0x12000..0x14080]);
                }
                let once = resumed.ram.clone();
                resumed.advance_overworld_map8_cursor(&mut cursor, cut);
                assert_eq!(resumed.ram, once);
                let mut expected = expected.clone();
                if cut == OVERWORLD_MAP8_TOTAL_STEPS {
                    // The next owner may reuse completed output before this
                    // caller retires. Completion must not redraw that prefix.
                    resumed.set_overworld_map16_stripe_word(0, 0xffff);
                    expected.set_overworld_map16_stripe_word(0, 0xffff);
                    resumed.ram[0x12000] = 0xf3;
                    expected.ram[0x12000] = 0xf3;
                }
                resumed.native_overworld_map8 = Some(cursor);
                resumed.Map16ToMap8(page, i32::from(upload_base));
                assert!(resumed.native_overworld_map8.is_none());
                assert_eq!(resumed.ram, expected.ram, "page={page:?} cut={cut}");
                assert_eq!(resumed.game_state, expected.game_state, "page={page:?} cut={cut}");
            }
        }
    }
}

// Frozen bulk translation used to verify resumable scroll publication.
impl ZeldaState {
    pub(super) fn reference_BufferAndBuildMap16Stripes_X(&mut self, mut dst: usize) -> usize {
        let strip = OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION
            [((self.screen_transition_direction_bits() >> 1) & 1) as usize];
        let mut pos = self.overworld_map16_src_off().wrapping_sub(strip);
        let mut y_unit_index = self.overworld_map16_y_unit() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_overworld_map16_stripe_word(y_unit_index, tile);
            y_unit_index = (y_unit_index + 1) & 0x1f;
            pos = pos.wrapping_add(0x80);
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut dst_unit = self.overworld_map16_dst_off();
        if dst_unit >= 0x10 {
            dst_unit &= 0x0f;
            r0 = 0x400;
        }
        r0 = r0.wrapping_add(dst_unit.wrapping_mul(2));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            self.write_overworld_vram_word(dst + 33, r0.wrapping_add(1));
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .overworld_map16_stripe_word(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 33, s1);
                self.write_overworld_vram_word(dst + 1, s2);
                self.write_overworld_vram_word(dst + 34, s3);
                dst += 2;
            }
            dst += 33;
            r0 = r0.wrapping_add(0x800);
        }
        dst
    }

    pub(super) fn reference_BufferAndBuildMap16Stripes_Y(&mut self, mut dst: usize) -> usize {
        let strip_index = 1 + ((self.screen_transition_direction_bits() >> 2) & 1) as usize;
        let mut pos = self
            .overworld_map16_src_off()
            .wrapping_sub(OVERWORLD_MAP16_STRIP_BACKTRACK_BY_DIRECTION[strip_index]);
        let mut dst_unit_index = self.overworld_map16_dst_off() as usize & 0x1f;
        for _ in 0..32 {
            let tile = self.overworld_map16_stripe_source_word(pos);
            self.set_overworld_map16_stripe_word(dst_unit_index, tile);
            pos = pos.wrapping_add(2);
            dst_unit_index = (dst_unit_index + 1) & 0x1f;
        }

        let map8 = self.GetMap16toMap8Table();
        let mut r0 = 0u16;
        let mut y_unit = self.overworld_map16_y_unit();
        if y_unit >= 0x10 {
            y_unit &= 0x0f;
            r0 = 0x800;
        }
        r0 = r0.wrapping_add(y_unit.wrapping_mul(64));

        let mut tmp = 0usize;
        for _ in 0..2 {
            self.write_overworld_vram_word(dst, r0);
            dst += 1;
            for _ in 0..16 {
                let k = self
                    .game_state
                    .world
                    .transient
                    .overworld_map16_stripe_word(tmp);
                tmp += 1;
                let s0 = self.overworld_map16_to_map8_word(&map8, k, 0);
                let s1 = self.overworld_map16_to_map8_word(&map8, k, 1);
                let s2 = self.overworld_map16_to_map8_word(&map8, k, 2);
                let s3 = self.overworld_map16_to_map8_word(&map8, k, 3);
                self.write_overworld_vram_word(dst, s0);
                self.write_overworld_vram_word(dst + 32, s2);
                self.write_overworld_vram_word(dst + 1, s1);
                self.write_overworld_vram_word(dst + 33, s3);
                dst += 2;
            }
            dst += 32;
            r0 = r0.wrapping_add(0x400);
        }
        dst
    }

    pub(super) fn reference_TriggerAndFinishMapLoadStripe_Y(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(8);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x0080);
        dst += 1;
        while n != 0 {
            dst = self.reference_BufferAndBuildMap16Stripes_Y(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(0x80);
            self.set_overworld_map16_src_off(src);
            let y_unit = self.overworld_map16_y_unit().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_y_unit(y_unit);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn reference_TriggerAndFinishMapLoadStripe_X(&mut self, mut n: i32) {
        self.set_screen_transition_direction_bits(2);
        self.set_pending_nmi_subroutine(3);
        let mut dst = 0usize;
        self.write_overworld_vram_word(dst, 0x8040);
        dst += 1;
        while n != 0 {
            dst = self.reference_BufferAndBuildMap16Stripes_X(dst);
            let src = self.overworld_map16_src_off().wrapping_sub(2);
            self.set_overworld_map16_src_off(src);
            let off = self.overworld_map16_dst_off().wrapping_sub(1) & 0x1f;
            self.set_overworld_map16_dst_off(off);
            n -= 1;
        }
        self.write_overworld_vram_word(dst, 0xffff);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_North(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(0x380);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_y_unit(31);
        self.reference_TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_South(&mut self) {
        let mut pos = self.overworld_map16_src_off();
        while pos >= 0x80 {
            pos = pos.wrapping_sub(0x80);
        }
        self.set_overworld_map16_src_off(pos.wrapping_add(0x780));
        self.set_overworld_map16_y_unit(7);
        self.reference_TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_West(&mut self) {
        let src = self.overworld_map16_src_off().wrapping_add(14);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(31);
        self.reference_TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Big_East(&mut self) {
        let src = self
            .overworld_map16_src_off()
            .wrapping_sub(0x60)
            .wrapping_add(0x1e);
        self.set_overworld_map16_src_off(src);
        self.set_overworld_map16_dst_off(7);
        self.reference_TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_North(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x700),
            dst_off: map16.dst_off,
            y_unit: 10,
        });
        self.set_overworld_map16_src_off(0x1390);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(31);
        self.reference_TriggerAndFinishMapLoadStripe_Y(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_South(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src & 0xff,
            dst_off: map16.dst_off,
            y_unit: 24,
        });
        self.set_overworld_map16_src_off(0x0790);
        self.set_overworld_map16_dst_off(0);
        self.set_overworld_map16_y_unit(7);
        self.reference_TriggerAndFinishMapLoadStripe_Y(8);
        let y_unit = self.overworld_map16_y_unit().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_y_unit(y_unit);
        let src = self.overworld_map16_src_off().wrapping_sub(0x0b80);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_West(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x20),
            dst_off: 8,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x044e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(31);
        self.reference_TriggerAndFinishMapLoadStripe_X(7);
    }

    pub(super) fn reference_CreateInitialOWScreenView_Small_East(&mut self) {
        let src = self.overworld_map16_src_off();
        let map16 = self.game_state.world.overworld.map16.active_load;
        self.store_small_overworld_map16_scroll_backup(SmallOverworldMap16ScrollBackupState {
            src_off: src.wrapping_sub(0x60),
            dst_off: 0x18,
            y_unit: map16.y_unit,
        });
        self.set_overworld_map16_src_off(0x041e);
        self.set_overworld_map16_y_unit(0);
        self.set_overworld_map16_dst_off(7);
        self.reference_TriggerAndFinishMapLoadStripe_X(8);
        let off = self.overworld_map16_dst_off().wrapping_add(9) & 0x1f;
        self.set_overworld_map16_dst_off(off);
        let src = self.overworld_map16_src_off().wrapping_sub(0x2e);
        self.set_overworld_map16_src_off(src);
    }

    pub(super) fn reference_CreateInitialNewScreenMapToScroll(&mut self) {
        let dir = self.screen_transition_direction_bits();
        if self.overworld_map_is_small() {
            match dir {
                1 => self.reference_CreateInitialOWScreenView_Small_East(),
                2 => self.reference_CreateInitialOWScreenView_Small_West(),
                4 => self.reference_CreateInitialOWScreenView_Small_South(),
                8 => self.reference_CreateInitialOWScreenView_Small_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "reference_CreateInitialNewScreenMapToScroll small invalid direction {}",
                        dir
                    );
                }
            }
        } else {
            match dir {
                1 => self.reference_CreateInitialOWScreenView_Big_East(),
                2 => self.reference_CreateInitialOWScreenView_Big_West(),
                4 => self.reference_CreateInitialOWScreenView_Big_South(),
                8 => self.reference_CreateInitialOWScreenView_Big_North(),
                _ => {
                    self.set_submodule(0);
                    panic!(
                        "reference_CreateInitialNewScreenMapToScroll big invalid direction {}",
                        dir
                    );
                }
            }
        }
    }

}
