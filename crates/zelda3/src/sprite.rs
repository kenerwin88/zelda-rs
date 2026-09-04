// Methods ported from zelda3/src/sprite.c and included inside ZeldaState.

use super::sprite_main_mothula::WallmasterMainPrefixOutcome;
use super::*;
use crate::types::{sign16, sign8, PairU8, Point16U, PointU8, ProjectSpeedRet, SpriteHitBox};

mod sprite_shared;
use sprite_shared::*;

// `Sprite_DrawMultiple` consumes a table of these draws (sprite.h:38-42).
#[derive(Copy, Clone)]
pub(super) struct DrawMultipleData {
    pub x: i8,
    pub y: i8,
    pub char_flags: u16,
    pub ext: u8,
}

#[derive(Copy, Clone)]
struct DrawMultipleWordData {
    x: u16,
    y: u16,
    char_flags: u16,
    ext: u8,
}

// `Sprite_SetSpawnedCoordinates` consumes this struct (sprite.h:28-34).
#[derive(Copy, Clone, Default)]
pub(super) struct SpriteSpawnInfo {
    pub r0_x: u16,
    pub r2_y: u16,
    pub r4_z: u8,
    pub r5_overlord_x: u16,
    pub r7_overlord_y: u16,
}

#[derive(Copy, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

/// A C statement boundary inside `Dungeon_LoadSingleSprite` (`sprite.c:3649-3660`).
///
/// The room loader can cross vblank between these writes.  Keeping the
/// checkpoint semantic lets the translated loader execute the same prefix and
/// resume the same record without replaying already-visible sprite fields.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum DungeonSpriteLoadCheckpoint {
    State,
    TempY,
    Floor,
    YLow,
    YHigh,
    SharedX,
    XLow,
    XHigh,
    Type,
    SubtypeClear,
    TempSubtype,
    SubtypeFinal,
    SpawnIndex,
    Complete,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DungeonLoadSpritesCpuProgress {
    /// Zero-based ordinal among normal sprite-slot publications. Room-list
    /// marker, overlord, and already-loaded records do not publish
    /// `sprite_state` and therefore do not consume an ordinal.
    pub normal_load_ordinal: u16,
    pub slot: u8,
    pub checkpoint: DungeonSpriteLoadCheckpoint,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DungeonSpriteDisableCpuProgress {
    /// The descending live-sprite loop has visited slots 15 through `slot`.
    SpriteStatesThrough { slot: u8 },
    /// The live-sprite loop is complete and the descending ancilla loop has
    /// cleared slots 9 through `slot`.
    AncillasThrough { slot: u8 },
    /// Both descending loops are complete and the held-ancilla flag is clear.
    AncillaPickupFlagCleared,
    /// `sprite_limit_instance = 0` has published after the two loops.
    SpriteLimitInstanceCleared,
}

impl DungeonSpriteDisableCpuProgress {
    const fn source_ordinal(self) -> u8 {
        match self {
            Self::SpriteStatesThrough { slot } => 15 - slot,
            Self::AncillasThrough { slot } => 16 + (9 - slot),
            Self::AncillaPickupFlagCleared => 26,
            Self::SpriteLimitInstanceCleared => 27,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DungeonResetSpritesCpuProgress {
    Cache {
        slot: u8,
        field: CachedSpriteCacheField,
    },
    /// Source-level progress within `Sprite_DisableAll`. This deliberately
    /// describes Zelda statements rather than the backend PC that observed
    /// them, so a native timing owner can later publish the same receipt.
    Disable(DungeonSpriteDisableCpuProgress),
    /// `Sprite_DisableAll` has completed, including its final garnish-slot
    /// clear, but the remaining room-history bookkeeping and new room sprite
    /// load have not yet published.
    SpritesDisabled,
    /// `Sprite_DisableAll` has returned and the caller has published
    /// `sprcoll_x_size = 0xffff`; the paired Y-size store is still pending.
    CollisionXSizeSet,
    /// Both collision-size stores have published. The following four-entry
    /// room-history search is read-only, so this is also the resumable source
    /// checkpoint anywhere inside that search before its first mutation.
    RoomHistorySearchStarted,
    Load(DungeonLoadSpritesCpuProgress),
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn sprite_init_value(table: usize, ty: u8) -> u8 {
    let idx = (table * SPRITE_INIT_TABLE_LEN + ty as usize) * 2;
    let bytes = SPRITE_INIT_TABLES_C_HEX.as_bytes();
    (hex_nibble(bytes[idx]) << 4) | hex_nibble(bytes[idx + 1])
}

fn empty_sprite_hit_box() -> SpriteHitBox {
    SpriteHitBox {
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
    }
}

impl ZeldaState {
    pub(super) fn sprite_where_in_room_mask(&self, room: u16) -> u16 {
        self.game_state
            .sprites
            .workspace
            .where_in_room(usize::from(room))
    }

    pub(super) fn set_sprite_where_in_room_mask(&mut self, room: u16, mask: u16) {
        self.sprite_workspace_mut()
            .set_where_in_room(usize::from(room), mask);
    }

    pub(super) fn prepare_apply_rumble_to_sprites(&mut self) {
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

    // void Oam_ResetRegionBases() {  // 8683d3
    //   memcpy(oam_region_base, kOam_ResetRegionBases, 12);
    // }
    pub(super) fn oam_reset_region_bases(&mut self) {
        for (i, value) in OAM_RESET_REGION_BASES_OAM_RESET_REGION_BASES
            .into_iter()
            .enumerate()
        {
            self.oam_state_mut().set_region_base_word(i, value);
        }
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
    pub(super) fn sprite_spawn_immediately_smashed_terrain(&mut self, what: u8, x: u16, y: u16) {
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
    pub(super) fn sprite_spawn_throwable_terrain(&mut self, what: u8, x: u16, y: u16) {
        self.set_sound_effect_1_with_link_pan(29);
        self.sprite_spawn_throwable_terrain_silently(what, x, y);
    }

    // int Sprite_SpawnThrowableTerrain_silently(uint8 what, uint16 x, uint16 y) {  // 868156
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_throwable_terrain_silently(
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
        self.sprite_prep_load_properties_for_helpers(k);
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
    pub(super) fn overworld_substitute_alternate_secret(&mut self) {
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

    fn entity_apply_rumble_to_sprites(&mut self, hb: &mut SpriteHitBox) {
        for j in (0..=15).rev() {
            if self.sprite_slot_view(j).deflection_bits() & 2 == 0
                || self.sprite_slot_view(j).e() == 0
            {
                continue;
            }
            if self.game_state.sprites.system.chr_halfslot_state() != 0x0e {
                self.sprite_setup_hit_box(j, hb);
                if !self.check_if_hit_boxes_overlap(hb) {
                    continue;
                }
            }
            let value = 0;
            self.sprite_slot_view_mut(j).set_e(value);
            self.set_sound_effect_2(0x30);
            let value = 0x30;
            self.sprite_slot_view_mut(j).set_z_velocity(value);
            let value = 0x10;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = 0x30;
            self.sprite_slot_view_mut(j).set_delay_aux3(value);
            let value = 255;
            self.sprite_slot_view_mut(j).set_stunned(value);
            if self.sprite_slot_view(j).sprite_type() == 0xd8 {
                self.sprite_transmute_to_bomb_for_sprite(j);
            }
        }
    }

    fn sprite_transmute_to_bomb_for_sprite(&mut self, k: usize) {
        let value = 0x4a;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        let value = 1;
        self.sprite_slot_view_mut(k).set_c(value);
        let value = 255;
        self.sprite_slot_view_mut(k).set_delay_aux1(value);
        let value = 0x18;
        self.sprite_slot_view_mut(k).set_flags3(value);
        let value = 8;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_health(value);
    }

    pub(super) fn sprite_nullify_hookshot_drag(&mut self) {
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

    pub(super) fn sprite_prep_reset_properties(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).clear_prep_runtime_state();
    }

    pub(super) fn sprite_prep_reset_properties_prefix(&mut self, k: usize, completed_stores: u8) {
        self.sprite_slot_view_mut(k)
            .clear_prep_runtime_state_prefix(completed_stores);
    }

    pub(super) fn sprite_prep_reset_properties_from(&mut self, k: usize, completed_stores: u8) {
        self.sprite_slot_view_mut(k)
            .clear_prep_runtime_state_from(completed_stores);
    }

    pub(super) fn sprite_prep_load_properties(&mut self, k: usize) {
        self.sprite_prep_reset_properties(k);
        self.sprite_prep_load_properties_after_reset(k);
    }

    pub(super) fn sprite_prep_load_properties_after_reset(&mut self, k: usize) {
        self.sprite_prep_load_properties_after_reset_prefix(k, 10);
    }

    pub(super) fn sprite_prep_load_properties_after_reset_prefix(
        &mut self,
        k: usize,
        completed_stores: u8,
    ) {
        assert!(completed_stores <= 10);
        for store in 0..completed_stores {
            self.sprite_prep_load_property_store(k, store);
        }
    }

    pub(super) fn sprite_prep_load_properties_after_reset_from(
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

    pub(super) fn sprite_prep_load_palette(&mut self, k: usize) {
        let flags3 = sprite_init_value(
            SPRITE_INIT_FLAGS3_TABLE,
            self.sprite_slot_view(k).sprite_type(),
        );
        let value = flags3;
        self.sprite_slot_view_mut(k).set_flags3(value);
        let value = flags3 & 0x0f;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    pub(super) fn ancilla_spawn_falling_prize(&mut self, item: u8) -> i32 {
        self.ancilla_add_falling_prize(0x29, item, 4)
    }

    pub(super) fn sprite_set_x(&mut self, k: usize, x: u16) {
        self.sprite_slot_view_mut(k).set_x(x);
    }

    pub(super) fn sprite_set_y(&mut self, k: usize, y: u16) {
        self.sprite_slot_view_mut(k).set_y(y);
    }

    // void SpriteAddXY(int k, int xv, int yv) {
    //   Sprite_SetX(k, Sprite_GetX(k) + xv);
    //   Sprite_SetY(k, Sprite_GetY(k) + yv);
    // }
    pub(super) fn sprite_add_xy(&mut self, k: usize, xv: i32, yv: i32) {
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(xv as i16 as u16));
        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_add(yv as i16 as u16));
    }

    // void SpriteFall_AdjustPosition(int k) {  // 86e624
    //   SpriteAddXY(k, dung_floor_x_vel, dung_floor_y_vel);
    // }
    pub(super) fn sprite_fall_adjust_position(&mut self, k: usize) {
        self.sprite_add_xy(
            k,
            self.game_state.dungeon.moving_floor.floor_x_velocity() as i16 as i32,
            self.game_state.dungeon.moving_floor.floor_y_velocity() as i16 as i32,
        );
    }

    pub(super) fn sprite_get_x(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).x()
    }

    pub(super) fn sprite_get_y(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).y()
    }

    pub(super) fn sprite_is_right_of_location(&self, k: usize, x: u16) -> PairU8 {
        let xv = x.wrapping_sub(self.sprite_get_x(k));
        PairU8 {
            a: u8::from((xv as i16).is_negative()),
            b: xv as u8,
        }
    }

    pub(super) fn sprite_is_below_location(&self, k: usize, y: u16) -> PairU8 {
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
    pub(super) fn sprite_direction_to_face_location(&mut self, k: usize, x: u16, y: u16) -> u8 {
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

    pub(super) fn sprite_is_right_of_link(&self, k: usize) -> PairU8 {
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

    pub(super) fn sprite_is_below_link(&self, k: usize) -> PairU8 {
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

    pub(super) fn sprite_project_speed_towards_link(
        &self,
        k: usize,
        mut vel: u8,
    ) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.sprite_is_below_link(k);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };

        let right = self.sprite_is_right_of_link(k);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    pub(super) fn sprite_project_speed_towards_location(
        &self,
        k: usize,
        x: u16,
        y: u16,
        mut vel: u8,
    ) -> ProjectSpeedRet {
        if vel == 0 {
            return ProjectSpeedRet {
                x: 0,
                y: 0,
                xdiff: 0,
                ydiff: 0,
            };
        }
        let below = self.sprite_is_below_location(k, y);
        let mut r12 = if (below.b as i8).is_negative() {
            0u8.wrapping_sub(below.b)
        } else {
            below.b
        };
        let right = self.sprite_is_right_of_location(k, x);
        let mut r13 = if (right.b as i8).is_negative() {
            0u8.wrapping_sub(right.b)
        } else {
            right.b
        };
        let mut swapped = false;
        if r13 < r12 {
            swapped = true;
            std::mem::swap(&mut r12, &mut r13);
        }
        let mut xvel = vel;
        let mut yvel = 0u8;
        let mut t = 0u8;
        loop {
            t = t.wrapping_add(r12);
            if t >= r13 {
                t = t.wrapping_sub(r13);
                yvel = yvel.wrapping_add(1);
            }
            vel = vel.wrapping_sub(1);
            if vel == 0 {
                break;
            }
        }
        if swapped {
            std::mem::swap(&mut xvel, &mut yvel);
        }
        ProjectSpeedRet {
            x: if right.a != 0 {
                0u8.wrapping_sub(xvel)
            } else {
                xvel
            },
            y: if below.a != 0 {
                0u8.wrapping_sub(yvel)
            } else {
                yvel
            },
            xdiff: right.b,
            ydiff: below.b,
        }
    }

    // void Sprite_ApproachTargetSpeed(int k, uint8 x, uint8 y) {
    //   if (sprite_x_vel[k] - x)
    //     sprite_x_vel[k] += sign8(sprite_x_vel[k] - x) ? 1 : -1;
    //   if (sprite_y_vel[k] - y)
    //     sprite_y_vel[k] += sign8(sprite_y_vel[k] - y) ? 1 : -1;
    // }
    pub(super) fn sprite_approach_target_speed(&mut self, k: usize, x: u8, y: u8) {
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

    pub(super) fn sprite_setup_hit_box(&self, k: usize, hb: &mut SpriteHitBox) {
        if (self.sprite_slot_view(k).z() as i8).is_negative() {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let i = (self.sprite_slot_view(k).flags4() & 0x1f) as usize;
        let t = u16::from(self.sprite_slot_view(k).x_low())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XLO[i] as u8));
        hb.r4_spr_xlo = t as u8;
        let t_hi = u16::from(self.sprite_slot_view(k).x_high())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XHI[i] as u8))
            .wrapping_add(t >> 8);
        hb.r10_spr_xhi = t_hi as u8;

        let t = u16::from(self.sprite_slot_view(k).y_low())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YLO[i] as u8));
        let u = t >> 8;
        let ylo = (t as u8).wrapping_sub(self.sprite_slot_view(k).z());
        hb.r5_spr_ylo = ylo;
        let t_hi = u16::from(self.sprite_slot_view(k).y_high())
            .wrapping_sub(u16::from((t as u8) < self.sprite_slot_view(k).z()));
        hb.r11_spr_yhi = t_hi
            .wrapping_add(u)
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YHI[i] as u8))
            as u8;

        hb.r6_spr_xsize = SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XSIZE[i];
        hb.r7_spr_ysize = SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YSIZE[i];
    }

    // bool Sprite_SetupHitBox00(int k) {  // 86f1f6
    //   return (uint16)(link_x_coord - cur_sprite_x + 11) < 23 &&
    //          (uint16)(link_y_coord - cur_sprite_y + sprite_z[k] + 16) < 24;
    // }
    pub(super) fn sprite_setup_hit_box00(&self, k: usize) -> bool {
        self.game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
            .wrapping_add(11)
            < 23
            && self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                .wrapping_add(self.sprite_slot_view(k).z() as u16)
                .wrapping_add(16)
                < 24
    }

    // void Sprite_PlaceWeaponTink(int k) {  // 86f6ca
    //   if (repulsespark_timer)
    //     return;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    //   Sprite_PlaceRupulseSpark_2(k);
    // }
    pub(super) fn sprite_place_weapon_tink(&mut self, k: usize) {
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
        self.sprite_place_rupulse_spark_2(k);
    }

    // void Sprite_PlaceRupulseSpark_2(int k) {  // 86f6d5
    //   uint16 x = Sprite_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Sprite_GetY(k) - BG2VOFS_copy2;
    //   if (x & ~0xff || y & ~0xff)
    //     return;
    //   repulsespark_x_lo = sprite_x_lo[k];
    //   repulsespark_y_lo = sprite_y_lo[k];
    //   repulsespark_timer = 5;
    //   repulsespark_floor_status = sprite_floor[k];
    // }
    pub(super) fn sprite_place_rupulse_spark_2(&mut self, k: usize) {
        let x = self
            .sprite_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        let x_low = self.sprite_slot_view(k).x_low();
        self.garnish_state_mut().set_repulsespark_x_lo(x_low);
        let y_low = self.sprite_slot_view(k).y_low();
        self.garnish_state_mut().set_repulsespark_y_lo(y_low);
        self.garnish_state_mut().set_repulsespark_timer(5);
        let floor = self.sprite_slot_view(k).floor();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
    }

    // void Link_PlaceWeaponTink() {  // 86f69f
    //   if (repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int t = (uint8)link_x_coord + player_oam_x_offset;
    //   repulsespark_x_lo = t;
    //   t = (uint8)link_y_coord + player_oam_y_offset + (t >> 8);  // carry wtf
    //   repulsespark_y_lo = t;
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = Link_CalculateSfxPan() | 5;
    // }
    pub(super) fn link_place_weapon_tink(&mut self) {
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0 {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(5);
        let t = u16::from(self.game_state.player.follower_link.x() as u8)
            + u16::from(self.game_state.player.follower_link.oam_x_offset());
        let y = u16::from(self.game_state.player.follower_link.y() as u8)
            + u16::from(self.game_state.player.follower_link.oam_y_offset())
            + (t >> 8);
        self.garnish_state_mut().set_repulsespark_x_lo(t as u8);
        self.garnish_state_mut().set_repulsespark_y_lo(y as u8);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
        self.set_sound_effect_1_with_link_pan(5);
    }

    // void Sprite_ApplyRecoilToLink(int k, uint8 vel) {  // 86f688
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   link_actual_vel_x = pt.x;
    //   link_actual_vel_y = pt.y;
    //   g_ram[0xc7] = link_actual_vel_z = vel >> 1;
    //   link_z_coord = 0;
    // }
    pub(super) fn sprite_apply_recoil_to_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.follower_link_state_mut()
            .set_actual_velocity_xy(pt.x, pt.y);
        {
            self.follower_link_state_mut()
                .set_actual_z_velocity(vel >> 1);
            self.follower_link_state_mut()
                .set_recoil_z_velocity(vel >> 1);
            self.follower_link_state_mut().set_z(0);
        }
    }

    fn player_action_hit_box_from_table(&self, hb: &mut SpriteHitBox, t: usize, shrink: bool) {
        let mut x = self.game_state.player.follower_link.x().wrapping_add(
            PLAYER_ACTION_HIT_BOX_FROM_TABLE_X_OFFSETS[t]
                .wrapping_add(self.game_state.player.follower_link.oam_x_offset_signed())
                as i16 as u16,
        );
        let mut y = self.game_state.player.follower_link.y().wrapping_add(
            PLAYER_ACTION_HIT_BOX_FROM_TABLE_Y_OFFSETS[t]
                .wrapping_add(self.game_state.player.follower_link.oam_y_offset_signed())
                as i16 as u16,
        );
        let mut w = PLAYER_ACTION_HIT_BOX_FROM_TABLE_WIDTHS[t];
        let mut h = PLAYER_ACTION_HIT_BOX_FROM_TABLE_HEIGHTS[t];
        if shrink {
            if w >= 2 {
                let r = w.wrapping_sub(2).min(6);
                w = w.wrapping_sub(r);
                x = x.wrapping_add(u16::from(r >> 1));
            }
            if h >= 2 {
                let r = h.wrapping_sub(2).min(6);
                h = h.wrapping_sub(r);
                y = y.wrapping_add(u16::from(r >> 1));
            }
        }
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r2 = w;
        hb.r3 = h;
    }

    // void Player_SetupActionHitBox(SpriteHitBox *hb) {  // 86f5e0
    pub(super) fn player_setup_action_hit_box(&self, hb: &mut SpriteHitBox) {
        if self.game_state.player.follower_link.is_running() {
            let j = self.game_state.player.follower_link.facing_index();
            let x = self.game_state.player.follower_link.x().wrapping_add(
                u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_LO[j])
                    | (u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_HI[j]) << 8),
            );
            let y = self.game_state.player.follower_link.y().wrapping_add(
                u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_LO[j])
                    | (u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_HI[j]) << 8),
            );
            hb.r0_xlo = x as u8;
            hb.r8_xhi = (x >> 8) as u8;
            hb.r1_ylo = y as u8;
            hb.r9_yhi = (y >> 8) as u8;
            hb.r2 = 16;
            hb.r3 = 16;
            return;
        }

        let mut t = 0usize;
        if !self.game_state.player.follower_link.item_in_hand_has(10)
            && !self.game_state.player.follower_link.position_mode_has(0x10)
        {
            if sign8(self.game_state.player.follower_link.button_b_frames()) {
                let x = self.game_state.player.follower_link.x().wrapping_sub(14);
                let y = self.game_state.player.follower_link.y().wrapping_sub(10);
                hb.r0_xlo = x as u8;
                hb.r8_xhi = (x >> 8) as u8;
                hb.r1_ylo = y as u8;
                hb.r9_yhi = (y >> 8) as u8;
                hb.r2 = 44;
                hb.r3 = 45;
                return;
            } else if PLAYER_SETUP_ACTION_HIT_BOX_SWORD_ACTION_INACTIVE_FRAMES
                [usize::from(self.game_state.player.follower_link.button_b_frames())]
                != 0
            {
                hb.r8_xhi = 0x80;
                return;
            }
            t = usize::from(self.game_state.player.follower_link.facing()) * 8
                + usize::from(self.game_state.player.follower_link.button_b_frames())
                + 1;
        }
        self.player_action_hit_box_from_table(hb, t, false);
    }

    // void Link_UpdateHitBoxWithSword(SpriteHitBox *hb) {  // new
    pub(super) fn link_update_hit_box_with_sword(&self, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_step_counter()
            != 0
            || sign8(self.game_state.player.follower_link.button_b_frames())
            || LINK_UPDATE_HIT_BOX_WITH_SWORD_SWORD_ACTION_INACTIVE_FRAMES
                [usize::from(self.game_state.player.follower_link.button_b_frames())]
                != 0
        {
            return;
        }
        let t = usize::from(self.game_state.player.follower_link.facing()) * 8
            + usize::from(self.game_state.player.follower_link.button_b_frames())
            + 1;
        self.player_action_hit_box_from_table(hb, t, true);
    }

    // void Sprite_DoHitBoxesFast(int k, SpriteHitBox *hb) {  // 86f645
    //   if (HIBYTE(dungmap_var8) == 0x80) {
    //     hb->r10_spr_xhi = 0x80;
    //     return;
    //   }
    //   int t;
    //   t = Sprite_GetX(k) + (int8)HIBYTE(dungmap_var8);
    //   hb->r4_spr_xlo = t;
    //   hb->r10_spr_xhi = t >> 8;
    //   t = Sprite_GetY(k) + (int8)BYTE(dungmap_var8);
    //   hb->r5_spr_ylo = t;
    //   hb->r11_spr_yhi = t >> 8;
    //   hb->r6_spr_xsize = hb->r7_spr_ysize = (sprite_type[k] == 0x6a) ? 16 : 3;
    // }
    pub(super) fn sprite_do_hit_boxes_fast(&self, k: usize, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .sprites
            .draw_hitbox_work
            .hitbox_x_high_offset()
            == 0x80
        {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let x = self.sprite_get_x(k).wrapping_add(
            self.game_state
                .sprites
                .draw_hitbox_work
                .hitbox_x_high_offset() as i8 as i16 as u16,
        );
        hb.r4_spr_xlo = x as u8;
        hb.r10_spr_xhi = (x >> 8) as u8;
        let y = self.sprite_get_y(k).wrapping_add(
            self.game_state
                .sprites
                .draw_hitbox_work
                .hitbox_y_low_offset() as i8 as i16 as u16,
        );
        hb.r5_spr_ylo = y as u8;
        hb.r11_spr_yhi = (y >> 8) as u8;
        let size = if self.sprite_slot_view(k).sprite_type() == 0x6a {
            16
        } else {
            3
        };
        hb.r6_spr_xsize = size;
        hb.r7_spr_ysize = size;
    }

    // void Sprite_CorrectOamEntries(int k, int n, uint8 islarge) {  // 86febc
    //   OamEnt *oam = GetOamCurPtr();
    //   uint8 *extp = &g_ram[oam_ext_cur_ptr];
    //   uint16 spr_x = Sprite_GetX(k);
    //   uint16 spr_y = Sprite_GetY(k);
    //   uint8 scrollx = spr_x - BG2HOFS_copy2;
    //   uint8 scrolly = spr_y - BG2VOFS_copy2;
    //   do {
    //     uint16 x = spr_x + (int8)(oam->x - scrollx);
    //     uint16 y = spr_y + (int8)(oam->y - scrolly);
    //     uint8 ext = sign8(islarge) ? (*extp & 2) : islarge;
    //     *extp = ext + ((uint16)(x - BG2HOFS_copy2) >= 0x100);
    //     if ((uint16)(y + 0x10 - BG2VOFS_copy2) >= 0x100)
    //       oam->y = 0xf0;
    //   } while (oam++, extp++, --n >= 0);
    // }
    pub(super) fn sprite_correct_oam_entries(&mut self, k: usize, n: i32, islarge: u8) {
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut extp = self.game_state.oam.current_extended_pointer_usize();
        let spr_x = self.sprite_get_x(k);
        let spr_y = self.sprite_get_y(k);
        let scrollx =
            spr_x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) as u8;
        let scrolly =
            spr_y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) as u8;
        for _ in 0..=n {
            let x = spr_x.wrapping_add(
                self.game_state.oam.entry_x(oam).wrapping_sub(scrollx) as i8 as i16 as u16,
            );
            let y = spr_y.wrapping_add(
                self.game_state.oam.entry_y(oam).wrapping_sub(scrolly) as i8 as i16 as u16,
            );
            let ext = if sign8(islarge) {
                self.game_state.oam.extended_byte_at(extp) & 2
            } else {
                islarge
            };
            let value = ext
                + u8::from(
                    x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) >= 0x100,
                );
            self.oam_state_mut().set_extended_byte_at(extp, value);
            if y.wrapping_add(0x10)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                >= 0x100
            {
                self.oam_state_mut().hide_entry(oam);
            }
            oam += 4;
            extp += 1;
        }
    }

    // void Link_SetupHitBox_conditional(SpriteHitBox *hb) {  // 86f705
    //   if (link_disable_sprite_damage)
    //     hb->r9_yhi = 0x80;
    //   else
    //     Link_SetupHitBox(hb);
    // }
    pub(super) fn link_setup_hit_box_conditional(&self, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .player
            .follower_link
            .sprite_damage_disable_timer()
            != 0
        {
            hb.r9_yhi = 0x80;
        } else {
            self.link_setup_hit_box(hb);
        }
    }

    // void Link_SetupHitBox(SpriteHitBox *hb) {  // 86f70a
    //   hb->r3 = hb->r2 = 8;
    //   uint16 x = link_x_coord + 4;
    //   hb->r0_xlo = x;
    //   hb->r8_xhi = x >> 8;
    //   uint16 y = link_y_coord + 8;
    //   hb->r1_ylo = y;
    //   hb->r9_yhi = y >> 8;
    // }
    pub(super) fn link_setup_hit_box(&self, hb: &mut SpriteHitBox) {
        hb.r2 = 8;
        hb.r3 = 8;
        let x = self.game_state.player.follower_link.x().wrapping_add(4);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        let y = self.game_state.player.follower_link.y().wrapping_add(8);
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
    }

    pub(super) fn check_if_hit_boxes_overlap(&self, hb: &SpriteHitBox) -> bool {
        if hb.r8_xhi == 0x80 || hb.r10_spr_xhi == 0x80 {
            return false;
        }

        let mut t = i32::from(hb.r5_spr_ylo) - i32::from(hb.r1_ylo);
        let r15 = (t + i32::from(hb.r7_spr_ysize)) as u8;
        let r12 = hb
            .r11_spr_yhi
            .wrapping_sub(hb.r9_yhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r3.wrapping_add(hb.r7_spr_ysize) < r15 {
            return false;
        }

        t = i32::from(hb.r4_spr_xlo) - i32::from(hb.r0_xlo);
        let r15 = (t + i32::from(hb.r6_spr_xsize)) as u8;
        let r12 = hb
            .r10_spr_xhi
            .wrapping_sub(hb.r8_xhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r2.wrapping_add(hb.r6_spr_xsize) < r15 {
            return false;
        }

        true
    }

    pub(super) fn sprite_prep_oam_coord_or_double_ret(
        &mut self,
        k: usize,
    ) -> Option<(u16, u16, u8)> {
        let (ret, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if out {
            None
        } else {
            Some((ret.x, ret.y, ret.flags))
        }
    }

    pub(super) fn sprite_prep_oam_coord_or_double_ret_with_out_flag(
        &mut self,
        k: usize,
    ) -> ((u16, u16, u8), bool) {
        let (ret, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        ((ret.x, ret.y, ret.flags), out)
    }

    // void Sprite_PrepOamCoord(int k, PrepOamCoordsRet *ret) {  // 86e416
    //   Sprite_PrepOamCoordOrDoubleRet(k, ret);
    // }
    pub(super) fn sprite_prep_oam_coord(&mut self, k: usize, ret: &mut PrepOamCoordsRet) {
        let (prepped, _) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        *ret = prepped;
    }

    // bool Sprite_PrepOamCoordOrDoubleRet(int k, PrepOamCoordsRet *ret) {  // 86e41e
    //   sprite_pause[k] = 0;
    //   uint16 x = cur_sprite_x - BG2HOFS_copy2;
    //   uint16 y = cur_sprite_y - BG2VOFS_copy2;
    //   bool out_of_bounds = false;
    //   prep_x = x;
    //   prep_y = y - sprite_z[k];
    //   ret->flags = sprite_oam_flags[k] ^ sprite_obj_prio[k];
    //   ret->r4 = 0;
    //   if ((uint16)(x + 0x40 + xt) >= (0x170 + xt * 2) ||
    //       (uint16)(y + 0x40) >= 0x170 && !(sprite_flags4[k] & 0x20)) {
    //     sprite_pause[k]++;
    //     if (!(sprite_defl_bits[k] & 0x80))
    //       Sprite_KillSelf(k);
    //     out_of_bounds = true;
    //   }
    //   ret->x = prep_x;
    //   ret->y = prep_y;
    //   BYTE(dungmap_var7) = ret->x;
    //   HIBYTE(dungmap_var7) = ret->y;
    //   return out_of_bounds;
    // }
    fn sprite_prep_oam_coord_or_double_ret_raw(&mut self, k: usize) -> (PrepOamCoordsRet, bool) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_pause(value);
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let x = cur_x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = cur_y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let prep_y = y.wrapping_sub(self.sprite_slot_view(k).z() as u16);
        self.sprite_workspace_mut().set_oam_prep_coords(x, prep_y);
        let flags =
            self.sprite_slot_view(k).oam_flags() ^ self.sprite_slot_view(k).object_priority();
        let xt = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        let out = x.wrapping_add(0x40 + xt) >= 0x170 + xt * 2
            || (y.wrapping_add(0x40) >= 0x170 && self.sprite_slot_view(k).flags4() & 0x20 == 0);
        if out {
            let value = self.sprite_slot_view(k).pause().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_pause(value);
            if (self.sprite_slot_view(k).deflection_bits() & 0x80) == 0 {
                self.sprite_kill_self(k);
            }
        }
        let ret_x = self.game_state.sprites.workspace.oam_prep_x();
        let ret_y = self.game_state.sprites.workspace.oam_prep_y();
        let ret = PrepOamCoordsRet {
            x: ret_x,
            y: ret_y,
            r4: 0,
            flags,
        };
        self.draw_scratch_position_mut()
            .set_low_position(ret_x as u8, ret_y as u8);
        (ret, out)
    }

    // void Sprite_InitializeSlots() {  // 89afd6
    //   for (int k = 15; k >= 0; k--) {
    //     uint8 st = sprite_state[k], ty = sprite_type[k];
    //     if (st != 0) {
    //       if (st == 10) {
    //         if (ty != 0xec && ty != 0xd2) {
    //           link_picking_throw_state = 0;
    //           link_state_bits = 0;
    //           sprite_state[k] = 0;
    //         }
    //       } else {
    //         if (ty != 0x6c && sprite_room[k] != BYTE(overworld_area_index))
    //           sprite_state[k] = 0;
    //       }
    //     }
    //   }
    //   for (int k = 7; k >= 0; k--) {
    //     if (overlord_type[k] && overlord_spawned_in_area[k] != BYTE(overworld_area_index))
    //       overlord_type[k] = 0;
    //   }
    // }
    pub(super) fn sprite_initialize_slots(&mut self) {
        let area = self.game_state.world.region.overworld_area_low();
        for k in (0..=15usize).rev() {
            let st = self.sprite_slot_view(k).state();
            let ty = self.sprite_slot_view(k).sprite_type();
            if st == 0 {
                continue;
            }
            if st == 10 {
                if ty != 0xec && ty != 0xd2 {
                    self.follower_link_state_mut().clear_picking_throw_state();
                    self.follower_link_state_mut().clear_state_bits();
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
            } else if ty != 0x6c && self.sprite_slot_view(k).room() != area {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
        for k in (0..=7usize).rev() {
            if self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .overlord_type()
                != 0
                && self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .spawned_area()
                    != area
            {
                self.overlord_slot_view_mut(k).clear();
            }
        }
    }

    // void Sprite_InitializeMirrorPortal() {  // 89af89
    //   for (int k = 15; k >= 0; k--) {
    //     if (sprite_state[k] && sprite_type[k] == 0x6c)
    //       sprite_state[k] = 0;
    //   }
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(0xff, 0x6c, &info);
    //   if (j < 0)
    //     j = 0;
    //   Sprite_SetX(j, bird_travel_x_hi[15] << 8 | bird_travel_x_lo[15]);
    //   Sprite_SetY(j, (bird_travel_y_hi[15] << 8 | bird_travel_y_lo[15]) + 8);
    //   sprite_floor[j] = 0;
    //   sprite_ignore_projectile[j] = 1;
    // }
    pub(super) fn sprite_initialize_mirror_portal(&mut self) {
        for k in (0..=15usize).rev() {
            if self.sprite_slot_view(k).state() != 0
                && self.sprite_slot_view(k).sprite_type() == 0x6c
            {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }

        let mut info = SpriteSpawnInfo::default();
        let mut j = self.sprite_spawn_dynamically(0xff, 0x6c, &mut info);
        if j < 0 {
            j = 0;
        }
        let ju = j as usize;
        let bird = self
            .game_state
            .world
            .overworld
            .bird_travel_destinations
            .destination(15);
        let x = bird.x;
        let y = bird.y.wrapping_add(8);
        self.sprite_set_x(ju, x);
        self.sprite_set_y(ju, y);
        let value = 0;
        self.sprite_slot_view_mut(ju).set_floor(value);
        let value = 1;
        self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
    }

    // void Sprite_ResetAll() {  // 89c44e
    //   Sprite_DisableAll();
    //   Sprite_ResetAll_noDisable();
    // }
    pub(super) fn sprite_reset_all(&mut self) {
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable();
    }

    // void Sprite_DisableAll() {  // 89c22f
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_disable_all(&mut self) {
        self.apply_sprite_disable_actions_through(
            None,
            DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
        );
        self.sprite_disable_all_after_limit_instance();
    }

    /// Publish only Sprite_DisableAll's early live-slot state clears while a
    /// later overworld reload generation remains deferred. The surrounding
    /// reset side effects already ran in the provisional loader body.
    pub(super) fn sprite_disable_live_slots_before_deferred_overworld_reload(&mut self) {
        for k in (0..=15usize).rev() {
            if self.sprite_slot_view(k).state() != 0
                && (self.game_state.world.location.is_indoors()
                    || self.sprite_slot_view(k).sprite_type() != 0x6c)
            {
                self.sprite_slot_view_mut(k).set_state(0);
            }
        }
    }

    fn apply_sprite_disable_actions_through(
        &mut self,
        completed: Option<DungeonSpriteDisableCpuProgress>,
        target: DungeonSpriteDisableCpuProgress,
    ) {
        let start = completed
            .map(DungeonSpriteDisableCpuProgress::source_ordinal)
            .map_or(0, |ordinal| ordinal + 1);
        let target = target.source_ordinal();
        assert!(
            start <= target,
            "Sprite_DisableAll semantic progress moved backwards",
        );
        for ordinal in start..=target {
            match ordinal {
                0..=15 => {
                    let k = usize::from(15 - ordinal);
                    if self.sprite_slot_view(k).state() != 0
                        && (self.game_state.world.location.is_indoors()
                            || self.sprite_slot_view(k).sprite_type() != 0x6c)
                    {
                        self.sprite_slot_view_mut(k).set_state(0);
                    }
                }
                16..=25 => {
                    let k = usize::from(9 - (ordinal - 16));
                    self.ancilla_slot_view_mut(k).clear();
                }
                26 => self.follower_link_state_mut().clear_ancilla_pickup_flag(),
                27 => self.sprite_system_mut().set_limit_instance(0),
                _ => unreachable!("Sprite_DisableAll action ordinal is bounded"),
            }
        }
    }

    fn sprite_disable_all_after_limit_instance(&mut self) {
        self.sprite_battle_mut().clear_item_drop_counter();
        self.archery_game_mut().clear_hit_counter();
        self.archery_game_mut().set_arrows_left(0);
        self.garnish_state_mut().clear_active_type();
        self.dungeon_room_effects_mut().clear_trap_trigger_latch();
        self.dungeon_room_effects_mut()
            .set_activate_bomb_trap_overlord(0);
        self.attract_scene_mut().clear_intro_palette_flash_count();
        self.sprite_workspace_mut().set_reset_scratch_a(0);
        self.sprite_workspace_mut().set_reset_scratch_b(0);
        self.follower_link_state_mut().clear_menu_block();
        self.garnish_state_mut().clear_boulder_trap_count();
        self.sprite_system_mut().set_chr_halfslot_state(0);
        self.minigame_state_mut().clear_is_archer_or_shovel_game();
        for k in (0..8).rev() {
            self.overlord_slot_view_mut(k).clear();
        }
        for k in (0..30).rev() {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        }
    }

    // void Sprite_ResetAll_noDisable() {  // 89c452
    //   haunted_grove_flute_event_latch = 0;
    //   sprite_alert_flag = 0;
    //   overworld_boulder_trap_count = 0;
    //   MESSAGE_OR_SPRITE_STATE_CACHE = 0;
    //   sprite_chr_halfslot_state = 0;
    //   sprite_limit_instance = 0;
    //   sort_sprites_setting = 0;
    //   if (follower_indicator != 13)
    //     super_bomb_indicator_unk2 = 0xfe;
    //   memset(sprite_where_in_room, 0, 0x1000);
    //   memset(overworld_sprite_was_loaded, 0, 0x200);
    //   memset(dungeon_room_history, 0xff, 8);
    // }
    pub(super) fn sprite_reset_all_no_disable(&mut self) {
        self.sprite_reset_all_no_disable_fixed_prefix();
        self.sprite_reset_all_no_disable_after_fixed_prefix();
    }

    /// Complete the fixed stores before Sprite_ResetAll_noDisable's first
    /// large descending clear. This is the exact source boundary at $09:C47B.
    pub(super) fn sprite_reset_all_no_disable_fixed_prefix(&mut self) {
        self.garnish_state_mut()
            .clear_haunted_grove_flute_event_latch();
        self.sprite_system_mut().set_alert_flag(0);
        self.garnish_state_mut().clear_boulder_trap_count();
        self.messaging_state_mut()
            .clear_message_or_sprite_state_cache();
        self.sprite_system_mut().set_chr_halfslot_state(0);
        self.sprite_system_mut().set_limit_instance(0);
        self.oam_state_mut().clear_sprite_sorting_setting();
        if self.game_state.sprites.follower_runtime.indicator() != 13 {
            self.set_super_bomb_indicator_timer(0xfe);
        }
    }

    pub(super) fn sprite_reset_all_no_disable_after_fixed_prefix(&mut self) {
        self.sprite_workspace_mut().clear_where_in_room();
        self.clear_all_overworld_sprite_loaded_masks();
        self.dungeon_room_tracking_mut().reset_room_history();
    }

    pub(super) fn sprite_reload_all_overworld(&mut self) -> OverworldSpriteReloadWorkload {
        self.sprite_disable_all();
        self.sprite_overworld_reload_all_just_load()
    }

    pub(super) fn sprite_begin_reload_all_overworld(&mut self) {
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable();
    }

    pub(super) fn sprite_overworld_reload_all_just_load(
        &mut self,
    ) -> OverworldSpriteReloadWorkload {
        self.sprite_reset_all_no_disable();
        self.sprite_finish_reload_all_overworld()
    }

    pub(super) fn sprite_finish_reload_all_overworld(&mut self) -> OverworldSpriteReloadWorkload {
        let sprite_records = self.overworld_load_sprites();
        let in_bounds_proximity_checks = self.sprite_activate_all_proxima();
        OverworldSpriteReloadWorkload {
            sprite_records,
            in_bounds_proximity_checks,
        }
    }

    pub(super) fn overworld_load_sprites(&mut self) -> usize {
        let area = self.game_state.world.region.overworld_area();
        let area_lo = self.game_state.world.region.overworld_area_low() as usize;
        self.garnish_state_mut().set_sprcoll_x_base((area & 7) << 9);
        self.garnish_state_mut()
            .set_sprcoll_y_base((((area & 0x3f) >> 2) & 0x0e) << 8);
        let size = u16::from(OVERWORLD_AREA_SPRCOLL_SIZES[area_lo]) << 8;
        self.garnish_state_mut().set_sprcoll_x_size(size);
        self.garnish_state_mut().set_sprcoll_y_size(size);

        let base = match self.game_state.inventory.save_progress.progress_indicator() {
            3 => 2,
            2 => 1,
            _ => 0,
        };
        let Some(offsets) = self.asset_raw(159).map(Vec::from) else {
            return 0;
        };
        let Some(sprites) = self.asset_raw(160).map(Vec::from) else {
            return 0;
        };
        let offs_idx = (area as usize + base * 144) * 2;
        if offs_idx + 1 >= offsets.len() {
            return 0;
        }
        let mut src = read_word_from_slice(&offsets, offs_idx) as usize;
        let mut sprite_records = 0usize;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            sprite_records += 1;
            if sprites[src + 2] == 0xf4 {
                self.garnish_state_mut().increment_boulder_trap_count();
                src += 3;
                continue;
            }

            let r2 = (sprites[src] >> 4) << 2;
            let r6 = (sprites[src + 1] >> 4).wrapping_add(r2);
            let r5 = (sprites[src + 1] & 0x0f) | (sprites[src] << 4);
            let idx = usize::from(r5) | (usize::from(r6) << 8);
            let value = sprites[src + 2].wrapping_add(1);
            self.set_overworld_sprite_presence_marker(idx, value);
            src += 3;
        }
        sprite_records
    }

    pub(super) fn sprite_activate_all_proxima(&mut self) -> usize {
        let bak0 = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bak1 = self.overworld_horizontal_scroll_delta_low();
        self.set_overworld_horizontal_scroll_delta_low(0xff);

        let xt: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        self.set_bg2_x(bak0.wrapping_sub(xt));
        let mut in_bounds_proximity_checks = 0usize;
        for _ in (0..=(21 + (xt >> 3))).rev() {
            in_bounds_proximity_checks += self.sprite_activate_when_proximal();
            let bg = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(16);
            self.set_bg2_x(bg);
        }
        self.set_overworld_horizontal_scroll_delta_low(bak1);
        self.set_bg2_x(bak0);
        in_bounds_proximity_checks
    }

    pub(super) fn sprite_proximity_activation(&mut self) {
        if self.game_state.frame.submodule != 0 {
            self.sprite_activate_when_proximal();
            self.sprite_activate_when_proximal_big();
        } else {
            if self.game_state.sprites.system.ranged_based_toggler() & 1 == 0 {
                self.sprite_activate_when_proximal();
            }
            if self.game_state.sprites.system.ranged_based_toggler() & 1 != 0 {
                self.sprite_activate_when_proximal_big();
            }
            self.sprite_system_mut().increment_ranged_based_toggler();
        }
    }

    pub(super) fn sprite_activate_when_proximal(&mut self) -> usize {
        if self.overworld_horizontal_scroll_delta_low() == 0 {
            return 0;
        }
        let xt: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        let x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(if sign8(self.overworld_horizontal_scroll_delta_low()) {
                0u16.wrapping_sub(0x10).wrapping_sub(xt)
            } else {
                0x110u16.wrapping_add(xt)
            });
        let mut y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_sub(0x30);
        let mut in_bounds_proximity_checks = 0usize;
        for _ in (0..=21).rev() {
            in_bounds_proximity_checks +=
                usize::from(self.sprite_overworld_proximity_motivated_load(x, y));
            y = y.wrapping_add(16);
        }
        in_bounds_proximity_checks
    }

    pub(super) fn sprite_activate_when_proximal_big(&mut self) {
        if self.overworld_vertical_scroll_delta_low() == 0 {
            return;
        }
        let xt: u16 = if self
            .game_state
            .enhanced_features
            .has(FEATURES0_EXTEND_SCREEN64_SPRITE)
        {
            0x40
        } else {
            0
        };
        let mut x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_sub(0x30)
            .wrapping_sub(xt);
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(if sign8(self.overworld_vertical_scroll_delta_low()) {
                0u16.wrapping_sub(0x10)
            } else {
                0x110
            });
        for _ in (0..=(21 + (xt >> 3))).rev() {
            self.sprite_overworld_proximity_motivated_load(x, y);
            x = x.wrapping_add(16);
        }
    }

    pub(super) fn sprite_overworld_proximity_motivated_load(&mut self, x: u16, y: u16) -> bool {
        let sprcoll_x_base = self.game_state.sprites.garnish_runtime.sprcoll_x_word();
        let sprcoll_y_base = self.game_state.sprites.garnish_runtime.sprcoll_y_word();
        let xt = x.wrapping_sub(sprcoll_x_base);
        let yt = y.wrapping_sub(sprcoll_y_base);
        if xt >= self.game_state.sprites.garnish_runtime.sprcoll_x_size()
            || yt >= self.game_state.sprites.garnish_runtime.sprcoll_y_size()
        {
            return false;
        }

        let r1 = (((yt >> 8) * 4) | (xt >> 8)) as u8;
        let r0 = ((y & 0x00f0) | ((x >> 4) & 0x000f)) as u8;
        self.overworld_load_proxima_sprite_if_alive((u16::from(r1) << 8) | u16::from(r0));
        true
    }

    pub(super) fn overworld_load_proxima_sprite_if_alive(&mut self, blk: u16) {
        let sprite_to_spawn = self
            .game_state
            .sprites
            .overworld_sprite_presence
            .marker(blk as usize);
        if sprite_to_spawn == 0 {
            return;
        }

        let loadedmask = 0x80u8 >> (blk & 7);
        if self
            .game_state
            .sprites
            .overworld_sprite_loaded
            .is_loaded(blk, loadedmask)
        {
            return;
        }

        if sprite_to_spawn >= 0xf4 {
            let k = self.alloc_overlord();
            if k < 0 {
                return;
            }
            let k = k as usize;
            self.set_overworld_sprite_loaded_mask(blk, loadedmask);
            self.overlord_slot_view_mut(k).set_sprite_block_pos(blk);
            self.overlord_slot_view_mut(k)
                .set_overlord_type(sprite_to_spawn.wrapping_sub(0xf3));
            let x_low = ((blk << 4) & 0x00f0) as u8
                + if self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .overlord_type()
                    == 1
                {
                    8
                } else {
                    0
                };
            let x_high = (((blk >> 8) & 3) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_x_word() >> 8) as u8,
            );
            self.overlord_slot_view_mut(k)
                .set_x(u16::from(x_low) | (u16::from(x_high) << 8));
            let y_low = (blk & 0x00f0) as u8;
            let y_high = ((blk >> 10) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_y_word() >> 8) as u8,
            );
            self.overlord_slot_view_mut(k)
                .set_y(u16::from(y_low) | (u16::from(y_high) << 8));
            self.overlord_slot_view_mut(k).set_floor(0);
            let area = self.game_state.world.region.overworld_area_low();
            self.overlord_slot_view_mut(k).set_spawned_area(area);
            self.overlord_slot_view_mut(k).set_gen2(0);
            self.overlord_slot_view_mut(k).set_gen1(0);
            self.overlord_slot_view_mut(k).set_gen3(0);
        } else {
            let k = self.overworld_alloc_sprite(sprite_to_spawn);
            if k < 0 {
                return;
            }
            let k = k as usize;
            if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                println!(
                    "ow-load frame={} blk=0x{:04x} raw=0x{:02x} type=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                    self.game_state.frame.frame_counter,
                    blk,
                    sprite_to_spawn,
                    sprite_to_spawn.wrapping_sub(1),
                    k,
                    self.sprite_slot_view(k).sprite_type(),
                    self.sprite_slot_view(k).state(),
                    self.sprite_slot_view(k).c(),
                    self.sprite_slot_view(k).bump_damage(),
                );
            }
            self.set_overworld_sprite_loaded_mask(blk, loadedmask);
            self.sprite_slot_view_mut(k).set_n_word(blk);
            let value = sprite_to_spawn.wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            let value = 8;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = ((blk << 4) & 0x00f0) as u8;
            self.sprite_slot_view_mut(k).set_x_low(value);
            let value = (blk & 0x00f0) as u8;
            self.sprite_slot_view_mut(k).set_y_low(value);
            let value = (((blk >> 8) & 3) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_x_word() >> 8) as u8,
            );
            self.sprite_slot_view_mut(k).set_x_high(value);
            let value = ((blk >> 10) as u8).wrapping_add(
                (self.game_state.sprites.garnish_runtime.sprcoll_y_word() >> 8) as u8,
            );
            self.sprite_slot_view_mut(k).set_y_high(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_floor(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_subtype(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_die_action(value);
        }
    }

    pub(super) fn dungeon_reset_sprites(&mut self) {
        self.dungeon_reset_sprites_before_room_load();
        self.dungeon_load_sprites();
    }

    pub(super) fn dungeon_reset_sprites_through_cpu_progress(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        match progress {
            DungeonResetSpritesCpuProgress::Cache { slot, field } => {
                self.dungeon_cache_trans_sprites_through_cpu_progress(slot, field);
            }
            DungeonResetSpritesCpuProgress::Disable(progress) => {
                if self.game_state.world.location.is_indoors() {
                    self.dungeon_cache_trans_sprites();
                }
                self.follower_link_state_mut().clear_picking_throw_state();
                self.follower_link_state_mut().clear_state_bits();
                self.apply_sprite_disable_actions_through(None, progress);
            }
            DungeonResetSpritesCpuProgress::SpritesDisabled => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
            }
            DungeonResetSpritesCpuProgress::CollisionXSizeSet => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
                self.garnish_state_mut().set_sprcoll_x_size(0xffff);
            }
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted => {
                self.dungeon_reset_sprites_through_sprite_disable_all();
                self.dungeon_reset_sprites_set_collision_sizes();
            }
            DungeonResetSpritesCpuProgress::Load(progress) => {
                self.dungeon_reset_sprites_before_room_load();
                self.dungeon_load_sprites_through_cpu_progress(progress);
            }
        }
    }

    pub(super) fn dungeon_resume_reset_sprites_after_cpu_progress(
        &mut self,
        progress: DungeonResetSpritesCpuProgress,
    ) {
        match progress {
            DungeonResetSpritesCpuProgress::Cache { slot, field } => {
                self.dungeon_resume_cache_trans_sprites_after_cpu_progress(slot, field);
                self.dungeon_reset_sprites_after_cache_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::Disable(progress) => {
                if progress != DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared {
                    self.apply_sprite_disable_actions_through(
                        Some(progress),
                        DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                    );
                }
                self.sprite_disable_all_after_limit_instance();
                self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::SpritesDisabled => {
                self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::CollisionXSizeSet => {
                self.garnish_state_mut().set_sprcoll_y_size(0xffff);
                self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted => {
                self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
                self.dungeon_load_sprites();
            }
            DungeonResetSpritesCpuProgress::Load(progress) => {
                self.dungeon_resume_load_sprites_after_cpu_progress(progress);
            }
        }
    }

    /// Advance an already-applied source prefix to a later receipt from the
    /// next timing-authority host call. This is deliberately narrow: the only
    /// cross-host refinements are within `Sprite_DisableAll` itself and
    /// within `Dungeon_CacheTransSprites`' slot/field walk.
    pub(super) fn dungeon_advance_reset_sprites_cpu_progress(
        &mut self,
        completed: DungeonResetSpritesCpuProgress,
        target: DungeonResetSpritesCpuProgress,
    ) -> bool {
        match (completed, target) {
            (
                DungeonResetSpritesCpuProgress::Disable(completed),
                DungeonResetSpritesCpuProgress::Disable(target),
            ) if completed.source_ordinal() <= target.source_ordinal() => {
                if completed != target {
                    self.apply_sprite_disable_actions_through(Some(completed), target);
                }
                true
            }
            (
                DungeonResetSpritesCpuProgress::Cache {
                    slot: completed_slot,
                    field: completed_field,
                },
                DungeonResetSpritesCpuProgress::Cache {
                    slot: target_slot,
                    field: target_field,
                },
            ) if target_slot < completed_slot
                || (target_slot == completed_slot && completed_field <= target_field) =>
            {
                // The C caching loop walks slots 15..0 with the fields of
                // each slot in source order; apply only the delta between
                // the two checkpoints (route host 133571 refined slot 15's
                // `I` to `IgnoreProjectile` at its accepting NMI).
                assert!(target_field <= self.dungeon_cache_last_field(usize::from(target_slot)));
                if target_slot == completed_slot {
                    if completed_field != target_field {
                        self.dungeon_cache_trans_sprite_fields(
                            usize::from(target_slot),
                            Some(completed_field),
                            target_field,
                        );
                    }
                } else {
                    let completed_slot = usize::from(completed_slot);
                    self.dungeon_cache_trans_sprite_fields(
                        completed_slot,
                        Some(completed_field),
                        self.dungeon_cache_last_field(completed_slot),
                    );
                    for k in (usize::from(target_slot) + 1..completed_slot).rev() {
                        self.dungeon_cache_trans_sprite_fields(
                            k,
                            None,
                            self.dungeon_cache_last_field(k),
                        );
                    }
                    self.dungeon_cache_trans_sprite_fields(
                        usize::from(target_slot),
                        None,
                        target_field,
                    );
                }
                true
            }
            (
                DungeonResetSpritesCpuProgress::SpritesDisabled,
                DungeonResetSpritesCpuProgress::CollisionXSizeSet,
            ) => {
                self.garnish_state_mut().set_sprcoll_x_size(0xffff);
                true
            }
            (
                DungeonResetSpritesCpuProgress::SpritesDisabled,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ) => {
                self.dungeon_reset_sprites_set_collision_sizes();
                true
            }
            (
                DungeonResetSpritesCpuProgress::CollisionXSizeSet,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ) => {
                self.garnish_state_mut().set_sprcoll_y_size(0xffff);
                true
            }
            _ => false,
        }
    }

    pub(super) fn dungeon_reset_sprites_before_room_load(&mut self) {
        self.dungeon_reset_sprites_through_sprite_disable_all();
        self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
    }

    fn dungeon_reset_sprites_through_sprite_disable_all(&mut self) {
        if self.game_state.world.location.is_indoors() {
            self.dungeon_cache_trans_sprites();
        }
        self.dungeon_reset_sprites_after_cache_through_sprite_disable_all();
    }

    fn dungeon_reset_sprites_after_cache_through_sprite_disable_all(&mut self) {
        {
            self.follower_link_state_mut().clear_picking_throw_state();
            self.follower_link_state_mut().clear_state_bits();
        }
        self.sprite_disable_all();
    }

    fn dungeon_reset_sprites_after_sprite_disable_before_room_load(&mut self) {
        self.dungeon_reset_sprites_set_collision_sizes();
        self.dungeon_reset_sprites_after_collision_sizes_before_room_load();
    }

    fn dungeon_reset_sprites_set_collision_sizes(&mut self) {
        self.garnish_state_mut().set_sprcoll_x_size(0xffff);
        self.garnish_state_mut().set_sprcoll_y_size(0xffff);
    }

    fn dungeon_reset_sprites_after_collision_sizes_before_room_load(&mut self) {
        let room = self.game_state.dungeon.room_tracking.room_index2_word();
        let seen =
            (0..4).any(|i| self.game_state.dungeon.room_tracking.room_history_entry(i) == room);
        if !seen {
            let dropped = self.game_state.dungeon.room_tracking.room_history_entry(3);
            for i in (1..4).rev() {
                let prev = self
                    .game_state
                    .dungeon
                    .room_tracking
                    .room_history_entry(i - 1);
                self.dungeon_room_tracking_mut()
                    .set_room_history_entry(i, prev);
            }
            self.dungeon_room_tracking_mut()
                .set_room_history_entry(0, room);
            if dropped != 0xffff {
                self.set_sprite_where_in_room_mask(dropped, 0);
            }
        }
    }

    fn dungeon_reset_sprites_after_cache_before_room_load(&mut self) {
        self.dungeon_reset_sprites_after_cache_through_sprite_disable_all();
        self.dungeon_reset_sprites_after_sprite_disable_before_room_load();
    }

    pub(super) fn dungeon_load_sprites(&mut self) {
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return;
        };
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;

        self.sprite_workspace_mut()
            .set_room_origin_y_high(((room >> 3) & 0xfe) as u8);
        self.sprite_workspace_mut()
            .set_room_origin_x_high(((room & 0x0f) << 1) as u8);
        self.oam_state_mut()
            .set_sprite_sorting_setting(sprites[start]);

        let mut k = 0isize;
        let mut src = start + 1;
        while src < sprites.len() && sprites[src] != 0xff {
            if src + 2 >= sprites.len() {
                break;
            }
            k = self.dungeon_load_single_sprite(
                k as usize,
                sprites[src],
                sprites[src + 1],
                sprites[src + 2],
            );
            k += 1;
            src += 3;
        }
    }

    fn dungeon_sprite_records(&self) -> Option<(Vec<u8>, usize)> {
        let sprites = self.asset_raw(58).map(Vec::from)?;
        let offsets = self.asset_raw(59).map(Vec::from)?;
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;
        let start = read_word_from_slice(&offsets, room * 2) as usize;
        (start < sprites.len()).then_some((sprites, start))
    }

    fn dungeon_load_sprites_through_cpu_progress(
        &mut self,
        progress: DungeonLoadSpritesCpuProgress,
    ) {
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return;
        };
        let room = self.game_state.dungeon.room_tracking.room_index2_word() as usize;
        self.sprite_workspace_mut()
            .set_room_origin_y_high(((room >> 3) & 0xfe) as u8);
        self.sprite_workspace_mut()
            .set_room_origin_x_high(((room & 0x0f) << 1) as u8);
        self.oam_state_mut()
            .set_sprite_sorting_setting(sprites[start]);

        let mut k = 0isize;
        let mut src = start + 1;
        let mut normal_load_ordinal = 0u16;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let y = sprites[src];
            let x = sprites[src + 1];
            let sprite_type = sprites[src + 2];
            let loads_normal_slot =
                self.dungeon_sprite_record_loads_normal_slot(k as usize, y, x, sprite_type);
            if loads_normal_slot && normal_load_ordinal == progress.normal_load_ordinal {
                assert_eq!(
                    k as u8, progress.slot,
                    "translated room sprite slot disagrees with the ROM checkpoint",
                );
                self.dungeon_load_single_sprite_fields(
                    k as usize,
                    y,
                    x,
                    sprite_type,
                    None,
                    progress.checkpoint,
                );
                return;
            }
            k = self.dungeon_load_single_sprite(k as usize, y, x, sprite_type);
            k += 1;
            if loads_normal_slot {
                normal_load_ordinal = normal_load_ordinal.saturating_add(1);
            }
            src += 3;
        }
        panic!(
            "ROM room-load checkpoint normal-load ordinal {} is outside the room sprite list",
            progress.normal_load_ordinal,
        );
    }

    fn dungeon_resume_load_sprites_after_cpu_progress(
        &mut self,
        progress: DungeonLoadSpritesCpuProgress,
    ) {
        let Some((sprites, start)) = self.dungeon_sprite_records() else {
            return;
        };
        let mut k = 0usize;
        let mut src = start + 1;
        let mut normal_load_ordinal = 0u16;
        let mut target_reached = false;
        while src + 2 < sprites.len() && sprites[src] != 0xff {
            let y = sprites[src];
            let x = sprites[src + 1];
            let sprite_type = sprites[src + 2];
            let loads_normal_slot =
                self.dungeon_sprite_record_loads_normal_slot(k, y, x, sprite_type);
            if target_reached {
                k = (self.dungeon_load_single_sprite(k, y, x, sprite_type) + 1) as usize;
            } else if loads_normal_slot && normal_load_ordinal == progress.normal_load_ordinal {
                assert_eq!(
                    k as u8, progress.slot,
                    "translated room sprite slot disagrees with the ROM checkpoint",
                );
                self.dungeon_load_single_sprite_fields(
                    k,
                    y,
                    x,
                    sprite_type,
                    Some(progress.checkpoint),
                    DungeonSpriteLoadCheckpoint::Complete,
                );
                k += 1;
                normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                target_reached = true;
            } else {
                if Self::dungeon_sprite_record_advances_slot(y, x, sprite_type) {
                    k += 1;
                }
                if loads_normal_slot {
                    normal_load_ordinal = normal_load_ordinal.saturating_add(1);
                }
            }
            src += 3;
        }
        assert!(
            target_reached,
            "ROM room-load checkpoint normal-load ordinal {} is outside the room sprite list",
            progress.normal_load_ordinal,
        );
    }

    fn dungeon_sprite_record_advances_slot(y: u8, x: u8, sprite_type: u8) -> bool {
        if sprite_type == 0xe4 && (y == 0xfe || y == 0xfd) {
            return false;
        }
        if sprite_type != 0xe4 && x >= 0xe0 {
            return false;
        }
        true
    }

    fn dungeon_sprite_record_loads_normal_slot(
        &self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
    ) -> bool {
        Self::dungeon_sprite_record_advances_slot(y, x, sprite_type)
            && (sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, sprite_type) & 1 != 0
                || self.sprite_where_in_room_mask(
                    self.game_state.dungeon.room_tracking.room_index2_word(),
                ) & (1 << k)
                    == 0)
    }

    pub(super) fn dungeon_load_single_sprite(
        &mut self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
    ) -> isize {
        if sprite_type == 0xe4 {
            if y == 0xfe || y == 0xfd {
                if k != 0 {
                    let value = if y == 0xfe { 1 } else { 2 };
                    self.sprite_slot_view_mut(k - 1).set_die_action(value);
                }
                return k as isize - 1;
            }
        } else if x >= 0xe0 {
            self.dungeon_load_single_overlord(&[y, x, sprite_type]);
            return k as isize - 1;
        }

        if sprite_init_value(SPRITE_INIT_DEFL_BITS_TABLE, sprite_type) & 1 == 0
            && self
                .sprite_where_in_room_mask(self.game_state.dungeon.room_tracking.room_index2_word())
                & (1 << k)
                != 0
        {
            return k as isize;
        }

        self.dungeon_load_single_sprite_fields(
            k,
            y,
            x,
            sprite_type,
            None,
            DungeonSpriteLoadCheckpoint::Complete,
        );
        k as isize
    }

    fn dungeon_load_single_sprite_fields(
        &mut self,
        k: usize,
        y: u8,
        x: u8,
        sprite_type: u8,
        completed: Option<DungeonSpriteLoadCheckpoint>,
        through: DungeonSpriteLoadCheckpoint,
    ) {
        let should_apply = |checkpoint| {
            completed.is_none_or(|completed| checkpoint > completed) && checkpoint <= through
        };
        if should_apply(DungeonSpriteLoadCheckpoint::State) {
            self.sprite_slot_view_mut(k).set_state(8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::TempY) {
            self.temp_counter_mut().set(y);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Floor) {
            self.sprite_slot_view_mut(k).set_floor(y >> 7);
        }
        let y_coord = (((y as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_y_high() as u16) << 8);
        if should_apply(DungeonSpriteLoadCheckpoint::YLow) {
            self.sprite_slot_view_mut(k).set_y_low(y_coord as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::YHigh) {
            self.sprite_slot_view_mut(k)
                .set_y_high((y_coord >> 8) as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SharedX) {
            self.sprite_workspace_mut().set_shared_scratch_a(x);
        }
        let x_coord = (((x as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_x_high() as u16) << 8);
        if should_apply(DungeonSpriteLoadCheckpoint::XLow) {
            self.sprite_slot_view_mut(k).set_x_low(x_coord as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::XHigh) {
            self.sprite_slot_view_mut(k)
                .set_x_high((x_coord >> 8) as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Type) {
            self.sprite_slot_view_mut(k).set_sprite_type(sprite_type);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SubtypeClear) {
            self.sprite_slot_view_mut(k).set_subtype(0);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::TempSubtype) {
            let counter = (self.game_state.scratch_counter.value() & 0x60) >> 2;
            self.temp_counter_mut().set(counter);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SubtypeFinal) {
            let subtype = self.game_state.scratch_counter.value() | (x >> 5);
            self.sprite_slot_view_mut(k).set_subtype(subtype);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::SpawnIndex) {
            self.sprite_slot_view_mut(k).set_n(k as u8);
        }
        if should_apply(DungeonSpriteLoadCheckpoint::Complete) {
            self.sprite_slot_view_mut(k).set_die_action(0);
        }
    }

    // void Dungeon_LoadSingleOverlord(const uint8 *src) {  // 89c3e8
    //   int k = AllocOverlord();
    //   if (k < 0)
    //     return;
    //   uint8 y = src[0], x = src[1], type = src[2];
    //   overlord_type[k] = type;
    //   overlord_floor[k] = (y >> 7);
    //   int t = ((y << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_Y_HI << 8);
    //   overlord_y_lo[k] = t;
    //   overlord_y_hi[k] = t >> 8;
    //   t = ((x << 4) & 0x1ff) + (SPRITE_ROOM_ORIGIN_X_HI << 8);
    //   overlord_x_lo[k] = t;
    //   overlord_x_hi[k] = t >> 8;
    //   overlord_spawned_in_area[k] = overworld_area_index;
    //   overlord_gen2[k] = 0;
    //   overlord_gen1[k] = 0;
    //   overlord_gen3[k] = 0;
    //   if (overlord_type[k] == 10 || overlord_type[k] == 11) {
    //     overlord_gen2[k] = 160;
    //   } else if (overlord_type[k] == 3) {
    //     overlord_gen2[k] = 255;
    //     overlord_x_lo[k] -= 8;
    //   }
    // }
    pub(super) fn dungeon_load_single_overlord(&mut self, src: &[u8]) {
        let k = self.alloc_overlord();
        if k < 0 || src.len() < 3 {
            return;
        }
        let k = k as usize;
        let y = src[0];
        let x = src[1];
        let type_ = src[2];
        self.overlord_slot_view_mut(k).set_overlord_type(type_);
        self.overlord_slot_view_mut(k).set_floor(y >> 7);
        let mut t = (((y as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_y_high() as u16) << 8);
        self.overlord_slot_view_mut(k).set_y(t);
        t = (((x as u16) << 4) & 0x01ff)
            + ((self.game_state.sprites.workspace.room_origin_x_high() as u16) << 8);
        self.overlord_slot_view_mut(k).set_x(t);
        let area = self.game_state.world.region.overworld_area_low();
        self.overlord_slot_view_mut(k).set_spawned_area(area);
        self.overlord_slot_view_mut(k).set_gen2(0);
        self.overlord_slot_view_mut(k).set_gen1(0);
        self.overlord_slot_view_mut(k).set_gen3(0);
        if self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type()
            == 10
            || self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .overlord_type()
                == 11
        {
            self.overlord_slot_view_mut(k).set_gen2(160);
        } else if self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type()
            == 3
        {
            self.overlord_slot_view_mut(k).set_gen2(255);
            self.overlord_slot_view_mut(k).subtract_x_low(8);
        }
    }

    /// `Sprite_Main`'s work before its descending slot loop: the timing
    /// workload census, proximity activation, region resets, garnish,
    /// follower, ancilla, and overlord passes.
    fn sprite_main_prefix(&mut self) {
        let mut timing_workload = SpriteMainTimingWorkload::default();
        for slot in 0..16 {
            let sprite = self.sprite_slot_view(slot);
            if sprite.state() != 0 {
                timing_workload.record_active_sprite(sprite.sprite_type(), sprite.c());
            }
        }
        let active_garnish_count = (0..30)
            .filter(|&slot| !self.garnish_slot_view(slot).is_empty())
            .count() as u8;
        timing_workload.record_garnish_table(
            self.game_state.sprites.garnish_runtime.active_type() != 0,
            active_garnish_count,
        );
        self.last_sprite_main_timing_workload = Some(timing_workload);

        if self.game_state.world.location.is_outdoors() {
            for j in 0..5 {
                self.ancilla_slot_view_mut(j).set_floor(0);
            }
            self.sprite_proximity_activation();
        }
        let dark_world = u8::from(self.game_state.inventory.save_progress.dark_world_state() != 0);
        self.set_dark_world_region_index(dark_world);
        if self.game_state.frame.submodule == 0 {
            self.follower_link_state_mut().set_drag_player_x(0);
            self.follower_link_state_mut().set_drag_player_y(0);
        }
        self.oam_reset_region_bases();
        self.replay_trace_ram_watch("sprite-after-oam-reset");
        self.garnish_execute_upper_slots();
        self.replay_trace_ram_watch("sprite-after-garnish-upper");
        self.follower_main();
        self.replay_trace_ram_watch("sprite-after-follower");
        let pickup_slot_cache = self.game_state.player.follower_link.sprite_pickup_flag();
        self.sprite_workspace_mut()
            .set_pickup_slot_cache(pickup_slot_cache);
        self.follower_link_state_mut().clear_sprite_pickup_flag();
        self.hitbox_scratch_offset_mut().set_x_high_offset(0x80);
        self.sprite_battle_mut().tick_damaging_enemies_timer();
        self.follower_link_state_mut()
            .clear_player_pose_draw_counter();
        {
            self.follower_link_state_mut().set_pull_action_state(0);
            self.follower_link_state_mut().clear_prevent_movement();
        }
        if self.game_state.sprites.system.alert_flag() != 0 {
            self.sprite_system_mut().decrement_alert_flag();
        }
        self.ancilla_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // An ancilla's synchronous item receipt suspended the stack; the
            // receipt's completion resumes the rest of this prefix.
            return;
        }
        self.complete_sprite_main_prefix_after_ancilla();
    }

    pub(super) fn complete_sprite_main_prefix_after_ancilla(&mut self) {
        self.replay_trace_ram_watch("sprite-after-ancilla");
        self.overlord_main();
        self.replay_trace_ram_watch("sprite-after-overlord");
        self.archery_game_mut().clear_out_of_arrows();
    }

    /// A saved `Sprite_Main` caller whose loop had not started (the
    /// `BeforeFirstSlot` boundary) resumed in this host and ran its prefix
    /// plus slots 15 down to `newly_completed_slot` before the wire suspended
    /// it again (route host 187518). Execute exactly that semantic delta.
    pub(super) fn advance_sprite_main_before_first_slot_to_after_slot(
        &mut self,
        newly_completed_slot: u8,
    ) {
        if let Some((main_module, submodule)) = self.pending_module09_frame_advance.take() {
            // The suspended Module09 handler's deferred advance lands right
            // before Sprite_Main, as the ROM writes it.
            if self.pending_overworld_sprite_reload_slots.is_some() {
                self.publish_deferred_module09_sprite_slots_at_reload_return();
            }
            self.set_submodule(submodule);
            self.set_main_module(main_module);
        }
        self.sprite_main_prefix();
        self.advance_sprite_main_after_slot_boundary(16, newly_completed_slot);
    }

    pub(super) fn sprite_main(&mut self) {
        if self.sprite_main_cpu_boundary == Some(SpriteMainCpuBoundary::BeforeFirstSlot) {
            let boundary = self
                .sprite_main_cpu_boundary
                .take()
                .expect("Sprite_Main boundary was checked above");
            let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
            assert_ne!(
                nmi_slices, 0,
                "Sprite_Main continuation requires a measured NMI phase",
            );
            let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
            self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            return;
        }

        self.sprite_main_prefix();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            // The prefix's ancilla item receipt suspended the stack; its
            // completion runs the slot loop and suffix.
            return;
        }
        let trace_sprite_slots = ZeldaState::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_FRAME")
            .is_some_and(|frame| self.trace_frame_matches(frame));

        for k in (0..16).rev() {
            self.sprite_system_mut().set_cur_object_index(k as u8);
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-before-execute-single slot={k}"));
            }
            if let Some(SpriteMainCpuBoundary::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            }) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(
                        nmi_slices, 0,
                        "sprite-init reset continuation requires a measured NMI phase",
                    );
                    let boundary = self
                        .sprite_main_cpu_boundary
                        .take()
                        .expect("sprite-init reset boundary was checked above");
                    assert_eq!(
                        self.sprite_slot_view(k).state(),
                        8,
                        "source sprite-init reset boundary requires state 8",
                    );
                    self.sprite_timers_and_oam(k);
                    if phase
                        == crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion
                    {
                        // The source is inside Fire Debirando's second
                        // property load: its first load, state promotion, and
                        // type conversion all precede this reset call.
                        self.sprite_module_initialize_properties(k);
                        self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                    }
                    self.sprite_prep_reset_properties_prefix(k, completed_stores);
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                    return;
                }
            }
            if let Some(SpriteMainCpuBoundary::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            }) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(
                        nmi_slices, 0,
                        "sprite property-load continuation requires a measured NMI phase",
                    );
                    let boundary = self
                        .sprite_main_cpu_boundary
                        .take()
                        .expect("sprite property-load boundary was checked above");
                    assert_eq!(self.sprite_slot_view(k).state(), 8);
                    self.sprite_timers_and_oam(k);
                    match phase {
                        crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                            self.sprite_prep_reset_properties(k);
                        }
                        crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                            self.sprite_module_initialize_properties(k);
                            self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                            self.sprite_prep_reset_properties(k);
                        }
                    }
                    self.sprite_prep_load_properties_after_reset_prefix(k, completed_stores);
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                    return;
                }
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Fire Debirando spawn continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Fire Debirando boundary was checked above");
                assert_eq!(
                    self.sprite_slot_view(k).state(),
                    8,
                    "Fire Debirando source boundary requires state 8 at slot entry",
                );
                assert_eq!(
                    self.sprite_slot_view(k).sprite_type(),
                    0x64,
                    "Fire Debirando source boundary requires type $64 at slot entry",
                );
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                self.sprite_prep_load_properties(k);
                self.sprite_prep_fire_debirando_after_property_reload_before_spawn(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if let Some(SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            }) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(
                        nmi_slices, 0,
                        "Fire Debirando dynamic-spawn continuation requires a measured NMI phase",
                    );
                    let boundary = self
                        .sprite_main_cpu_boundary
                        .take()
                        .expect("Fire Debirando dynamic-spawn boundary was checked above");
                    assert_eq!(self.sprite_slot_view(k).state(), 8);
                    assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x64);
                    let spawned = usize::from(spawned_slot);
                    assert_ne!(spawned, k);
                    for candidate in (spawned + 1)..16 {
                        assert_ne!(
                            self.sprite_slot_view(candidate).state(),
                            0,
                            "source dynamic-spawn receipt skipped a higher free slot {candidate}",
                        );
                    }
                    self.sprite_timers_and_oam(k);
                    self.sprite_module_initialize_properties(k);
                    self.sprite_slot_view_mut(k).set_sprite_type(0x63);
                    self.sprite_prep_load_properties(k);
                    self.sprite_prep_fire_debirando_after_property_reload_before_spawn(k);
                    let mut info = SpriteSpawnInfo::default();
                    self.sprite_spawn_dynamically_selected_prefix(
                        k, 0x64, &mut info, spawned, progress,
                    );
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                    return;
                }
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterTimersAndOam { slot, state: None })
                    if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "timer/OAM continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                let state = self.sprite_slot_view(k).state();
                assert_ne!(
                    state, 0,
                    "source timer/OAM return requires an active sprite slot",
                );
                self.sprite_timers_and_oam(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterTimersAndOam {
                        slot: k as u8,
                        state: Some(state),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                    slot,
                    continuation: None,
                }) if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Antfairy subtype continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(
                    self.sprite_slot_view(k).state(),
                    9,
                    "source Antfairy subtype boundary requires an active sprite",
                );
                self.sprite_timers_and_oam(k);
                let continuation = self.antfairy_draw_continuation(k);
                self.sprite_slot_view_mut(k).add_subtype2(1);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                        slot: k as u8,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                    slot,
                    continuation: None,
                }) if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Lanmola subtype continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(
                    self.sprite_slot_view(k).state(),
                    9,
                    "source Lanmola subtype boundary requires an active sprite",
                );
                assert_eq!(
                    self.sprite_slot_view(k).sprite_type(),
                    0x54,
                    "source Lanmola subtype boundary requires a Lanmola",
                );
                self.sprite_timers_and_oam(k);
                let continuation = self.lanmola_prep_and_draw_through_subtype2_increment(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                        slot: k as u8,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BonkItemGraphicsEntered(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "bonk-item graphics continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("bonk-item graphics boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x3b);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_slot_view_mut(k).set_floor(2);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BariBeforeRandom(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Bari pre-RNG continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert!(matches!(
                    self.sprite_slot_view(k).sprite_type(),
                    0x23 | 0x24
                ));
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                self.sprite_prep_bari_before_random(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::BariBeforeRandom(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                    slot,
                    state: None,
                }) if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "main/aux1 timer decrement continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                let state = self.sprite_slot_view(k).state();
                assert_ne!(
                    state, 0,
                    "source main/aux1 timer decrement boundary requires an active sprite slot",
                );
                self.sprite_timers_and_oam_through_main_and_aux1_timer_decrements(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                        slot: k as u8,
                        state: Some(state),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { slot, state: None })
                    if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "primary timer decrement continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                let state = self.sprite_slot_view(k).state();
                assert_ne!(
                    state, 0,
                    "source primary timer decrement boundary requires an active sprite slot",
                );
                self.sprite_timers_and_oam_through_primary_timer_decrements(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterPrimaryTimerDecrements {
                        slot: k as u8,
                        state: Some(state),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterTimerDecrements { slot, state: None })
                    if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "timer decrement continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                let state = self.sprite_slot_view(k).state();
                assert_ne!(
                    state, 0,
                    "source timer decrement boundary requires an active sprite slot",
                );
                self.sprite_timers_and_oam_through_timer_decrements(k);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterTimerDecrements {
                        slot: k as u8,
                        state: Some(state),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::AfterWallmasterResetPrefix(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Wallmaster reset continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Wallmaster reset boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x90);
                self.sprite_timers_and_oam(k);
                assert_eq!(
                    self.sprite_90_wallmaster_through_send_decision(k),
                    WallmasterMainPrefixOutcome::SendPlayer,
                    "source Wallmaster reset boundary requires the send-player branch: state={:#04x} sub={:#04x} modal={:#04x} pause={:#04x} deflection={:#04x}",
                    self.sprite_slot_view(k).state(),
                    self.game_state.frame.submodule,
                    self.game_state.frame.modal_pause_flag,
                    self.sprite_slot_view(k).pause(),
                    self.sprite_slot_view(k).deflection_bits(),
                );
                self.wall_master_send_player_through_reset_fixed_prefix();
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(
                    k as u8,
                ))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "throwable-scenery continuation requires a measured NMI phase",
                );
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("throwable-scenery boundary was checked above");
                assert_eq!(self.sprite_slot_view(k).state(), 6);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xec);
                assert!(
                    sign8(self.sprite_slot_view(k).c()) || self.sprite_slot_view(k).c() < 6,
                    "throwable-scenery state-clear boundary requires the small-debris branch",
                );
                self.sprite_timers_and_oam(k);
                self.sprite_slot_view_mut(k).set_state(0);
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Zelda follower-graphics continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 8);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x76);
                self.sprite_timers_and_oam(k);
                self.sprite_module_initialize_properties(k);
                let saved_follower_indicator = self
                    .sprite_prep_zelda_before_follower_graphics(k)
                    .expect("ROM follower-graphics boundary requires Zelda's live prep path");
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
                        slot: k as u8,
                        saved_follower_indicator,
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::ZazakAfterGraphics(k as u8))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Zazak graphics continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert!(matches!(
                    self.sprite_slot_view(k).sprite_type(),
                    0xa5 | 0xa6 | 0xa7
                ));
                self.sprite_timers_and_oam(k);
                assert!(
                    self.sprite_zazak_before_graphics_boundary(k),
                    "source Zazak graphics boundary requires the ordinary live body",
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::ZazakAfterGraphics(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if let Some(SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller: follower_graphics_caller,
                prefix_completed: false,
                saved_follower_indicator: None,
                stage,
            }) = self.sprite_main_cpu_boundary
            {
                if slot == k as u8 {
                    let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                    assert_ne!(
                        nmi_slices, 0,
                        "partial sprite follower-graphics continuation requires a measured NMI phase",
                    );
                    self.sprite_main_cpu_boundary = None;
                    let saved_follower_indicator = match follower_graphics_caller {
                        crate::SpriteFollowerGraphicsCaller::BlindMaiden => {
                            assert_eq!(self.sprite_slot_view(k).state(), 8);
                            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb7);
                            self.sprite_timers_and_oam(k);
                            self.sprite_module_initialize_properties(k);
                            assert!(
                                self.sprite_prep_blind_maiden_before_follower_graphics(k),
                                "source follower-graphics progress requires Blind Maiden's live prep path",
                            );
                            None
                        }
                        crate::SpriteFollowerGraphicsCaller::Zelda => {
                            assert_eq!(self.sprite_slot_view(k).state(), 8);
                            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x76);
                            self.sprite_timers_and_oam(k);
                            self.sprite_module_initialize_properties(k);
                            Some(self.sprite_prep_zelda_before_follower_graphics(k).expect(
                                "source follower-graphics progress requires Zelda's live prep path",
                            ))
                        }
                        crate::SpriteFollowerGraphicsCaller::BlindMaidenBody => {
                            assert_eq!(self.sprite_slot_view(k).state(), 9);
                            assert_eq!(self.sprite_slot_view(k).sprite_type(), 0xb7);
                            self.sprite_timers_and_oam(k);
                            assert!(
                                self.sprite_b7_blind_maiden_before_follower_graphics(k),
                                "source follower-graphics progress requires Blind Maiden's live become-follower path",
                            );
                            None
                        }
                    };
                    self.apply_follower_graphics_progress(None, stage);
                    let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                    self.schedule_sprite_main_cpu_continuation(
                        SpriteMainCpuBoundary::FollowerGraphics {
                            slot,
                            caller: follower_graphics_caller,
                            prefix_completed: true,
                            saved_follower_indicator,
                            stage,
                        },
                        nmi_slices,
                        caller,
                    );
                    return;
                }
            }
            if self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(
                    k as u8,
                ))
            {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "King Zora flippers graphics continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x52);
                assert_eq!(self.sprite_slot_view(k).ai_state(), 3);
                self.sprite_timers_and_oam(k);
                assert!(
                    self.sprite_52_king_zora_before_flippers_graphics(k),
                    "source King Zora flippers boundary requires the live purchase-completion path",
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                assert!(matches!(caller, SpriteMainCpuCaller::Module09 { .. }));
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(k as u8),
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                    slot,
                    continuation: None,
                }) if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "single-small draw continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert!(matches!(
                    self.sprite_slot_view(k).sprite_type(),
                    0x23 | 0x24
                ));
                assert!(
                    self.sprite_slot_view(k).c() != 0 && !sign8(self.sprite_slot_view(k).c()),
                    "source single-small draw boundary requires Red Bari's positive-C draw path",
                );
                self.sprite_timers_and_oam(k);
                let continuation = self
                    .sprite_draw_single_small_position_prefix(k)
                    .expect("source single-small draw boundary requires visible OAM preparation");
                assert!(
                    continuation.visible,
                    "source single-small draw position boundary requires the visible Y store",
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                        slot: k as u8,
                        continuation: Some(continuation),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                    slot,
                    oam_position: None,
                }) if slot == k as u8
            ) {
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "guard-probe continuation requires a measured NMI phase",
                );
                self.sprite_main_cpu_boundary = None;
                assert_eq!(self.sprite_slot_view(k).state(), 9);
                assert_eq!(self.sprite_slot_view(k).sprite_type(), 0x41);
                assert_ne!(
                    self.sprite_slot_view(k).c(),
                    0,
                    "source guard-probe boundary requires Probe rather than Guard_Main",
                );
                self.sprite_timers_and_oam(k);
                let oam_position = self
                    .probe_until_after_oam_coordinates(k)
                    .expect("source guard-probe boundary did not reach its OAM-coordinate return");
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(
                    SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                        slot: k as u8,
                        oam_position: Some(oam_position),
                    },
                    nmi_slices,
                    caller,
                );
                return;
            }
            let enters_big_key_graphics = self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(k as u8));
            if enters_big_key_graphics {
                self.sprite_main_cpu_boundary = None;
                assert_ne!(
                    std::mem::take(&mut self.sprite_main_cpu_nmi_slices),
                    0,
                    "big-key graphics continuation requires a source timing boundary",
                );
                assert!(matches!(
                    std::mem::take(&mut self.sprite_main_cpu_caller),
                    SpriteMainCpuCaller::DungeonModule07Live { .. }
                ));
            }
            let enters_item_receipt_graphics = self.sprite_main_cpu_boundary
                == Some(SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(k as u8));
            if enters_item_receipt_graphics {
                self.sprite_main_cpu_boundary = None;
                assert_ne!(
                    std::mem::take(&mut self.sprite_main_cpu_nmi_slices),
                    0,
                    "item-receipt graphics continuation requires a source timing boundary",
                );
                assert!(matches!(
                    std::mem::take(&mut self.sprite_main_cpu_caller),
                    SpriteMainCpuCaller::DungeonModule07Live { .. }
                        | SpriteMainCpuCaller::Module09 { .. }
                ));
            }
            self.sprite_execute_single(k);
            if enters_big_key_graphics {
                assert!(
                    matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(GameWorkContinuation::FinishBigKeyDropGraphics {
                            sprite_slot,
                            ..
                        }) if sprite_slot == k as u8
                    ),
                    "source big-key boundary did not enter the native graphics continuation"
                );
                return;
            }
            if enters_item_receipt_graphics {
                assert!(
                    matches!(
                        self.game_execution_scheduler.current_work(),
                        Some(GameWorkContinuation::FinishItemReceiptGraphics {
                            continuation:
                                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                                    sprite_slot,
                                    ..
                                }
                                | ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                                    sprite_slot,
                                    ..
                                },
                        }) if sprite_slot == k as u8
                    ),
                    "source item-receipt boundary did not enter the native graphics continuation: host={} slot={} sprite_type={:#04x} sprite_state={:#04x} item_receipt_method={} work={:?}",
                    self.frame_ctr_dbg,
                    k,
                    self.sprite_slot_view(k).sprite_type(),
                    self.sprite_slot_view(k).state(),
                    self.game_state.player.follower_link.item_receipt_method(),
                    self.game_execution_scheduler.current_work(),
                );
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(
                    SpriteMainCpuBoundary::AfterSlot(slot)
                        | SpriteMainCpuBoundary::AfterActiveCuccoX { slot, .. }
                        | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoFleeMovement { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements { slot, .. }
                        | SpriteMainCpuBoundary::AfterCuccoGraphicsPublication { slot, .. }
                ) if slot == k as u8
            ) {
                let boundary = self
                    .sprite_main_cpu_boundary
                    .take()
                    .expect("Sprite_Main boundary was checked above");
                let nmi_slices = std::mem::take(&mut self.sprite_main_cpu_nmi_slices);
                assert_ne!(
                    nmi_slices, 0,
                    "Sprite_Main continuation requires a measured NMI phase",
                );
                let caller = std::mem::take(&mut self.sprite_main_cpu_caller);
                self.schedule_sprite_main_cpu_continuation(boundary, nmi_slices, caller);
            }
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return;
            }
            if trace_sprite_slots {
                self.replay_trace_ram_watch(&format!("sprite-after-execute-single slot={k}"));
            }
        }
        self.complete_sprite_main_after_all_slots();
        let suffix_nmi_slices =
            std::mem::take(&mut self.dungeon_room_load_module_suffix_nmi_slices);
        if suffix_nmi_slices != 0 {
            self.game_execution_scheduler
                .schedule_cpu_timed_work_before_trailing_nmi(
                    GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    },
                    suffix_nmi_slices,
                );
        }
    }

    pub(super) fn arm_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        nmi_slices: u8,
        caller: SpriteMainCpuCaller,
    ) {
        assert_ne!(nmi_slices, 0, "Sprite_Main continuation requires an NMI");
        assert!(
            self.sprite_main_cpu_boundary.is_none(),
            "Sprite_Main CPU continuation was already armed",
        );
        assert_eq!(self.sprite_main_cpu_nmi_slices, 0);
        assert_eq!(self.sprite_main_cpu_caller, SpriteMainCpuCaller::default());
        self.sprite_main_cpu_boundary = Some(boundary);
        self.sprite_main_cpu_nmi_slices = nmi_slices;
        self.sprite_main_cpu_caller = caller;
    }

    /// Replace an unconsumed reconstructed Sprite_Main checkpoint with the
    /// boundary published by the continuous semantic authority.
    ///
    /// Legacy CPU plans populate these three fields before the translated
    /// caller reaches Sprite_Main. No C work has executed at that checkpoint,
    /// so replacing it is an ownership transfer, not a replay or rollback.
    /// Once scheduled work exists the call is already in flight and must be
    /// refined through the scheduler instead.
    pub(super) fn arm_authoritative_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        caller: SpriteMainCpuCaller,
    ) {
        assert!(
            !self
                .game_execution_scheduler
                .work_suspends_translated_call_stack(),
            "source Sprite_Main entry receipt cannot replace in-flight work",
        );
        self.sprite_main_cpu_boundary = None;
        self.sprite_main_cpu_nmi_slices = 0;
        self.sprite_main_cpu_caller = SpriteMainCpuCaller::default();
        self.arm_sprite_main_cpu_continuation(boundary, 1, caller);
    }

    fn schedule_sprite_main_cpu_continuation(
        &mut self,
        boundary: SpriteMainCpuBoundary,
        nmi_slices: u8,
        caller: SpriteMainCpuCaller,
    ) {
        self.stage_interrupted_sprite_main_oam_scanout();
        let continuation = GameWorkContinuation::FinishSpriteMain { boundary, caller };
        let resumes_after_current_nmi = match caller {
            SpriteMainCpuCaller::DungeonModule07 => {
                self.dungeon_quadrant_cpu_continuation_active && nmi_slices == 1
            }
            SpriteMainCpuCaller::DungeonModule07Live { boundary } => {
                match boundary {
                    OriginalTimingBoundary::HostReturn => self
                        .game_execution_scheduler
                        .schedule_work(continuation, nmi_slices),
                    OriginalTimingBoundary::NmiAccepted => self
                        .game_execution_scheduler
                        .schedule_after_current_trailing_nmi(continuation),
                }
                return;
            }
            SpriteMainCpuCaller::WorldMapOverlayReload { .. } => {
                self.game_execution_scheduler
                    .schedule_cpu_timed_work_resuming_after_current_trailing_nmi(
                        continuation,
                        nmi_slices,
                    );
                return;
            }
            SpriteMainCpuCaller::Module09 { boundary }
            | SpriteMainCpuCaller::BossVictory { boundary }
            | SpriteMainCpuCaller::SaveAndQuit { boundary } => {
                match boundary {
                    OriginalTimingBoundary::HostReturn => self
                        .game_execution_scheduler
                        .schedule_work(continuation, nmi_slices),
                    OriginalTimingBoundary::NmiAccepted => self
                        .game_execution_scheduler
                        .schedule_after_current_trailing_nmi(continuation),
                }
                return;
            }
        };
        if resumes_after_current_nmi {
            self.game_execution_scheduler
                .schedule_after_current_trailing_nmi(continuation);
        } else {
            self.game_execution_scheduler
                .schedule_work(continuation, nmi_slices);
        }
    }

    pub(super) fn complete_sprite_main_after_interrupted_slot(&mut self, interrupted_slot: usize) {
        for k in (0..interrupted_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(k as u8);
            self.sprite_execute_single(k);
            debug_assert!(!self.game_execution_scheduler.work_is_pending());
        }
        self.complete_sprite_main_after_all_slots();
        let suffix_nmi_slices =
            std::mem::take(&mut self.dungeon_room_load_module_suffix_nmi_slices);
        if suffix_nmi_slices != 0 {
            self.game_execution_scheduler
                .schedule_cpu_timed_work_before_trailing_nmi(
                    GameWorkContinuation::FinishDungeonSupertileTransition {
                        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
                    },
                    suffix_nmi_slices,
                );
        }
    }

    /// Advance an already-suspended descending `Sprite_Main` slot loop to a
    /// later source statement without replaying its prefix or completing its
    /// caller.  Both boundaries name calls which have returned, so the exact
    /// semantic delta is the intervening descending slot range.
    pub(super) fn advance_sprite_main_after_slot_boundary(
        &mut self,
        completed_slot: u8,
        newly_completed_slot: u8,
    ) {
        assert!(
            newly_completed_slot < completed_slot,
            "Sprite_Main source boundary did not advance: {completed_slot} -> {newly_completed_slot}",
        );
        for slot in (newly_completed_slot..completed_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(slot);
            self.sprite_execute_single(usize::from(slot));
        }
    }

    pub(super) fn complete_sprite_main_after_cpu_boundary(
        &mut self,
        boundary: SpriteMainCpuBoundary,
    ) {
        match boundary {
            SpriteMainCpuBoundary::BeforeFirstSlot => {
                if let Some((main_module, submodule)) = self.pending_module09_frame_advance.take() {
                    // The suspended Module09 handler's deferred advance lands
                    // right before Sprite_Main, as the ROM writes it.
                    if self.pending_overworld_sprite_reload_slots.is_some() {
                        self.publish_deferred_module09_sprite_slots_at_reload_return();
                    }
                    self.set_submodule(submodule);
                    self.set_main_module(main_module);
                }
                self.sprite_main()
            }
            SpriteMainCpuBoundary::AfterSlot(interrupted_slot) => {
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot as usize)
            }
            SpriteMainCpuBoundary::AfterTimersAndOam {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_execute_single_after_timers(interrupted_slot, state);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterTimersAndOam { state: None, .. } => unreachable!(
                "source timer/OAM boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                let expected_state = match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => 8,
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => 9,
                };
                assert_eq!(
                    self.sprite_slot_view(interrupted_slot).state(),
                    expected_state,
                    "resumed sprite-init reset boundary lost its source state",
                );
                self.sprite_prep_reset_properties_from(interrupted_slot, completed_stores);
                match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                        self.sprite_module_initialize_properties_after_reset(interrupted_slot);
                        self.sprite_module_initialize_after_properties(interrupted_slot);
                    }
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                        self.sprite_prep_load_properties_after_reset(interrupted_slot);
                        self.sprite_prep_fire_debirando_after_property_reload(interrupted_slot);
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                let expected_state = match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => 8,
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => 9,
                };
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), expected_state);
                self.sprite_prep_load_properties_after_reset_from(
                    interrupted_slot,
                    completed_stores,
                );
                match phase {
                    crate::SpriteInitializeResetPropertiesPhase::InitialPropertyLoad => {
                        self.sprite_slot_view_mut(interrupted_slot).increment_state();
                        self.sprite_module_initialize_after_properties(interrupted_slot);
                    }
                    crate::SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion => {
                        self.sprite_prep_fire_debirando_after_property_reload(interrupted_slot);
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::FireDebirandoBeforeSpawn(slot) => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x63);
                self.sprite_prep_debirando_pit_after_before_spawn(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            } => {
                let interrupted_slot = usize::from(slot);
                let spawned = usize::from(spawned_slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert_eq!(self.sprite_slot_view(interrupted_slot).sprite_type(), 0x63);
                assert_eq!(self.sprite_slot_view(spawned).sprite_type(), 0x64);
                let mut info = SpriteSpawnInfo::default();
                self.sprite_spawn_dynamically_selected_from(
                    interrupted_slot,
                    &mut info,
                    spawned,
                    progress,
                );
                self.sprite_prep_debirando_pit_after_spawn(interrupted_slot, spawned, &info);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_antfairy_after_subtype2_increment(
                    interrupted_slot,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "source Antfairy subtype boundary did not bind to a native draw caller"
            ),
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_lanmola_after_subtype2_increment(interrupted_slot, continuation);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "source Lanmola subtype boundary did not bind to a native draw caller"
            ),
            SpriteMainCpuBoundary::AfterTimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                self.sprite_execute_single_after_timers(interrupted_slot, state);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterTimerDecrements { state: None, .. } => unreachable!(
                "source timer decrement boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_timers_and_oam_after_primary_through_timer_decrements(interrupted_slot);
                self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                self.sprite_execute_single_after_timers(interrupted_slot, state);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterPrimaryTimerDecrements { state: None, .. } => unreachable!(
                "source primary timer decrement boundary did not bind to the saved native dispatch state"
            ),
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements {
                slot,
                state: Some(state),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(
                    interrupted_slot,
                );
                self.sprite_timers_and_oam_after_primary_through_timer_decrements(interrupted_slot);
                self.sprite_timers_and_oam_after_timer_decrements(interrupted_slot);
                self.sprite_execute_single_after_timers(interrupted_slot, state);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterMainAndAux1TimerDecrements { state: None, .. } => {
                unreachable!(
                    "source main/aux1 timer decrement boundary did not bind to the saved native dispatch state"
                )
            }
            SpriteMainCpuBoundary::BariBeforeRandom(slot) => {
                let interrupted_slot = usize::from(slot);
                assert_eq!(self.sprite_slot_view(interrupted_slot).state(), 9);
                assert!(matches!(
                    self.sprite_slot_view(interrupted_slot).sprite_type(),
                    0x23 | 0x24
                ));
                self.sprite_system_mut().set_cur_object_index(slot);
                self.sprite_prep_bari_after_random_boundary(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.throwable_scenery_scatter_after_state_clear(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                slot,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_sprite_slot_after_single_small_draw_position(
                    interrupted_slot,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                slot,
                oam_position: Some(oam_position),
            } => {
                let interrupted_slot = usize::from(slot);
                self.sprite_system_mut().set_cur_object_index(slot);
                self.complete_probe_after_oam_coordinates(interrupted_slot, oam_position);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterWallmasterResetPrefix(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.sprite_system_mut()
                    .set_cur_object_index(interrupted_slot as u8);
                self.wall_master_send_player_after_reset_fixed_prefix();
                self.link_initialize();
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::ZazakAfterGraphics(interrupted_slot) => {
                let interrupted_slot = usize::from(interrupted_slot);
                self.sprite_zazak_after_graphics_boundary(interrupted_slot);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterActiveCuccoX {
                slot,
                helper_ordinal,
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_active_cucco_after_x(interrupted_slot, helper_ordinal);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
                y_low: Some(y_low),
                y_high: Some(y_high),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_active_cucco_after_y_subpixel(
                    interrupted_slot,
                    helper_ordinal,
                    y_low,
                    y_high,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_flee_after_movement(interrupted_slot, helper_ordinal);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                slot,
                completed,
                total,
                continuation: Some(continuation),
                ..
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_after_subtype_increments(
                    interrupted_slot,
                    completed,
                    total,
                    continuation,
                );
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal: _,
                continuation: Some(continuation),
            } => {
                let interrupted_slot = usize::from(slot);
                self.complete_cucco_after_graphics_publication(interrupted_slot, continuation);
                self.complete_sprite_main_after_interrupted_slot(interrupted_slot);
            }
            SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                continuation: None, ..
            }
            | SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel { .. }
            | SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                continuation: None, ..
            } => unreachable!("source Cucco boundary did not bind to a native C call site"),
            SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                continuation: None, ..
            } => unreachable!(
                "source single-small draw boundary did not bind to a native C call site"
            ),
            SpriteMainCpuBoundary::ProbeAfterOamCoordinates {
                oam_position: None,
                ..
            } => unreachable!(
                "source guard-probe boundary did not bind to its native OAM coordinates"
            ),
            SpriteMainCpuBoundary::BigKeyDropGraphicsStarted(_) => {
                unreachable!(
                    "big-key graphics boundary transfers directly to its typed continuation"
                )
            }
            SpriteMainCpuBoundary::ItemReceiptGraphicsStarted(_) => {
                unreachable!(
                    "item-receipt graphics boundary transfers directly to its typed continuation"
                )
            }
            SpriteMainCpuBoundary::KingZoraFlippersGraphicsStarted(slot) => {
                let slot = usize::from(slot);
                self.DecodeAnimatedSpriteTile_variable(0x11);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
                slot,
                saved_follower_indicator,
            } => {
                let slot = usize::from(slot);
                self.load_follower_graphics();
                self.sprite_prep_zelda_after_follower_graphics(slot, saved_follower_indicator);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller,
                prefix_completed: true,
                saved_follower_indicator,
                stage,
            } => {
                let slot = usize::from(slot);
                self.complete_follower_graphics_decompression(stage);
                match caller {
                    crate::SpriteFollowerGraphicsCaller::BlindMaiden => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_prep_blind_maiden_after_follower_graphics();
                    }
                    crate::SpriteFollowerGraphicsCaller::Zelda => {
                        self.sprite_prep_zelda_after_follower_graphics(
                            slot,
                            saved_follower_indicator.expect(
                                "Zelda follower-graphics continuation lost its saved follower",
                            ),
                        );
                    }
                    crate::SpriteFollowerGraphicsCaller::BlindMaidenBody => {
                        assert_eq!(saved_follower_indicator, None);
                        self.sprite_b7_blind_maiden_after_follower_graphics(slot);
                    }
                }
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
            SpriteMainCpuBoundary::FollowerGraphics {
                prefix_completed: false,
                ..
            } => unreachable!("source follower-graphics boundary did not bind its caller prefix"),
            SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(_) => {
                unreachable!("unstarted Zelda prep boundary cannot complete scheduled work")
            }
            SpriteMainCpuBoundary::BonkItemGraphicsEntered(slot) => {
                // The initialize prefix already ran at suspension time; only
                // the interrupted sheet decode and the loop below this slot
                // remain (SpritePrep_BonkItem's room-$107 branch has no
                // statements after the decode).
                let slot = usize::from(slot);
                self.DecodeAnimatedSpriteTile_variable(0x0e);
                self.complete_sprite_main_after_interrupted_slot(slot);
            }
        }
    }

    pub(super) fn complete_sprite_slot_after_single_small_draw_position(
        &mut self,
        slot: usize,
        continuation: SingleSmallDrawContinuation,
    ) {
        assert_eq!(self.sprite_slot_view(slot).state(), 9);
        assert!(matches!(
            self.sprite_slot_view(slot).sprite_type(),
            0x23 | 0x24
        ));
        self.sprite_system_mut().set_cur_object_index(slot as u8);
        self.sprite_draw_single_small_after_position(slot, continuation);
        self.sprite_23_red_bari_after_draw(slot);
    }

    fn complete_sprite_main_after_all_slots(&mut self) {
        // Every Sprite_Main body reaching its slot-zero return crosses this
        // boundary, whether it ran whole or resumed from a saved CPU slot, so
        // the one claimed source return is consumed here exactly once.
        if let Some(claims_remaining) = self.original_timing_sprite_main_return_claims_remaining {
            assert_ne!(
                claims_remaining, 0,
                "the native body returned from more Sprite_Main loops than its immutable host plan claimed: host={} frame={:?} scheduler={:?}",
                self.frame_ctr_dbg, self.game_state.frame, self.game_execution_scheduler,
            );
            assert!(
                self.take_original_timing_sprite_main_returned(),
                "the native Sprite_Main slot-zero boundary lost its claimed source return receipt",
            );
            self.original_timing_sprite_main_return_claims_remaining = Some(claims_remaining - 1);
        }
        self.garnish_execute_lower_slots();
        self.clear_overworld_vertical_scroll_delta_low();
        self.set_overworld_horizontal_scroll_delta_low(0);
        self.execute_cached_sprites();
        if self.game_state.display.has_chr_halfslot_request() {
            let chr_halfslot_request = self.game_state.display.chr_halfslot_request;
            self.sprite_system_mut()
                .set_chr_halfslot_state(chr_halfslot_request);
        }
    }

    // void Sprite_ExecuteSingle(int k) {  // 8684e2
    //   uint8 st = sprite_state[k];
    //   if (st != 0)
    //     Sprite_TimersAndOam(k);
    //   kSprite_ExecuteSingle[st](k);
    // }
    pub(super) fn sprite_execute_single(&mut self, k: usize) {
        let st = self.sprite_slot_view(k).state();
        if st != 0 {
            self.sprite_timers_and_oam(k);
        }
        self.sprite_execute_single_after_timers(k, st);
    }

    fn sprite_execute_single_after_timers(&mut self, k: usize, st: u8) {
        match st {
            0 => self.sprite_inactive_sprite(k),
            1 => self.sprite_module_fall1(k),
            2 => self.sprite_module_poof(k),
            3 => self.sprite_module_drown(k),
            4 => self.sprite_module_explode(k),
            5 => self.sprite_module_fall2(k),
            6 => self.sprite_module_die(k),
            7 => self.sprite_module_burn(k),
            8 => self.sprite_module_initialize(k),
            9 => self.sprite_active_main(k),
            10 => self.sprite_module_carried(k),
            11 => self.sprite_module_stunned(k),
            _ => self.sprite_active_main(k),
        }
    }

    // void ExecuteCachedSprites() {  // 9de9da
    //   ...see sprite.c...
    // }
    pub(super) fn execute_cached_sprites(&mut self) {
        if self.game_state.world.location.is_outdoors()
            || self.game_state.frame.submodule == 0
            || self.game_state.frame.submodule == 14
            || self.game_state.sprites.system.alt_sprites_flag() == 0
        {
            self.sprite_system_mut().clear_alt_sprites_flag();
            return;
        }
        let interruption = self.take_dungeon_cached_sprite_cpu_interruption();
        if crate::zelda_rtl::debug_cached_sprite_cpu_for_host(self.frame_ctr_dbg) {
            eprintln!(
                "cached_sprite_native host={} state={:02x}/{:02x}/{:02x} interruption={interruption:?}",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
            );
        }
        for i in (0..16usize).rev() {
            self.sprite_system_mut().set_cur_object_index(i as u8);
            if self.cached_sprite_slot(i).is_active() {
                if interruption.is_some_and(|(boundary, _)| usize::from(boundary.slot()) == i) {
                    let mut live_slot_backup = [0; 24];
                    let (mut boundary, authority_boundary) = interruption.unwrap();
                    match boundary {
                        CachedSpriteCpuInterruption::Loading { copied_fields, .. } => {
                            self.cached_sprite_slot_mut(i)
                                .load_cached_fields_into_live_before_nmi(
                                    &mut live_slot_backup,
                                    usize::from(copied_fields),
                                );
                        }
                        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                            continuation: None,
                            ..
                        } => {
                            self.cached_sprite_slot_mut(i)
                                .load_cached_into_live(&mut live_slot_backup);
                            assert_eq!(
                                self.sprite_slot_view(i).state(),
                                9,
                                "cached Antfairy body checkpoint requires an active sprite",
                            );
                            self.sprite_timers_and_oam(i);
                            let continuation = self.antfairy_draw_continuation(i);
                            self.sprite_slot_view_mut(i).add_subtype2(1);
                            boundary = CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                                slot: i as u8,
                                continuation: Some(continuation),
                            };
                        }
                        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                            continuation: Some(_),
                            ..
                        } => unreachable!(
                            "cached Antfairy boundary was already bound before entering its native call site"
                        ),
                        CachedSpriteCpuInterruption::Restoring { live_fields, .. } => {
                            self.cached_sprite_slot_mut(i)
                                .load_cached_into_live(&mut live_slot_backup);
                            self.sprite_execute_single(i);
                            if self.sprite_slot_view(i).pause() != 0 {
                                self.cached_sprite_slot_mut(i).clear_state();
                            }
                            self.cached_sprite_slot_mut(i)
                                .restore_live_suffix_from_backup_before_nmi(
                                    &live_slot_backup,
                                    usize::from(live_fields),
                                );
                        }
                    }
                    let dungeon = self
                        .active_dungeon_sprite_main_return
                        .take()
                        .expect("cached-sprite interruption must suspend a Module 7 sprite loop");
                    self.stage_interrupted_sprite_main_oam_scanout();
                    let continuation = GameWorkContinuation::FinishDungeonCachedSpriteMain {
                        boundary,
                        live_slot_backup,
                        dungeon,
                    };
                    match authority_boundary {
                        Some(OriginalTimingBoundary::NmiAccepted) => self
                            .game_execution_scheduler
                            .schedule_after_current_trailing_nmi(continuation),
                        Some(OriginalTimingBoundary::HostReturn) => {
                            self.game_execution_scheduler.schedule_work(continuation, 1);
                        }
                        None => {
                            if !self
                                .game_execution_scheduler
                                .schedule_after_pending_pre_main_nmi(continuation)
                            {
                                self.game_execution_scheduler.schedule_work(continuation, 1);
                            }
                        }
                    }
                    return;
                }
                self.uncache_and_execute_sprite(i);
            }
        }
    }

    pub(super) fn complete_cached_sprite_main_after_interrupted_slot(
        &mut self,
        boundary: CachedSpriteCpuInterruption,
        live_slot_backup: &[u8; 24],
    ) {
        let interrupted_slot = usize::from(boundary.slot());
        let mut live_slot_backup = *live_slot_backup;
        match boundary {
            CachedSpriteCpuInterruption::Loading { copied_fields, .. } => {
                self.cached_sprite_slot_mut(interrupted_slot)
                    .complete_cached_dynamic_fields_into_live_after_nmi(
                        &mut live_slot_backup,
                        usize::from(copied_fields),
                    );
                self.sprite_execute_single(interrupted_slot);
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                }
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                continuation: Some(continuation),
                ..
            } => {
                self.complete_antfairy_after_subtype2_increment(interrupted_slot, continuation);
                if self.sprite_slot_view(interrupted_slot).pause() != 0 {
                    self.cached_sprite_slot_mut(interrupted_slot).clear_state();
                }
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_from_backup(&live_slot_backup);
            }
            CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
                continuation: None,
                ..
            } => unreachable!(
                "cached Antfairy subtype boundary did not bind to a native draw caller"
            ),
            CachedSpriteCpuInterruption::Restoring { live_fields, .. } => {
                self.cached_sprite_slot_mut(interrupted_slot)
                    .restore_live_prefix_from_backup_after_nmi(
                        &live_slot_backup,
                        usize::from(live_fields),
                    );
            }
        }

        for i in (0..interrupted_slot).rev() {
            self.sprite_system_mut().set_cur_object_index(i as u8);
            if self.cached_sprite_slot(i).is_active() {
                self.uncache_and_execute_sprite(i);
            }
        }
        if self.game_state.display.has_chr_halfslot_request() {
            let chr_halfslot_request = self.game_state.display.chr_halfslot_request;
            self.sprite_system_mut()
                .set_chr_halfslot_state(chr_halfslot_request);
        }
    }

    // void UncacheAndExecuteSprite(int k) {  // 9dea00
    //   ...see sprite.c...
    // }
    pub(super) fn uncache_and_execute_sprite(&mut self, k: usize) {
        let mut bak = [0u8; 24];
        self.cached_sprite_slot_mut(k)
            .load_cached_into_live(&mut bak);
        self.sprite_execute_single(k);
        if self.sprite_slot_view(k).pause() != 0 {
            self.cached_sprite_slot_mut(k).clear_state();
        }
        self.cached_sprite_slot_mut(k)
            .restore_live_from_backup(&bak);
    }

    // void Dungeon_CacheTransSprites() {  // 89c176
    //   ...see sprite.c...
    // }
    pub(super) fn dungeon_cache_trans_sprites(&mut self) {
        if self.game_state.world.location.is_outdoors() {
            return;
        }
        let value = self.game_state.world.location.indoor_flag();
        self.sprite_system_mut().set_alt_sprites_flag(value);
        for k in (0..16usize).rev() {
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
    }

    fn dungeon_cache_trans_sprites_through_cpu_progress(
        &mut self,
        target_slot: u8,
        through: CachedSpriteCacheField,
    ) {
        assert!(self.game_state.world.location.is_indoors());
        let indoor_flag = self.game_state.world.location.indoor_flag();
        self.sprite_system_mut().set_alt_sprites_flag(indoor_flag);
        for k in (usize::from(target_slot)..16).rev() {
            if k == usize::from(target_slot) {
                assert!(through <= self.dungeon_cache_last_field(k));
                self.dungeon_cache_trans_sprite_fields(k, None, through);
                return;
            }
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
        unreachable!("cache checkpoint slot is outside the 16-slot C loop");
    }

    fn dungeon_resume_cache_trans_sprites_after_cpu_progress(
        &mut self,
        target_slot: u8,
        completed: CachedSpriteCacheField,
    ) {
        assert!(self.game_state.world.location.is_indoors());
        let target_slot = usize::from(target_slot);
        assert!(completed <= self.dungeon_cache_last_field(target_slot));
        self.dungeon_cache_trans_sprite_fields(
            target_slot,
            Some(completed),
            self.dungeon_cache_last_field(target_slot),
        );
        for k in (0..target_slot).rev() {
            self.dungeon_cache_trans_sprite_fields(k, None, self.dungeon_cache_last_field(k));
        }
    }

    fn dungeon_cache_last_field(&self, k: usize) -> CachedSpriteCacheField {
        let slot = self.sprite_slot_view(k);
        if slot.pause() != 0 || matches!(slot.state(), 4 | 10) {
            CachedSpriteCacheField::YHigh
        } else {
            CachedSpriteCacheField::IgnoreProjectile
        }
    }

    fn dungeon_cache_trans_sprite_fields(
        &mut self,
        k: usize,
        completed: Option<CachedSpriteCacheField>,
        through: CachedSpriteCacheField,
    ) {
        for field in CachedSpriteCacheField::C_SOURCE_ORDER {
            if completed.is_none_or(|completed| field > completed) && field <= through {
                self.cached_sprite_slot_mut(k).cache_field_from_live(field);
            }
        }
    }

    pub(super) fn oam_allocate_from_region_a(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 0)
    }

    pub(super) fn oam_allocate_from_region_b(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 2)
    }

    pub(super) fn oam_allocate_from_region_c(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 4)
    }

    pub(super) fn oam_allocate_from_region_d(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 6)
    }

    pub(super) fn oam_allocate_from_region_e(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 8)
    }

    pub(super) fn oam_allocate_from_region_f(&mut self, num: u8) -> u16 {
        self.oam_get_buffer_position(num, 10)
    }

    // void Sprite_TimersAndOam(int k) {  // 8683f2
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_timers_and_oam(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_timer_decrements(k);
        self.sprite_timers_and_oam_after_timer_decrements(k);
    }

    fn sprite_timers_and_oam_through_timer_decrements(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_primary_timer_decrements(k);
        self.sprite_timers_and_oam_after_primary_through_timer_decrements(k);
    }

    fn sprite_timers_and_oam_through_primary_timer_decrements(&mut self, k: usize) {
        self.sprite_timers_and_oam_through_main_and_aux1_timer_decrements(k);
        self.sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(k);
    }

    fn sprite_timers_and_oam_through_main_and_aux1_timer_decrements(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);

        let num = ((self.sprite_slot_view(k).flags2() & 0x1f).wrapping_add(1)).wrapping_mul(4);
        if self.game_state.oam.has_sprite_sorting() {
            if self.sprite_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_f(num);
            } else {
                self.oam_allocate_from_region_d(num);
            }
        } else {
            self.oam_allocate_from_region_a(num);
        }

        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let value = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            if self.sprite_slot_view(k).delay_aux1() != 0 {
                let value = self.sprite_slot_view(k).delay_aux1().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
            }
        }
    }

    fn sprite_timers_and_oam_after_main_and_aux1_through_primary_timer_decrements(
        &mut self,
        k: usize,
    ) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            if self.sprite_slot_view(k).delay_aux2() != 0 {
                let value = self.sprite_slot_view(k).delay_aux2().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux2(value);
            }
            if self.sprite_slot_view(k).delay_aux3() != 0 {
                let value = self.sprite_slot_view(k).delay_aux3().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux3(value);
            }
        }
    }

    fn sprite_timers_and_oam_after_primary_through_timer_decrements(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            let timer = self.sprite_slot_view(k).hit_timer() & 0x7f;
            if timer != 0 {
                if self.sprite_slot_view(k).state() >= 9 {
                    if timer == 31 {
                        self.sprite_hit_timer31(k);
                    } else if timer == 24 {
                        self.sprite_mini_moldorm_recoil(k);
                    }
                }
                if self.sprite_slot_view(k).incoming_damage() < 251 {
                    let value =
                        ((u16::from(self.sprite_slot_view(k).hit_timer()) * 2) & 0x0e) as u8;
                    self.sprite_slot_view_mut(k).set_object_priority(value);
                }
                let value = self.sprite_slot_view(k).hit_timer().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_hit_timer(value);
            } else {
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_object_priority(value);
            }

            if self.sprite_slot_view(k).delay_aux4() != 0 {
                let value = self.sprite_slot_view(k).delay_aux4().wrapping_sub(1);
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
            }
        }
    }

    fn sprite_timers_and_oam_after_timer_decrements(&mut self, k: usize) {
        let mut floor = self.game_state.player.follower_link.lower_level_state() as usize;
        if floor != 3 {
            floor = self.sprite_slot_view(k).floor() as usize;
        }
        let value = (self.sprite_slot_view(k).object_priority() & 0xcf)
            | SPRITE_TIMERS_AND_OAM_SPRITE_PRIOS[floor];
        self.sprite_slot_view_mut(k).set_object_priority(value);
    }

    pub(super) fn oam_get_buffer_position(&mut self, num: u8, y: u8) -> u16 {
        let region = (y >> 1) as usize;
        let mut pstart = self.game_state.oam.region_base_word(region);
        let p = pstart.wrapping_add(num as u16);
        if p >= OAM_GET_BUFFER_POSITION_LIMITS[region] {
            let alloc = self.game_state.oam.region_alloc_counter(region);
            let j = alloc & 7;
            self.oam_state_mut()
                .set_region_alloc_counter(region, alloc.wrapping_add(1));
            pstart = OAM_GET_BUFFER_POSITION_FALLBACKS[region * 8 + j as usize];
        } else {
            self.oam_state_mut().set_region_base_word(region, p);
        }
        let oam = 0x0800 + pstart;
        self.oam_state_mut()
            .set_current_extended_pointer(0x0a20 + (pstart >> 2));
        self.oam_state_mut().set_current_pointer(oam);
        oam
    }

    pub(super) fn sprite_move_xy(&mut self, k: usize) {
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    /// Execute `Sprite_MoveXY` through the first assignment in
    /// `Sprite_MoveY`. The original C helper publishes X completely, then
    /// writes Y subpixel, low, and high in that order. Returning the already
    /// computed low/high bytes lets a host-boundary continuation finish those
    /// assignments without evaluating the velocity twice.
    pub(super) fn sprite_move_xy_through_y_subpixel(&mut self, k: usize) -> (u8, u8) {
        self.sprite_move_x(k);
        self.sprite_move_y_through_subpixel(k)
    }

    /// Execute `Sprite_MoveY` through its first source assignment. This is the
    /// exact suffix of `Sprite_MoveXY` after `Sprite_MoveX` has returned.
    pub(super) fn sprite_move_y_through_subpixel(&mut self, k: usize) -> (u8, u8) {
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
    pub(super) fn complete_sprite_move_y_after_subpixel(
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
    pub(super) fn sprite_move_xyz(&mut self, k: usize) {
        self.sprite_move_z(k);
        self.sprite_move_x(k);
        self.sprite_move_y(k);
    }

    pub(super) fn sprite_move_x(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_x();
    }

    pub(super) fn sprite_move_y(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_y();
    }

    pub(super) fn sprite_draw_shadow(&mut self, k: usize, x: u16) {
        if self.sprite_slot_view(k).pause() != 0
            || (self.sprite_slot_view(k).state() == 10
                && self.sprite_slot_view(k).draw_work_byte_3() == 3)
        {
            return;
        }
        let y = self
            .sprite_y(k)
            .wrapping_add(10)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam = (self.game_state.oam.current_pointer_usize())
            + usize::from(self.sprite_slot_view(k).flags2() & 0x1f) * 4;
        let flags = (self.sprite_slot_view(k).oam_flags()
            ^ self.sprite_slot_view(k).object_priority())
            & 0x30;
        if self.sprite_slot_view(k).flags3() & 0x20 != 0 {
            self.set_oam_helper1_at(oam, x, y.wrapping_add(1) as u8, 0x38, flags | 8, 0);
        } else {
            self.set_oam_helper1_at(oam, x, y as u8, 0x6c, flags | 8, 2);
        }
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
    pub(super) fn alloc_overlord(&self) -> i32 {
        let mut i = 7i32;
        while i >= 0
            && self
                .game_state
                .sprites
                .overlord_slots
                .slot(i as usize)
                .overlord_type()
                != 0
        {
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
    pub(super) fn overworld_alloc_sprite(&self, type_: u8) -> i32 {
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

    // uint16 Garnish_GetX(int k) {
    //   return garnish_x_lo[k] | garnish_x_hi[k] << 8;
    // }
    pub(super) fn garnish_get_x(&self, k: usize) -> u16 {
        self.garnish_slot_view(k).x()
    }

    // uint16 Garnish_GetY(int k) {
    //   return garnish_y_lo[k] | garnish_y_hi[k] << 8;
    // }
    pub(super) fn garnish_get_y(&self, k: usize) -> u16 {
        self.garnish_slot_view(k).y()
    }

    // bool Garnish_ReturnIfPrepFails(int k, Point16U *pt) {  // 86e75e
    //   uint16 x = Garnish_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Garnish_GetY(k) - BG2VOFS_copy2;
    //   if (x >= 256 || y >= 256) {
    //     garnish_type[k] = 0;
    //     return true;
    //   }
    //   pt->x = x;
    //   pt->y = y - 16;
    //   return false;
    // }
    pub(super) fn garnish_return_if_prep_fails(&mut self, k: usize, pt: &mut Point16U) -> bool {
        let x = self
            .garnish_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .garnish_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x >= 256 || y >= 256 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return true;
        }
        pt.x = x;
        pt.y = y.wrapping_sub(16);
        false
    }

    fn set_oam_plain_at_for_sprite(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut().write_entry(oam, x, y, charnum, flags);
        let ext_index = (oam - OAM_BUF) / 4;
        let value = big;
        self.oam_state_mut().set_extended_byte(ext_index, value);
    }

    // void Garnish_SparkleCommon(int k, uint8 shift) {  // 86dfb1
    //   static const uint8 kGarnishSparkle_Char[4] = {0x83, 0xc7, 0x80, 0xb7};
    //   uint8 t = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   int j = garnish_sprite[k];
    //   SetOamPlain(oam, pt.x, pt.y, kGarnishSparkle_Char[t],
    //                (sprite_oam_flags[j] | sprite_obj_prio[j]) & 0xf0 | 4, 0);
    // }
    pub(super) fn garnish_sparkle_common(&mut self, k: usize, shift: u8) {
        let t = usize::from(self.garnish_slot_view(k).countdown() >> shift);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        let flags = (self.sprite_slot_view(j).oam_flags()
            | self.sprite_slot_view(j).object_priority())
            & 0xf0
            | 4;
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH_SPARKLE_COMMON_GARNISH_SPARKLE_CHAR[t],
            flags,
            0,
        );
    }

    // void Garnish_DustCommon(int k, uint8 shift) {  // 86dfdc
    //   static const uint8 kRunningManDust_Char[3] = {0xdf, 0xcf, 0xa9};
    //   tmp_counter = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   SetOamPlain(oam, pt.x, pt.y, kRunningManDust_Char[tmp_counter], 0x24, 0);
    // }
    pub(super) fn garnish_dust_common(&mut self, k: usize, shift: u8) {
        let counter = self.garnish_slot_view(k).countdown() >> shift;
        self.temp_counter_mut().set(counter);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH_DUST_COMMON_RUNNING_MAN_DUST_CHAR
                [usize::from(self.game_state.scratch_counter.value())],
            0x24,
            0,
        );
    }

    // void Garnish12_Sparkle(int k) { Garnish_SparkleCommon(k, 2); }
    pub(super) fn garnish12_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 2);
    }

    // void Garnish_SimpleSparkle(int k) { Garnish_SparkleCommon(k, 3); }
    pub(super) fn garnish_simple_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 3);
    }

    // void Garnish14_KakKidDashDust(int k) { Garnish_DustCommon(k, 2); }
    pub(super) fn garnish14_kak_kid_dash_dust(&mut self, k: usize) {
        self.garnish_dust_common(k, 2);
    }

    // void Garnish_WaterTrail(int k) { Garnish_DustCommon(k, 3); }
    pub(super) fn garnish_water_trail(&mut self, k: usize) {
        self.garnish_dust_common(k, 3);
    }

    // void Garnish04_LaserTrail(int k) {  // 86e000
    //   static const uint8 kLaserBeamTrail_Char[2] = {0xd2, 0xf3};
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, kLaserBeamTrail_Char[garnish_oam_flags[k]], 0x25, 0);
    // }
    pub(super) fn garnish04_laser_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH04_LASER_TRAIL_LASER_BEAM_TRAIL_CHAR
                [usize::from(self.garnish_slot_view(k).oam_flags())],
            0x25,
            0,
        );
    }

    // void Garnish06_ZoroTrail(int k) {  // 86e025
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x75, sprite_oam_flags[j] | sprite_obj_prio[j], 0);
    // }
    pub(super) fn garnish06_zoro_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x75,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            0,
        );
    }

    // void Garnish01_FireSnakeTail(int k) {  // 86e03e
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x28, sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(super) fn garnish01_fire_snake_tail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x28,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            2,
        );
    }

    // void Garnish02_MothulaBeamTrail(int k) {  // 86e057
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), garnish_x_lo[k] - BG2HOFS_copy2, garnish_y_lo[k] - BG2VOFS_copy2, 0xaa,
    //                sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(super) fn garnish02_mothula_beam_trail(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            oam,
            self.garnish_slot_view(k)
                .x_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low()),
            self.garnish_slot_view(k)
                .y_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low()),
            0xaa,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            2,
        );
    }

    // void Garnish_CheckPlayerCollision(int k, int x, int y) {  // 89b459
    //   if ((k ^ frame_counter) & 7 | countdown_for_blink | link_disable_sprite_damage)
    //     return;
    //
    //   if ((uint8)(link_x_coord - BG2HOFS_copy2 - x + 12) < 24 &&
    //       (uint8)(link_y_coord - BG2VOFS_copy2 - y + 22) < 28) {
    //     link_auxiliary_state = 1;
    //     link_incapacitated_timer = 16;
    //     link_give_damage = 16;
    //     link_actual_vel_x ^= 255;
    //     link_actual_vel_y ^= 255;
    //   }
    // }
    /// `Garnish_CheckPlayerCollision` ($09:B459). The ROM chains 8-bit
    /// `SBC`/`ADC` without resetting the carry: the x test starts from the
    /// caller's carry (`carry_in`), each `SBC` borrows into the next, the
    /// `ADC #$0C` adds the final borrow flag, and the y test starts from the
    /// `CMP #$18` result (a passing x leaves the carry clear, so the y chain
    /// begins one lower). Route host 1555077: Ganon's bat flame at screen
    /// (0xa1, 0x7c) misses Link at (0xa3, 0x66) in the ROM (y chain yields
    /// 0xff), while the carry-free `link_y - bg - y + 22 < 28` form hits.
    pub(super) fn garnish_check_player_collision(
        &mut self,
        k: usize,
        x: i32,
        y: i32,
        carry_in: bool,
    ) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 7)
            | self.game_state.player.follower_link.blink_countdown()
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
            != 0
        {
            return;
        }

        fn sbc8(a: u8, b: u8, carry: bool) -> (u8, bool) {
            let r = i32::from(a) - i32::from(b) - i32::from(!carry);
            (r as u8, r >= 0)
        }
        fn adc8(a: u8, b: u8, carry: bool) -> (u8, bool) {
            let r = u32::from(a) + u32::from(b) + u32::from(carry);
            (r as u8, r > 0xff)
        }

        let link_x = self.game_state.player.follower_link.x() as u8;
        let link_y = self.game_state.player.follower_link.y() as u8;
        let bg_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2() as u8;
        let bg_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2() as u8;
        let (dx, carry) = sbc8(link_x, bg_x, carry_in);
        let (dx, carry) = sbc8(dx, x as u8, carry);
        let (dx, _) = adc8(dx, 0x0c, carry);
        if dx >= 0x18 {
            return;
        }
        // `CMP #$18` with dx < 0x18 clears the carry for the y chain.
        let (dy, carry) = sbc8(link_y, bg_y, false);
        let (dy, carry) = sbc8(dy, y as u8, carry);
        let (dy, _) = adc8(dy, 0x16, carry);
        if dy >= 0x1c {
            return;
        }
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().set_given_damage(16);
        self.follower_link_state_mut().xor_actual_velocity_xy(255);
    }

    // void Garnish15_ArrghusSplash(int k) {  // 89b178
    //   ...see sprite.c...
    // }
    pub(super) fn garnish15_arrghus_splash(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from((self.garnish_slot_view(k).countdown() >> 1) & 6);
        for i in (0..=1).rev() {
            let j = i + g;
            self.set_oam_plain_at_for_sprite(
                oam,
                pt.x.wrapping_add(GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_X[j] as i16 as u16) as u8,
                pt.y.wrapping_add(GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_Y[j] as i16 as u16) as u8,
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_CHAR[j],
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_FLAGS[j],
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_EXT[j],
            );
            oam += 4;
        }
    }

    // void Garnish13_PyramidDebris(int k) {  // 89b216
    //   ...see sprite.c...
    // }
    pub(super) fn garnish13_pyramid_debris(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();

        let garnish = self.garnish_slot_view(k);
        let y = (i32::from(garnish.y_low()) << 8)
            + i32::from(garnish.y_subpixel())
            + ((garnish.y_velocity() as i8 as i32) << 4);
        let value = y as u8;
        self.garnish_slot_view_mut(k).set_y_subpixel(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(k).set_y_low(value);

        let garnish = self.garnish_slot_view(k);
        let x = (i32::from(garnish.x_low()) << 8)
            + i32::from(garnish.x_subpixel())
            + ((garnish.x_velocity() as i8 as i32) << 4);
        let value = x as u8;
        self.garnish_slot_view_mut(k).set_x_subpixel(value);
        let value = (x >> 8) as u8;
        self.garnish_slot_view_mut(k).set_x_low(value);

        let value = self.garnish_slot_view(k).y_velocity().wrapping_add(3);
        self.garnish_slot_view_mut(k).set_y_velocity(value);
        let t = self
            .garnish_slot_view(k)
            .x_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
        if t >= 248 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return;
        }
        self.oam_state_mut().set_entry_x(oam, t);
        let t = self
            .garnish_slot_view(k)
            .y_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        if t >= 240 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return;
        }
        self.oam_state_mut().set_entry_y(oam, t);
        self.oam_state_mut().set_entry_char(oam, 0x5c);
        let flags = (self.game_state.frame.frame_counter << 3) & 0xc0 | 0x34;
        self.oam_state_mut().set_entry_flags(oam, flags);
        let ext_index = (oam - OAM_BUF) / 4;
        let value = 0;
        self.oam_state_mut().set_extended_byte(ext_index, value);
    }

    // void Garnish11_WitheringGanonBatFlame(int k) {  // 89b2b2
    //   ...see sprite.c...
    // }
    pub(super) fn garnish11_withering_ganon_bat_flame(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            let y = self.garnish_get_y(k).wrapping_sub(1);
            self.garnish_set_y(k, y);
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at_for_sprite(oam, pt.x as u8, pt.y as u8, 0xa4, 0x22, 0);
        self.set_oam_plain_at_for_sprite(
            oam + 4,
            pt.x.wrapping_add(8) as u8,
            pt.y as u8,
            0xa5,
            0x22,
            0,
        );
    }

    // void Garnish10_GanonBatFlame(int k) {  // 89b306
    //   ...see sprite.c...
    // }
    pub(super) fn garnish10_ganon_bat_flame(&mut self, k: usize) {
        if self.garnish_slot_view(k).countdown() == 8 {
            let value = 0x11;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_IDX
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
        );
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_CHAR[j],
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_FLAGS[j] | 0x22,
            2,
        );
        // The ROM reaches the collision check with the carry left by the
        // `LSR A` x3 that indexed the frame table: bit 2 of the countdown.
        let carry_in = self.garnish_slot_view(k).countdown() & 0x04 != 0;
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y), carry_in);
    }

    // void Garnish0A_CannonSmoke(int k) {  // 89b3ee
    //   ...see sprite.c...
    // }
    pub(super) fn garnish0_a_cannon_smoke(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
            GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_FLAGS[j] | 4,
            2,
        );
    }

    fn dungeon_update_tile_map_with_common_tile_for_garnish(&mut self, x: u16, y: u16, v: u8) {
        self.Dungeon_UpdateTileMapWithCommonTile(i32::from(x), i32::from(y), v);
    }

    // void Garnish0C_TrinexxIceBreath(int k) {  // 89b34f
    pub(super) fn garnish0_c_trinexx_ice_breath(&mut self, k: usize) {
        if self.garnish_slot_view(k).countdown() == 0x50
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
        {
            self.dungeon_update_tile_map_with_common_tile_for_garnish(
                self.garnish_get_x(k),
                self.garnish_get_y(k).wrapping_sub(16),
                18,
            );
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 4)],
            GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_FLAGS
                [usize::from((self.garnish_slot_view(k).countdown() >> 2) & 3)]
                | 0x35,
            2,
        );
    }

    // void Garnish09_LightningTrail(int k) {  // 89b429
    //   ...see sprite.c...
    // }
    pub(super) fn garnish09_lightning_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        let room_offset = if self.game_state.dungeon.room_tracking.room_index2() == 0x20 {
            0x80
        } else {
            0
        };
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_CHAR[j].wrapping_sub(room_offset),
            (self.game_state.frame.frame_counter << 1) & 0x0e
                | GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_FLAGS[j],
            2,
        );
        // The ROM reaches the collision check with the carry left by the
        // `ASL A` on the frame counter: its bit 7.
        let carry_in = self.game_state.frame.frame_counter & 0x80 != 0;
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y), carry_in);
    }

    // void Garnish03_FallingTile(int k) {  // 89b627
    pub(super) fn garnish03_falling_tile(&mut self, k: usize) {
        let mut j = self.garnish_slot_view(k).countdown();
        if j == 0x1e {
            j = self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag;
            if j == 0 {
                self.dungeon_update_tile_map_with_common_tile_for_garnish(
                    self.garnish_get_x(k),
                    self.garnish_get_y(k).wrapping_sub(16),
                    4,
                );
            }
        }
        let j = usize::from(j >> 3);
        let x = self
            .garnish_get_x(k)
            .wrapping_add(u16::from(GARNISH03_FALLING_TILE_CRUMBLE_TILE_XY[j]))
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .garnish_get_y(k)
            .wrapping_add(u16::from(GARNISH03_FALLING_TILE_CRUMBLE_TILE_XY[j]))
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x < 256 && y < 256 {
            self.set_oam_plain_at_for_sprite(
                self.game_state.oam.current_pointer_usize(),
                x as u8,
                y.wrapping_sub(16) as u8,
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_CHAR[j],
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_FLAGS[j],
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_EXT[j],
            );
        }
    }

    // void Garnish07_BabasuFlash(int k) {  // 89b49e
    pub(super) fn garnish07_babasu_flash(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).countdown() >> 3);
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH07_BABASU_FLASH_BABUSU_FLASH_CHAR[j],
            GARNISH07_BABASU_FLASH_BABUSU_FLASH_FLAGS[j],
            2,
        );
    }

    // void Garnish08_KholdstareTrail(int k) {  // 89b4c6
    pub(super) fn garnish08_kholdstare_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let i = usize::from(self.garnish_slot_view(k).countdown() >> 2);
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x.wrapping_add(GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            pt.y.wrapping_add(GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_CHAR[i],
            (self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority())
                & !1,
            0,
        );
    }

    // void Garnish0E_TrinexxFireBreath(int k) {  // 89b55d
    pub(super) fn garnish0_e_trinexx_fire_breath(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_E_TRINEXX_FIRE_BREATH_TRINEXX_LAVA_BUBBLE_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
            (self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority())
                & 0xf0
                | 0x0e,
            0,
        );
    }

    // void Garnish0F_BlindLaserTrail(int k) {  // 89b591
    pub(super) fn garnish0_f_blind_laser_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at_for_sprite(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_F_BLIND_LASER_TRAIL_BLIND_LASER_TRAIL_CHAR
                [usize::from(self.garnish_slot_view(k).oam_flags().wrapping_sub(7))],
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            0,
        );
    }

    // void Garnish_ExecuteUpperSlots() {  // 89b08c
    //   HandleScreenFlash();
    //
    //   if (garnish_active) {
    //     for (int i = 29; i >= 15; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(super) fn garnish_execute_upper_slots(&mut self) {
        self.handle_screen_flash();

        if self.game_state.sprites.garnish_runtime.active_type() != 0 {
            for i in (15..=29).rev() {
                self.garnish_execute_single(i);
            }
        }
    }

    // void Garnish_ExecuteLowerSlots() {  // 89b097
    //   if (garnish_active) {
    //     for (int i = 14; i >= 0; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(super) fn garnish_execute_lower_slots(&mut self) {
        if self.game_state.sprites.garnish_runtime.active_type() != 0 {
            for i in (0..=14).rev() {
                self.garnish_execute_single(i);
            }
        }
    }

    // void Garnish_ExecuteSingle(int k) {  // 89b0b6
    //   ...see sprite.c...
    // }
    pub(super) fn garnish_execute_single(&mut self, k: usize) {
        self.sprite_system_mut().set_cur_object_index(k as u8);
        let type_ = self.garnish_slot_view(k).garnish_type();
        if type_ == 0 {
            return;
        }
        if (type_ == 5
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0)
            && self.garnish_slot_view(k).countdown() != 0
        {
            let value = self.garnish_slot_view(k).countdown().wrapping_sub(1);
            self.garnish_slot_view_mut(k).set_countdown(value);
            if self.garnish_slot_view(k).countdown() == 0 {
                let value = 0;
                self.garnish_slot_view_mut(k).set_garnish_type(value);
                return;
            }
        }

        let sprsize = GARNISH_EXECUTE_SINGLE_GARNISH_OAM_MEM_SIZE[usize::from(type_)];
        if self.game_state.oam.has_sprite_sorting() {
            if self.garnish_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_f(sprsize);
            } else {
                self.oam_allocate_from_region_d(sprsize);
            }
        } else {
            self.oam_allocate_from_region_a(sprsize);
        }

        match type_ {
            1 => self.garnish01_fire_snake_tail(k),
            2 => self.garnish02_mothula_beam_trail(k),
            3 => self.garnish03_falling_tile(k),
            4 => self.garnish04_laser_trail(k),
            5 => self.garnish_simple_sparkle(k),
            6 => self.garnish06_zoro_trail(k),
            7 => self.garnish07_babasu_flash(k),
            8 => self.garnish08_kholdstare_trail(k),
            9 => self.garnish09_lightning_trail(k),
            10 => self.garnish0_a_cannon_smoke(k),
            11 => self.garnish_water_trail(k),
            12 => self.garnish0_c_trinexx_ice_breath(k),
            13 => {}
            14 => self.garnish0_e_trinexx_fire_breath(k),
            15 => self.garnish0_f_blind_laser_trail(k),
            16 => self.garnish10_ganon_bat_flame(k),
            17 => self.garnish11_withering_ganon_bat_flame(k),
            18 => self.garnish12_sparkle(k),
            19 => self.garnish13_pyramid_debris(k),
            20 => self.garnish14_kak_kid_dash_dust(k),
            21 => self.garnish15_arrghus_splash(k),
            22 => self.garnish16_thrown_item_debris(k),
            _ => {}
        }
    }

    // void Sprite_Get16BitCoords(int k) {
    //   cur_sprite_x = sprite_x_lo[k] | sprite_x_hi[k] << 8;
    //   cur_sprite_y = sprite_y_lo[k] | sprite_y_hi[k] << 8;
    // }
    pub(super) fn sprite_get16_bit_coords(&mut self, k: usize) {
        let x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        self.sprite_workspace_mut().set_current_sprite_x(x);
        self.sprite_workspace_mut().set_current_sprite_y(y);
    }

    // void Sprite_inactiveSprite(int k) {  // 868510
    //   if (!player_is_indoors) {
    //     sprite_N_word[k] = 0xffff;
    //   } else {
    //     sprite_N[k] = 0xff;
    //   }
    // }
    pub(super) fn sprite_inactive_sprite(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_view_mut(k).set_n_word(0xffff);
        } else {
            let value = 0xff;
            self.sprite_slot_view_mut(k).set_n(value);
        }
    }

    // void Sprite_KillSelf(int k) {  // 89f1f8
    //   if (!(sprite_defl_bits[k] & 0x40) && player_is_indoors)
    //     return;
    //   sprite_state[k] = 0;
    //   uint16 blk = sprite_N_word[k];
    //   g_ram[0] = blk;
    //   WORD(g_ram[1]) = (blk >> 3) + 0xef80;
    //   uint8 loadedmask = (0x80 >> (blk & 7));
    //   uint16 addr = 0xEF80 + (blk >> 3);
    //   uint8 *loadedp = &g_ram[addr + 0x10000];
    //   if (blk < 0xffff)
    //     *loadedp &= ~loadedmask;
    //   if (!player_is_indoors)
    //     sprite_N_word[k] = 0xffff;
    //   else
    //     sprite_N[k] = 0xff;
    // }
    pub(super) fn sprite_kill_self(&mut self, k: usize) {
        if (self.sprite_slot_view(k).deflection_bits() & 0x40) == 0
            && self.game_state.world.location.is_indoors()
        {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        let blk = self.sprite_slot_view(k).n_word();
        self.sprite_workspace_mut()
            .set_killed_sprite_load_block(blk);
        let loadedmask = 0x80 >> (blk & 7);
        if blk < 0xffff {
            self.clear_overworld_sprite_loaded_mask_wrapped(blk, loadedmask as u8);
        }
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_view_mut(k).set_n_word(0xffff);
        } else {
            let value = 0xff;
            self.sprite_slot_view_mut(k).set_n(value);
        }
    }

    // void Sprite_HitTimer31(int k) {
    //   if (sprite_type[k] != 0x7a || is_in_dark_world)
    //     return;
    //   if (sprite_health[k] <= sprite_give_damage[k]) {
    //     dialogue_message_index = 0x140;
    //     Sprite_ShowMessageMinimal();
    //   }
    // }
    pub(super) fn sprite_hit_timer31(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() != 0x7a
            || self.game_state.world.region.is_in_dark_world()
        {
            return;
        }
        if self.sprite_slot_view(k).health() <= self.sprite_slot_view(k).incoming_damage() {
            self.dialogue_message_index_mut().set_value(0x140);
            self.sprite_show_message_minimal_c();
        }
    }

    // bool Sprite_TrackBodyToHead(int k) {  // 85dca2
    //   if (sprite_head_dir[k] != sprite_D[k]) {
    //     if (frame_counter & 0x1f)
    //       return false;
    //     if (!((sprite_head_dir[k] ^ sprite_D[k]) & 2)) {
    //       sprite_D[k] = (((k ^ frame_counter) >> 5 | 2) & 3) ^ (sprite_head_dir[k] & 2);
    //       return false;
    //     }
    //   }
    //   sprite_D[k] = sprite_head_dir[k];
    //   return true;
    // }
    pub(super) fn sprite_track_body_to_head(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).head_direction() != self.sprite_slot_view(k).direction() {
            if (self.game_state.frame.frame_counter & 0x1f) != 0 {
                return false;
            }
            if ((self.sprite_slot_view(k).head_direction() ^ self.sprite_slot_view(k).direction())
                & 2)
                == 0
            {
                let value = ((((k as u8) ^ self.game_state.frame.frame_counter) >> 5) | 2) & 3
                    ^ (self.sprite_slot_view(k).head_direction() & 2);
                self.sprite_slot_view_mut(k).set_direction(value);
                return false;
            }
        }
        let value = self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_direction(value);
        true
    }

    // bool Sprite_CheckIfLinkIsBusy() {  // 87f4d0
    //   if (link_auxiliary_state | link_pose_for_item | (link_state_bits & 0x80))
    //     return true;
    //   for (int i = 4; i >= 0; i--) {
    //     if (ancilla_type[i] == 0x27)
    //       return true;
    //   }
    //   return false;
    // }
    pub(super) fn sprite_check_if_link_is_busy(&self) -> bool {
        if self.game_state.player.follower_link.has_auxiliary_state()
            || self.game_state.player.follower_link.item_hold_pose() != 0
            || self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            return true;
        }
        for i in (0..=4usize).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x27 {
                return true;
            }
        }
        false
    }

    // bool Sprite_ReturnIfInactive(int k) {
    //   return (sprite_state[k] != 9 || modal_pause_flag || submodule_index
    //           || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    // Note: in C this returns true when the caller should bail. Same here.
    pub(super) fn sprite_return_if_inactive(&self, k: usize) -> bool {
        if self.sprite_slot_view(k).state() != 9 {
            return true;
        }
        if self.game_state.frame.modal_pause_flag != 0 || self.game_state.frame.submodule != 0 {
            return true;
        }
        (self.sprite_slot_view(k).deflection_bits() & 0x80) == 0
            && self.sprite_slot_view(k).pause() != 0
    }

    // bool Sprite_ReturnIfPaused(int k) {  // 86d9f3
    //   return (modal_pause_flag || submodule_index || !(sprite_defl_bits[k] & 0x80) && sprite_pause[k]);
    // }
    pub(super) fn sprite_return_if_paused(&self, k: usize) -> bool {
        self.game_state.frame.modal_pause_flag != 0
            || self.game_state.frame.submodule != 0
            || ((self.sprite_slot_view(k).deflection_bits() & 0x80) == 0
                && self.sprite_slot_view(k).pause() != 0)
    }

    // bool Sprite_ReturnIfPhasingOut(int k) {  // 86d0ed
    //   if (!sprite_stunned[k] || (submodule_index | modal_pause_flag))
    //     return false;
    //   if (!(frame_counter & 1))
    //     sprite_stunned[k]--;
    //   uint8 a = sprite_stunned[k];
    //   if (a == 0)
    //     sprite_state[k] = 0;
    //   else if (a >= 0x28 || (a & 1) != 0)
    //     return false;
    //   PrepOamCoordsRet info;
    //   Sprite_PrepOamCoordOrDoubleRet(k, &info);
    //   return true;
    // }
    pub(super) fn sprite_return_if_phasing_out(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).stunned() == 0
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0
        {
            return false;
        }
        if (self.game_state.frame.frame_counter & 1) == 0 {
            let value = self.sprite_slot_view(k).stunned().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_stunned(value);
        }
        let a = self.sprite_slot_view(k).stunned();
        if a == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        } else if a >= 0x28 || (a & 1) != 0 {
            return false;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        true
    }

    // bool SpriteDraw_AbsorbableTransient(int k, bool transient) {  // 86d22f
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_absorbable_transient(&mut self, k: usize, transient: bool) -> bool {
        if transient && self.sprite_return_if_phasing_out(k) {
            return false;
        }
        if !self.game_state.oam.has_sprite_sorting() && self.game_state.world.location.is_indoors()
        {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            return false;
        }
        if self.sprite_slot_view(k).delay_aux2() != 0 {
            self.oam_allocate_from_region_c(12);
        }
        if self.sprite_slot_view(k).e() != 0 {
            if self.game_state.enhanced_features.has(4096) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_b(value);
            }
            return true;
        }

        let j = self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8) as usize;
        let a = SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_GFX_BY_TYPE[j];
        if a != 0 {
            self.sprite_draw_numbered_absorbable(k, i32::from(a));
            return false;
        }

        let t = SPRITE_DRAW_ABSORBABLE_TRANSIENT_ABSORBABLE_OAM_EXT_SIZE_BY_TYPE[j];
        if t == 0 {
            self.sprite_draw_single_small(k);
            return false;
        }
        if t == 2 {
            if self.sprite_slot_view(k).sprite_type() == 0xe6 {
                if self.sprite_slot_view(k).subtype() == 1 {
                    self.sprite_draw_thin_and_tall(k);
                    return false;
                }
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            self.sprite_draw_single_large(k);
            return false;
        }
        self.sprite_draw_thin_and_tall(k);
        false
    }

    // void Sprite_DrawNumberedAbsorbable(int k, int a) {  // 86d2fa
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_numbered_absorbable(&mut self, k: usize, a: i32) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let base = ((a - 1) * 3).max(0) as usize;
        let n = if self.sprite_slot_view(k).head_direction() < 1 {
            2
        } else {
            1
        };
        for i in (0..=n).rev() {
            let j = (base + i).min(SPRITE_DRAW_NUMBERED_ABSORBABLE_CHARS.len() - 1);
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(SPRITE_DRAW_NUMBERED_ABSORBABLE_X_OFFSETS[j] as u16),
                y.wrapping_add(SPRITE_DRAW_NUMBERED_ABSORBABLE_Y_OFFSETS[j] as u16),
                SPRITE_DRAW_NUMBERED_ABSORBABLE_CHARS[j],
                flags,
                SPRITE_DRAW_NUMBERED_ABSORBABLE_EXT_SIZES[j],
            );
            oam += 4;
        }
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // void Sprite_HalveSpeed_XY(int k) {
    //   sprite_x_vel[k] = (int8)sprite_x_vel[k] >> 1;
    //   sprite_y_vel[k] = (int8)sprite_y_vel[k] >> 1;
    // }
    pub(super) fn sprite_halve_speed_xy(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.halve_x_velocity();
        sprite.halve_y_velocity();
    }

    // void Sprite_ApplyRicochet(int k) {  // 86e229
    //   Sprite_InvertSpeed_XY(k);
    //   Sprite_HalveSpeed_XY(k);
    //   ThrowableScenery_TransmuteIfValid(k);
    // }
    pub(super) fn sprite_apply_ricochet(&mut self, k: usize) {
        self.sprite_invert_speed_xy(k);
        self.sprite_halve_speed_xy(k);
        self.throwable_scenery_transmute_if_valid(k);
    }

    // void ThrowableScenery_TransmuteIfValid(int k) {  // 86e22f
    //   if (sprite_type[k] != 0xec)
    //     return;
    //   repulsespark_timer = 0;
    //   ThrowableScenery_TransmuteToDebris(k);
    // }
    pub(super) fn throwable_scenery_transmute_if_valid(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() != 0xec {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(0);
        self.throwable_scenery_transmute_to_debris(k);
    }

    // void ThrowableScenery_TransmuteToDebris(int k) {  // 86e239
    //   uint8 a = sprite_graphics[k];
    //   if (a != 0) {
    //     BYTE(dung_secrets_unk1) = a;
    //     Sprite_SpawnSecret(k);
    //     BYTE(dung_secrets_unk1) = 0;
    //   }
    //   a = player_is_indoors ? 0 : sprite_C[k];
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, kSprite_Func21_Sfx[a]);
    //   Sprite_ScheduleForBreakage(k);
    // }
    pub(super) fn throwable_scenery_transmute_to_debris(&mut self, k: usize) {
        let mut a = self.sprite_slot_view(k).graphics();
        if a != 0 {
            self.dungeon_secret_scratch_mut().set_pending_kind(a);
            self.sprite_spawn_secret(k);
            self.dungeon_secret_scratch_mut().clear_pending_kind();
        }
        a = if self.game_state.world.location.is_indoors() {
            0
        } else {
            self.sprite_slot_view(k).c()
        };
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(
            k,
            THROWABLE_SCENERY_TRANSMUTE_TO_DEBRIS_THROWN_SPRITE_IMPACT_SFX[a as usize],
        );
        self.sprite_schedule_for_breakage(k);
    }

    // void Sprite_Func18(int k, uint8 new_type) {  // 86edcb
    //   sprite_type[k] = new_type;
    //   SpritePrep_LoadProperties(k);
    //   Sprite_SpawnPoofGarnish(k);
    //   sound_effect_2 = 0;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x32);
    //   sprite_hit_timer[k] = 0;
    //   sprite_give_damage[k] = 0;
    // }
    pub(super) fn sprite_func18(&mut self, k: usize, new_type: u8) {
        let value = new_type;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        self.sprite_prep_load_properties_for_helpers(k);
        self.sprite_spawn_poof_garnish(k);
        self.set_sound_effect_2(0);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
        let value = 0;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_incoming_damage(value);
    }

    // void Sprite_Func15(int k, int a) {  // 86ed25
    //   damage_type_determiner = a;
    //   Sprite_ApplyCalculatedDamage(k, a == 8 ? 0x35 : 0x20);
    // }
    pub(super) fn sprite_func15(&mut self, k: usize, a: u8) {
        self.sprite_battle_mut().set_damage_type_determiner(a);
        self.sprite_apply_calculated_damage(k, if a == 8 { 0x35 } else { 0x20 });
    }

    // void Sprite_CalculateSwordDamage(int k) {  // 86ed3f
    //   if (sprite_flags3[k] & 0x40)
    //     return;
    //   sprite_unk1[k] = link_is_running;
    //   uint8 a = link_sword_type - 1;
    //   if (!link_is_running)
    //     a |= sign8(button_b_frames) ? 4 : sign8(button_b_frames - 9) ? 0 : 8;
    //   damage_type_determiner = kSprite_Func14_Damage[a];
    //   if (link_item_in_hand & 10)
    //     damage_type_determiner = 3;
    //   link_sword_delay_timer = 4;
    //   set_when_damaging_enemies = 16;
    //   Sprite_ApplyCalculatedDamage(k, 0x9d);
    // }
    pub(super) fn sprite_calculate_sword_damage(&mut self, k: usize) {
        if self.sprite_slot_view(k).flags3() & 0x40 != 0 {
            return;
        }
        let is_running = self.game_state.player.follower_link.is_running();
        let item_in_hand_has_sword_mask = self.game_state.player.follower_link.item_in_hand_has(10);
        let value = u8::from(is_running);
        self.sprite_slot_view_mut(k).set_draw_work_byte_1(value);
        let mut a = self.game_state.inventory.items.sword_type().wrapping_sub(1);
        if !is_running {
            a |= if sign8(self.game_state.player.follower_link.button_b_frames()) {
                4
            } else if sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                0
            } else {
                8
            };
        }
        self.sprite_battle_mut().set_damage_type_determiner(
            SPRITE_CALCULATE_SWORD_DAMAGE_SPRITE_DAMAGE_BY_PLAYER_WEAPON[a as usize],
        );
        if item_in_hand_has_sword_mask {
            self.sprite_battle_mut().set_damage_type_determiner(3);
        }
        self.follower_link_state_mut().set_sword_delay_timer(4);
        self.sprite_battle_mut().set_damaging_enemies_timer(16);
        self.sprite_apply_calculated_damage(k, 0x9d);
    }

    // void Sprite_ApplyCalculatedDamage(int k, int a) {  // 86ed89
    //   if ((sprite_flags3[k] & 0x40) || sprite_type[k] >= 0xD8)
    //     return;
    //   uint8 dmg = kEnemyDamages[damage_type_determiner * 8 | enemy_damage_data[sprite_type[k] * 16 | damage_type_determiner]];
    //   Sprite_GiveDamage(k, dmg, a);
    // }
    pub(super) fn sprite_apply_calculated_damage(&mut self, k: usize, a: u8) {
        if self.sprite_slot_view(k).flags3() & 0x40 != 0
            || self.sprite_slot_view(k).sprite_type() >= 0xd8
        {
            return;
        }
        let damage_type = self.game_state.sprite_battle.damage_type_determiner() as usize;
        let enemy_damage_index = self.sprite_slot_view(k).sprite_type() as usize * 16 + damage_type;
        let dmg = SPRITE_APPLY_CALCULATED_DAMAGE_ENEMY_CONTACT_DAMAGE_BY_TYPE[damage_type * 8
            | self
                .game_state
                .sprites
                .enemy_damage_subclasses
                .entry(enemy_damage_index) as usize];
        self.sprite_give_damage(k, dmg, a);
    }

    // void Sprite_GiveDamage(int k, uint8 dmg, uint8 r0_hit_timer) {  // 86edc5
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_give_damage(&mut self, k: usize, dmg: u8, r0_hit_timer: u8) {
        if std::env::var_os("ZELDA3_TRACE_GIVE_DAMAGE").is_some()
            && self.game_state.world.location.dungeon_room() == 0x00a8
            && self.sprite_slot_view(k).sprite_type() == 0xa7
            && k == 2
        {
            eprintln!(
                "R give-damage entry fc={} k={} dmg=0x{:02x} hit=0x{:02x} type=0x{:02x} dmgtype=0x{:02x} x=0x{:04x} y=0x{:04x} f=0x{:02x} health=0x{:02x} give=0x{:02x} item=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                dmg,
                r0_hit_timer,
                self.sprite_slot_view(k).sprite_type(),
                self.game_state.sprite_battle.damage_type_determiner(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.sprite_slot_view(k).f(),
                self.sprite_slot_view(k).health(),
                self.sprite_slot_view(k).incoming_damage(),
                self.game_state.player.follower_link.item_in_hand(),
            );
        }
        if dmg == 249 {
            self.sprite_func18(k, 0xe3);
            return;
        }
        if dmg == 250 {
            self.sprite_func18(k, 0x8f);
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 8;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_health(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_bump_damage(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_flags5(value);
            return;
        }
        if dmg >= self.sprite_slot_view(k).incoming_damage() {
            let value = dmg;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
        }
        if dmg == 0 {
            if self.game_state.sprite_battle.damage_type_determiner() != 10 {
                if self.sprite_slot_view(k).flags() & 4 != 0 {
                    self.sprite_set_damage_stun(k);
                    return;
                }
                self.follower_link_state_mut().clear_sword_delay_timer();
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if dmg >= 254 && self.sprite_slot_view(k).state() == 11 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x9a
            && self.sprite_slot_view(k).incoming_damage() < 0xf0
        {
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
            return;
        }
        if self.sprite_slot_view(k).sprite_type() == 0x1b {
            self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            self.sprite_schedule_for_breakage(k);
            self.sprite_place_weapon_tink(k);
            return;
        }
        let value = r0_hit_timer;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        if self.sprite_slot_view(k).sprite_type() != 0x92 || self.sprite_slot_view(k).c() >= 3 {
            let sfx = if self.sprite_slot_view(k).flags() & 2 != 0 {
                0x21
            } else if self.sprite_slot_view(k).flags5() & 0x10 != 0 {
                0x1c
            } else {
                8
            };
            self.set_sound_effect_2_with_sprite_pan(k, sfx);
        }
        self.sprite_set_damage_stun(k);
        if std::env::var_os("ZELDA3_TRACE_GIVE_DAMAGE").is_some()
            && self.game_state.world.location.dungeon_room() == 0x00a8
            && self.sprite_slot_view(k).sprite_type() == 0xa7
            && k == 2
        {
            eprintln!(
                "R give-damage set-f fc={} k={} f=0x{:02x} dmg=0x{:02x} hit=0x{:02x} dmgtype=0x{:02x} xr=0x{:02x} yr=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).f(),
                dmg,
                r0_hit_timer,
                self.game_state.sprite_battle.damage_type_determiner(),
                self.sprite_slot_view(k).x_recoil(),
                self.sprite_slot_view(k).y_recoil(),
            );
        }
    }

    fn sprite_set_damage_stun(&mut self, k: usize) {
        let ty = self.sprite_slot_view(k).sprite_type();
        let value = if self.game_state.sprite_battle.damage_type_determiner() >= 13 {
            0
        } else if ty == 9 {
            20
        } else if ty == 0x53 || ty == 0x18 {
            11
        } else {
            15
        };
        self.sprite_slot_view_mut(k).set_f(value);
    }

    // void Sprite_ScheduleForBreakage(int k) {  // 86e25a
    //   sprite_delay_main[k] = 31;
    //   sprite_state[k] = 6;
    //   sprite_flags2[k] += 4;
    // }
    pub(super) fn sprite_schedule_for_breakage(&mut self, k: usize) {
        let value = 31;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = self.sprite_slot_view(k).flags2().wrapping_add(4);
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_ZeroVelocity_XY(int k) {  // 86cf5d
    //   sprite_x_vel[k] = sprite_y_vel[k] = 0;
    // }
    pub(super) fn sprite_zero_velocity_xy(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = self.sprite_slot_view(k).y_velocity();
        self.sprite_slot_view_mut(k).set_x_velocity(value);
    }

    // void Sprite_Invert_XY_Speeds(int k) {
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_invert_xy_speeds(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.negate_x_velocity();
        sprite.negate_y_velocity();
    }

    // void Sprite_BounceOffWall(int k) {  // 86d9c0
    //   if (sprite_wallcoll[k] & 3)
    //     sprite_x_vel[k] = -sprite_x_vel[k];
    //   if (sprite_wallcoll[k] & 12)
    //     sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_bounce_off_wall(&mut self, k: usize) {
        if (self.sprite_slot_view(k).wall_collision() & 3) != 0 {
            self.sprite_slot_view_mut(k).negate_x_velocity();
        }
        if (self.sprite_slot_view(k).wall_collision() & 12) != 0 {
            self.sprite_slot_view_mut(k).negate_y_velocity();
        }
    }

    // void Sprite_InvertSpeed_XY(int k) {  // 86d9d5
    //   sprite_x_vel[k] = -sprite_x_vel[k];
    //   sprite_y_vel[k] = -sprite_y_vel[k];
    // }
    pub(super) fn sprite_invert_speed_xy(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.negate_x_velocity();
        sprite.negate_y_velocity();
    }

    // void Sprite_MoveZ(int k) {
    //   uint16 z = (sprite_z[k] << 8 | sprite_z_subpos[k]) + ((int8)sprite_z_vel[k] << 4);
    //   sprite_z_subpos[k] = z;
    //   sprite_z[k] = z >> 8;
    // }
    pub(super) fn sprite_move_z(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).move_z();
    }

    // void Sprite_ApplySpeedTowardsLink(int k, uint8 vel) {
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   sprite_x_vel[k] = pt.x;
    //   sprite_y_vel[k] = pt.y;
    // }
    pub(super) fn sprite_apply_speed_towards_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_x_velocity(pt.x);
        sprite.set_y_velocity(pt.y);
    }

    // void Sprite_SetSpawnedCoordinates(int k, SpriteSpawnInfo *info) {
    //   sprite_x_lo[k] = info->r0_x;
    //   sprite_x_hi[k] = info->r0_x >> 8;
    //   sprite_y_lo[k] = info->r2_y;
    //   sprite_y_hi[k] = info->r2_y >> 8;
    //   sprite_z[k] = info->r4_z;
    // }
    pub(super) fn sprite_set_spawned_coordinates(&mut self, k: usize, info: &SpriteSpawnInfo) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_x(info.r0_x);
        sprite.set_y(info.r2_y);
        sprite.set_z(info.r4_z);
    }

    pub(super) fn sprite_explode_spawn_ea(&mut self, k: usize) {
        let sprite_type = self.sprite_slot_view(k).sprite_type();
        self.temp_counter_mut().set(sprite_type);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0xea, &mut info, 14);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 32;
        self.sprite_slot_view_mut(j).set_z_velocity(value);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(j).set_floor(value);
        let value = if j == 9 { 2 } else { 6 };
        self.sprite_slot_view_mut(j).set_a(value);
        self.sprite_set_y(j, info.r2_y.wrapping_add(3));
        if self.game_state.scratch_counter.value() == 0xce {
            self.sprite_set_y(j, info.r2_y.wrapping_add(16));
            return;
        }
        if self.game_state.scratch_counter.value() == 0xcb {
            let link_x_hi = (self.game_state.player.follower_link.x() >> 8) as u8;
            let link_y_hi = (self.game_state.player.follower_link.y() >> 8) as u8;
            let value = 0x78;
            self.sprite_slot_view_mut(j).set_y_low(value);
            let value = 0x78;
            self.sprite_slot_view_mut(j).set_x_low(value);
            let value = link_x_hi;
            self.sprite_slot_view_mut(j).set_x_high(value);
            let value = link_y_hi;
            self.sprite_slot_view_mut(j).set_y_high(value);
        }
    }

    // void SpriteModule_Die(int k) {  // 86f8a2
    //   SpriteDeath_MainEx(k, false);
    // }
    pub(super) fn sprite_module_die(&mut self, k: usize) {
        self.sprite_death_main_ex(k, false);
    }

    // void SpriteDeath_MainEx(int k, bool second_entry) {  // 86823a
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_death_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            let type_ = self.sprite_slot_view(k).sprite_type();
            if type_ == 0xec {
                self.throwable_scenery_scatter_into_debris(k);
                return;
            }
            if type_ == 0x53
                || type_ == 0x54
                || type_ == 0x92
                || (type_ == 0x4a && self.sprite_slot_view(k).c() >= 2)
            {
                self.sprite_active_main_for_death(k);
                return;
            }
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_do_the_death(k);
                return;
            }
        }
        if sign8(self.sprite_slot_view(k).flags3()) {
            self.sprite_active_main_for_death(k);
            return;
        }
        if ((self.game_state.frame.frame_counter & 3)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag)
            == 0
        {
            let value = self.sprite_slot_view(k).delay_main().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        self.sprite_death_draw_poof(k);

        if self.sprite_slot_view(k).sprite_type() != 0x40
            && self.sprite_slot_view(k).delay_main() < 10
        {
            return;
        }
        let oam = self.game_state.oam.current_pointer().wrapping_add(16);
        let ext = self
            .game_state
            .oam
            .current_extended_pointer()
            .wrapping_add(4);
        self.oam_state_mut().set_current_pointer(oam);
        self.oam_state_mut().set_current_extended_pointer(ext);
        let bak = self.sprite_slot_view(k).flags2();
        let value = self.sprite_slot_view(k).flags2().wrapping_sub(4);
        self.sprite_slot_view_mut(k).set_flags2(value);
        self.sprite_active_main_for_death(k);
        let value = bak;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    fn sprite_active_main_for_death(&mut self, k: usize) {
        self.sprite_active_main(k);
    }

    // void Sprite_DoTheDeath(int k) {  // 86f923
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_do_the_death(&mut self, k: usize) {
        let type_ = self.sprite_slot_view(k).sprite_type();
        if type_ == 0xbe {
            let g = self.sprite_slot_view(0).g().wrapping_sub(1);
            self.sprite_slot_view_mut(0).set_g(g);
        }

        if type_ == 0xaa && self.sprite_slot_view(k).e() != 0 {
            let bak = self.sprite_slot_view(k).subtype();
            let item = SPRITE_DO_THE_DEATH_PIKIT_DROP_ITEMS
                [usize::from(self.sprite_slot_view(k).e().wrapping_sub(1))];
            self.prepare_enemy_drop(k, item);
            let value = bak;
            self.sprite_slot_view_mut(k).set_subtype(value);
            if bak == 1 {
                let value = 9;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                let value = 0xf0;
                self.sprite_slot_view_mut(k).set_flags3(value);
            }
            let value = self.sprite_slot_view(k).head_direction().wrapping_add(1);
            self.sprite_slot_view_mut(k).set_head_direction(value);
            return;
        }

        if type_ == 0x45
            && self.game_state.inventory.save_progress.progress_indicator() == 2
            && self.game_state.world.region.overworld_area_low() == 0x18
        {
            self.set_music_control(7);
        }

        let drop_item = self.sprite_slot_view(k).die_action();
        if drop_item != 0 {
            let value = self.sprite_slot_view(k).n();
            self.sprite_slot_view_mut(k).set_subtype(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_n(value);
            let arg = if drop_item == 1 {
                0xe4
            } else if drop_item == 3 {
                0xd9
            } else {
                0xe5
            };
            self.prepare_enemy_drop(k, arg);
            return;
        }

        let mut prize = self.sprite_slot_view(k).flags5() & 0x0f;
        if prize != 0 {
            prize = prize.wrapping_sub(1);
            let luck = self.game_state.sprite_battle.item_drop_luck();
            if luck != 0 {
                self.sprite_battle_mut().increment_luck_kill_counter();
                if self.game_state.sprite_battle.luck_kill_counter() >= 10 {
                    self.sprite_battle_mut().set_item_drop_luck(0);
                }
                if luck == 1 {
                    self.force_prize_drop(k, prize, 1);
                    return;
                }
            } else {
                if (self.get_random_number() & SPRITE_DO_THE_DEATH_PRIZE_MASKS[usize::from(prize)])
                    == 0
                {
                    self.force_prize_drop(k, prize, prize);
                    return;
                }
            }
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        self.sprite_death_func4(k);
    }

    // void ForcePrizeDrop(int k, uint8 prize, uint8 slot) {  // 86f9bc
    //   prize = prize * 8 | prizes_arr1[slot];
    //   prizes_arr1[slot] = (prizes_arr1[slot] + 1) & 7;
    //   PrepareEnemyDrop(k, kPrizeItems[prize]);
    // }
    pub(super) fn force_prize_drop(&mut self, k: usize, prize: u8, slot: u8) {
        let slot = usize::from(slot);
        let cycle_index = self.prize_drop_cycle_mut().take_next_index(slot);
        let prize = usize::from(prize) * 8 | usize::from(cycle_index);
        self.prepare_enemy_drop(k, FORCE_PRIZE_DROP_PRIZE_ITEMS[prize]);
    }

    // void PrepareEnemyDrop(int k, uint8 item) {  // 86f9d1
    //   ...see sprite.c...
    // }
    pub(super) fn prepare_enemy_drop(&mut self, k: usize, item: u8) {
        let value = item;
        self.sprite_slot_view_mut(k).set_sprite_type(value);
        if item == 0xe5 {
            if self.begin_big_key_drop_graphics_work(k) {
                return;
            }
            self.sprite_prep_big_key_load_graphics(k);
        } else if item == 0xe4 {
            self.sprite_prep_key_set_item_drop(k);
        }

        self.complete_prepare_enemy_drop(k);
    }

    pub(super) fn complete_prepare_enemy_drop(&mut self, k: usize) {
        let value = 9;
        self.sprite_slot_view_mut(k).set_state(value);
        let zbak = self.sprite_slot_view(k).z();
        self.sprite_prep_load_properties_for_helpers(k);
        let value = self.sprite_slot_view(k).ignore_projectile().wrapping_add(1);
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);

        let pz = PREPARE_ENEMY_DROP_PRIZE_Z
            [usize::from(self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8))];
        let value = pz & 0xf0;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(u16::from(pz & 0x0f)));
        let value = zbak;
        self.sprite_slot_view_mut(k).set_z(value);
        let value = 21;
        self.sprite_slot_view_mut(k).set_delay_aux4(value);
        let value = 255;
        self.sprite_slot_view_mut(k).set_stunned(value);
        self.sprite_death_func4(k);
    }

    // void SpriteDeath_Func4(int k) {  // 86fa25
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_death_func4(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() == 0xa2 && self.sprite_check_if_screen_is_clear()
        {
            self.ancilla_spawn_falling_prize(4);
        }
        self.sprite_manually_set_death_flag_uw(k);
        self.sprite_battle_mut().increment_sprites_killed();
        if self.sprite_slot_view(k).sprite_type() == 0x40 {
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_death_main_ex(k, true);
        }
    }

    pub(super) fn sprite_death_draw_poof(&mut self, k: usize) {
        if self.game_state.dungeon.room_load.header_collision() == 4 {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        let Some((_x, _y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r12 = (self.sprite_slot_view(k).flags3() & 0x20) >> 3;
        let scratch_position = &self.game_state.sprites.draw_hitbox_work;
        let dungmap_x = scratch_position.x_low();
        let dungmap_y = scratch_position.y_low();
        let mut i = usize::from((self.sprite_slot_view(k).delay_main() & 0x1c) ^ 0x1c) + 3;
        for _ in 0..4 {
            if SPRITE_DEATH_DRAW_POOF_CHARS[i] != 0 {
                self.oam_state_mut()
                    .set_entry_char(oam, SPRITE_DEATH_DRAW_POOF_CHARS[i]);
                self.oam_state_mut().set_entry_y(
                    oam,
                    dungmap_y
                        .wrapping_sub(r12)
                        .wrapping_add(SPRITE_DEATH_DRAW_POOF_Y_OFFSETS[i] as u8),
                );
                self.oam_state_mut().set_entry_x(
                    oam,
                    dungmap_x
                        .wrapping_sub(r12)
                        .wrapping_add(SPRITE_DEATH_DRAW_POOF_X_OFFSETS[i] as u8),
                );
                self.oam_state_mut()
                    .set_entry_flags(oam, (flags & 0x30) | SPRITE_DEATH_DRAW_POOF_FLAGS[i]);
            }
            oam += 4;
            i = i.wrapping_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0);
    }

    // void SpriteModule_Fall1(int k) {  // 86852e
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_fall1(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_manually_set_death_flag_uw(k);
            return;
        }
        let (mut info, out) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if !out {
            self.sprite_fall_draw(k, &mut info);
        }
    }

    // void SpriteModule_Burn(int k) {  // sprite.c:747
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_burn(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_hit_timer(value);
        let j = i16::from(self.sprite_slot_view(k).delay_main()) - 1;
        if j == 0 {
            self.sprite_do_the_death(k);
            return;
        }
        let bak_graphics = self.sprite_slot_view(k).graphics();
        let bak_oam = self.sprite_slot_view(k).oam_flags();
        let value = SPRITE_MODULE_BURN_FLAME_GFX[(j >> 3) as usize];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = 3;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.flame_draw(k);
        let value = bak_oam;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = bak_graphics;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let next_oam = self.game_state.oam.current_pointer().wrapping_add(8);
        let next_ext = self
            .game_state
            .oam
            .current_extended_pointer()
            .wrapping_add(2);
        self.oam_state_mut().set_current_pointer(next_oam);
        self.oam_state_mut().set_current_extended_pointer(next_ext);
        if self.sprite_slot_view(k).delay_main() >= 0x10 {
            let bak = self.sprite_slot_view(k).flags2();
            let value = self.sprite_slot_view(k).flags2().wrapping_sub(2);
            self.sprite_slot_view_mut(k).set_flags2(value);
            self.sprite_active_main_for_death(k);
            let value = bak;
            self.sprite_slot_view_mut(k).set_flags2(value);
        }
    }

    // void SpriteModule_Poof(int k) {  // 86e393
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_poof(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            if self.sprite_slot_view(k).sprite_type() == 0x0d
                && self.sprite_slot_view(k).head_direction() != 0
            {
                let bakx = self.sprite_get_x(k);
                self.prepare_enemy_drop(k, 0x0d);
                self.sprite_set_x(k, bakx);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            } else if self.sprite_slot_view(k).die_action() == 0 {
                self.force_prize_drop(k, 2, 2);
            } else {
                self.sprite_do_the_death(k);
            }
            return;
        }

        let Some((_x, _y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut j = usize::from(((self.sprite_slot_view(k).delay_main() >> 1) & !3) + 3).min(15);
        let scratch_position = &self.game_state.sprites.draw_hitbox_work;
        let base_x = scratch_position.x_low();
        let base_y = scratch_position.y_low();
        for _ in 0..4 {
            self.set_oam_plain_at_for_sprite(
                oam,
                base_x.wrapping_add(SPRITE_MODULE_POOF_X_OFFSETS[j] as u8),
                base_y.wrapping_add(SPRITE_MODULE_POOF_Y_OFFSETS[j] as u8),
                SPRITE_MODULE_POOF_CHARS[j],
                SPRITE_MODULE_POOF_FLAGS[j],
                SPRITE_MODULE_POOF_EXT_SIZES[j],
            );
            oam += 4;
            j = j.saturating_sub(1);
        }
        self.sprite_correct_oam_entries(k, 3, 0xff);
    }

    // void SpriteModule_Drown(int k) {  // 86859c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_drown(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).a() == 6 {
                self.oam_allocate_from_region_c(8);
            }
            self.sprite_slot_view_mut(k).xor_flags3(16);
            self.sprite_draw_single_large(k);
            let oam = self.game_state.oam.current_pointer_usize();
            let j = self.sprite_slot_view(k).delay_main();
            if j == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if j != 0 {
                self.oam_state_mut().set_entry_char(
                    oam,
                    SPRITE_MODULE_DROWN_OAM_CHARS[usize::from((j >> 1).min(10))],
                );
                self.oam_state_mut().set_entry_flags(oam, 0x24);
                return;
            }
            self.oam_state_mut().set_entry_char(oam, 0x8a);
            let flags = SPRITE_MODULE_DROWN_OAM_FLAGS
                [usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3)]
                | 0x24;
            self.oam_state_mut().set_entry_flags(oam, flags);
            if self.sprite_return_if_paused(k) {
                return;
            }
            self.sprite_slot_view_mut(k).increment_subtype2();
            self.sprite_move_xy(k);
            self.sprite_move_z(k);
            let value = self.sprite_slot_view(k).z_velocity().wrapping_sub(2);
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if sign8(self.sprite_slot_view(k).z()) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_z(value);
                let value = 18;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_slot_view_mut(k).and_flags3(!0x10);
            }
        } else {
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.game_state.frame.frame_counter & 1 == 0 {
                let value = self.sprite_slot_view(k).delay_main().wrapping_add(1);
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            let base =
                usize::from(((self.sprite_slot_view(k).delay_main() << 1) & 0xf8) >> 2).min(6);
            self.sprite_draw_multiple(
                k,
                &SPRITE_MODULE_DROWN_DROWN_DRAW_FRAMES[base..base + 2],
                None,
            );
        }
    }

    // void SpriteModule_Explode(int k) {  // sprite.c:616
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_explode(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                if !(0..16).any(|j| self.sprite_slot_view(j).state() == 4) {
                    self.set_chr_halfslot_request(1);
                    if !self.sprite_check_if_screen_is_clear() {
                        self.follower_link_state_mut().clear_menu_block();
                    }
                }
            } else {
                let base = usize::from((self.sprite_slot_view(k).delay_main() >> 2) ^ 7) * 4;
                self.sprite_draw_multiple(
                    k,
                    &SPRITE_MODULE_EXPLODE_SPRITE_EXPLODE_DRAW_FRAMES[base..base + 4],
                    None,
                );
            }
            return;
        }
        let value = 2;
        self.sprite_slot_view_mut(k).set_floor(value);
        if self.sprite_slot_view(k).delay_main() == 32 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_link_state_mut().clear_immobilized();
            if !self.game_state.player.follower_link.near_pit_state_is(2)
                && self.sprite_check_if_screen_is_clear()
            {
                if self.sprite_slot_view(k).sprite_type() >= 0xd6 {
                    self.set_music_control(0x13);
                } else if self.sprite_slot_view(k).sprite_type() == 0x7a {
                    self.prepare_dungeon_exit_from_boss_fight();
                } else {
                    self.sprite_explode_spawn_ea(k);
                    return;
                }
            }
        }
        if self.sprite_slot_view(k).delay_main() >= 64
            && (self.sprite_slot_view(k).delay_main() >= 0x70
                || (self.sprite_slot_view(k).delay_main() & 1) == 0)
        {
            self.sprite_active_main_for_death(k);
        }

        let type_ = self.sprite_slot_view(k).sprite_type();
        let delay = self.sprite_slot_view(k).delay_main();
        if delay >= 0xc0 {
            return;
        }
        if delay & 3 == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        }
        if delay & if type_ == 0x92 { 3 } else { 7 } != 0 {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x1c, &mut info);
        if j >= 0 {
            const SPRITE_EXPLODE_RANDOM_XY: [i8; 16] =
                [0, 4, 8, 12, -4, -8, -12, 0, 0, 8, 16, 24, -24, -16, -8, 0];
            let j = j as usize;
            self.set_chr_halfslot_request(11);
            let value = 4;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 0x0c;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let random_base = if type_ == 0x92 { 8 } else { 0 };
            let xoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            let yoff =
                SPRITE_EXPLODE_RANDOM_XY[usize::from(self.get_random_number() & 7) | random_base];
            self.sprite_set_x(j, info.r0_x.wrapping_add(xoff as i16 as u16));
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add(yoff as i16 as u16)
                    .wrapping_sub(u16::from(info.r4_z)),
            );
            let value = 31;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 31;
            self.sprite_slot_view_mut(j).set_a(value);
        }
    }

    // void SpriteModule_Fall2(int k) {  // 86fbea
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_fall2(&mut self, k: usize) {
        let mut delay = self.sprite_slot_view(k).delay_main();
        if delay == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_manually_set_death_flag_uw(k);
            return;
        }
        if delay >= 0x40 {
            if self.sprite_slot_view(k).oam_flags() != 5 {
                if ((delay & 7)
                    | self.game_state.frame.submodule
                    | self.game_state.frame.modal_pause_flag)
                    == 0
                {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x31);
                }
                self.sprite_active_main_for_death(k);
                let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
                    return;
                };
                self.sprite_draw_distress_custom(x, y.wrapping_sub(8), delay.wrapping_add(20));
                return;
            }
            let value = 63;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            delay = 63;
        }
        if delay == 61 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }
        let j = usize::from(delay >> 1);
        if self.sprite_slot_view(k).sprite_type() == 0x26
            || self.sprite_slot_view(k).sprite_type() == 0x13
        {
            let value = SPRITE_MODULE_FALL2_FALLING_HELMA_BEETLE_GFX_BY_DELAY[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_draw_falling_helma_beetle(k);
        } else {
            let mut t = SPRITE_MODULE_FALL2_FALLING_HUMANOID_GFX_BY_DELAY[j];
            if t < 12 {
                t = t.wrapping_add(
                    SPRITE_MODULE_FALL2_FALLING_DIRECTION_GFX_OFFSETS
                        [usize::from(self.sprite_slot_view(k).direction() & 3)],
                );
            }
            let value = t;
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_draw_falling_humanoid(k);
        }
        if (self.game_state.frame.frame_counter
            & SPRITE_MODULE_FALL2_FALLING_TILE_CHECK_FRAME_MASKS
                [usize::from(self.sprite_slot_view(k).delay_main() >> 3)])
            | self.game_state.frame.submodule
            != 0
        {
            return;
        }
        self.sprite_check_tile_property(k, 0x68);
        if self.game_state.sprites.workspace.tile_type() != 0x20 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_y_recoil(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_x_recoil(value);
        }
        let value = ((self.sprite_slot_view(k).y_recoil() as i8) >> 2) as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = ((self.sprite_slot_view(k).x_recoil() as i8) >> 2) as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy(k);
    }

    // bool Sprite_CheckDamageToAndFromLink(int k) {  // 85ab93
    //   Sprite_CheckDamageFromLink(k);
    //   return Sprite_CheckDamageToLink(k);
    // }
    pub(super) fn sprite_check_damage_to_and_from_link(&mut self, k: usize) -> bool {
        self.sprite_check_damage_from_link(k);
        self.sprite_check_damage_to_link(k)
    }

    // void SpriteModule_Carried(int k) {  // 86de83
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_module_carried(&mut self, k: usize) {
        let value = self.game_state.world.region.overworld_area_low();
        self.sprite_slot_view_mut(k).set_room(value);
        if self.sprite_slot_view(k).draw_work_byte_3() != 3 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = if self.sprite_slot_view(k).c() == 6 {
                    8
                } else {
                    4
                };
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_slot_view_mut(k).increment_draw_work_byte_3();
            }
        } else {
            self.sprite_slot_view_mut(k).and_flags3(!0x10);
        }

        let t = self.sprite_slot_view(k).delay_aux4().wrapping_sub(1);
        let r0 = u16::from(t < 63 && (t & 2) != 0);
        let j = usize::from(
            self.game_state
                .player
                .follower_link
                .facing()
                .wrapping_mul(2)
                .wrapping_add(self.sprite_slot_view(k).draw_work_byte_3())
                & 0x0f,
        );
        let link_x = self.game_state.player.follower_link.x();
        let offset = SPRITE_MODULE_CARRIED_SPRITE_HELD_X[j] as i16 as u16;
        let t0 = u16::from(link_x as u8) + u16::from(offset as u8);
        let t1 = u16::from(t0 as u8) + ((t0 >> 8) & 1) + r0;
        let t2 = u16::from((link_x >> 8) as u8)
            + ((t1 >> 8) & 1)
            + ((t0 >> 8) & 1)
            + u16::from((offset >> 8) as u8);
        let value = t1 as u8;
        self.sprite_slot_view_mut(k).set_x_low(value);
        let value = t2 as u8;
        self.sprite_slot_view_mut(k).set_x_high(value);
        let value = SPRITE_MODULE_CARRIED_SPRITE_HELD_Z[j];
        self.sprite_slot_view_mut(k).set_z(value);
        let an = if self.game_state.player.follower_link.animation_step() < 6 {
            self.game_state.player.follower_link.animation_step_index()
        } else {
            0
        };
        let z = self
            .game_state
            .player
            .follower_link
            .z()
            .wrapping_add(1)
            .wrapping_add(u16::from(SPRITE_MODULE_CARRIED_SPRITE_HELD_Z_FOR_FRAME[an]));
        self.sprite_set_y(
            k,
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(8)
                .wrapping_sub(z),
        );
        let value = self.game_state.player.follower_link.lower_level_state() & 1;
        self.sprite_slot_view_mut(k).set_floor(value);

        self.carried_sprite_check_for_throw(k);
        self.sprite_get16_bit_coords(k);
        if self.sprite_slot_view(k).draw_work_byte_4() != 11 {
            self.sprite_active_main_for_death(k);
            if self.sprite_slot_view(k).delay_aux4() == 1 {
                let value = 9;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_b(value);
                let value = 96;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_slot_view_mut(k).or_flags3(0x10);
                self.follower_link_state_mut().set_picking_throw_state(2);
            }
        } else {
            self.sprite_stunned_main_func1(k);
        }
    }

    // void CarriedSprite_CheckForThrow(int k) {  // 86df6d
    //   ...see sprite.c...
    // }
    pub(super) fn carried_sprite_check_for_throw(&mut self, k: usize) {
        if self.game_state.frame.main_module == 14 {
            return;
        }
        if !self.game_state.player.follower_link.near_pit_state_is(2) {
            let t = (self.game_state.player.follower_link.auxiliary_state() & 1)
                | self.game_state.player.follower_link.deep_water_state()
                | u8::from(self.game_state.player.follower_link.is_bunny_mirror())
                | self.game_state.player.follower_link.item_hold_pose()
                | if self
                    .game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer()
                    != 0
                {
                    0
                } else {
                    self.game_state.player.follower_link.incapacitated_timer()
                };
            if t == 0 {
                if self.sprite_slot_view(k).draw_work_byte_3() != 3
                    || ((self.game_state.player.follower_link.filtered_joypad_h()
                        | self.game_state.player.follower_link.filtered_joypad_l())
                        & 0x80)
                        == 0
                {
                    return;
                }
                self.follower_link_state_mut()
                    .clear_filtered_joypad_l_bits(0x80);
            }
        }

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
        self.follower_link_state_mut().set_picking_throw_state(2);
        let value = self.sprite_slot_view(k).draw_work_byte_4();
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_draw_work_byte_3(value);
        let value = (self.sprite_slot_view(k).flags3() & !0x10)
            | (sprite_init_value(
                SPRITE_INIT_FLAGS3_TABLE,
                self.sprite_slot_view(k).sprite_type(),
            ) & 0x10);
        self.sprite_slot_view_mut(k).set_flags3(value);
        let j = self.game_state.player.follower_link.facing_index() & 3;
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_XVEL[j];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_YVEL[j];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = CARRIED_SPRITE_CHECK_FOR_THROW_SPRITE_HELD_THROW_ZVEL[j];
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_delay_aux4(value);
    }

    // void SpriteModule_Stunned(int k) {  // 86dffa
    //   SpriteStunned_MainEx(k, false);
    // }
    pub(super) fn sprite_module_stunned(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, false);
    }

    // void ThrownSprite_TileAndSpriteInteraction(int k) {  // 86e02a
    //   SpriteStunned_MainEx(k, true);
    // }
    pub(super) fn thrown_sprite_tile_and_sprite_interaction(&mut self, k: usize) {
        self.sprite_stunned_main_ex(k, true);
    }

    // void ThrowableScenery_InteractWithSpritesAndTiles(int k) {  // 86e164
    //   Sprite_MoveXY(k);
    //   if (!sprite_E[k])
    //     Sprite_CheckTileCollision(k);
    //   ThrownSprite_TileAndSpriteInteraction(k);
    // }
    pub(super) fn throwable_scenery_interact_with_sprites_and_tiles(&mut self, k: usize) {
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).e() == 0 {
            self.sprite_check_tile_collision(k);
        }
        self.thrown_sprite_tile_and_sprite_interaction(k);
    }

    // void SpriteStunned_MainEx(int k, bool second_entry) {  // 86dfcf
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_stunned_main_ex(&mut self, k: usize, second_entry: bool) {
        if !second_entry {
            self.sprite_draw_ripple_if_in_water(k);
            self.sprite_stunned_main_func1(k);
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.sprite_slot_view(k).f() != 0 {
                if sign8(self.sprite_slot_view(k).f()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_f(value);
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            if self.sprite_slot_view(k).delay_main() < 0x20 {
                self.sprite_check_damage_from_link(k);
            }
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_move_xy(k);
            if self.sprite_slot_view(k).e() == 0 {
                self.sprite_check_tile_collision(k);
                if self.sprite_slot_view(k).state() == 0 {
                    return;
                }
            }
        }

        if (second_entry || self.sprite_slot_view(k).e() == 0)
            && (self.sprite_slot_view(k).wall_collision() & 0x0f) != 0
        {
            self.sprite_apply_ricochet(k);
            if self.sprite_slot_view(k).state() == 11 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 5);
            }
        }
        self.sprite_check_tile_property(k, 0x68);

        if sprite_init_value(
            SPRITE_INIT_FLAGS3_TABLE,
            self.sprite_slot_view(k).sprite_type(),
        ) & 0x10
            != 0
        {
            self.sprite_slot_view_mut(k).or_flags3(0x10);
            if self.game_state.sprites.workspace.tile_type() == 32 {
                self.sprite_slot_view_mut(k).and_flags3(!0x10);
            }
        }
        self.sprite_move_z(k);
        let value = self.sprite_slot_view(k).z_velocity().wrapping_sub(2);
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        if self.sprite_slot_view(k).z().wrapping_sub(1) >= 0xf0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            if self.sprite_slot_view(k).sprite_type() == 0xe8
                && (self.sprite_slot_view(k).z_velocity().wrapping_sub(0xe8) as i8).is_negative()
            {
                let value = 6;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 8;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 3;
                self.sprite_slot_view_mut(k).set_flags2(value);
                return;
            }

            self.throwable_scenery_transmute_if_valid(k);
            let mut tile = self.game_state.sprites.workspace.tile_type();
            if self.game_state.sprites.workspace.tile_type() == 32 {
                tile = self.sprite_slot_view(k).flags() >> 1;
                if self.sprite_slot_view(k).flags() & 1 == 0 {
                    self.sprite_func8(k);
                    return;
                }
            }
            if tile == 9 {
                let z_vel = self.sprite_slot_view(k).z_velocity();
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if (z_vel.wrapping_sub(0xf0) as i8).is_negative() {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_set_spawned_coordinates(j, &info);
                        self.sprite_func22(j);
                    }
                }
            } else if tile == 8 {
                if self.sprite_slot_view(k).sprite_type() == 0xd2
                    || (self.get_random_number() & 1) != 0
                {
                    self.sprite_spawn_leaping_fish(k);
                }
                self.sprite_func22(k);
                return;
            }

            let z_vel = self.sprite_slot_view(k).z_velocity();
            if (z_vel as i8).is_negative() {
                let bounced = z_vel.wrapping_neg() >> 1;
                let value = if bounced < 9 { 0 } else { bounced };
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            }
            let value = ((self.sprite_slot_view(k).x_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            if self.sprite_slot_view(k).x_velocity() == 0xff {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let value = ((self.sprite_slot_view(k).y_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            if self.sprite_slot_view(k).y_velocity() == 0xff {
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        if self.sprite_slot_view(k).state() != 11
            || self.sprite_slot_view(k).draw_work_byte_5() != 0
        {
            if self.sprite_return_if_lifted(k) {
                return;
            }
            if self.sprite_slot_view(k).sprite_type() != 0x4a {
                self.thrown_sprite_check_damage_to_sprites(k);
            }
        }
    }

    // void SpriteStunned_Main_Func1(int k) {  // 86e2ba
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_stunned_main_func1(&mut self, k: usize) {
        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R stunned-before fc=0x{:02x} rng=0x{:02x} k={} type=0x{:02x} state=0x{:02x} draw_work5=0x{:02x} delay=0x{:02x} stunned=0x{:02x} give=0x{:02x} z=0x{:02x} zv=0x{:02x} ai=0x{:02x}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.rng_seed(),
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.sprite_slot_view(k).state(),
                self.sprite_slot_view(k).draw_work_byte_5(),
                self.sprite_slot_view(k).delay_main(),
                self.sprite_slot_view(k).stunned(),
                self.sprite_slot_view(k).incoming_damage(),
                self.sprite_slot_view(k).z(),
                self.sprite_slot_view(k).z_velocity(),
                self.sprite_slot_view(k).ai_state(),
            );
        }
        self.sprite_active_main_for_death(k);
        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R stunned-after-active fc=0x{:02x} rng=0x{:02x} k={} type=0x{:02x} state=0x{:02x} draw_work5=0x{:02x} delay=0x{:02x} stunned=0x{:02x} give=0x{:02x} z=0x{:02x} zv=0x{:02x} ai=0x{:02x}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.rng_seed(),
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.sprite_slot_view(k).state(),
                self.sprite_slot_view(k).draw_work_byte_5(),
                self.sprite_slot_view(k).delay_main(),
                self.sprite_slot_view(k).stunned(),
                self.sprite_slot_view(k).incoming_damage(),
                self.sprite_slot_view(k).z(),
                self.sprite_slot_view(k).z_velocity(),
                self.sprite_slot_view(k).ai_state(),
            );
        }
        if self.sprite_slot_view(k).draw_work_byte_5() != 0 {
            if self.sprite_slot_view(k).delay_main() < 32 {
                let value = (self.sprite_slot_view(k).oam_flags() & 0xf1) | 4;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            }
            let t = (((k as u8) << 4) ^ self.game_state.frame.frame_counter)
                | self.game_state.frame.submodule;
            let mask = SPRITE_STUNNED_MAIN_FUNC1_SPRITE_STUNNED_MAIN_FUNC1_MASKS
                [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
            if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
                eprintln!(
                    "R stunned-sparkle-check fc=0x{:02x} k={} t=0x{:02x} mask=0x{:02x} delay=0x{:02x}",
                    self.game_state.frame.frame_counter,
                    k,
                    t,
                    mask,
                    self.sprite_slot_view(k).delay_main(),
                );
            }
            if t & mask != 0 {
                return;
            }
            let x = SPRITE_STUNNED_MAIN_FUNC1_SPARKLE_GARNISH_XY
                [usize::from(self.get_random_number() & 3)] as i16 as u16;
            let y = SPRITE_STUNNED_MAIN_FUNC1_SPARKLE_GARNISH_XY
                [usize::from(self.get_random_number() & 3)] as i16 as u16;
            self.sprite_garnish_spawn_sparkle(k, x, y);
            return;
        }

        if (self.game_state.frame.frame_counter & 1)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag
            != 0
        {
            return;
        }
        let t = self.sprite_slot_view(k).stunned();
        if t != 0 {
            let value = self.sprite_slot_view(k).stunned().wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_stunned(value);
            if t < 0x38 {
                let value = if (t & 1) != 0 { (-8i8) as u8 } else { 8 };
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                self.sprite_move_x(k);
            }
            return;
        }
        let value = 9;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_x_recoil(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_recoil(value);
    }

    // void Sprite_SpawnLeapingFish(int k) {  // 86e286
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_leaping_fish(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xd2, &mut info);
        if j < 0 {
            return;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 2;
        self.sprite_slot_view_mut(j).set_ai_state(value);
        let value = 48;
        self.sprite_slot_view_mut(j).set_delay_main(value);
        if self.sprite_slot_view(k).sprite_type() == 0xd2 {
            let value = 0xd2;
            self.sprite_slot_view_mut(j).set_a(value);
        }
    }

    // bool Sprite_HandleDraggingByAncilla(int k) {  // 86cf64
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_handle_dragging_by_ancilla(&mut self, k: usize) -> bool {
        let mut j = self.sprite_slot_view(k).b();
        if j == 0 {
            return false;
        }
        j = j.wrapping_sub(1);
        let j = usize::from(j);
        if self.ancilla_slot_view(j).ancilla_type() == 0 {
            self.sprite_handle_absorption_by_player(k);
        } else {
            let ancilla = self.ancilla_slot_view(j);
            let x_low = ancilla.x_low();
            let x_high = ancilla.x_high();
            let y_low = ancilla.y_low();
            let y_high = ancilla.y_high();
            self.sprite_slot_view_mut(k).set_x_low(x_low);
            self.sprite_slot_view_mut(k).set_x_high(x_high);
            self.sprite_slot_view_mut(k).set_y_low(y_low);
            self.sprite_slot_view_mut(k).set_y_high(y_high);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
        }
        true
    }

    // void Sprite_CheckAbsorptionByPlayer(int k) {  // 86d116
    //   if (!sprite_delay_aux4[k] && Sprite_CheckDamageToPlayer_1(k))
    //     Sprite_HandleAbsorptionByPlayer(k);
    // }
    pub(super) fn sprite_check_absorption_by_player(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() == 0 && self.sprite_check_damage_to_player_1(k) {
            self.sprite_handle_absorption_by_player(k);
        }
    }

    // void Sprite_HandleAbsorptionByPlayer(int k) {  // 86d13c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_handle_absorption_by_player(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        let t = self.sprite_slot_view(k).sprite_type().wrapping_sub(0xd8);
        if usize::from(t) < SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORPTION_SFX.len() {
            self.sprite_sfx_queue_sfx3_with_pan(
                k,
                SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORPTION_SFX[usize::from(t)],
            );
        }
        match t {
            0 => self.player_resources_mut().increment_heart_filler_by(8),
            1..=3 => {
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_add(
                        SPRITE_HANDLE_ABSORPTION_BY_PLAYER_RUPEES_ABSORPTION[usize::from(t - 1)],
                    );
                self.player_resources_mut().set_rupees_goal(rupees);
            }
            4..=6 => {
                self.player_resources_mut().increment_bomb_filler_by(
                    SPRITE_HANDLE_ABSORPTION_BY_PLAYER_BOMBS_ABSORPTION[usize::from(t - 4)],
                );
            }
            7 => self.player_resources_mut().increment_magic_filler_by(0x10),
            8 => self.player_resources_mut().set_magic_filler(0x80),
            9 => {
                let arrows = if self.sprite_slot_view(k).head_direction() == 0 {
                    5
                } else {
                    self.sprite_slot_view(k).head_direction()
                };
                self.player_resources_mut()
                    .increment_arrow_filler_by(arrows);
            }
            10 => self.player_resources_mut().increment_arrow_filler_by(10),
            11 => {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x31);
                self.player_resources_mut().increment_heart_filler_by(56);
            }
            12 => {
                self.player_resources_mut().increment_keys();
                self.finish_absorbed_key_or_big_key(
                    k,
                    &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY,
                );
            }
            13 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x32,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::BigKeyAbsorption,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.finish_absorbed_key_or_big_key(
                    k,
                    &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY,
                );
            }
            14 => {
                let shield = self.sprite_slot_view(k).subtype();
                self.inventory_items_mut().set_shield_type(shield);
                if self.game_state.enhanced_features.has(4096) {
                    self.Palette_Load_Shield();
                }
            }
            _ => {}
        }
    }

    /// Source suffix after the absorption handler's synchronous
    /// `Link_ReceiveItem(0x32)` call (ROM `$86d13c` case 13). A live timing
    /// authority suspends the decompressor there; only the shared
    /// key/big-key savegame-bit tail remains.
    pub(super) fn complete_big_key_absorption_item_receipt(&mut self, k: usize) {
        self.finish_absorbed_key_or_big_key(k, &SPRITE_HANDLE_ABSORPTION_BY_PLAYER_ABSORB_BIG_KEY);
    }

    fn finish_absorbed_key_or_big_key(&mut self, k: usize, absorb_big_key: &[u16; 2]) {
        let value = self.sprite_slot_view(k).subtype();
        self.sprite_slot_view_mut(k).set_n(value);
        let idx = usize::from(self.sprite_slot_view(k).die_action());
        let bits =
            self.game_state.dungeon.savegame_state.savegame_state_bits() | absorb_big_key[idx];
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
        self.sprite_manually_set_death_flag_uw(k);
    }

    // uint8 Sprite_CheckDamageFromLink(int k) {  // 86f2b4
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_damage_from_link(&mut self, k: usize) -> u8 {
        if (self.sprite_slot_view(k).hit_timer() & 0x80) != 0
            || self.sprite_slot_view(k).floor()
                != self.game_state.player.follower_link.lower_level_state()
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
        {
            return 0;
        }

        let mut hb = empty_sprite_hit_box();
        self.player_setup_action_hit_box(&mut hb);
        self.sprite_setup_hit_box(k, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_DAMAGE_FROM_LINK").is_some()
            && self.game_state.world.location.dungeon_room() == 0x00a8
            && self.sprite_slot_view(k).sprite_type() == 0xa7
            && k == 2
        {
            eprintln!(
                "R damage-from-link fc={} k={} overlap={} type=0x{:02x} dmgtype=0x{:02x} link=0x{:04x},0x{:04x} spr=0x{:04x},0x{:04x} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x} item=0x{:02x} sword_delay=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                overlap,
                self.sprite_slot_view(k).sprite_type(),
                self.game_state.sprite_battle.damage_type_determiner(),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
                self.game_state.player.follower_link.item_in_hand(),
                self.game_state.player.follower_link.sword_delay_timer(),
            );
        }
        if !overlap {
            return 0;
        }

        self.sprite_battle_mut().clear_damaging_enemies_timer();
        if self.game_state.player.follower_link.position_mode_has(0x10) {
            return CHECK_DAMAGE_FROM_PLAYER_CARRY | CHECK_DAMAGE_FROM_PLAYER_NON_ELEMENTAL;
        }

        if self.game_state.player.follower_link.item_in_hand_has(10) {
            if self.sprite_slot_view(k).sprite_type() >= 0xd6 {
                return 0;
            }
            if self.sprite_slot_view(k).state() == 11
                && self.sprite_slot_view(k).draw_work_byte_5() != 0
            {
                let value = 2;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = (self.sprite_slot_view(k).flags2() & 0xe0) | 3;
                self.sprite_slot_view_mut(k).set_flags2(value);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
                return CHECK_DAMAGE_FROM_PLAYER_CARRY | CHECK_DAMAGE_FROM_PLAYER_NON_ELEMENTAL;
            }
        }

        let ty = self.sprite_slot_view(k).sprite_type();
        if ty == 0x7b {
            if !sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                return 0;
            }
        } else if ty == 9 {
            if self.sprite_slot_view(k).a() == 0 {
                self.sprite_apply_recoil_to_link(k, 48);
                self.sprite_battle_mut().set_damaging_enemies_timer(144);
                self.follower_link_state_mut().set_incapacitated_timer(16);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                let value = 48;
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
                let effect = if self.game_state.enhanced_features.has(4096) {
                    0x32
                } else {
                    0
                };
                self.set_sound_effect_2_with_sprite_pan(k, effect);
                self.link_place_weapon_tink();
                return CHECK_DAMAGE_FROM_PLAYER_CARRY;
            }
        } else if ty == 0x92 {
            if self.sprite_slot_view(k).c() >= 3 {
                self.sprite_apply_recoil_to_link(k, 32);
                self.sprite_battle_mut().set_damaging_enemies_timer(144);
                self.follower_link_state_mut().set_incapacitated_timer(16);
            } else {
                return self.sprite_check_damage_from_link_getting_out(k);
            }
        } else if ty == 0x26 || ty == 0x13 || ty == 2 {
            const SPRITE_DAMAGE_FACING_BY_DIRECTION: [u8; 4] = [4, 6, 0, 2];
            let cond = (ty == 0x13
                && SPRITE_DAMAGE_FACING_BY_DIRECTION
                    [usize::from(self.sprite_slot_view(k).direction() & 3)]
                    == self.game_state.player.follower_link.facing())
                || ty == 2;
            self.sprite_attempt_zap_damage(k);
            self.sprite_apply_recoil_to_link(k, 32);
            self.sprite_battle_mut().set_damaging_enemies_timer(16);
            self.follower_link_state_mut().set_incapacitated_timer(16);
            if cond {
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                self.link_place_weapon_tink();
            }
            return 0;
        } else if matches!(ty, 0xcb | 0xcd | 0xcc | 0xd6 | 0xd7 | 0xce | 0x54) {
            self.sprite_apply_recoil_to_link(k, 32);
            self.sprite_battle_mut().set_damaging_enemies_timer(144);
            self.follower_link_state_mut().set_incapacitated_timer(16);
        }

        if (self.sprite_slot_view(k).deflection_bits() & 4) == 0 {
            self.sprite_attempt_zap_damage(k);
            return CHECK_DAMAGE_FROM_PLAYER_CARRY;
        }

        self.sprite_check_damage_from_link_getting_out(k)
    }

    fn sprite_check_damage_from_link_getting_out(&mut self, k: usize) -> u8 {
        if self.game_state.sprite_battle.damaging_enemies_timer() == 0 {
            self.sprite_apply_recoil_to_link(k, 4);
            self.follower_link_state_mut().set_incapacitated_timer(16);
            self.sprite_battle_mut().set_damaging_enemies_timer(16);
        }
        self.link_place_weapon_tink();
        CHECK_DAMAGE_FROM_PLAYER_CARRY
    }

    // bool Sprite_CheckDamageToLink(int k) {  // 86f145
    //   if (link_disable_sprite_damage)
    //     return false;
    //   return Sprite_CheckDamageToPlayer_1(k);
    // }
    pub(super) fn sprite_check_damage_to_link(&mut self, k: usize) -> bool {
        self.game_state
            .player
            .follower_link
            .sprite_damage_disable_timer()
            == 0
            && self.sprite_check_damage_to_player_1(k)
    }

    // bool Sprite_CheckDamageToPlayer_1(int k) {  // 86f14a
    //   if ((k ^ frame_counter) & 3 | sprite_hit_timer[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_same_layer(k);
    // }
    pub(super) fn sprite_check_damage_to_player_1(&mut self, k: usize) -> bool {
        (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && self.sprite_slot_view(k).hit_timer() == 0
            && self.sprite_check_damage_to_link_same_layer(k)
    }

    // bool Sprite_CheckDamageToLink_same_layer(int k) {  // 86f154
    //   if (link_is_on_lower_level != sprite_floor[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_ignore_layer(k);
    // }
    pub(super) fn sprite_check_damage_to_link_same_layer(&mut self, k: usize) -> bool {
        self.game_state.player.follower_link.lower_level_state() == self.sprite_slot_view(k).floor()
            && self.sprite_check_damage_to_link_ignore_layer(k)
    }

    // bool Sprite_CheckDamageToLink_ignore_layer(int k) {  // 86f15c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_damage_to_link_ignore_layer(&mut self, k: usize) -> bool {
        let carry = if self.sprite_slot_view(k).flags4() != 0 {
            let mut hitbox = empty_sprite_hit_box();
            self.link_setup_hit_box(&mut hitbox);
            if (0xd8..=0xe6).contains(&self.sprite_slot_view(k).sprite_type())
                && self
                    .game_state
                    .enhanced_features
                    .has(FEATURES0_COLLECT_ITEMS_WITH_SWORD_SPRITE)
            {
                self.link_update_hit_box_with_sword(&mut hitbox);
            }
            self.sprite_setup_hit_box(k, &mut hitbox);
            self.check_if_hit_boxes_overlap(&hitbox)
        } else {
            self.sprite_setup_hit_box00(k)
        };

        if sign8(self.sprite_slot_view(k).flags2()) {
            return carry;
        }
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.game_state.world.location.is_indoors()
            && self.game_state.world.location.dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-ignore-layer fc={} k={} type=0x{:02x} carry={} flags2=0x{:02x} flags4=0x{:02x} flags5=0x{:02x} shield=0x{:02x} bunny=0x{:02x} statebits=0x{:02x} facing=0x{:02x} d=0x{:02x} aux=0x{:02x} link=0x{:04x},0x{:04x} cur=0x{:04x},0x{:04x} z=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).sprite_type(),
                carry,
                self.sprite_slot_view(k).flags2(),
                self.sprite_slot_view(k).flags4(),
                self.sprite_slot_view(k).flags5(),
                self.game_state.inventory.items.shield_type(),
                u8::from(self.game_state.player.follower_link.is_bunny_mirror()),
                self.game_state.player.follower_link.state_bits(),
                self.game_state.player.follower_link.facing(),
                self.sprite_slot_view(k).direction(),
                self.game_state.player.follower_link.auxiliary_state(),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.game_state.sprites.workspace.current_sprite_x(),
                self.game_state.sprites.workspace.current_sprite_y(),
                self.sprite_slot_view(k).z(),
            );
        }
        if !carry || self.game_state.player.follower_link.has_auxiliary_state() {
            return false;
        }

        if !self.game_state.player.follower_link.is_bunny_mirror()
            && !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
            && (self.sprite_slot_view(k).flags5() & 0x20) != 0
            && self.game_state.inventory.items.shield_type() != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            let t = if self.game_state.player.follower_link.button_b_frames() != 0 {
                SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SHIELD_BLOCK_FACING_TO_DIRECTION
                    [self.game_state.player.follower_link.facing_index() & 3]
            } else {
                self.game_state.player.follower_link.facing()
            };
            // ROM $06:F19F `LDY $0DE0,X`: the shield comparison indexes by the
            // sprite's direction and leaves it in Y (see
            // `rom_damage_check_y_register`).
            self.rom_damage_check_y_register = Some(self.sprite_slot_view(k).direction());
            if t == SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SPRITE_DAMAGE_FACING_BY_DIRECTION
                [usize::from(self.sprite_slot_view(k).direction() & 3)]
            {
                self.sprite_sfx_queue_sfx2_with_pan(k, 6);
                self.sprite_place_rupulse_spark_2(k);
                match self.sprite_slot_view(k).sprite_type() {
                    0x95 => {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                        return false;
                    }
                    0x9b => {
                        self.sprite_invert_xy_speeds(k);
                        self.sprite_slot_view_mut(k).xor_direction(1);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 9;
                        self.sprite_slot_view_mut(k).set_state(value);
                        return false;
                    }
                    0x1b => {
                        self.sprite_schedule_for_breakage(k);
                        return false;
                    }
                    0x0c => {
                        self.sprite_func3(k);
                        return true;
                    }
                    _ => return false,
                }
            }
        }

        self.sprite_attempt_damage_to_link_plus_recoil(k);
        if self.sprite_slot_view(k).sprite_type() == 0x0c {
            self.sprite_func3(k);
        }
        true
    }

    // void Sprite_AttemptDamageToLinkWithCollisionCheck(int k) {  // 86f3ca
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_damage_to_link_with_collision_check(&mut self, k: usize) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) != 0 {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        self.link_setup_hit_box_conditional(&mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.game_state.world.location.is_indoors()
            && self.game_state.world.location.dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-damage-check fc={} k={} type=0x{:02x} st=0x{:02x} bump=0x{:02x} link=0x{:04x},0x{:04x} overlap={} blink=0x{:02x} disable=0x{:02x} aux=0x{:02x} incap=0x{:02x} hp=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.sprite_slot_view(k).state(),
                self.sprite_slot_view(k).bump_damage(),
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                overlap,
                self.game_state.player.follower_link.blink_countdown(),
                self.game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer(),
                self.game_state.player.follower_link.auxiliary_state(),
                self.game_state.player.follower_link.incapacitated_timer(),
                self.game_state.inventory.player_resources.current_health(),
            );
        }
        if overlap {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
    }

    // void Guard_ParrySwordAttacks(int k) {  // 86eb5e
    pub(super) fn guard_parry_sword_attacks(&mut self, k: usize) {
        if self.game_state.player.follower_link.lower_level_state()
            != self.sprite_slot_view(k).floor()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.has_auxiliary_state()
            || sign8(self.sprite_slot_view(k).hit_timer())
        {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        if self.game_state.player.follower_link.position_mode_has(0x10)
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
        {
            self.sprite_attempt_damage_to_link_with_collision_check(k);
            return;
        }
        self.player_setup_action_hit_box(&mut hb);
        let button_neg = sign8(self.game_state.player.follower_link.button_b_frames());
        let action_overlap = self.check_if_hit_boxes_overlap(&hb);
        if std::env::var_os("ZELDA3_TRACE_GUARD_PARRY").is_some()
            && self.game_state.world.location.dungeon_room() == 0x00a8
            && self.sprite_slot_view(k).sprite_type() == 0xa7
            && k == 2
        {
            eprintln!(
                "R guard-parry action fc={} k={} button=0x{:02x} neg={} overlap={} link=0x{:04x},0x{:04x} spr=0x{:04x},0x{:04x} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.game_state.player.follower_link.button_b_frames(),
                button_neg,
                action_overlap,
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                hb.r0_xlo,
                hb.r8_xhi,
                hb.r1_ylo,
                hb.r9_yhi,
                hb.r2,
                hb.r3,
                hb.r4_spr_xlo,
                hb.r10_spr_xhi,
                hb.r5_spr_ylo,
                hb.r11_spr_yhi,
                hb.r6_spr_xsize,
                hb.r7_spr_ysize,
            );
        }
        if button_neg || !action_overlap {
            self.sprite_setup_hit_box(k, &mut hb);
            let body_overlap = self.check_if_hit_boxes_overlap(&hb);
            if std::env::var_os("ZELDA3_TRACE_GUARD_PARRY").is_some()
                && self.game_state.world.location.dungeon_room() == 0x00a8
                && self.sprite_slot_view(k).sprite_type() == 0xa7
                && k == 2
            {
                eprintln!(
                    "R guard-parry body fc={} k={} overlap={} hb={:02x}/{:02x},{:02x}/{:02x} sz={:02x},{:02x} sprhb={:02x}/{:02x},{:02x}/{:02x} sprsz={:02x},{:02x}",
                    self.game_state.frame.frame_counter,
                    k,
                    body_overlap,
                    hb.r0_xlo,
                    hb.r8_xhi,
                    hb.r1_ylo,
                    hb.r9_yhi,
                    hb.r2,
                    hb.r3,
                    hb.r4_spr_xlo,
                    hb.r10_spr_xhi,
                    hb.r5_spr_ylo,
                    hb.r11_spr_yhi,
                    hb.r6_spr_xsize,
                    hb.r7_spr_ysize,
                );
            }
            if !body_overlap {
                self.sprite_attempt_damage_to_link_with_collision_check(k);
            } else {
                self.sprite_attempt_zap_damage(k);
            }
            return;
        }
        if self.sprite_slot_view(k).sprite_type() != 0x6a {
            let j = usize::from(self.get_random_number() & 7);
            let value = GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_HITBOX_SIZE_BY_DIRECTION[j];
            self.sprite_slot_view_mut(k).set_f(value);
        }
        let j = usize::from(self.get_random_number() & 7);
        self.follower_link_state_mut().set_incapacitated_timer(
            GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_SWORD_STEP_BY_DIRECTION[j],
        );
        let fast_sword = sign8(
            self.game_state
                .player
                .follower_link
                .button_b_frames()
                .wrapping_sub(9),
        );
        let pt = self.sprite_project_speed_towards_link(k, if fast_sword { 32 } else { 24 });
        let value = 0u8.wrapping_sub(pt.x);
        self.sprite_slot_view_mut(k).set_x_recoil(value);
        let value = 0u8.wrapping_sub(pt.y);
        self.sprite_slot_view_mut(k).set_y_recoil(value);
        self.sprite_apply_recoil_to_link(k, if fast_sword { 8 } else { 16 });
        self.link_place_weapon_tink();
        self.sprite_battle_mut().set_damaging_enemies_timer(0x90);
    }

    // void Sprite_AttemptDamageToLinkPlusRecoil(int k) {  // 86f3db
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_damage_to_link_plus_recoil(&mut self, k: usize) {
        if std::env::var_os("ZELDA3_TRACE_SPRITE_DAMAGE").is_some()
            && self.game_state.world.location.is_indoors()
            && self.game_state.world.location.dungeon_room() == 0x00a8
        {
            eprintln!(
                "R sprite-damage-plus entry fc={} k={} type=0x{:02x} blink=0x{:02x} disable=0x{:02x} aux=0x{:02x} incap=0x{:02x} vx=0x{:02x} vy=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.game_state.player.follower_link.blink_countdown(),
                self.game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer(),
                self.game_state.player.follower_link.auxiliary_state(),
                self.game_state.player.follower_link.incapacitated_timer(),
                self.game_state.player.follower_link.actual_x_velocity(),
                self.game_state.player.follower_link.actual_y_velocity(),
            );
        }
        if (self.game_state.player.follower_link.blink_countdown()
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer())
            != 0
        {
            return;
        }

        self.follower_link_state_mut().set_incapacitated_timer(19);
        self.sprite_apply_recoil_to_link(k, 24);
        self.follower_link_state_mut().set_auxiliary_state(1);
        let idx = 3 * usize::from(self.sprite_slot_view(k).bump_damage() & 0x0f)
            + usize::from(self.game_state.inventory.items.armor());
        // ROM $06:F3FF `TAY`: the damage table index stays in Y for callers
        // that keep using Y afterwards (Sprite_C3_Gibo's pursuit target).
        self.rom_damage_check_y_register = Some(idx as u8);
        self.follower_link_state_mut()
            .set_given_damage(SPRITE_ATTEMPT_DAMAGE_TO_LINK_PLUS_RECOIL_PLAYER_DAMAGES[idx]);
        if self.sprite_slot_view(k).sprite_type() == 0x61 && self.sprite_slot_view(k).c() != 0 {
            let actual_x_velocity = self.sprite_slot_view(k).x_velocity().wrapping_mul(2);
            let actual_y_velocity = self.sprite_slot_view(k).y_velocity().wrapping_mul(2);
            self.follower_link_state_mut()
                .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
        }
    }

    // void Sprite_AttemptZapDamage(int k) {  // 86ec02
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_attempt_zap_damage(&mut self, k: usize) {
        let ty = self.sprite_slot_view(k).sprite_type();
        let electric = (ty == 0x7a
            || (ty == 0x0d && self.game_state.inventory.items.sword_type() < 4)
            || ((ty == 0x24 || ty == 0x23) && self.sprite_slot_view(k).delay_main() != 0))
            && self.sprite_slot_view(k).state() == 9;
        if electric {
            if self.game_state.player.follower_link.blink_countdown() == 0 {
                let value = 64;
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
                self.follower_link_state_mut().set_electrocute_on_touch(64);
                self.sprite_attempt_damage_to_link_plus_recoil(k);
            }
        } else {
            let vel = if sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                0x50
            } else {
                0x40
            };
            let pt = self.sprite_project_speed_towards_link(k, vel);
            let value = 0u8.wrapping_sub(pt.x);
            self.sprite_slot_view_mut(k).set_x_recoil(value);
            let value = 0u8.wrapping_sub(pt.y);
            self.sprite_slot_view_mut(k).set_y_recoil(value);
            self.sprite_calculate_sword_damage(k);
        }
    }

    // bool Sprite_CheckTileProperty(int k, int j) {  // 86e73c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_tile_property(&mut self, k: usize, j: i32) -> bool {
        let orig_j = j;
        let mut trace_tile_matches = std::env::var_os("ZELDA3_TRACE_TILE_COLLISION").is_some()
            && std::env::var("ZELDA3_TRACE_TILE_COLLISION_FRAME")
                .ok()
                .and_then(|s| {
                    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                        u8::from_str_radix(hex, 16).ok()
                    } else {
                        s.parse::<u8>().ok()
                    }
                })
                .map_or(true, |target| self.game_state.frame.frame_counter == target);
        if trace_tile_matches {
            if let Ok(value) = std::env::var("ZELDA3_TRACE_TILE_COLLISION_TYPE") {
                let target = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                    .or_else(|| value.parse::<u8>().ok());
                if target != Some(self.sprite_slot_view(k).sprite_type()) {
                    trace_tile_matches = false;
                }
            }
        }
        if trace_tile_matches {
            if let Ok(value) = std::env::var("ZELDA3_TRACE_TILE_COLLISION_SLOT") {
                if value.parse::<usize>().ok() != Some(k) {
                    trace_tile_matches = false;
                }
            }
        }
        let j = (j >> 1) as usize;

        let mut x;
        let y;
        let in_bounds;
        if self.game_state.world.location.is_indoors() {
            x = (self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(8)
                & 0x01ff)
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_X[j] as i16 as u16)
                .wrapping_sub(8);
            y = (self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(8)
                & 0x01ff)
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_Y[j] as i16 as u16)
                .wrapping_sub(8);
            in_bounds = x < 0x0200 && y < 0x0200;
        } else {
            x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_X[j] as i16 as u16);
            y = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_Y[j] as i16 as u16);
            in_bounds = x.wrapping_sub(self.game_state.sprites.garnish_runtime.sprcoll_x_word())
                < self.game_state.sprites.garnish_runtime.sprcoll_x_size()
                && y.wrapping_sub(self.game_state.sprites.garnish_runtime.sprcoll_y_word())
                    < self.game_state.sprites.garnish_runtime.sprcoll_y_size();
        }
        if !in_bounds {
            if trace_tile_matches {
                eprintln!(
                    "R tile fc={} k={} orig={} j={} x=0x{:04x} y=0x{:04x} in=0 floor=0x{:02x} flags2=0x{:02x} ret={}",
                    self.game_state.frame.frame_counter,
                    k,
                    orig_j,
                    j,
                    x,
                    y,
                    self.sprite_slot_view(k).floor(),
                    self.sprite_slot_view(k).flags2(),
                    if self.sprite_slot_view(k).flags2() & 0x40 != 0 {
                        0
                    } else {
                        1
                    }
                );
            }
            if self.sprite_slot_view(k).flags2() & 0x40 != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                return false;
            }
            return true;
        }
        let b = self.sprite_get_tile_attribute(k, &mut x, y);
        if trace_tile_matches {
            eprintln!(
                "R tile fc={} k={} orig={} j={} x=0x{:04x} y=0x{:04x} in=1 floor=0x{:02x} b=0x{:02x} tile=0x{:02x} defl=0x{:02x} flags5=0x{:02x} tab3=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                orig_j,
                j,
                x,
                y,
                self.sprite_slot_view(k).floor(),
                b,
                self.game_state.sprites.workspace.tile_type(),
                self.sprite_slot_view(k).deflection_bits(),
                self.sprite_slot_view(k).flags5(),
                SPRITE_CHECK_TILE_PROPERTY_SPRITE_TILE_ATTR_SIMPLIFIED[usize::from(b)] as u8
            );
        }
        if self.sprite_slot_view(k).deflection_bits() & 8 != 0 {
            let a = SPRITE_CHECK_TILE_PROPERTY_SIMPLIFIED_TILE_ATTR[usize::from(b)];
            if a == 4 {
                if self.game_state.world.location.is_outdoors() {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_e(value);
                }
            } else if a >= 1 {
                return if (0x10..0x14).contains(&self.game_state.sprites.workspace.tile_type()) {
                    self.entity_check_sloped_tile_collision(x, y)
                } else {
                    true
                };
            }
            return false;
        }

        if self.sprite_slot_view(k).flags5() & 0x40 != 0 {
            let typ = self.sprite_slot_view(k).sprite_type();
            if (typ == 0xd2 || typ == 0x8a) && b == 9 {
                return false;
            }
            if (typ == 0x94 && self.sprite_slot_view(k).e() == 0)
                || typ == 0xe3
                || typ == 0x8c
                || typ == 0x9a
                || typ == 0x81
            {
                return b != 8 && b != 9;
            }
        }

        if SPRITE_CHECK_TILE_PROPERTY_SPRITE_TILE_ATTR_SIMPLIFIED[usize::from(b)] == 0 {
            return false;
        }
        if (0x10..0x14).contains(&self.game_state.sprites.workspace.tile_type()) {
            return self.entity_check_sloped_tile_collision(x, y);
        }
        if self.game_state.sprites.workspace.tile_type() == 0x44 {
            if self.sprite_slot_view(k).f() != 0
                && !sign8(self.sprite_slot_view(k).incoming_damage())
            {
                if self.sprite_slot_view(k).sprite_type() == 0x88
                    && self.game_state.enhanced_features.has(4096)
                {
                    if self.sprite_slot_view(k).hit_timer() == 0 {
                        self.ancilla_check_damage_to_sprite_preset(k, 6);
                    }
                } else {
                    self.ancilla_check_damage_to_sprite_preset(k, 4);
                }
                if self.sprite_slot_view(k).hit_timer() != 0 {
                    let value = 153;
                    self.sprite_slot_view_mut(k).set_hit_timer(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_f(value);
                }
            }
        } else if self.game_state.sprites.workspace.tile_type() == 0x20 {
            return self.sprite_slot_view(k).flags() & 1 == 0 || self.sprite_slot_view(k).f() == 0;
        }
        true
    }

    // void Sprite_CheckForTileInDirection_horizontal(int k, int yy) {  // 86e5b8
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_for_tile_in_direction_horizontal(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }

        let idx = (yy as usize) & 3;
        self.sprite_slot_view_mut(k).or_wall_collision(
            SPRITE_CHECK_FOR_TILE_IN_DIRECTION_HORIZONTAL_SPRITE_TILE_DIRECTION_BITS[idx],
        );
        if (self.sprite_slot_view(k).subtype() & 7) < 5 {
            let n = if self.sprite_slot_view(k).f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(k, if (yy & 1) != 0 { -n } else { n }, 0);
        }
    }

    // void Sprite_CheckForTileInDirection_vertical(int k, int yy) {  // 86e5ee
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_for_tile_in_direction_vertical(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }

        let idx = (yy as usize) & 3;
        self.sprite_slot_view_mut(k).or_wall_collision(
            SPRITE_CHECK_FOR_TILE_IN_DIRECTION_VERTICAL_SPRITE_TILE_DIRECTION_BITS[idx],
        );
        if (self.sprite_slot_view(k).subtype() & 7) < 5 {
            let n = if self.sprite_slot_view(k).f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(k, 0, if (yy & 1) != 0 { -n } else { n });
        }
    }

    // bool Sprite_CheckTileInDirection(int k, int yy) {  // 86e72f
    //   uint8 t = (sprite_flags[k] & 0xf0);
    //   yy = 2 * ((t >> 2) + yy);
    //   return Sprite_CheckTileProperty(k, yy);
    // }
    pub(super) fn sprite_check_tile_in_direction(&mut self, k: usize, yy: i32) -> bool {
        let t = i32::from(self.sprite_slot_view(k).flags() & 0xf0);
        self.sprite_check_tile_property(k, 2 * ((t >> 2) + yy))
    }

    // bool Entity_CheckSlopedTileCollision(uint16 x, uint16 y) {  // 86e8fe
    //   ...see sprite.c...
    // }
    pub(super) fn entity_check_sloped_tile_collision(&mut self, x: u16, y: u16) -> bool {
        let a = (y & 7) as u8;
        let r6 = self
            .game_state
            .sprites
            .workspace
            .tile_type()
            .wrapping_sub(0x10);
        let b = ENTITY_CHECK_SLOPED_TILE_COLLISION_SLOPED_TILE
            [usize::from(r6) * 8 + usize::from(x & 7)];
        if r6 < 2 {
            b >= a
        } else {
            a >= b
        }
    }

    // void Sprite_DrawRippleIfInWater(int k) {  // 9eff8d
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_ripple_if_in_water(&mut self, k: usize) {
        if self.sprite_slot_view(k).draw_i() != 8 && self.sprite_slot_view(k).draw_i() != 9 {
            return;
        }
        if self.sprite_slot_view(k).flags3() & 0x20 != 0 {
            let x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_sub(4);
            self.sprite_workspace_mut().set_current_sprite_x(x);
            if self.sprite_slot_view(k).sprite_type() == 0xdf {
                let y = self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_y()
                    .wrapping_sub(7);
                self.sprite_workspace_mut().set_current_sprite_y(y);
            }
        }
        self.sprite_draw_water_ripple(k);
        self.sprite_get16_bit_coords(k);
        self.oam_allocate_from_region_a(((self.sprite_slot_view(k).flags2() & 0x1f) + 1) * 4);
    }

    // void ThrownSprite_CheckDamageToSprites(int k) {  // 86e172
    //   ...see sprite.c...
    // }
    pub(super) fn thrown_sprite_check_damage_to_sprites(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() != 0
            || (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity()) == 0
        {
            return;
        }
        for i in (0..=15usize).rev() {
            if i != self.game_state.sprites.system.cur_object_index() as usize
                && self.sprite_slot_view(k).sprite_type() != 0xd2
                && self.sprite_slot_view(i).state() >= 9
                && ((((i as u8) ^ self.game_state.frame.frame_counter) & 3)
                    | self.sprite_slot_view(i).ignore_projectile()
                    | self.sprite_slot_view(i).hit_timer())
                    == 0
                && self.sprite_slot_view(k).floor() == self.sprite_slot_view(i).floor()
            {
                self.thrown_sprite_check_damage_to_single_sprite(k, i as i32);
            }
        }
    }

    // void ThrownSprite_CheckDamageToSingleSprite(int k, int j) {  // 86e1b2
    //   ...see sprite.c...
    // }
    pub(super) fn thrown_sprite_check_damage_to_single_sprite(&mut self, k: usize, j: i32) {
        let j = j as usize;
        let t =
            i32::from(self.sprite_slot_view(k).y_low()) - i32::from(self.sprite_slot_view(k).z());
        let u = ((t & 0xff) + 8) as u8;
        let mut hb = SpriteHitBox {
            r0_xlo: self.sprite_slot_view(k).x_low(),
            r8_xhi: self.sprite_slot_view(k).x_high(),
            r1_ylo: u,
            r9_yhi: self
                .sprite_slot_view(k)
                .y_high()
                .wrapping_add(u8::from((t & 0xff) + 8 >= 0x100))
                .wrapping_sub(u8::from(t < 0)),
            r2: 15,
            r3: 8,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.sprite_setup_hit_box(j, &mut hb);
        if !self.check_if_hit_boxes_overlap(&hb) {
            return;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x3f {
            self.sprite_place_weapon_tink(k);
        } else {
            let a = if self.sprite_slot_view(k).sprite_type() == 0xec
                && self.sprite_slot_view(k).c() == 2
                && self.game_state.world.location.is_outdoors()
            {
                1
            } else {
                3
            };
            self.ancilla_check_damage_to_sprite_preset(j, a);
            let value = self.sprite_slot_view(k).x_velocity().wrapping_mul(2);
            self.sprite_slot_view_mut(j).set_x_recoil(value);
            let value = self.sprite_slot_view(k).y_velocity().wrapping_mul(2);
            self.sprite_slot_view_mut(j).set_y_recoil(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
        }
        self.sprite_apply_ricochet(k);
    }

    // void Sprite_KillFriends() {
    //   for(int j = 15; j >= 0; j--) {
    //     if (j != cur_object_index && sprite_state[j] && !(sprite_defl_bits[j] & 2)
    //         && sprite_type[j] != 0x7a) {
    //       sprite_state[j] = 6;
    //       sprite_delay_main[j] = 15;
    //       sprite_flags3[j] = 0;
    //       sprite_flags5[j] = 0;
    //       sprite_flags2[j] = 3;
    //     }
    //   }
    // }
    pub(super) fn sprite_kill_friends(&mut self) {
        let cur = self.game_state.sprites.system.cur_object_index() as usize;
        for j in (0..=15usize).rev() {
            if j == cur {
                continue;
            }
            if self.sprite_slot_view(j).state() == 0
                || (self.sprite_slot_view(j).deflection_bits() & 2) != 0
                || self.sprite_slot_view(j).sprite_type() == 0x7a
            {
                continue;
            }
            let value = 6;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 15;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags3(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags5(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
        }
    }

    // void Sprite_Func8(int k) {  // 86e0ab
    //   sprite_state[k] = 1;
    //   sprite_delay_main[k] = 0x1f;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 0x20);
    // }
    pub(super) fn sprite_func8(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0x1f;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
    }

    // void Sprite_Func22(int k) {  // 86e0f6
    //   sound_effect_1 = Sprite_CalculateSfxPan(k) | 0x28;
    //   sprite_state[k] = 3;
    //   sprite_delay_main[k] = 15;
    //   sprite_ai_state[k] = 0;
    //   GetRandomNumber(); // wtf
    //   sprite_flags2[k] = 3;
    // }
    pub(super) fn sprite_func22(&mut self, k: usize) {
        self.set_sound_effect_1_with_sprite_pan(k, 0x28);
        let value = 3;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 15;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_ai_state(value);
        self.get_random_number();
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_Func3(int k) {  // 86efda
    //   sprite_state[k] = 6;
    //   sprite_delay_main[k] = 31;
    //   sprite_flags2[k] = 3;
    // }
    pub(super) fn sprite_func3(&mut self, k: usize) {
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 31;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_SpawnSecret(int k) {  // 868264
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_spawn_secret(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            let before_rng = self.game_state.world.region.rng_seed();
            let roll = self.get_random_number();
            if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                println!(
                    "secret-spawn frame={} parent={} before=0x{:02x} roll=0x{:02x} b=0x{:02x} indoors={}",
                    self.game_state.frame.frame_counter,
                    k,
                    before_rng,
                    roll,
                    self.game_state.dungeon_secret.pending_kind(),
                    self.game_state.world.location.indoor_flag(),
                );
            }
            if (roll & 8) != 0 {
                return;
            }
        }
        let mut b = self.game_state.dungeon_secret.pending_kind();
        if !self.game_state.dungeon_secret.has_pending_kind() {
            return;
        }
        if b == 4 {
            b = 19 + (self.get_random_number() & 3);
        }
        let i = b.wrapping_sub(1) as usize;
        if i >= SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE.len()
            || SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE[i] == 0
        {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(
            k,
            SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE[i],
            &mut info,
        );
        if j < 0 {
            return;
        }
        let ju = j as usize;
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_SPAWN_FLAGS[i];
        self.sprite_slot_view_mut(ju).set_ai_state(value);
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_IGNORE_PROJECTILE_FLAGS[i];
        self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_Z_VELOCITIES[i];
        self.sprite_slot_view_mut(ju).set_z_velocity(value);
        self.sprite_set_x(
            ju,
            info.r0_x
                .wrapping_add(u16::from(SPRITE_SPAWN_SECRET_SECRET_ITEM_X_LOW_OFFSETS[i])),
        );
        self.sprite_set_y(ju, info.r2_y);
        let value = info.r4_z;
        self.sprite_slot_view_mut(ju).set_z(value);
        let value = 0;
        self.sprite_slot_view_mut(ju).set_graphics(value);
        let value = 32;
        self.sprite_slot_view_mut(ju).set_delay_aux4(value);
        let value = 48;
        self.sprite_slot_view_mut(ju).set_delay_aux2(value);

        let ty = self.sprite_slot_view(ju).sprite_type();
        if ty == 0xe4 {
            self.sprite_prep_small_key(ju);
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
        } else if ty == 0x0b {
            self.set_sound_effect_1(0x30);
            if self.game_state.dungeon.room_tracking.room_index2() == 1 {
                let value = 1;
                self.sprite_slot_view_mut(ju).set_subtype(value);
            }
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
        } else if ty == 0x41 || ty == 0x42 {
            self.set_sound_effect_2(4);
            let value = 0;
            self.sprite_slot_view_mut(ju).set_incoming_damage(value);
            let value = 160;
            self.sprite_slot_view_mut(ju).set_hit_timer(value);
        } else if ty == 0x3e {
            let value = 9;
            self.sprite_slot_view_mut(ju).set_oam_flags(value);
        } else {
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
            if ty == 0x79 {
                let value = 32;
                self.sprite_slot_view_mut(ju).set_a(value);
            }
        }
    }

    // void Ancilla_CheckDamageToSprite_preset(int k, int a) {  // 86ece0
    //   if (a == 15 && sprite_z[k] != 0)
    //     return;
    //   if (a != 0 && a != 7) {
    //     Sprite_Func15(k, a);
    //     return;
    //   }
    //   Sprite_Func15(k, a);
    //   if (sprite_give_damage[k] || repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int j = SPRITE_SHARED_WORK_A;
    //   repulsespark_x_lo = ancilla_x_lo[j] + 4;
    //   repulsespark_y_lo = ancilla_y_lo[j];
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    // }
    pub(super) fn ancilla_check_damage_to_sprite_preset(&mut self, k: usize, a: u8) {
        if a == 15 && self.sprite_slot_view(k).z() != 0 {
            return;
        }

        self.sprite_func15(k, a);
        if a != 0 && a != 7 {
            return;
        }
        if self.sprite_slot_view(k).incoming_damage() != 0
            || self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0
        {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(5);
        let j = self.game_state.sprites.workspace.shared_scratch_a() as usize;
        let ancilla = self.ancilla_slot_view(j);
        let x_low = ancilla.x_low().wrapping_add(4);
        let y_low = ancilla.y_low();
        self.garnish_state_mut().set_repulsespark_x_lo(x_low);
        self.garnish_state_mut().set_repulsespark_y_lo(y_low);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
    }

    // void Sprite_MiniMoldorm_Recoil(int k) {  // 86eec8
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_mini_moldorm_recoil(&mut self, k: usize) {
        if self.sprite_slot_view(k).state() < 9 {
            return;
        }
        let sprite_state = self.sprite_slot_view(k).state();
        self.temp_counter_mut().set(sprite_state);

        let dmg = self.sprite_slot_view(k).incoming_damage();
        if dmg == 253 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
            let value = 7;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0x70;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).flags2().wrapping_add(2);
            self.sprite_slot_view_mut(k).set_flags2(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }

        if dmg >= 251 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            if self.sprite_slot_view(k).state() == 11 {
                return;
            }
            let value = u8::from(dmg == 254);
            self.sprite_slot_view_mut(k).set_draw_work_byte_5(value);
            if self.sprite_slot_view(k).draw_work_byte_5() != 0 {
                self.sprite_slot_view_mut(k).or_deflection_bits(8);
                self.sprite_slot_view_mut(k).and_flags5(!0x80);
                self.sprite_sfx_queue_sfx2_with_pan(k, 15);
                let value = 24;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_slot_view_mut(k).and_bump_damage(!0x80);
                self.sprite_zero_velocity_xy(k);
            }
            let value = 11;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            const HIT_TIMER24_STUN_VALUES: [u8; 5] = [0x20, 0x80, 0, 0, 0xff];
            let value = HIT_TIMER24_STUN_VALUES[dmg.wrapping_add(5) as usize];
            self.sprite_slot_view_mut(k).set_stunned(value);
            if self.sprite_slot_view(k).sprite_type() == 0x23 {
                let value = 0x24;
                self.sprite_slot_view_mut(k).set_sprite_type(value);
            }
            return;
        }

        let t = i32::from(self.sprite_slot_view(k).health()) - i32::from(dmg);
        let value = t as u8;
        self.sprite_slot_view_mut(k).set_health(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_incoming_damage(value);
        if t > 0 {
            return;
        }

        if self.sprite_slot_view(k).die_action() == 0 {
            if self.sprite_slot_view(k).state() == 11 {
                let value = 3;
                self.sprite_slot_view_mut(k).set_die_action(value);
            }
            if self.sprite_slot_view(k).draw_work_byte_1() != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_draw_work_byte_1(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_flags5(value);
            }
        }

        let ty = self.sprite_slot_view(k).sprite_type();
        if ty != 0x1b {
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
        }

        if ty == 0x40 {
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            self.set_overworld_event_bits(screen, 0x40);
        } else if ty == 0xec {
            if self.sprite_slot_view(k).c() == 2 {
                self.throwable_scenery_transmute_to_debris(k);
            }
            return;
        }

        if self.sprite_slot_view(k).state() == 10 {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_picking_throw_state();
        }
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);

        if ty == 0x0c {
            self.sprite_func3(k);
        } else if ty == 0x92 {
            self.sprite_kill_friends();
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcb {
            let value = 128;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcc || ty == 0xcd {
            let value = 128;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 96;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x53 {
            let value = 35;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common2(k);
        } else if ty == 0x54 {
            let value = 5;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 0xc0;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0xc0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x09 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 160;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x7a {
            self.sprite_kill_friends();
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if !self.game_state.world.region.is_in_dark_world() {
                let value = 10;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 255;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            } else {
                let value = 255;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 8;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 9;
                self.sprite_slot_view_mut(1).set_ai_state(value);
                let value = 9;
                self.sprite_slot_view_mut(2).set_ai_state(value);
                let value = 0;
                self.sprite_slot_view_mut(1).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(2).set_graphics(value);
            }
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x23 && self.sprite_slot_view(k).c() == 0 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
        } else if ty == 0x0f {
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        } else if self.sprite_slot_view(k).flags() & 2 == 0 {
            let value = if self.sprite_slot_view(k).hit_timer() & 0x80 != 0 {
                31
            } else {
                15
            };
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).flags2().wrapping_add(4);
            self.sprite_slot_view_mut(k).set_flags2(value);
            if self.game_state.scratch_counter.value() == 11 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_flags5(value);
            }
        } else {
            if ty != 0xa2 {
                self.sprite_kill_friends();
            }
            let value = 4;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_a(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        }
    }

    fn sprite_mini_moldorm_recoil_out_common(&mut self, k: usize) {
        self.follower_link_state_mut().increment_menu_block_flag();
        self.sprite_mini_moldorm_recoil_out_common2(k);
    }

    fn sprite_mini_moldorm_recoil_out_common2(&mut self, k: usize) {
        self.set_sound_effect_2(0);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x22);
    }

    // bool Sprite_ReturnIfRecoiling(int k) {
    //   ...see sprite.c:3143...
    // }
    pub(super) fn sprite_return_if_recoiling(&mut self, k: usize) -> bool {
        let trace_recoil_matches = std::env::var_os("ZELDA3_TRACE_RECOIL").is_some()
            && std::env::var("ZELDA3_TRACE_RECOIL_FRAME")
                .ok()
                .and_then(|value| {
                    let trimmed = value.trim();
                    if let Some(hex) = trimmed.strip_prefix("0x") {
                        u8::from_str_radix(hex, 16).ok()
                    } else {
                        trimmed.parse::<u8>().ok()
                    }
                })
                .is_none_or(|frame| frame == self.game_state.frame.frame_counter);
        if self.sprite_slot_view(k).f() == 0 {
            return false;
        }
        if self.sprite_slot_view(k).f() & 0x7f == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            return false;
        }
        let yvbak = self.sprite_slot_view(k).y_velocity();
        let xvbak = self.sprite_slot_view(k).x_velocity();
        if trace_recoil_matches {
            eprintln!(
                "R recoil fc={} entry k={} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} xv=0x{:02x} yv=0x{:02x} bump=0x{:02x} x=0x{:04x} y=0x{:04x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).f(),
                self.sprite_slot_view(k).x_recoil(),
                self.sprite_slot_view(k).y_recoil(),
                self.sprite_slot_view(k).x_velocity(),
                self.sprite_slot_view(k).y_velocity(),
                self.sprite_slot_view(k).bump_damage(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        let new_f = self.sprite_slot_view(k).f().wrapping_sub(1);
        let value = new_f;
        self.sprite_slot_view_mut(k).set_f(value);
        if new_f == 0
            && (self.sprite_slot_view(k).x_recoil().wrapping_add(0x20) >= 0x40
                || self.sprite_slot_view(k).y_recoil().wrapping_add(0x20) >= 0x40)
        {
            let value = 144;
            self.sprite_slot_view_mut(k).set_f(value);
        }
        let i = self.sprite_slot_view(k).f();
        // !sign8(i) -> top bit clear
        if (i & 0x80) == 0
            && (self.game_state.frame.frame_counter
                & SPRITE_RETURN_IF_RECOILING_SPRITE_RECOIL_DIRECTION_MASKS[(i >> 2) as usize])
                == 0
        {
            let value = self.sprite_slot_view(k).y_recoil();
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = self.sprite_slot_view(k).x_recoil();
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let bump = self.sprite_slot_view(k).bump_damage();
            let t = if (bump as i8) >= 0 {
                self.sprite_check_tile_collision(k) & 0xf
            } else {
                0
            };
            if trace_recoil_matches {
                eprintln!(
                    "R recoil fc={} collide k={} i=0x{:02x} mask=0x{:02x} t=0x{:02x} xv=0x{:02x} yv=0x{:02x} x=0x{:04x} y=0x{:04x}",
                    self.game_state.frame.frame_counter,
                    k,
                    i,
                    SPRITE_RETURN_IF_RECOILING_SPRITE_RECOIL_DIRECTION_MASKS[(i >> 2) as usize],
                    t,
                    self.sprite_slot_view(k).x_velocity(),
                    self.sprite_slot_view(k).y_velocity(),
                    self.sprite_get_x(k),
                    self.sprite_get_y(k),
                );
            }
            if (bump as i8) >= 0 && t != 0 {
                if t < 4 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_recoil(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_recoil(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            } else {
                self.sprite_move_xy(k);
                if trace_recoil_matches {
                    eprintln!(
                        "R recoil fc={} move k={} x=0x{:04x} y=0x{:04x}",
                        self.game_state.frame.frame_counter,
                        k,
                        self.sprite_get_x(k),
                        self.sprite_get_y(k),
                    );
                }
            }
        }
        let value = yvbak;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = xvbak;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        if trace_recoil_matches {
            eprintln!(
                "R recoil fc={} exit k={} ret={} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} x=0x{:04x} y=0x{:04x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_slot_view(k).sprite_type() != 0x7a,
                self.sprite_slot_view(k).f(),
                self.sprite_slot_view(k).x_recoil(),
                self.sprite_slot_view(k).y_recoil(),
                self.sprite_get_x(k),
                self.sprite_get_y(k),
            );
        }
        self.sprite_slot_view(k).sprite_type() != 0x7a
    }

    // void Sprite_DrawMultiple(int k, const DrawMultipleData *src, int n,
    //                          PrepOamCoordsRet *info)
    //   See sprite.c:900.
    // Mirrors C: if `info` is None we use a local buffer; otherwise the
    // caller's out-pointer is populated.
    //
    // C always populates info->x / info->y before the out-of-bounds return,
    // because Sprite_PrepOamCoordOrDoubleRet writes the coords then returns a
    // bool.  We mirror that by calling the raw variant and always writing info
    // before the early-return check.
    pub(super) fn sprite_draw_multiple(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: Option<&mut PrepOamCoordsRet>,
    ) {
        let Some(info) = self.sprite_prepare_draw_multiple(k, info) else {
            return;
        };
        self.sprite_draw_multiple_words_with_info(
            k,
            src.iter().map(|entry| DrawMultipleWordData {
                x: entry.x as i16 as u16,
                y: entry.y as i16 as u16,
                char_flags: entry.char_flags,
                ext: entry.ext,
            }),
            info,
        );
    }

    fn sprite_prepare_draw_multiple(
        &mut self,
        k: usize,
        info: Option<&mut PrepOamCoordsRet>,
    ) -> Option<(u16, u16, u8)> {
        let (prepped, out_of_bounds) = self.sprite_prep_oam_coord_or_double_ret_raw(k);
        if let Some(out) = info {
            out.x = prepped.x;
            out.y = prepped.y;
            out.r4 = 0;
            out.flags = prepped.flags;
        }
        if out_of_bounds {
            return None;
        }
        Some((prepped.x, prepped.y, prepped.flags))
    }

    /// Draw the ROM's native eight-byte `DrawMultipleData` records after a
    /// 16-bit table-address calculation has wrapped into low WRAM.
    pub(super) fn sprite_draw_multiple_from_wram_records<const N: usize>(
        &mut self,
        k: usize,
        source: u16,
        info: Option<&mut PrepOamCoordsRet>,
    ) {
        let mut cursor = source;
        let entries: [DrawMultipleWordData; N] = std::array::from_fn(|_| {
            let word = |address: u16| {
                u16::from(self.ram[usize::from(address)])
                    | (u16::from(self.ram[usize::from(address.wrapping_add(1))]) << 8)
            };
            let entry = DrawMultipleWordData {
                x: word(cursor),
                y: word(cursor.wrapping_add(2)),
                char_flags: word(cursor.wrapping_add(4)),
                ext: self.ram[usize::from(cursor.wrapping_add(7))],
            };
            cursor = cursor.wrapping_add(8);
            entry
        });
        let Some(info) = self.sprite_prepare_draw_multiple(k, info) else {
            return;
        };
        self.sprite_draw_multiple_words_with_info(k, entries, info);
    }

    // Variant that takes a precomputed PrepOamCoord triple (x, y, flags). The
    // C version mutates a caller-supplied PrepOamCoordsRet pointer; Rust
    // callers pass the triple directly so the API stays panic-free.
    pub(super) fn sprite_draw_multiple_with_info(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: (u16, u16, u8),
    ) {
        self.sprite_draw_multiple_words_with_info(
            k,
            src.iter().map(|entry| DrawMultipleWordData {
                x: entry.x as i16 as u16,
                y: entry.y as i16 as u16,
                char_flags: entry.char_flags,
                ext: entry.ext,
            }),
            info,
        );
    }

    fn sprite_draw_multiple_words_with_info(
        &mut self,
        k: usize,
        src: impl IntoIterator<Item = DrawMultipleWordData>,
        info: (u16, u16, u8),
    ) {
        let (info_x, info_y, info_flags) = info;
        // r4 is always 0 in C's Sprite_PrepOamCoordOrDoubleRet (sprite.c:1843).
        let info_r4: u8 = 0;
        self.sprite_workspace_mut().clear_draw_priority_override();
        let mut a = self.sprite_slot_view(k).state();
        if a == 10 {
            a = self.sprite_slot_view(k).draw_work_byte_4();
        }
        if a == 11 {
            let priority = self.sprite_slot_view(k).draw_work_byte_5();
            self.sprite_workspace_mut()
                .set_draw_priority_override_low(priority);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let combined_flags = (u16::from(info_flags) << 8) | u16::from(info_r4);
        for entry in src {
            let mut d = entry.char_flags ^ combined_flags;
            if self.game_state.sprites.workspace.draw_priority_override() >= 1 {
                d = (d & !0x0e00) | 0x0400;
            }
            let x = info_x.wrapping_add(entry.x);
            let y = info_y.wrapping_add(entry.y);
            self.set_oam_helper0_at(oam, x, y, d as u8, (d >> 8) as u8, entry.ext);
            oam += 4;
        }
    }

    // void Sprite_DrawMultiplePlayerDeferred(int k, ...) {
    //   Oam_AllocateDeferToPlayer(k);
    //   Sprite_DrawMultiple(k, src, n, info);
    // }
    pub(super) fn sprite_draw_multiple_player_deferred(
        &mut self,
        k: usize,
        src: &[DrawMultipleData],
        info: Option<&mut PrepOamCoordsRet>,
    ) {
        self.oam_allocate_defer_to_player(k);
        self.sprite_draw_multiple(k, src, info);
    }

    fn sprite_single_draw_char(&self, k: usize) -> u8 {
        let base = SINGLE_LARGE_SPRITE_CHAR_BASE_BY_TYPE
            .get(usize::from(self.sprite_slot_view(k).sprite_type()))
            .copied()
            .unwrap_or(0);
        SINGLE_LARGE_SPRITE_CHAR_BY_BASE_AND_GFX
            .get(usize::from(base) + usize::from(self.sprite_slot_view(k).graphics()))
            .copied()
            .unwrap_or(0)
    }

    // void SpriteDraw_SingleLarge(int k) {  // 86dc10
    //   PrepOamCoordsRet info;
    //   if (Sprite_PrepOamCoordOrDoubleRet(k, &info))
    //     return;
    //   Sprite_PrepAndDrawSingleLargeNoPrep(k, &info);
    // }
    pub(super) fn sprite_draw_single_large(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_prep_and_draw_single_large_no_prep(k, &mut info);
    }

    // void Sprite_PrepAndDrawSingleLargeNoPrep(int k, PrepOamCoordsRet *info) {  // 86dc13
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_prep_and_draw_single_large_no_prep(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        let oam = self.game_state.oam.current_pointer_usize();
        let chr = self.sprite_single_draw_char(k);
        self.oam_state_mut().set_entry_x(oam, info.x as u8);
        if info.y.wrapping_add(0x10) < 0x100 {
            self.oam_state_mut().set_entry_y(oam, info.y as u8);
            self.oam_state_mut().set_entry_char(oam, chr);
            self.oam_state_mut().set_entry_flags(oam, info.flags);
        }
        let ext_index = (oam - OAM_BUF) / 4;
        let value = 2 | u8::from(info.x >= 256);
        self.oam_state_mut().set_extended_byte(ext_index, value);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            self.sprite_draw_shadow_custom(k, info, 10);
        }
    }

    // void SpriteDraw_Shadow_custom(int k, PrepOamCoordsRet *info, uint8 a) {  // 86dc5c
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_shadow_custom(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
        a: u8,
    ) {
        let mut y = self.sprite_get_y(k).wrapping_add(u16::from(a));
        info.y = y;
        if self.sprite_slot_view(k).pause() != 0
            || (self.sprite_slot_view(k).state() == 10
                && self.sprite_slot_view(k).draw_work_byte_3() == 3)
        {
            return;
        }
        y = y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        info.y = y;
        if y.wrapping_add(0x10) >= 0x100 {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize()
            + usize::from(self.sprite_slot_view(k).flags2() & 0x1f) * 4;
        if self.sprite_slot_view(k).flags3() & 0x20 != 0 {
            self.set_oam_helper1_at(
                oam,
                info.x,
                y.wrapping_add(1) as u8,
                0x38,
                (info.flags & 0x30) | 8,
                0,
            );
        } else {
            self.set_oam_helper1_at(oam, info.x, y as u8, 0x6c, (info.flags & 0x30) | 8, 2);
        }
    }

    // void SpriteDraw_SingleSmall(int k) {  // 86dcef
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_single_small(&mut self, k: usize) {
        let Some(continuation) = self.sprite_draw_single_small_position_prefix(k) else {
            return;
        };
        self.sprite_draw_single_small_after_position(k, continuation);
    }

    fn sprite_draw_single_small_position_prefix(
        &mut self,
        k: usize,
    ) -> Option<SingleSmallDrawContinuation> {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return None;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        self.oam_state_mut().set_entry_x(oam, x as u8);
        // The ROM publishes the extended-OAM size/X bit before its vertical
        // clipping branch ($86:DCFD), not after the character/flags stores as
        // the decompilation orders it. This is observable when vblank lands
        // at the source statement after the visible Y write.
        let ext_index = (oam - OAM_BUF) / 4;
        let value = u8::from(x >= 256);
        self.oam_state_mut().set_extended_byte(ext_index, value);
        let visible = y.wrapping_add(0x10) < 0x100;
        if visible {
            self.oam_state_mut().set_entry_y(oam, y as u8);
        }
        Some(SingleSmallDrawContinuation {
            x,
            y,
            oam: u16::try_from(oam).expect("single-small OAM pointer exceeded one word"),
            flags,
            visible,
        })
    }

    fn sprite_draw_single_small_after_position(
        &mut self,
        k: usize,
        continuation: SingleSmallDrawContinuation,
    ) {
        let SingleSmallDrawContinuation {
            x,
            y,
            oam,
            flags,
            visible,
        } = continuation;
        if visible {
            let chr = self.sprite_single_draw_char(k);
            self.oam_state_mut().set_entry_char(usize::from(oam), chr);
            self.oam_state_mut()
                .set_entry_flags(usize::from(oam), flags);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 2);
        }
    }

    // void Sprite_DrawThinAndTall(int k) {  // 86dd40
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_thin_and_tall(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let chr = self.sprite_single_draw_char(k);
        self.set_oam_helper0_at(oam, x, y, chr, flags, 0);
        self.set_oam_helper0_at(
            oam + 4,
            x,
            y.wrapping_add(8),
            chr.wrapping_add(0x10),
            flags,
            0,
        );
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut info = PrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // void SpriteFall_Draw(int k, PrepOamCoordsRet *info) {  // 9dffc5
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_fall_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        let oam = self.game_state.oam.current_pointer_usize();
        let idx = usize::from(self.sprite_slot_view(k).delay_main() >> 2);
        self.oam_state_mut().write_entry(
            oam,
            info.x.wrapping_add(4) as u8,
            info.y.wrapping_add(4) as u8,
            SPRITE_FALL_DRAW_SPRITE_FALL_CHAR[idx],
            (info.flags & 0x30) | 0x04,
        );
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // void Sprite_DrawDistress_custom(uint16 xin, uint16 yin, uint8 time) {  // 86a733
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_distress_custom(&mut self, xin: u16, yin: u16, time: u8) {
        self.oam_allocate_from_region_a(0x10);
        if (time & 0x18) == 0 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            self.set_oam_helper0_at(
                oam,
                xin.wrapping_add(SPRITE_DRAW_DISTRESS_CUSTOM_X_OFFSETS[i] as i16 as u16),
                yin.wrapping_add(SPRITE_DRAW_DISTRESS_CUSTOM_Y_OFFSETS[i] as i16 as u16),
                0x83,
                0x22,
                0,
            );
            oam += 4;
        }
    }

    // void SpriteDraw_FallingHelmaBeetle(int k) {  // 86fd17
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_falling_helma_beetle(&mut self, k: usize) {
        let mut base = usize::from(self.sprite_slot_view(k).graphics()).min(5);
        if self.sprite_slot_view(k).sprite_type() == 0x13 {
            base += 6;
        }
        self.sprite_draw_multiple(
            k,
            &SPRITE_DRAW_FALLING_HELMA_BEETLE_FALL0[base..base + 1],
            None,
        );
    }

    // void SpriteDraw_FallingHumanoid(int k) {  // 86fe5b
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_draw_falling_humanoid(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let q = usize::from(self.sprite_slot_view(k).graphics());
        let mut oam = self.game_state.oam.current_pointer_usize();
        let n = if q < 12 && (q & 3) == 0 { 3 } else { 0 };
        for n_cur in (0..=n).rev() {
            let i = q * 4 + n_cur;
            self.set_oam_plain_at_for_sprite(
                oam,
                x.wrapping_add(SPRITE_DRAW_FALLING_HUMANOID_X_OFFSETS[i] as i16 as u16) as u8,
                y.wrapping_add(SPRITE_DRAW_FALLING_HUMANOID_Y_OFFSETS[i] as i16 as u16) as u8,
                SPRITE_DRAW_FALLING_HUMANOID_CHARS[i],
                flags ^ SPRITE_DRAW_FALLING_HUMANOID_FLAGS[i],
                SPRITE_DRAW_FALLING_HUMANOID_EXT_SIZES[i],
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries(k, n as i32, 0xff);
    }

    // void ScatterDebris_Draw(int k, Point16U pt) {  // 89f198
    //   ...see sprite.c...
    // }
    pub(super) fn scatter_debris_draw(&mut self, k: usize, pt: Point16U) {
        if self.garnish_slot_view(k).countdown() == 16 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let base = usize::from(((self.garnish_slot_view(k).countdown() & 0x0f) >> 2) * 3);
        for i in (0..=2usize).rev() {
            let j = base + i;
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(SCATTER_DEBRIS_DRAW_X_OFFSETS[j] as i16 as u16),
                pt.y.wrapping_add(SCATTER_DEBRIS_DRAW_Y_OFFSETS[j] as i16 as u16) as u8,
                SCATTER_DEBRIS_DRAW_CHARS[j],
                SCATTER_DEBRIS_DRAW_FLAGS[j] | 0x22,
                0,
            );
            oam += 4;
        }
    }

    // void Garnish16_ThrownItemDebris(int k) {  // 89f0cb
    pub(super) fn garnish16_thrown_item_debris(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let r5 = self.garnish_slot_view(k).oam_flags();
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            return;
        }
        if self.garnish_slot_view(k).sprite() == 3 {
            self.scatter_debris_draw(k, pt);
            return;
        }
        let garnish_sprite = self.garnish_slot_view(k).sprite();
        self.temp_counter_mut().set(garnish_sprite);
        let mut base = ((self.garnish_slot_view(k).countdown() >> 2) ^ 7) << 2;
        if self.game_state.scratch_counter.value() == 4
            || (self.game_state.scratch_counter.value() == 2
                && self.game_state.world.location.is_outdoors())
        {
            base = base.wrapping_add(0x20);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..=3usize).rev() {
            let j = usize::from(base) + i;
            let chr = if self.game_state.scratch_counter.value() == 0 {
                0x4e
            } else if self.game_state.scratch_counter.value() >= 0x80 {
                0xf2
            } else {
                GARNISH16_THROWN_ITEM_DEBRIS_CHARS[j]
            };
            self.set_oam_helper1_at(
                oam,
                pt.x.wrapping_add(GARNISH16_THROWN_ITEM_DEBRIS_X_OFFSETS[j] as u16),
                pt.y.wrapping_add(GARNISH16_THROWN_ITEM_DEBRIS_Y_OFFSETS[j] as i16 as u16) as u8,
                chr,
                GARNISH16_THROWN_ITEM_DEBRIS_FLAGS[j] | r5,
                0,
            );
            oam += 4;
        }
    }

    // void Oam_AllocateDeferToPlayer(int k) — sprite.c:2920
    pub(super) fn oam_allocate_defer_to_player(&mut self, k: usize) {
        if self.sprite_slot_view(k).floor()
            != self.game_state.player.follower_link.lower_level_state()
        {
            return;
        }
        let right = self.sprite_is_right_of_link(k);
        if right.b.wrapping_add(0x10) >= 0x20 {
            return;
        }
        let below = self.sprite_is_below_link(k);
        if below.b.wrapping_add(0x20) >= 0x48 {
            return;
        }
        let nslots = ((self.sprite_slot_view(k).flags2() & 0x1f) + 1) << 2;
        if below.a != 0 {
            self.oam_allocate_from_region_c(nslots);
        } else {
            self.oam_allocate_from_region_b(nslots);
        }
    }

    // bool Sprite_ReturnIfLifted(int k) — sprite.c:2602
    pub(super) fn sprite_return_if_lifted(&mut self, k: usize) -> bool {
        if self.game_state.frame.submodule != 0
            || self.game_state.player.follower_link.button_b_frames() != 0
            || self.game_state.frame.modal_pause_flag != 0
            || self.sprite_slot_view(k).floor()
                != self.game_state.player.follower_link.lower_level_state()
        {
            return false;
        }
        for j in (0..=15usize).rev() {
            if self.sprite_slot_view(j).state() == 10 {
                return false;
            }
        }
        if self.sprite_slot_view(k).sprite_type() != 0xb
            && self.sprite_slot_view(k).sprite_type() != 0x4a
            && (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity()) != 0
        {
            return false;
        }
        if self.game_state.player.follower_link.is_running() {
            return false;
        }
        self.sprite_return_if_lifted_permissive(k)
    }

    // bool Sprite_ReturnIfLiftedPermissive(int k) — sprite.c:2615
    pub(super) fn sprite_return_if_lifted_permissive(&mut self, k: usize) -> bool {
        if self.game_state.player.follower_link.is_running() {
            return false;
        }
        if self
            .game_state
            .player
            .follower_link
            .sprite_pickup_flag_cached()
            .wrapping_sub(1)
            != self.game_state.sprites.system.cur_object_index()
        {
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
                let v = (k as u8).wrapping_add(1);
                self.sprite_workspace_mut().set_pickup_slot_cache(v);
                self.follower_link_state_mut().set_sprite_pickup_flag(v);
            }
            false
        } else {
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
            let dir = self.sprite_direction_to_face_link(k, None) as usize;
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
    pub(super) fn sprite_check_if_lifted_permissive(&mut self, k: usize) {
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
    pub(super) fn sprite_direction_to_face_link(
        &mut self,
        k: usize,
        coords_out: Option<&mut PointU8>,
    ) -> u8 {
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
        if let Some(coords) = coords_out {
            coords.x = right.b;
            coords.y = below.b;
        }
        if xm >= ym {
            right.a
        } else {
            below.a + 2
        }
    }

    // int Sprite_SpawnDynamically(int k, uint8 what, SpriteSpawnInfo *info) { // 9df65d
    //   return Sprite_SpawnDynamicallyEx(k, what, info, 15);
    // }
    //
    // Canonical 1:1 port. Returns -1 if no slot was found, otherwise the
    // 0..15 slot index. `info` is populated with the spawn coordinates the
    // caller will consume via `Sprite_SetSpawnedCoordinates`.
    pub(super) fn sprite_spawn_dynamically(
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
    pub(super) fn sprite_spawn_dynamically_ex(
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
                if std::env::var_os("ZELDA3_REPLAY_SPRITE_SPAWN_SCAN_DUMP").is_some() {
                    println!(
                        "dyn-scan frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                        self.game_state.frame.frame_counter,
                        k,
                        what,
                        ju,
                        self.sprite_slot_view(ju).sprite_type(),
                        self.sprite_slot_view(ju).state(),
                        self.sprite_slot_view(ju).c(),
                        self.sprite_slot_view(ju).bump_damage(),
                    );
                }
                if std::env::var_os("ZELDA3_REPLAY_SPRITE_LOAD_DUMP").is_some() {
                    println!(
                        "dyn-spawn frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                        self.game_state.frame.frame_counter,
                        k,
                        what,
                        ju,
                        self.sprite_slot_view(ju).sprite_type(),
                        self.sprite_slot_view(ju).state(),
                        self.sprite_slot_view(ju).c(),
                        self.sprite_slot_view(ju).bump_damage(),
                    );
                }
                self.sprite_spawn_dynamically_selected_prefix(
                    k,
                    what,
                    info,
                    ju,
                    crate::SpriteDynamicSpawnProgress::SubtypeCleared,
                );
                break;
            }
            if j >= 0 && std::env::var_os("ZELDA3_REPLAY_SPRITE_SPAWN_SCAN_DUMP").is_some() {
                let ju = j as usize;
                println!(
                    "dyn-scan frame={} parent={} what=0x{:02x} slot={} old_t=0x{:02x} old_st=0x{:02x} old_c=0x{:02x} old_bump=0x{:02x}",
                    self.game_state.frame.frame_counter,
                    k,
                    what,
                    ju,
                    self.sprite_slot_view(ju).sprite_type(),
                    self.sprite_slot_view(ju).state(),
                    self.sprite_slot_view(ju).c(),
                    self.sprite_slot_view(ju).bump_damage(),
                );
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

    pub(super) fn sprite_spawn_dynamically_selected_prefix(
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
            _ => self.sprite_prep_load_properties_for_helpers(j),
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

    pub(super) fn sprite_spawn_dynamically_selected_from(
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
                self.sprite_prep_load_properties_for_helpers(j);
            }
            crate::SpriteDynamicSpawnProgress::StatePublished => {
                self.sprite_prep_load_properties_for_helpers(j);
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
    pub(super) fn release_fairy(&mut self) -> i32 {
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

    // uint8 Sprite_CheckTileCollision(int k) {  // 85b88d
    //   Sprite_CheckTileCollision2(k);
    //   return sprite_wallcoll[k];
    // }
    //
    // Canonical 1:1 port. Runs the 2-layer collision pass and returns the
    // cached `sprite_wallcoll[k]` byte.
    pub(super) fn sprite_check_tile_collision(&mut self, k: usize) -> u8 {
        self.sprite_check_tile_collision2(k);
        self.sprite_slot_view(k).wall_collision()
    }

    // void Sprite_CheckTileCollision2(int k) {  // 86e4ab
    //   sprite_wallcoll[k] = 0;
    //   if (sign8(sprite_flags4[k]) || !dung_hdr_collision) {
    //     Sprite_CheckTileCollisionSingleLayer(k);
    //     return;
    //   }
    //   SPRITE_SHARED_WORK_A = sprite_floor[k];
    //   sprite_floor[k] = 1;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   if (dung_hdr_collision == 4) {
    //     sprite_floor[k] = SPRITE_SHARED_WORK_A;
    //     return;
    //   }
    //   sprite_floor[k] = 0;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   byte_7FFABC[k] = sprite_tiletype;
    // }
    //
    // 1:1 port of the dispatcher around `Sprite_CheckTileCollisionSingleLayer`.
    pub(super) fn sprite_check_tile_collision2(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_wall_collision(value);
        let f4 = self.sprite_slot_view(k).flags4();
        let dung_coll = self.game_state.dungeon.room_load.header_collision();
        // sign8: top bit set.
        if (f4 & 0x80) != 0 || dung_coll == 0 {
            self.sprite_check_tile_collision_single_layer(k);
            return;
        }
        let floor = self.sprite_slot_view(k).floor();
        self.sprite_workspace_mut().set_shared_scratch_a(floor);
        let value = 1;
        self.sprite_slot_view_mut(k).set_floor(value);
        self.sprite_check_tile_collision_single_layer(k);
        if dung_coll == 4 {
            let value = self.game_state.sprites.workspace.shared_scratch_a();
            self.sprite_slot_view_mut(k).set_floor(value);
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_floor(value);
        self.sprite_check_tile_collision_single_layer(k);
        // byte_7FFABC[k] = sprite_tiletype — write-through to the
        // dual-layer cache so the next iteration sees the lower-layer tile.
        let tt = self.game_state.sprites.workspace.tile_type();
        self.dual_layer_tile_cache_mut().set_tile_type(k, tt);
    }

    // void Sprite_CheckTileCollisionSingleLayer(int k) {  // 86e4db
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_check_tile_collision_single_layer(&mut self, k: usize) {
        if self.sprite_slot_view(k).flags2() & 0x20 != 0 {
            if self.sprite_check_tile_property(k, 0x6a) {
                let value = self.sprite_slot_view(k).wall_collision().wrapping_add(1);
                self.sprite_slot_view_mut(k).set_wall_collision(value);
            }
            return;
        }

        if sign8(self.sprite_slot_view(k).flags4())
            || self.game_state.dungeon.room_load.header_collision() == 0
        {
            if self.sprite_slot_view(k).y_velocity() != 0 {
                self.sprite_check_for_tile_in_direction_vertical(
                    k,
                    if sign8(self.sprite_slot_view(k).y_velocity()) {
                        0
                    } else {
                        1
                    },
                );
            }
            if self.sprite_slot_view(k).x_velocity() != 0 {
                self.sprite_check_for_tile_in_direction_horizontal(
                    k,
                    if sign8(self.sprite_slot_view(k).x_velocity()) {
                        2
                    } else {
                        3
                    },
                );
            }
        } else {
            self.sprite_check_for_tile_in_direction_vertical(k, 1);
            self.sprite_check_for_tile_in_direction_vertical(k, 0);
            self.sprite_check_for_tile_in_direction_horizontal(k, 3);
            self.sprite_check_for_tile_in_direction_horizontal(k, 2);
        }

        if sign8(self.sprite_slot_view(k).flags5()) || self.sprite_slot_view(k).z() != 0 {
            return;
        }

        self.sprite_check_tile_property(k, 0x68);
        let value = self.game_state.sprites.workspace.tile_type();
        self.sprite_slot_view_mut(k).set_draw_i(value);
        match self.game_state.sprites.workspace.tile_type() {
            0x1c => {
                if self.game_state.oam.has_sprite_sorting()
                    && self.sprite_slot_view(k).state() == 11
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_floor(value);
                }
            }
            0x20 => {
                if self.sprite_slot_view(k).flags() & 1 != 0 {
                    if self.game_state.world.location.is_outdoors() {
                        self.sprite_func8(k);
                    } else {
                        let value = 5;
                        self.sprite_slot_view_mut(k).set_state(value);
                        if self.sprite_slot_view(k).sprite_type() == 0x13
                            || self.sprite_slot_view(k).sprite_type() == 0x26
                        {
                            self.sprite_slot_view_mut(k).and_oam_flags(!1);
                            let value = 63;
                            self.sprite_slot_view_mut(k).set_delay_main(value);
                        } else {
                            let value = 95;
                            self.sprite_slot_view_mut(k).set_delay_main(value);
                        }
                    }
                }
            }
            0x0c => {
                if self.game_state.sprites.dual_layer_tile_cache.tile_type(k) == 0x1c {
                    self.sprite_fall_adjust_position(k);
                    self.sprite_slot_view_mut(k).or_wall_collision(0x20);
                }
            }
            0x68..=0x6b => self
                .sprite_apply_conveyor(k, i32::from(self.game_state.sprites.workspace.tile_type())),
            8 => {
                if self.game_state.dungeon.room_load.header_collision() == 4 {
                    self.sprite_apply_conveyor(k, 0x6a);
                }
            }
            _ => {}
        }
    }

    // uint8 GetTileAttribute(uint8 floor, uint16 *x, uint16 y) {  // 86e87b
    //   uint8 tiletype;
    //   if (player_is_indoors) {
    //     int t = (floor >= 1) ? 0x1000 : 0;
    //     t += (*x & 0x1f8) >> 3;
    //     t += (y & 0x1f8) << 3;
    //     tiletype = dung_bg2_attr_table[t];
    //   } else {
    //     tiletype = Overworld_GetTileAttributeAtLocation(*x >>= 3, y);
    //   }
    //   sprite_tiletype = tiletype;
    //   return tiletype;
    // }
    #[allow(non_snake_case)]
    pub(super) fn GetTileAttribute(&mut self, floor: u8, x: &mut u16, y: u16) -> u8 {
        let tiletype = if self.game_state.world.location.is_indoors() {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((*x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.game_state.dungeon.bg2_attributes.bg2_attr(t)
        } else {
            *x >>= 3;
            self.overworld_get_tile_attribute_at_location(*x, y)
        };
        self.sprite_workspace_mut().set_tile_type(tiletype);
        tiletype
    }

    // uint8 Sprite_GetTileAttribute(int k, uint16 *x, uint16 y) {  // 86e883
    //   return GetTileAttribute(sprite_floor[k], x, y);
    // }
    pub(super) fn sprite_get_tile_attribute(&mut self, k: usize, x: &mut u16, y: u16) -> u8 {
        self.GetTileAttribute(self.sprite_slot_view(k).floor(), x, y)
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
    pub(super) fn sprite_show_solicited_message(&mut self, k: usize, msg: u16) -> u16 {
        self.dialogue_message_index_mut().set_value(msg);
        if !self.sprite_check_damage_to_link_same_layer_for_helpers(k)
            || self.sprite_check_if_link_is_busy_for_helpers()
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
        let dir = self.sprite_direction_to_face_link_for_helpers(k);
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
    pub(super) fn sprite_show_message_on_contact(&mut self, k: usize, msg: u16) -> u16 {
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
        u16::from(self.sprite_direction_to_face_link(k, None)) ^ 0x103
    }

    // bool Sprite_TutorialGuard_ShowMessageOnContact(int k, uint16 msg) {  // 85fa59
    //   ...see sprite.c...
    // }
    pub(super) fn sprite_tutorial_guard_show_message_on_contact(
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
    pub(super) fn sprite_show_message_unconditional(&mut self, msg: u16) {
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
    pub(super) fn sprite_show_message_minimal_c(&mut self) {
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
    pub(super) fn sprite_apply_conveyor(&mut self, k: usize, j: i32) {
        if (self.game_state.frame.frame_counter & 1) == 0 {
            return;
        }
        let idx = (j - 0x68) as usize;
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
    pub(super) fn sprite_bounce_from_tile_collision(&mut self, k: usize) -> u8 {
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

    // int Sprite_SpawnSimpleSparkleGarnishEx(int k, uint16 x, uint16 y, int limit) {
    //   int j = GarnishAllocLimit(limit);
    //   if (j >= 0) {
    //     garnish_type[j] = 5;
    //     garnish_active = 5;
    //     Garnish_SetX(j, Sprite_GetX(k) + x);
    //     Garnish_SetY(j, Sprite_GetY(k) + y - sprite_z[k] + 16);
    //     garnish_countdown[j] = 31;
    //     garnish_sprite[j] = k;
    //     garnish_floor[j] = sprite_floor[k];
    //   }
    //   g_ram[15] = j;
    //   return j;
    // }
    pub(super) fn sprite_spawn_simple_sparkle_garnish_ex(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        limit: i32,
    ) -> i32 {
        let j = self.garnish_alloc_limit(limit as usize);
        if std::env::var_os("ZELDA3_REPLAY_GARNISH_TRACE").is_some() {
            eprintln!(
                "R garnish-spawn fc=0x{:02x} rng=0x{:02x} room=0x{:04x} k={} type=0x{:02x} state=0x{:02x} delay=0x{:02x} xarg=0x{:04x} yarg=0x{:04x} limit={} slot={} sx=0x{:04x} sy=0x{:04x} z=0x{:02x} r12=0x{:04x} r14=0x{:04x}",
                self.game_state.frame.frame_counter,
                self.game_state.world.region.rng_seed(),
                self.game_state.world.location.dungeon_room(),
                k,
                self.sprite_slot_view(k).sprite_type(),
                self.sprite_slot_view(k).state(),
                self.sprite_slot_view(k).delay_main(),
                x,
                y,
                limit,
                j,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.sprite_slot_view(k).z(),
                self.game_state.player.tile_detection.slope_collision_bits(),
                self.game_state.player.tile_detection.collision_bits(),
            );
        }
        if j >= 0 {
            let j = j as usize;
            let value = 5;
            self.garnish_slot_view_mut(j).set_garnish_type(value);
            self.garnish_state_mut().set_active_type(5);
            self.garnish_set_x(j, self.sprite_get_x(k).wrapping_add(x));
            self.garnish_set_y(
                j,
                self.sprite_get_y(k)
                    .wrapping_add(y)
                    .wrapping_sub(self.sprite_slot_view(k).z() as u16)
                    .wrapping_add(16),
            );
            let value = 31;
            self.garnish_slot_view_mut(j).set_countdown(value);
            let value = k as u8;
            self.garnish_slot_view_mut(j).set_sprite(value);
            let value = self.sprite_slot_view(k).floor();
            self.garnish_slot_view_mut(j).set_floor(value);
        }
        self.sprite_workspace_mut().set_last_garnish_index(j);
        // 0x0f (R15 / SPRITE_LAST_GARNISH_INDEX) is the SAME scratch byte as the HIGH byte of
        // collision_bits (R14 u16 @ 0x0e-0x0f). C writes `ram[R15] = j` as raw scratch here;
        // TileDetectionState bulk-projects collision_bits as a u16 every frame, so without this
        // its stale high byte re-stamps over the garnish index at frame end (e.g. 0x0f reverts
        // 0x1d->0x00 at f191098). Keep the native model coherent with the scratch write so the
        // projection writes this garnish index, matching C's last-writer-wins on the shared byte.
        let collision_low = self.game_state.player.tile_detection.collision_bits() as u8;
        self.tile_detect_position_mut()
            .set_collision_bits(u16::from(collision_low) | (u16::from(j as u8) << 8));
        j
    }

    // void Sprite_GarnishSpawn_Sparkle_limited(int k, uint16 x, uint16 y) {  // 9ea001
    //   Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 14);
    // }
    pub(super) fn sprite_garnish_spawn_sparkle_limited(&mut self, k: usize, x: u16, y: u16) {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 14);
    }

    // int Sprite_GarnishSpawn_Sparkle(int k, uint16 x, uint16 y) {  // 9ea007
    //   return Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 29);
    // }
    pub(super) fn sprite_garnish_spawn_sparkle(&mut self, k: usize, x: u16, y: u16) -> i32 {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 29)
    }

    // void Sprite_HaltAllMovement() {  // 9ef508
    //   Sprite_NullifyHookshotDrag();
    //   link_speed_setting = 0;
    //   Link_CancelDash();
    // }
    pub(super) fn sprite_halt_all_movement(&mut self) {
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
    pub(super) fn sprite_behave_as_barrier(&mut self, k: usize) {
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
    pub(super) fn sprite_check_if_screen_is_clear(&self) -> bool {
        for i in (0..=15usize).rev() {
            if self.sprite_slot_view(i).state() != 0
                && (self.sprite_slot_view(i).flags4() & 0x40) == 0
            {
                let x = self
                    .sprite_get_x(i)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
                let y = self
                    .sprite_get_y(i)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
                if x < 256 && y < 256 {
                    return false;
                }
            }
        }
        self.sprite_check_if_overlords_clear()
    }

    // bool Sprite_CheckIfRoomIsClear() {  // 89af61
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && !(sprite_flags4[i] & 0x40))
    //       return false;
    //   }
    //   return Sprite_CheckIfOverlordsClear();
    // }
    pub(super) fn sprite_check_if_room_is_clear(&self) -> bool {
        for i in (0..=15usize).rev() {
            if self.sprite_slot_view(i).state() != 0
                && (self.sprite_slot_view(i).flags4() & 0x40) == 0
            {
                return false;
            }
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
    pub(super) fn sprite_check_if_overlords_clear(&self) -> bool {
        for i in (0..=7usize).rev() {
            if self
                .game_state
                .sprites
                .overlord_slots
                .slot(i)
                .overlord_type()
                == 0x14
                || self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(i)
                    .overlord_type()
                    == 0x18
            {
                return false;
            }
        }
        true
    }

    // void Sprite_ManuallySetDeathFlagUW(int k) {  // 89c2f5
    //   if (!player_is_indoors || sprite_defl_bits[k] & 1 || sign8(sprite_N[k]))
    //     return;
    //   sprite_where_in_room[dungeon_room_index2] |= 1 << sprite_N[k];
    // }
    pub(super) fn sprite_manually_set_death_flag_uw(&mut self, k: usize) {
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
    pub(super) fn sprite_convert_velocity_to_angle(x: u8, y: u8) -> u8 {
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

    fn sprite_prep_load_properties_for_helpers(&mut self, k: usize) {
        self.sprite_prep_load_properties(k);
    }

    fn sprite_check_damage_to_link_same_layer_for_helpers(&mut self, k: usize) -> bool {
        self.sprite_check_damage_to_link_same_layer(k)
    }

    fn sprite_check_if_link_is_busy_for_helpers(&self) -> bool {
        self.sprite_check_if_link_is_busy()
    }

    fn sprite_direction_to_face_link_for_helpers(&mut self, k: usize) -> u8 {
        self.sprite_direction_to_face_link(k, None)
    }
}

#[cfg(test)]
#[path = "sprite_tests.rs"]
mod tests;
