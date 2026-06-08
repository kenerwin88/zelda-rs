// Methods ported from zelda3/src/player.c and included inside ZeldaState.

use super::sprite::SpriteSpawnInfo;
use super::*;
use crate::types::Point16U;

const DUNG_SECRETS_UNK1_PLAYER: usize = 0x0b9c;
const DOOR_ANIMATION_STEP_INDICATOR_PLAYER: usize = 0x0690;
const MEMORIZED_TILE_ADDR_PLAYER: usize = 0x0f800;
const MEMORIZED_TILE_VALUE_PLAYER: usize = 0x0fa00;
const ATTRIBUTES_FOR_TILE_PLAYER: usize = 0x0fe00;
const PUSH_BLOCK_DIRECTION_PLAYER: usize = 0x0474;
const PUSHEDBLOCK_FACING_PLAYER: usize = 0x05f8;
const SAVE_OW_EVENT_INFO_PLAYER: usize = 0x0f280;
const SPRITE_C_PLAYER: usize = 0x0db0;

fn player_memory_location_to_give_item_to(item: u8) -> usize {
    const MEMORY_LOCATIONS: [usize; 76] = [
        0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342,
        0xf340, 0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350,
        0xf35c, 0xf36b, 0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a,
        0xf34d, 0xf35b, 0xf35b, 0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c,
        0xf35c, 0xf35c, 0xf36d, 0xf36e, 0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360,
        0xf374, 0xf374, 0xf374, 0xf340, 0xf340, 0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360,
        0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360, 0xf35c, 0xf359, 0xf34c, 0xf355,
    ];
    MEMORY_LOCATIONS.get(item as usize).copied().unwrap_or(0)
}

fn replay_trace_u16_env(name: &str) -> Option<u16> {
    let value = std::env::var(name).ok()?;
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u16>().ok()
    }
}

impl ZeldaState {
    pub(super) fn replay_trace_player_state(&self, label: &str) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_STATE").is_none() {
            return;
        }
        let room = self.world_state_view().dungeon_room();
        let x = self.player_state_view().x();
        let y = self.player_state_view().y();
        if let Some(frame) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME") {
            if self.ram[FRAME_COUNTER] as u16 != frame {
                return;
            }
        }
        if let Some(frame_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MIN") {
            if (self.ram[FRAME_COUNTER] as u16) < frame_min {
                return;
            }
        }
        if let Some(frame_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_FRAME_MAX") {
            if self.ram[FRAME_COUNTER] as u16 > frame_max {
                return;
            }
        }
        if let Some(expected_room) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_ROOM") {
            if room != expected_room {
                return;
            }
        }
        if let Some(expected_ow) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_OW") {
            if u16::from(self.world_state_view().overworld_screen()) != expected_ow {
                return;
            }
        }
        if let Some(x_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_X_MIN") {
            if x < x_min {
                return;
            }
        }
        if let Some(x_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_X_MAX") {
            if x > x_max {
                return;
            }
        }
        if let Some(y_min) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_Y_MIN") {
            if y < y_min {
                return;
            }
        }
        if let Some(y_max) = replay_trace_u16_env("ZELDA3_REPLAY_TRACE_STATE_Y_MAX") {
            if y > y_max {
                return;
            }
        }
        eprintln!(
            "state-trace frame={} {label} main={} sub={} state=0x{:02x} aux=0x{:02x} incap=0x{:02x} recoil=0x{:02x} scratch_a=0x{:02x} z=0x{:04x} vz=0x{:02x} vzcopy=0x{:02x} x=0x{x:04x} y=0x{y:04x} subpix=0x{:02x}/0x{:02x} vel=0x{:02x}/0x{:02x} dir=0x{:02x} last=0x{:02x} dlast=0x{:02x} pit=0x{:02x} below=0x{:02x} r14=0x{:04x} normal=0x{:04x} drag=0x{:02x} lspeed=0x{:02x}/0x{:02x}",
            self.ram[FRAME_COUNTER],
            self.frame_control_view().main_module(),
            self.frame_control_view().submodule(),
            self.ram[LINK_PLAYER_HANDLER_STATE],
            self.ram[LINK_AUXILIARY_STATE],
            self.ram[LINK_INCAPACITATED_TIMER],
            self.ram[LINK_RECOILMODE_TIMER],
            self.ram[SCRATCH_A],
            self.player_state_view().z(),
            self.ram[LINK_ACTUAL_VEL_Z],
            self.ram[LINK_ACTUAL_VEL_Z_COPY],
            self.ram[LINK_SUBPIXEL_X],
            self.ram[LINK_SUBPIXEL_Y],
            self.ram[LINK_ACTUAL_VEL_X],
            self.ram[LINK_ACTUAL_VEL_Y],
            self.ram[LINK_DIRECTION],
            self.ram[LINK_DIRECTION_LAST],
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS],
            self.ram[TILEDETECT_PIT_TILE],
            self.ram[LINK_TILE_BELOW],
            read_le_u16(&self.ram, R14),
            read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES),
            self.ram[PLAYER_DEFENSE_FLAGS],
            self.ram[LINK_SPEED_SETTING],
            self.ram[LINK_SPEED_MODIFIER],
        );
    }

    pub(super) fn replay_trace_drag_tail(&self, label: &str) {
        if std::env::var_os("ZELDA3_REPLAY_TRACE_SUB_FRAME").is_none() {
            return;
        }
        eprintln!(
            "drag-tail frame={} {label} r14=0x{:04x} tilecoll=0x{:02x} misc=0x{:04x} drag=0x{:02x} timer=0x{:02x} bframes=0x{:02x} lastmove=0x{:02x} face=0x{:02x}",
            self.ram[FRAME_COUNTER],
            read_le_u16(&self.ram, R14),
            self.ram[TILE_COLL_FLAG],
            read_le_u16(&self.ram, TILEDETECT_MISC_TILES),
            self.ram[PLAYER_DEFENSE_FLAGS],
            self.ram[LINK_TIMER_PUSH_GET_TIRED],
            self.ram[BUTTON_B_FRAMES],
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS],
            self.ram[LINK_DIRECTION_FACING],
        );
    }

    pub(super) fn bit_sum4(value: u8) -> u8 {
        (value & 1) + ((value >> 1) & 1) + ((value >> 2) & 1) + ((value >> 3) & 1)
    }

    pub(super) fn dungeon_handle_layer_change(&mut self) {
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] = 1;
        if self.ram[KIND_OF_IN_ROOM_STAIRCASE] == 0 {
            self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_add(16);
        }
        if self.ram[KIND_OF_IN_ROOM_STAIRCASE] != 2 {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        }
        self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        self.SetAndSaveVisitedQuadrantFlags();
    }

    pub(super) fn check_ability_to_swim(&mut self) {
        self.replay_trace_submodule("check_ability_to_swim-entry");
        if self.ram[LINK_IS_BUNNY_MIRROR] == 0 && self.ram[LINK_ITEM_FLIPPERS] != 0 {
            return;
        }
        if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        }
        self.ram[LINK_VISIBILITY_STATUS] = 0x0c;
        let submodule = if self.ram[PLAYER_IS_INDOORS] != 0 {
            20
        } else {
            42
        };
        self.frame_control_view_mut().set_submodule(submodule);
        self.replay_trace_submodule("check_ability_to_swim-exit");
    }

    pub(super) fn link_initialize(&mut self) {
        self.ram[LINK_DIRECTION_FACING] = 2;
        self.ram[LINK_DIRECTION_LAST] = 0;
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_VAR30D] = 0;
        self.ram[LINK_VAR30E] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.link_reset_swimming_state();
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_Z_COORD + 1] = 0;
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.ram[LINK_CAPE_MODE] = 0;
        self.link_force_unequip_cape_quietly();
        self.link_reset_sword_and_item_usage();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
            self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
            self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
            self.ram[LINK_FLAG_MOVING] = 0;
            write_le_u16(&mut self.ram, BG1_Y_OFFSET, 0);
            write_le_u16(&mut self.ram, BG1_X_OFFSET, 0);

            if self.ram[LINK_ITEM_MOON_PEARL] == 0 && self.ram[SAVEGAME_IS_DARKWORLD] != 0 {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 23;
                self.ram[LINK_IS_BUNNY] = 1;
                self.ram[LINK_IS_BUNNY_MIRROR] = 1;
                self.load_gear_palettes_bunny();
            }
        }
    }

    pub(super) fn link_reset_properties_a(&mut self) {
        self.ram[LINK_DIRECTION_LAST] = 0;
        self.ram[LINK_DIRECTION] = 0;
        self.ram[LINK_FLAG_MOVING] = 0;
        self.link_reset_swimming_state();
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.ram[ANCILLA_ARR24] = 0;
        self.ram[LINK_IS_BUNNY] = 0;
        self.ram[LINK_IS_BUNNY_MIRROR] = 0;
        self.ram[LINK_TIMER_TEMPBUNNY] = 0;
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.ram[BIT9_OF_XCOORD] = 0;
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_SPIN_OFFSETS] = 0;
        self.ram[TAGALONG_EVENT_FLAGS] = 0;
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
        self.ram[TILEDETECT_TILE_TYPE] = 0;
        self.ram[ITEM_RECEIPT_METHOD] = 0;
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
        self.link_reset_properties_b();
    }

    pub(super) fn link_reset_properties_b(&mut self) {
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.link_reset_properties_c();
    }

    pub(super) fn link_reset_properties_c(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
            self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0;
        }

        self.ram[TILE_ACTION_INDEX] = 0;
        self.ram[STATE_FOR_SPIN_ATTACK] = 0;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[TILE_COLL_FLAG] = 0;
        self.ram[LINK_FORCE_HOLD_SWORD_UP] = 0;
        self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, 0);
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_VAR30D] = 0;
        self.ram[LINK_VAR30E] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_POSE_FOR_ITEM] = 0;
        self.ram[LINK_CAPE_MODE] = 0;
        self.link_reset_sword_and_item_usage();
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[RELATED_TO_HOOKSHOT] = 0;
        self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 0;
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
    }

    pub(super) fn link_tuck_into_bed(&mut self) {
        self.player_state_view_mut().set_y(0x215a);
        self.player_state_view_mut().set_x(0x0940);
        self.ram[LINK_PLAYER_HANDLER_STATE] = 0x16;
        self.ram[PLAYER_SLEEP_IN_BED_STATE] = 0;
        self.ram[LINK_POSE_DURING_OPENING] = 0;
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 3;
        self.ancilla_add_blanket(0x20);
    }

    pub(super) fn link_reset_swimming_state(&mut self) {
        self.ram[SWIMMING_COUNTDOWN] = 0;
        self.ram[LINK_SWIM_HARD_STROKE] = 0;
        self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
        self.reset_all_acceleration();
    }

    pub(super) fn link_reset_state_after_damaging_pit(&mut self) {
        self.link_reset_swimming_state();
        self.ram[LINK_PLAYER_HANDLER_STATE] =
            if self.ram[LINK_IS_BUNNY] != 0 && self.ram[LINK_ITEM_MOON_PEARL] == 0 {
                23
            } else {
                0
            };
        self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
    }

    pub(super) fn link_state_bunny_recache(&mut self) {
        self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        write_le_u16(&mut self.ram, LINK_TIMER_TEMPBUNNY, 0);
        if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_AUXILIARY_STATE] = 0;
        }
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_IS_TRANSFORMING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.link_reset_swimming_state();
        self.ram[LINK_PLAYER_HANDLER_STATE] = 6;
        if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.load_actual_gear_palettes();
        }
    }

    pub(super) fn link_set_to_deep_water(&mut self) {
        self.ram[LINK_IS_IN_DEEP_WATER] = 1;
        self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
        self.link_reset_swimming_state();
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
    }

    pub(super) fn link_splash_upon_landing(&mut self) {
        if self.ram[LINK_IS_BUNNY_MIRROR] != 0 {
            if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
                self.ancilla_add_splash(21, 0);
                self.link_state_bunny_recache();
                return;
            }
            self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                3
            } else {
                23
            };
        } else if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
            if self.ram[LINK_PLAYER_HANDLER_STATE] != 2 {
                self.ancilla_add_splash(21, 0);
            }
            self.link_force_unequip_cape_quietly();
            self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
        } else {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        }
    }

    pub(super) fn link_handle_swim_accels(&mut self) {
        const SWIMMING_TAB3: [u16; 9] = [128, 160, 192, 224, 256, 288, 320, 352, 384];

        let mut mask = 0x0c;
        for offset in [0, 2] {
            if self.ram[JOYPAD1H_LAST] & mask != 0 {
                let var7 = read_le_u16(&self.ram, SWIM_ACCELERATION + offset);
                let var9 = read_le_u16(&self.ram, SWIM_MAX_SPEED + offset);
                if var7 != 0 && var9 >= 384 {
                    let target = SWIMMING_TAB3
                        .iter()
                        .copied()
                        .find(|value| *value >= var7)
                        .unwrap_or(384);
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, target);
                } else if var9 != 0 {
                    let target = var9.wrapping_add(160).min(384);
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, target);
                } else {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION + offset, 1);
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 240);
                }
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_flag_max_accels(&mut self) {
        if self.ram[LINK_FLAG_MOVING] == 0 {
            return;
        }

        for offset in [2, 0] {
            let var7 = read_le_u16(&self.ram, SWIM_ACCELERATION + offset);
            if var7 != 0 {
                write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, var7);
                write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 1);
            }
        }
    }

    pub(super) fn link_set_ice_max_accel(&mut self) {
        if self.ram[LINK_FLAG_MOVING] == 0 {
            return;
        }
        write_le_u16(&mut self.ram, SWIM_MAX_SPEED, 0x0180);
        write_le_u16(&mut self.ram, SWIM_MAX_SPEED + 2, 0x0180);
    }

    pub(super) fn link_set_momentum(&mut self) {
        const SWIMMING_TAB2: [u8; 2] = [32, 8];

        let joy = self.ram[JOYPAD1H_LAST] & 0x0f;
        let mut mask = 0x0c;
        let mut bit = 0x08;
        for offset in [0, 2] {
            if joy & mask != 0 {
                let var3 = if self.ram[LINK_FLAG_MOVING] != 0 {
                    SWIMMING_TAB2[(self.ram[LINK_FLAG_MOVING] - 1) as usize]
                } else {
                    32
                };
                write_le_u16(
                    &mut self.ram,
                    SWIM_STROKE_FRAME_COUNTER + offset,
                    var3 as u16,
                );

                if (self.ram[SWIM_PLAYER_DIRECTION_FLAGS] | self.ram[LINK_DIRECTION]) & mask == mask
                {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 2);
                } else {
                    let direction = if joy & bit != 0 { 0 } else { 1 };
                    write_le_u16(
                        &mut self.ram,
                        SWIM_ACCELERATION_DIRECTION + offset,
                        direction,
                    );
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 0);
                }

                if read_le_u16(&self.ram, SWIM_MAX_SPEED + offset) == 0 {
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 240);
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub(super) fn player_handler_04_swimming(&mut self) {
        const SWIMMING_TAB1: [u8; 4] = [2, 0, 1, 0];

        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 2;
            self.ram[LINK_Z_COORD + 1] = 0;
            self.reset_all_acceleration();
            self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
            self.ram[LINK_SWIM_HARD_STROKE] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.link_state_recoil();
            return;
        }

        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        if self.ram[LINK_ITEM_FLIPPERS] == 0 {
            return;
        }

        let has_swim_velocity = read_le_u16(&self.ram, SWIM_ACCELERATION)
            | read_le_u16(&self.ram, SWIM_ACCELERATION + 2)
            != 0;
        if !has_swim_velocity {
            if self.ram[SWIM_ACCELERATION_MODE] != 2 && self.ram[SWIM_ACCELERATION_MODE + 2] != 2 {
                self.reset_all_acceleration();
            }
            self.ram[LINK_ANIMATION_STEPS] &= 1;
            self.ram[LINK_FRAME_CHANGE_COUNTER] =
                self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
            if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 16 {
                self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
                self.ram[SWIM_STROKE_ANIM_STEP] = 0;
                self.ram[LINK_ANIMATION_STEPS] = (self.ram[LINK_ANIMATION_STEPS] & 1) ^ 1;
            }
        } else {
            self.ram[LINK_FRAME_CHANGE_COUNTER] =
                self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
            if self.ram[LINK_FRAME_CHANGE_COUNTER] >= 8 {
                self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
                self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1) & 3;
                self.ram[SWIM_STROKE_ANIM_STEP] =
                    SWIMMING_TAB1[self.ram[LINK_ANIMATION_STEPS] as usize];
            }
        }

        if self.ram[LINK_SWIM_HARD_STROKE] == 0 {
            let hard_stroke =
                ((self.ram[FILTERED_JOYPAD_L] & 0x80) | self.ram[FILTERED_JOYPAD_H]) & 0xc0;
            if !has_swim_velocity || hard_stroke == 0 {
                self.link_handle_swim_movements();
                return;
            }
            self.ram[LINK_SWIM_HARD_STROKE] = hard_stroke;
            self.ancilla_sfx2_near(37);
            self.ram[LINK_MAYBE_SWIM_FASTER] = 1;
            self.ram[SWIMMING_COUNTDOWN] = 7;
            self.link_handle_swim_accels();
        }

        self.ram[SWIMMING_COUNTDOWN] = self.ram[SWIMMING_COUNTDOWN].wrapping_sub(1);
        if (self.ram[SWIMMING_COUNTDOWN] as i8).is_negative() {
            self.ram[SWIMMING_COUNTDOWN] = 7;
            self.ram[LINK_MAYBE_SWIM_FASTER] = self.ram[LINK_MAYBE_SWIM_FASTER].wrapping_add(1);
            if self.ram[LINK_MAYBE_SWIM_FASTER] == 5 {
                self.ram[LINK_MAYBE_SWIM_FASTER] = 0;
                self.ram[LINK_SWIM_HARD_STROKE] &= !0xc0;
            }
        }

        self.link_handle_swim_movements();
    }

    fn advance_link_animation_step(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
        if self.ram[LINK_FRAME_CHANGE_COUNTER] >= delay {
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
            if self.ram[LINK_ANIMATION_STEPS] == wrap_at {
                self.ram[LINK_ANIMATION_STEPS] = wrap_to;
            }
        }
    }

    fn advance_link_animation_step_at_least(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        self.ram[LINK_FRAME_CHANGE_COUNTER] = self.ram[LINK_FRAME_CHANGE_COUNTER].wrapping_add(1);
        if self.ram[LINK_FRAME_CHANGE_COUNTER] >= delay {
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
            self.ram[LINK_ANIMATION_STEPS] = self.ram[LINK_ANIMATION_STEPS].wrapping_add(1);
            if self.ram[LINK_ANIMATION_STEPS] >= wrap_at {
                self.ram[LINK_ANIMATION_STEPS] = wrap_to;
            }
        }
    }

    pub(super) fn link_handle_swim_movements(&mut self) {
        let mut direction = (read_le_u16(&self.ram, FORCE_MOVE_ANY_DIRECTION) as u8) & 0x0f;
        if direction == 0 {
            direction = self.ram[JOYPAD1H_LAST] & 0x0f;
        }

        if direction == 0 {
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_X_VEL] = 0;
            self.link_flag_max_accels();
            if self.ram[LINK_FLAG_MOVING] != 0 {
                if self.ram[LINK_IS_RUNNING] != 0 {
                    direction = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                } else {
                    if read_le_u16(&self.ram, SWIM_ACCELERATION)
                        | read_le_u16(&self.ram, SWIM_ACCELERATION + 2)
                        == 0
                    {
                        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                        self.link_reset_swimming_state();
                    }
                    self.finish_swim_movement_tail();
                    return;
                }
            } else {
                if self.ram[LINK_PLAYER_HANDLER_STATE] != 4 {
                    self.ram[LINK_ANIMATION_STEPS] = 0;
                }
                self.finish_swim_movement_tail();
                return;
            }
        }

        if direction != self.ram[SWIM_PLAYER_DIRECTION_FLAGS] {
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = direction;
            self.ram[LINK_SUBPIXEL_X] = 0;
            self.ram[LINK_SUBPIXEL_Y] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        }
        self.link_set_ice_max_accel();
        self.link_set_momentum();
        self.link_set_the_max_accel();
        self.finish_swim_movement_tail();
    }

    fn finish_swim_movement_tail(&mut self) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_set_the_max_accel(&mut self) {
        if self.ram[LINK_FLAG_MOVING] != 0 || self.ram[LINK_SWIM_HARD_STROKE] != 0 {
            return;
        }

        let mut mask = 0x0c;
        for offset in [0, 2] {
            let var5 = read_le_u16(&self.ram, SWIM_ACCELERATION_MODE + offset);
            if self.ram[JOYPAD1H_LAST] & mask != 0 && var5 != 2 {
                let var1 = read_le_u16(&self.ram, SWIM_SPEED_ACTIVE_FLAG + offset);
                let var7 = read_le_u16(&self.ram, SWIM_ACCELERATION + offset);
                let var9 = read_le_u16(&self.ram, SWIM_MAX_SPEED + offset);
                if var1 != 0 || (var7 >= 240 && var7 >= var9) {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 0);
                    if var7 >= 240 {
                        write_le_u16(&mut self.ram, SWIM_SPEED_ACTIVE_FLAG + offset, 1);
                        write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 1);
                    } else {
                        write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 240);
                        write_le_u16(&mut self.ram, SWIM_SPEED_ACTIVE_FLAG + offset, 0);
                    }
                }
            } else {
                write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 240);
                write_le_u16(&mut self.ram, SWIM_SPEED_ACTIVE_FLAG + offset, 0);
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_handle_toss(&mut self) -> bool {
        if self.ram[Y_BUTTON_ACTION_FLAGS] & 0x80 == 0
            || self.ram[FILTERED_JOYPAD_L] & 0x80 == 0
            || self.ram[LINK_PICKING_THROW_STATE] & 1 != 0
        {
            return false;
        }

        self.ram[LINK_VAR30D] = 0;
        self.ram[LINK_VAR30E] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        true
    }

    pub(super) fn link_cancel_dash(&mut self) {
        if self.ram[LINK_IS_RUNNING] == 0 {
            return;
        }
        for i in (0..=4).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x1e {
                self.ram[ANCILLA_TYPE + i] = 0;
            }
        }
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_IS_RUNNING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE, 0);
    }

    pub(super) fn repel_dash(&mut self) {
        if self.ram[LINK_IS_RUNNING] != 0 && self.ram[LINK_DASH_CTR] != 64 {
            self.link_reset_swimming_state();
            self.ancilla_add_dash_tremor(29, 1);
            self.prepare_apply_rumble_to_sprites();
            if self.ram[SOUND_EFFECT_2] & 0x3f != 27 && self.ram[SOUND_EFFECT_2] & 0x3f != 50 {
                self.ancilla_sfx3_near(3);
            }
            self.link_apply_tile_rebound();
        }
    }

    pub(super) fn sprite_repel_dash(&mut self) {
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = self.ram[LINK_DIRECTION_FACING] >> 1;
        self.repel_dash();
    }

    pub(super) fn link_apply_tile_rebound(&mut self) {
        const DASH_TAB_6Y: [u8; 4] = [24, (-24i8) as u8, 0, 0];
        const DASH_TAB_6X: [u8; 4] = [0, 0, 24, (-24i8) as u8];
        const DASH_TAB_SW11Y: [u8; 4] = [1, 0, 0, 0];
        const DASH_TAB_SW11X: [u8; 4] = [0, 0, 1, 0];
        const DASH_TAB_SW7Y: [u16; 8] = [384, 384, 0, 0, 256, 256, 0, 0];
        const DASH_TAB_SW7X: [u16; 8] = [0, 0, 384, 384, 0, 0, 256, 256];
        const DASH_TAB_DIR: [u8; 4] = [8, 4, 2, 1];

        let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
        self.ram[LINK_ACTUAL_VEL_Y] = DASH_TAB_6Y[dir];
        self.ram[LINK_ACTUAL_VEL_X] = DASH_TAB_6X[dir];
        self.ram[LINK_INCAPACITATED_TIMER] = 24;
        self.ram[LINK_ACTUAL_VEL_Z] = 36;
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = 36;
        if self.ram[LINK_FLAG_MOVING] != 0 {
            self.ram[LINK_DIRECTION] = DASH_TAB_DIR[dir];
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION];
            write_le_u16(
                &mut self.ram,
                SWIM_ACCELERATION_DIRECTION,
                DASH_TAB_SW11Y[dir] as u16,
            );
            write_le_u16(
                &mut self.ram,
                SWIM_ACCELERATION_DIRECTION + 2,
                DASH_TAB_SW11X[dir] as u16,
            );
            let i = (self.ram[LINK_FLAG_MOVING] - 1) as usize * 4 + dir;
            write_le_u16(&mut self.ram, SWIM_ACCELERATION, DASH_TAB_SW7Y[i]);
            write_le_u16(&mut self.ram, SWIM_ACCELERATION + 2, DASH_TAB_SW7X[i]);
        }
        self.ram[LINK_AUXILIARY_STATE] = 1;
        self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 1;
        self.ram[SCRATCH_1] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 != 0 {
            self.ram[LINK_Y_VEL] = 0;
        } else {
            self.ram[LINK_X_VEL] = 0;
        }
    }

    pub(super) fn link_handle_moving_animation_full_long_entry(&mut self) {
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 4 {
            self.link_handle_moving_animation_swimming();
            return;
        }

        const TAB: [u8; 4] = [8, 4, 2, 1];
        let mut r0 = self.ram[LINK_DIRECTION_LAST];
        if r0 == 0 {
            return;
        }
        if self.ram[LINK_FLAG_MOVING] != 0 {
            r0 = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
        }
        if self.ram[LINK_CANT_CHANGE_DIRECTION] == 0 {
            let mut y;
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            } else if self.ram[IS_STANDING_IN_DOORWAY] != 0 {
                y = self.ram[IS_STANDING_IN_DOORWAY].wrapping_mul(2) & !3;
            } else if r0 & TAB[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize] != 0 {
                self.link_handle_moving_animation_start_with_dash();
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
            if y != 4 {
                y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
            } else {
                y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
            }
            self.ram[LINK_DIRECTION_FACING] = y;
        }
        self.link_handle_moving_animation_start_with_dash();
    }

    pub(super) fn link_handle_moving_animation_start_with_dash(&mut self) {
        if self.ram[LINK_IS_RUNNING] != 0 {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut x = self.ram[LINK_DIRECTION_FACING] >> 1;
        if self.ram[LINK_SPEED_SETTING] == 6 {
            x = x.wrapping_add(4);
        } else if self.ram[LINK_FLAG_MOVING] != 0 {
            if self.ram[JOYPAD1H_LAST] & 0x0f == 0 {
                self.ram[LINK_ANIMATION_STEPS] = 0;
                return;
            }
            x = x.wrapping_add(4);
        }

        const TAB2: [u8; 16] = [4, 4, 4, 4, 1, 1, 1, 1, 2, 2, 2, 2, 8, 8, 8, 8];
        const TAB3: [u8; 24] = [
            1, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1, 1, 2, 1, 2, 2, 3, 2, 2, 2, 3, 2,
        ];
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 23
            || (self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0
                && self.ram[LINK_PLAYER_HANDLER_STATE] == 28)
        {
            if self.ram[LINK_ANIMATION_STEPS] < 4 && self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 2 {
                self.advance_link_animation_step(TAB2[x as usize], 4, 0);
            } else {
                self.ram[LINK_ANIMATION_STEPS] = 0;
            }
            return;
        }

        if self.frame_control_view().submodule() == 18
            || self.frame_control_view().submodule() == 19
        {
            x = 12;
        } else if self.frame_control_view().submodule() != 14
            && self.ram[LINK_STATE_BITS] & 0x80 == 0
        {
            if self.ram[PLAYER_DEFENSE_FLAGS] & 0x8d != 0 {
                x = 12;
            } else if self.ram[DRAW_WATER_RIPPLES_OR_GRASS] == 0 && self.ram[BUTTON_B_FRAMES] == 0 {
                let mut idx = self.ram[LINK_ANIMATION_STEPS];
                if self.ram[LINK_SPEED_SETTING] == 6 {
                    idx = idx.wrapping_add(8);
                }
                if self.ram[LINK_FLAG_MOVING] != 0 {
                    idx = idx.wrapping_add(8);
                }
                if self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 2 {
                    self.advance_link_animation_step(TAB3[idx as usize], 9, 1);
                }
                return;
            }
        }

        if self.ram[LINK_ANIMATION_STEPS] < 6 && self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 2 {
            self.advance_link_animation_step(TAB2[x as usize], 6, 0);
        } else {
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }
    }

    pub(super) fn link_handle_moving_animation_swimming(&mut self) {
        const TAB: [u8; 4] = [8, 4, 2, 1];
        let r0 = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
        if r0 == 0 || self.ram[LINK_CANT_CHANGE_DIRECTION] != 0 {
            return;
        }
        let mut y;
        if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] != 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 {
                y = self.ram[IS_STANDING_IN_DOORWAY].wrapping_mul(2) & !3;
            } else if r0 & TAB[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize] != 0 {
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
        } else {
            y = if r0 & 0x0c != 0 { 0 } else { 4 };
        }
        if y != 4 {
            y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
        } else {
            y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
        }
        self.ram[LINK_DIRECTION_FACING] = y;
    }

    pub(super) fn link_handle_moving_animation_dash(&mut self) {
        const DASH_TAB3: [u8; 7] = [48, 36, 24, 16, 12, 8, 4];
        const DASH_TAB4: [u8; 56] = [
            3, 3, 5, 3, 3, 3, 5, 3, 2, 2, 4, 2, 2, 2, 4, 2, 2, 2, 3, 2, 2, 2, 3, 2, 1, 1, 2, 1, 1,
            1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        const DASH_TAB5: [u8; 7] = [1, 2, 2, 2, 2, 2, 2];
        let mut t = 6usize;
        while self.ram[LINK_COUNTDOWN_FOR_DASH] >= DASH_TAB3[t] && t != 0 {
            t -= 1;
        }
        if self.ram[BUTTON_B_FRAMES] < 9 && self.ram[DRAW_WATER_RIPPLES_OR_GRASS] == 0 {
            self.advance_link_animation_step(DASH_TAB4[t * 8], 9, 1);
        } else {
            self.advance_link_animation_step_at_least(DASH_TAB5[t], 6, 0);
        }
    }

    pub(super) fn link_apply_moving_floor_velocity(&mut self) {
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL));
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(read_le_u16(&self.ram, DUNG_FLOOR_X_VEL));
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().set_x(x);
    }

    pub(super) fn link_apply_conveyor(&mut self) {
        const MOVE_POS_DIR_FLAG: [u8; 4] = [8, 4, 2, 1];
        const MOVING_BELT_Y: [i8; 4] = [-8, 8, 0, 0];
        const MOVING_BELT_X: [i8; 4] = [0, 0, -8, 8];
        if self.ram[LINK_ON_CONVEYOR_BELT] == 0 {
            return;
        }
        let z = self.ram[LINK_Z_COORD];
        if z != 0 && z != 0xff {
            return;
        }
        if self.ram[LINK_GRABBING_WALL] & 1 != 0
            || self.ram[LINK_PLAYER_HANDLER_STATE] == 19
            || self.ram[LINK_AUXILIARY_STATE] != 0
        {
            return;
        }
        let j = self.ram[LINK_ON_CONVEYOR_BELT].wrapping_sub(1) as usize;
        if j >= MOVE_POS_DIR_FLAG.len() {
            return;
        }
        if self.ram[LINK_IS_RUNNING] != 0
            && self.ram[LINK_DASH_CTR] == 32
            && self.ram[LINK_DIRECTION] & MOVE_POS_DIR_FLAG[j] != 0
        {
            return;
        }
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        self.ram[LINK_DIRECTION] |= MOVE_POS_DIR_FLAG[j];
        let y = ((self.player_state_view().y() as u32) << 8
            | self.ram[BG1_MOVE_CALC_BUFFER] as u32)
            .wrapping_add(((MOVING_BELT_Y[j] as i32) << 4) as u32);
        self.ram[BG1_MOVE_CALC_BUFFER] = y as u8;
        self.player_state_view_mut().set_y((y >> 8) as u16);
        let x = ((self.player_state_view().x() as u32) << 8
            | self.ram[BG1_MOVE_CALC_BUFFER + 1] as u32)
            .wrapping_add(((MOVING_BELT_X[j] as i32) << 4) as u32);
        self.ram[BG1_MOVE_CALC_BUFFER + 1] = x as u8;
        self.player_state_view_mut().set_x((x >> 8) as u16);
    }

    pub(super) fn flag67_with_directions(&mut self) {
        self.ram[LINK_DIRECTION] = 0;
        if self.ram[LINK_ACTUAL_VEL_Y] != 0 {
            self.ram[LINK_DIRECTION] |= if (self.ram[LINK_ACTUAL_VEL_Y] as i8).is_negative() {
                8
            } else {
                4
            };
        }
        if self.ram[LINK_ACTUAL_VEL_X] != 0 {
            self.ram[LINK_DIRECTION] |= if (self.ram[LINK_ACTUAL_VEL_X] as i8).is_negative() {
                2
            } else {
                1
            };
        }
    }

    pub(super) fn link_add_in_velocity_y_falling(&mut self) {
        let adjust = i16::from(self.ram[TILEDETECT_WHICH_Y_POS] & 7)
            - if (self.ram[LINK_Y_VEL] as i8).is_negative() {
                8
            } else {
                0
            };
        let y = self.player_state_view().y().wrapping_sub(adjust as u16);
        self.player_state_view_mut().set_y(y);
    }

    pub(super) fn link_add_in_velocity_y(&mut self) {
        let y = self
            .player_state_view()
            .y()
            .wrapping_sub(self.ram[LINK_Y_VEL] as i8 as i16 as u16);
        self.player_state_view_mut().set_y(y);
    }

    pub(super) fn player_change_z(&mut self, z_delta: u8) {
        if (self.ram[LINK_ACTUAL_VEL_Z] as i8).is_negative() {
            if self.ram[LINK_Z_COORD] == 0 {
                return;
            }
            if (self.ram[LINK_Z_COORD] as i8).is_negative() {
                self.player_state_view_mut().set_z(0xffff);
                self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
                return;
            }
        }
        self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z].wrapping_sub(z_delta);
    }

    pub(super) fn link_move_position(&mut self) {
        let x = self.player_state_view().x();
        let y = self.player_state_view().y();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = x as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (x >> 8) as u8;

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 10 && self.ram[PLAYER_ON_SOMARIA_PLATFORM] == 2 {
            self.link_handle_velocity_and_sand_drag(x, y);
            return;
        }

        self.move_link_coord(LINK_SUBPIXEL_X, LINK_X_COORD, self.ram[LINK_ACTUAL_VEL_X]);
        self.move_link_coord(LINK_SUBPIXEL_Y, LINK_Y_COORD, self.ram[LINK_ACTUAL_VEL_Y]);
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.move_link_coord(LINK_SUBPIXEL_Z, LINK_Z_COORD, self.ram[LINK_ACTUAL_VEL_Z]);
        }

        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(x, y);
    }

    pub(super) fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(read_le_u16(&self.ram, DRAG_PLAYER_Y));
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(read_le_u16(&self.ram, DRAG_PLAYER_X));
        self.player_state_view_mut().set_y(y);
        self.player_state_view_mut().set_x(x);
        self.ram[LINK_Y_VEL] = y.wrapping_sub(old_y) as u8;
        self.ram[LINK_X_VEL] = x.wrapping_sub(old_x) as u8;
    }

    pub(super) fn link_handle_moving_floor(&mut self) {
        if self.ram[DUNG_HDR_COLLISION] == 0 {
            return;
        }
        let z = self.ram[LINK_Z_COORD];
        if z != 0 && z != 0xff {
            return;
        }
        if !self.has_player_layer_collision(crate::ram::player::LAYER_COLLISION_BOTH) {
            return;
        }
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 19 {
            return;
        }

        let floor_y = read_le_u16(&self.ram, DUNG_FLOOR_Y_VEL);
        if floor_y != 0 {
            self.ram[LINK_DIRECTION] |= if (floor_y as i16).is_negative() { 8 } else { 4 };
        }
        let floor_x = read_le_u16(&self.ram, DUNG_FLOOR_X_VEL);
        if floor_x != 0 {
            self.ram[LINK_DIRECTION] |= if (floor_x as i16).is_negative() { 2 } else { 1 };
        }

        self.link_apply_moving_floor_velocity();
    }

    pub(super) fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        if self.ram[DUNG_HDR_COLLISION] == 0 || self.ram[DUNG_HDR_COLLISION] == 4 {
            return false;
        }

        if self.ram[DUNG_HDR_COLLISION] >= 2 {
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(read_le_u16(&self.ram, BG1VOFS_COPY2))
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
            self.player_state_view_mut().set_y(y);
            write_le_u16(&mut self.ram, RELATED_TO_MOVING_FLOOR_Y, y);

            let x = self
                .player_state_view()
                .x()
                .wrapping_add(read_le_u16(&self.ram, BG1HOFS_COPY2))
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            self.player_state_view_mut().set_x(x);
            write_le_u16(&mut self.ram, RELATED_TO_MOVING_FLOOR_X, x);
        }
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        true
    }

    pub(super) fn create_velocity_from_moving_background(&mut self) {
        if self.ram[DUNG_HDR_COLLISION] != 1 {
            let x = self
                .player_state_view()
                .x()
                .wrapping_sub(read_le_u16(&self.ram, RELATED_TO_MOVING_FLOOR_X));
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, RELATED_TO_MOVING_FLOOR_Y));
            let new_y = self
                .player_state_view()
                .y()
                .wrapping_add(read_le_u16(&self.ram, BG2VOFS_COPY2))
                .wrapping_sub(read_le_u16(&self.ram, BG1VOFS_COPY2));
            let new_x = self
                .player_state_view()
                .x()
                .wrapping_add(read_le_u16(&self.ram, BG2HOFS_COPY2))
                .wrapping_sub(read_le_u16(&self.ram, BG1HOFS_COPY2));
            self.player_state_view_mut().set_y(new_y);
            self.player_state_view_mut().set_x(new_x);
            if self.ram[LINK_DIRECTION] != 0 {
                self.ram[LINK_X_VEL] = self.ram[LINK_X_VEL].wrapping_add(x as u8);
                self.ram[LINK_Y_VEL] = self.ram[LINK_Y_VEL].wrapping_add(y as u8);
            }
        }
        self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
    }

    pub(super) fn calculate_snap_scratch_y(&mut self) {
        let mut y_vel = self.ram[LINK_Y_VEL] as i8;
        if read_le_u16(&self.ram, R14) & 4 != 0 {
            if y_vel >= 0 {
                y_vel = y_vel.wrapping_neg();
            }
        } else if y_vel < 0 {
            y_vel = y_vel.wrapping_neg();
        }

        let x = self.player_state_view().x();
        let delta = if y_vel < 0 { -1i16 } else { 1i16 };
        self.player_state_view_mut()
            .set_x(x.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 2;
        let r14 = read_le_u16(&self.ram, R14);
        let t = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if t & 7 == 0 {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
            return;
        }

        let mut t = self.ram[LINK_Y_VEL];
        let dir = if self.ram[LINK_X_COORD] >= 0x80 {
            if t & 0x80 == 0 {
                t = t.wrapping_neg();
            }
            4
        } else {
            if t & 0x80 != 0 {
                t = t.wrapping_neg();
            }
            6
        };
        let vel = if t & 0x80 != 0 { -1i16 } else { 1i16 };
        if self.ram[LINK_CANT_CHANGE_DIRECTION] & 1 == 0 {
            self.ram[LINK_DIRECTION_FACING] = dir;
        }
        let x = self.player_state_view().x();
        self.player_state_view_mut()
            .set_x(x.wrapping_add(vel as u16));
    }

    pub(super) fn snap_on_x(&mut self) {
        let x = self.player_state_view().x();
        let adjust = (x & 7).wrapping_sub(if (self.ram[LINK_X_VEL] as i8).is_negative() {
            8
        } else {
            0
        });
        self.player_state_view_mut().set_x(x.wrapping_sub(adjust));
    }

    pub(super) fn calculate_snap_scratch_x(&mut self) {
        let mut x_vel = self.ram[LINK_X_VEL] as i8;
        if read_le_u16(&self.ram, R14) & 4 != 0 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
        } else if x_vel < 0 {
            x_vel = x_vel.wrapping_neg();
        }

        let y = self.player_state_view().y();
        let delta = if x_vel < 0 { -1i16 } else { 1i16 };
        self.player_state_view_mut()
            .set_y(y.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 2;
        let r14 = read_le_u16(&self.ram, R14);
        let r0 = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if r0 & 7 == 0 {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
            return r0 as i8;
        }

        let mut x_vel = self.ram[LINK_X_VEL] as i8;
        let dir = if self.ram[LINK_Y_COORD] >= 0x80 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
            0
        } else {
            if x_vel < 0 {
                x_vel = x_vel.wrapping_neg();
            }
            2
        };
        if self.ram[LINK_CANT_CHANGE_DIRECTION] & 1 == 0 {
            self.ram[LINK_DIRECTION_FACING] = dir;
        }
        let y = self.player_state_view().y();
        write_le_u16(
            &mut self.ram,
            LINK_Y_COORD,
            y.wrapping_add(x_vel as i16 as u16),
        );
        x_vel
    }

    pub(super) fn tile_behavior_handle_item_and_execute(&mut self, x: u16, y: u16) {
        let tile = self.handle_item_tile_action_overworld(x, y);
        self.tile_detect_execute_inner(tile, 0, 1, false);
    }

    pub(super) fn push_block_get_target_tile_flag(&self, x: u16, y: u16) -> u8 {
        let offset = ((y & !7) as usize) * 8
            + (x & 0x3f) as usize
            + if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                0x1000
            } else {
                0
            };
        self.ram[DUNG_BG2_ATTR_TABLE + offset]
    }

    pub(super) fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(if self.ram[LINK_PLAYER_HANDLER_STATE] == 19 {
            1
        } else {
            2
        });
    }

    pub(super) fn run_slope_collision_checks_vertical_first(&mut self) {
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x20 == 0 {
            self.start_movement_collision_checks_y();
        }
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x10 == 0 {
            self.start_movement_collision_checks_x();
        }
    }

    pub(super) fn run_slope_collision_checks_horizontal_first(&mut self) {
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x10 == 0 {
            self.start_movement_collision_checks_x();
        }
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x20 == 0 {
            self.start_movement_collision_checks_y();
        }
    }

    pub(super) fn link_hop_in_or_out_of_water_y(&mut self) {
        const RECOIL_VEL_Y: [u8; 3] = [24, 16, 16];
        const RECOIL_VEL_Z: [u8; 3] = [36, 24, 24];
        let ts = if self.ram[PLAYER_IS_INDOORS] == 0 {
            2
        } else if self.ram[ABOUT_TO_JUMP_OFF_LEDGE] != 0 {
            0
        } else {
            self.ram[TS_COPY]
        };

        let mut vel = RECOIL_VEL_Y[ts as usize];
        if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] == 0 {
            vel = 0u8.wrapping_sub(vel);
        }

        self.ram[LINK_ACTUAL_VEL_Y] = vel;
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Z] = RECOIL_VEL_Z[ts as usize];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = self.ram[LINK_ACTUAL_VEL_Z];
        self.player_state_view_mut().set_z(0);
        self.ram[LINK_INCAPACITATED_TIMER] = 16;
        if self.ram[LINK_AUXILIARY_STATE] != 2 {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        }
        self.ram[LINK_PLAYER_HANDLER_STATE] = 6;
    }

    pub(super) fn link_hop_in_or_out_of_water_x(&mut self) {
        const RECOIL_VEL_X: [u8; 3] = [28, 24, 16];
        const RECOIL_VEL_Z: [u8; 3] = [32, 24, 24];
        let ts = if self.ram[PLAYER_IS_INDOORS] == 0 {
            2
        } else if self.ram[ABOUT_TO_JUMP_OFF_LEDGE] != 0 {
            0
        } else {
            self.ram[TS_COPY]
        };

        let mut vel = RECOIL_VEL_X[ts as usize];
        if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 == 0 {
            vel = 0u8.wrapping_sub(vel);
        }
        self.ram[LINK_ACTUAL_VEL_X] = vel;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_ACTUAL_VEL_Z] = RECOIL_VEL_Z[ts as usize];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = self.ram[LINK_ACTUAL_VEL_Z];
        self.ram[LINK_INCAPACITATED_TIMER] = 16;
        if self.ram[LINK_AUXILIARY_STATE] != 2 {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        }
        self.ram[LINK_PLAYER_HANDLER_STATE] = 6;
    }

    pub(super) fn run_ledge_hop_timer(&mut self) -> bool {
        let mut rv = false;
        if self.ram[LINK_AUXILIARY_STATE] != 1 {
            if self.ram[LINK_IS_RUNNING] == 0 {
                self.ram[LINK_TIMER_JUMP_LEDGE] = self.ram[LINK_TIMER_JUMP_LEDGE].wrapping_sub(1);
                if (self.ram[LINK_TIMER_JUMP_LEDGE] as i8).is_negative() {
                    self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
                    return true;
                }
            } else {
                rv = true;
            }
        }
        copy_le_u16(&mut self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
        copy_le_u16(&mut self.ram, LINK_X_COORD, LINK_X_COORD_PREV);
        self.ram[LINK_SUBPIXEL_Y] = 0;
        self.ram[LINK_SUBPIXEL_X] = 0;
        rv
    }

    pub(super) fn flag_moving_into_slopes_y(&mut self) {
        const AVOID_JUDDER: [i8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4,
            5, 6, 7,
        ];
        let diag_state = read_le_u16(&self.ram, TILEDETECT_DIAG_STATE) as usize;
        let x =
            self.player_state_view()
                .x()
                .wrapping_sub(if read_le_u16(&self.ram, R12) & 4 != 0 {
                    1
                } else {
                    0
                });
        let o = diag_state * 4 + (x & 7) as usize;
        let mut y = (self.ram[TILEDETECT_WHICH_Y_POS] & 7) as i8;

        if read_le_u16(&self.ram, TILEDETECT_DIAGONAL_TILE) & 5 != 0 {
            let mut ym = (self.ram[TILEDETECT_WHICH_Y_POS] & 7) as i8;
            if self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0 {
                if diag_state & 2 != 0 {
                    ym = ym.wrapping_neg();
                } else {
                    ym = AVOID_JUDDER[o].wrapping_sub(8 - ym);
                }
            } else {
                if diag_state & 2 == 0 {
                    ym = 8 - ym;
                } else {
                    ym += 8;
                }
                ym = AVOID_JUDDER[o].wrapping_sub(ym);
            }

            if self.ram[LINK_Y_VEL] == 0 {
                return;
            }
            if (self.ram[LINK_Y_VEL] as i8).is_negative() {
                ym = ym.wrapping_neg();
            }
            y = ym;
        } else {
            y = AVOID_JUDDER[o].wrapping_sub(y);
        }

        if (self.ram[LINK_Y_VEL] as i8).is_negative() {
            if y <= 0 {
                return;
            }
            let coord = self.player_state_view().y().wrapping_add(y as i16 as u16);
            self.player_state_view_mut().set_y(coord);
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 8;
        } else {
            if y >= 0 {
                return;
            }
            let coord = self.player_state_view().y().wrapping_add(y as i16 as u16);
            self.player_state_view_mut().set_y(coord);
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 4;
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] |= if read_le_u16(&self.ram, R12) & 4 != 0 {
            0x12
        } else {
            0x11
        };
    }

    pub(super) fn flag_moving_into_slopes_x(&mut self) {
        const AVOID_JUDDER: [i8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4,
            5, 6, 7,
        ];
        let diag_state = read_le_u16(&self.ram, TILEDETECT_DIAG_STATE);
        let mut x = (self
            .player_state_view()
            .x()
            .wrapping_sub(if diag_state == 6 { 1 } else { 0 })
            & 7) as i8;
        let which_y_offset = if read_le_u16(&self.ram, R12) & 4 != 0 {
            2
        } else {
            0
        };
        let o = diag_state as usize * 4
            + (self.ram[TILEDETECT_WHICH_Y_POS + which_y_offset] & 7) as usize;

        if read_le_u16(&self.ram, TILEDETECT_DIAGONAL_TILE) & 5 != 0 {
            let mut xm = (self.player_state_view().x() & 7) as i8;
            if diag_state != 4 && diag_state != 6 {
                xm = xm.wrapping_neg();
            } else {
                xm = AVOID_JUDDER[o].wrapping_sub(8 - xm);
            }
            if self.ram[LINK_X_VEL] == 0 {
                return;
            }
            if (self.ram[LINK_X_VEL] as i8).is_negative() {
                xm = xm.wrapping_neg();
            }
            x = xm;
        } else {
            x = AVOID_JUDDER[o].wrapping_sub(x);
        }

        if (self.ram[LINK_X_VEL] as i8).is_negative() {
            if x <= 0 {
                return;
            }
            let coord = self.player_state_view().x().wrapping_add(x as i16 as u16);
            self.player_state_view_mut().set_x(coord);
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 2;
        } else {
            if x >= 0 {
                return;
            }
            let coord = self.player_state_view().x().wrapping_add(x as i16 as u16);
            self.player_state_view_mut().set_x(coord);
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 1;
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] |= if diag_state & 2 != 0 { 0x28 } else { 0x24 };
    }

    pub(super) fn player_something_with_velocity_tired_or_swim(&mut self, xvel: u16, yvel: u16) {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = old_y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (old_y >> 8) as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_LO] = old_x as u8;
        self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (old_x >> 8) as u8;

        self.move_link_coord_subpixel_delta(LINK_SUBPIXEL_X, LINK_X_COORD, xvel);
        let u = (xvel >> 8) as u8;
        self.ram[LINK_ACTUAL_VEL_X] = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((xvel as u8) >> 4);

        self.move_link_coord_subpixel_delta(LINK_SUBPIXEL_Y, LINK_Y_COORD, yvel);
        let u = (yvel >> 8) as u8;
        self.ram[LINK_ACTUAL_VEL_Y] = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((yvel as u8) >> 4);

        if self.ram[DUNG_HDR_COLLISION] == 4 {
            self.link_apply_moving_floor_velocity();
        }
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.link_handle_velocity_and_sand_drag(old_x, old_y);
    }

    pub(super) fn link_check_for_edge_screen_transition(&mut self) -> bool {
        if matches!(self.ram[LINK_PLAYER_HANDLER_STATE], 3 | 8 | 9 | 10)
            || self.ram[LINK_INCAPACITATED_TIMER] == 0
        {
            return false;
        }
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_RECOILMODE_TIMER] = 3;
        copy_le_u16(&mut self.ram, LINK_X_COORD, LINK_X_COORD_PREV);
        copy_le_u16(&mut self.ram, LINK_Y_COORD, LINK_Y_COORD_PREV);
        true
    }

    pub(super) fn player_limit_directions_inner(&mut self) {
        self.ram[LINK_DIRECTION_MASK_A] = 0x0f;
        self.ram[LINK_DIRECTION_MASK_B] = 0x0f;
        self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;

        const MASKS: [u8; 4] = [0x07, 0x0b, 0x0d, 0x0e];

        if self.ram[LINK_DIRECTION] & 0x0c != 0 {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] =
                self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS].wrapping_add(1);
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = if self.ram[LINK_DIRECTION] & 8 != 0 {
                0
            } else {
                1
            };
            self.tile_detect_movement_vertical_slopes(
                self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16,
            );

            let r14 = read_le_u16(&self.ram, R14);
            if r14 & 0x30 != 0
                && self.ram[TILEDETECT_DOOR_DIRECTION_FLAGS] & 2 == 0
                && ((((r14 & 0x30) >> 4) as u8) & self.ram[LINK_DIRECTION]) == 0
                && self.ram[LINK_DIRECTION] & 3 != 0
            {
                self.ram[LINK_DIRECTION_MASK_A] = MASKS[if self.ram[LINK_DIRECTION] & 2 != 0 {
                    2
                } else {
                    3
                }];
            } else {
                let mut set_thingy = false;
                if self.ram[DUNG_HDR_COLLISION] == 0
                    && self.ram[LINK_AUXILIARY_STATE] != 0
                    && read_le_u16(&self.ram, R12) & 3 != 0
                {
                    set_thingy = true;
                }

                if r14 & 3 != 0 {
                    self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                    if self.ram[LINK_FLAG_MOVING] != 0
                        && self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 3 == 0
                        && self.ram[LINK_DIRECTION] & 3 != 0
                    {
                        write_le_u16(&mut self.ram, SWIM_SPEED_ACTIVE_FLAG, 0);
                        write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE, 0);
                        write_le_u16(&mut self.ram, SWIM_ACCELERATION, 0);
                        write_le_u16(&mut self.ram, SWIM_MAX_SPEED, 0);
                    }
                    set_thingy = true;
                }

                if set_thingy {
                    self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
                    self.ram[LINK_DIRECTION_MASK_A] =
                        MASKS[self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize];
                }
            }
        }

        if self.ram[LINK_DIRECTION] & 0x0c != 0 && self.ram[LINK_DIRECTION] & 3 != 0 {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] =
                self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS].wrapping_add(1);
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = if self.ram[LINK_DIRECTION] & 2 != 0 {
                2
            } else {
                3
            };
            self.tile_detect_movement_horizontal_slopes(
                self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16,
            );

            let r14 = read_le_u16(&self.ram, R14);
            if r14 & 0x30 != 0
                && self.ram[TILEDETECT_DOOR_DIRECTION_FLAGS] & 2 != 0
                && ((((r14 & 0x30) >> 2) as u8) & self.ram[LINK_DIRECTION]) == 0
                && self.ram[LINK_DIRECTION] & 0x0c != 0
            {
                self.ram[LINK_DIRECTION_MASK_B] = MASKS[if self.ram[LINK_DIRECTION] & 8 != 0 {
                    0
                } else {
                    1
                }];
            } else {
                let mut set_thingy_b = false;
                if self.ram[DUNG_HDR_COLLISION] == 0
                    && self.ram[LINK_AUXILIARY_STATE] != 0
                    && read_le_u16(&self.ram, R12) & 3 != 0
                {
                    set_thingy_b = true;
                }

                if r14 & 3 != 0 {
                    self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                    if self.ram[LINK_FLAG_MOVING] != 0
                        && self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 3 == 0
                        && self.ram[LINK_DIRECTION] & 0x0c != 0
                    {
                        write_le_u16(&mut self.ram, SWIM_SPEED_ACTIVE_FLAG + 2, 0);
                        write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + 2, 0);
                        write_le_u16(&mut self.ram, SWIM_ACCELERATION + 2, 0);
                        write_le_u16(&mut self.ram, SWIM_MAX_SPEED + 2, 0);
                    }
                    set_thingy_b = true;
                }

                if set_thingy_b {
                    self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
                    self.ram[LINK_DIRECTION_MASK_B] =
                        MASKS[self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize];
                }
            }

            self.ram[LINK_DIRECTION] &=
                self.ram[LINK_DIRECTION_MASK_A] & self.ram[LINK_DIRECTION_MASK_B];
        }

        if self.ram[LINK_DIRECTION] & 0x0f != 0
            && self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0
        {
            self.ram[LINK_DIRECTION] = self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f;
        }

        if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 2 {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = if self.ram[LINK_DIRECTION_FACING] & 4 != 0 {
                2
            } else {
                1
            };
        } else {
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
        }
    }

    pub(super) fn link_handle_velocity(&mut self) {
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();

        if (self.frame_control_view().submodule() == 2
            && self.frame_control_view().main_module() == 14)
            || self.ram[LINK_PREVENT_FROM_MOVING] != 0
        {
            self.store_link_safe_return_position(old_x, old_y);
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return;
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] == 4 {
            self.handle_swim_stroke_and_subpixels();
            return;
        }

        let mut speed_index = if self.ram[LINK_FLAG_MOVING] != 0 {
            if self.ram[LINK_IS_RUNNING] == 0 {
                self.handle_swim_stroke_and_subpixels();
                return;
            }
            24
        } else {
            if self.ram[LINK_IS_RUNNING] != 0 {
                self.ram[LINK_SPEED_MODIFIER] = 0;
                assert!(self.ram[LINK_DASH_CTR] >= 32);
            }
            if (self.ram[TILE_COLLISION_BITS_PRIMARY] | self.ram[TILE_COLLISION_BITS_SECONDARY])
                == 0x0f
            {
                return;
            }
            if self.ram[DRAW_WATER_RIPPLES_OR_GRASS] != 0 {
                match self.ram[LINK_SPEED_SETTING] {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            } else {
                self.ram[LINK_SPEED_SETTING]
            }
        };

        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;

        if (self.ram[LINK_DIRECTION] & 0x0c) != 0 && (self.ram[LINK_DIRECTION] & 0x03) != 0 {
            speed_index = speed_index.wrapping_add(1);
        }

        if self.ram[PLAYER_NEAR_PIT_STATE] != 0 {
            if self.ram[PLAYER_NEAR_PIT_STATE] == 3 {
                self.ram[LINK_SPEED_MODIFIER] = if self.ram[LINK_SPEED_MODIFIER] < 48 {
                    self.ram[LINK_SPEED_MODIFIER].wrapping_add(8)
                } else {
                    32
                };
            }
        } else if self.ram[LINK_SPEED_MODIFIER] != 0 {
            speed_index = if self.frame_control_view().submodule() == 8
                || self.frame_control_view().submodule() == 16
            {
                10
            } else {
                2
            };
            if self.ram[LINK_SPEED_MODIFIER] != 1 && self.ram[LINK_SPEED_MODIFIER] < 16 {
                self.ram[LINK_SPEED_MODIFIER] = self.ram[LINK_SPEED_MODIFIER].wrapping_add(1);
                speed_index = 26;
            } else if self.ram[LINK_SPEED_MODIFIER] != 1 {
                self.ram[LINK_SPEED_MODIFIER] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
            }
        }

        const SPEED_MOD: [u8; 27] = [
            24, 16, 10, 24, 16, 8, 8, 4, 12, 16, 9, 25, 20, 13, 16, 8, 64, 42, 16, 8, 4, 2, 48, 24,
            32, 21, 0,
        ];
        let vel = self.ram[LINK_SPEED_MODIFIER].wrapping_add(SPEED_MOD[speed_index as usize]);
        if self.ram[LINK_DIRECTION] & 0x03 != 0 {
            self.ram[LINK_ACTUAL_VEL_X] = if self.ram[LINK_DIRECTION] & 0x02 != 0 {
                0u8.wrapping_sub(vel)
            } else {
                vel
            };
        }
        if self.ram[LINK_DIRECTION] & 0x0c != 0 {
            self.ram[LINK_ACTUAL_VEL_Y] = if self.ram[LINK_DIRECTION] & 0x08 != 0 {
                0u8.wrapping_sub(vel)
            } else {
                vel
            };
        }

        self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
        self.player_state_view_mut().set_z(0xffff);
        self.ram[LINK_SUBPIXEL_Z] = 0;
        self.link_move_position();
    }

    pub(super) fn handle_swim_stroke_and_subpixels(&mut self) {
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;

        const SWIMMING_TAB4: [i8; 12] = [8, -12, -8, -16, 4, -6, -12, -6, 10, -16, -12, -6];
        const SWIMMING_TAB5: [u8; 2] = [!0x0c, !0x03];
        const SWIMMING_TAB6: [u8; 4] = [8, 4, 2, 1];
        let mut stroke = [0u16; 2];

        for i in (0..=1).rev() {
            let offset = i * 2;
            let var3 = read_le_u16(&self.ram, SWIM_STROKE_FRAME_COUNTER + offset).wrapping_sub(1);
            write_le_u16(&mut self.ram, SWIM_STROKE_FRAME_COUNTER + offset, var3);
            if (var3 as i16) < 0 {
                write_le_u16(&mut self.ram, SWIM_STROKE_FRAME_COUNTER + offset, 0);
                write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 1);
            }

            let mut table_index = read_le_u16(&self.ram, SWIM_ACCELERATION_MODE + offset);
            if self.ram[LINK_FLAG_MOVING] != 0 {
                table_index = table_index.wrapping_add(u16::from(self.ram[LINK_FLAG_MOVING]) * 4);
            }

            let delta = SWIMMING_TAB4[table_index as usize] as i16 as u16;
            let mut sum = read_le_u16(&self.ram, SWIM_ACCELERATION + offset).wrapping_add(delta);
            if (sum as i16) <= 0 {
                self.ram[LINK_DIRECTION] &= SWIMMING_TAB5[i];
                self.ram[LINK_DIRECTION_LAST] = self.ram[LINK_DIRECTION];
                if read_le_u16(&self.ram, SWIM_ACCELERATION_MODE + offset) == 2 {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 0);
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 240);
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION + offset, 2);
                } else {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION_MODE + offset, 0);
                    write_le_u16(&mut self.ram, SWIM_MAX_SPEED + offset, 0);
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION + offset, 0);
                }
            } else {
                let dir_index =
                    read_le_u16(&self.ram, SWIM_ACCELERATION_DIRECTION + offset) as usize + i * 2;
                self.ram[LINK_DIRECTION] |= SWIMMING_TAB6[dir_index];
                let max_sum = read_le_u16(&self.ram, SWIM_MAX_SPEED + offset);
                if sum >= max_sum {
                    sum = max_sum;
                }
                write_le_u16(&mut self.ram, SWIM_ACCELERATION + offset, sum);
            }

            stroke[i] = read_le_u16(&self.ram, SWIM_ACCELERATION + offset);
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] | self.ram[LINK_MOVING_AGAINST_DIAG_TILE]
                != 0
            {
                stroke[i] = stroke[i].wrapping_sub(stroke[i] >> 2);
            }
            if read_le_u16(&self.ram, SWIM_ACCELERATION_DIRECTION + offset) == 0 {
                stroke[i] = 0u16.wrapping_sub(stroke[i]);
            }
        }

        self.player_something_with_velocity_tired_or_swim(stroke[1], stroke[0]);
    }

    pub(super) fn link_receive_item(&mut self, item: u8, chest_position: u16) {
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[COUNTDOWN_FOR_BLINK] = 0;
            self.ram[LINK_STATE_BITS] = 0;
        }
        self.ram[LINK_RECEIVEITEM_INDEX] = item;
        if item == 0x3e {
            self.ancilla_sfx3_near(0x2e);
        }
        self.ram[LINK_ITEM_HOLDING_TIMER] = 0x60;
        if self.ram[ITEM_RECEIPT_METHOD] == 0 || self.ram[ITEM_RECEIPT_METHOD] == 3 {
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 21;
            self.ram[LINK_POSE_FOR_ITEM] = 1;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            if item == 0x20 {
                self.ram[LINK_POSE_FOR_ITEM] = 2;
            }
        }
        self.ancilla_add_item_receipt(0x22, 4, chest_position);
        if item != 0x20 && item != 0x37 && item != 0x38 && item != 0x39 {
            self.hud_refresh_icon();
        }
        self.link_cancel_dash();
    }

    pub(super) fn handle_nudging(&mut self, arg_r0: i8) {
        const TAB0: [u8; 8] = [8, 8, 23, 23, 8, 23, 8, 23];
        const TAB1: [u8; 8] = [0, 15, 0, 15, 0, 0, 15, 15];
        const TAB2: [u8; 8] = [23, 23, 8, 8, 8, 23, 8, 23];
        const TAB3: [u8; 8] = [0, 15, 0, 15, 15, 15, 0, 0];

        let p = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 == 0 {
            if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 != 0 {
                4
            } else {
                0
            }
        } else if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 != 0 {
            12
        } else {
            8
        };
        let o = (((if read_le_u16(&self.ram, R14) & 4 != 0 {
            0
        } else {
            2
        }) + p)
            >> 1) as usize;

        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();

        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let y0 = link_y.wrapping_add(TAB0[o] as u16) & mask;
        let x0 = (link_x.wrapping_add(TAB1[o] as u16) & mask) >> 3;
        let y1 = link_y.wrapping_add(TAB2[o] as u16) & mask;
        let x1 = (link_x.wrapping_add(TAB3[o] as u16) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let blocked = (read_le_u16(&self.ram, R14)
            | self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] as u16)
            & 3
            != 0
            || (self.ram[TILEDETECT_VERTICAL_LEDGE] | self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES])
                & 0x33
                != 0;
        if blocked {
            if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 != 0 {
                let y = self.player_state_view().y();
                write_le_u16(
                    &mut self.ram,
                    LINK_Y_COORD,
                    y.wrapping_sub(arg_r0 as i16 as u16),
                );
            } else {
                let x = self.player_state_view().x();
                write_le_u16(
                    &mut self.ram,
                    LINK_X_COORD,
                    x.wrapping_sub(arg_r0 as i16 as u16),
                );
            }
        }
    }

    pub(super) fn handle_pushing_bonking_snaps_y(&mut self) {
        let r14 = read_le_u16(&self.ram, R14);
        if r14 & 7 == 0 {
            if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                return;
            }
            self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        let mut used_swim_axis_reprobe = false;
        if self.ram[LINK_PLAYER_HANDLER_STATE] == 4 {
            if self.ram[DUNG_FLOOR_Y_VEL] == 0 {
                self.reset_all_acceleration();
            }
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] != 0 {
                self.link_add_in_velocity_y_falling();
                used_swim_axis_reprobe = true;
            }
        }

        if r14 & 2 != 0 || (r14 & 5) == 5 {
            self.replay_trace_drag_tail("snaps-y-before-first-bonk");
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            write_le_u16(&mut self.ram, R14, bak);
            self.replay_trace_drag_tail("snaps-y-after-first-bonk");
        }

        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;

        if !used_swim_axis_reprobe {
            if r14 & 2 == 2 {
                self.replay_trace_drag_tail("snaps-y-before-add-vel");
                self.link_add_in_velocity_y_falling();
                self.replay_trace_drag_tail("snaps-y-after-add-vel");
            } else {
                if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 1 {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
                self.link_add_in_velocity_y_falling();
                if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 2 {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
            }
        }

        if (r14 & 5) == 5 {
            self.replay_trace_drag_tail("snaps-y-before-second-bonk");
            self.link_bonk_and_smash();
            self.repel_dash();
            self.replay_trace_drag_tail("snaps-y-after-second-bonk");
        } else if r14 & 2 == 0 {
            let y_vel = self.ram[LINK_Y_VEL];
            let tt = if r14 & 4 != 0 {
                if (y_vel as i8).is_negative() {
                    y_vel
                } else {
                    0u8.wrapping_sub(y_vel)
                }
            } else if (y_vel as i8).is_negative() {
                0u8.wrapping_sub(y_vel)
            } else {
                y_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.player_state_view().x() & 7 != 0 {
                let x = self.player_state_view().x();
                self.player_state_view_mut()
                    .set_x(x.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    pub(super) fn handle_pushing_bonking_snaps_x(&mut self) {
        let r14 = read_le_u16(&self.ram, R14);
        if r14 & 7 == 0 {
            if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                return;
            }
            self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] == 4 && self.ram[DUNG_FLOOR_X_VEL] == 0 {
            self.reset_all_acceleration();
        }

        if r14 & 2 != 0 {
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            write_le_u16(&mut self.ram, R14, bak);
        }

        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;

        if r14 & 7 == 7 {
            self.snap_on_x();
        } else {
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 2 {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
            self.snap_on_x();
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 1 {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
        }

        if (r14 & 5) == 5 {
            self.link_bonk_and_smash();
            self.repel_dash();
        } else if r14 & 2 == 0 {
            let x_vel = self.ram[LINK_X_VEL];
            let tt = if r14 & 4 != 0 {
                if (x_vel as i8).is_negative() {
                    x_vel
                } else {
                    0u8.wrapping_sub(x_vel)
                }
            } else if (x_vel as i8).is_negative() {
                0u8.wrapping_sub(x_vel)
            } else {
                x_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.player_state_view().y() & 7 != 0 {
                let y = self.player_state_view().y();
                self.player_state_view_mut()
                    .set_y(y.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    fn handle_pushing_bonking_dragstate_tail(&mut self) -> bool {
        if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS].wrapping_mul(2)
            != self.ram[LINK_DIRECTION_FACING]
        {
            return false;
        }

        self.replay_trace_drag_tail("tail-match-entry");
        self.ram[PLAYER_DEFENSE_FLAGS] |= (self.ram[TILE_COLL_FLAG] & 1) << 1;
        self.replay_trace_drag_tail("tail-after-lowbit");
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = self.ram[LINK_TIMER_PUSH_GET_TIRED].wrapping_sub(1);
        if self.ram[BUTTON_B_FRAMES] == 0
            && !(self.ram[LINK_TIMER_PUSH_GET_TIRED] as i8).is_negative()
        {
            self.replay_trace_drag_tail("tail-return-timer");
            return true;
        }

        let tile_coll = self.ram[TILE_COLL_FLAG];
        let drag_bits = if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 0x20 != 0 {
            tile_coll << 3
        } else {
            tile_coll
        };
        self.ram[PLAYER_DEFENSE_FLAGS] |= drag_bits;
        self.replay_trace_drag_tail("tail-after-fullbits");
        false
    }

    fn handle_pushing_bonking_snaps_return(&mut self) {
        self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
        self.ram[PLAYER_DEFENSE_FLAGS] &= !2;
    }

    pub(super) fn push_block_attempt_to_push_the_block(&self, what: u8, x: u16, y: u16) -> bool {
        const Y0: [i8; 4] = [-4, 20, 4, 4];
        const Y1: [i8; 4] = [-4, 20, 12, 12];
        const X0: [i8; 4] = [4, 4, -4, 20];
        const X1: [i8; 4] = [12, 12, -4, 20];

        let idx = what as usize * 4 + self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);

        let x0 = (x.wrapping_add(X0[idx] as i16 as u16) & mask) >> 3;
        let y0 = y.wrapping_add(Y0[idx] as i16 as u16) & mask;
        let xt = self.push_block_get_target_tile_flag(x0, y0);
        if push_block_target_is_blocked(xt) {
            return true;
        }

        let x1 = (x.wrapping_add(X1[idx] as i16 as u16) & mask) >> 3;
        let y1 = y.wrapping_add(Y1[idx] as i16 as u16) & mask;
        push_block_target_is_blocked(self.push_block_get_target_tile_flag(x1, y1))
    }

    pub(super) fn link_find_valid_landing_tile_north(&mut self) {
        const DY: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];
        const DZ: [u8; 32] = [
            24, 24, 24, 24, 28, 28, 28, 28, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];
        const TIMER: [u8; 32] = [
            16, 16, 20, 20, 24, 24, 28, 28, 32, 32, 36, 36, 40, 40, 44, 44, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ];

        let y_coord_bak = self.player_state_view().y();
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, y_coord_bak);
        loop {
            let y = self.player_state_view().y().wrapping_sub(16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);
            let terrain = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES)
                | read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
                | read_le_u16(&self.ram, TILEDETECT_THICK_GRASS)
                | read_le_u16(&self.ram, TILEDETECT_DEEPWATER);
            if terrain & 7 == 7 {
                break;
            }
        }

        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 7 != 0 {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.link_reset_swimming_state();
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
        }

        let y = self.player_state_view().y().wrapping_sub(16);
        self.player_state_view_mut().set_y(y);
        let diff = read_le_u16(&self.ram, LINK_Y_COORD_ORIGINAL).wrapping_sub(y);
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, diff);
        self.player_state_view_mut().set_y(y_coord_bak);
        let o = ((diff as u8) >> 3) as usize;
        let dy = DY[o];
        self.ram[LINK_ACTUAL_VEL_Y] = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] != 0 {
            dy
        } else {
            0u8.wrapping_sub(dy)
        };
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Z] = DZ[o];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = DZ[o];
        self.player_state_view_mut().set_z(0);
        self.ram[LINK_INCAPACITATED_TIMER] = TIMER[o];
        self.ram[LINK_AUXILIARY_STATE] = 2;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 6;
    }

    pub(super) fn link_find_valid_landing_tile_diagonal_north(&mut self) {
        const DX: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 16, 16, 24, 24, 24, 24, 16, 16, 16, 16, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DY: [u8; 32] = [
            8, 8, 8, 8, 16, 16, 20, 20, 24, 24, 24, 24, 32, 32, 32, 32, 8, 20, 20, 20, 24, 24, 24,
            24, 28, 28, 28, 28, 32, 32, 32, 32,
        ];
        const DZ: [u8; 32] = [
            32, 32, 32, 32, 32, 32, 32, 32, 36, 36, 36, 36, 40, 40, 40, 40, 32, 40, 40, 40, 44, 44,
            44, 44, 48, 48, 48, 48, 52, 52, 52, 52,
        ];

        let y_safe = self.ram[LINK_Y_COORD_SAFE_RETURN_LO];
        let x_bak = self.player_state_view().x();
        let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS];

        self.ram[LINK_ACTUAL_VEL_X] = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] != 2 {
            1
        } else {
            0xff
        };
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = 0;
        self.link_hop_find_landing_spot_diagonally_down();

        self.player_state_view_mut().set_x(x_bak);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y_safe;

        let diff = read_le_u16(&self.ram, LINK_Y_COORD_ORIGINAL)
            .wrapping_sub(self.player_state_view().y());
        let o = (diff >> 3) as usize;
        copy_le_u16(&mut self.ram, LINK_Y_COORD, LINK_Y_COORD_ORIGINAL);

        self.ram[LINK_ACTUAL_VEL_Y] = 0u8.wrapping_sub(DY[o]);
        self.ram[LINK_ACTUAL_VEL_X] = if dir != 2 {
            DX[o]
        } else {
            0u8.wrapping_sub(DX[o])
        };
        self.ram[LINK_ACTUAL_VEL_Z] = DZ[o];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = DZ[o];
        self.player_state_view_mut().set_z(0);
        let z_mirror = self.read_u16_ram(LINK_Z_COORD_MIRROR) & !0x00ff;
        self.write_u16_ram(LINK_Z_COORD_MIRROR, z_mirror);
        self.ram[LINK_AUXILIARY_STATE] = 2;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 13;
    }

    pub(super) fn tile_detect_main_handler(&mut self, item: u8) {
        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();

        let probe_base = if item == 8 {
            let spin = self.ram[STATE_FOR_SPIN_ATTACK].wrapping_sub(2);
            if spin >= 8 {
                return;
            }
            const SPIN_OFFSETS: [u8; 8] = [10, 6, 14, 2, 12, 4, 8, 0];
            SPIN_OFFSETS[spin as usize] as u16 + 0x40
        } else {
            item as u16 * 8 + self.ram[LINK_DIRECTION_FACING] as u16
        };
        let offset = probe_base >> 1;
        const X: [i8; 40] = [
            8, 8, 8, 8, 6, 8, -1, 22, 19, 19, 0, 19, 6, 8, -1, 22, 8, 8, 8, 8, 8, 8, 0, 15, 6, 8,
            -10, 29, 6, 8, -6, 22, 6, 8, -4, 22, -4, 22, -4, 22,
        ];
        const Y: [i8; 40] = [
            20, 20, 20, 20, 4, 28, 16, 16, 22, 22, 22, 22, 4, 24, 16, 16, 16, 16, 16, 16, 20, 20,
            23, 23, -4, 36, 16, 16, 4, 28, 16, 16, 4, 28, 16, 16, 4, 4, 28, 28,
        ];
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        let x = (link_x.wrapping_add(X[offset as usize] as i16 as u16) & mask) >> 3;
        let y = link_y.wrapping_add(Y[offset as usize] as i16 as u16) & mask;

        if matches!(item, 1 | 2 | 3 | 6 | 7 | 8) {
            self.tile_behavior_handle_item_and_execute(x, y);
            return;
        }

        self.tile_detection_execute(x, y, 1);
        if item == 5 {
            return;
        }

        if read_le_u16(&self.ram, TILEDETECT_THICK_GRASS) & 0x10 != 0 {
            let tx = self.player_state_view().x() & 0x0f;
            let ty = self.player_state_view().y().wrapping_add(8) & 0x0f;
            if !(4..11).contains(&ty)
                && !(4..12).contains(&tx)
                && self.ram[COUNTDOWN_FOR_BLINK] == 0
                && self.ram[LINK_AUXILIARY_STATE] == 0
            {
                if self.ram[PLAYER_IS_INDOORS] != 0 {
                    self.Dungeon_FlagRoomData_Quadrants();
                    self.ancilla_sfx2_near(0x33);
                    self.ram[LINK_SPEED_SETTING] = 0;
                    self.frame_control_view_mut().set_submodule(21);
                    self.ram[DUNGEON_ROOM_INDEX_PREV] = self.ram[DUNGEON_ROOM_INDEX];
                    self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNG_HDR_TRAVEL_DESTINATIONS];
                    self.handle_layer_of_destination();
                } else if self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] == 0 {
                    self.do_sword_interaction_with_tiles_mirror();
                }
            }
        } else {
            self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 0;
            if read_le_u16(&self.ram, TILEDETECT_THICK_GRASS) & 1 != 0 {
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 2;
                if !self.link_permission_for_slosh_sounds() && self.ram[LINK_AUXILIARY_STATE] == 0 {
                    self.ancilla_sfx2_near(26);
                }
                return;
            }

            if read_le_u16(&self.ram, TILEDETECT_SHALLOW_WATER) & 1 != 0 {
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 1;
                if self.ram[PLAYER_IS_INDOORS] == 0
                    && self.ram[LINK_IS_IN_DEEP_WATER] != 0
                    && self.ram[LINK_IS_BUNNY_MIRROR] == 0
                {
                    if self.ram[LINK_ITEM_FLIPPERS] != 0 {
                        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
                        self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                        self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
                    }
                } else if !self.link_permission_for_slosh_sounds() {
                    if self.ram[OVERWORLD_SCREEN_INDEX] == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if self.ram[LINK_AUXILIARY_STATE] == 0 {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }

            if self.ram[PLAYER_IS_INDOORS] == 0
                && self.ram[LINK_IS_IN_DEEP_WATER] == 0
                && read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 1 != 0
            {
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 1;
                if !self.link_permission_for_slosh_sounds() {
                    if self.ram[OVERWORLD_SCREEN_INDEX] == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if self.ram[LINK_AUXILIARY_STATE] == 0 {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }
        }

        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        if self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] & 1 != 0 {
            self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = 1;
            return;
        }
        self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = 0;

        if self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] & 0x10 != 0 {
            self.ram[LINK_GIVE_DAMAGE] = 0;
            if self.ram[LINK_CAPE_MODE] == 0
                && !self.search_for_byrna_spark()
                && self.ram[COUNTDOWN_FOR_BLINK] == 0
            {
                self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
                write_le_u16(&mut self.ram, LINK_TIMER_TEMPBUNNY, 0);
                if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                    self.ram[LINK_IS_BUNNY] = 0;
                    self.ram[LINK_IS_BUNNY_MIRROR] = 0;
                }
                self.ram[LINK_GIVE_DAMAGE] = 8;
                self.link_cancel_dash();
                return;
            }
        }

        if read_le_u16(&self.ram, TILEDETECT_ICY_FLOOR) & 0x11 != 0 {
            if self.ram[LINK_FLAG_MOVING] != 0 {
                if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] != 0 {
                    self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                }
            } else {
                if self.ram[LINK_DIRECTION] & 0x0c != 0 {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION, 0x0180);
                }
                if self.ram[LINK_DIRECTION] & 3 != 0 {
                    write_le_u16(&mut self.ram, SWIM_ACCELERATION, 0x0180);
                }
                self.ram[LINK_FLAG_MOVING] =
                    if read_le_u16(&self.ram, TILEDETECT_ICY_FLOOR) & 1 != 0 {
                        1
                    } else {
                        2
                    };
                self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
                self.link_reset_swimming_state();
            }
        } else {
            if self.ram[LINK_PLAYER_HANDLER_STATE] != 4 {
                if self.ram[LINK_FLAG_MOVING] != 0 {
                    self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                }
                self.link_reset_swimming_state();
            }
            self.ram[LINK_FLAG_MOVING] = 0;
        }

        if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 0x10 != 0 && self.ram[COUNTDOWN_FOR_BLINK] == 0 {
            self.ram[COUNTDOWN_FOR_BLINK] = 58;
        }
    }

    pub(super) fn start_movement_collision_checks_y(&mut self) {
        self.replay_trace_submodule("start-y-entry");
        if self.ram[LINK_Y_VEL] == 0 {
            return;
        }
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = if self.ram[IS_STANDING_IN_DOORWAY] == 1 {
            if self.ram[LINK_Y_COORD] < 0x80 {
                0
            } else {
                1
            }
        } else if (self.ram[LINK_Y_VEL] as i8).is_negative() {
            0
        } else {
            1
        };
        self.tile_detect_movement_y(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);
        self.replay_trace_submodule("start-y-after-tiledetect");
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.start_movement_collision_checks_y_handle_indoors();
        } else {
            self.start_movement_collision_checks_y_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_x(&mut self) {
        if self.ram[LINK_X_VEL] == 0 {
            return;
        }
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = if self.ram[IS_STANDING_IN_DOORWAY] == 2 {
            if self.ram[LINK_X_COORD] < 0x80 {
                2
            } else {
                3
            }
        } else if (self.ram[LINK_X_VEL] as i8).is_negative() {
            2
        } else {
            3
        };
        self.tile_detect_movement_x(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.start_movement_collision_checks_x_handle_indoors();
        } else {
            self.start_movement_collision_checks_x_handle_outdoors();
        }
    }

    pub(super) fn start_movement_collision_checks_y_handle_indoors(&mut self) {
        let mut r14 = read_le_u16(&self.ram, R14);
        if (self.ram[LINK_STATE_BITS] as i8).is_negative()
            || self.ram[LINK_INCAPACITATED_TIMER] != 0
        {
            r14 |= r14 >> 4;
            write_le_u16(&mut self.ram, R14, r14);
        } else {
            if self.ram[IS_STANDING_IN_DOORWAY] == 2 {
                if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
                    if self.ram[DUNG_HDR_COLLISION] != 3 || self.ram[LINK_IS_ON_LOWER_LEVEL] == 0 {
                        self.link_add_in_velocity_y();
                        self.change_axis_of_perpendicular_door_movement_y();
                        return;
                    }
                } else if read_le_u16(&self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS) != 0 {
                    self.link_add_in_velocity_y();
                    self.finish_indoor_y_collision();
                    return;
                }
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if (self.ram[LINK_Y_VEL] as i8).is_negative() {
                        8
                    } else {
                        4
                    };
                    write_le_u16(&mut self.ram, FORCE_MOVE_ANY_DIRECTION, force_move);
                }

                self.ram[IS_STANDING_IN_DOORWAY] = 1;
                self.ram[LINK_ON_CONVEYOR_BELT] = 0;
                if r14 & 0x70 != 0x70 {
                    if r14 & 5 != 0 {
                        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                        self.link_add_in_velocity_y_falling();
                        self.calculate_snap_scratch_y();
                        self.ram[IS_STANDING_IN_DOORWAY] = 0;
                        if r14 & 0x20 != 0 && r14 & 1 == 0 && self.player_state_view().x() & 7 == 1
                        {
                            let x = self.player_state_view().x() & !7;
                            self.player_state_view_mut().set_x(x);
                        }
                        if self.ram[TILE_COLL_FLAG] & 2 == 0 {
                            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
                        }
                        return;
                    }
                    if r14 & 0x20 != 0 {
                        if self.ram[TILE_COLL_FLAG] & 2 == 0 {
                            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
                        }
                        return;
                    }
                } else {
                    if self.ram[TILE_COLL_FLAG] & 2 == 0 {
                        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
                    }
                    return;
                }
            }
        }

        if self.ram[TILE_COLL_FLAG] & 2 == 0 {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
        }
        self.finish_indoor_y_collision();
    }

    pub(super) fn start_movement_collision_checks_x_handle_indoors(&mut self) {
        let mut r14 = read_le_u16(&self.ram, R14);
        if (self.ram[LINK_STATE_BITS] as i8).is_negative()
            || self.ram[LINK_INCAPACITATED_TIMER] != 0
        {
            r14 |= r14 >> 4;
            write_le_u16(&mut self.ram, R14, r14);
        } else {
            if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
                self.ram[LINK_SPEED_MODIFIER] = 0;
            }

            if self.ram[IS_STANDING_IN_DOORWAY] == 1
                && self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0
                && (self.ram[DUNG_HDR_COLLISION] != 3 || self.ram[LINK_IS_ON_LOWER_LEVEL] == 0)
            {
                self.snap_on_x();
                let spd = self.change_axis_of_perpendicular_door_movement_x();
                self.handle_nudging_in_a_door(spd);
                return;
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if (self.ram[LINK_X_VEL] as i8).is_negative() {
                        2
                    } else {
                        1
                    };
                    write_le_u16(&mut self.ram, FORCE_MOVE_ANY_DIRECTION, force_move);
                }

                self.ram[IS_STANDING_IN_DOORWAY] = 2;
                self.ram[LINK_ON_CONVEYOR_BELT] = 0;
                if r14 & 0x70 != 0x70 {
                    if r14 & 7 != 0 {
                        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                        self.ram[IS_STANDING_IN_DOORWAY] = 0;
                        self.snap_on_x();
                        self.calculate_snap_scratch_x();
                        return;
                    }
                    self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
                    return;
                }

                self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
                return;
            }
        }

        if self.ram[TILE_COLL_FLAG] & 2 == 0 {
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
            self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
            write_le_u16(&mut self.ram, FORCE_MOVE_ANY_DIRECTION, 0);
        }

        if read_le_u16(&self.ram, R14) & 2 == 0 && read_le_u16(&self.ram, R12) & 5 != 0 {
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
            self.flag_moving_into_slopes_x();
            if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0 {
                return;
            }
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.finish_indoor_collision_common(false);
    }

    pub(super) fn start_movement_collision_checks_y_handle_outdoors(&mut self) {
        self.replay_trace_submodule("outdoor-y-entry");
        self.replay_trace_drag_tail("outdoor-y-drag-entry");
        if self.ram[LINK_SPEED_SETTING] == 2 {
            self.ram[LINK_SPEED_SETTING] = if self.ram[LINK_IS_RUNNING] != 0 {
                16
            } else {
                0
            };
        }
        self.replay_trace_drag_tail("outdoor-y-after-speed-setting");

        if self.ram[TILEDETECT_PIT_TILE] & 5 != 0 && read_le_u16(&self.ram, R14) & 2 == 0 {
            self.start_falling_into_hole();
            return;
        }

        self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1] =
            if read_le_u16(&self.ram, TILEDETECT_READ_SOMETHING) & 2 != 0 {
                self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] >> 1
            } else {
                0
            };

        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 2 != 0
            && self.ram[LINK_IS_IN_DEEP_WATER] == 0
            && self.ram[LINK_AUXILIARY_STATE] == 0
        {
            self.link_reset_sword_and_item_usage();
            self.link_cancel_dash();
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.link_reset_swimming_state();
            if self.ram[DRAW_WATER_RIPPLES_OR_GRASS] == 1 && {
                self.link_force_unequip_cape_quietly();
                self.ram[LINK_ITEM_FLIPPERS] != 0
            } {
                if self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                }
            } else {
                self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
                self.restore_link_safe_return_position();
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.link_hop_in_or_out_of_water_y();
            }
        }

        if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
            if self.ram[TILEDETECT_VERTICAL_LEDGE] & 7 != 0 {
                let r14 = (self.ram[TILEDETECT_VERTICAL_LEDGE] & 7) as u16;
                write_le_u16(&mut self.ram, R14, r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            if (self.ram[TILEDETECT_STAIR_TILE] & 7) == 7
                || read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) & 7 == 7
            {
                self.link_cancel_dash();
                self.ram[LINK_IS_IN_DEEP_WATER] = 0;
                if self.ram[LINK_AUXILIARY_STATE] == 0 {
                    self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                    self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                    self.ancilla_add_splash(0x15, 0);
                    self.link_hop_in_or_out_of_water_y();
                    return;
                }
            }
        }

        if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 2 != 0
            || self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] & 0x22 != 0
        {
            write_le_u16(&mut self.ram, R14, 7);
            self.handle_pushing_bonking_snaps_y();
            return;
        }

        if self.ram[TILEDETECT_VERTICAL_LEDGE] & 0x70 != 0 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[ALLOW_SCROLL_Z] = 1;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 11;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            write_le_u16(&mut self.ram, LINK_Z_COORD_MIRROR, 0xffff);
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            let zvel = if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
                14
            } else {
                20
            };
            self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = zvel;
            self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = zvel;
            self.ram[LINK_AUXILIARY_STATE] = if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
                4
            } else {
                2
            };
            return;
        }

        if self.ram[TILEDETECT_VERTICAL_LEDGE] & 7 != 0 && self.run_ledge_hop_timer() {
            self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.link_cancel_dash();
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.link_find_valid_landing_tile_north();
            return;
        }

        if self.ram[LINK_IS_IN_DEEP_WATER] == 0 {
            if self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] & 7 != 0
                && self.ram[TILEDETECT_VERTICAL_LEDGE] & 0x77 == 0
            {
                let xand = if read_le_u16(&self.ram, INDEX_OF_INTERACTING_TILE) == 0x2f {
                    4
                } else {
                    1
                };
                if self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] & xand != 0
                    && self.run_ledge_hop_timer()
                {
                    self.link_cancel_dash();
                    self.ram[LINK_ACTUAL_VEL_X] =
                        if self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] & 4 != 0 {
                            16
                        } else {
                            0u8.wrapping_sub(16)
                        };
                    self.setup_horizontal_ledge_hop(14);
                    return;
                }
            }

            if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 0x70 != 0
                && self.ram[TILEDETECT_VERTICAL_LEDGE] & 0x77 == 0
                && self.run_ledge_hop_timer()
            {
                self.link_cancel_dash();
                self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
                self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] =
                    if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 0x40 != 0 {
                        3
                    } else {
                        2
                    };
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
                self.link_find_valid_landing_tile_diagonal_north();
                return;
            }
        }

        if (self.ram[TILEDETECT_STAIR_TILE] & 7) == 7 {
            if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
                let r14 = (self.ram[TILEDETECT_STAIR_TILE] & 7) as u16;
                write_le_u16(&mut self.ram, R14, r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            } else if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 == 0 {
                self.ram[LINK_SPEED_SETTING] = 2;
                self.ram[LINK_SPEED_MODIFIER] = 1;
                return;
            }
        }

        if self.ram[LINK_SPEED_SETTING] == 2 {
            self.ram[LINK_SPEED_SETTING] = if self.ram[LINK_IS_RUNNING] != 0 {
                16
            } else {
                0
            };
        }
        self.replay_trace_drag_tail("outdoor-y-after-late-speed-setting");
        if self.ram[LINK_SPEED_MODIFIER] == 1 {
            self.ram[LINK_SPEED_MODIFIER] = 2;
        }
        self.replay_trace_drag_tail("outdoor-y-after-speed-modifier");

        if read_le_u16(&self.ram, R14) & 7 == 0 && read_le_u16(&self.ram, R12) & 5 != 0 {
            self.replay_trace_drag_tail("outdoor-y-before-slopes");
            self.flag_moving_into_slopes_y();
            self.replay_trace_drag_tail("outdoor-y-after-slopes");
            if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0 {
                return;
            }
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.replay_trace_drag_tail("outdoor-y-after-clear-diag");
        if self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] & 2 != 0
            && self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] == 0
        {
            let timeout = self.ram[GRAVESTONE_PUSH_TIMEOUT].wrapping_sub(1);
            self.ram[GRAVESTONE_PUSH_TIMEOUT] = timeout;
            if self.ram[LINK_IS_RUNNING] != 0 || (timeout as i8).is_negative() {
                let bak = read_le_u16(&self.ram, R14);
                self.ancilla_add_grave_stone(0x24, 4);
                write_le_u16(&mut self.ram, R14, bak);
                self.ram[GRAVESTONE_PUSH_TIMEOUT] = 52;
            }
        } else {
            self.ram[GRAVESTONE_PUSH_TIMEOUT] = 52;
        }
        self.replay_trace_drag_tail("outdoor-y-after-gravestone");

        if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7 != 0 {
            if (self.ram[LINK_INCAPACITATED_TIMER]
                | self.ram[COUNTDOWN_FOR_BLINK]
                | self.ram[LINK_CAPE_MODE])
                == 0
            {
                let should_damage = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] == 0 {
                    self.player_state_view().y() & 4 == 0
                } else {
                    self.player_state_view().y() & 4 != 0
                };
                if should_damage {
                    self.ram[LINK_GIVE_DAMAGE] = 8;
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7) as u16;
                write_le_u16(&mut self.ram, R14, r14);
            }
        }
        self.replay_trace_drag_tail("outdoor-y-before-snaps");
        self.handle_pushing_bonking_snaps_y();
        self.replay_trace_submodule("outdoor-y-after-snaps");
    }

    pub(super) fn start_movement_collision_checks_x_handle_outdoors(&mut self) {
        if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
            self.ram[LINK_SPEED_MODIFIER] = 0;
            if self.ram[LINK_SPEED_SETTING] == 2 {
                self.ram[LINK_SPEED_SETTING] = 0;
            }
        }

        if self.ram[TILEDETECT_PIT_TILE] & 5 != 0 && read_le_u16(&self.ram, R14) & 2 == 0 {
            self.start_falling_into_hole();
            return;
        }

        self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1B] =
            if read_le_u16(&self.ram, TILEDETECT_READ_SOMETHING) & 2 != 0 {
                self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] >> 1
            } else {
                0
            };

        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 4 != 0
            && self.ram[LINK_IS_IN_DEEP_WATER] == 0
            && self.ram[LINK_AUXILIARY_STATE] == 0
        {
            self.link_cancel_dash();
            self.link_reset_sword_and_item_usage();
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.link_reset_swimming_state();
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            if self.ram[DRAW_WATER_RIPPLES_OR_GRASS] == 1 && {
                self.link_force_unequip_cape_quietly();
                self.ram[LINK_ITEM_FLIPPERS] != 0
            } {
                if self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                }
            } else {
                self.restore_link_safe_return_position();
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.link_hop_in_or_out_of_water_x();
                self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            }
        }

        if if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
            self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 7 == 7
        } else {
            self.ram[TILEDETECT_VERTICAL_LEDGE] & 0x42 != 0
        } {
            write_le_u16(&mut self.ram, R14, 7);
            self.handle_pushing_bonking_snaps_x();
            return;
        }

        if read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) & 7 == 7
            && self.ram[LINK_IS_IN_DEEP_WATER] != 0
        {
            self.link_cancel_dash();
            if self.ram[LINK_AUXILIARY_STATE] == 0 {
                self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                self.ram[LINK_IS_IN_DEEP_WATER] = 0;
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ancilla_add_splash(0x15, 0);
                self.link_hop_in_or_out_of_water_x();
                return;
            }
        }

        if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 7 != 0 && self.run_ledge_hop_timer() {
            self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            self.ram[LINK_ACTUAL_VEL_X] = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 != 0 {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(12);
            if self.ram[PLAYER_IS_INDOORS] == 0 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 2;
            }
            let x_bak = self.player_state_view().x();
            let rv = self.link_hopping_horizontally_find_tile_x(
                (self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & !2) * 2,
            );
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = 1;
            if rv != 0xff {
                self.link_hopping_horizontally_find_tile_y();
            } else {
                self.link_hop_find_tile_to_land_on_south();
            }
            self.player_state_view_mut().set_x(x_bak);
            return;
        }

        if self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] & 0x77 != 0 && self.run_ledge_hop_timer() {
            self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[SOUND_EFFECT_1] & 7 == 0 {
                16
            } else {
                15
            };
            self.ram[LINK_ACTUAL_VEL_X] = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 != 0 {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.link_cancel_dash();
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = 20;
            self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = 20;
            self.set_link_z_coord_mirror_low_ff();
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[ALLOW_SCROLL_Z] = 1;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            return;
        }

        if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 0x70 != 0
            && self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 7 == 0
            && self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] & 0x77 == 0
            && self.ram[LINK_PLAYER_HANDLER_STATE] != 13
            && self.run_ledge_hop_timer()
        {
            self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            self.link_cancel_dash();
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.link_find_valid_landing_tile_diagonal_north();
            return;
        }

        if self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] & 7 != 0
            && self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 7 == 0
            && self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] & 0x77 == 0
            && self.run_ledge_hop_timer()
        {
            self.ram[LINK_ACTUAL_VEL_X] = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 != 0 {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(14);
            return;
        }

        if read_le_u16(&self.ram, R14) & 2 == 0 && read_le_u16(&self.ram, R12) & 5 != 0 {
            let skip_check =
                self.ram[LINK_IS_RUNNING] != 0 && self.ram[LINK_DIRECTION_FACING] & 4 == 0;
            const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
            if !skip_check || self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
            {
                self.flag_moving_into_slopes_x();
                if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0 {
                    return;
                }
            }
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7 != 0 {
            if (self.ram[LINK_INCAPACITATED_TIMER]
                | self.ram[COUNTDOWN_FOR_BLINK]
                | self.ram[LINK_CAPE_MODE])
                == 0
            {
                let should_damage = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] == 2 {
                    self.player_state_view().x() & 4 == 0
                } else {
                    self.player_state_view().x() & 4 != 0
                };
                if should_damage {
                    self.ram[LINK_GIVE_DAMAGE] = 8;
                    self.link_cancel_dash();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7) as u16;
                write_le_u16(&mut self.ram, R14, r14);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn start_falling_into_hole(&mut self) {
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 5 && self.ram[LINK_PLAYER_HANDLER_STATE] != 2 {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
            self.ram[PLAYER_NEAR_PIT_STATE] = 1;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
        }
    }

    fn setup_horizontal_ledge_hop(&mut self, player_state: u8) {
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[ALLOW_SCROLL_Z] = 1;
        self.ram[LINK_AUXILIARY_STATE] = 2;
        self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = 20;
        self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = 20;
        self.set_link_z_coord_mirror_low_ff();
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = player_state;
    }

    fn finish_indoor_y_collision(&mut self) {
        if self.ram[TILE_COLL_FLAG] & 2 == 0 {
            self.ram[IS_STANDING_IN_DOORWAY] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !2;
            self.ram[ROOM_TRANSITIONING_FLAGS] = 0;
            write_le_u16(&mut self.ram, FORCE_MOVE_ANY_DIRECTION, 0);
        }

        if read_le_u16(&self.ram, R14) & 7 == 0 && read_le_u16(&self.ram, R12) & 5 != 0 {
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
            self.flag_moving_into_slopes_y();
            if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f != 0 {
                return;
            }
        }

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        if read_le_u16(&self.ram, TILEDETECT_KEY_LOCK_GRAVESTONES) & 0x20 != 0 {
            let bak = read_le_u16(&self.ram, R14);
            let mut chest_position = 0;
            let _ = self.OpenChestForItem(
                read_le_u16(&self.ram, TILEDETECT_TILE_TYPE) as u8,
                &mut chest_position,
            );
            write_le_u16(&mut self.ram, TILEDETECT_TILE_TYPE, 0);
            write_le_u16(&mut self.ram, R14, bak);
        }

        self.finish_indoor_collision_common(true);
    }

    fn finish_indoor_collision_common(&mut self, y_axis: bool) {
        let r14 = read_le_u16(&self.ram, R14);
        if self.ram[LINK_IS_ON_LOWER_LEVEL] == 0 {
            if read_le_u16(&self.ram, TILEDETECT_WATER_STAIRCASE) & 7 != 0 {
                self.set_player_layer_collision(crate::ram::player::LAYER_COLLISION_BG1, true);
            } else if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7 == 0 && r14 & 2 == 0 {
                self.set_player_layer_collision(crate::ram::player::LAYER_COLLISION_BG1, false);
            }
        } else if read_le_u16(&self.ram, TILEDETECT_MOVING_FLOOR_TILES) & 7 != 0 {
            self.set_player_layer_collision(crate::ram::player::LAYER_COLLISION_BG2, true);
        } else {
            self.set_player_layer_collision(crate::ram::player::LAYER_COLLISION_BG2, false);
        }

        if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 0x2200 != 0 {
            const DX: [u8; 4] = [8, 8, 0, 15];
            const DY: [u8; 4] = [8, 24, 16, 16];
            let dy = if read_le_u16(&self.ram, TILEDETECT_MISC_TILES) & 0x2000 != 0 {
                8
            } else {
                0
            };
            let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
            let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_add(5);
            write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(DY[dir] as u16)
                .wrapping_sub(dy);
            let x = self.player_state_view().x().wrapping_add(DX[dir] as u16);
            self.dungeon_delete_rupee_tile_for_player(x, y);
            self.ancilla_sfx3_near(10);
        }

        let var4 = read_le_u16(&self.ram, MOVING_FLOOR_BG_CHECK_FLAGS);
        if var4 & 0x22 != 0 {
            self.ram[LINK_ON_CONVEYOR_BELT] = if var4 & 0x20 != 0 { 2 } else { 1 };
        } else if var4 & 0x2200 != 0 {
            self.ram[LINK_ON_CONVEYOR_BELT] = if var4 & 0x2000 != 0 { 4 } else { 3 };
        } else if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7 == 0 && r14 & 2 == 0 {
            self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        }

        if y_axis {
            self.finish_indoor_y_collision_tail();
        } else {
            self.finish_indoor_x_collision_tail();
        }
    }

    fn finish_indoor_y_collision_tail(&mut self) {
        if (self.ram[TILEDETECT_VERTICAL_LEDGE] & 7) == 7 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.ram[ABOUT_TO_JUMP_OFF_LEDGE].wrapping_add(1);
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.ancilla_sfx2_near(0x20);
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.link_hop_in_or_out_of_water_y();
        } else if (read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 7) == 7
            && self.ram[LINK_IS_IN_DEEP_WATER] == 0
        {
            self.link_cancel_dash();
            if self.ram[TS_COPY] == 0 {
                self.dungeon_handle_layer_change();
            } else {
                self.ram[LINK_IS_IN_DEEP_WATER] = 1;
                self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
                self.ram[LINK_STATE_BITS] = 0;
                self.ram[LINK_PICKING_THROW_STATE] = 0;
                self.ram[LINK_GRABBING_WALL] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
                self.link_reset_swimming_state();
                self.ancilla_sfx2_near(0x20);
            }
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.link_hop_in_or_out_of_water_y();
        } else if read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) & 2 != 0
            && self.ram[LINK_IS_IN_DEEP_WATER] != 0
        {
            if self.ram[LINK_AUXILIARY_STATE] != 0 {
                write_le_u16(&mut self.ram, R14, 7);
            } else {
                self.link_cancel_dash();
                self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
                self.ram[LINK_IS_IN_DEEP_WATER] = 0;
                if self.ancilla_add_splash(0x15, 0) {
                    self.ram[LINK_IS_IN_DEEP_WATER] = 1;
                    write_le_u16(&mut self.ram, R14, 7);
                } else {
                    self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                    self.link_hop_in_or_out_of_water_y();
                }
            }
        }

        if (self.ram[TILEDETECT_STAIR_TILE] & 7) == 7 {
            if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
                let stair_bits = (self.ram[TILEDETECT_STAIR_TILE] & 7) as u16;
                write_le_u16(&mut self.ram, R14, stair_bits);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            let stairs = read_le_u16(&self.ram, TILEDETECT_INROOM_STAIRCASE);
            if stairs & 0x77 != 0 {
                let submodule = if stairs & 0x70 != 0 { 16 } else { 8 };
                self.frame_control_view_mut().set_submodule(submodule);
                self.frame_control_view_mut().set_main_module(7);
                self.link_cancel_dash();
            } else {
                const FEATURES0_TURN_WHILE_DASHING: u32 = 4;
                if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_TURN_WHILE_DASHING != 0 {
                    self.link_cancel_dash();
                }
            }
            if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 == 0 {
                self.ram[LINK_SPEED_SETTING] = 2;
                self.ram[LINK_SPEED_MODIFIER] = 1;
                return;
            }
        }

        if self.finish_indoor_collision_shared_tail(true) {
            return;
        }
        self.handle_pushing_bonking_snaps_y();
    }

    fn finish_indoor_x_collision_tail(&mut self) {
        if self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] & 7 == 7 && self.run_ledge_hop_timer() {
            self.link_cancel_dash();
            self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = self.ram[ABOUT_TO_JUMP_OFF_LEDGE].wrapping_add(1);
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.link_hop_in_or_out_of_water_x();
            self.ram[SOUND_EFFECT_1] = 0x20 | self.link_calculate_sfx_pan();
            return;
        }

        if self.finish_indoor_collision_shared_tail(false) {
            return;
        }
        if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
            self.ram[LINK_SPEED_MODIFIER] = 0;
            if self.ram[LINK_SPEED_SETTING] == 2 {
                self.ram[LINK_SPEED_SETTING] = 0;
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn finish_indoor_collision_shared_tail(&mut self, y_axis: bool) -> bool {
        if y_axis {
            if self.ram[LINK_SPEED_SETTING] == 2 {
                self.ram[LINK_SPEED_SETTING] = if self.ram[LINK_IS_RUNNING] != 0 {
                    16
                } else {
                    0
                };
            }
            if self.ram[LINK_SPEED_MODIFIER] == 1 {
                self.ram[LINK_SPEED_MODIFIER] = 2;
            }
        }

        let r14 = read_le_u16(&self.ram, R14);
        if self.ram[TILEDETECT_PIT_TILE] & 5 != 0 && r14 & 2 == 0 {
            self.start_falling_into_hole();
            return true;
        }

        if y_axis {
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        } else {
            self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        }
        if self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7 != 0 {
            if (self.ram[LINK_INCAPACITATED_TIMER]
                | self.ram[COUNTDOWN_FOR_BLINK]
                | self.ram[LINK_CAPE_MODE])
                == 0
            {
                let coord = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 == 0 {
                    self.player_state_view().y()
                } else {
                    self.player_state_view().x()
                };
                let low_phase = coord & 4 == 0;
                let damage = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 1 == 0 {
                    low_phase
                } else {
                    !low_phase
                };
                if damage {
                    self.ram[LINK_GIVE_DAMAGE] = 8;
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return true;
                }
            } else {
                let spike_bits = (self.ram[BITFIELD_SPIKE_CACTUS_TILES] & 7) as u16;
                write_le_u16(&mut self.ram, R14, spike_bits);
            }
        }

        if self.ram[DUNG_HDR_COLLISION] == 0
            || self.ram[DUNG_HDR_COLLISION] == 4
            || self.ram[LINK_IS_ON_LOWER_LEVEL] == 0
        {
            self.handle_indoor_pushblock_timeout(y_axis);
        }
        false
    }

    fn handle_indoor_pushblock_timeout(&mut self, y_axis: bool) {
        let var2 = read_le_u16(&self.ram, TILEDETECT_BLOCK_FLAGS_LO);
        if var2 != 0 && self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] == 0 {
            self.ram[TILEDETECT_STAIRCASE_CACHE] = var2 as u8;
            self.ram[GRAVESTONE_PUSH_TIMEOUT] = self.ram[GRAVESTONE_PUSH_TIMEOUT].wrapping_sub(1);
            if !(self.ram[GRAVESTONE_PUSH_TIMEOUT] as i8).is_negative() {
                return;
            }
            let mut bits = var2;
            for i in (0..=15).rev() {
                if bits & 0x8000 != 0 {
                    let idx = self.find_free_moving_block_slot(i);
                    if idx != 0xff {
                        let slot = idx as usize;
                        write_le_u16(&mut self.ram, R14, idx as u16);
                        if !self.initialize_push_block(idx, (i * 2) as u8) {
                            self.sprite_dungeon_draw_single_push_block(slot * 2);
                            write_le_u16(&mut self.ram, R14, 4);
                            let facing = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] * 2;
                            self.ram[PUSHEDBLOCK_FACING_PLAYER + slot * 2] = facing;
                            self.ram[PUSH_BLOCK_DIRECTION_PLAYER] = facing;
                            self.ram[PUSHEDBLOCKS_TARGET + slot * 2] = if y_axis {
                                let y_lo = self.ram[PUSHEDBLOCKS_Y_LO + slot * 2];
                                y_lo.wrapping_sub(u8::from(
                                    self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] == 1,
                                ))
                            } else {
                                let x_lo = self.ram[PUSHEDBLOCKS_X_LO + slot * 2];
                                x_lo.wrapping_sub(u8::from(
                                    self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] != 2,
                                ))
                            } & 0x0f;
                        }
                    }
                }
                bits <<= 1;
            }
        }
        self.ram[GRAVESTONE_PUSH_TIMEOUT] = 21;
    }

    fn dungeon_delete_rupee_tile_for_player(&mut self, x: u16, y: u16) {
        let pos = ((y & 0x01f8) * 8) | ((x & 0x01f8) >> 3);
        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        write_le_u16(&mut self.ram, dst + 4, 0x190f);
        write_le_u16(&mut self.ram, dst + 10, 0x190f);
        write_le_u16(&mut self.ram, DUNG_BG2 + pos as usize * 2, 0x190f);
        write_le_u16(&mut self.ram, DUNG_BG2 + (pos + 64) as usize * 2, 0x190f);
        let attr = u16::from(self.ram[ATTRIBUTES_FOR_TILE_PLAYER + (0x190f & 0x03ff)]) * 0x0101;
        let vram0 = self.Dungeon_MapVramAddr(pos);
        let vram1 = self.Dungeon_MapVramAddr(pos + 64);
        write_le_u16(&mut self.ram, DUNG_BG2_ATTR_TABLE + pos as usize, attr);
        write_le_u16(
            &mut self.ram,
            DUNG_BG2_ATTR_TABLE + (pos + 64) as usize,
            attr,
        );
        write_le_u16(&mut self.ram, dst, vram0);
        write_le_u16(&mut self.ram, dst + 6, vram1);
        write_le_u16(&mut self.ram, dst + 2, 0x0100);
        write_le_u16(&mut self.ram, dst + 8, 0x0100);
        write_le_u16(&mut self.ram, dst + 12, 0xffff);
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, (upload + 24) as u16);
        self.ram[DUNG_SAVEGAME_STATE_BITS + 1] |= 0x10;
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    }

    pub(super) fn link_handle_liftables(&mut self) -> u8 {
        const ACTION_FOR_GLOVES: [u8; 7] = [0, 1, 0, 0, 2, 1, 2];
        const ACTION_FOR_TILE: [u8; 7] = [2, 3, 1, 4, 0, 5, 6];
        const ACTION_X: [i8; 4] = [7, 7, -3, 16];
        const ACTION_Y: [i8; 4] = [6, 24, 12, 12];

        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();

        let facing = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let y0 = link_y.wrapping_add(ACTION_Y[facing] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(20) & mask;
        let x0 = (link_x.wrapping_add(ACTION_X[facing] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(8) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let mut action = if (read_le_u16(&self.ram, R14)
            | self.ram[TILEDETECT_VERTICAL_LEDGE] as u16)
            & 1
            != 0
        {
            3
        } else {
            2
        };

        if self.ram[PLAYER_IS_INDOORS] != 0 {
            let liftable = self.Dungeon_CheckForAndIDLiftableTile();
            if liftable != 0xffff {
                self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1] =
                    ACTION_FOR_TILE[(liftable & 0x0f) as usize];
            } else {
                if read_le_u16(&self.ram, TILEDETECT_READ_SOMETHING) & 1 != 0
                    && self.ram[LINK_DIRECTION_FACING] == 0
                    && self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] == 0
                {
                    action = 4;
                }
                if read_le_u16(&self.ram, TILEDETECT_CHEST) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
        } else {
            if read_le_u16(&self.ram, TILEDETECT_READ_SOMETHING) & 1 == 0 {
                if read_le_u16(&self.ram, TILEDETECT_CHEST) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            if self.ram[LINK_DIRECTION_FACING] == 0
                && self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] == 0
            {
                action = 4;
                if read_le_u16(&self.ram, TILEDETECT_CHEST) & 1 != 0 {
                    action = 5;
                }
                return action;
            }
            self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1] =
                self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] >> 1;
        }

        let liftable_index = self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1] as usize;
        if self.ram[LINK_ITEM_GLOVES] >= ACTION_FOR_GLOVES[liftable_index] {
            action = 1;
        }

        if read_le_u16(&self.ram, TILEDETECT_CHEST) & 1 != 0 {
            action = 5;
        }
        action
    }

    pub(super) fn link_bonk_and_smash(&mut self) {
        if self.ram[LINK_IS_RUNNING] == 0
            || self.ram[LINK_DASH_CTR] == 64
            || self.ram[BITMASK_FOR_DASHABLE_TILES] & 0x70 == 0
        {
            return;
        }
        const LINK_LIFT_TAB: [u8; 9] = [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
        for i in 0..2 {
            if let Some((j, x, y)) = self.overworld_smash_rock_pile_result(i != 0) {
                if let Some(k) = LINK_LIFT_TAB.iter().position(|&v| v == j) {
                    if k == 2 || k == 4 {
                        self.ancilla_sfx3_near(0x32);
                    }
                    self.sprite_spawn_immediately_smashed_terrain(k as u8, x, y);
                }
            }
        }
    }

    pub(super) fn overworld_smash_rock_pile_result(
        &mut self,
        down_one_tile: bool,
    ) -> Option<(u8, u16, u16)> {
        let bak = self.player_state_view().y();
        if down_one_tile {
            self.player_state_view_mut().set_y(bak.wrapping_add(8));
        }
        let (pos, x, y) = self.overworld_get_link_map16_coords_result();
        self.player_state_view_mut().set_y(bak);
        let a = read_le_u16(&self.ram, DUNG_BG2 + ((pos >> 1) as usize) * 2);
        match a {
            0x226 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 0, x, y), x, y)),
            0x227 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 1, x, y), x, y)),
            0x228 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 2, x, y), x, y)),
            0x229 => Some((self.smash_rock_pile_from_lift_impl(a, pos, 3, x, y), x, y)),
            0x36 => Some((
                self.overworld_lifting_small_obj_impl(a, pos, 0x0dc7, x, y),
                x,
                y,
            )),
            _ => None,
        }
    }

    pub(super) fn overworld_get_link_map16_coords_result(&self) -> (u16, u16, u16) {
        const X: [i16; 4] = [7, 7, -3, 16];
        const Y: [i16; 4] = [6, 24, 12, 12];
        let dir = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let x = self.player_state_view().x().wrapping_add(X[dir] as u16) & !0x0f;
        let y = self.player_state_view().y().wrapping_add(Y[dir] as u16) & !0x0f;
        let pos = ((y.wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_Y))
            & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_Y))
            << 3)
            .wrapping_add(
                ((x >> 3).wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_X)))
                    & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_X),
            );
        (pos, x, y)
    }

    pub(super) fn overworld_lifting_small_obj_impl(
        &mut self,
        a: u16,
        pos: u16,
        mut tile: u16,
        x: u16,
        y: u16,
    ) -> u8 {
        let secret = self.overworld_reveal_secret_for_smash(pos);
        if secret != 0 {
            tile = secret;
        }
        write_le_u16(&mut self.ram, DUNG_BG2 + ((pos >> 1) as usize) * 2, tile);
        self.overworld_memorize_map16_change_for_smash(pos, tile);
        self.overworld_draw_map16_for_smash(pos, tile);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
        self.map16_quadrant_attr(a, x, y)
    }

    pub(super) fn smash_rock_pile_from_lift_impl(
        &mut self,
        a: u16,
        pos: u16,
        quadrant: usize,
        x: u16,
        y: u16,
    ) -> u8 {
        const BIG_ROCK_TAB1: [i16; 4] = [0, -1, -64, -65];
        const BIG_ROCK_TAB_Y: [i16; 4] = [0, 0, -64, -64];
        const BIG_ROCK_TAB_X: [i16; 4] = [0, -1, 0, -1];
        let pos = 2 * ((pos >> 1).wrapping_add(BIG_ROCK_TAB1[quadrant] as u16));
        write_le_u16(&mut self.ram, BIG_ROCK_STARTING_ADDRESS, pos);
        write_le_u16(&mut self.ram, DOOR_OPEN_CLOSED_COUNTER, 40);
        let secret = self.overworld_reveal_secret_for_smash(pos);
        if secret == 0xffff {
            let screen = u16::from(self.world_state_view().overworld_screen()) as usize;
            self.ram[SAVE_OW_EVENT_INFO_PLAYER + screen] |= 0x20;
            self.ram[SOUND_EFFECT_2] = 27;
            write_le_u16(&mut self.ram, DOOR_OPEN_CLOSED_COUNTER, 80);
        }
        let x = x.wrapping_add((BIG_ROCK_TAB_X[quadrant] * 2) as u16);
        let y = y.wrapping_add((BIG_ROCK_TAB_Y[quadrant] * 2) as u16);
        self.overworld_do_map_update32x32_b_for_smash();
        self.map16_quadrant_attr(a, x, y)
    }

    pub(super) fn overworld_reveal_secret_for_smash(&mut self, pos: u16) -> u16 {
        self.ram[DUNG_SECRETS_UNK1_PLAYER] = 0;

        let screen = u16::from(self.world_state_view().overworld_screen()) as usize;
        if screen >= 0x80 {
            self.adjust_secret_for_powder_for_smash();
            return 0;
        }

        let secret_offsets = self
            .asset_raw(157)
            .expect("overworld_reveal_secret_for_smash missing kOverworldSecrets_Offs asset")
            .to_vec();
        let secrets = self
            .asset_raw(158)
            .expect("overworld_reveal_secret_for_smash missing kOverworldSecrets asset")
            .to_vec();
        let ptr = u16::from(secret_offsets[screen * 2])
            | (u16::from(secret_offsets[screen * 2 + 1]) << 8);
        let mut ptr = ptr as usize;
        loop {
            let x = u16::from(secrets[ptr]) | (u16::from(secrets[ptr + 1]) << 8);
            if x == 0xffff {
                self.adjust_secret_for_powder_for_smash();
                return 0;
            }
            if x & 0x7fff == pos {
                break;
            }
            ptr += 3;
        }

        let data = secrets[ptr + 2];
        if data != 0 && data < 0x80 {
            self.ram[DUNG_SECRETS_UNK1_PLAYER] |= data;
        }
        if data < 0x80 {
            self.adjust_secret_for_powder_for_smash();
            return 0;
        }

        self.ram[DUNG_SECRETS_UNK1_PLAYER] = 0xff;
        if data != 0x84 && self.ram[SAVE_OW_EVENT_INFO_PLAYER + screen] & 2 == 0 {
            if screen == 0x5b && self.ram[FOLLOWER_INDICATOR] != 13 {
                self.adjust_secret_for_powder_for_smash();
                return 0;
            }
            self.ram[SOUND_EFFECT_2] = 0x1b;
        } else if data == 0x82 && self.read_u32_ram(ENHANCED_FEATURES0) & 4096 != 0 {
            self.ram[SOUND_EFFECT_2] = 0x1b;
        }

        const TILE_BELOW: [u16; 4] = [0x0dcc, 0x0212, 0xffff, 0x0db4];
        self.adjust_secret_for_powder_for_smash();
        TILE_BELOW[((data & 0x0f) >> 1) as usize]
    }

    fn adjust_secret_for_powder_for_smash(&mut self) {
        if self.ram[LINK_ITEM_IN_HAND] & 0x40 != 0 {
            write_le_u16(&mut self.ram, DUNG_SECRETS_UNK1_PLAYER, 4);
        }
    }

    pub(super) fn overworld_memorize_map16_change_for_smash(&mut self, pos: u16, value: u16) {
        if value == 0x0dc5 || value == 0x0dc9 {
            return;
        }
        let x = read_le_u16(&self.ram, NUM_MEMORIZED_TILES) as usize;
        write_le_u16(&mut self.ram, MEMORIZED_TILE_VALUE_PLAYER + x, value);
        write_le_u16(&mut self.ram, MEMORIZED_TILE_ADDR_PLAYER + x, pos);
        write_le_u16(&mut self.ram, NUM_MEMORIZED_TILES, (x + 2) as u16);
    }

    pub(super) fn overworld_draw_map16_for_smash(&mut self, pos: u16, value: u16) {
        let vram_pos = self.overworld_find_map16_vram_address_for_smash(pos);
        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        let dst = VRAM_UPLOAD_DATA + upload;
        let src = value as usize * 4;
        let map8 = self
            .asset_raw(70)
            .expect("overworld_draw_map16_for_smash missing kMap16ToMap8 asset");
        let tile0 = u16::from(map8[src * 2]) | (u16::from(map8[src * 2 + 1]) << 8);
        let tile1 = u16::from(map8[(src + 1) * 2]) | (u16::from(map8[(src + 1) * 2 + 1]) << 8);
        let tile2 = u16::from(map8[(src + 2) * 2]) | (u16::from(map8[(src + 2) * 2 + 1]) << 8);
        let tile3 = u16::from(map8[(src + 3) * 2]) | (u16::from(map8[(src + 3) * 2 + 1]) << 8);
        write_le_u16(&mut self.ram, dst, vram_pos.swap_bytes());
        write_le_u16(&mut self.ram, dst + 2, 0x0300);
        write_le_u16(&mut self.ram, dst + 4, tile0);
        write_le_u16(&mut self.ram, dst + 6, tile1);
        write_le_u16(
            &mut self.ram,
            dst + 8,
            vram_pos.wrapping_add(0x20).swap_bytes(),
        );
        write_le_u16(&mut self.ram, dst + 10, 0x0300);
        write_le_u16(&mut self.ram, dst + 12, tile2);
        write_le_u16(&mut self.ram, dst + 14, tile3);
        write_le_u16(&mut self.ram, dst + 16, 0xffff);
        write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET, (upload + 16) as u16);
    }

    fn overworld_find_map16_vram_address_for_smash(&self, addr: u16) -> u16 {
        (if addr & 0x3f >= 0x20 { 0x0400 } else { 0 })
            + (if addr & 0x0fff >= 0x0800 { 0x0800 } else { 0 })
            + (addr & 0x001f)
            + ((addr & 0x0780) >> 1)
    }

    fn overworld_do_map_update32x32_b_for_smash(&mut self) {
        self.overworld_do_map_update32x32_for_smash();
        self.ram[DOOR_OPEN_CLOSED_COUNTER] = 0;
    }

    fn overworld_do_map_update32x32_for_smash(&mut self) {
        const DOOR_ANIM_TILES: [u16; 56] = [
            0x0da8, 0x0da9, 0x0daa, 0x0dab, 0x0dac, 0x0dad, 0x0dae, 0x0daf, 0x0db0, 0x0db1, 0x0db2,
            0x0db3, 0x0db6, 0x0db7, 0x0db8, 0x0db9, 0x0dba, 0x0dbb, 0x0dbc, 0x0dbd, 0x0dcd, 0x0dce,
            0x0dcf, 0x0dd0, 0x0dd3, 0x0dd4, 0x0dd5, 0x0dd6, 0x0dd7, 0x0dd8, 0x0dd9, 0x0dda, 0x0dd1,
            0x0dd2, 0x0dd3, 0x0dd4, 0x0dd1, 0x0dd2, 0x0dd7, 0x0dd8, 0x0918, 0x0919, 0x091a, 0x091b,
            0x0ddb, 0x0ddc, 0x0ddd, 0x0dde, 0x0dd1, 0x0dd2, 0x0ddb, 0x0ddc, 0x0e21, 0x0e22, 0x0e23,
            0x0e24,
        ];
        let i = read_le_u16(&self.ram, NUM_MEMORIZED_TILES) as usize;
        let j = (read_le_u16(&self.ram, DOOR_OPEN_CLOSED_COUNTER) >> 1) as usize;
        let base = read_le_u16(&self.ram, BIG_ROCK_STARTING_ADDRESS);
        let entries = [
            (base, DOOR_ANIM_TILES[j]),
            (base.wrapping_add(2), DOOR_ANIM_TILES[j + 1]),
            (base.wrapping_add(0x80), DOOR_ANIM_TILES[j + 2]),
            (base.wrapping_add(0x82), DOOR_ANIM_TILES[j + 3]),
        ];
        for (n, (pos, tile)) in entries.into_iter().enumerate() {
            write_le_u16(&mut self.ram, MEMORIZED_TILE_ADDR_PLAYER + i + n * 2, pos);
            write_le_u16(&mut self.ram, MEMORIZED_TILE_VALUE_PLAYER + i + n * 2, tile);
            self.overworld_draw_map16_persist_for_smash(pos, tile);
        }
        let upload = read_le_u16(&self.ram, VRAM_UPLOAD_OFFSET) as usize;
        write_le_u16(&mut self.ram, VRAM_UPLOAD_DATA + upload, 0xffff);
        write_le_u16(&mut self.ram, NUM_MEMORIZED_TILES, (i + 8) as u16);
        let step = read_le_u16(&self.ram, DOOR_ANIMATION_STEP_INDICATOR_PLAYER).wrapping_add(
            if read_le_u16(&self.ram, DOOR_OPEN_CLOSED_COUNTER) == 32 {
                2
            } else {
                1
            },
        );
        write_le_u16(&mut self.ram, DOOR_ANIMATION_STEP_INDICATOR_PLAYER, step);
        self.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
        self.ram[DOOR_OPEN_CLOSED_COUNTER] = self.ram[DOOR_OPEN_CLOSED_COUNTER].wrapping_add(1);
    }

    fn overworld_draw_map16_persist_for_smash(&mut self, pos: u16, value: u16) {
        write_le_u16(&mut self.ram, DUNG_BG2 + ((pos >> 1) as usize) * 2, value);
        self.overworld_draw_map16_for_smash(pos, value);
    }

    fn map16_quadrant_attr(&self, map16: u16, x: u16, y: u16) -> u8 {
        let index = map16 as usize * 4 + usize::from(x & 8 != 0) * 2 + usize::from(y & 8 != 0);
        let map16_to_map8 = self
            .asset_raw(70)
            .expect("map16_quadrant_attr missing kMap16ToMap8 asset");
        let tile_attrs = self
            .asset_raw(163)
            .expect("map16_quadrant_attr missing kMap8DataToTileAttr asset");
        let map8 = read_word_from_slice(map16_to_map8, index * 2);
        tile_attrs[(map8 & 0x01ff) as usize]
    }

    pub(super) fn link_handle_diagonal_collision(&mut self) {
        if self.check_if_room_needs_double_layer_check() {
            self.player_limit_directions_inner();
            self.create_velocity_from_moving_background();
        }
        self.ram[LINK_DIRECTION] &= 0x0f;
        self.player_limit_directions_inner();
    }

    pub(super) fn link_handle_diagonal_kickback(&mut self) {
        if self.ram[LINK_X_VEL] == 0 || self.ram[LINK_Y_VEL] == 0 {
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            return;
        }

        copy_le_u16(&mut self.ram, LINK_Y_COORD_COPY, LINK_Y_COORD);
        copy_le_u16(&mut self.ram, LINK_X_COORD_COPY, LINK_X_COORD);

        self.tile_detect_movement_x(if (self.ram[LINK_X_VEL] as i8).is_negative() {
            2
        } else {
            3
        });
        if read_le_u16(&self.ram, R12) & 5 == 0 {
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            return;
        }
        self.flag_moving_into_slopes_x();
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f == 0 {
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            return;
        }

        let xd = self
            .player_state_view()
            .x()
            .wrapping_sub(read_le_u16(&self.ram, LINK_X_COORD_COPY)) as u8;
        copy_le_u16(&mut self.ram, LINK_X_COORD, LINK_X_COORD_COPY);
        self.ram[LINK_X_VEL] = xd;

        self.tile_detect_movement_y(if (self.ram[LINK_Y_VEL] as i8).is_negative() {
            0
        } else {
            1
        });
        if read_le_u16(&self.ram, R12) & 5 == 0 {
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            return;
        }
        self.flag_moving_into_slopes_y();
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0f == 0 {
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            return;
        }

        self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] = self.ram[LINK_MOVING_AGAINST_DIAG_TILE];
        let yd = self
            .player_state_view()
            .y()
            .wrapping_sub(read_le_u16(&self.ram, LINK_Y_COORD_COPY)) as u8;
        self.ram[LINK_Y_VEL] = yd;

        const X0: [i8; 10] = [0, 1, 1, 1, 2, 2, 2, 3, 3, 3];
        const X1: [i8; 10] = [0, -1, -1, -1, -2, -2, -2, -3, -3, -3];
        let x_vel = self.ram[LINK_X_VEL] as i8;
        let x_idx = x_vel.unsigned_abs() as usize;
        let x_delta = if x_vel < 0 { X1[x_idx] } else { X0[x_idx] };
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(x_delta as i16 as u16);
        self.player_state_view_mut().set_x(x);

        const Y0: [i8; 10] = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3];
        const Y1: [i8; 16] = [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, -91, 48, -16, 4, -91, 49];
        let y_vel = self.ram[LINK_Y_VEL] as i8;
        let y_idx = y_vel.unsigned_abs() as usize;
        let y_delta = if y_vel < 0 { Y1[y_idx] } else { Y0[y_idx] };
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(y_delta as i16 as u16);
        self.player_state_view_mut().set_y(y);

        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
    }

    pub(super) fn link_handle_cardinal_collision(&mut self) {
        self.replay_trace_submodule("cardinal-entry");
        write_le_u16(&mut self.ram, TILEDETECT_DIAG_STATE, 0);
        write_le_u16(&mut self.ram, TILEDETECT_DIAGONAL_TILE, 0);

        let can_double_layer = if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x30 != 0 {
            true
        } else {
            self.link_handle_diagonal_kickback();
            self.ram[MOVING_AGAINST_DIAG_DEADLOCKED] == 0
        };

        if can_double_layer && self.check_if_room_needs_double_layer_check() {
            if self.ram[DUNG_HDR_COLLISION] >= 2 && self.ram[DUNG_HDR_COLLISION] != 3 {
                self.ram[TILE_COLL_FLAG] = 2;
                self.player_tile_detect_nearby();
                self.ram[TILE_COLLISION_BITS_PRIMARY] = read_le_u16(&self.ram, R14) as u8;
                if self.ram[TILE_COLLISION_BITS_PRIMARY] != 0 {
                    self.ram[LINK_Y_VEL] =
                        self.ram[LINK_Y_VEL].wrapping_add(self.ram[DUNG_FLOOR_Y_VEL]);
                    self.ram[LINK_X_VEL] =
                        self.ram[LINK_X_VEL].wrapping_add(self.ram[DUNG_FLOOR_X_VEL]);

                    let a = read_le_u16(&self.ram, R14) as u8;
                    let horizontal_first = if a == 12 || a == 3 {
                        false
                    } else if a == 10 || a == 5 {
                        true
                    } else if (a & 0x0c) == 0 && (a & 3) == 0 {
                        false
                    } else if self.ram[LINK_Y_VEL] != 0 {
                        true
                    } else if self.ram[LINK_X_VEL] == 0 {
                        false
                    } else {
                        (self.ram[DUNG_FLOOR_Y_VEL] as i8) >= 0
                    };
                    if horizontal_first {
                        self.run_slope_collision_checks_horizontal_first();
                    } else {
                        self.run_slope_collision_checks_vertical_first();
                    }
                } else {
                    self.run_slope_collision_checks_vertical_first();
                }
            } else {
                self.run_slope_collision_checks_vertical_first();
            }
            self.create_velocity_from_moving_background();
        }

        let collision = self.ram[DUNG_HDR_COLLISION];
        let moved = (self.ram[LINK_X_VEL] | self.ram[LINK_Y_VEL]) != 0;
        if collision == 2 {
            self.player_tile_detect_nearby();
            if (read_le_u16(&self.ram, R14) as u8 | self.ram[TILE_COLLISION_BITS_PRIMARY]) == 0x0f {
                if self.ram[COUNTDOWN_FOR_BLINK] == 0 {
                    self.ram[COUNTDOWN_FOR_BLINK] = 58;
                }
                if self.ram[LINK_DIRECTION] == 0 {
                    if self.ram[DUNG_FLOOR_Y_VEL] != 0 {
                        self.ram[LINK_Y_VEL] = (0u8).wrapping_sub(self.ram[LINK_Y_VEL]);
                    }
                    if self.ram[DUNG_FLOOR_X_VEL] != 0 {
                        self.ram[LINK_X_VEL] = (0u8).wrapping_sub(self.ram[LINK_X_VEL]);
                    }
                }
            }
            self.ram[TILE_COLL_FLAG] = 1;
            self.run_slope_collision_checks_vertical_first();
        } else if collision == 3 {
            self.ram[TILE_COLL_FLAG] = 1;
            self.run_slope_collision_checks_horizontal_first();
        } else if collision == 4 || moved {
            self.ram[TILE_COLL_FLAG] = 1;
            self.run_slope_collision_checks_vertical_first();
        } else if !matches!(self.ram[LINK_PLAYER_HANDLER_STATE], 19 | 8 | 9 | 10 | 3) {
            self.player_tile_detect_nearby();
            if self.ram[TILEDETECT_PIT_TILE] & 0x0f != 0 {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
                if self.ram[LINK_IS_RUNNING] == 0 {
                    self.ram[LINK_SPEED_SETTING] = 4;
                }
            }
        }

        self.tile_detect_main_handler(0);
        self.replay_trace_submodule("cardinal-after-tile-main");
        if self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] != 0 {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 11 {
            self.ram[LINK_Y_VEL] =
                self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);
        }
        if self.ram[LINK_Y_VEL] != 0 {
            self.ram[LINK_DIRECTION] = (self.ram[LINK_DIRECTION] & 3)
                | if (self.ram[LINK_Y_VEL] as i8).is_negative() {
                    8
                } else {
                    4
                };
        }

        self.ram[LINK_X_VEL] =
            self.ram[LINK_X_COORD].wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_LO]);
        if self.ram[LINK_X_VEL] != 0 {
            self.ram[LINK_DIRECTION] = (self.ram[LINK_DIRECTION] & 0x0c)
                | if (self.ram[LINK_X_VEL] as i8).is_negative() {
                    2
                } else {
                    1
                };
        }
        self.replay_trace_submodule("cardinal-after-dir-vel");

        if self.ram[PLAYER_IS_INDOORS] == 0
            || self.ram[DUNG_HDR_COLLISION] != 4
            || self.ram[LINK_PLAYER_HANDLER_STATE] != 4
        {
            return;
        }

        if self.ram[DUNG_FLOOR_Y_VEL] != 0
            && self.ram[LINK_Y_VEL].wrapping_sub(self.ram[DUNG_FLOOR_Y_VEL]) == 0
        {
            if (self.ram[DUNG_FLOOR_Y_VEL] as i8).is_negative() {
                self.ram[LINK_DIRECTION] &= !8;
            } else {
                self.ram[LINK_DIRECTION] &= !4;
            }
        }
        if self.ram[DUNG_FLOOR_X_VEL] != 0
            && self.ram[LINK_X_VEL].wrapping_sub(self.ram[DUNG_FLOOR_X_VEL]) == 0
        {
            if (self.ram[DUNG_FLOOR_X_VEL] as i8).is_negative() {
                self.ram[LINK_DIRECTION] &= !2;
            } else {
                self.ram[LINK_DIRECTION] &= !1;
            }
        }
    }

    pub(super) fn link_state_recoil(&mut self) {
        self.replay_trace_player_state("recoil-entry");
        let old_x = self.player_state_view().x();
        let old_y = self.player_state_view().y();
        self.store_link_safe_return_position(old_x, old_y);

        self.link_handle_change_in_z_velocity();
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;

        if (self.ram[LINK_Z_COORD] as i8).is_negative()
            && (self.ram[LINK_ACTUAL_VEL_Z] as i8).is_negative()
        {
            self.tile_detect_main_handler(5);
            if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 1 != 0 {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                self.link_set_to_deep_water();
                self.link_reset_sword_and_item_usage();
                self.ancilla_add_splash(21, 0);
                self.link_handle_recoil_and_timer(true);
            } else {
                self.ram[LINK_RECOILMODE_TIMER] = self.ram[LINK_RECOILMODE_TIMER].wrapping_add(1);
                if self.ram[LINK_RECOILMODE_TIMER] != 4 {
                    let mut z = self.ram[LINK_ACTUAL_VEL_Z_COPY];
                    let mut s = self.ram[LINK_RECOILMODE_TIMER];
                    loop {
                        z >>= 1;
                        s = s.wrapping_sub(1);
                        if s != 0 {
                            break;
                        }
                    }
                    self.ram[LINK_ACTUAL_VEL_Z] = z;
                } else {
                    self.ram[LINK_RECOILMODE_TIMER] = 3;
                }
                self.link_handle_recoil_and_timer(false);
            }
        } else {
            self.link_handle_recoil_and_timer(false);
        }
        self.ram[LINK_Z_COORD + 1] = 0;
        self.replay_trace_player_state("recoil-exit");
    }

    pub(super) fn link_state_sleeping(&mut self) {
        match self.ram[PLAYER_SLEEP_IN_BED_STATE] {
            0 => {
                if self.ram[FRAME_COUNTER] & 0x1f == 0 {
                    self.ancilla_add_snoring(0x21, 1);
                }
            }
            1 => {
                if self.frame_control_view().submodule() == 0 {
                    self.ram[LINK_COUNTDOWN_FOR_DASH] =
                        self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_sub(1);
                    if (self.ram[LINK_COUNTDOWN_FOR_DASH] as i8).is_negative() {
                        self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
                        let input = (self.ram[FILTERED_JOYPAD_H] & 0xe0)
                            | (self.ram[FILTERED_JOYPAD_H] << 4)
                            | self.ram[FILTERED_JOYPAD_L];
                        if input & 0xf0 != 0 {
                            self.ram[LINK_POSE_DURING_OPENING] =
                                self.ram[LINK_POSE_DURING_OPENING].wrapping_add(1);
                            self.ram[LINK_DIRECTION_FACING] = 6;
                            self.ram[PLAYER_SLEEP_IN_BED_STATE] =
                                self.ram[PLAYER_SLEEP_IN_BED_STATE].wrapping_add(1);
                            self.ram[LINK_COUNTDOWN_FOR_DASH] = 4;
                        }
                    }
                }
            }
            2 => {
                self.ram[LINK_COUNTDOWN_FOR_DASH] =
                    self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_sub(1);
                if (self.ram[LINK_COUNTDOWN_FOR_DASH] as i8).is_negative() {
                    self.ram[LINK_ACTUAL_VEL_Y] = 4;
                    self.ram[LINK_ACTUAL_VEL_X] = 21;
                    self.ram[LINK_ACTUAL_VEL_Z] = 24;
                    self.ram[LINK_ACTUAL_VEL_Z_COPY] = 24;
                    self.ram[LINK_INCAPACITATED_TIMER] = 16;
                    self.ram[LINK_AUXILIARY_STATE] = 2;
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 6;
                }
            }
            _ => {}
        }
    }

    pub(super) fn link_state_zapped(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.LinkZap_HandleMosaic();

        let delay = self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = delay;
        if !(delay as i8).is_negative() {
            return;
        }

        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 2;
        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        if self.ram[PLAYER_HANDLER_TIMER] & 1 != 0 {
            self.palette_electro_themed_gear();
        } else {
            self.load_actual_gear_palettes();
        }
        if self.ram[PLAYER_HANDLER_TIMER] == 8 {
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.Player_SetCustomMosaicLevel(0);
        }
    }

    pub(super) fn link_state_exiting_dash(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.ram[JOYPAD1H_LAST] & 0x0f != 0 || self.ram[LINK_COUNTDOWN_FOR_DASH] >= 16 {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_IS_RUNNING] = 0;
            self.ram[SWIM_ACCELERATION_MODE] = 0;
            if self.ram[BUTTON_B_FRAMES] < 9 {
                self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            }
        } else {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_add(1);
        }
        self.link_handle_moving_animation_full_long_entry();
    }

    pub(super) fn reset_all_acceleration(&mut self) {
        for offset in [
            SWIM_SPEED_ACTIVE_FLAG,
            SWIM_SPEED_ACTIVE_FLAG + 2,
            SWIM_STROKE_FRAME_COUNTER,
            SWIM_STROKE_FRAME_COUNTER + 2,
            SWIM_ACCELERATION_MODE,
            SWIM_ACCELERATION_MODE + 2,
            SWIM_ACCELERATION,
            SWIM_ACCELERATION + 2,
            SWIM_MAX_SPEED,
            SWIM_MAX_SPEED + 2,
        ] {
            write_le_u16(&mut self.ram, offset, 0);
        }
    }

    pub(super) fn link_force_unequip_cape_quietly(&mut self) {
        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 32;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        self.ram[LINK_CAPE_MODE] = 0;
        self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 0;
    }

    pub(super) fn link_force_unequip_cape(&mut self) {
        self.ancilla_add_cape_poof(35, 4);
        self.ancilla_sfx2_near(21);
        self.link_force_unequip_cape_quietly();
    }

    pub(super) fn halt_link_when_using_items(&mut self) {
        if self.ram[DUNG_HDR_COLLISION_2] == 2
            && self.has_player_layer_collision(crate::ram::player::LAYER_COLLISION_BOTH)
        {
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_X_VEL] = 0;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_SUBPIXEL_Y] = 0;
            self.ram[LINK_SUBPIXEL_X] = 0;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        }
        if self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 0 {
            self.ram[LINK_DIRECTION] = 0;
        }
    }

    pub(super) fn link_handle_cape_passive_lift_check(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.ram[LINK_STATE_BITS] & 0x80 != 0
            || (self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
                && self.ram[LINK_GRABBING_WALL] != 0)
        {
            self.player_check_handle_cape_stuff();
        }
    }

    pub(super) fn player_check_handle_cape_stuff(&mut self) {
        const CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];
        if self.ram[LINK_CAPE_MODE] == 0 || self.ram[CURRENT_ITEM_ACTIVE] != 19 {
            return;
        }
        if self.ram[CURRENT_ITEM_ACTIVE] == self.ram[CURRENT_ITEM_Y] {
            self.ram[CAPE_DECREMENT_COUNTER] = self.ram[CAPE_DECREMENT_COUNTER].wrapping_sub(1);
            if self.ram[CAPE_DECREMENT_COUNTER] != 0 {
                return;
            }
            self.ram[CAPE_DECREMENT_COUNTER] =
                CAPE_DEPLETION_TIMERS[self.ram[LINK_MAGIC_CONSUMPTION] as usize];
            if self.ram[LINK_MAGIC_POWER] == 0 {
                return;
            }
            self.ram[LINK_MAGIC_POWER] = self.ram[LINK_MAGIC_POWER].wrapping_sub(1);
            if self.ram[LINK_MAGIC_POWER] != 0 {
                return;
            }
        }
        self.link_force_unequip_cape();
    }

    pub(super) fn check_y_button_press(&mut self) -> bool {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 != 0
            || self.ram[LINK_INCAPACITATED_TIMER] != 0
            || self.ram[FILTERED_JOYPAD_H] & 0x40 == 0
        {
            return false;
        }
        self.ram[BUTTON_MASK_B_Y] |= 0x40;
        true
    }

    pub(super) fn link_check_magic_cost(&mut self, item: u8) -> bool {
        const LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
            16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
        ];
        let idx = item as usize * 3 + self.ram[LINK_MAGIC_CONSUMPTION] as usize;
        let cost = LINK_ITEM_MAGIC_COSTS[idx];
        let new_magic = self.ram[LINK_MAGIC_POWER].wrapping_sub(cost);
        if self.ram[LINK_MAGIC_POWER] != 0 && new_magic < 0x80 {
            self.ram[LINK_MAGIC_POWER] = new_magic;
            return true;
        }
        if item != 3 {
            self.ancilla_sfx2_near(60);
            write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 123);
            self.main_show_text_message();
        }
        false
    }

    pub(super) fn refund_magic(&mut self, item: u8) {
        const LINK_ITEM_MAGIC_COSTS: [u8; 27] = [
            16, 8, 4, 32, 16, 8, 8, 4, 2, 8, 4, 2, 8, 4, 2, 16, 8, 4, 4, 2, 2, 8, 4, 2, 16, 8, 4,
        ];
        let idx = item as usize * 3 + self.ram[LINK_MAGIC_CONSUMPTION] as usize;
        let cost = LINK_ITEM_MAGIC_COSTS[idx];
        let mut new_magic = self.ram[LINK_MAGIC_POWER] as u16 + cost as u16;
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 && new_magic >= 128
        {
            new_magic = 128;
        }
        self.ram[LINK_MAGIC_POWER] = new_magic as u8;
    }

    pub(super) fn link_item_reset_from_overworld_things(&mut self) {
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
    }

    pub(super) fn link_item_cape(&mut self) {
        const CAPE_DEPLETION_TIMERS: [u8; 3] = [4, 8, 8];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if self.ram[LINK_CAPE_MODE] == 0 {
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] =
                self.ram[LINK_BUNNY_TRANSFORM_TIMER].wrapping_sub(1);
            if (self.ram[LINK_BUNNY_TRANSFORM_TIMER] as i8) >= 0 {
                self.ram[LINK_DIRECTION] &= !0x0f;
                self.halt_link_when_using_items();
                return;
            }

            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            if self.ram[LINK_MAGIC_POWER] == 0 {
                self.ancilla_sfx2_near(60);
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 123);
                self.main_show_text_message();
                return;
            }

            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_CAPE_MODE] = 1;
            self.ram[CAPE_DECREMENT_COUNTER] =
                CAPE_DEPLETION_TIMERS[self.ram[LINK_MAGIC_CONSUMPTION] as usize];
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 20;
            self.ancilla_add_cape_poof(35, 4);
            self.ancilla_sfx2_near(20);
            return;
        }

        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[CAPE_DECREMENT_COUNTER] = self.ram[CAPE_DECREMENT_COUNTER].wrapping_sub(1);
        if self.ram[CAPE_DECREMENT_COUNTER] == 0 {
            self.ram[CAPE_DECREMENT_COUNTER] =
                CAPE_DEPLETION_TIMERS[self.ram[LINK_MAGIC_CONSUMPTION] as usize];
            if self.ram[LINK_MAGIC_POWER] == 0
                && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
            {
                self.link_force_unequip_cape();
                return;
            }
            self.ram[LINK_MAGIC_POWER] = self.ram[LINK_MAGIC_POWER].wrapping_sub(1);
            if self.ram[LINK_MAGIC_POWER] == 0 {
                self.link_force_unequip_cape();
                return;
            }
        }

        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = self.ram[LINK_BUNNY_TRANSFORM_TIMER].wrapping_sub(1);
        if (self.ram[LINK_BUNNY_TRANSFORM_TIMER] as i8) < 0 {
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
            if self.ram[FILTERED_JOYPAD_H] & 0x40 != 0 {
                self.link_force_unequip_cape();
            }
        }
    }

    pub(super) fn link_item_rod(&mut self) {
        const ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(0) {
                self.ram[BUTTON_MASK_B_Y] &= !0x40;
                return;
            }
            self.ram[LINK_DEBUG_VALUE_2] = 1;
            if self.ram[EQ_SELECTED_ROD] == 1 {
                self.ancilla_add_fire_rod_shot(2, 1);
            } else {
                self.ancilla_add_ice_rod_shot(11, 1);
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = ROD_ANIM_DELAYS[0];
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 1;
        }
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }
        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        if step < ROD_ANIM_DELAYS.len() {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = ROD_ANIM_DELAYS[step];
            return;
        }
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_ITEM_IN_HAND] &= !1;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
    }

    pub(super) fn link_item_hammer(&mut self) {
        const HAMMER_ANIM_DELAYS: [u8; 3] = [3, 3, 16];
        if self.ram[LINK_ITEM_IN_HAND] & 0x10 != 0 {
            return;
        }
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || self.ram[FILTERED_JOYPAD_H] & 0x40 == 0 {
                return;
            }
            self.ram[BUTTON_MASK_B_Y] |= 0x40;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = HAMMER_ANIM_DELAYS[0];
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 2;
        }
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }
        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            HAMMER_ANIM_DELAYS[step.min(HAMMER_ANIM_DELAYS.len() - 1)];
        if self.ram[PLAYER_HANDLER_TIMER] == 1 {
            self.tile_detect_main_handler(3);
            self.ancilla_add_hit_stars(22, 0);
            if self.ram[SOUND_EFFECT_1] == 0 {
                self.ancilla_sfx2_near(16);
                self.spawn_hammer_water_splash();
            }
        } else if self.ram[PLAYER_HANDLER_TIMER] == 3 {
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[LINK_ITEM_IN_HAND] &= !2;
        }
    }

    pub(super) fn link_item_bow(&mut self) {
        const BOW_DELAYS: [u8; 3] = [3, 3, 8];
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = BOW_DELAYS[0];
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 16;
        }
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }
        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        if step < BOW_DELAYS.len() {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = BOW_DELAYS[step];
            return;
        }

        let k = self.ancilla_add_arrow(
            9,
            self.ram[LINK_DIRECTION_FACING],
            2,
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
        if k >= 0 {
            let k = k as usize;
            if self.ram[ARCHERY_GAME_ARROWS_LEFT] != 0 {
                self.ram[ARCHERY_GAME_ARROWS_LEFT] =
                    self.ram[ARCHERY_GAME_ARROWS_LEFT].wrapping_sub(1);
                self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_add(2);
            }
            if self.ram[ARCHERY_GAME_OUT_OF_ARROWS] == 0 && self.ram[LINK_NUM_ARROWS] != 0 {
                self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_sub(1);
                if self.ram[LINK_NUM_ARROWS] == 0 {
                    self.hud_refresh_icon();
                }
            } else {
                self.ram[ANCILLA_TYPE + k] = 0;
                self.ancilla_sfx2_near(60);
            }
        }

        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_ITEM_IN_HAND] &= !0x10;
        if self.ram[BUTTON_B_FRAMES] >= 9 {
            self.ram[BUTTON_B_FRAMES] = 9;
        }
    }

    pub(super) fn link_item_boomerang(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0
                || !self.check_y_button_press()
                || self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] != 0
            {
                return;
            }
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 0x80;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 7;
            let s0 = self.ancilla_add_boomerang(5, 0);
            if self.ram[BUTTON_B_FRAMES] >= 9 {
                self.link_reset_boomerang_y_stuff();
                return;
            }
            if s0 == 0 {
                self.ram[LINK_DIRECTION_LAST] = self.ram[JOYPAD1H_LAST] & 0x0f;
            } else {
                self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            }
        } else {
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        }

        if self.ram[LINK_ITEM_IN_HAND] != 0 {
            self.halt_link_when_using_items();
            self.ram[LINK_DIRECTION] &= !0x0f;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
                return;
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 5;
            self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
            if self.ram[PLAYER_HANDLER_TIMER] != 2 {
                return;
            }
        }
        self.link_reset_boomerang_y_stuff();
    }

    pub(super) fn link_reset_boomerang_y_stuff(&mut self) {
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        }
    }

    pub(super) fn link_handle_a_press(&mut self) {
        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] = 0;
        if self.ram[LINK_ITEM_IN_HAND] != 0
            || (self.ram[LINK_POSITION_MODE] & 0x1f) != 0
            || self.ram[PLAYER_POSE_DRAW_COUNTER] != 0
        {
            return;
        }
        if self.ram[BUTTON_B_FRAMES] < 9 && (self.ram[BUTTON_MASK_B_Y] & 0x80) != 0 {
            return;
        }

        let mut action = self.ram[TILE_ACTION_INDEX];
        if (self.ram[LINK_STATE_BITS] | self.ram[LINK_GRABBING_WALL]) == 0 {
            if !self.link_check_new_a_press() {
                self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
                return;
            }
            if self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] != 0
                && self.ram[LINK_DIRECTION_FACING] == 0
            {
                action = 7;
            } else if self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] != 0 {
                action = 6;
            } else {
                let mut attempt_action = false;
                if self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] == 0 {
                    if self.ram[FLAG_IS_SPRITE_TO_PICK_UP] == 0 {
                        action = self.link_handle_liftables();
                        attempt_action = true;
                    } else {
                        self.ram[FLAG_IS_SPRITE_TO_PICK_UP_CACHED] =
                            self.ram[FLAG_IS_SPRITE_TO_PICK_UP];
                    }
                }
                if !attempt_action {
                    if self.ram[BUTTON_B_FRAMES] != 0 {
                        self.link_reset_sword_and_item_usage();
                    }
                    if self.ram[LINK_ITEM_IN_HAND] != 0 || self.ram[LINK_POSITION_MODE] != 0 {
                        self.ram[LINK_ITEM_IN_HAND] = 0;
                        self.ram[LINK_POSITION_MODE] = 0;
                        self.link_reset_boomerang_y_stuff();
                        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
                        if self.ram[ANCILLA_TYPE] == 5 {
                            self.ram[ANCILLA_TYPE] = 0;
                        }
                    }
                    action = 1;
                }
            }

            const ABILITY_BITMASKS: [u8; 8] = [0xe0, 0x40, 4, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0];
            if action as usize >= ABILITY_BITMASKS.len()
                || (ABILITY_BITMASKS[action as usize] & self.ram[LINK_ABILITY_FLAGS]) == 0
            {
                self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
                return;
            }
            self.ram[TILE_ACTION_INDEX] = action;
            self.link_a_press_perform_basic(action.wrapping_mul(2));
        }

        self.ram[UNUSED_2] = self.ram[TILE_ACTION_INDEX];
        match self.ram[TILE_ACTION_INDEX] {
            1 => self.link_a_press_lift_carry_throw(),
            3 => self.link_a_press_pull_object(),
            6 => self.link_a_press_statue_drag(),
            _ => {}
        }
    }

    pub(super) fn link_a_press_perform_basic(&mut self, action_x2: u8) {
        match action_x2 >> 1 {
            0 => self.link_perform_desert_prayer(),
            1 => self.link_perform_throw(),
            2 => self.link_perform_dash(),
            3 => self.link_perform_grab(),
            4 => self.link_perform_read(),
            5 => self.link_perform_open_chest(),
            6 => self.link_perform_statue_drag(),
            7 => self.link_perform_rupee_pull(),
            // C Link_APress_PerformBasic asserts for action slots outside 0..=7.
            _ => panic!("Link_APress_PerformBasic action {}", action_x2 >> 1),
        }
    }

    pub(super) fn link_check_new_a_press(&mut self) -> bool {
        if self.ram[Y_BUTTON_ACTION_FLAGS] & 0x80 != 0
            || self.ram[LINK_INCAPACITATED_TIMER] != 0
            || self.ram[FILTERED_JOYPAD_L] & 0x80 == 0
        {
            return false;
        }
        self.ram[Y_BUTTON_ACTION_FLAGS] |= 0x80;
        true
    }

    pub(super) fn link_perform_dash(&mut self) {
        if self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 0
            || (self.ram[FLAG_IS_SPRITE_TO_PICK_UP] | self.ram[FLAG_IS_ANCILLA_TO_PICK_UP]) != 0
            || self.ram[LINK_STATE_BITS] & 0x80 != 0
        {
            return;
        }
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 29;
        self.ram[LINK_DASH_CTR] = 64;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 17;
        self.ram[LINK_IS_RUNNING] = 1;
        self.ram[BUTTON_MASK_B_Y] &= 0x80;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;

        const TAGALONG_ARR1: [u8; 15] = [0xff, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let follower = self.ram[FOLLOWER_INDICATOR] as usize;
        if self.ram[FOLLOWER_INDICATOR] == TAGALONG_ARR1[follower] {
            self.ram[LINK_SPEED_SETTING] = 0;
            write_le_u16(&mut self.ram, TIMER_TAGALONG_REACQUIRE, 64);
        }
    }

    pub(super) fn link_perform_grab(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x80 != 0 && self.ram[BUTTON_B_FRAMES] >= 9 {
            return;
        }
        self.ram[LINK_GRABBING_WALL] = 1;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_TIMER] = 0;
        self.ram[LINK_VAR30D] = 0;
    }

    pub(super) fn link_perform_read(&mut self) {
        let message = if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.Dungeon_GetTeleMsg(self.world_state_view().dungeon_room() as usize)
        } else if self.ram[SRAM_PROGRESS_INDICATOR] < 2 {
            0x003a
        } else {
            self.asset_u16(
                110,
                u16::from(self.world_state_view().overworld_screen()) as usize,
            )
        };
        write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, message);
        self.main_show_text_message();
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
    }

    pub(super) fn link_perform_open_chest(&mut self) {
        if self.ram[LINK_DIRECTION_FACING] != 0
            || self.ram[ITEM_RECEIPT_METHOD] != 0
            || self.ram[LINK_AUXILIARY_STATE] != 0
        {
            return;
        }

        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        let Some((mut item, chest_position)) =
            self.OpenChestForItemResult(read_le_u16(&self.ram, INDEX_OF_INTERACTING_TILE) as u8)
        else {
            self.ram[ITEM_RECEIPT_METHOD] = 0;
            return;
        };

        self.ram[ITEM_RECEIPT_METHOD] = 1;
        const RECEIVE_ITEM_ALTERNATES: [u8; 76] = [
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 68, 255, 255, 255, 255,
            255, 53, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 70, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255,
        ];
        if let Some(&alternate) = RECEIVE_ITEM_ALTERNATES.get(item as usize) {
            if alternate != 0xff {
                let ram_addr = player_memory_location_to_give_item_to(item);
                if self.ram[ram_addr] != 0 {
                    item = alternate;
                }
            }
        }

        self.link_receive_item(item, chest_position);
    }

    pub(super) fn link_perform_statue_drag(&mut self) {
        self.ram[LINK_GRABBING_WALL] = 2;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_TIMER] = 0;
        self.ram[LINK_VAR30D] = 0;
    }

    pub(super) fn link_perform_rupee_pull(&mut self) {
        if self.ram[LINK_DIRECTION_FACING] != 0 {
            return;
        }
        self.link_reset_properties_a();
        self.ram[LINK_GRABBING_WALL] = 2;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 2;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_TIMER] = 0;
        self.ram[LINK_VAR30D] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 29;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
    }

    pub(super) fn search_for_byrna_spark(&self) -> bool {
        if self.ram[LINK_POSITION_MODE] & 8 != 0 {
            return false;
        }
        (0..=4).rev().any(|i| self.ram[ANCILLA_TYPE + i] == 0x31)
    }

    pub(super) fn link_permission_for_slosh_sounds(&self) -> bool {
        if self.ram[LINK_DIRECTION] & 0x0f == 0 {
            return true;
        }
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 17 {
            self.ram[FRAME_COUNTER] & 0x0f != 0
        } else {
            self.ram[FRAME_COUNTER] & 0x07 != 0
        }
    }

    pub(super) fn link_a_press_lift_carry_throw(&mut self) {
        const LIFT_TAB0: [u8; 10] = [8, 24, 8, 24, 8, 32, 6, 8, 13, 13];
        const LIFT_TAB1: [u8; 10] = [0, 1, 0, 1, 0, 1, 0, 1, 2, 3];
        const LIFT_TAB2: [u8; 29] = [
            6, 7, 7, 5, 10, 0, 23, 0, 18, 0, 18, 0, 8, 0, 8, 0, 254, 255, 17, 0, 0x54, 0x52, 0x50,
            0xff, 0x51, 0x53, 0x55, 0x56, 0x57,
        ];
        if self.ram[LINK_STATE_BITS] == 0 {
            return;
        }
        if self.ram[LINK_PICKING_THROW_STATE] & 2 != 0 && self.ram[Y_BUTTON_ACTION_TIMER] >= 5 {
            self.ram[Y_BUTTON_ACTION_TIMER] = 5;
        }
        if self.ram[LINK_PICKING_THROW_STATE] != 0 {
            self.halt_link_when_using_items();
        }
        if self.ram[LINK_PICKING_THROW_STATE] & 1 != 0 {
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_FRAME_CHANGE_COUNTER] = 0;
            self.ram[LINK_DIRECTION] &= !0x0f;
        }
        self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
        if self.ram[Y_BUTTON_ACTION_TIMER] != 0 {
            return;
        }
        if self.ram[LINK_PICKING_THROW_STATE] & 2 != 0 {
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            if self.ram[LINK_PLAYER_HANDLER_STATE] == 24 {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            }
        } else if self.ram[PLAYER_HANDLER_TIMER] != 0 {
            if self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1) != 9 {
                self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
                let timer = self.ram[PLAYER_HANDLER_TIMER] as usize;
                self.ram[Y_BUTTON_ACTION_TIMER] = LIFT_TAB0[timer];
                self.ram[Y_BUTTON_ACTION_STEP] = LIFT_TAB1[timer];
                if self.ram[PLAYER_HANDLER_TIMER] == 6 {
                    self.ram[DUNG_SECRETS_UNK1_PLAYER] = 0;
                    let (what, x, y) = if self.ram[PLAYER_IS_INDOORS] != 0 {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                        (what, pt.x, pt.y)
                    } else {
                        let mut pt = Point16U { x: 0, y: 0 };
                        let what = self.Overworld_HandleLiftableTiles(&mut pt);
                        (what, pt.x, pt.y)
                    };
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 24;
                    self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 1;
                    self.sprite_spawn_throwable_terrain((what & 0x0f).wrapping_add(1), x, y);
                    self.ram[FILTERED_JOYPAD_L] &= !0x80;
                }
                return;
            }
        } else {
            if self.ram[Y_BUTTON_ACTION_STEP] as usize >= LIFT_TAB2.len() - 1 {
                return;
            }
            self.ram[Y_BUTTON_ACTION_STEP] = self.ram[Y_BUTTON_ACTION_STEP].wrapping_add(1);
            self.ram[Y_BUTTON_ACTION_TIMER] = LIFT_TAB2[self.ram[Y_BUTTON_ACTION_STEP] as usize];
            if self.ram[Y_BUTTON_ACTION_STEP] != 3 {
                return;
            }
        }
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
    }

    pub(super) fn link_a_press_pull_object(&mut self) {
        const GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];
        const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
        const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
        self.ram[LINK_DIRECTION] &= !0x0f;
        let facing = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        if GRAB_WALL_DIRS[facing] & self.ram[JOYPAD1H_LAST] == 0 {
            self.ram[LINK_VAR30D] = 0;
            let step = self.ram[LINK_VAR30D] as usize;
            self.ram[Y_BUTTON_ACTION_STEP] = GRAB_WALL_ANIM_STEPS[step];
            self.ram[Y_BUTTON_ACTION_TIMER] = GRAB_WALL_ANIM_TIMER[step];
        } else {
            self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
            if (self.ram[Y_BUTTON_ACTION_TIMER] as i8) < 0 {
                self.ram[LINK_VAR30D] = if self.ram[LINK_VAR30D].wrapping_add(1) == 7 {
                    1
                } else {
                    self.ram[LINK_VAR30D].wrapping_add(1)
                };
                let step = self.ram[LINK_VAR30D] as usize;
                self.ram[Y_BUTTON_ACTION_STEP] = GRAB_WALL_ANIM_STEPS[step];
                self.ram[Y_BUTTON_ACTION_TIMER] = GRAB_WALL_ANIM_TIMER[step];
            }
        }
        if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
            self.ram[LINK_VAR30D] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        }
    }

    pub(super) fn link_a_press_statue_drag(&mut self) {
        const GRAB_WALL_DIRS: [u8; 4] = [4, 8, 1, 2];
        const GRAB_WALL_ANIM_STEPS: [u8; 7] = [0, 1, 2, 3, 1, 2, 3];
        const GRAB_WALL_ANIM_TIMER: [u8; 7] = [0, 5, 5, 12, 5, 5, 12];
        self.ram[LINK_SPEED_SETTING] = 20;
        let j = self.ram[JOYPAD1H_LAST]
            & GRAB_WALL_DIRS[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize];
        if j == 0 {
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_X_VEL] = 0;
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_VAR30D] = 0;
        } else {
            self.ram[LINK_DIRECTION] = j;
            self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
            if (self.ram[Y_BUTTON_ACTION_TIMER] as i8) >= 0 {
                if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
                    self.link_a_press_statue_drag_release();
                }
                return;
            }
            self.ram[LINK_VAR30D] = if self.ram[LINK_VAR30D].wrapping_add(1) == 7 {
                1
            } else {
                self.ram[LINK_VAR30D].wrapping_add(1)
            };
        }
        let step = self.ram[LINK_VAR30D] as usize;
        self.ram[Y_BUTTON_ACTION_STEP] = GRAB_WALL_ANIM_STEPS[step];
        self.ram[Y_BUTTON_ACTION_TIMER] = GRAB_WALL_ANIM_TIMER[step];
        if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
            self.link_a_press_statue_drag_release();
        }
    }

    fn link_a_press_statue_drag_release(&mut self) {
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[LINK_IS_NEAR_MOVEABLE_STATUE] = 0;
        self.ram[LINK_VAR30D] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
    }

    pub(super) fn link_item_bombs(&mut self) {
        const FEATURES0_MORE_ACTIVE_BOMBS: u32 = 1 << 2;
        if self.ram[IS_STANDING_IN_DOORWAY] != 0
            || self.ram[FOLLOWER_INDICATOR] == 13
            || !self.check_y_button_press()
        {
            return;
        }
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        let limit = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MORE_ACTIVE_BOMBS != 0 {
            3
        } else {
            1
        };
        self.ancilla_add_bomb(7, limit);
        self.ram[LINK_ITEM_IN_HAND] = 0;
    }

    pub(super) fn link_item_book(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 != 0
            || self.ram[IS_STANDING_IN_DOORWAY] != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        if self.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] != 0 {
            self.link_perform_desert_prayer();
        } else {
            self.ancilla_sfx2_near(60);
        }
    }

    pub(super) fn link_item_bottle(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        let btidx = self.ram[LINK_ITEM_BOTTLE_INDEX].wrapping_sub(1) as usize;
        if btidx >= 4 {
            return;
        }
        let bottle = self.ram[LINK_BOTTLE_INFO + btidx];
        if bottle == 0 {
            return;
        }
        if bottle < 3 {
            self.ancilla_sfx2_near(60);
        } else if bottle == 3 {
            if self.ram[LINK_HEALTH_CAPACITY] == self.ram[LINK_HEALTH_CURRENT] {
                self.ancilla_sfx2_near(60);
                return;
            }
            self.ram[LINK_BOTTLE_INFO + btidx] = 2;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            let main_module = self.frame_control_view().main_module();
            self.frame_control_view_mut().set_submodule(4);
            self.ram[SAVED_MODULE_FOR_MENU] = main_module;
            self.frame_control_view_mut().set_main_module(14);
            self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] = 7;
            self.hud_rebuild();
        } else if bottle == 4 {
            if self.ram[LINK_MAGIC_POWER] == 128 {
                self.ancilla_sfx2_near(60);
                return;
            }
            self.ram[LINK_BOTTLE_INFO + btidx] = 2;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            let main_module = self.frame_control_view().main_module();
            self.frame_control_view_mut().set_submodule(8);
            self.ram[SAVED_MODULE_FOR_MENU] = main_module;
            self.frame_control_view_mut().set_main_module(14);
            self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] = 7;
            self.hud_rebuild();
        } else if bottle == 5 {
            if self.ram[LINK_HEALTH_CAPACITY] == self.ram[LINK_HEALTH_CURRENT]
                && self.ram[LINK_MAGIC_POWER] == 128
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            self.ram[LINK_BOTTLE_INFO + btidx] = 2;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            let main_module = self.frame_control_view().main_module();
            self.frame_control_view_mut().set_submodule(9);
            self.ram[SAVED_MODULE_FOR_MENU] = main_module;
            self.frame_control_view_mut().set_main_module(14);
            self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] = 7;
            self.hud_rebuild();
        } else if bottle == 6 {
            self.ram[LINK_ITEM_IN_HAND] = 0;
            if self.release_fairy() < 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            self.ram[LINK_BOTTLE_INFO + btidx] = 2;
            self.hud_rebuild();
        } else if bottle == 7 || bottle == 8 {
            if self.release_bee_from_bottle(btidx) == 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            self.ram[LINK_BOTTLE_INFO + btidx] = 2;
            self.hud_rebuild();
        }
    }

    pub(super) fn link_perform_desert_prayer(&mut self) {
        let main_module = self.frame_control_view().main_module();
        self.frame_control_view_mut().set_submodule(5);
        self.ram[SAVED_MODULE_FOR_MENU] = main_module;
        self.frame_control_view_mut().set_main_module(14);
        self.ram[FLAG_UNK1] = 1;
        self.ram[Y_BUTTON_ACTION_TIMER] = 22;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_STATE_BITS] = 2;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[SOUND_EFFECT_AMBIENT] = 17;
        self.ram[MUSIC_CONTROL] = 242;
    }

    pub(super) fn link_item_lamp(&mut self) {
        if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
            return;
        }
        if self.ram[LINK_ITEM_TORCH] != 0 && self.link_check_magic_cost(6) {
            self.ancilla_add_magic_powder(0x1a, 0);
            self.dungeon_light_torch();
            self.ancilla_add_lamp_flame(0x2f, 2);
        }
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
    }

    pub(super) fn link_item_powder(&mut self) {
        const MUSHROOM_TIMER: [u8; 10] = [2, 1, 1, 3, 2, 2, 2, 2, 6, 0];

        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            if self.ram[LINK_ITEM_MUSHROOM] != 2 {
                self.ancilla_sfx2_near(60);
                self.finish_powder_item();
                return;
            }
            if !self.link_check_magic_cost(2) {
                self.finish_powder_item();
                return;
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = MUSHROOM_TIMER[0];
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_DIRECTION] &= !0x0f;
            self.ram[LINK_ITEM_IN_HAND] = 0x40;
        }

        self.ram[LINK_X_VEL] = 0;
        self.ram[LINK_Y_VEL] = 0;
        self.ram[LINK_DIRECTION] = 0;
        self.ram[LINK_SUBPIXEL_X] = 0;
        self.ram[LINK_SUBPIXEL_Y] = 0;
        self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = MUSHROOM_TIMER[step];
        if self.ram[PLAYER_HANDLER_TIMER] == 4 {
            self.ancilla_add_magic_powder(26, 0);
        }
        if self.ram[PLAYER_HANDLER_TIMER] == 9 {
            if self.frame_control_view().submodule() == 0 {
                self.tile_detect_main_handler(1);
            }
            self.finish_powder_item();
        }
    }

    pub(super) fn link_item_shovel_and_flute(&mut self) {
        if self.ram[LINK_ITEM_FLUTE] == 1 {
            self.link_item_shovel();
        } else if self.ram[LINK_ITEM_FLUTE] != 0 {
            self.link_item_flute();
        }
    }

    pub(super) fn link_item_shovel(&mut self) {
        const SHOVEL_ANIM_DELAY: [u8; 6] = [7, 18, 16, 7, 18, 16];
        const SHOVEL_ANIM_DELAY2: [u8; 6] = [0, 1, 2, 0, 1, 2];
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = SHOVEL_ANIM_DELAY[0];
            self.ram[LINK_VAR30D] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_POSITION_MODE] = 1;
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }
        self.ram[LINK_VAR30D] = self.ram[LINK_VAR30D].wrapping_add(1);
        let step = self.ram[LINK_VAR30D] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = SHOVEL_ANIM_DELAY[step];
        self.ram[PLAYER_HANDLER_TIMER] = SHOVEL_ANIM_DELAY2[step];

        if self.ram[PLAYER_HANDLER_TIMER] == 1 {
            self.tile_detect_main_handler(2);
            if self.ram[OVERWORLD_HOLE_TILEMAP_POS] != 0 {
                self.ancilla_sfx3_near(27);
                self.ancilla_add_dug_up_flute(54, 0);
            }
            if (read_le_u16(&self.ram, TILEDETECT_THICK_GRASS)
                | read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH))
                & 1
                == 0
            {
                self.ancilla_add_hit_stars(22, 0);
                self.ancilla_sfx2_near(5);
            } else {
                self.ancilla_add_shovel_dirt(23, 0);
                if self.ram[IS_ARCHER_OR_SHOVEL_GAME] != 0 {
                    self.digging_game_guy_attempt_prize_spawn();
                }
                self.ancilla_sfx2_near(18);
            }
        }

        if self.ram[LINK_VAR30D] == 3 {
            self.ram[LINK_VAR30D] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[BUTTON_MASK_B_Y] &= 0x80;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        }
    }

    pub(super) fn link_item_flute(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 != 0 {
            self.ram[FLUTE_COUNTDOWN] = self.ram[FLUTE_COUNTDOWN].wrapping_sub(1);
            if self.ram[FLUTE_COUNTDOWN] != 0 {
                return;
            }
            self.ram[BUTTON_MASK_B_Y] &= !0x40;
        }
        if !self.check_y_button_press() {
            return;
        }
        self.ram[FLUTE_COUNTDOWN] = 128;
        self.ancilla_sfx2_near(19);
        if self.ram[PLAYER_IS_INDOORS] != 0
            || u16::from(self.world_state_view().overworld_screen()) & 0x40 != 0
            || self.frame_control_view().main_module() == 11
        {
            return;
        }
        if (0..5).any(|i| self.ram[ANCILLA_TYPE + i] == 0x27) {
            return;
        }
        if self.ram[LINK_ITEM_FLUTE] == 2 {
            let screen = u16::from(self.world_state_view().overworld_screen());
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            if screen == 0x18 && (0x760..0x7e0).contains(&y) && (0x1cf..0x230).contains(&x) {
                self.frame_control_view_mut().set_submodule(45);
                self.ancilla_add_exploding_weather_vane(55, 0);
            }
        } else {
            self.ancilla_add_duck_take_off(39, 4);
            self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        }
    }

    pub(super) fn link_handle_y_item(&mut self) {
        if self.ram[BUTTON_B_FRAMES] != 0 && self.ram[BUTTON_B_FRAMES] < 9 {
            return;
        }

        let mut item = self.ram[CURRENT_ITEM_Y];
        if self.ram[LINK_IS_BUNNY_MIRROR] != 0 && item != 11 && item != 20 {
            return;
        }

        if self.ram[IS_ARCHER_OR_SHOVEL_GAME] != 0 && self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
            if self.ram[IS_ARCHER_OR_SHOVEL_GAME] == 2 {
                self.link_item_bow();
            } else {
                self.link_item_shovel();
            }
            return;
        }

        let old_down = self.ram[JOYPAD1H_LAST];
        let old_pressed = self.ram[FILTERED_JOYPAD_H];
        let old_bottle = self.ram[LINK_ITEM_BOTTLE_INDEX];
        if (self.ram[LINK_ITEM_IN_HAND] | self.ram[LINK_POSITION_MODE]) == 0 && old_down & 0x40 == 0
        {
            let btn_index = self.get_current_item_button_index();
            if btn_index != 0 {
                let ptr = self.current_item_button_ptr(btn_index);
                let hud_item = self.ram[ptr];
                if hud_item != 0 {
                    if hud_item >= 21 {
                        self.ram[LINK_ITEM_BOTTLE_INDEX] = hud_item - 20;
                    }
                    item = self.hud_lookup_inventory_item(hud_item);
                    self.ram[JOYPAD1H_LAST] = old_down | 0x40;
                    const BUTTON_INDEX_KEYS: [u8; 4] = [0, 0x40, 0x20, 0x10];
                    if self.ram[FILTERED_JOYPAD_L] & BUTTON_INDEX_KEYS[btn_index] != 0 {
                        self.ram[FILTERED_JOYPAD_H] = old_pressed | 0x40;
                    }
                }
            }
        }

        if item != self.ram[CURRENT_ITEM_ACTIVE] {
            if self.ram[CURRENT_ITEM_ACTIVE] == 8 && self.ram[LINK_ITEM_FLUTE] & 2 != 0 {
                self.ram[BUTTON_MASK_B_Y] &= !0x40;
            }
            if self.ram[CURRENT_ITEM_ACTIVE] == 19 && self.ram[LINK_CAPE_MODE] != 0 {
                self.link_force_unequip_cape();
            }
        }

        if (self.ram[LINK_ITEM_IN_HAND] | self.ram[LINK_POSITION_MODE]) == 0 {
            self.ram[CURRENT_ITEM_ACTIVE] = item;
        }
        if matches!(self.ram[CURRENT_ITEM_ACTIVE], 5 | 6) {
            self.ram[EQ_SELECTED_ROD] = self.ram[CURRENT_ITEM_ACTIVE] - 4;
        }

        match self.ram[CURRENT_ITEM_ACTIVE] {
            0 => {}
            1 => self.link_item_bombs(),
            2 => self.link_item_boomerang(),
            3 => self.link_item_bow(),
            4 => self.link_item_hammer(),
            5 | 6 => self.link_item_rod(),
            7 => self.link_item_net(),
            8 => self.link_item_shovel_and_flute(),
            9 => self.link_item_lamp(),
            10 => self.link_item_powder(),
            11 => self.link_item_bottle(),
            12 => self.link_item_book(),
            13 => self.link_item_cane_of_byrna(),
            14 => self.link_item_hookshot(),
            15 => self.link_item_bombos(),
            16 => self.link_item_ether(),
            17 => self.link_item_quake(),
            18 => self.link_item_cane_of_somaria(),
            19 => self.link_item_cape(),
            20 => self.link_item_mirror(),
            21 => self.link_item_shovel(),
            // C Link_HandleYItem asserts outside item slots 0..=21.
            _ => panic!(
                "Link_HandleYItem current_item_active {}",
                self.ram[CURRENT_ITEM_ACTIVE]
            ),
        }

        self.ram[JOYPAD1H_LAST] = old_down;
        self.ram[FILTERED_JOYPAD_H] = old_pressed;
        self.ram[LINK_ITEM_BOTTLE_INDEX] = old_bottle;
    }

    fn current_item_button_ptr(&self, index: usize) -> usize {
        match index {
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            3 => HUD_CUR_ITEM_R,
            _ => HUD_CUR_ITEM,
        }
    }

    pub(super) fn link_item_ether(&mut self) {
        const ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];
        const ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];
        self.start_medallion_item(8, ETHER_ANIM_DELAYS[0], ETHER_ANIM_STATES[0], None);
    }

    pub(super) fn link_item_bombos(&mut self) {
        const BOMBOS_ANIM_DELAYS: [u8; 20] =
            [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];
        const BOMBOS_ANIM_STATES: [u8; 20] = [
            0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
        ];
        self.start_medallion_item(9, BOMBOS_ANIM_DELAYS[0], BOMBOS_ANIM_STATES[0], None);
    }

    pub(super) fn link_item_quake(&mut self) {
        const QUAKE_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];
        const QUAKE_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];
        self.start_medallion_item(10, QUAKE_ANIM_DELAYS[0], QUAKE_ANIM_STATES[0], Some(()));
    }

    pub(super) fn link_state_using_ether(&mut self) {
        const ETHER_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 3, 3];
        const ETHER_ANIM_DELAYS_NO_FLASH: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 24, 24];
        const ETHER_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7];
        const FEATURES0_DIM_FLASHES: u32 = 65536;

        self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] =
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK].wrapping_add(1);
        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 9 {
            self.ancilla_sfx2_near(44);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 12 {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 10;
        }

        let step = self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] as usize;
        let delays = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_DIM_FLASHES != 0 {
            ETHER_ANIM_DELAYS_NO_FLASH
        } else {
            ETHER_ANIM_DELAYS
        };
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = delays[step];
        self.ram[STATE_FOR_SPIN_ATTACK] = ETHER_ANIM_STATES[step];
        if self.ram[SPIN_ATTACK_SOUND_LATCH] == 0 && self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 10 {
            self.ram[SPIN_ATTACK_SOUND_LATCH] = 1;
            self.ancilla_add_ether_spell(24, 0);
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
        }
    }

    pub(super) fn link_state_using_bombos(&mut self) {
        const BOMBOS_ANIM_DELAYS: [u8; 20] =
            [5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 7, 1, 1, 1, 1, 1, 13];
        const BOMBOS_ANIM_STATES: [u8; 20] = [
            0, 1, 2, 3, 0, 1, 2, 3, 8, 9, 10, 11, 12, 10, 8, 13, 14, 15, 16, 17,
        ];

        self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] =
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK].wrapping_add(1);
        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 10 {
            self.ancilla_sfx2_near(44);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 20 {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 19;
        }
        let step = self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = BOMBOS_ANIM_DELAYS[step];
        self.ram[STATE_FOR_SPIN_ATTACK] = BOMBOS_ANIM_STATES[step];
        if self.ram[SPIN_ATTACK_SOUND_LATCH] == 0 && self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 19 {
            self.ram[SPIN_ATTACK_SOUND_LATCH] = 1;
            self.ancilla_add_bombos_spell(25, 0);
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
        }
    }

    pub(super) fn link_state_using_quake(&mut self) {
        const QUAKE_ANIM_DELAYS: [u8; 12] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 19];
        const QUAKE_ANIM_STATES: [u8; 12] = [0, 1, 2, 3, 0, 1, 2, 3, 18, 19, 20, 22];
        self.ram[FLAG_UNK1] = self.ram[FLAG_UNK1].wrapping_add(1);
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;

        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 10 {
            self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z_MIRROR];
            self.ram[LINK_ACTUAL_VEL_Z_COPY] = self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR];
            self.ram[LINK_Z_COORD] = self.ram[LINK_Z_COORD_MIRROR];
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.player_change_z(2);
            self.link_move_position();
            self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z];
            self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z_COPY];
            self.ram[LINK_Z_COORD_MIRROR] = self.ram[LINK_Z_COORD];
            if (self.ram[LINK_Z_COORD] as i8) >= 0 {
                self.ram[STATE_FOR_SPIN_ATTACK] = if (self.ram[LINK_ACTUAL_VEL_Z] as i8) < 0 {
                    21
                } else {
                    20
                };
                return;
            }
        } else {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
                return;
            }
        }

        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] =
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK].wrapping_add(1);
        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 4 {
            self.ancilla_sfx3_near(35);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 10 {
            self.ancilla_sfx2_near(44);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 11 {
            self.ancilla_sfx2_near(12);
        } else if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 12 {
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 11;
        }
        let step = self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = QUAKE_ANIM_DELAYS[step];
        self.ram[STATE_FOR_SPIN_ATTACK] = QUAKE_ANIM_STATES[step];
        if self.ram[SPIN_ATTACK_SOUND_LATCH] == 0 && self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 11 {
            self.ram[SPIN_ATTACK_SOUND_LATCH] = 1;
            self.ancilla_add_quake_spell(28, 0);
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
        }
    }

    pub(super) fn link_item_mirror(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if !self.check_y_button_press() {
                return;
            }
            if self.ram[FOLLOWER_INDICATOR] == 10 {
                write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 289);
                self.main_show_text_message();
                return;
            }
        }
        self.ram[BUTTON_MASK_B_Y] &= !0x40;

        const FEATURES0_MIRROR_TO_DARKWORLD: u32 = 8;
        if self.ram[IS_STANDING_IN_DOORWAY] != 0
            || (self.ram[CHEAT_WALK_THROUGH_WALLS] == 0
                && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MIRROR_TO_DARKWORLD == 0
                && self.ram[PLAYER_IS_INDOORS] == 0
                && u16::from(self.world_state_view().overworld_screen()) & 0x40 == 0)
        {
            self.ancilla_sfx2_near(60);
            return;
        }

        self.do_sword_interaction_with_tiles_mirror();
    }

    pub(super) fn do_sword_interaction_with_tiles_mirror(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            if self.ram[FLAG_BLOCK_LINK_MENU] != 0 {
                return;
            }
            self.Mirror_SaveRoomData();
            if self.ram[SOUND_EFFECT_1] != 60 {
                self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS] = 0;
                self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1] = 0;
            }
            return;
        }
        if self.frame_control_view().main_module() == 11 {
            return;
        }
        let screen = u16::from(self.world_state_view().overworld_screen());
        self.ram[LAST_LIGHT_VS_DARK_WORLD] = (screen & 0x40) as u8;
        if self.ram[LAST_LIGHT_VS_DARK_WORLD] != 0 {
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            self.ram[BIRD_TRAVEL_Y_LO + 15] = y as u8;
            self.ram[BIRD_TRAVEL_Y_HI + 15] = (y >> 8) as u8;
            self.ram[BIRD_TRAVEL_X_LO + 15] = x as u8;
            self.ram[BIRD_TRAVEL_X_HI + 15] = (x >> 8) as u8;
        }
        self.frame_control_view_mut().set_submodule(35);
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 1;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 20;
    }

    pub(super) fn link_state_crossing_worlds(&mut self) {
        self.link_reset_properties_b();
        self.tile_check_for_mirror_bonk();
        let world_changed = (u16::from(self.world_state_view().overworld_screen()) as u8 & 0x40)
            != self.ram[LAST_LIGHT_VS_DARK_WORLD];
        let bonk_bits = self.ram[R12] | self.ram[R14];
        if world_changed && bonk_bits & 0x0c != 0 && Self::bit_sum4(bonk_bits) >= 2 {
            self.start_mirror_transition(44);
            return;
        }

        if Self::bit_sum4(read_le_u16(&self.ram, TILEDETECT_DEEPWATER) as u8) >= 2 {
            if self.ram[LINK_ITEM_FLIPPERS] != 0 {
                self.link_set_to_deep_water();
                self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                self.link_force_unequip_cape_quietly();
                return;
            }
            if world_changed {
                self.start_mirror_transition(44);
                return;
            }
            self.check_ability_to_swim();
        }

        if self.ram[LINK_IS_IN_DEEP_WATER] != 0 {
            self.ram[LINK_IS_IN_DEEP_WATER] = 0;
            self.ram[LINK_DIRECTION_LAST] = self.ram[SWIM_PLAYER_DIRECTION_FLAGS];
        }
        self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
        self.ram[LINK_IS_RUNNING] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[SWIM_ACCELERATION_MODE] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        if world_changed {
            write_le_u16(&mut self.ram, NUM_MEMORIZED_TILES, 0);
        }
        self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[LINK_ITEM_MOON_PEARL] != 0
            || u16::from(self.world_state_view().overworld_screen()) & 0x40 == 0
        {
            0
        } else {
            23
        };
    }

    pub(super) fn handle_followers_after_mirroring(&mut self) {
        self.tile_detect_main_handler(0);
        self.ram[LINK_ANIMATION_STEPS] = 0;
        match self.ram[FOLLOWER_INDICATOR] {
            12 | 13 => {
                if self.ram[FOLLOWER_INDICATOR] == 13 {
                    self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0xfe;
                    self.ram[SUPER_BOMB_INDICATOR_COUNTER] = 0;
                }
                if self.ram[FOLLOWER_DROPPED] != 0 {
                    self.ram[FOLLOWER_DROPPED] = 0;
                    self.ram[FOLLOWER_INDICATOR] = 0;
                }
            }
            9 | 10 => self.ram[FOLLOWER_INDICATOR] = 0,
            7 | 8 => {
                self.ram[FOLLOWER_INDICATOR] ^= 7 ^ 8;
                self.load_follower_graphics();
                self.ancilla_add_dwarf_poof(0x40, 4);
            }
            _ => {}
        }

        if self.ram[LINK_ITEM_MOON_PEARL] == 0 {
            self.ancilla_add_bunny_poof(0x23, 4);
            self.link_force_unequip_cape_quietly();
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
        } else if self.ram[LINK_CAPE_MODE] != 0 {
            self.link_force_unequip_cape();
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 0;
        }
    }

    pub(super) fn link_item_hookshot(&mut self) {
        if self.ram[BUTTON_MASK_B_Y] & 0x40 != 0
            || self.ram[IS_STANDING_IN_DOORWAY] != 0
            || self.ram[PLAYER_DEFENSE_FLAGS] & 2 != 0
            || !self.check_y_button_press()
        {
            return;
        }

        self.reset_all_acceleration();
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 7;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_POSITION_MODE] = 4;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 19;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ancilla_add_hookshot(0x1f, 3);
    }

    pub(super) fn link_state_hookshotting(&mut self) {
        const HOOKSHOT_ARR_A: [i8; 4] = [-8, -16, 0, 0];
        const HOOKSHOT_ARR_B: [i8; 4] = [0, 0, 4, -12];
        const HOOKSHOT_ARR_C: [u8; 4] = [0xc0, 0x40, 0, 0];
        const HOOKSHOT_ARR_D: [u8; 4] = [0, 0, 0xc0, 0x40];

        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        let hookshot = (0..=4).rev().find(|&i| self.ram[ANCILLA_TYPE + i] == 0x1f);
        let Some(_k) = hookshot else {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
                return;
            }
            self.finish_hookshot_state();
            return;
        };

        if self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] != 0 {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
            if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) < 0 {
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            }
        }

        if self.ram[RELATED_TO_HOOKSHOT] == 0 {
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.ram[LINK_Y_COORD];
            self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.ram[LINK_X_COORD];
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_X_VEL] = 0;
            self.link_handle_cardinal_collision();
            return;
        }

        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;

        let hei = self.ram[HOOKSHOT_EFFECT_INDEX] as usize;
        self.ram[ANCILLA_ITEM_TO_LINK + hei] = self.ram[ANCILLA_ITEM_TO_LINK + hei].wrapping_sub(1);
        if (self.ram[ANCILLA_ITEM_TO_LINK + hei] as i8) < 0 {
            self.ram[ANCILLA_ITEM_TO_LINK + hei] = 0;
        } else {
            let dir = self.ram[ANCILLA_DIR + hei] as usize;
            let x =
                self.ram[ANCILLA_X_LO + hei] as u16 | ((self.ram[ANCILLA_X_HI + hei] as u16) << 8);
            let y =
                self.ram[ANCILLA_Y_LO + hei] as u16 | ((self.ram[ANCILLA_Y_HI + hei] as u16) << 8);
            let target_y = y.wrapping_add(HOOKSHOT_ARR_A[dir] as i16 as u16);
            let target_x = x.wrapping_add(HOOKSHOT_ARR_B[dir] as i16 as u16);
            self.ram[LINK_ACTUAL_VEL_X] = 0;
            self.ram[LINK_ACTUAL_VEL_Y] = 0;
            let yd = target_y.wrapping_sub(self.player_state_view().y()) as i16;
            if yd.wrapping_abs() >= 2 {
                self.ram[LINK_ACTUAL_VEL_Y] = HOOKSHOT_ARR_C[dir];
            }
            let xd = target_x.wrapping_sub(self.player_state_view().x()) as i16;
            if xd.wrapping_abs() >= 2 {
                self.ram[LINK_ACTUAL_VEL_X] = HOOKSHOT_ARR_D[dir];
            }
            if (self.ram[LINK_ACTUAL_VEL_X] | self.ram[LINK_ACTUAL_VEL_Y]) != 0 {
                self.continue_hookshot_drag();
                return;
            }
        }

        self.ram[ANCILLA_TYPE + hei] = 0;
        self.ram[TAGALONG_VAR7] = self.ram[TAGALONG_VAR1];
        self.finish_hookshot_state_without_button_clamp();

        if self.ram[ANCILLA_ARR1 + hei] != 0 {
            self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] ^= 1;
            self.ram[DUNG_CUR_FLOOR] = self.ram[DUNG_CUR_FLOOR].wrapping_sub(1);
            if self.ram[KIND_OF_IN_ROOM_STAIRCASE] == 0 {
                self.ram[DUNGEON_ROOM_INDEX2] = self.ram[DUNGEON_ROOM_INDEX];
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_add(0x10);
            }
            if self.ram[KIND_OF_IN_ROOM_STAIRCASE] != 2 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
            }
            self.Dungeon_FlagRoomData_Quadrants();
        }

        self.player_tile_detect_nearby();
        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 0x0f != 0
            && self.ram[LINK_IS_IN_DEEP_WATER] == 0
        {
            self.link_set_to_deep_water();
            self.ancilla_add_splash(21, 0);
            self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
            self.link_force_unequip_cape_quietly();
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            if self.ram[PLAYER_IS_INDOORS] != 0 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
            }
            if self.ram[BUTTON_B_FRAMES] >= 9 {
                self.ram[BUTTON_B_FRAMES] = 9;
            }
        } else if self.ram[TILEDETECT_PIT_TILE] & 0x0f != 0 {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
            self.ram[PLAYER_NEAR_PIT_STATE] = 1;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
            if self.ram[BUTTON_B_FRAMES] >= 9 {
                self.ram[BUTTON_B_FRAMES] = 9;
            }
        } else {
            let y = self.player_state_view().y();
            let x = self.player_state_view().x();
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
            self.ram[LINK_X_COORD_SAFE_RETURN_LO] = x as u8;
            self.ram[LINK_X_COORD_SAFE_RETURN_HI] = (x >> 8) as u8;
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
        }
    }

    fn continue_hookshot_drag(&mut self) {
        self.link_move_position();
        self.tile_detect_main_handler(5);
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            let x = (self.ram[TILEDETECT_VERTICAL_LEDGE] >> 4)
                | self.ram[TILEDETECT_VERTICAL_LEDGE]
                | self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ];
            if x & 1 != 0 {
                self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] =
                    self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER].wrapping_sub(1);
                if (self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] as i8) < 0 {
                    self.ram[HOOKSHOT_BG_CHECK_OFF_TIMER] = 3;
                    self.ram[RELATED_TO_HOOKSHOT] ^= 2;
                }
            }
        }
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        if self.ram[RELATED_TO_HOOKSHOT] & 2 == 0 {
            if self.ram[TILEDETECT_THICK_GRASS] & 1 != 0 {
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 2;
                if !self.link_permission_for_slosh_sounds() {
                    self.ancilla_sfx2_near(26);
                }
            } else if (self.ram[TILEDETECT_SHALLOW_WATER]
                | read_le_u16(&self.ram, TILEDETECT_DEEPWATER) as u8)
                & 1
                != 0
            {
                self.ram[DRAW_WATER_RIPPLES_OR_GRASS] =
                    self.ram[DRAW_WATER_RIPPLES_OR_GRASS].wrapping_add(1);
                self.ancilla_sfx2_near(if self.ram[OVERWORLD_SCREEN_INDEX] == 0x70 {
                    27
                } else {
                    28
                });
            }
        }
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_item_cane_of_somaria(&mut self) {
        const ROD_ANIM_DELAYS: [u8; 3] = [3, 3, 5];
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 0
                || self.ram[IS_STANDING_IN_DOORWAY] != 0
                || !self.check_y_button_press()
            {
                return;
            }

            let mut did_charge_magic = false;
            if !(0..5).any(|i| self.ram[ANCILLA_TYPE + i] == 0x2c) {
                if !self.link_check_magic_cost(4) {
                    if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
                        self.ram[BUTTON_MASK_B_Y] &= !0x40;
                    }
                    return;
                }
                did_charge_magic = true;
            }

            self.ram[LINK_DEBUG_VALUE_2] = 1;
            if self.ancilla_add_somaria_block(0x2c, 1).is_none() {
                if did_charge_magic
                    || self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES == 0
                {
                    self.refund_magic(4);
                }
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = ROD_ANIM_DELAYS[0];
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[LINK_POSITION_MODE] |= 8;
        }

        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        if step < ROD_ANIM_DELAYS.len() {
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = ROD_ANIM_DELAYS[step];
            return;
        }
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_POSITION_MODE] &= !8;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
    }

    pub(super) fn link_item_cane_of_byrna(&mut self) {
        const BYRNA_DELAYS: [u8; 4] = [19, 7, 13, 32];
        if self.search_for_byrna_spark() {
            return;
        }
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            if !self.link_check_magic_cost(8) {
                self.finish_byrna_item();
                return;
            }
            self.ancilla_add_cane_of_byrna_init_spark(0x30, 0);
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = BYRNA_DELAYS[0];
            self.ram[LINK_VAR30D] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_POSITION_MODE] = 8;
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }

        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[PLAYER_HANDLER_TIMER] = self.ram[PLAYER_HANDLER_TIMER].wrapping_add(1);
        let step = self.ram[PLAYER_HANDLER_TIMER] as usize;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = BYRNA_DELAYS[step];
        if self.ram[PLAYER_HANDLER_TIMER] == 1 {
            self.ancilla_sfx3_near(42);
        } else if self.ram[PLAYER_HANDLER_TIMER] == 3 {
            self.finish_byrna_item();
        }
    }

    pub(super) fn link_item_net(&mut self) {
        const BUG_NET_TIMERS: [u8; 40] = [
            11, 6, 7, 8, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 9, 4, 5, 6, 7, 8, 1, 2, 3,
            4, 10, 8, 1, 2, 3, 4, 5, 6, 7, 8,
        ];
        let base = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize * 10;
        if self.ram[BUTTON_MASK_B_Y] & 0x40 == 0 {
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 || !self.check_y_button_press() {
                return;
            }
            self.ram[PLAYER_HANDLER_TIMER] = BUG_NET_TIMERS[base];
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 3;
            self.ram[LINK_VAR30D] = 0;
            self.ram[LINK_POSITION_MODE] = 16;
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ancilla_sfx2_near(50);
        }

        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[LINK_VAR30D] = self.ram[LINK_VAR30D].wrapping_add(1);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 3;
        if self.ram[LINK_VAR30D] == 10 {
            self.ram[LINK_VAR30D] = 0;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[BUTTON_MASK_B_Y] &= 0x80;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[PLAYER_OAM_X_OFFSET] = 0x80;
            self.ram[PLAYER_OAM_Y_OFFSET] = 0x80;
            return;
        }

        let index = base + self.ram[LINK_VAR30D] as usize;
        self.ram[PLAYER_HANDLER_TIMER] = BUG_NET_TIMERS[index];
    }

    pub(super) fn ancilla_add_dug_up_flute(&mut self, ty: u8, limit: u8) {
        let Some(k) = self.ancilla_add_simple(ty, limit) else {
            return;
        };
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_Z + k] = 0;
        self.ram[ANCILLA_Z_VEL + k] = 24;
        self.ram[ANCILLA_X_VEL + k] = if self.ram[LINK_DIRECTION_FACING] == 4 {
            (-8i8) as u8
        } else {
            8
        };
        self.DecodeAnimatedSpriteTile_variable(12);
        self.ancilla_set_xy(k, 0x0490, 0x0a8a);
    }

    pub(super) fn ancilla_add_cane_of_byrna_init_spark(&mut self, ty: u8, limit: u8) {
        for k in (0..5).rev() {
            if self.ram[ANCILLA_TYPE + k] == 0x31 {
                self.ram[ANCILLA_TYPE + k] = 0;
            }
        }
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_AUX_TIMER + k] = 9;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[ANCILLA_ARR3 + k] = 2;
        }
    }

    pub(super) fn ancilla_add_shovel_dirt(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
            self.ram[ANCILLA_TIMER + k] = 20;
            self.ancilla_set_xy(
                k,
                self.player_state_view().x(),
                self.player_state_view().y(),
            );
        }
    }

    fn start_medallion_item(
        &mut self,
        player_state: u8,
        delay: u8,
        spin_state: u8,
        quake: Option<()>,
    ) {
        if !self.check_y_button_press() {
            return;
        }
        self.ram[BUTTON_MASK_B_Y] &= !0x40;

        let sword_ok = self.ram[LINK_SWORD_TYPE].wrapping_add(1) & !1 != 0;
        let blocked = self.ram[IS_STANDING_IN_DOORWAY] != 0
            || self.ram[FLAG_BLOCK_LINK_MENU] != 0
            || read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 != 0
            || !sword_ok
            || (self.ram[FOLLOWER_DROPPED] != 0 && self.ram[FOLLOWER_INDICATOR] == 13);
        if blocked {
            self.ancilla_sfx2_near(60);
            return;
        }

        if self.ram[ANCILLA_TYPE] | self.ram[ANCILLA_TYPE + 1] | self.ram[ANCILLA_TYPE + 2] != 0 {
            return;
        }
        if !self.link_check_magic_cost(1) {
            return;
        }

        self.ram[LINK_PLAYER_HANDLER_STATE] = player_state;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = delay;
        self.ram[STATE_FOR_SPIN_ATTACK] = spin_state;
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[SPIN_ATTACK_SOUND_LATCH] = 0;
        if quake.is_some() {
            self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = 40;
            self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = 40;
            self.ram[LINK_Z_COORD_MIRROR] = 0;
        }
        self.ancilla_sfx3_near(35);
    }

    fn start_mirror_transition(&mut self, submodule: u8) {
        self.frame_control_view_mut().set_submodule(submodule);
        self.ram[LINK_NEED_FOR_PULLFORRUPEES_SPRITE] = 0;
        self.ram[LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE] = 1;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 20;
    }

    pub(super) fn ancilla_add_hookshot(&mut self, a: u8, y: u8) {
        self.ancilla_add_hookshot_inner(a, y);
    }

    fn ancilla_add_hookshot_inner(&mut self, a: u8, y: u8) -> Option<usize> {
        const HOOKSHOT_Y_VEL: [u8; 4] = [0xc0, 0x40, 0, 0];
        const HOOKSHOT_X_VEL: [u8; 4] = [0, 0, 0xc0, 0x40];
        const HOOKSHOT_Y_DELTA: [i16; 4] = [4, 20, 8, 8];
        const HOOKSHOT_X_DELTA: [i16; 4] = [0, 0, -4, 11];

        let k = self.ancilla_add_simple(a, y)?;
        self.ram[ANCILLA_AUX_TIMER + k] = 3;
        self.ram[ANCILLA_ITEM_TO_LINK + k] = 0;
        self.ram[ANCILLA_STEP + k] = 0;
        self.ram[ANCILLA_L + k] = 0;
        self.ram[RELATED_TO_HOOKSHOT] = 0;
        self.ram[HOOKSHOT_EFFECT_INDEX] = k as u8;
        self.ram[ANCILLA_K + k] = 0;
        self.ram[ANCILLA_G + k] = 0xff;
        self.ram[ANCILLA_ARR1 + k] = 0;
        self.ram[ANCILLA_TIMER + k] = 0;
        let dir = self.ram[LINK_DIRECTION_FACING] >> 1;
        self.ram[ANCILLA_DIR + k] = dir;
        self.ram[ANCILLA_X_VEL + k] = HOOKSHOT_X_VEL[dir as usize];
        self.ram[ANCILLA_Y_VEL + k] = HOOKSHOT_Y_VEL[dir as usize];
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HOOKSHOT_X_DELTA[dir as usize] as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOOKSHOT_Y_DELTA[dir as usize] as u16);
        self.ancilla_set_xy(k, x, y);
        Some(k)
    }

    fn finish_hookshot_state(&mut self) {
        self.finish_hookshot_state_without_button_clamp();
        if self.ram[BUTTON_B_FRAMES] >= 9 {
            self.ram[BUTTON_B_FRAMES] = 9;
        }
    }

    fn finish_hookshot_state_without_button_clamp(&mut self) {
        self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[RELATED_TO_HOOKSHOT] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_POSITION_MODE] &= !4;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
    }

    fn finish_byrna_item(&mut self) {
        self.ram[LINK_VAR30D] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[BUTTON_MASK_B_Y] &= 0x80;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
    }

    fn finish_powder_item(&mut self) {
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[PLAYER_HANDLER_TIMER] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
    }

    pub(super) fn link_reset_sword_and_item_usage(&mut self) {
        self.ram[LINK_SPEED_SETTING] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x81;
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
    }
}

impl ZeldaState {
    pub(super) fn cache_camera_properties(&mut self) {
        copy_le_u16(&mut self.ram, BG2HOFS_COPY2_CACHED, BG2HOFS_COPY2);
        copy_le_u16(&mut self.ram, BG2VOFS_COPY2_CACHED, BG2VOFS_COPY2);
        copy_le_u16(&mut self.ram, LINK_Y_COORD_CACHED, LINK_Y_COORD);
        copy_le_u16(&mut self.ram, LINK_X_COORD_CACHED, LINK_X_COORD);
        copy_le_u16(
            &mut self.ram,
            ROOM_SCROLL_VARS_Y_VOFS1_CACHED,
            ROOM_BOUNDS_Y,
        );
        copy_le_u16(
            &mut self.ram,
            ROOM_SCROLL_VARS_Y_VOFS2_CACHED,
            ROOM_BOUNDS_Y + 4,
        );
        copy_le_u16(
            &mut self.ram,
            ROOM_SCROLL_VARS_X_VOFS1_CACHED,
            ROOM_BOUNDS_X,
        );
        copy_le_u16(
            &mut self.ram,
            ROOM_SCROLL_VARS_X_VOFS2_CACHED,
            ROOM_BOUNDS_X + 4,
        );
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_CACHED,
            UP_DOWN_SCROLL_TARGET,
        );
        copy_le_u16(
            &mut self.ram,
            UP_DOWN_SCROLL_TARGET_END_CACHED,
            UP_DOWN_SCROLL_TARGET_END,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_CACHED,
            LEFT_RIGHT_SCROLL_TARGET,
        );
        copy_le_u16(
            &mut self.ram,
            LEFT_RIGHT_SCROLL_TARGET_END_CACHED,
            LEFT_RIGHT_SCROLL_TARGET_END,
        );
        copy_le_u16(
            &mut self.ram,
            CAMERA_Y_COORD_SCROLL_LOW_CACHED,
            CAMERA_Y_COORD_SCROLL_LOW,
        );
        copy_le_u16(
            &mut self.ram,
            CAMERA_X_COORD_SCROLL_LOW_CACHED,
            CAMERA_X_COORD_SCROLL_LOW,
        );
        copy_le_u16(
            &mut self.ram,
            QUADRANT_FULLSIZE_X_CACHED,
            QUADRANT_FULLSIZE_X,
        );
        copy_le_u16(&mut self.ram, LINK_QUADRANT_X_CACHED, LINK_QUADRANT_X);
        self.ram[LINK_DIRECTION_FACING_CACHED] = self.ram[LINK_DIRECTION_FACING];
        self.ram[LINK_IS_ON_LOWER_LEVEL_CACHED] = self.ram[LINK_IS_ON_LOWER_LEVEL];
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR_CACHED] = self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR];
        self.ram[IS_STANDING_IN_DOORWAY_CACHED] = self.ram[IS_STANDING_IN_DOORWAY];
        self.ram[DUNG_CUR_FLOOR_CACHED] = self.ram[DUNG_CUR_FLOOR];
    }

    pub(super) fn link_main(&mut self) {
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        self.ram[FLAG_UNK1] = 0;
        if self.ram[FLAG_IS_LINK_IMMOBILIZED] == 0 {
            self.link_control_handler();
        }
        self.handle_somaria_and_graves();
    }

    pub(super) fn link_control_handler(&mut self) {
        if self.ram[LINK_GIVE_DAMAGE] != 0 {
            if self.ram[LINK_CAPE_MODE] != 0 {
                self.ram[LINK_GIVE_DAMAGE] = 0;
                self.ram[LINK_AUXILIARY_STATE] = 0;
                self.ram[LINK_INCAPACITATED_TIMER] = 0;
            } else if self.ram[LINK_DISABLE_SPRITE_DAMAGE] == 0 {
                let dmg = self.ram[LINK_GIVE_DAMAGE];
                self.ram[LINK_GIVE_DAMAGE] = 0;
                if self.ram[ANCILLA_TYPE] == 5
                    && self.ram[PLAYER_HANDLER_TIMER] == 0
                    && self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] != 0
                {
                    self.ram[ANCILLA_TYPE] = 0;
                    self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
                }
                if self.ram[COUNTDOWN_FOR_BLINK] == 0 {
                    self.ram[COUNTDOWN_FOR_BLINK] = 58;
                }
                self.ancilla_sfx2_near(38);
                self.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES] =
                    self.ram[NUMBER_OF_TIMES_HURT_BY_SPRITES].wrapping_add(1);
                let new_dmg = self.ram[LINK_HEALTH_CURRENT].wrapping_sub(dmg);
                let new_dmg = if new_dmg == 0 || new_dmg >= 0xa8 {
                    self.ram[MAPBAK_TM] = self.ram[TM_COPY];
                    self.ram[MAPBAK_TS] = self.ram[TS_COPY];
                    let main_module = self.frame_control_view().main_module();
                    self.ram[SAVED_MODULE_FOR_MENU] = main_module;
                    self.frame_control_view_mut().set_main_module(18);
                    self.frame_control_view_mut().set_submodule(1);
                    self.ram[COUNTDOWN_FOR_BLINK] = 0;
                    self.ram[LINK_HEARTS_FILLER] = 0;
                    0
                } else {
                    new_dmg
                };
                self.ram[LINK_HEALTH_CURRENT] = new_dmg;
            }
        }
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 0 {
            self.player_check_handle_cape_stuff();
        }
        match self.ram[LINK_PLAYER_HANDLER_STATE] {
            0 => self.link_state_default(),
            1 => self.link_state_pits(),
            2 | 6 => self.link_state_recoil(),
            3 | 30 => self.link_state_spin_attack(),
            4 => self.player_handler_04_swimming(),
            5 => self.link_state_on_ice(),
            7 => self.link_state_zapped(),
            8 => self.link_state_using_ether(),
            9 => self.link_state_using_bombos(),
            10 => self.link_state_using_quake(),
            11 => self.link_hop_hopping_south_ow(),
            12 => self.link_state_hopping_horizontally_ow(),
            13 => self.link_state_hopping_diagonally_up_ow(),
            14 => self.link_state_hopping_diagonally_down_ow(),
            15 | 16 => self.link_state_0_f(),
            17 => self.link_state_dashing(),
            18 => self.link_state_exiting_dash(),
            19 => self.link_state_hookshotting(),
            20 => self.link_state_crossing_worlds(),
            21 => self.player_handler_15_hold_item(),
            22 => self.link_state_sleeping(),
            23 => self.player_handler_17_bunny(),
            24 => self.link_state_holding_big_rock(),
            25 => self.link_state_receiving_ether(),
            26 => self.link_state_receiving_bombos(),
            27 => self.link_state_reading_desert_tablet(),
            28 => self.link_state_temporary_bunny(),
            29 => self.link_state_tree_pull(),
            _ => {}
        }
    }

    pub(super) fn link_state_default(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.ram[LINK_PLAYER_HANDLER_STATE] == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        self.ram[PIT_CORRECTION_TIMER] = 0;
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.handle_link_from1_d();
        } else {
            self.player_handler_00_ground_3();
        }
    }

    pub(super) fn handle_link_from1_d(&mut self) {
        self.ram[LINK_ITEM_IN_HAND] = 0;
        self.ram[LINK_POSITION_MODE] = 0;
        self.ram[LINK_DEBUG_VALUE_1] = 0;
        self.ram[LINK_DEBUG_VALUE_2] = 0;
        self.ram[LINK_VAR30D] = 0;
        self.ram[LINK_VAR30E] = 0;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
        self.ram[BUTTON_MASK_B_Y] &= !0x40;
        self.ram[LINK_STATE_BITS] = 0;
        self.ram[LINK_PICKING_THROW_STATE] = 0;
        self.ram[LINK_GRABBING_WALL] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] = 0;
        self.link_reset_swimming_state();
        self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
        self.ram[LINK_Z_COORD + 1] = 0;
        if self.ram[LINK_ELECTROCUTE_ON_TOUCH] != 0 {
            if self.ram[LINK_CAPE_MODE] != 0 {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[PLAYER_HANDLER_TIMER] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 2;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_DIRECTION] &= !0x0f;
            self.ancilla_sfx3_near(43);
            self.ram[LINK_PLAYER_HANDLER_STATE] = 7;
            self.link_state_zapped();
        } else {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 2;
            self.link_state_recoil();
        }
    }

    pub(super) fn link_state_0_f(&mut self) {
        // LinkState_0F is an assert-only unreachable state in the C port.
        panic!("LinkState_0F reached");
    }

    pub(super) fn player_handler_15_hold_item(&mut self) {}

    pub(super) fn link_handle_bunny_transformation(&mut self) -> bool {
        if read_le_u16(&self.ram, LINK_TIMER_TEMPBUNNY) == 0 {
            return false;
        }

        if self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] == 0 {
            if matches!(self.ram[LINK_PLAYER_HANDLER_STATE], 23 | 28) {
                write_le_u16(&mut self.ram, LINK_TIMER_TEMPBUNNY, 0);
                return false;
            }
            if self.ram[LINK_PICKING_THROW_STATE] & 2 != 0 {
                self.ram[LINK_STATE_BITS] = 0;
            }
            let preserved_lift_bit = self.ram[LINK_STATE_BITS] & 0x80;
            self.link_reset_properties_a();
            self.ram[LINK_STATE_BITS] = preserved_lift_bit;

            for i in 0..5 {
                if matches!(self.ram[ANCILLA_TYPE + i], 0x30 | 0x31) {
                    self.ram[ANCILLA_TYPE + i] = 0;
                }
            }
            self.link_cancel_dash();
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x14);
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 20;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 1;
            self.ram[LINK_VISIBILITY_STATUS] = 12;
        }

        self.ram[LINK_BUNNY_TRANSFORM_TIMER] = self.ram[LINK_BUNNY_TRANSFORM_TIMER].wrapping_sub(1);
        if (self.ram[LINK_BUNNY_TRANSFORM_TIMER] as i8).is_negative() {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 28;
            self.ram[LINK_IS_BUNNY_MIRROR] = 1;
            self.ram[LINK_IS_BUNNY] = 1;
            self.load_gear_palettes_bunny();
            self.ram[LINK_VISIBILITY_STATUS] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
        }
        true
    }

    pub(super) fn link_state_temporary_bunny(&mut self) {
        let timer = read_le_u16(&self.ram, LINK_TIMER_TEMPBUNNY);
        if timer == 0 {
            self.ancilla_add_cape_poof(0x23, 4);
            self.ancilla_sfx2_near(0x15);
            self.ram[LINK_BUNNY_TRANSFORM_TIMER] = 32;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.link_reset_properties_c();
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            self.ram[LINK_IS_BUNNY] = 0;
            self.ram[LINK_IS_BUNNY_MIRROR] = 0;
            self.load_actual_gear_palettes();
            self.ram[LINK_NEED_FOR_POOF_FOR_TRANSFORM] = 0;
            self.link_state_default();
        } else {
            write_le_u16(&mut self.ram, LINK_TIMER_TEMPBUNNY, timer.wrapping_sub(1));
            self.player_handler_17_bunny();
        }
    }

    pub(super) fn player_handler_17_bunny(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.ram[PIT_CORRECTION_TIMER] = 0;
        if self.ram[LINK_IS_IN_DEEP_WATER] == 0 {
            if self.ram[LINK_AUXILIARY_STATE] == 0 {
                self.link_temp_bunny_func2();
                return;
            }
            if self.ram[LINK_ITEM_MOON_PEARL] != 0 {
                self.ram[LINK_IS_BUNNY_MIRROR] = 0;
            }
        }
        self.link_state_bunny_recache();
    }

    pub(super) fn link_temp_bunny_func2(&mut self) {
        if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
            self.link_handle_recoil_and_timer(false);
            return;
        }
        self.player_state_view_mut().set_z(0xffff);
        self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
        self.ram[LINK_RECOILMODE_TIMER] = 0;
        if self.ram[LINK_FLAG_MOVING] != 0 {
            write_le_u16(&mut self.ram, SWIM_MAX_SPEED, 0x0180);
            write_le_u16(&mut self.ram, SWIM_MAX_SPEED + 2, 0x0180);
            self.link_handle_swim_movements();
            return;
        }

        self.reset_all_acceleration();
        self.link_handle_y_item();
        let mut dir = self.ram[FORCE_MOVE_ANY_DIRECTION] & 0x0f;
        if dir == 0 {
            dir = self.ram[JOYPAD1H_LAST] & 0x0f;
        }
        if dir == 0 {
            self.ram[LINK_X_VEL] = 0;
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_DIRECTION_LAST] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
            self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
        } else {
            self.ram[LINK_DIRECTION] = dir;
            if dir != self.ram[LINK_DIRECTION_LAST] {
                self.ram[LINK_DIRECTION_LAST] = dir;
                self.ram[LINK_SUBPIXEL_X] = 0;
                self.ram[LINK_SUBPIXEL_Y] = 0;
                self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
                self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
            }
        }
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_state_holding_big_rock(&mut self) {
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.ram[LINK_ITEM_IN_HAND] = 0;
            self.ram[LINK_POSITION_MODE] = 0;
            self.ram[LINK_DEBUG_VALUE_1] = 0;
            self.ram[LINK_DEBUG_VALUE_2] = 0;
            self.ram[LINK_VAR30D] = 0;
            self.ram[LINK_VAR30E] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[LINK_Z_COORD] = 0;
            if self.ram[LINK_ELECTROCUTE_ON_TOUCH] != 0 {
                self.link_reset_sword_and_item_usage();
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ram[PLAYER_HANDLER_TIMER] = 0;
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 2;
                self.ram[LINK_ANIMATION_STEPS] = 0;
                self.ram[LINK_DIRECTION] &= !0x0f;
                self.ancilla_sfx3_near(43);
                self.ram[LINK_PLAYER_HANDLER_STATE] = 7;
                self.link_state_zapped();
            } else {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 2;
                self.link_state_recoil();
            }
            return;
        }

        self.player_state_view_mut().set_z(0xffff);
        self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
        self.ram[LINK_RECOILMODE_TIMER] = 0;
        if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
            self.ram[LINK_VAR30D] = 0;
            self.ram[LINK_VAR30E] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
                self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        self.link_handle_a_press();
        let dir = self.ram[JOYPAD1H_LAST] & 0x0f;
        if dir == 0 {
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_X_VEL] = 0;
            self.ram[LINK_DIRECTION] = 0;
            self.ram[LINK_DIRECTION_LAST] = 0;
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] &= !9;
            self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
            self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
        } else {
            self.ram[LINK_DIRECTION] = dir;
            if dir != self.ram[LINK_DIRECTION_LAST] {
                self.ram[LINK_DIRECTION_LAST] = dir;
                self.ram[LINK_SUBPIXEL_X] = 0;
                self.ram[LINK_SUBPIXEL_Y] = 0;
                self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
                self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
            }
        }
        self.link_handle_moving_animation_full_long_entry();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn handle_somaria_and_graves(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0 && self.ram[LINK_SOMETHING_WITH_HOOKSHOT] != 0 {
            for i in (0..5).rev() {
                if self.ram[ANCILLA_TYPE + i] == 0x24 {
                    self.gravestone_move(i);
                }
            }
        }
        for i in (0..5).rev() {
            if self.ram[ANCILLA_TYPE + i] == 0x2c {
                self.somaria_block_handle_player_interaction(i);
                return;
            }
        }
    }

    pub(super) fn link_state_on_ice(&mut self) {
        // LinkState_OnIce is an assert-only unreachable state in the C port.
        panic!("LinkState_OnIce reached");
    }

    pub(super) fn link_state_receiving_ether(&mut self) {
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
        let frames = read_le_u16(&self.ram, BUTTON_B_FRAMES).wrapping_sub(1);
        write_le_u16(&mut self.ram, BUTTON_B_FRAMES, frames);
        if (frames as i16).is_negative() {
            write_le_u16(&mut self.ram, BUTTON_B_FRAMES, 0);
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        } else if frames == 0xbf {
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        } else if frames == 160 {
            let x = self.player_state_view().x();
            let y = self.player_state_view().y();
            self.player_state_view_mut().set_x(0x06b0);
            self.player_state_view_mut().set_y(0x0037);
            self.ancilla_add_ether_spell(0x18, 0);
            self.player_state_view_mut().set_x(x);
            self.player_state_view_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 0, 4);
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            self.ram[FLAG_BLOCK_LINK_MENU] = 0;
        }
    }

    pub(super) fn link_state_receiving_bombos(&mut self) {
        self.ram[LINK_AUXILIARY_STATE] = 0;
        self.ram[LINK_INCAPACITATED_TIMER] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
        let frames = read_le_u16(&self.ram, BUTTON_B_FRAMES).wrapping_sub(1);
        write_le_u16(&mut self.ram, BUTTON_B_FRAMES, frames);
        if (frames as i16).is_negative() {
            write_le_u16(&mut self.ram, BUTTON_B_FRAMES, 0);
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        } else if frames == 223 {
            self.ram[LINK_FORCE_HOLD_SWORD_UP] = 1;
        } else if frames == 160 {
            let x = self.player_state_view().x();
            let y = self.player_state_view().y();
            self.player_state_view_mut().set_x(0x0378);
            self.player_state_view_mut().set_y(0x0eb0);
            self.ancilla_add_bombos_spell(0x19, 0);
            self.player_state_view_mut().set_x(x);
            self.player_state_view_mut().set_y(y);
        } else if frames == 0 {
            self.ancilla_add_falling_prize(0x29, 5, 4);
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
        }
    }

    pub(super) fn ether_tablet_start_cutscene(&mut self) {
        write_le_u16(&mut self.ram, BUTTON_B_FRAMES, 0x00c0);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 25;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ram[FLAG_BLOCK_LINK_MENU] = 1;
    }

    pub(super) fn bombos_tablet_start_cutscene(&mut self) {
        write_le_u16(&mut self.ram, BUTTON_B_FRAMES, 0x00e0);
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 26;
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
        self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 1;
    }

    pub(super) fn link_state_reading_desert_tablet(&mut self) {
        self.ram[BUTTON_B_FRAMES] = self.ram[BUTTON_B_FRAMES].wrapping_sub(1);
        if self.ram[BUTTON_B_FRAMES] == 0 {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.link_perform_desert_prayer();
        }
    }

    pub(super) fn link_state_pits(&mut self) {
        self.ram[LINK_DIRECTION] = 0;
        if self.ram[PIT_CORRECTION_ACTIVE_FLAG] != 0 && {
            self.ram[PIT_CORRECTION_TIMER] = self.ram[PIT_CORRECTION_TIMER].wrapping_add(1);
            self.ram[PIT_CORRECTION_TIMER] == 0x20
        } {
            self.ram[PIT_CORRECTION_TIMER] = 31;
        } else {
            if self.ram[LINK_IS_RUNNING] == 0 {
                if self.ram[LINK_AUXILIARY_STATE] != 1 {
                    self.ram[LINK_DIRECTION] = self.ram[JOYPAD1H_LAST] & 0x0f;
                }
                self.link_state_pits_after_aux_state();
                return;
            }
            const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
            if self.ram[LINK_COUNTDOWN_FOR_DASH] != 0
                && (self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES == 0
                    || self.ram[JOYPAD1L_LAST] & 0x80 != 0)
            {
                self.link_state_dashing();
                return;
            }
            if self.ram[JOYPAD1H_LAST] & 0x0f != 0
                && (self.ram[JOYPAD1H_LAST] & 0x0f & self.ram[LINK_DIRECTION]) == 0
            {
                self.link_cancel_dash();
                if self.ram[LINK_AUXILIARY_STATE] != 1 {
                    self.ram[LINK_DIRECTION] = self.ram[JOYPAD1H_LAST] & 0x0f;
                }
            }
        }

        self.link_state_pits_after_aux_state();
    }

    pub(super) fn handle_dungeon_landing_from_pit(&mut self) {
        self.link_oam_main();
        copy_le_u16(&mut self.ram, LINK_X_COORD_PREV, LINK_X_COORD);
        copy_le_u16(&mut self.ram, LINK_Y_COORD_PREV, LINK_Y_COORD);
        if self.frame_control_view().submodule() == 7 {
            self.ram[LINK_VISIBILITY_STATUS] = 0;
        }
        if self.ram[FRAME_COUNTER] & 3 == 0 {
            self.ram[PLAYER_PIT_DATA_INDEX] = self.ram[PLAYER_PIT_DATA_INDEX].wrapping_add(1);
            if self.ram[PLAYER_PIT_DATA_INDEX] == 10 {
                self.ram[PLAYER_PIT_DATA_INDEX] = 6;
            }
        }
        self.ram[LINK_DIRECTION] = 4;
        self.link_handle_velocity();

        let link_y = self.player_state_view().y();
        let target_y = read_le_u16(&self.ram, TILEDETECT_WHICH_Y_POS);
        if (link_y as i16).is_negative() && !(target_y as i16).is_negative() {
            if (!link_y).wrapping_add(1).wrapping_add(target_y) < 0x8000 {
                return;
            }
        } else if target_y >= link_y {
            return;
        }

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
            self.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
        }
        self.player_state_view_mut().set_y(target_y);
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_SPEED_MODIFIER] = 0;
        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
        self.ram[PLAYER_NEAR_PIT_STATE] = 0;
        self.ram[LINK_SPEED_SETTING] = 0;
        self.frame_control_view_mut().set_subsubmodule(0);
        self.frame_control_view_mut().set_submodule(0);
        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        if self.ram[FOLLOWER_INDICATOR] != 0 && self.ram[FOLLOWER_INDICATOR] != 3 {
            self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = 0;
            if self.ram[FOLLOWER_INDICATOR] == 13 {
                self.ram[FOLLOWER_INDICATOR] = 0;
                self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0;
                self.ram[SUPER_BOMB_INDICATOR_COUNTER] = 0;
                self.ram[FOLLOWER_DROPPED] = 0;
            } else {
                self.follower_initialize();
            }
        }
        self.tile_detect_main_handler(0);
        if read_le_u16(&self.ram, TILEDETECT_SHALLOW_WATER) & 1 != 0 {
            self.ancilla_sfx2_near(0x24);
        }
        self.player_tile_detect_nearby();
        if self.ram[SOUND_EFFECT_1] & 0x3f != 0x24 {
            self.ancilla_sfx2_near(0x21);
        }

        if self.ram[DUNG_HDR_COLLISION_2] == 2
            && read_le_u16(&self.ram, TILEDETECT_WATER_STAIRCASE) & 0x0f != 0
        {
            self.ram[PLAYER_LAYER_COLLISION_FLAGS] = crate::ram::player::LAYER_COLLISION_BOTH;
        }
        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 0x0f == 0x0f {
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.link_reset_swimming_state();
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
            self.ancilla_add_splash(0x15, 1);
            self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
            self.link_force_unequip_cape_quietly();
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
        } else {
            self.ram[LINK_PLAYER_HANDLER_STATE] = if self.ram[TILEDETECT_PIT_TILE] & 0x0f != 0 {
                1
            } else {
                0
            };
        }
    }

    pub(super) fn link_state_spin_attack(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            for i in (0..5).rev() {
                if matches!(self.ram[ANCILLA_TYPE + i], 0x2a | 0x2b) {
                    self.ram[ANCILLA_TYPE + i] = 0;
                }
            }
            self.ram[LINK_Z_COORD + 1] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[BUTTON_MASK_B_Y] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[STATE_FOR_SPIN_ATTACK] = 0;
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            if self.ram[LINK_ELECTROCUTE_ON_TOUCH] != 0 {
                if self.ram[LINK_CAPE_MODE] != 0 {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ram[PLAYER_HANDLER_TIMER] = 0;
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 2;
                self.ram[LINK_ANIMATION_STEPS] = 0;
                self.ram[LINK_DIRECTION] &= !0x0f;
                self.ancilla_sfx3_near(43);
                self.ram[LINK_PLAYER_HANDLER_STATE] = 7;
                self.link_state_zapped();
            } else {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 2;
                self.link_state_recoil();
            }
            return;
        }

        if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
            self.link_handle_recoil_and_timer(false);
        } else {
            self.ram[LINK_DIRECTION] = 0;
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.ram[LINK_PLAYER_HANDLER_STATE] = 3;
            self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
            self.handle_indoor_camera_and_doors();
        }

        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8) >= 0 {
            return;
        }

        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] =
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK].wrapping_add(1);
        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 2 {
            self.ancilla_sfx3_near(35);
        }
        if self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] == 12 {
            self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[STATE_FOR_SPIN_ATTACK] = 0;
            self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
            if self.ram[LINK_PLAYER_HANDLER_STATE] != 30 {
                self.ram[BUTTON_MASK_B_Y] = if self.ram[BUTTON_B_FRAMES] != 0 {
                    self.ram[JOYPAD1H_LAST] & 0x80
                } else {
                    0
                };
            }
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        } else {
            let idx = self.ram[STEP_COUNTER_FOR_SPIN_ATTACK]
                .wrapping_add(self.ram[LINK_SPIN_OFFSETS]) as usize;
            self.ram[STATE_FOR_SPIN_ATTACK] = LINK_SPIN_GRAPHICS_BY_DIR[idx];
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                LINK_SPIN_DELAYS[self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] as usize];
            self.tile_detect_main_handler(8);
        }
    }

    pub(super) fn link_hop_hopping_south_ow(&mut self) {
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = 1;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        if self.ram[LINK_INCAPACITATED_TIMER] == 0 && self.ram[LINK_ACTUAL_VEL_Z_MIRROR] == 0 {
            self.ancilla_sfx2_near(32);
            self.link_hop_find_tile_to_land_on_south();
            if self.ram[PLAYER_IS_INDOORS] == 0 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 2;
            }
        }

        self.write_u16_ram(LINK_Z_COORD, self.read_u16_ram(LINK_Z_COORD_MIRROR));
        self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z_MIRROR];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR];
        self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z].wrapping_sub(2);
        self.link_move_position();

        if (self.ram[LINK_ACTUAL_VEL_Z] as i8).is_negative() {
            if self.ram[LINK_ACTUAL_VEL_Z] < 0xa0 {
                self.ram[LINK_ACTUAL_VEL_Z] = 0xa0;
            }
            if self.read_u16_ram(LINK_Z_COORD) >= 0xfff0 {
                self.write_u16_ram(LINK_Z_COORD, 0);
                self.link_splash_upon_landing();
                if self.ram[PLAYER_NEAR_PIT_STATE] != 0 {
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
                }
                if self.ram[LINK_PLAYER_HANDLER_STATE] != 4
                    && self.ram[LINK_PLAYER_HANDLER_STATE] != 1
                    && self.ram[LINK_IS_IN_DEEP_WATER] == 0
                {
                    self.ancilla_sfx2_near(33);
                }
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                self.ram[ALLOW_SCROLL_Z] = 0;
                self.ram[LINK_AUXILIARY_STATE] = 0;
                self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
                self.write_u16_ram(LINK_Z_COORD, 0xffff);
                self.ram[LINK_INCAPACITATED_TIMER] = 0;
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
                }
            } else {
                self.ram[LINK_Y_VEL] =
                    self.ram[LINK_Z_COORD_MIRROR].wrapping_sub(self.ram[LINK_Z_COORD]);
            }
        } else {
            self.ram[LINK_Y_VEL] =
                self.ram[LINK_Z_COORD_MIRROR].wrapping_sub(self.ram[LINK_Z_COORD]);
        }
        self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z];
        self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z_COPY];
        self.write_u16_ram(LINK_Z_COORD_MIRROR, self.read_u16_ram(LINK_Z_COORD));
    }

    pub(super) fn link_hop_find_tile_to_land_on_south(&mut self) {
        let original_y = self.player_state_view().y();
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, original_y);
        self.ram[LINK_Y_VEL] =
            self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);
        loop {
            let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);
            let terrain = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES)
                | read_le_u16(&self.ram, TILEDETECT_PIT_TILE)
                | read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
                | read_le_u16(&self.ram, TILEDETECT_THICK_GRASS)
                | read_le_u16(&self.ram, TILEDETECT_DEEPWATER);
            if terrain & 7 == 7 {
                break;
            }
        }
        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 7 != 0 {
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            if self.ram[LINK_AUXILIARY_STATE] != 4 {
                self.ram[LINK_AUXILIARY_STATE] = 2;
            }
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.link_reset_swimming_state();
            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
        }
        if read_le_u16(&self.ram, TILEDETECT_PIT_TILE) & 7 != 0 {
            self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
            self.ram[PLAYER_PIT_DATA_INDEX] = 0;
            self.ram[PLAYER_NEAR_PIT_STATE] = 1;
        }
        let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.ram[LINK_INCAPACITATED_TIMER] = 1;
        let mut z = self.ram[LINK_Z_COORD];
        if z >= 0xf0 {
            z = 0;
        }
        let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
        self.write_u16_ram(LINK_Z_COORD_MIRROR, z);
        self.write_u16_ram(LINK_Z_COORD, z);
    }

    pub(super) fn link_state_hopping_horizontally_ow(&mut self) {
        self.ram[LINK_DIRECTION] = if (self.ram[LINK_ACTUAL_VEL_X] as i8).is_negative() {
            6
        } else {
            5
        };
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        self.link_state_handling_jump();
    }

    pub(super) fn link_hopping_horizontally_find_tile_y(&mut self) {
        let original_y = self.player_state_view().y();
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, original_y);
        self.ram[LINK_Y_VEL] =
            self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);

        let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.tile_detect_movement_y(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);

        let terrain = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES)
            | read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
            | read_le_u16(&self.ram, TILEDETECT_THICK_GRASS)
            | read_le_u16(&self.ram, TILEDETECT_DEEPWATER);

        if terrain & 7 != 7 {
            self.player_state_view_mut().set_y(original_y);
            self.ram[LINK_INCAPACITATED_TIMER] = 1;

            let org_velx = self.ram[LINK_ACTUAL_VEL_X];
            let mut velx = org_velx as i8;
            if velx < 0 {
                velx = velx.wrapping_neg();
            }
            let idx = ((velx as u8) >> 4) as usize;
            self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = HOP_HORIZ_VEL_Z[idx];
            self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = HOP_HORIZ_VEL_Z[idx];
            let mut xt = HOP_HORIZ_VEL_X[idx];
            if (org_velx as i8) < 0 {
                xt = 0u8.wrapping_sub(xt);
            }
            self.ram[LINK_ACTUAL_VEL_X] = xt;
        } else {
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
            self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
            self.ram[LINK_INCAPACITATED_TIMER] = 1;
            let mut z = self.ram[LINK_Z_COORD];
            if z == 255 {
                z = 0;
            }
            let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
            self.write_u16_ram(LINK_Z_COORD_MIRROR, z);
            self.write_u16_ram(LINK_Z_COORD, z);
        }

        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 7 != 0 {
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.link_set_to_deep_water();
        }
    }

    pub(super) fn link_hopping_horizontally_find_tile_x(&mut self, o: u8) -> u8 {
        assert!(o == 0 || o == 2);
        let original_x = self.player_state_view().x();
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, original_x);
        let table_idx = (o >> 1) as usize;
        let mut i: i16 = 7;
        loop {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(HOP_HORIZ_X_STEP[table_idx] as i16 as u16);
            self.player_state_view_mut().set_x(x);
            self.tile_detect_movement_x(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);

            let terrain = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES)
                | read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH)
                | read_le_u16(&self.ram, TILEDETECT_THICK_GRASS)
                | read_le_u16(&self.ram, TILEDETECT_DEEPWATER)
                | read_le_u16(&self.ram, TILEDETECT_PIT_TILE);

            if terrain & 7 == 7 {
                if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 7 == 7 {
                    self.ram[LINK_IS_IN_DEEP_WATER] = 1;
                    self.ram[LINK_AUXILIARY_STATE] = 2;
                    self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
                    self.ram[SWIMMING_COUNTDOWN] = 0;
                    self.ram[LINK_SPEED_SETTING] = 0;
                    self.ram[LINK_GRABBING_WALL] = 0;
                    self.reset_all_acceleration();
                }
                break;
            }
            i -= 1;
            if i < 0 {
                let x = original_x.wrapping_add(HOP_HORIZ_X_FALLBACK[table_idx] as i16 as u16);
                self.player_state_view_mut().set_x(x);
                break;
            }
        }

        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HOP_HORIZ_X_FINAL[table_idx] as i16 as u16);
        self.player_state_view_mut().set_x(x);
        let distance = original_x.wrapping_sub(x) as i16;
        let distance = if distance < 0 { -distance } else { distance };
        let idx = (distance as u16 >> 3) as usize;
        let mut velx = HOP_HORIZ_X_VEL[idx];
        if o != 2 {
            velx = 0u8.wrapping_sub(velx);
        }
        self.ram[LINK_ACTUAL_VEL_X] = velx;
        self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = HOP_HORIZ_Z_VEL[idx];
        self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = HOP_HORIZ_Z_VEL[idx];
        i as u8
    }

    pub(super) fn link_state_hopping_diagonally_up_ow(&mut self) {
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        self.player_change_z(2);
        self.link_move_position();
        if (self.ram[LINK_Z_COORD] as i8).is_negative() {
            self.link_splash_upon_landing();
            if self.ram[LINK_PLAYER_HANDLER_STATE] != 4 && self.ram[LINK_IS_IN_DEEP_WATER] == 0 {
                self.ancilla_sfx2_near(33);
            }
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_AUXILIARY_STATE] = 0;
            self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
            self.player_state_view_mut().set_z(0xffff);
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        }
    }

    pub(super) fn link_state_hopping_diagonally_down_ow(&mut self) {
        let dir = if (self.ram[LINK_ACTUAL_VEL_X] as i8).is_negative() {
            2
        } else {
            3
        };
        self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = dir;
        self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        if self.ram[LINK_INCAPACITATED_TIMER] == 0 && self.ram[LINK_ACTUAL_VEL_Z_MIRROR] == 0 {
            self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] = 1;
            let old_x = self.player_state_view().x();
            self.ancilla_sfx2_near(32);
            self.link_hop_find_landing_spot_diagonally_down();
            self.player_state_view_mut().set_x(old_x);

            let distance = self
                .player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, LINK_Y_COORD_ORIGINAL));
            let idx = ((distance >> 3) as usize).min(23);
            let mut velx = LEDGE_DOWN_X_VEL[idx];
            if dir == 2 {
                velx = 0u8.wrapping_sub(velx);
            }
            self.ram[LINK_ACTUAL_VEL_X] = velx;
            if self.ram[PLAYER_IS_INDOORS] == 0 {
                self.ram[LINK_IS_ON_LOWER_LEVEL] = 2;
            }
        }
        self.link_state_handling_jump();
    }

    pub(super) fn link_hop_find_landing_spot_diagonally_down(&mut self) {
        let original_y = self.player_state_view().y();
        write_le_u16(&mut self.ram, LINK_Y_COORD_ORIGINAL, original_y);
        self.ram[LINK_Y_VEL] =
            self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);

        let scratch = loop {
            let o = if (self.ram[LINK_ACTUAL_VEL_X] as i8).is_negative() {
                0
            } else {
                1
            };
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(LEDGE_DIAG_DX[o] as i16 as u16);
            self.player_state_view_mut().set_x(x);
            let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(LEDGE_DIAG_DY[dir] as i16 as u16);
            self.player_state_view_mut().set_y(y);
            self.tile_detect_movement_y(self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as u16);
            let scratch = LEDGE_DIAG_BITS[o];
            let terrain = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES)
                | self.ram[TILEDETECT_DESTRUCTION_AFTERMATH] as u16
                | self.ram[TILEDETECT_THICK_GRASS] as u16
                | read_le_u16(&self.ram, TILEDETECT_DEEPWATER);
            if terrain & scratch as u16 == scratch as u16 {
                break scratch;
            }
        };

        if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & scratch as u16 != 0 {
            self.ram[LINK_IS_IN_DEEP_WATER] = 1;
            self.ram[LINK_AUXILIARY_STATE] = 2;
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = self.ram[LINK_DIRECTION_LAST];
            self.link_reset_swimming_state();
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
        }

        let dir = self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] as usize;
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(LEDGE_DIAG_DY2[dir] as i16 as u16);
        self.player_state_view_mut().set_y(y);
        self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = y as u8;
        self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = (y >> 8) as u8;
        self.ram[LINK_INCAPACITATED_TIMER] = 1;
        let z = y
            .wrapping_sub(original_y)
            .wrapping_add(self.ram[LINK_Z_COORD] as u16);
        self.write_u16_ram(LINK_Z_COORD_MIRROR, z);
        self.write_u16_ram(LINK_Z_COORD, z);
    }

    pub(super) fn link_state_handling_jump(&mut self) {
        self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z_MIRROR];
        self.ram[LINK_ACTUAL_VEL_Z_COPY] = self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR];
        self.ram[LINK_Z_COORD] = self.ram[LINK_Z_COORD_MIRROR];
        self.ram[LINK_ACTUAL_VEL_Z] = self.ram[LINK_ACTUAL_VEL_Z].wrapping_sub(2);
        self.link_move_position();
        if (self.ram[LINK_ACTUAL_VEL_Z] as i8).is_negative() {
            if self.ram[LINK_ACTUAL_VEL_Z] < 0xa0 {
                self.ram[LINK_ACTUAL_VEL_Z] = 0xa0;
            }
            if self.ram[LINK_Z_COORD] >= 0xf0 {
                self.player_state_view_mut().set_z(0);
                let mut falling_into_pit = false;
                if matches!(self.ram[LINK_PLAYER_HANDLER_STATE], 12 | 14) {
                    self.tile_detect_main_handler(0);
                    if self.ram[TILEDETECT_DEEPWATER] & 1 != 0 {
                        self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                        self.link_set_to_deep_water();
                        self.link_reset_sword_and_item_usage();
                        self.ancilla_add_splash(21, 0);
                    } else if self.ram[TILEDETECT_PIT_TILE] & 1 != 0 {
                        self.ram[LINK_SPRITE_OAM_STATE_TIMER] = 9;
                        self.ram[PLAYER_PIT_DATA_INDEX] = 0;
                        self.ram[PLAYER_NEAR_PIT_STATE] = 1;
                        self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
                        falling_into_pit = true;
                    }
                }
                if !falling_into_pit {
                    self.link_splash_upon_landing();
                    if self.ram[LINK_PLAYER_HANDLER_STATE] != 4
                        && self.ram[LINK_IS_IN_DEEP_WATER] == 0
                    {
                        self.ancilla_sfx2_near(33);
                    }
                }
                if self.ram[LINK_PLAYER_HANDLER_STATE] != 4 || self.ram[LINK_IS_BUNNY_MIRROR] == 0 {
                    self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                }
                self.ram[ALLOW_SCROLL_Z] = 0;
                self.ram[LINK_AUXILIARY_STATE] = 0;
                self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
                self.player_state_view_mut().set_z(0xffff);
                self.ram[LINK_INCAPACITATED_TIMER] = 0;
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
                }
            } else {
                self.ram[LINK_Y_VEL] =
                    self.ram[LINK_Z_COORD_MIRROR].wrapping_sub(self.ram[LINK_Z_COORD]);
            }
        } else {
            self.ram[LINK_Y_VEL] =
                self.ram[LINK_Z_COORD_MIRROR].wrapping_sub(self.ram[LINK_Z_COORD]);
        }
        self.ram[LINK_ACTUAL_VEL_Z_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z];
        self.ram[LINK_ACTUAL_VEL_Z_COPY_MIRROR] = self.ram[LINK_ACTUAL_VEL_Z_COPY];
        self.ram[LINK_Z_COORD_MIRROR] = self.ram[LINK_Z_COORD];
    }

    pub(super) fn link_state_dashing(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.ram[LINK_PLAYER_HANDLER_STATE] == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if self.ram[LINK_IS_RUNNING] == 0 {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            return;
        }
        if self.ram[BUTTON_MASK_B_Y] & 0x80 != 0 && self.ram[BUTTON_B_FRAMES] >= 9 {
            self.ram[BUTTON_B_FRAMES] = 9;
        }
        self.ram[PIT_CORRECTION_TIMER] = 0;

        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
            self.ram[LINK_SPEED_SETTING] = 0;
            self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
            self.ram[LINK_IS_RUNNING] = 0;
            self.ram[PLAYER_DEFENSE_FLAGS] = 0;
            if self.ram[LINK_ELECTROCUTE_ON_TOUCH] != 0 {
                if self.ram[LINK_CAPE_MODE] != 0 {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.ram[PLAYER_HANDLER_TIMER] = 0;
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 2;
                self.ram[LINK_ANIMATION_STEPS] = 0;
                self.ram[LINK_DIRECTION] &= !0x0f;
                self.ancilla_sfx3_near(43);
                self.ram[LINK_PLAYER_HANDLER_STATE] = 7;
                self.link_state_zapped();
            } else {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 2;
                self.link_state_recoil();
            }
            return;
        }

        const DASH_TAB1: [u8; 3] = [7, 15, 15];
        const DASH_TAB2: [u8; 4] = [8, 4, 2, 1];
        let mut a = self.ram[LINK_COUNTDOWN_FOR_DASH];
        if a == 0 {
            a = self.ram[INDEX_OF_DASHING_SFX];
            self.ram[INDEX_OF_DASHING_SFX] = self.ram[INDEX_OF_DASHING_SFX].wrapping_sub(1);
        }
        if DASH_TAB1[(self.ram[LINK_COUNTDOWN_FOR_DASH] >> 4) as usize] & a == 0 {
            self.ancilla_sfx2_near(35);
        }
        self.ram[LINK_COUNTDOWN_FOR_DASH] = self.ram[LINK_COUNTDOWN_FOR_DASH].wrapping_sub(1);
        if (self.ram[LINK_COUNTDOWN_FOR_DASH] as i8).is_negative() {
            self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
            let follower = self.ram[FOLLOWER_INDICATOR] as usize;
            if self.ram[FOLLOWER_INDICATOR] == TAGALONG_ARR1[follower] {
                self.ram[FOLLOWER_INDICATOR] = TAGALONG_ARR2[follower];
            }
        } else {
            self.ram[INDEX_OF_DASHING_SFX] = 0;
            if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
                self.ram[LINK_ANIMATION_STEPS] = 0;
                self.ram[LINK_COUNTDOWN_FOR_DASH] = 0;
                self.ram[LINK_SPEED_SETTING] = 0;
                self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
                self.ram[LINK_IS_RUNNING] = 0;
                if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
                    self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
                }
                return;
            }
            self.ancilla_add_dash_dust_charging(30, 0);
            self.ram[LINK_X_VEL] = 0;
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_DASH_CTR] = 64;
            self.ram[LINK_SPEED_SETTING] = 16;
            let mut dir = self.ram[JOYPAD1H_LAST] & 0x0f;
            if self.ram[BUTTON_MASK_B_Y] & 0x80 != 0
                || self.ram[IS_STANDING_IN_DOORWAY] != 0
                || dir == 0
            {
                dir = DASH_TAB2[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize];
            }
            self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = dir;
            self.ram[LINK_DIRECTION] = dir;
            self.ram[LINK_DIRECTION_LAST] = dir;
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            self.link_handle_moving_animation_full_long_entry();
            let org_x = self.player_state_view().x();
            let org_y = self.player_state_view().y();
            self.store_link_safe_return_position(org_x, org_y);
            self.link_handle_moving_floor();
            self.link_apply_conveyor();
            if self.ram[PLAYER_ON_SOMARIA_PLATFORM] != 0 {
                self.link_handle_velocity_and_sand_drag(org_x, org_y);
            }
            self.ram[LINK_Y_VEL] = self.player_state_view().y().wrapping_sub(org_y) as u8;
            self.ram[LINK_X_VEL] = self.player_state_view().x().wrapping_sub(org_x) as u8;
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
            return;
        }

        if self.ram[LINK_ANIMATION_STEPS] >= 6 {
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }
        self.ram[LINK_DASH_CTR] = self.ram[LINK_DASH_CTR].wrapping_sub(1);
        if self.ram[LINK_DASH_CTR] < 32 {
            self.ram[LINK_DASH_CTR] = 32;
        }
        self.ancilla_add_dash_dust(30, 0);
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
        if self.ram[LINK_SWORD_TYPE].wrapping_add(1) & 0xfe != 0 {
            self.tile_detect_main_handler(7);
        }
        if self.ram[SRAM_PROGRESS_INDICATOR] != 0 {
            self.ram[BUTTON_MASK_B_Y] |= 0x80;
            self.ram[BUTTON_B_FRAMES] = 9;
        }
        self.ram[LINK_INCAPACITATED_TIMER] = 0;

        let mut want_stop_dash = false;
        const FEATURES0_TURN_WHILE_DASHING: u32 = 4;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_TURN_WHILE_DASHING != 0 {
            if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
                self.ram[LINK_COUNTDOWN_FOR_DASH] = 0x11;
                want_stop_dash = true;
            } else {
                const DASH_CTRLS_TO_DIR: [u8; 16] =
                    [0, 1, 2, 0, 4, 4, 4, 0, 8, 8, 8, 0, 0, 0, 0, 0];
                let t = DASH_CTRLS_TO_DIR[(self.ram[JOYPAD1H_LAST] & 0x0f) as usize];
                if t != 0 && t != self.ram[LINK_DIRECTION_LAST] {
                    self.ram[LINK_DIRECTION] = t;
                    self.ram[LINK_DIRECTION_LAST] = t;
                    self.ram[SWIM_PLAYER_DIRECTION_FLAGS] = t;
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
        } else {
            let dir = self.ram[JOYPAD1H_LAST] & 0x0f;
            want_stop_dash =
                dir != 0 && dir != DASH_TAB2[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize];
        }
        if want_stop_dash {
            self.ram[LINK_PLAYER_HANDLER_STATE] = 18;
            self.ram[BUTTON_MASK_B_Y] &= !0x80;
            self.ram[BUTTON_B_FRAMES] = 0;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
            self.link_state_exiting_dash();
            return;
        }

        if self.ram[LINK_SPEED_SETTING] == 0
            && self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_TURN_WHILE_DASHING != 0
        {
            self.ram[LINK_SPEED_SETTING] = 16;
        }
        let mut dir = self.ram[FORCE_MOVE_ANY_DIRECTION] & 0x0f;
        if dir == 0 {
            dir = DASH_TAB2[(self.ram[LINK_DIRECTION_FACING] >> 1) as usize];
        }
        self.ram[LINK_DIRECTION] = dir;
        self.ram[LINK_DIRECTION_LAST] = dir;
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_handle_sword_cooldown(&mut self) {
        self.ram[LINK_SWORD_DELAY_TIMER] = self.ram[LINK_SWORD_DELAY_TIMER].wrapping_sub(1);
        if (self.ram[LINK_SWORD_DELAY_TIMER] as i8) >= 0 {
            return;
        }
        self.ram[LINK_SWORD_DELAY_TIMER] = 0;
        if self.ram[LINK_ITEM_IN_HAND] | self.ram[LINK_POSITION_MODE] != 0 {
            return;
        }
        if self.ram[BUTTON_B_FRAMES] < 9 {
            if self.ram[LINK_IS_RUNNING] == 0 {
                self.link_check_for_sword_swing();
            }
        } else {
            self.handle_sword_controls();
        }
    }

    pub(super) fn handle_sword_sfx_and_beam(&mut self) {
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[BUTTON_B_FRAMES] = 0;
        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;

        let health = self.ram[LINK_HEALTH_CAPACITY].wrapping_sub(4);
        if health < self.ram[LINK_HEALTH_CURRENT]
            && self.ram[LINK_SWORD_TYPE].wrapping_add(1) & 0xfe != 0
            && self.ram[LINK_SWORD_TYPE] >= 2
            && !(0..5).rev().any(|i| self.ram[ANCILLA_TYPE + i] == 0x31)
        {
            self.add_sword_beam(0);
        }
        let sword = self.ram[LINK_SWORD_TYPE].wrapping_sub(1);
        if sword != 0xfe && sword != 0xff {
            self.ram[SOUND_EFFECT_1] =
                FIRE_BEAM_SOUNDS[sword as usize] | self.link_calculate_sfx_pan();
        }
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 1;
    }

    pub(super) fn link_check_for_sword_swing(&mut self) {
        if self.ram[Y_BUTTON_ACTION_FLAGS] & 0x10 != 0 {
            return;
        }
        if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
            if self.ram[FILTERED_JOYPAD_H] & 0x80 == 0 {
                return;
            }
            if self.ram[IS_STANDING_IN_DOORWAY] != 0 {
                self.tile_detect_sword_swing_deep_in_door(self.ram[IS_STANDING_IN_DOORWAY]);
                if self.ram[R14] & 0x30 == 0x30 {
                    return;
                }
            }
            self.ram[BUTTON_MASK_B_Y] |= 0x80;
            self.handle_sword_sfx_and_beam();
            self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
            self.ram[LINK_ANIMATION_STEPS] = 0;
        }

        if self.ram[JOYPAD1H_LAST] & 0x80 == 0 {
            self.ram[BUTTON_MASK_B_Y] |= 1;
        }
        self.halt_link_when_using_items();
        self.ram[LINK_DIRECTION] &= !0x0f;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8).is_negative() {
            self.ram[BUTTON_B_FRAMES] = self.ram[BUTTON_B_FRAMES].wrapping_add(1);
            if self.ram[BUTTON_B_FRAMES] >= 9 {
                self.handle_sword_controls();
                return;
            }
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                SPIN_ATTACK_DELAYS[self.ram[BUTTON_B_FRAMES] as usize];
            if self.ram[BUTTON_B_FRAMES] == 5 {
                if self.ram[LINK_SWORD_TYPE] != 0
                    && self.ram[LINK_SWORD_TYPE] != 1
                    && self.ram[LINK_SWORD_TYPE] != 0xff
                {
                    self.ancilla_add_sword_swing_sparkle(0x26, 4);
                }
                if self.ram[LINK_SWORD_TYPE] != 0 && self.ram[LINK_SWORD_TYPE] != 0xff {
                    self.tile_detect_main_handler(if self.ram[LINK_SWORD_TYPE] == 1 {
                        1
                    } else {
                        6
                    });
                }
            } else if self.ram[BUTTON_B_FRAMES] >= 4
                && self.ram[BUTTON_MASK_B_Y] & 1 != 0
                && self.ram[JOYPAD1H_LAST] & 0x80 != 0
            {
                self.ram[BUTTON_MASK_B_Y] &= !1;
                self.handle_sword_sfx_and_beam();
                return;
            }
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn handle_sword_controls(&mut self) {
        if self.ram[JOYPAD1H_LAST] & 0x80 != 0 {
            self.player_sword_spin_attack_jerks_hold_down();
        } else if self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] < 48 {
            self.link_reset_sword_and_item_usage();
        } else {
            self.link_reset_sword_and_item_usage();
            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
            self.link_activate_spin_attack();
        }
    }

    pub(super) fn player_sword_spin_attack_jerks_hold_down(&mut self) {
        if self.ram[PLAYER_DEFENSE_FLAGS] & 0x80 != 0 || self.ram[PLAYER_DEFENSE_FLAGS] & 9 == 0 {
            if self.ram[SET_WHEN_DAMAGING_ENEMIES] == 0 {
                self.ram[BUTTON_B_FRAMES] = 9;
                self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
                self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
                if self.ram[LINK_SPEED_SETTING] != 4 && self.ram[LINK_SPEED_SETTING] != 16 {
                    self.ram[LINK_SPEED_SETTING] = 12;
                    if self.ram[LINK_SWORD_TYPE].wrapping_add(1) & !1 == 0 {
                        return;
                    }
                    if (0..5)
                        .rev()
                        .any(|i| matches!(self.ram[ANCILLA_TYPE + i], 0x30 | 0x31))
                    {
                        return;
                    }
                    if self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] >= 6
                        && self.ram[FRAME_COUNTER] & 3 == 0
                    {
                        self.ancilla_spawn_sword_charge_sparkle();
                    }
                    if self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] < 64 {
                        self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] =
                            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER].wrapping_add(1);
                        if self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] == 48 {
                            self.ancilla_sfx2_near(55);
                            self.ancilla_add_charged_spin_attack_sparkle();
                        }
                    }
                } else {
                    self.calculate_sword_hit_box();
                }
                return;
            } else if self.ram[SET_WHEN_DAMAGING_ENEMIES] == 1 {
                self.link_reset_sword_and_item_usage();
                return;
            }
        }
        if self.ram[BUTTON_B_FRAMES] == 9 {
            self.ram[BUTTON_B_FRAMES] = 10;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                SPIN_ATTACK_DELAYS[self.ram[BUTTON_B_FRAMES] as usize];
        }
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK].wrapping_sub(1);
        if (self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] as i8).is_negative() {
            let mut frames = self.ram[BUTTON_B_FRAMES].wrapping_add(1);
            if frames == 13 {
                if self.ram[LINK_SWORD_TYPE].wrapping_add(1) & !1 != 0
                    && self.ram[PLAYER_DEFENSE_FLAGS] & 9 != 0
                {
                    self.ancilla_add_wall_tap_spark(27, 1);
                    self.ancilla_sfx2_near(if self.ram[PLAYER_DEFENSE_FLAGS] & 8 != 0 {
                        6
                    } else {
                        5
                    });
                    self.tile_detect_main_handler(1);
                }
                frames = 10;
            }
            self.ram[BUTTON_B_FRAMES] = frames;
            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] =
                SPIN_ATTACK_DELAYS[self.ram[BUTTON_B_FRAMES] as usize];
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn link_activate_spin_attack(&mut self) {
        self.ancilla_add_spin_attack_init_spark(42, 0, 0);
        self.link_animate_victory_spin();
    }

    pub(super) fn link_animate_victory_spin(&mut self) {
        self.ram[LINK_PLAYER_HANDLER_STATE] = 3;
        self.ram[LINK_SPIN_OFFSETS] = (self.ram[LINK_DIRECTION_FACING] >> 1) * 12;
        self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 3;
        self.ram[STATE_FOR_SPIN_ATTACK] =
            LINK_SPIN_GRAPHICS_BY_DIR[self.ram[LINK_SPIN_OFFSETS] as usize];
        self.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 0;
        self.ram[BUTTON_B_FRAMES] = 144;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
        self.ram[BUTTON_MASK_B_Y] = 0x80;
        self.link_state_spin_attack();
    }

    pub(super) fn link_state_tree_pull(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.ram[LINK_AUXILIARY_STATE] != 0 {
            self.handle_link_from1_d();
            return;
        }

        if self.ram[LINK_GRABBING_WALL] != 0 {
            if self.ram[BUTTON_MASK_B_Y] == 0 {
                if self.ram[JOYPAD1L_LAST] & 0x80 == 0 {
                    self.ram[LINK_GRABBING_WALL] = 0;
                    self.ram[LINK_VAR30D] = 0;
                    self.ram[Y_BUTTON_ACTION_TIMER] = 2;
                    self.ram[Y_BUTTON_ACTION_STEP] = 0;
                    self.ram[LINK_CANT_CHANGE_DIRECTION] = 0;
                    self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
                    self.link_state_default();
                    return;
                }
                if self.ram[JOYPAD1H_LAST] & 4 == 0 {
                    self.link_state_tree_pull_tail();
                    return;
                }
                self.ram[BUTTON_MASK_B_Y] = 4;
                self.ancilla_sfx2_near(0x22);
            }

            self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
            if (self.ram[Y_BUTTON_ACTION_TIMER] as i8) >= 0 {
                self.link_state_tree_pull_tail();
                return;
            }
            self.ram[LINK_VAR30D] = self.ram[LINK_VAR30D].wrapping_add(1);
            let j = self.ram[LINK_VAR30D] as usize;
            self.ram[Y_BUTTON_ACTION_STEP] = *GRAB_WALL_ANIM_STEPS.get(j).unwrap_or(&0);
            self.ram[Y_BUTTON_ACTION_TIMER] = *GRAB_WALL_ANIM_TIMER.get(j).unwrap_or(&0);
            if j != 7 {
                self.link_state_tree_pull_tail();
                return;
            }

            self.ram[LINK_GRABBING_WALL] = 0;
            self.ram[LINK_VAR30D] = 0;
            self.ram[Y_BUTTON_ACTION_TIMER] = 2;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[LINK_STATE_BITS] = 1;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
        }

        if self.ram[PLAYER_DEFENSE_FLAGS] & 9 != 0 {
            self.link_state_tree_pull_reset_to_normal();
            return;
        }
        if self.ram[LINK_VAR30D] == 9 {
            if self.ram[FILTERED_JOYPAD_H] & 0x0f == 0 {
                self.link_handle_cardinal_collision();
                self.handle_indoor_camera_and_doors();
                return;
            }
            self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
            self.link_state_default();
            return;
        }
        self.ancilla_add_dash_dust_charging(0x1e, 0);
        self.ram[Y_BUTTON_ACTION_TIMER] = self.ram[Y_BUTTON_ACTION_TIMER].wrapping_sub(1);
        if (self.ram[Y_BUTTON_ACTION_TIMER] as i8) < 0 {
            self.ram[LINK_VAR30D] = self.ram[LINK_VAR30D].wrapping_add(1);
            let j = self.ram[LINK_VAR30D] as usize;
            self.ram[Y_BUTTON_ACTION_STEP] = GRAB_WALL_ANIM_STEPS2[j];
            self.ram[Y_BUTTON_ACTION_TIMER] = 2;
            self.ram[LINK_ACTUAL_VEL_Y] = 48;
            if j == 9 {
                self.link_state_tree_pull_reset_to_normal();
                return;
            }
        }
        self.flag67_with_directions();
        if self.ram[LINK_DIRECTION] & 3 == 0 {
            self.ram[LINK_ACTUAL_VEL_X] = 0;
        }
        if self.ram[LINK_DIRECTION] & 0x0c == 0 {
            self.ram[LINK_ACTUAL_VEL_Y] = 0;
        }
        self.link_state_tree_pull_tail();
    }

    pub(super) fn link_handle_recoil_and_timer(&mut self, jump_into_middle: bool) {
        self.replay_trace_player_state(if jump_into_middle {
            "recoil-timer-entry-jump"
        } else {
            "recoil-timer-entry"
        });
        if !jump_into_middle {
            self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
            self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;
            self.ram[LINK_NUM_ORTHOGONAL_DIRECTIONS] = 0;
            self.link_handle_recoiling();
            self.ram[LINK_INCAPACITATED_TIMER] = self.ram[LINK_INCAPACITATED_TIMER].wrapping_sub(1);
            if self.ram[LINK_INCAPACITATED_TIMER] == 0 {
                self.ram[LINK_INCAPACITATED_TIMER] = 1;
                let z = (self.ram[LINK_Z_COORD] & 0xfe) as i8;
                if z <= 0 && (self.ram[LINK_ACTUAL_VEL_Z] as i8) < 0 {
                    if self.ram[LINK_AUXILIARY_STATE] != 0 {
                        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                        let old_state = self.ram[LINK_PLAYER_HANDLER_STATE];
                        write_le_u16(&mut self.ram, SCRATCH_0, old_state as u16);
                        if self.ram[LINK_PLAYER_HANDLER_STATE] != 6 {
                            self.ram[BUTTON_B_FRAMES] = 0;
                            self.ram[BUTTON_MASK_B_Y] = 0;
                            self.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
                            self.ram[LINK_SPIN_ATTACK_STEP_COUNTER] = 0;
                        }
                        self.link_splash_upon_landing();
                        if self.ram[LINK_IS_BUNNY_MIRROR] == 0
                            || self.ram[LINK_IS_IN_DEEP_WATER] == 0
                        {
                            if self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] != 0 {
                                self.ram[LINK_WANT_MAKE_NOISE_WHEN_DASHED] = 0;
                                self.ancilla_sfx2_near(33);
                            } else if old_state != 2 && self.ram[LINK_PLAYER_HANDLER_STATE] != 4 {
                                self.ancilla_sfx2_near(33);
                            }
                            if self.ram[LINK_PLAYER_HANDLER_STATE] == 4 {
                                self.link_force_unequip_cape_quietly();
                                if self.ram[PLAYER_IS_INDOORS] != 0
                                    && old_state != 2
                                    && self.ram[LINK_ITEM_FLIPPERS] != 0
                                {
                                    self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
                                }
                                self.ancilla_add_splash(21, 0);
                            }
                            self.tile_detect_main_handler(0);
                            if self.ram[TILEDETECT_THICK_GRASS] & 1 != 0 {
                                self.ancilla_sfx2_near(26);
                            }
                            if self.ram[TILEDETECT_SHALLOW_WATER] & 1 != 0
                                && self.ram[SOUND_EFFECT_1] != 36
                            {
                                self.ancilla_sfx2_near(28);
                            }
                            if read_le_u16(&self.ram, TILEDETECT_DEEPWATER) & 1 != 0 {
                                self.ram[LINK_PLAYER_HANDLER_STATE] = 4;
                                self.link_set_to_deep_water();
                                self.link_reset_sword_and_item_usage();
                                self.ancilla_add_splash(21, 0);
                            }
                        }
                        self.finish_recoil_landing();
                    }
                    self.ram[LINK_ANIMATION_STEPS] = 0;
                    self.ram[LINK_INCAPACITATED_TIMER] = 0;
                }
            }
        } else {
            self.finish_recoil_landing();
            self.ram[LINK_ANIMATION_STEPS] = 0;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
        }

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 5 && self.ram[LINK_INCAPACITATED_TIMER] >= 33 {
            self.ram[LINK_INCAPACITATED_CAMERA_TIMER] =
                self.ram[LINK_INCAPACITATED_CAMERA_TIMER].wrapping_sub(1);
            if (self.ram[LINK_INCAPACITATED_CAMERA_TIMER] as i8) >= 0 {
                self.handle_indoor_camera_and_doors();
                self.ram[LINK_Z_COORD + 1] = 0;
                return;
            }
            self.ram[LINK_INCAPACITATED_CAMERA_TIMER] = self.ram[LINK_INCAPACITATED_TIMER] >> 4;
        }

        self.flag67_with_directions();
        if self.ram[LINK_PLAYER_HANDLER_STATE] != 6 {
            self.link_handle_diagonal_collision();
            if self.ram[LINK_DIRECTION] & 3 == 0 {
                self.ram[LINK_ACTUAL_VEL_X] = 0;
            }
            if self.ram[LINK_DIRECTION] & 0x0c == 0 {
                self.ram[LINK_ACTUAL_VEL_Y] = 0;
            }
        }
        self.link_move_position();

        if self.ram[LINK_PLAYER_HANDLER_STATE] != 6 {
            self.link_handle_cardinal_collision();
            self.ram[PIT_CORRECTION_ACTIVE_FLAG] = 0;
        }
        self.handle_indoor_camera_and_doors();
        if self.ram[LINK_Z_COORD] == 0 || self.ram[LINK_Z_COORD] >= 0xe0 {
            self.player_tile_detect_nearby();
            self.replay_trace_player_state("recoil-timer-after-nearby");
            if self.ram[TILEDETECT_PIT_TILE] & 0x0f == 0x0f {
                self.ram[LINK_PLAYER_HANDLER_STATE] = 1;
                self.ram[LINK_SPEED_SETTING] = 4;
                self.replay_trace_player_state("recoil-timer-set-pit");
            }
        }
        self.ram[LINK_Z_COORD + 1] = 0;
        self.replay_trace_player_state("recoil-timer-exit");
    }

    pub(super) fn gravestone_move(&mut self, k: usize) {
        if self.frame_control_view().submodule() != 0 {
            return;
        }
        self.ram[ANCILLA_Y_VEL + k] = (-8i8) as u8;
        self.ancilla_move_y(k);

        self.gravestone_act_as_barrier(k);
        let y_target =
            u16::from(self.ram[ANCILLA_A + k]) | (u16::from(self.ram[ANCILLA_B + k]) << 8);
        let y_cur = self.ancilla_get_y(k);
        if y_cur >= y_target {
            return;
        }

        self.ram[ANCILLA_TYPE + k] = 0;
        self.ram[LINK_SOMETHING_WITH_HOOKSHOT] = 0;
        self.ram[PLAYER_DEFENSE_FLAGS] &= !4;
        self.ram[SCRATCH_0] = self.ram[DOOR_DEBRIS_Y + k];
        self.ram[SCRATCH_0 + 1] = self.ram[DOOR_DEBRIS_X + k];
        let big_rock = read_le_u16(&self.ram, SCRATCH_0);
        write_le_u16(&mut self.ram, BIG_ROCK_STARTING_ADDRESS, big_rock);
        let counter = match big_rock {
            0x0532 => 0x48,
            0x0488 => 0x60,
            _ => 0x40,
        };
        write_le_u16(&mut self.ram, DOOR_OPEN_CLOSED_COUNTER, counter);
        self.overworld_do_map_update32x32_b_for_smash();
    }

    pub(super) fn somaria_block_handle_player_interaction(&mut self, k: usize) {
        self.ram[CUR_OBJECT_INDEX] = k as u8;
        if self.ram[ANCILLA_G + k] != 0 {
            return;
        }

        if self.ram[ANCILLA_H + k] == 0 {
            if self.ram[LINK_AUXILIARY_STATE] != 0
                || self.ram[LINK_STATE_BITS] & 1 != 0
                || (self.ram[ANCILLA_Z + k] != 0 && self.ram[ANCILLA_Z + k] != 0xff)
                || self.ram[ANCILLA_K + k] != 0
                || self.ram[ANCILLA_L + k] != 0
            {
                return;
            }
            if self.ram[JOYPAD1H_LAST] & 0x0f == 0 {
                self.ram[ANCILLA_ARR3 + k] = 0;
                self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                self.ram[ANCILLA_A + k] = 255;
                if self.ram[LINK_IS_RUNNING] == 0 {
                    self.ram[LINK_SPEED_SETTING] = 0;
                    return;
                }
            } else if self.ram[JOYPAD1H_LAST] & 0x0f == self.ram[ANCILLA_ARR3 + k] {
                if self.ram[LINK_SPEED_SETTING] == 18 {
                    self.ram[PLAYER_DEFENSE_FLAGS] |= 0x81;
                }
            } else {
                self.ram[ANCILLA_ARR3 + k] = self.ram[JOYPAD1H_LAST] & 0x0f;
                self.ram[LINK_SPEED_SETTING] = 0;
            }

            if !self.ancilla_check_link_collision(k, 4)
                || self.ram[ANCILLA_FLOOR + k] != self.ram[LINK_IS_ON_LOWER_LEVEL]
            {
                return;
            }

            if self.ram[LINK_IS_RUNNING] == 0 || self.ram[LINK_DASH_CTR] == 64 {
                self.ram[ANCILLA_X_VEL + k] = 0;
                self.ram[ANCILLA_Y_VEL + k] = 0;
                let t = self.ram[JOYPAD1H_LAST] & 0x0f;
                self.ram[ANCILLA_ARR3 + k] = t;
                if t & 3 != 0 {
                    self.ram[ANCILLA_X_VEL + k] = if t & 1 != 0 { 16 } else { (-16i8) as u8 };
                    self.ram[ANCILLA_DIR + k] = if t & 1 != 0 { 3 } else { 2 };
                } else {
                    self.ram[ANCILLA_Y_VEL + k] = if t & 8 != 0 { (-16i8) as u8 } else { 16 };
                    self.ram[ANCILLA_DIR + k] = if t & 8 != 0 { 0 } else { 1 };
                }
                if self.ram[LINK_ACTUAL_VEL_Y] == 0 || self.ram[LINK_ACTUAL_VEL_X] == 0 {
                    if !self.ancilla_check_tile_collision_class2(k) {
                        self.ancilla_move_y(k);
                        self.ancilla_move_x(k);
                        self.ram[ANCILLA_A + k] = self.ram[ANCILLA_A + k].wrapping_add(1);
                        if self.ram[LINK_STATE_BITS] & 0x80 == 0 && self.ram[ANCILLA_A + k] & 7 == 0
                        {
                            self.ancilla_sfx2_pan(k, 0x22);
                        }
                    }
                    self.ram[PLAYER_DEFENSE_FLAGS] = 0x81;
                    self.ram[LINK_SPEED_SETTING] = 0x12;
                }
                self.sprite_nullify_hookshot_drag();
                return;
            }

            const SOMARIA_BLOCK_YVEL: [u8; 4] = [(-40i8) as u8, 40, 0, 0];
            const SOMARIA_BLOCK_XVEL: [u8; 4] = [0, 0, (-40i8) as u8, 40];
            if self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] == k as u8 + 1 {
                self.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 0;
            }
            self.link_cancel_dash();
            self.ancilla_sfx3_pan(k, 0x32);
            let j = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
            self.ram[ANCILLA_DIR + k] = j as u8;
            self.ram[ANCILLA_Y_VEL + k] = SOMARIA_BLOCK_YVEL[j];
            self.ram[ANCILLA_X_VEL + k] = SOMARIA_BLOCK_XVEL[j];
            self.ram[ANCILLA_Z_VEL + k] = 48;
            self.ram[ANCILLA_H + k] = 1;
            self.ram[ANCILLA_Z + k] = 0;
        }

        self.ram[ANCILLA_Z_VEL + k] = self.ram[ANCILLA_Z_VEL + k].wrapping_sub(2);
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        self.ancilla_move_z(k);
        if self.ram[ANCILLA_Z + k] != 0 && self.ram[ANCILLA_Z + k] < 252 {
            return;
        }

        self.ancilla_sfx2_pan(k, 0x21);
        self.ram[ANCILLA_Z + k] = 0;
        let j = self.ram[ANCILLA_H + k];
        self.ram[ANCILLA_H + k] = self.ram[ANCILLA_H + k].wrapping_add(1);
        if j == 3 {
            self.ram[ANCILLA_ARR4 + k] = 0;
            self.ram[ANCILLA_H + k] = 0;
        } else {
            const SOMARIA_BLOCK_ZVEL: [u8; 4] = [48, 24, 16, 8];
            self.ram[ANCILLA_Z_VEL + k] = SOMARIA_BLOCK_ZVEL[j.wrapping_sub(1) as usize];
            self.ram[ANCILLA_Y_VEL + k] = ((self.ram[ANCILLA_Y_VEL + k] as i8) / 2) as u8;
            self.ram[ANCILLA_X_VEL + k] = ((self.ram[ANCILLA_X_VEL + k] as i8) / 2) as u8;
        }
    }

    pub(super) fn gravestone_act_as_barrier(&mut self, k: usize) {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        let r4 = y.wrapping_add(0x18);
        let r6 = x.wrapping_add(0x20);
        let lx = self.player_state_view().x().wrapping_add(8);
        let ly = self.player_state_view().y().wrapping_add(8);
        if ly >= y && ly < r4 && lx >= x && lx < r6 {
            let r10 = ly.abs_diff(r4);
            let new_y = self.player_state_view().y().wrapping_add(r10);
            self.player_state_view_mut().set_y(new_y);
            self.ram[LINK_Y_VEL] = self.ram[LINK_Y_VEL].wrapping_add(r10 as u8);
            self.ram[PLAYER_DEFENSE_FLAGS] |= 4;
        }
        if self.ram[LINK_DIRECTION_FACING] != 0 {
            self.ram[LINK_DIRECTION_FACING] &= !4;
        }
    }

    pub(super) fn link_handle_recoiling(&mut self) {
        self.ram[LINK_DIRECTION] = 0;
        if self.ram[LINK_ACTUAL_VEL_Y] != 0 {
            let direction = if (self.ram[LINK_ACTUAL_VEL_Y] as i8).is_negative() {
                8
            } else {
                4
            };
            self.ram[LINK_DIRECTION] |= direction;
            self.ram[LINK_DIRECTION_LAST] = self.ram[LINK_DIRECTION];
            self.player_handle_incapacitated_inner2();
        }
        if self.ram[LINK_ACTUAL_VEL_X] != 0 {
            let direction = if (self.ram[LINK_ACTUAL_VEL_X] as i8).is_negative() {
                2
            } else {
                1
            };
            self.ram[LINK_DIRECTION] |= direction;
            self.ram[LINK_DIRECTION_LAST] = self.ram[LINK_DIRECTION];
        }
        self.player_handle_incapacitated_inner2();
    }

    pub(super) fn player_handle_incapacitated_inner2(&mut self) {
        if self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 0x0c != 0
            && self.ram[LINK_MOVING_AGAINST_DIAG_TILE] & 3 != 0
            && self.ram[LINK_PLAYER_HANDLER_STATE] == 2
        {
            self.ram[LINK_ACTUAL_VEL_X] = (-(self.ram[LINK_ACTUAL_VEL_X] as i8)) as u8;
            self.ram[LINK_ACTUAL_VEL_Y] = (-(self.ram[LINK_ACTUAL_VEL_Y] as i8)) as u8;
        }
        if self.ram[IS_STANDING_IN_DOORWAY] == 1 {
            self.ram[LINK_DIRECTION_LAST] &= 0x0c;
            self.ram[LINK_DIRECTION] &= 0x0c;
            self.ram[LINK_ACTUAL_VEL_X] = 0;
        } else if self.ram[IS_STANDING_IN_DOORWAY] == 2 {
            self.ram[LINK_DIRECTION_LAST] &= 3;
            self.ram[LINK_DIRECTION] &= 3;
            self.ram[LINK_ACTUAL_VEL_Y] = 0;
        }
    }

    pub(super) fn find_free_moving_block_slot(&mut self, x: u8) -> u8 {
        if self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1] == 0 {
            self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + 1] = x.wrapping_add(1);
            return 1;
        }
        if self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS] == 0 {
            self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS] = x.wrapping_add(1);
            return 0;
        }
        0xff
    }

    pub(super) fn initialize_push_block(&mut self, r14: u8, idx: u8) -> bool {
        let slot = r14 as usize;
        let idx_word = (idx >> 1) as usize;
        let pos = read_le_u16(&self.ram, DUNG_OBJECT_TILEMAP_POS + idx_word * 2);
        let mut x = (pos & 0x007e) << 2;
        let mut y = (pos & 0x1f80) >> 4;
        x = x.wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_H_COPY) & 0xff00);
        y = y.wrapping_add(read_le_u16(&self.ram, DUNG_LOADE_BGOFFS_V_COPY) & 0xff00);

        write_le_u16(&mut self.ram, PUSHEDBLOCKS_X_LO + slot * 2, x & 0x00ff);
        write_le_u16(&mut self.ram, PUSHEDBLOCKS_X_HI + slot * 2, x >> 8);
        write_le_u16(&mut self.ram, PUSHEDBLOCKS_Y_LO + slot * 2, y & 0x00ff);
        write_le_u16(&mut self.ram, PUSHEDBLOCKS_Y_HI + slot * 2, y >> 8);
        write_le_u16(&mut self.ram, PUSHEDBLOCKS_TARGET + slot * 2, 0);
        write_le_u16(&mut self.ram, PUSHEDBLOCKS_SUBPIXEL + slot * 2, 0);

        if self.ram[DUNG_HDR_TAG] != 38
            && read_le_u16(&self.ram, DUNG_REPLACEMENT_TILE_STATE + idx_word * 2) == 0
        {
            if !self.push_block_attempt_to_push_the_block(0, x, y) {
                self.ancilla_sfx2_near(0x22);
                write_le_u16(&mut self.ram, DUNG_REPLACEMENT_TILE_STATE + idx_word * 2, 1);
                return false;
            }
        }

        self.ram[INDEX_OF_CHANGABLE_DUNGEON_OBJS + slot] = 0;
        true
    }

    pub(super) fn sprite_dungeon_draw_single_push_block(&mut self, mut j: usize) {
        const TAB1: [usize; 9] = [0, 1, 2, 3, 4, 0, 0, 0, 0];
        const CHAR: [u8; 4] = [0x0c, 0x0c, 0x0c, 0xff];
        j >>= 1;
        self.oam_allocate_from_region_b(4);
        let y = (self.ram[PUSHEDBLOCKS_Y_LO + j * 2] as u16
            | ((self.ram[PUSHEDBLOCKS_Y_HI + j * 2] as u16) << 8))
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
            .wrapping_sub(1);
        let x = (self.ram[PUSHEDBLOCKS_X_LO + j * 2] as u16
            | ((self.ram[PUSHEDBLOCKS_X_HI + j * 2] as u16) << 8))
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let ch = CHAR[TAB1[self.ram[PUSHED_BLOCK_MODE] as usize].min(CHAR.len() - 1)];
        if ch != 0xff {
            let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
            self.ram[oam] = x as u8;
            self.ram[oam + 1] = y as u8;
            self.ram[oam + 2] = ch;
            self.ram[oam + 3] = 0x20;
            let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR) as usize;
            self.ram[ext] = 2;
        }
    }

    pub(super) fn handle_layer_of_destination(&mut self) {
        self.ram[LINK_IS_ON_LOWER_LEVEL_MIRROR] =
            u8::from(self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE] >= 1);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = u8::from(self.ram[DUNG_HDR_HOLE_TELEPORTER_PLANE] >= 2);
    }

    pub(super) fn dungeon_pit_do_damage(&mut self) {
        self.frame_control_view_mut().set_submodule(20);
        self.ram[LINK_HEALTH_CURRENT] = self.ram[LINK_HEALTH_CURRENT].wrapping_sub(8);
        if self.ram[LINK_HEALTH_CURRENT] >= 0xa8 {
            self.ram[LINK_HEALTH_CURRENT] = 0;
        }
    }

    pub(super) fn reset_some_things_after_death(&mut self, speed_setting: u8) {
        self.ram[LINK_IS_IN_DEEP_WATER] = 0;
        self.ram[LINK_SPEED_SETTING] = speed_setting;
        self.ram[LINK_ON_CONVEYOR_BELT] = 0;
        self.ram[PLAYER_LAYER_COLLISION_FLAGS] = 0;
        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
        self.ram[PALETTE_SWAP_FLAG] = 0;
        self.ram[LINK_FAINT_ANIMATION_ACTIVE] = 0;
        self.ram[LINK_GIVE_DAMAGE] = 0;
        self.ram[LINK_ACTUAL_VEL_Y] = 0;
        self.ram[LINK_ACTUAL_VEL_X] = 0;
        self.ram[LINK_ACTUAL_VEL_Z] = 0;
        self.ram[LINK_Z_COORD] = 0;
        self.ram[DRAW_WATER_RIPPLES_OR_GRASS] = 0;
        self.ram[TILE_COLLISION_BITS_PRIMARY] = 0;
        self.ram[COUNTDOWN_FOR_BLINK] = 0;
        self.ram[LINK_PLAYER_HANDLER_STATE] = 0;
        self.ram[LINK_VISIBILITY_STATUS] = 0;
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    pub(super) fn player_handler_00_ground_3(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;

        self.apply_links_movement_to_camera_called = false;
        self.player_state_view_mut().set_z(0xffff);
        self.ram[LINK_ACTUAL_VEL_Z] = 0xff;
        self.ram[LINK_RECOILMODE_TIMER] = 0;

        let mut clear_vel_after = false;
        if !self.link_handle_toss() {
            self.link_handle_a_press();
            if (self.ram[LINK_STATE_BITS] | self.ram[LINK_GRABBING_WALL]) == 0
                && self.ram[LINK_PULL_ACTION_STATE] == 0
                && self.ram[LINK_PLAYER_HANDLER_STATE] != 17
            {
                self.link_handle_y_item();
                if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
                    && ((self.frame_control_view().main_module() == 14
                        && self.frame_control_view().submodule() != 2)
                        || matches!(self.ram[LINK_PLAYER_HANDLER_STATE], 8 | 9 | 10))
                {
                    self.finish_ground_movement_clear_vel_tail();
                    return;
                }
                if self.ram[SRAM_PROGRESS_INDICATOR] != 0 {
                    self.link_handle_sword_cooldown();
                    if self.ram[LINK_PLAYER_HANDLER_STATE] == 3 {
                        self.finish_ground_movement_clear_vel_tail();
                        return;
                    }
                }
            }
        }

        self.link_handle_cape_passive_lift_check();
        if self.ram[LINK_INCAPACITATED_TIMER] != 0 {
            self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
            self.ram[LINK_VAR30D] = 0;
            self.ram[LINK_VAR30E] = 0;
            self.ram[Y_BUTTON_ACTION_STEP] = 0;
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            if self.ram[BUTTON_MASK_B_Y] & 0x80 == 0 {
                self.ram[LINK_CANT_CHANGE_DIRECTION] &= !1;
            }
            self.link_handle_recoil_and_timer(false);
            return;
        }

        if self.ram[LINK_PULL_ACTION_STATE] != 0 {
            self.ram[LINK_DIRECTION] = 0;
            clear_vel_after = true;
        } else if self.ram[LINK_IS_TRANSFORMING] == 0
            && (self.ram[LINK_GRABBING_WALL] & !2) == 0
            && (self.ram[LINK_STATE_BITS] & 0x7f) == 0
            && ((self.ram[LINK_STATE_BITS] & 0x80) == 0
                || (self.ram[LINK_PICKING_THROW_STATE] & 1) == 0)
            && self.ram[LINK_ITEM_IN_HAND] == 0
            && self.ram[LINK_POSITION_MODE] == 0
            && (self.ram[BUTTON_B_FRAMES] >= 9
                || (self.ram[BUTTON_MASK_B_Y] & 0x20) != 0
                || (self.ram[BUTTON_MASK_B_Y] & 0x80) == 0)
        {
            if self.ram[LINK_FLAG_MOVING] != 0 {
                write_le_u16(&mut self.ram, SWIM_MAX_SPEED, 0x0180);
                write_le_u16(&mut self.ram, SWIM_MAX_SPEED + 2, 0x0180);
                self.link_handle_swim_movements();
                return;
            }

            self.reset_all_acceleration();
            let mut dir = (read_le_u16(&self.ram, FORCE_MOVE_ANY_DIRECTION) as u8) & 0x0f;
            if dir == 0 {
                if self.ram[LINK_GRABBING_WALL] & 2 != 0 {
                    self.finish_ground_movement_tail(clear_vel_after);
                    return;
                }
                dir = self.ram[JOYPAD1H_LAST] & 0x0f;
            }
            if dir == 0 {
                self.ram[LINK_X_VEL] = 0;
                self.ram[LINK_Y_VEL] = 0;
                self.ram[LINK_DIRECTION] = 0;
                self.ram[LINK_DIRECTION_LAST] = 0;
                self.ram[LINK_ANIMATION_STEPS] = 0;
                self.ram[PLAYER_DEFENSE_FLAGS] &= !0x0f;
                self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
                self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
            } else {
                self.ram[LINK_DIRECTION] = dir;
                if dir != self.ram[LINK_DIRECTION_LAST] {
                    self.ram[LINK_DIRECTION_LAST] = dir;
                    self.ram[LINK_SUBPIXEL_X] = 0;
                    self.ram[LINK_SUBPIXEL_Y] = 0;
                    self.ram[LINK_MOVING_AGAINST_DIAG_TILE] = 0;
                    self.ram[PLAYER_DEFENSE_FLAGS] = 0;
                    self.ram[LINK_TIMER_PUSH_GET_TIRED] = 32;
                    self.ram[LINK_TIMER_JUMP_LEDGE] = 19;
                }
            }
        }

        self.finish_ground_movement_tail(clear_vel_after);
    }

    pub(super) fn link_perform_throw(&mut self) {
        const LINK_LIFT_TAB: [u8; 9] = [0x54, 0x52, 0x50, 0xff, 0x51, 0x53, 0x55, 0x56, 0x57];
        if (self.ram[FLAG_IS_SPRITE_TO_PICK_UP] | self.ram[FLAG_IS_ANCILLA_TO_PICK_UP]) == 0 {
            self.link_reset_sword_and_item_usage();
            self.ram[Y_BUTTON_ACTION_FLAGS] = 0;
            let mut i = 15i8;
            while self.ram[SPRITE_STATE + i as usize] != 0 {
                i -= 1;
                if i < 0 {
                    return;
                }
            }

            if matches!(self.ram[INTERACTING_WITH_LIFTABLE_TILE_X1], 5 | 6) {
                self.ram[PLAYER_HANDLER_TIMER] = 1;
            } else {
                let (attr, x, y) = if self.ram[PLAYER_IS_INDOORS] != 0 {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Dungeon_LiftAndReplaceLiftable(&mut pt);
                    (attr, pt.x, pt.y)
                } else {
                    let mut pt = Point16U { x: 0, y: 0 };
                    let attr = self.Overworld_HandleLiftableTiles(&mut pt);
                    (attr, pt.x, pt.y)
                };
                let Some(idx) = LINK_LIFT_TAB.iter().rposition(|&value| value == attr) else {
                    return;
                };
                self.ram[FLAG_IS_SPRITE_TO_PICK_UP] = 1;
                self.sprite_spawn_throwable_terrain(idx as u8, x, y);
                self.ram[FILTERED_JOYPAD_L] &= !0x80;
                self.ram[PLAYER_HANDLER_TIMER] = 0;
            }
        } else {
            self.ram[PLAYER_HANDLER_TIMER] = 0;
        }

        self.ram[BUTTON_MASK_B_Y] = 0;
        self.ram[Y_BUTTON_ACTION_TIMER] = 6;
        self.ram[LINK_PICKING_THROW_STATE] = 1;
        self.ram[LINK_STATE_BITS] = 0x80;
        self.ram[Y_BUTTON_ACTION_STEP] = 0;
        self.ram[LINK_SPEED_SETTING] = 12;
        self.ram[LINK_ANIMATION_STEPS] = 0;
        self.ram[LINK_DIRECTION] &= 0xf0;
        self.ram[LINK_CANT_CHANGE_DIRECTION] |= 1;
    }

    pub(super) fn spawn_hammer_water_splash(&mut self) {
        const HAMMER_WATER_X: [i8; 4] = [0, 12, -8, 24];
        const HAMMER_WATER_Y: [i8; 4] = [8, 32, 24, 24];
        if (self.frame_control_view().submodule()
            | self.ram[FLAG_IS_LINK_IMMOBILIZED]
            | self.ram[FLAG_UNK1])
            != 0
        {
            return;
        }
        let i = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
        let x = self
            .player_state_view()
            .x()
            .wrapping_add(HAMMER_WATER_X[i] as i16 as u16);
        let y = self
            .player_state_view()
            .y()
            .wrapping_add(HAMMER_WATER_Y[i] as i16 as u16);
        let tiletype = if self.ram[PLAYER_IS_INDOORS] != 0 {
            let mut t = if self.ram[LINK_IS_ON_LOWER_LEVEL] >= 1 {
                0x1000
            } else {
                0
            };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE + t]
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };

        if matches!(tiletype, 8 | 9) {
            let j = self.sprite_spawn_small_splash(0);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, x.wrapping_sub(8));
                self.sprite_set_y(j, y.wrapping_sub(16));
                self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
                self.ram[SPRITE_Z + j] = 0;
            }
        }
    }

    pub(super) fn digging_game_guy_attempt_prize_spawn(&mut self) {
        const DIGGING_GAME_XVEL: [u8; 2] = [(-16i8) as u8, 16];
        const DIGGING_GAME_X: [i8; 2] = [0, 19];
        const DIGGING_GAME_ITEMS: [u8; 4] = [0xdb, 0xda, 0xd9, 0xdf];

        self.ram[BEAMOS_X_HI + 1] = self.ram[BEAMOS_X_HI + 1].wrapping_add(1);
        if self.player_state_view().y() >= 0x0b18 {
            return;
        }
        let j = self.get_random_number() & 7;
        let item_to_spawn = match j {
            0..=3 => DIGGING_GAME_ITEMS[j as usize],
            4 => {
                if self.ram[BEAMOS_X_HI + 1] < 25
                    || self.ram[BEAMOS_X_HI] != 0
                    || self.get_random_number() & 3 != 0
                {
                    return;
                }
                self.ram[BEAMOS_X_HI] = 0xeb;
                0xeb
            }
            _ => return,
        };

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(4, item_to_spawn, &mut info);
        if j >= 0 {
            let j = j as usize;
            let i = usize::from(self.ram[LINK_DIRECTION_FACING] != 4);
            self.ram[SPRITE_X_VEL + j] = DIGGING_GAME_XVEL[i];
            self.ram[SPRITE_Y_VEL + j] = 0;
            self.ram[SPRITE_Z_VEL + j] = 24;
            self.ram[SPRITE_STUNNED + j] = 255;
            self.ram[SPRITE_DELAY_AUX4 + j] = 48;
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(DIGGING_GAME_X[i] as i16 as u16)
                & !0x0f;
            let y = self.player_state_view().y().wrapping_add(22) & !0x0f;
            self.sprite_set_x(j, x);
            self.sprite_set_y(j, y);
            self.ram[SPRITE_FLOOR + j] = 0;
            self.sprite_sfx_queue_sfx3_with_pan(j, 0x30);
        }
    }

    fn sprite_spawn_small_splash_for_player(&mut self) -> Option<usize> {
        self.sprite_spawn_dynamically_for_player(0, 0xec)
    }

    fn sprite_spawn_dynamically_for_player(&mut self, _k: u8, what: u8) -> Option<usize> {
        let j = (0..16).rev().find(|&j| self.ram[SPRITE_STATE + j] == 0)?;
        self.ram[SPRITE_STATE + j] = 9;
        self.ram[SPRITE_TYPE + j] = what;
        Some(j)
    }

    pub(super) fn handle_indoor_camera_and_doors(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            return;
        }
        if self.ram[IS_STANDING_IN_DOORWAY] != 0 {
            self.handle_door_transitions();
        } else {
            self.apply_links_movement_to_camera();
        }
    }

    pub(super) fn cache_camera_properties_if_outdoors(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.cache_camera_properties_for_player();
        }
    }

    pub(super) fn handle_door_transitions(&mut self) {
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] = 0;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] = 0;

        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0
            && !(self.frame_control_view().main_module() == 7
                && self.frame_control_view().submodule() == 0)
        {
            return;
        }

        if self.ram[LINK_DIRECTION_LAST] & 0x0c != 0 && self.ram[IS_STANDING_IN_DOORWAY] == 1 {
            if self.ram[LINK_DIRECTION_LAST] & 4 != 0 {
                let t = self.player_state_view().y().wrapping_add(28);
                if t & 0x00fc == 0 {
                    self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] =
                        ((t >> 8) as u8).wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_HI]);
                }
            } else {
                let t = self.player_state_view().y().wrapping_sub(18);
                self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] =
                    ((t >> 8) as u8).wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_HI]);
            }
        }

        if self.ram[LINK_DIRECTION_LAST] & 3 != 0 && self.ram[IS_STANDING_IN_DOORWAY] == 2 {
            if self.ram[LINK_DIRECTION_LAST] & 1 != 0 {
                let t = self.player_state_view().x().wrapping_add(21);
                if t & 0x00fc == 0 {
                    self.ram[LINK_X_PAGE_MOVEMENT_DELTA] =
                        ((t >> 8) as u8).wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_HI]);
                }
            } else {
                let t = self.player_state_view().x().wrapping_sub(8);
                self.ram[LINK_X_PAGE_MOVEMENT_DELTA] =
                    ((t >> 8) as u8).wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_HI]);
            }
        }

        if self.ram[LINK_X_PAGE_MOVEMENT_DELTA] != 0 {
            self.ram[Y_BUTTON_ACTION_TIMER] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            if (self.ram[LINK_X_PAGE_MOVEMENT_DELTA] as i8).is_negative() {
                self.Dung_StartInterRoomTrans_Left_Plus();
            } else {
                self.HandleEdgeTransitionMovementEast_RightBy8();
            }
        } else if self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] != 0 {
            self.ram[Y_BUTTON_ACTION_TIMER] = 0;
            self.ram[LINK_STATE_BITS] = 0;
            self.ram[LINK_PICKING_THROW_STATE] = 0;
            self.ram[LINK_GRABBING_WALL] = 0;
            if (self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] as i8).is_negative() {
                self.Dungeon_StartInterRoomTrans_Up();
            } else {
                self.HandleEdgeTransitionMovementSouth_DownBy16();
            }
        }
    }

    pub(super) fn apply_links_movement_to_camera(&mut self) {
        self.apply_links_movement_to_camera_called = true;
        self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] =
            self.ram[LINK_Y_COORD + 1].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_HI]);
        self.ram[LINK_X_PAGE_MOVEMENT_DELTA] =
            self.ram[LINK_X_COORD + 1].wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_HI]);

        if self.ram[LINK_X_PAGE_MOVEMENT_DELTA] != 0 {
            if (self.ram[LINK_X_PAGE_MOVEMENT_DELTA] as i8).is_negative() {
                self.AdjustQuadrantAndCamera_left();
            } else {
                self.AdjustQuadrantAndCamera_right();
            }
        }
        if self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] != 0 {
            if (self.ram[LINK_Y_PAGE_MOVEMENT_DELTA] as i8).is_negative() {
                self.AdjustQuadrantAndCamera_up();
            } else {
                self.AdjustQuadrantAndCamera_down();
            }
        }
    }
}
