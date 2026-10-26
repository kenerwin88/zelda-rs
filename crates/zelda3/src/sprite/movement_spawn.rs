//! Sprite properties, movement, dynamic spawning, terrain, messages, and room-clear helpers.

use super::*;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn sprite_where_in_room_mask(&self, room: u16) -> u16 {
        self.game_state
            .sprites
            .workspace
            .where_in_room(usize::from(room))
    }

    pub(in crate::zelda_rtl) fn set_sprite_where_in_room_mask(&mut self, room: u16, mask: u16) {
        self.sprite_workspace_mut()
            .set_where_in_room(usize::from(room), mask);
    }

    pub(in crate::zelda_rtl) fn prepare_apply_rumble_to_sprites(&mut self) {
        let j = self.game_state.player.follower_link.facing_index();
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_X[j] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_Y[j] as i16 as u16);
        let mut hb = SpriteHitBox {
            r0_xlo: x as u8,
            r1_ylo: y as u8,
            r2: PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_WH[j],
            r3: PREPARE_APPLY_RUMBLE_TO_SPRITES_APPLY_RUMBLE_WH[j + 2],
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
            r8_xhi: (x >> 8) as u8,
            r9_yhi: (y >> 8) as u8,
        };
        self.entity_apply_rumble_to_sprites(&mut hb);
    }

    // void Sprite_SpawnImmediatelySmashedTerrain(uint8 what, uint16 x, uint16 y) {  // 86812d
    //   uint8 bak1 = flag_is_sprite_to_pick_up;
    //   uint8 bak2 = sprite_pickup_slot_cache;
    //   int k = Sprite_SpawnThrowableTerrain_silently(what, x, y);
    //   if (k >= 0)
    //     ThrowableScenery_TransmuteToDebris(k);
    //   sprite_pickup_slot_cache = bak2;
    //   flag_is_sprite_to_pick_up = bak1;
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_immediately_smashed_terrain(&mut self, what: u8, x: u16, y: u16) {
        let bak1 = self.game_state.player.follower_link.sprite_pickup_flag();
        let bak2 = self.game_state.sprites.workspace.pickup_slot_cache();
        let k = self.sprite_spawn_throwable_terrain_silently(what, x, y);
        if k >= 0 {
            self.throwable_scenery_transmute_to_debris(k as usize);
        }
        self.sprite_workspace_mut().set_pickup_slot_cache(bak2);
        self.follower_link_state_mut().set_sprite_pickup_flag(bak1);
    }

    // void Sprite_SpawnThrowableTerrain(uint8 what, uint16 x, uint16 y) {  // 86814b
    //   sound_effect_1 = Link_CalculateSfxPan() | 29;
    //   Sprite_SpawnThrowableTerrain_silently(what, x, y);
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_throwable_terrain(&mut self, what: u8, x: u16, y: u16) {
        self.set_sound_effect_1_with_link_pan(29);
        self.sprite_spawn_throwable_terrain_silently(what, x, y);
    }

    // int Sprite_SpawnThrowableTerrain_silently(uint8 what, uint16 x, uint16 y) {  // 868156
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_throwable_terrain_silently(
        &mut self,
        what: u8,
        x: u16,
        y: u16,
    ) -> i32 {
        let Some(k) = (0..16)
            .rev()
            .find(|&k| self.sprite_slot_view(k).state() == 0)
        else {
            return -1;
        };

        let value = 10;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0xec;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        self.sprite_prep_load_properties(k);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(k).set_floor(value);
        let value = what;
        self.sprite_slot_view_mut(k).set_c(value);
        if what >= 6 {
            let value = 0xa6;
            self.sprite_slot_view_mut(k).set_flags2(value);
        }

        let mut flags =
            SPRITE_SPAWN_THROWABLE_TERRAIN_SILENTLY_THROWABLE_SCENERY_OAM_FLAGS[what as usize];
        if what == 2 && self.game_state.world.location.is_indoors() {
            let value = 0x80;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            flags = 0x50;
        }
        let value = flags;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = 9;
        self.sprite_slot_view_mut(k).set_draw_work_byte_4(value);
        self.follower_link_state_mut().set_sprite_pickup_flag(2);
        self.sprite_workspace_mut().set_pickup_slot_cache(2);
        let value = 16;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(k).set_floor(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);

        if self.game_state.dungeon_secret.is_available() {
            if (self.game_state.dungeon_secret.pending_kind()
                | self.game_state.world.location.indoor_flag())
                == 0
                && self.sprite_slot_view(k).c().wrapping_sub(2) < 2
            {
                self.overworld_substitute_alternate_secret();
            }
            if let Some(value) = self.game_state.dungeon_secret.graphics_kind() {
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.dungeon_secret_scratch_mut().clear_pending_kind();
            }
            self.sprite_spawn_secret(k);
        }

        k as i32
    }

    // void Overworld_SubstituteAlternateSecret() {  // 9afbdb
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn overworld_substitute_alternate_secret(&mut self) {
        if self.get_random_number() & 1 != 0 {
            return;
        }

        let mut n = 0;
        for j in (0..16).rev() {
            if self.sprite_slot_view(j).state() != 0
                && self.sprite_slot_view(j).sprite_type() != 0x6c
            {
                n += 1;
            }
        }
        if n >= 4 || self.game_state.inventory.save_progress.progress_indicator() < 2 {
            return;
        }

        let j = ((self.game_state.dungeon_secret.overworld_subst_counter() & 7)
            + if self.game_state.world.region.is_in_dark_world() {
                8
            } else {
                0
            }) as usize;
        self.dungeon_secret_scratch_mut()
            .increment_overworld_subst_counter();
        let area = (self.game_state.world.region.overworld_area_low() & 0x3f) as usize;
        if OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_ITEMS[area]
            & OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_VERTICAL_OFFSETS[j]
            == 0
        {
            self.dungeon_secret_scratch_mut().set_pending_kind(
                OVERWORLD_SUBSTITUTE_ALTERNATE_SECRET_SECRET_SUBSTITUTION_HORIZONTAL_OFFSETS[j],
            );
        }
    }

    pub(in crate::zelda_rtl) fn sprite_nullify_hookshot_drag(&mut self) {
        for i in (0..5).rev() {
            if self.ancilla_slot_view(i).ancilla_type() & 0x1f == 0
                && self
                    .game_state
                    .player
                    .follower_link
                    .has_hookshot_interlock()
            {
                self.follower_link_state_mut().clear_hookshot_interlock();
                break;
            }
        }
        self.follower_link_state_mut()
            .cache_safe_return_high_from_current();
        self.follower_link_state_mut()
            .restore_position_from_previous();
        self.handle_indoor_camera_and_doors();
    }

    pub(in crate::zelda_rtl) fn sprite_prep_reset_properties(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).clear_prep_runtime_state();
    }

    pub(in crate::zelda_rtl) fn sprite_prep_reset_properties_prefix(&mut self, k: usize, completed_stores: u8) {
        self.sprite_slot_view_mut(k)
            .clear_prep_runtime_state_prefix(completed_stores);
    }

    pub(in crate::zelda_rtl) fn sprite_prep_reset_properties_from(&mut self, k: usize, completed_stores: u8) {
        self.sprite_slot_view_mut(k)
            .clear_prep_runtime_state_from(completed_stores);
    }

    /// ROM: `SpritePrep_LoadProperties`
    pub(in crate::zelda_rtl) fn sprite_prep_load_properties(&mut self, k: usize) {
        self.sprite_prep_reset_properties(k);
        self.sprite_prep_load_properties_after_reset(k);
    }

    pub(in crate::zelda_rtl) fn sprite_prep_load_properties_after_reset(&mut self, k: usize) {
        self.sprite_prep_load_properties_after_reset_prefix(k, 10);
    }

    pub(in crate::zelda_rtl) fn sprite_prep_load_properties_after_reset_prefix(
        &mut self,
        k: usize,
        completed_stores: u8,
    ) {
        assert!(completed_stores <= 10);
        for store in 0..completed_stores {
            self.sprite_prep_load_property_store(k, store);
        }
    }

    pub(in crate::zelda_rtl) fn sprite_prep_load_properties_after_reset_from(
        &mut self,
        k: usize,
        completed_stores: u8,
    ) {
        assert!(completed_stores <= 10);
        for store in completed_stores..10 {
            self.sprite_prep_load_property_store(k, store);
        }
    }

    fn sprite_prep_load_property_store(&mut self, k: usize, store: u8) {
        let ty = self.sprite_slot_view(k).sprite_type();
        match store {
            0 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS2_TABLE, ty);
                self.sprite_slot_view_mut(k).set_flags2(value);
            }
            1 => {
                let value = sprite_init_value(SPRITE_INIT_HEALTH_TABLE, ty);
                self.sprite_slot_view_mut(k).set_health(value);
            }
            2 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS4_TABLE, ty);
                self.sprite_slot_view_mut(k).set_flags4(value);
            }
            3 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS5_TABLE, ty);
                self.sprite_slot_view_mut(k).set_flags5(value);
            }
            4 => {
                let value = sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, ty);
                self.sprite_slot_view_mut(k).set_deflection_bits(value);
            }
            5 => {
                let value = sprite_init_value(SPRITE_INIT_BUMP_DAMAGE_TABLE, ty);
                self.sprite_slot_view_mut(k).set_bump_damage(value);
            }
            6 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS_TABLE, ty);
                self.sprite_slot_view_mut(k).set_flags(value);
            }
            7 => {
                let value = if self.game_state.world.location.is_indoors() {
                    self.game_state.dungeon.room_tracking.room_index2_word() as u8
                } else {
                    self.game_state.world.region.overworld_area() as u8
                };
                self.sprite_slot_view_mut(k).set_room(value);
            }
            8 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, ty);
                self.sprite_slot_view_mut(k).set_flags3(value);
            }
            9 => {
                let value = sprite_init_value(SPRITE_INIT_FLAGS3_TABLE, ty) & 0x0f;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            }
            _ => unreachable!("property-store cursor was validated above"),
        }
    }

    pub(in crate::zelda_rtl) fn sprite_prep_load_palette(&mut self, k: usize) {
        let flags3 = sprite_init_value(
            SPRITE_INIT_FLAGS3_TABLE,
            self.sprite_slot_view(k).sprite_type(),
        );
        let value = flags3;
        self.sprite_slot_view_mut(k).set_flags3(value);
        let value = flags3 & 0x0f;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    pub(in crate::zelda_rtl) fn ancilla_spawn_falling_prize(&mut self, item: u8) -> i32 {
        self.ancilla_add_falling_prize(0x29, item, 4)
    }

    /// ROM: `Sprite_SetX`
    pub(in crate::zelda_rtl) fn sprite_set_x(&mut self, k: usize, x: u16) {
        self.sprite_slot_view_mut(k).set_x(x);
    }

    /// ROM: `Sprite_SetY`
    pub(in crate::zelda_rtl) fn sprite_set_y(&mut self, k: usize, y: u16) {
        self.sprite_slot_view_mut(k).set_y(y);
    }

    // void SpriteAddXY(int k, int xv, int yv) {
    //   Sprite_SetX(k, Sprite_GetX(k) + xv);
    //   Sprite_SetY(k, Sprite_GetY(k) + yv);
    // }
    pub(in crate::zelda_rtl) fn sprite_add_xy(&mut self, k: usize, xv: i32, yv: i32) {
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(xv as i16 as u16));
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_add(yv as i16 as u16));
    }

    // void SpriteFall_AdjustPosition(int k) {  // 86e624
    //   SpriteAddXY(k, dung_floor_x_vel, dung_floor_y_vel);
    // }
    pub(in crate::zelda_rtl) fn sprite_fall_adjust_position(&mut self, k: usize) {
        self.sprite_add_xy(
            k,
            self.game_state.dungeon.moving_floor.floor_x_velocity() as i16 as i32,
            self.game_state.dungeon.moving_floor.floor_y_velocity() as i16 as i32,
        );
    }

    pub(in crate::zelda_rtl) fn sprite_get_x(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).x()
    }

    pub(in crate::zelda_rtl) fn sprite_get_y(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).y()
    }

    pub(in crate::zelda_rtl) fn sprite_is_right_of_location(&self, k: usize, x: u16) -> PairU8 {
        let xv = x.wrapping_sub(self.sprite_get_x(k));
        PairU8 {
            a: u8::from((xv as i16).is_negative()),
            b: xv as u8,
        }
    }

    pub(in crate::zelda_rtl) fn sprite_is_below_location(&self, k: usize, y: u16) -> PairU8 {
        let yv = y.wrapping_sub(self.sprite_get_y(k));
        PairU8 {
            a: u8::from((yv as i16).is_negative()),
            b: yv as u8,
        }
    }

    // uint8 Sprite_DirectionToFaceLocation(int k, uint16 x, uint16 y) {  // 86eb30
    //   PairU8 below = Sprite_IsBelowLocation(k, y);
    //   PairU8 right = Sprite_IsRightOfLocation(k, x);
    //   uint8 ym = sign8(below.b) ? -below.b : below.b;
    //   tmp_counter = ym;
    //   uint8 xm = sign8(right.b) ? -right.b : right.b;
    //   return (xm >= ym) ? right.a : below.a + 2;
    // }
    pub(in crate::zelda_rtl) fn sprite_direction_to_face_location(&mut self, k: usize, x: u16, y: u16) -> u8 {
        let below = self.sprite_is_below_location(k, y);
        let right = self.sprite_is_right_of_location(k, x);
        let ym = if sign8(below.b) {
            below.b.wrapping_neg()
        } else {
            below.b
        };
        self.temp_counter_mut().set(ym);
        let xm = if sign8(right.b) {
            right.b.wrapping_neg()
        } else {
            right.b
        };
        if xm >= ym {
            right.a
        } else {
            below.a + 2
        }
    }

    pub(in crate::zelda_rtl) fn sprite_is_right_of_link(&self, k: usize) -> PairU8 {
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.sprite_get_x(k));
        PairU8 {
            a: u8::from(sign16(x)),
            b: x as u8,
        }
    }

    pub(in crate::zelda_rtl) fn sprite_is_below_link(&self, k: usize) -> PairU8 {
        let link_y = self.game_state.player.follower_link.y();
        let t = (link_y as u8) as i32 + 8;
        let u = (t & 0xff) + self.sprite_slot_view(k).z() as i32;
        let v = (u & 0xff) - self.sprite_slot_view(k).y_low() as i32;
        let w = (link_y >> 8) as i32 - self.sprite_slot_view(k).y_high() as i32 - i32::from(v < 0);
        let y = ((w & 0xff) + (t >> 8) + (u >> 8)) as u8;
        PairU8 {
            a: u8::from((y as i8).is_negative()),
            b: v as u8,
        }
    }

    pub(in crate::zelda_rtl) fn sprite_project_speed_towards_link(&self, k: usize, vel: u8) -> ProjectSpeedRet {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06_e991));
        if vel == 0 {
            // $06:E991..E995 -> E98E..E990: STA, CMP, taken BEQ, STZ, RTS.
            if self.lag_free { crate::cycle_ledger::charge(56 + 6 + 66); }
            return ProjectSpeedRet::ZERO;
        }
        let below = self.sprite_is_below_link(k);
        let right = self.sprite_is_right_of_link(k);
        let result = project_speed_from_differences(vel, below, right);
        if self.lag_free {
            let ym = (below.b as i8).unsigned_abs();
            let xm = (right.b as i8).unsigned_abs();
            let swapped = xm < ym;
            // The unsigned minor result counts the SBC/INC paths actually
            // executed in $06:E9CC..E9DC, before axis swap and output signs.
            let (minor, negative) = if swapped { (result.x, right.a) } else { (result.y, below.a) };
            let subtractions = if negative != 0 { minor.wrapping_neg() } else { minor };
            crate::cycle_ledger::charge(
                56 + 154 + if below.b & 0x80 == 0 { 6 } else { 30 }
                + 134 + if right.b & 0x80 == 0 { 6 } else { 30 }
                + 104 + if swapped { 136 } else { 6 }
                + 72 // $E9C6..E9CA: accumulator/output/index initialization.
                // Each pass: 102 + taken BCC(6) + 54 + taken BNE(6).
                // A subtraction replaces taken BCC(6) with SBC/INC(62).
                + u64::from(vel) * 168 - 6 + u64::from(subtractions) * 56
                + 30 + if swapped { 146 } else { 6 } // $E9DE..E9E9.
                + 64 + if below.a != 0 { 54 } else { 6 } // $E9EB..E9F4.
                + 64 + if right.a != 0 { 54 } else { 6 } // $E9F6..E9FF.
                + 98, // $EA01..EA03: PLY, PLX, RTS. Callees priced separately.
            );
        }
        result
    }

    pub(in crate::zelda_rtl) fn sprite_project_speed_towards_location(
        &self,
        k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet::ZERO;
        }
        let below = self.sprite_is_below_location(k, y);
        let right = self.sprite_is_right_of_location(k, x);
        project_speed_from_differences(vel, below, right)
    }

    // void Sprite_ApproachTargetSpeed(int k, uint8 x, uint8 y) {
    //   if (sprite_x_vel[k] - x)
    //     sprite_x_vel[k] += sign8(sprite_x_vel[k] - x) ? 1 : -1;
    //   if (sprite_y_vel[k] - y)
    //     sprite_y_vel[k] += sign8(sprite_y_vel[k] - y) ? 1 : -1;
    // }
    pub(in crate::zelda_rtl) fn sprite_approach_target_speed(&mut self, k: usize, x: u8, y: u8) {
        let mut sprite = self.sprite_slot_view_mut(k);
        let x_diff = sprite.x_velocity().wrapping_sub(x);
        if x_diff != 0 {
            sprite.add_x_velocity(if sign8(x_diff) { 1 } else { 0xff });
        }
        let y_diff = sprite.y_velocity().wrapping_sub(y);
        if y_diff != 0 {
            sprite.add_y_velocity(if sign8(y_diff) { 1 } else { 0xff });
        }
    }

    pub(in crate::zelda_rtl) fn sprite_move_xy(&mut self, k: usize) {
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    /// Execute only `Sprite_MoveX`'s first source assignment and return the
    /// already-computed coordinate bytes for an exact host-boundary resume.
    pub(in crate::zelda_rtl) fn sprite_move_x_through_subpixel(&mut self, k: usize) -> (u8, u8) {
        let sprite = self.sprite_slot_view(k);
        let velocity = sprite.x_velocity();
        let position = u32::from(sprite.x_subpixel())
            | (u32::from(sprite.x_low()) << 8)
            | (u32::from(sprite.x_high()) << 16);
        let delta = ((velocity as i8 as i32) << 4) as u32;
        let moved = position.wrapping_add(delta);
        self.sprite_slot_view_mut(k).set_x_subpixel(moved as u8);
        ((moved >> 8) as u8, (moved >> 16) as u8)
    }

    /// Runs `Sprite_MoveXY` through `checkpoint` on the interrupted host and
    /// returns the coordinate bytes the resumed call still has to publish.
    pub(in crate::zelda_rtl) fn sprite_move_xy_through_checkpoint(
        &mut self,
        k: usize,
        checkpoint: crate::SpriteMoveXYCheckpoint,
    ) -> SpriteMoveXYContinuation {
        let mut continuation = SpriteMoveXYContinuation::default();
        match checkpoint {
            crate::SpriteMoveXYCheckpoint::BeforeMovement => {}
            crate::SpriteMoveXYCheckpoint::AfterXSubpixel => {
                (continuation.x_low, continuation.x_high) = self.sprite_move_x_through_subpixel(k);
            }
            crate::SpriteMoveXYCheckpoint::AfterXLow => {
                (continuation.x_low, continuation.x_high) = self.sprite_move_x_through_subpixel(k);
                self.sprite_slot_view_mut(k).set_x_low(continuation.x_low);
            }
            crate::SpriteMoveXYCheckpoint::AfterXHigh => self.sprite_move_x(k),
            crate::SpriteMoveXYCheckpoint::AfterYSubpixel => {
                (continuation.y_low, continuation.y_high) =
                    self.sprite_move_xy_through_y_subpixel(k);
            }
            crate::SpriteMoveXYCheckpoint::AfterYLow => {
                (continuation.y_low, continuation.y_high) =
                    self.sprite_move_xy_through_y_subpixel(k);
                self.sprite_slot_view_mut(k).set_y_low(continuation.y_low);
            }
            crate::SpriteMoveXYCheckpoint::AfterYHigh => self.sprite_move_xy(k),
        }
        continuation
    }

    /// Finishes `Sprite_MoveXY` from `checkpoint` with the bytes bound by
    /// `sprite_move_xy_through_checkpoint`.
    pub(in crate::zelda_rtl) fn sprite_move_xy_from_checkpoint(
        &mut self,
        k: usize,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: SpriteMoveXYContinuation,
    ) {
        match checkpoint {
            crate::SpriteMoveXYCheckpoint::BeforeMovement => self.sprite_move_xy(k),
            crate::SpriteMoveXYCheckpoint::AfterXSubpixel => {
                self.complete_sprite_move_x_after_subpixel(
                    k,
                    continuation.x_low,
                    continuation.x_high,
                );
                self.sprite_move_y(k);
            }
            crate::SpriteMoveXYCheckpoint::AfterXLow => {
                self.sprite_slot_view_mut(k).set_x_high(continuation.x_high);
                self.sprite_move_y(k);
            }
            crate::SpriteMoveXYCheckpoint::AfterXHigh => self.sprite_move_y(k),
            crate::SpriteMoveXYCheckpoint::AfterYSubpixel => self
                .complete_sprite_move_y_after_subpixel(k, continuation.y_low, continuation.y_high),
            crate::SpriteMoveXYCheckpoint::AfterYLow => {
                self.sprite_slot_view_mut(k).set_y_high(continuation.y_high)
            }
            crate::SpriteMoveXYCheckpoint::AfterYHigh => {}
        }
    }

    pub(in crate::zelda_rtl) fn complete_sprite_move_x_after_subpixel(
        &mut self,
        k: usize,
        x_low: u8,
        x_high: u8,
    ) {
        self.sprite_slot_view_mut(k).set_x_low(x_low);
        self.sprite_slot_view_mut(k).set_x_high(x_high);
    }

    /// Execute `Sprite_MoveXY` through the first assignment in
    /// `Sprite_MoveY`. The original C helper publishes X completely, then
    /// writes Y subpixel, low, and high in that order. Returning the already
    /// computed low/high bytes lets a host-boundary continuation finish those
    /// assignments without evaluating the velocity twice.
    pub(in crate::zelda_rtl) fn sprite_move_xy_through_y_subpixel(&mut self, k: usize) -> (u8, u8) {
        self.sprite_move_x(k);
        self.sprite_move_y_through_subpixel(k)
    }

    /// Execute `Sprite_MoveY` through its first source assignment. This is the
    /// exact suffix of `Sprite_MoveXY` after `Sprite_MoveX` has returned.
    pub(in crate::zelda_rtl) fn sprite_move_y_through_subpixel(&mut self, k: usize) -> (u8, u8) {
        let sprite = self.sprite_slot_view(k);
        let velocity = sprite.y_velocity();
        assert_ne!(
            velocity, 0,
            "a Y-subpixel publication boundary requires Sprite_MoveY to enter its source body",
        );
        let position = u32::from(sprite.y_subpixel())
            | (u32::from(sprite.y_low()) << 8)
            | (u32::from(sprite.y_high()) << 16);
        let delta = ((velocity as i8 as i32) << 4) as u32;
        let moved = position.wrapping_add(delta);
        self.sprite_slot_view_mut(k).set_y_subpixel(moved as u8);
        ((moved >> 8) as u8, (moved >> 16) as u8)
    }

    /// Finish the two pending assignments from
    /// [`Self::sprite_move_xy_through_y_subpixel`].
    pub(in crate::zelda_rtl) fn complete_sprite_move_y_after_subpixel(
        &mut self,
        k: usize,
        y_low: u8,
        y_high: u8,
    ) {
        self.sprite_slot_view_mut(k).set_y_low(y_low);
        self.sprite_slot_view_mut(k).set_y_high(y_high);
    }

    // void Sprite_MoveXYZ(int k) {
    //   Sprite_MoveZ(k);
    //   Sprite_MoveX(k);
    //   Sprite_MoveY(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_move_xyz(&mut self, k: usize) {
        self.sprite_move_z(k);
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    pub(in crate::zelda_rtl) fn sprite_move_x(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_x();
    }

    pub(in crate::zelda_rtl) fn sprite_move_y(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_y();
    }

    // ----- Common sprite-AI helpers (round-3 agent) -----
    // Direct 1:1 ports of the small Sprite_* helpers that are widely shared by
    // sprite-AI handlers. Bodies preserved verbatim modulo Rust-isms.

    // static int AllocOverlord() {
    //   int i = 7;
    //   while (i >= 0 && overlord_type[i] != 0)
    //     i--;
    //   return i;
    // }
    pub(in crate::zelda_rtl) fn alloc_overlord(&self) -> i32 {
        let mut i = 7i32;
        while i >= 0 && self.overlord_slot_view(i as usize).overlord_type() != 0 {
            i -= 1;
        }
        i
    }

    // static int Overworld_AllocSprite(uint8 type) {
    //   int i = (type == 0x58) ? 4 :
    //           (type == 0xd0) ? 5 :
    //           (type == 0xeb || type == 0x53 || type  == 0xf3) ? 14 : 13;
    //   for (; i >= 0; i--) {
    //     if (sprite_state[i] == 0 || sprite_type[i] == 0x41 && sprite_C[i] != 0)
    //       break;
    //   }
    //   return i;
    // }
    pub(in crate::zelda_rtl) fn overworld_alloc_sprite(&self, type_: u8) -> i32 {
        let mut i = if type_ == 0x58 {
            4
        } else if type_ == 0xd0 {
            5
        } else if type_ == 0xeb || type_ == 0x53 || type_ == 0xf3 {
            14
        } else {
            13
        };
        while i >= 0 {
            let k = i as usize;
            if self.sprite_slot_view(k).state() == 0
                || (self.sprite_slot_view(k).sprite_type() == 0x41
                    && self.sprite_slot_view(k).c() != 0)
            {
                break;
            }
            i -= 1;
        }
        i
    }

    // bool Sprite_ReturnIfLifted(int k) — sprite.c:2602
    pub(in crate::zelda_rtl) fn sprite_return_if_lifted(&mut self, k: usize) -> bool {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06_f228));
        // $06:F228..F22F: three loads/ORs and the first branch.
        if self.lag_free { crate::cycle_ledger::charge(96); }
        if self.game_state.frame.submodule != 0
            || self.game_state.player.follower_link.button_b_frames() != 0
            || self.game_state.frame.modal_pause_flag != 0
        {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); }
            return false;
        }
        // $06:F231..F236: floor comparison, then the actual descending scan.
        if self.lag_free { crate::cycle_ledger::charge(72); }
        if self.sprite_slot_view(k).floor()
            != self.game_state.player.follower_link.lower_level_state() {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); }
            return false;
        }
        if self.lag_free { crate::cycle_ledger::charge(16); } // $F238 LDY.
        for j in (0..=15usize).rev() {
            // $06:F23A..F242; the hit returns without executing DEY/BPL.
            if self.lag_free { crate::cycle_ledger::charge(64); }
            if self.sprite_slot_view(j).state() == 10 {
                if self.lag_free { crate::cycle_ledger::charge(6 + 42); }
                return false;
            }
            if self.lag_free { crate::cycle_ledger::charge(30 + if j != 0 { 6 } else { 0 }); }
        }
        // $06:F244..F255: the two exempt types skip the velocity test.
        if self.lag_free {
            let kind = self.sprite_slot_view(k).sprite_type();
            crate::cycle_ledger::charge(if kind == 0x0b { 64 + 6 }
                else if kind == 0x4a { 64 + 32 + 6 } else { 64 + 32 + 80 });
        }
        if self.sprite_slot_view(k).sprite_type() != 0xb
            && self.sprite_slot_view(k).sprite_type() != 0x4a
            && (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity()) != 0
        {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); }
            return false;
        }
        self.sprite_return_if_lifted_body(k)
    }

    // bool Sprite_ReturnIfLiftedPermissive(int k) — sprite.c:2615
    pub(in crate::zelda_rtl) fn sprite_return_if_lifted_permissive(&mut self, k: usize) -> bool {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x06_f257));
        self.sprite_return_if_lifted_body(k)
    }

    fn sprite_return_if_lifted_body(&mut self, k: usize) -> bool {
        // $06:F257..F263 is a fallthrough of F228, or a direct F257 call.
        if self.lag_free { crate::cycle_ledger::charge(48); }
        if self.game_state.player.follower_link.is_running() {
            if self.lag_free { crate::cycle_ledger::charge(6 + 42); }
            return false;
        }
        if self.lag_free { crate::cycle_ledger::charge(94); }
        if self
            .game_state
            .player
            .follower_link
            .sprite_pickup_flag_cached()
            .wrapping_sub(1)
            != self.game_state.sprites.system.cur_object_index()
        {
            // $06:F265..F26E: three JSRs and BCC; children own their bodies.
            if self.lag_free { crate::cycle_ledger::charge(154); }
            let mut hb = SpriteHitBox {
                r0_xlo: 0,
                r8_xhi: 0,
                r1_ylo: 0,
                r9_yhi: 0,
                r2: 0,
                r3: 0,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.link_setup_hit_box_conditional(&mut hb);
            self.sprite_setup_hit_box(k, &mut hb);
            if self.check_if_hit_boxes_overlap(&hb) {
                if self.lag_free { crate::cycle_ledger::charge(134); } // $F270..F278.
                let v = (k as u8).wrapping_add(1);
                self.sprite_workspace_mut().set_pickup_slot_cache(v);
                self.follower_link_state_mut().set_sprite_pickup_flag(v);
            } else if self.lag_free {
                crate::cycle_ledger::charge(6 + 42); // $F26E taken, $F2A9 RTS.
            }
            false
        } else {
            // $06:F279..F2A9, including BEQ taken and both call opcodes.
            // The two PLAs discard the caller's return before the final RTS.
            if self.lag_free { crate::cycle_ledger::charge(6 + 574 + 42); }
            self.follower_link_state_mut().set_filtered_joypad_l(0);
            let value = 0;
            self.sprite_slot_view_mut(k).set_e(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x1d);
            let value = self.sprite_slot_view(k).state();
            self.sprite_slot_view_mut(k).set_draw_work_byte_4(value);
            let value = 10;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_draw_work_byte_3(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_draw_i(value);
            let dir = self.sprite_direction_to_face_link(k) as usize;
            self.follower_link_state_mut().set_facing(
                SPRITE_RETURN_IF_LIFTED_PERMISSIVE_LIFTED_SPRITE_PLAYER_FACING_BY_DIRECTION
                    [dir & 3],
            );
            true
        }
    }

    // void Sprite_CheckIfLifted_permissive(int k) {  // 86aa0c
    //   Sprite_ReturnIfLiftedPermissive(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_if_lifted_permissive(&mut self, k: usize) {
        let _ = self.sprite_return_if_lifted_permissive(k);
    }

    // uint8 Sprite_DirectionToFaceLink(int k, PointU8 *coords_out) {  // 86eaa4
    //   PairU8 below = Sprite_IsBelowLink(k);
    //   PairU8 right = Sprite_IsRightOfLink(k);
    //   uint8 ym = sign8(below.b) ? -below.b : below.b;
    //   tmp_counter = ym;
    //   uint8 xm = sign8(right.b) ? -right.b : right.b;
    //   if (coords_out)
    //     coords_out->x = right.b, coords_out->y = below.b;
    //   return (xm >= ym) ? right.a : below.a + 2;
    // }
    pub(in crate::zelda_rtl) fn sprite_direction_to_face_link(&mut self, k: usize) -> u8 {
        self.sprite_direction_and_offset_to_face_link(k).0
    }

    /// The direction the sprite must face to look at Link, and the sprite's
    /// signed byte offset from Link (positive when the sprite is right of or
    /// below Link).
    pub(in crate::zelda_rtl) fn sprite_direction_and_offset_to_face_link(&mut self, k: usize) -> (u8, PointU8) {
        let below = self.sprite_is_below_link(k);
        let right = self.sprite_is_right_of_link(k);
        let ym = if sign8(below.b) {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };
        self.temp_counter_mut().set(ym);
        let xm = if sign8(right.b) {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let offset = PointU8 {
            x: right.b,
            y: below.b,
        };
        (if xm >= ym { right.a } else { below.a + 2 }, offset)
    }

    // int Sprite_SpawnDynamically(int k, uint8 what, SpriteSpawnInfo *info) { // 9df65d
    //   return Sprite_SpawnDynamicallyEx(k, what, info, 15);
    // }
    //
    // Canonical 1:1 port. Returns -1 if no slot was found, otherwise the
    // 0..15 slot index. `info` is populated with the spawn coordinates the
    // caller will consume via `Sprite_SetSpawnedCoordinates`.
    pub(in crate::zelda_rtl) fn sprite_spawn_dynamically(
        &mut self,
        k: usize,
        what: u8,
        info: &mut SpriteSpawnInfo,
    ) -> i32 {
        self.sprite_spawn_dynamically_ex(k, what, info, 15)
    }

    // int Sprite_SpawnDynamicallyEx(int k, uint8 what, SpriteSpawnInfo *info, int j) { // 9df65f
    //   do {
    //     if (sprite_state[j] == 0) {
    //       sprite_type[j] = what;
    //       sprite_state[j] = 9;
    //       info->r0_x = Sprite_GetX(k);
    //       info->r2_y = Sprite_GetY(k);
    //       info->r4_z = sprite_z[k];
    //       info->r5_overlord_x = overlord_x_lo[k] | overlord_x_hi[k] << 8;
    //       info->r7_overlord_y = overlord_y_lo[k] | overlord_y_hi[k] << 8;
    //       SpritePrep_LoadProperties(j);
    //       if (!player_is_indoors) {
    //         sprite_N_word[j] = 0xffff;
    //       } else {
    //         sprite_N[j] = 0xff;
    //       }
    //       sprite_floor[j] = sprite_floor[k];
    //       sprite_D[j] = sprite_D[k];
    //       sprite_die_action[j] = 0;
    //       sprite_subtype[j] = 0;
    //       break;
    //     }
    //   } while (--j >= 0);
    //   return j;
    // }
    //
    // Canonical 1:1 port. The do/while loop walks j down from the caller's
    // starting bound (15 from `Sprite_SpawnDynamically`, or 13 / 14 for
    // narrower variants); the first slot with `sprite_state[j] == 0` wins.
    /// `Sprite_SpawnDynamically` with the original's negative "no free slot" result
    /// folded into `None`. The spawn record carries the parent's x, y, and z for
    /// `sprite_set_spawned_coordinates`.
    pub(in crate::zelda_rtl) fn spawn_sprite_dynamically(
        &mut self,
        k: usize,
        what: u8,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        self.spawn_sprite_dynamically_ex(k, what, 15)
    }

    /// `Sprite_SpawnDynamicallyEx`: the search for a free slot starts at `j_in`.
    pub(in crate::zelda_rtl) fn spawn_sprite_dynamically_ex(
        &mut self,
        k: usize,
        what: u8,
        j_in: i32,
    ) -> Option<(usize, SpriteSpawnInfo)> {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, what, &mut info, j_in);
        (j >= 0).then_some((j as usize, info))
    }

    pub(in crate::zelda_rtl) fn sprite_spawn_dynamically_ex(
        &mut self,
        k: usize,
        what: u8,
        info: &mut SpriteSpawnInfo,
        j_in: i32,
    ) -> i32 {
        let mut j = j_in;
        loop {
            if j >= 0 && self.sprite_slot_view(j as usize).state() == 0 {
                let ju = j as usize;
                self.sprite_spawn_dynamically_selected_prefix(
                    k,
                    what,
                    info,
                    ju,
                    crate::SpriteDynamicSpawnProgress::SubtypeCleared,
                );
                break;
            }
            j -= 1;
            if j < 0 {
                break;
            }
        }
        j
    }

    fn sprite_spawn_dynamically_capture_info(&self, k: usize, info: &mut SpriteSpawnInfo) {
        info.r0_x = self.sprite_get_x(k);
        info.r2_y = self.sprite_get_y(k);
        info.r4_z = self.sprite_slot_view(k).z();
        info.r5_overlord_x = self.overlord_slot_view(k).x();
        info.r7_overlord_y = self.overlord_slot_view(k).y();
    }

    pub(in crate::zelda_rtl) fn sprite_spawn_dynamically_selected_prefix(
        &mut self,
        k: usize,
        what: u8,
        info: &mut SpriteSpawnInfo,
        j: usize,
        progress: crate::SpriteDynamicSpawnProgress,
    ) {
        assert!(j < 16);
        assert_eq!(
            self.sprite_slot_view(j).state(),
            0,
            "source dynamic-spawn slot must be free before its first publication",
        );
        self.sprite_slot_view_mut(j).set_sprite_type(what);
        if progress == crate::SpriteDynamicSpawnProgress::TypePublished {
            return;
        }
        self.sprite_slot_view_mut(j).set_state(9);
        self.sprite_spawn_dynamically_capture_info(k, info);
        if progress == crate::SpriteDynamicSpawnProgress::StatePublished {
            return;
        }

        match progress {
            crate::SpriteDynamicSpawnProgress::ResetProperties { completed_stores } => {
                self.sprite_prep_reset_properties_prefix(j, completed_stores);
                return;
            }
            crate::SpriteDynamicSpawnProgress::LoadProperties { completed_stores } => {
                self.sprite_prep_reset_properties(j);
                self.sprite_prep_load_properties_after_reset_prefix(j, completed_stores);
                return;
            }
            _ => self.sprite_prep_load_properties(j),
        }
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_view_mut(j).set_n_word(0xffff);
        } else {
            self.sprite_slot_view_mut(j).set_n(0xff);
        }
        if progress == crate::SpriteDynamicSpawnProgress::IdentityPublished {
            return;
        }
        let floor = self.sprite_slot_view(k).floor();
        self.sprite_slot_view_mut(j).set_floor(floor);
        if progress == crate::SpriteDynamicSpawnProgress::FloorPublished {
            return;
        }
        let direction = self.sprite_slot_view(k).direction();
        self.sprite_slot_view_mut(j).set_direction(direction);
        if progress == crate::SpriteDynamicSpawnProgress::DirectionPublished {
            return;
        }
        self.sprite_slot_view_mut(j).set_die_action(0);
        if progress == crate::SpriteDynamicSpawnProgress::DieActionCleared {
            return;
        }
        self.sprite_slot_view_mut(j).set_subtype(0);
        assert_eq!(progress, crate::SpriteDynamicSpawnProgress::SubtypeCleared);
    }

    pub(in crate::zelda_rtl) fn sprite_spawn_dynamically_selected_from(
        &mut self,
        k: usize,
        info: &mut SpriteSpawnInfo,
        j: usize,
        progress: crate::SpriteDynamicSpawnProgress,
    ) {
        self.sprite_spawn_dynamically_capture_info(k, info);
        match progress {
            crate::SpriteDynamicSpawnProgress::TypePublished => {
                self.sprite_slot_view_mut(j).set_state(9);
                self.sprite_prep_load_properties(j);
            }
            crate::SpriteDynamicSpawnProgress::StatePublished => {
                self.sprite_prep_load_properties(j);
            }
            crate::SpriteDynamicSpawnProgress::ResetProperties { completed_stores } => {
                self.sprite_prep_reset_properties_from(j, completed_stores);
                self.sprite_prep_load_properties_after_reset(j);
            }
            crate::SpriteDynamicSpawnProgress::LoadProperties { completed_stores } => {
                self.sprite_prep_load_properties_after_reset_from(j, completed_stores);
            }
            _ => {}
        }
        if matches!(
            progress,
            crate::SpriteDynamicSpawnProgress::TypePublished
                | crate::SpriteDynamicSpawnProgress::StatePublished
                | crate::SpriteDynamicSpawnProgress::ResetProperties { .. }
                | crate::SpriteDynamicSpawnProgress::LoadProperties { .. }
        ) {
            if self.game_state.world.location.is_outdoors() {
                self.sprite_slot_view_mut(j).set_n_word(0xffff);
            } else {
                self.sprite_slot_view_mut(j).set_n(0xff);
            }
        }
        if !matches!(
            progress,
            crate::SpriteDynamicSpawnProgress::FloorPublished
                | crate::SpriteDynamicSpawnProgress::DirectionPublished
                | crate::SpriteDynamicSpawnProgress::DieActionCleared
                | crate::SpriteDynamicSpawnProgress::SubtypeCleared
        ) {
            let floor = self.sprite_slot_view(k).floor();
            self.sprite_slot_view_mut(j).set_floor(floor);
        }
        if !matches!(
            progress,
            crate::SpriteDynamicSpawnProgress::DirectionPublished
                | crate::SpriteDynamicSpawnProgress::DieActionCleared
                | crate::SpriteDynamicSpawnProgress::SubtypeCleared
        ) {
            let direction = self.sprite_slot_view(k).direction();
            self.sprite_slot_view_mut(j).set_direction(direction);
        }
        if !matches!(
            progress,
            crate::SpriteDynamicSpawnProgress::DieActionCleared
                | crate::SpriteDynamicSpawnProgress::SubtypeCleared
        ) {
            self.sprite_slot_view_mut(j).set_die_action(0);
        }
        if progress != crate::SpriteDynamicSpawnProgress::SubtypeCleared {
            self.sprite_slot_view_mut(j).set_subtype(0);
        }
    }

    // int ReleaseFairy() {  // 9efe33
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(0, 0xe3, &info);
    //   if (j >= 0) {
    //     sprite_floor[j] = link_is_on_lower_level;
    //     Sprite_SetX(j, link_x_coord + 8);
    //     Sprite_SetY(j, link_y_coord + 16);
    //     sprite_D[j] = 0;
    //     sprite_delay_aux4[j] = 96;
    //   }
    //   return j;
    // }
    pub(in crate::zelda_rtl) fn release_fairy(&mut self) -> i32 {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0xe3, &mut info);
        if j >= 0 {
            let ju = j as usize;
            let value = self.game_state.player.follower_link.lower_level_state();
            self.sprite_slot_view_mut(ju).set_floor(value);
            self.sprite_set_x(ju, self.game_state.player.follower_link.x().wrapping_add(8));
            self.sprite_set_y(
                ju,
                self.game_state.player.follower_link.y().wrapping_add(16),
            );
            let value = 0;
            self.sprite_slot_view_mut(ju).set_direction(value);
            let value = 96;
            self.sprite_slot_view_mut(ju).set_delay_aux4(value);
        }
        j
    }

    // int Sprite_ShowSolicitedMessage(int k, uint16 msg) {  // 85e1a7
    //   static const uint8 kShowMessageFacing_Tab0[4] = {4, 6, 0, 2};
    //   dialogue_message_index = msg;
    //   if (!Sprite_CheckDamageToLink_same_layer(k) ||
    //       Sprite_CheckIfLinkIsBusy() ||
    //       !(filtered_joypad_L & 0x80) ||
    //       sprite_delay_aux4[k] || link_auxiliary_state == 2)
    //     return sprite_D[k];
    //   uint8 dir = Sprite_DirectionToFaceLink(k, NULL);
    //   if (link_direction_facing != kShowMessageFacing_Tab0[dir])
    //     return sprite_D[k];
    //   Sprite_ShowMessageUnconditional(dialogue_message_index);
    //   sprite_delay_aux4[k] = 64;
    //   return dir ^ 0x103;
    // }
    //
    // Canonical 1:1 port. Returns either `sprite_D[k]` (low byte; caller
    // typically gates on the 0x100 bit) or `dir ^ 0x103` once dialogue is
    // actually triggered. Mirrors the C `int` return.
    pub(in crate::zelda_rtl) fn sprite_show_solicited_message(&mut self, k: usize, msg: u16) -> u16 {
        self.dialogue_message_index_mut().set_value(msg);
        if !self.sprite_check_damage_to_link_same_layer(k)
            || self.sprite_check_if_link_is_busy()
            || (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0
            || self.sprite_slot_view(k).delay_aux4() != 0
            || self
                .game_state
                .player
                .follower_link
                .is_in_auxiliary_state(2)
        {
            return u16::from(self.sprite_slot_view(k).direction());
        }
        let dir = self.sprite_direction_to_face_link(k);
        if self.game_state.player.follower_link.facing()
            != SPRITE_SHOW_SOLICITED_MESSAGE_MESSAGE_FACING_BY_DIRECTION[(dir & 3) as usize]
        {
            return u16::from(self.sprite_slot_view(k).direction());
        }
        let msg_index = self.game_state.messaging.dialogue_message_index.value();
        self.sprite_show_message_unconditional(msg_index);
        let value = 64;
        self.sprite_slot_view_mut(k).set_delay_aux4(value);
        u16::from(dir) ^ 0x103
    }

    // int Sprite_ShowMessageOnContact(int k, uint16 msg) {  // 85e1f0
    //   dialogue_message_index = msg;
    //   if (!Sprite_CheckDamageToLink_same_layer(k) || link_auxiliary_state == 2)
    //     return sprite_D[k];
    //   Sprite_ShowMessageUnconditional(dialogue_message_index);
    //   return Sprite_DirectionToFaceLink(k, NULL) ^ 0x103;
    // }
    pub(in crate::zelda_rtl) fn sprite_show_message_on_contact(&mut self, k: usize, msg: u16) -> u16 {
        self.dialogue_message_index_mut().set_value(msg);
        if !self.sprite_check_damage_to_link_same_layer(k)
            || self
                .game_state
                .player
                .follower_link
                .is_in_auxiliary_state(2)
        {
            return u16::from(self.sprite_slot_view(k).direction());
        }
        let msg_index = self.game_state.messaging.dialogue_message_index.value();
        self.sprite_show_message_unconditional(msg_index);
        u16::from(self.sprite_direction_to_face_link(k)) ^ 0x103
    }

    // bool Sprite_TutorialGuard_ShowMessageOnContact(int k, uint16 msg) {  // 85fa59
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_tutorial_guard_show_message_on_contact(
        &mut self,
        k: usize,
        msg: u16,
    ) -> bool {
        self.dialogue_message_index_mut().set_value(msg);
        let bak2 = self.sprite_slot_view(k).flags2();
        let bak4 = self.sprite_slot_view(k).flags4();
        let value = 0x80;
        self.sprite_slot_view_mut(k).set_flags2(value);
        let value = 0x07;
        self.sprite_slot_view_mut(k).set_flags4(value);
        let rv = self.sprite_check_damage_to_link_same_layer(k);
        let value = bak2;
        self.sprite_slot_view_mut(k).set_flags2(value);
        let value = bak4;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if !rv {
            return false;
        }
        self.sprite_nullify_hookshot_drag();
        self.follower_link_state_mut().clear_running();
        self.follower_link_state_mut().set_speed_setting(0);
        if !self.game_state.player.follower_link.has_auxiliary_state() {
            self.sprite_show_message_minimal_c();
        }
        true
    }

    // void Sprite_ShowMessageUnconditional(uint16 msg) {  // 85e219
    //   dialogue_message_index = msg;
    //   TILE_INTERACTION_SHARED_FLAG = 0;
    //   messaging_module = 0;
    //   submodule_index = 2;
    //   saved_module_for_menu = main_module_index;
    //   main_module_index = 14;
    //   Sprite_NullifyHookshotDrag();
    //   link_speed_setting = 0;
    //   Link_CancelDash();
    //   link_auxiliary_state = 0;
    //   link_incapacitated_timer = 0;
    //   if (link_player_handler_state == kPlayerState_RecoilWall)
    //     link_player_handler_state = kPlayerState_Ground;
    // }
    //
    // Canonical 1:1 port. `kPlayerState_RecoilWall == 13`, `kPlayerState_Ground == 0`
    // (see player.h).
    pub(in crate::zelda_rtl) fn sprite_show_message_unconditional(&mut self, msg: u16) {
        self.dialogue_message_index_mut().set_value(msg);
        self.clear_tile_interaction_shared_flag();
        self.messaging_state_mut().clear_module();
        let main_module = self.game_state.frame.main_module;
        self.set_submodule(2);
        self.set_saved_module_for_menu(main_module);
        self.set_main_module(14);
        self.sprite_nullify_hookshot_drag();
        self.follower_link_state_mut().set_speed_setting(0);
        self.link_cancel_dash();
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        if self.game_state.player.follower_link.handler_state()
            == SPRITE_SHOW_MESSAGE_UNCONDITIONAL_PLAYER_HANDLER_STATE_RECOIL_WALL_LOCAL
        {
            self.follower_link_state_mut().set_handler_state(
                SPRITE_SHOW_MESSAGE_UNCONDITIONAL_PLAYER_HANDLER_STATE_GROUND_LOCAL,
            );
        }
    }

    // void Sprite_ShowMessageMinimal() {  // 85fa8e
    //   TILE_INTERACTION_SHARED_FLAG = 0;
    //   messaging_module = 0;
    //   submodule_index = 2;
    //   saved_module_for_menu = main_module_index;
    //   main_module_index = 14;
    // }
    pub(in crate::zelda_rtl) fn sprite_show_message_minimal_c(&mut self) {
        // $05:FA8E-FAA1: three absolute stores, two immediates, three
        // direct-page accesses and the actual RTL (m8 x8, slow ROM).
        crate::cycle_ledger::charge_routine(0x05_fa8e, 244);
        self.clear_tile_interaction_shared_flag();
        self.messaging_state_mut().clear_module();
        let main_module = self.game_state.frame.main_module;
        self.set_submodule(2);
        self.set_saved_module_for_menu(main_module);
        self.set_main_module(14);
    }

    // void Sprite_ApplyConveyor(int k, int j) {  // 9d8010
    //   if (!(frame_counter & 1))
    //     return;
    //   static const int8 kConveyorAdjustment_X[] = {0, 0, -1, 1};
    //   static const int8 kConveyorAdjustment_Y[] = {-1, 1, 0, 0};
    //   Sprite_SetX(k, Sprite_GetX(k) + kConveyorAdjustment_X[j - 0x68]);
    //   Sprite_SetY(k, Sprite_GetY(k) + kConveyorAdjustment_Y[j - 0x68]);
    // }
    pub(in crate::zelda_rtl) fn sprite_apply_conveyor(&mut self, k: usize, idx: usize) {
        if (self.game_state.frame.frame_counter & 1) == 0 {
            return;
        }
        self.sprite_set_x(
            k,
            self.sprite_get_x(k)
                .wrapping_add(SPRITE_APPLY_CONVEYOR_CONVEYOR_TILE_X_ADJUSTMENTS[idx] as i16 as u16),
        );
        self.sprite_set_y(
            k,
            self.sprite_get_y(k)
                .wrapping_add(SPRITE_APPLY_CONVEYOR_CONVEYOR_TILE_Y_ADJUSTMENTS[idx] as i16 as u16),
        );
    }

    // uint8 Sprite_BounceFromTileCollision(int k) {  // 9dc751
    //   int j = Sprite_CheckTileCollision(k);
    //   if (j & 3) {
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //     sprite_G[k]++;
    //   }
    //   if (j & 12) {
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    //     sprite_G[k]++;
    //     return sprite_G[k]; // wtf
    //   }
    //   return 0;
    // }
    pub(in crate::zelda_rtl) fn sprite_bounce_from_tile_collision(&mut self, k: usize) -> u8 {
        let j = self.sprite_check_tile_collision(k);
        if (j & 3) != 0 {
            let value = self.sprite_slot_view(k).x_velocity().wrapping_neg();
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            self.sprite_slot_view_mut(k).increment_g();
        }
        if (j & 12) != 0 {
            let value = self.sprite_slot_view(k).y_velocity().wrapping_neg();
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            self.sprite_slot_view_mut(k).increment_g();
            return self.sprite_slot_view(k).g();
        }
        0
    }

    // void Sprite_HaltAllMovement() {  // 9ef508
    //   Sprite_NullifyHookshotDrag();
    //   link_speed_setting = 0;
    //   Link_CancelDash();
    // }
    pub(in crate::zelda_rtl) fn sprite_halt_all_movement(&mut self) {
        self.sprite_nullify_hookshot_drag();
        self.follower_link_state_mut().set_speed_setting(0);
        self.link_cancel_dash();
    }

    // void Sprite_BehaveAsBarrier(int k) {  // 9ef4f3
    //   uint8 bak = sprite_flags4[k];
    //   sprite_flags4[k] = 0;
    //   if (Sprite_CheckDamageToLink_same_layer(k))
    //     Sprite_HaltAllMovement();
    //   sprite_flags4[k] = bak;
    // }
    pub(in crate::zelda_rtl) fn sprite_behave_as_barrier(&mut self, k: usize) {
        let bak = self.sprite_slot_view(k).flags4();
        let value = 0;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_halt_all_movement();
        }
        let value = bak;
        self.sprite_slot_view_mut(k).set_flags4(value);
    }

    // bool Sprite_CheckIfScreenIsClear() {  // 89af32
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && !(sprite_flags4[i] & 0x40)) {
    //       uint16 x = Sprite_GetX(i) - BG2HOFS_copy2;
    //       uint16 y = Sprite_GetY(i) - BG2VOFS_copy2;
    //       if (x < 256 && y < 256)
    //         return false;
    //     }
    //   }
    //   return Sprite_CheckIfOverlordsClear();
    // }
    pub(in crate::zelda_rtl) fn sprite_check_if_screen_is_clear(&self) -> bool {
        let _scope = crate::cycle_ledger::routine(0x09_af32);
        crate::cycle_ledger::charge(38); // $AF32..$AF33: PHX, LDX #15.
        for i in (0..=15usize).rev() {
            crate::cycle_ledger::charge(48); // $AF35..$AF38: state and BEQ.
            if self.sprite_slot_view(i).state() == 0 {
                crate::cycle_ledger::charge(6); // $AF38: taken BEQ.
            } else {
                crate::cycle_ledger::charge(64); // $AF3A..$AF3F: flags and BNE.
                if self.sprite_slot_view(i).flags4() & 0x40 != 0 {
                    crate::cycle_ledger::charge(6); // $AF3F: taken BNE.
                }
            }
            if self.sprite_slot_view(i).state() != 0
                && (self.sprite_slot_view(i).flags4() & 0x40) == 0
            {
                let x = self
                    .sprite_get_x(i)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
                let y = self
                    .sprite_get_y(i)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
                crate::cycle_ledger::charge(128); // $AF41..$AF4B: X subtraction and BNE.
                if x >= 256 {
                    crate::cycle_ledger::charge(6); // $AF4B: taken BNE skips Y.
                } else {
                    crate::cycle_ledger::charge(128); // $AF4D..$AF57: Y subtraction and BNE.
                    if y >= 256 {
                        crate::cycle_ledger::charge(6); // $AF57: taken BNE.
                    }
                }
                if x < 256 && y < 256 {
                    crate::cycle_ledger::charge(86); // $AF59..$AF5B: restore X, CLC, RTL.
                    return false;
                }
            }
            crate::cycle_ledger::charge(if i == 0 { 30 } else { 36 }); // $AF5C..$AF5D: DEX, BPL.
        }
        crate::cycle_ledger::charge(22); // $AF5F: BRA, sharing this caller's saved X.
        self.sprite_check_if_overlords_clear()
    }

    // bool Sprite_CheckIfRoomIsClear() {  // 89af61
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && !(sprite_flags4[i] & 0x40))
    //       return false;
    //   }
    //   return Sprite_CheckIfOverlordsClear();
    // }
    pub(in crate::zelda_rtl) fn sprite_check_if_room_is_clear(&self) -> bool {
        let lag_free = self.lag_free;
        let charge = |master| { if lag_free { crate::cycle_ledger::charge(master); } };
        // $09:AF61..AF74, M8/X8. The shared overlord tail below already
        // charges its own loop and the saved-X/RTL return.
        charge(38); // PHX, LDX #15.
        for i in (0..=15usize).rev() {
            charge(48); // LDA state, BEQ.
            if self.sprite_slot_view(i).state() != 0 {
                charge(64); // LDA flags4, AND, BNE.
                if (self.sprite_slot_view(i).flags4() & 0x40) == 0 {
                    charge(86); // $AF70..AF72 PLX, CLC, RTL.
                    return false;
                }
                charge(6); // Taken BNE.
            } else {
                charge(6); // Taken BEQ.
            }
            charge(if i == 0 { 30 } else { 36 }); // DEX, BPL.
        }
        self.sprite_check_if_overlords_clear()
    }

    // bool Sprite_CheckIfOverlordsClear() {  // 89af76
    //   for (int i = 7; i >= 0; i--) {
    //     if (overlord_type[i] == 0x14 || overlord_type[i] == 0x18)
    //       return false;
    //   }
    //   return true;
    // }
    pub(in crate::zelda_rtl) fn sprite_check_if_overlords_clear(&self) -> bool {
        // Screen/room checks branch here with X already saved on their stack.
        crate::cycle_ledger::charge(16); // $AF76: LDX #7.
        for i in (0..=7usize).rev() {
            crate::cycle_ledger::charge(64); // $AF78..$AF7D: type and first comparison.
            if self.overlord_slot_view(i).overlord_type() != 0x14 {
                crate::cycle_ledger::charge(32); // $AF7F..$AF81: second comparison.
            }
            if self.overlord_slot_view(i).overlord_type() == 0x14
                || self.overlord_slot_view(i).overlord_type() == 0x18
            {
                crate::cycle_ledger::charge(6 + 86); // Taken BEQ, $AF70..$AF72 return.
                return false;
            }
            crate::cycle_ledger::charge(if i == 0 { 30 } else { 36 }); // $AF83..$AF84: DEX, BPL.
        }
        crate::cycle_ledger::charge(86); // $AF86..$AF88: restore X, SEC, RTL.
        true
    }

    // void Sprite_ManuallySetDeathFlagUW(int k) {  // 89c2f5
    //   if (!player_is_indoors || sprite_defl_bits[k] & 1 || sign8(sprite_N[k]))
    //     return;
    //   sprite_where_in_room[dungeon_room_index2] |= 1 << sprite_N[k];
    // }
    pub(in crate::zelda_rtl) fn sprite_manually_set_death_flag_uw(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors()
            || (self.sprite_slot_view(k).deflection_bits() & 1) != 0
            || sign8(self.sprite_slot_view(k).n())
        {
            return;
        }
        let room = self.game_state.dungeon.room_tracking.room_index2_word();
        let bit = 1u16 << self.sprite_slot_view(k).n();
        let mask = self.sprite_where_in_room_mask(room) | bit;
        self.set_sprite_where_in_room_mask(room, mask);
    }

    // uint8 Sprite_ConvertVelocityToAngle(uint8 x, uint8 y) {  // 9df614
    //   static const uint8 kConvertVelocityToAngle_Tab0[32] = {
    //     0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 15, 15, 15, 14, 14, 14,
    //     8, 8, 7, 7, 7, 6, 6, 6, 8, 8,  9,  9,  9, 10, 10, 10,
    //   };
    //   static const uint8 kConvertVelocityToAngle_Tab1[32] = {
    //     4, 4, 3, 3, 3, 2, 2, 2, 12, 12, 13, 13, 13, 14, 14, 14,
    //     4, 4, 5, 5, 5, 6, 6, 6, 12, 12, 11, 11, 11, 10, 10, 10,
    //   };
    //   int s = ((y >> 7) + (x >> 7) * 2) * 8;
    //   if (sign8(x)) x = -x;
    //   if (sign8(y)) y = -y;
    //   if (x >= y) {
    //     return kConvertVelocityToAngle_Tab0[(y >> 2) + s];
    //   } else {
    //     return kConvertVelocityToAngle_Tab1[(x >> 2) + s];
    //   }
    // }
    pub(in crate::zelda_rtl) fn sprite_convert_velocity_to_angle(x: u8, y: u8) -> u8 {
        let mut x = x;
        let mut y = y;
        let s = (((y >> 7) + (x >> 7) * 2) * 8) as usize;
        if sign8(x) {
            x = x.wrapping_neg();
        }
        if sign8(y) {
            y = y.wrapping_neg();
        }
        // Both lookups index the contiguous ROM region: a magnitude past 31
        // runs off its 8-entry quadrant block into the following bytes, as
        // the ROM does (route host 1310660 read x-dominant index 39).
        if x >= y {
            sprite_convert_velocity_to_angle_rom_byte(((y >> 2) as usize) + s)
        } else {
            sprite_convert_velocity_to_angle_rom_byte(32 + ((x >> 2) as usize) + s)
        }
    }

    // -----------------------------------------------------------------
    // `_for_helpers` adapters used by the canonical message helpers above.
    // -----------------------------------------------------------------

}
