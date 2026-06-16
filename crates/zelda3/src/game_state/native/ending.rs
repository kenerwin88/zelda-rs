use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

const INTRO_ACTOR_COUNT: usize = INTRO_SPRITE_SUBTYPE - INTRO_SPRITE_IS_INITED;

fn read_byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

fn read_word(ram: &[u8], offset: usize) -> u16 {
    if offset + 1 < ram.len() {
        read_le_u16(ram, offset)
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AttractSceneState {
    pub(crate) state_word: u16,
    pub(crate) sequence: u8,
    pub(crate) scene_timer: u8,
    pub(crate) scene_substep: u8,
    pub(crate) x_base: u16,
    pub(crate) y_base: u8,
    pub(crate) story_text_pointer: u16,
    pub(crate) oam_index: u8,
    pub(crate) maiden_warp_step: u8,
    pub(crate) intro_step_index: u8,
    pub(crate) intro_step_timer: u8,
    pub(crate) intro_frame_counter: u8,
    pub(crate) intro_did_run_step: u8,
    pub(crate) intro_palette_flash_count: u8,
    pub(crate) legend_flag: u8,
    pub(crate) next_legend_gfx: u8,
    pub(crate) bg2_vofs_backup: u16,
    pub(crate) throne_fade_timer: u8,
    pub(crate) prison_zelda_y_base: u8,
    pub(crate) anim_step_counter: u8,
    pub(crate) soldier_anim_step: u8,
    pub(crate) prison_soldier_x_lo: u8,
    pub(crate) scene_frame_counter: u8,
    pub(crate) scene_done_flag: u8,
    pub(crate) legend_ctr: u16,
    pub(crate) fade_in_complete_flag: u8,
    pub(crate) fade_in_done_flag: u8,
    pub(crate) substep_delay_counter: u8,
    pub(crate) maiden_warp_timer_a: u8,
    pub(crate) maiden_warp_timer_b: u8,
    pub(crate) mode7_zoom_timer: u8,
}

impl AttractSceneState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            state_word: read_word(ram, ATTRACT_STATE),
            sequence: read_byte(ram, ATTRACT_SEQUENCE),
            scene_timer: read_byte(ram, ATTRACT_SCENE_TIMER),
            scene_substep: read_byte(ram, ATTRACT_SCENE_SUBSTEP),
            x_base: read_word(ram, ATTRACT_X_BASE),
            y_base: read_byte(ram, ATTRACT_Y_BASE),
            story_text_pointer: read_word(ram, ATTRACT_STORY_TEXT_POINTER),
            oam_index: read_byte(ram, ATTRACT_OAM_IDX),
            maiden_warp_step: read_byte(ram, ATTRACT_MAIDEN_WARP_STEP),
            intro_step_index: read_byte(ram, INTRO_STEP_INDEX),
            intro_step_timer: read_byte(ram, INTRO_STEP_TIMER),
            intro_frame_counter: read_byte(ram, INTRO_FRAME_CTR),
            intro_did_run_step: read_byte(ram, INTRO_DID_RUN_STEP),
            intro_palette_flash_count: read_byte(ram, INTRO_TIMES_PAL_FLASH),
            legend_flag: read_byte(ram, ATTRACT_LEGEND_FLAG),
            next_legend_gfx: read_byte(ram, ATTRACT_NEXT_LEGEND_GFX),
            bg2_vofs_backup: read_word(ram, ATTRACT_BG2_VOFS_BACKUP),
            throne_fade_timer: read_byte(ram, ATTRACT_THRONE_FADE_TIMER),
            prison_zelda_y_base: read_byte(ram, ATTRACT_PRISON_ZELDA_Y_BASE),
            anim_step_counter: read_byte(ram, ATTRACT_ANIM_STEP_COUNTER),
            soldier_anim_step: read_byte(ram, ATTRACT_SOLDIER_ANIM_STEP),
            prison_soldier_x_lo: read_byte(ram, ATTRACT_PRISON_SOLDIER_X_LO),
            scene_frame_counter: read_byte(ram, ATTRACT_SCENE_FRAME_COUNTER),
            scene_done_flag: read_byte(ram, ATTRACT_SCENE_DONE_FLAG),
            legend_ctr: read_word(ram, ATTRACT_LEGEND_CTR),
            fade_in_complete_flag: read_byte(ram, ATTRACT_FADE_IN_COMPLETE_FLAG),
            fade_in_done_flag: read_byte(ram, ATTRACT_FADE_IN_DONE_FLAG),
            substep_delay_counter: read_byte(ram, ATTRACT_SUBSTEP_DELAY_COUNTER),
            maiden_warp_timer_a: read_byte(ram, ATTRACT_MAIDEN_WARP_TIMER_A),
            maiden_warp_timer_b: read_byte(ram, ATTRACT_MAIDEN_WARP_TIMER_B),
            mode7_zoom_timer: read_byte(ram, TIMER_FOR_MODE7_ZOOM),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ATTRACT_STATE, self.state_word);
        ram[ATTRACT_SEQUENCE] = self.sequence;
        ram[ATTRACT_SCENE_TIMER] = self.scene_timer;
        ram[ATTRACT_SCENE_SUBSTEP] = self.scene_substep;
        write_le_u16(ram, ATTRACT_X_BASE, self.x_base);
        ram[ATTRACT_Y_BASE] = self.y_base;
        write_le_u16(ram, ATTRACT_STORY_TEXT_POINTER, self.story_text_pointer);
        ram[ATTRACT_OAM_IDX] = self.oam_index;
        ram[ATTRACT_MAIDEN_WARP_STEP] = self.maiden_warp_step;
        ram[INTRO_STEP_INDEX] = self.intro_step_index;
        ram[INTRO_STEP_TIMER] = self.intro_step_timer;
        ram[INTRO_FRAME_CTR] = self.intro_frame_counter;
        ram[INTRO_DID_RUN_STEP] = self.intro_did_run_step;
        ram[INTRO_TIMES_PAL_FLASH] = self.intro_palette_flash_count;
        ram[ATTRACT_LEGEND_FLAG] = self.legend_flag;
        ram[ATTRACT_NEXT_LEGEND_GFX] = self.next_legend_gfx;
        write_le_u16(ram, ATTRACT_BG2_VOFS_BACKUP, self.bg2_vofs_backup);
        ram[ATTRACT_THRONE_FADE_TIMER] = self.throne_fade_timer;
        ram[ATTRACT_PRISON_ZELDA_Y_BASE] = self.prison_zelda_y_base;
        ram[ATTRACT_ANIM_STEP_COUNTER] = self.anim_step_counter;
        ram[ATTRACT_SOLDIER_ANIM_STEP] = self.soldier_anim_step;
        ram[ATTRACT_PRISON_SOLDIER_X_LO] = self.prison_soldier_x_lo;
        ram[ATTRACT_SCENE_FRAME_COUNTER] = self.scene_frame_counter;
        ram[ATTRACT_SCENE_DONE_FLAG] = self.scene_done_flag;
        write_le_u16(ram, ATTRACT_LEGEND_CTR, self.legend_ctr);
        ram[ATTRACT_FADE_IN_COMPLETE_FLAG] = self.fade_in_complete_flag;
        ram[ATTRACT_FADE_IN_DONE_FLAG] = self.fade_in_done_flag;
        ram[ATTRACT_SUBSTEP_DELAY_COUNTER] = self.substep_delay_counter;
        ram[ATTRACT_MAIDEN_WARP_TIMER_A] = self.maiden_warp_timer_a;
        ram[ATTRACT_MAIDEN_WARP_TIMER_B] = self.maiden_warp_timer_b;
        ram[TIMER_FOR_MODE7_ZOOM] = self.mode7_zoom_timer;
    }

    pub(crate) fn state(&self) -> u8 {
        self.state_word as u8
    }

    pub(crate) fn state_word(&self) -> u16 {
        self.state_word
    }

    pub(crate) fn sequence(&self) -> u8 {
        self.sequence
    }

    pub(crate) fn scene_timer(&self) -> u8 {
        self.scene_timer
    }

    pub(crate) fn scene_substep(&self) -> u8 {
        self.scene_substep
    }

    pub(crate) fn x_base(&self) -> u8 {
        self.x_base as u8
    }

    pub(crate) fn x_base_word(&self) -> u16 {
        self.x_base
    }

    pub(crate) fn x_base_high(&self) -> u8 {
        (self.x_base >> 8) as u8
    }

    pub(crate) fn y_base(&self) -> u8 {
        self.y_base
    }

    pub(crate) fn oam_index(&self) -> u8 {
        self.oam_index
    }

    pub(crate) fn maiden_warp_step(&self) -> u8 {
        self.maiden_warp_step
    }

    pub(crate) fn intro_step_index(&self) -> u8 {
        self.intro_step_index
    }

    pub(crate) fn intro_step_timer(&self) -> u8 {
        self.intro_step_timer
    }

    pub(crate) fn intro_frame_counter(&self) -> u8 {
        self.intro_frame_counter
    }

    pub(crate) fn intro_did_run_step(&self) -> u8 {
        self.intro_did_run_step
    }

    pub(crate) fn intro_palette_flash_count(&self) -> u8 {
        self.intro_palette_flash_count
    }

    pub(crate) fn legend_flag(&self) -> u8 {
        self.legend_flag
    }

    pub(crate) fn next_legend_gfx(&self) -> u8 {
        self.next_legend_gfx
    }

    pub(crate) fn next_legend_image(&self) -> u8 {
        self.next_legend_gfx >> 1
    }

    pub(crate) fn bg2_vofs_backup(&self) -> u16 {
        self.bg2_vofs_backup
    }

    pub(crate) fn throne_fade_timer(&self) -> u8 {
        self.throne_fade_timer
    }

    pub(crate) fn prison_zelda_y_base(&self) -> u8 {
        self.prison_zelda_y_base
    }

    pub(crate) fn anim_step_counter(&self) -> u8 {
        self.anim_step_counter
    }

    pub(crate) fn soldier_anim_step(&self) -> u8 {
        self.soldier_anim_step
    }

    pub(crate) fn prison_soldier_x_lo(&self) -> u8 {
        self.prison_soldier_x_lo
    }

    pub(crate) fn scene_frame_counter(&self) -> u8 {
        self.scene_frame_counter
    }

    pub(crate) fn scene_done_flag(&self) -> u8 {
        self.scene_done_flag
    }

    pub(crate) fn legend_ctr(&self) -> u16 {
        self.legend_ctr
    }

    pub(crate) fn fade_in_complete_flag(&self) -> u8 {
        self.fade_in_complete_flag
    }

    pub(crate) fn fade_in_done_flag(&self) -> u8 {
        self.fade_in_done_flag
    }

    pub(crate) fn substep_delay_counter(&self) -> u8 {
        self.substep_delay_counter
    }

    pub(crate) fn maiden_warp_timer_a(&self) -> u8 {
        self.maiden_warp_timer_a
    }

    pub(crate) fn maiden_warp_timer_b(&self) -> u8 {
        self.maiden_warp_timer_b
    }

    pub(crate) fn mode7_zoom_timer(&self) -> u8 {
        self.mode7_zoom_timer
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntroSceneState {
    pub(crate) triangle_motion_pause: u8,
    pub(crate) sprite_oam_cursor: u16,
    pub(crate) triforce_countdown: u16,
}

impl IntroSceneState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            triangle_motion_pause: ram.get(INTRO_WANT_DOUBLE_RET).copied().unwrap_or(0),
            sprite_oam_cursor: if INTRO_SPRITE_ALLOC + 1 < ram.len() {
                read_le_u16(ram, INTRO_SPRITE_ALLOC)
            } else {
                0
            },
            triforce_countdown: if TRIFORCE_CTR + 1 < ram.len() {
                read_le_u16(ram, TRIFORCE_CTR)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[INTRO_WANT_DOUBLE_RET] = self.triangle_motion_pause;
        write_le_u16(ram, INTRO_SPRITE_ALLOC, self.sprite_oam_cursor);
        write_le_u16(ram, TRIFORCE_CTR, self.triforce_countdown);
    }

    pub(crate) fn triangle_motion_is_paused(&self) -> bool {
        self.triangle_motion_pause != 0
    }

    pub(crate) fn pause_triangle_motion(&mut self) {
        self.triangle_motion_pause = 1;
    }

    pub(crate) fn resume_triangle_motion(&mut self) {
        self.triangle_motion_pause = 0;
    }

    pub(crate) fn allocate_oam_entries(&mut self, entry_count: usize) -> usize {
        let cursor = self.sprite_oam_cursor as usize;
        let byte_count = entry_count.wrapping_mul(4);
        self.sprite_oam_cursor = self.sprite_oam_cursor.wrapping_add(byte_count as u16);
        cursor
    }

    pub(crate) fn decrement_triforce_countdown(&mut self) {
        self.triforce_countdown = self.triforce_countdown.wrapping_sub(1);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntroActorSlotState {
    pub(crate) init_phase: u8,
    pub(crate) subtype: u8,
    pub(crate) state: u8,
    pub(crate) x_subpixel: u8,
    pub(crate) x_low: u8,
    pub(crate) x_high: u8,
    pub(crate) y_subpixel: u8,
    pub(crate) y_low: u8,
    pub(crate) y_high: u8,
    pub(crate) x_velocity: u8,
    pub(crate) y_velocity: u8,
}

impl IntroActorSlotState {
    pub(crate) fn x(&self) -> u16 {
        u16::from(self.x_low) | (u16::from(self.x_high) << 8)
    }

    pub(crate) fn y(&self) -> u16 {
        u16::from(self.y_low) | (u16::from(self.y_high) << 8)
    }

    fn set_x(&mut self, value: i16) {
        self.x_low = value as u8;
        self.x_high = (value >> 8) as u8;
    }

    fn set_y(&mut self, value: i16) {
        self.y_low = value as u8;
        self.y_high = (value >> 8) as u8;
    }

    fn move_x(&mut self) {
        move_axis24(
            &mut self.x_subpixel,
            &mut self.x_low,
            &mut self.x_high,
            self.x_velocity,
        );
    }

    fn move_y(&mut self) {
        move_axis24(
            &mut self.y_subpixel,
            &mut self.y_low,
            &mut self.y_high,
            self.y_velocity,
        );
    }
}

fn move_axis24(subpixel: &mut u8, low: &mut u8, high: &mut u8, velocity: u8) {
    let pos = u32::from(*subpixel) | (u32::from(*low) << 8) | (u32::from(*high) << 16);
    let delta = ((velocity as i8 as i32) << 4) as u32;
    let moved = pos.wrapping_add(delta);
    *subpixel = moved as u8;
    *low = (moved >> 8) as u8;
    *high = (moved >> 16) as u8;
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntroActorState {
    slots: [IntroActorSlotState; INTRO_ACTOR_COUNT],
}

impl Default for IntroActorState {
    fn default() -> Self {
        Self {
            slots: [IntroActorSlotState::default(); INTRO_ACTOR_COUNT],
        }
    }
}

impl IntroActorState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..INTRO_ACTOR_COUNT {
            state.slots[slot] = IntroActorSlotState {
                init_phase: read_byte(ram, INTRO_SPRITE_IS_INITED + slot),
                subtype: read_byte(ram, INTRO_SPRITE_SUBTYPE + slot),
                state: read_byte(ram, INTRO_SPRITE_STATE + slot),
                x_subpixel: read_byte(ram, INTRO_X_SUBPIXEL + slot),
                x_low: read_byte(ram, INTRO_X_LO + slot),
                x_high: read_byte(ram, INTRO_X_HI + slot),
                y_subpixel: read_byte(ram, INTRO_Y_SUBPIXEL + slot),
                y_low: read_byte(ram, INTRO_Y_LO + slot),
                y_high: read_byte(ram, INTRO_Y_HI + slot),
                x_velocity: read_byte(ram, INTRO_X_VEL + slot),
                y_velocity: read_byte(ram, INTRO_Y_VEL + slot),
            };
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, actor) in self.slots.iter().copied().enumerate() {
            ram[INTRO_SPRITE_IS_INITED + slot] = actor.init_phase;
            ram[INTRO_SPRITE_SUBTYPE + slot] = actor.subtype;
            ram[INTRO_SPRITE_STATE + slot] = actor.state;
            ram[INTRO_X_SUBPIXEL + slot] = actor.x_subpixel;
            ram[INTRO_X_LO + slot] = actor.x_low;
            ram[INTRO_X_HI + slot] = actor.x_high;
            ram[INTRO_Y_SUBPIXEL + slot] = actor.y_subpixel;
            ram[INTRO_Y_LO + slot] = actor.y_low;
            ram[INTRO_Y_HI + slot] = actor.y_high;
            ram[INTRO_X_VEL + slot] = actor.x_velocity;
            ram[INTRO_Y_VEL + slot] = actor.y_velocity;
        }
    }

    pub(crate) fn slot(&self, slot: usize) -> IntroActorSlotState {
        self.slots.get(slot).copied().unwrap_or_default()
    }
}

pub(crate) struct IntroActorRead<'a> {
    state: &'a IntroActorState,
    slot: usize,
}

impl<'a> IntroActorRead<'a> {
    pub(crate) fn new(state: &'a IntroActorState, slot: usize) -> Self {
        Self { state, slot }
    }

    fn actor(&self) -> IntroActorSlotState {
        self.state.slot(self.slot)
    }

    pub(crate) fn x(&self) -> u16 {
        self.actor().x()
    }

    pub(crate) fn y(&self) -> u16 {
        self.actor().y()
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.actor().x_low
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.actor().y_low
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.actor().x_velocity
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.actor().y_velocity
    }

    pub(crate) fn init_phase(&self) -> u8 {
        self.actor().init_phase
    }

    pub(crate) fn subtype(&self) -> u8 {
        self.actor().subtype
    }

    pub(crate) fn state(&self) -> u8 {
        self.actor().state
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EndingCreditState {
    pub(crate) palace_death_count_digit_step: u16,
    pub(crate) death_count_digit_tile_base: u16,
}

impl EndingCreditState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            palace_death_count_digit_step: if ENDING_WHICH_DUNG + 1 < ram.len() {
                read_le_u16(ram, ENDING_WHICH_DUNG)
            } else {
                0
            },
            death_count_digit_tile_base: if ENDING_CREDIT_DIGIT_CHAR + 1 < ram.len() {
                read_le_u16(ram, ENDING_CREDIT_DIGIT_CHAR)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, ENDING_WHICH_DUNG, self.palace_death_count_digit_step);
        write_le_u16(
            ram,
            ENDING_CREDIT_DIGIT_CHAR,
            self.death_count_digit_tile_base,
        );
    }

    pub(crate) fn palace_death_count_index(&self) -> usize {
        (self.palace_death_count_digit_step >> 1) as usize
    }

    pub(crate) fn digit_tile_base_index(&self) -> usize {
        (self.palace_death_count_digit_step & 1) as usize
    }

    pub(crate) fn should_write_digit_for_scroll_y(
        &self,
        current_scroll_y: u16,
        scheduled_scroll_y: u16,
    ) -> bool {
        self.digit_tile_base_index() != 0 || current_scroll_y == scheduled_scroll_y
    }

    pub(crate) fn clear_palace_death_count_digit_step(&mut self) {
        self.palace_death_count_digit_step = 0;
    }

    pub(crate) fn advance_palace_death_count_digit_step(&mut self) {
        self.palace_death_count_digit_step = self.palace_death_count_digit_step.wrapping_add(1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EndingState {
    pub(crate) attract_scene: AttractSceneState,
    pub(crate) intro_scene: IntroSceneState,
    pub(crate) intro_actors: IntroActorState,
    pub(crate) credits: EndingCreditState,
}

impl EndingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            attract_scene: AttractSceneState::load_from_ram(ram),
            intro_scene: IntroSceneState::load_from_ram(ram),
            intro_actors: IntroActorState::load_from_ram(ram),
            credits: EndingCreditState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.attract_scene.write_to_ram(ram);
        self.intro_scene.write_to_ram(ram);
        self.intro_actors.write_to_ram(ram);
        self.credits.write_to_ram(ram);
    }
}

pub(crate) struct NativeAttractSceneBridgeMut<'a> {
    attract_scene: &'a mut AttractSceneState,
    ram: &'a mut [u8],
}

impl<'a> NativeAttractSceneBridgeMut<'a> {
    pub(crate) fn new(attract_scene: &'a mut AttractSceneState, ram: &'a mut [u8]) -> Self {
        Self { attract_scene, ram }
    }

    fn apply_byte(&mut self, offset: usize, value: u8) {
        match offset {
            ATTRACT_STATE => {
                self.attract_scene.state_word =
                    (self.attract_scene.state_word & 0xff00) | u16::from(value)
            }
            ATTRACT_SEQUENCE => {
                self.attract_scene.sequence = value;
                self.attract_scene.state_word =
                    (self.attract_scene.state_word & 0x00ff) | (u16::from(value) << 8);
            }
            ATTRACT_SCENE_TIMER => self.attract_scene.scene_timer = value,
            ATTRACT_SCENE_SUBSTEP => self.attract_scene.scene_substep = value,
            ATTRACT_X_BASE => {
                self.attract_scene.x_base = (self.attract_scene.x_base & 0xff00) | u16::from(value)
            }
            ATTRACT_X_BASE_HI => {}
            ATTRACT_Y_BASE => {
                self.attract_scene.y_base = value;
                self.attract_scene.x_base =
                    (self.attract_scene.x_base & 0x00ff) | (u16::from(value) << 8);
            }
            ATTRACT_OAM_IDX => self.attract_scene.oam_index = value,
            ATTRACT_MAIDEN_WARP_STEP => self.attract_scene.maiden_warp_step = value,
            INTRO_STEP_INDEX => self.attract_scene.intro_step_index = value,
            INTRO_STEP_TIMER => self.attract_scene.intro_step_timer = value,
            INTRO_FRAME_CTR => self.attract_scene.intro_frame_counter = value,
            INTRO_DID_RUN_STEP => self.attract_scene.intro_did_run_step = value,
            INTRO_TIMES_PAL_FLASH => self.attract_scene.intro_palette_flash_count = value,
            ATTRACT_LEGEND_FLAG => self.attract_scene.legend_flag = value,
            ATTRACT_NEXT_LEGEND_GFX => self.attract_scene.next_legend_gfx = value,
            ATTRACT_THRONE_FADE_TIMER => self.attract_scene.throne_fade_timer = value,
            ATTRACT_PRISON_ZELDA_Y_BASE => self.attract_scene.prison_zelda_y_base = value,
            ATTRACT_ANIM_STEP_COUNTER => self.attract_scene.anim_step_counter = value,
            ATTRACT_SOLDIER_ANIM_STEP => self.attract_scene.soldier_anim_step = value,
            ATTRACT_PRISON_SOLDIER_X_LO => self.attract_scene.prison_soldier_x_lo = value,
            ATTRACT_SCENE_FRAME_COUNTER => self.attract_scene.scene_frame_counter = value,
            ATTRACT_SCENE_DONE_FLAG => self.attract_scene.scene_done_flag = value,
            ATTRACT_FADE_IN_COMPLETE_FLAG => self.attract_scene.fade_in_complete_flag = value,
            ATTRACT_FADE_IN_DONE_FLAG => self.attract_scene.fade_in_done_flag = value,
            ATTRACT_SUBSTEP_DELAY_COUNTER => self.attract_scene.substep_delay_counter = value,
            ATTRACT_MAIDEN_WARP_TIMER_A => self.attract_scene.maiden_warp_timer_a = value,
            ATTRACT_MAIDEN_WARP_TIMER_B => self.attract_scene.maiden_warp_timer_b = value,
            TIMER_FOR_MODE7_ZOOM => self.attract_scene.mode7_zoom_timer = value,
            _ => {}
        }
    }

    fn apply_word(&mut self, offset: usize, value: u16) {
        match offset {
            ATTRACT_STATE => {
                self.attract_scene.state_word = value;
                self.attract_scene.sequence = (value >> 8) as u8;
            }
            ATTRACT_X_BASE => {
                self.attract_scene.x_base = value;
                self.attract_scene.y_base = (value >> 8) as u8;
            }
            ATTRACT_STORY_TEXT_POINTER => self.attract_scene.story_text_pointer = value,
            ATTRACT_BG2_VOFS_BACKUP => self.attract_scene.bg2_vofs_backup = value,
            ATTRACT_LEGEND_CTR => self.attract_scene.legend_ctr = value,
            _ => {}
        }
    }

    fn write_byte(&mut self, offset: usize, value: u8) -> u8 {
        self.apply_byte(offset, value);
        self.ram[offset] = value;
        value
    }

    fn write_word(&mut self, offset: usize, value: u16) -> u16 {
        self.apply_word(offset, value);
        write_le_u16(self.ram, offset, value);
        value
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.write_byte(ATTRACT_STATE, value);
    }

    pub(crate) fn set_state_word(&mut self, value: u16) {
        self.write_word(ATTRACT_STATE, value);
    }

    pub(crate) fn increment_state(&mut self) -> u8 {
        self.write_byte(ATTRACT_STATE, self.attract_scene.state().wrapping_add(1))
    }

    pub(crate) fn add_state(&mut self, value: u8) -> u8 {
        self.write_byte(
            ATTRACT_STATE,
            self.attract_scene.state().wrapping_add(value),
        )
    }

    pub(crate) fn subtract_state(&mut self, value: u8) -> u8 {
        self.write_byte(
            ATTRACT_STATE,
            self.attract_scene.state().wrapping_sub(value),
        )
    }

    pub(crate) fn set_sequence(&mut self, value: u8) {
        self.write_byte(ATTRACT_SEQUENCE, value);
    }

    pub(crate) fn increment_sequence(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_SEQUENCE,
            self.attract_scene.sequence.wrapping_add(1),
        )
    }

    pub(crate) fn set_scene_timer(&mut self, value: u8) {
        self.write_byte(ATTRACT_SCENE_TIMER, value);
    }

    pub(crate) fn decrement_scene_timer(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_SCENE_TIMER,
            self.attract_scene.scene_timer.wrapping_sub(1),
        )
    }

    pub(crate) fn set_scene_substep(&mut self, value: u8) {
        self.write_byte(ATTRACT_SCENE_SUBSTEP, value);
    }

    pub(crate) fn increment_scene_substep(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_SCENE_SUBSTEP,
            self.attract_scene.scene_substep.wrapping_add(1),
        )
    }

    pub(crate) fn set_x_base(&mut self, value: u8) {
        self.write_byte(ATTRACT_X_BASE, value);
    }

    pub(crate) fn set_x_base_high(&mut self, value: u8) {
        self.write_byte(ATTRACT_X_BASE_HI, value);
    }

    pub(crate) fn set_y_base(&mut self, value: u8) {
        self.write_byte(ATTRACT_Y_BASE, value);
    }

    pub(crate) fn set_story_text_pointer(&mut self, value: u16) {
        self.write_word(ATTRACT_STORY_TEXT_POINTER, value);
    }

    pub(crate) fn set_oam_index(&mut self, value: u8) {
        self.write_byte(ATTRACT_OAM_IDX, value);
    }

    pub(crate) fn advance_oam_index_by(&mut self, value: u8) -> u8 {
        self.write_byte(
            ATTRACT_OAM_IDX,
            self.attract_scene.oam_index.wrapping_add(value),
        )
    }

    pub(crate) fn set_maiden_warp_step(&mut self, value: u8) {
        self.write_byte(ATTRACT_MAIDEN_WARP_STEP, value);
    }

    pub(crate) fn increment_maiden_warp_step(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_MAIDEN_WARP_STEP,
            self.attract_scene.maiden_warp_step.wrapping_add(1),
        )
    }

    pub(crate) fn decrement_maiden_warp_step(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_MAIDEN_WARP_STEP,
            self.attract_scene.maiden_warp_step.wrapping_sub(1),
        )
    }

    pub(crate) fn set_intro_step_index(&mut self, value: u8) {
        self.write_byte(INTRO_STEP_INDEX, value);
    }

    pub(crate) fn clear_intro_step_state_block(&mut self) {
        self.ram[INTRO_STEP_INDEX..INTRO_STEP_INDEX + 7 * 16].fill(0);
        self.attract_scene.intro_step_index = 0;
        self.attract_scene.intro_step_timer = 0;
        self.attract_scene.intro_frame_counter = 0;
    }

    pub(crate) fn increment_intro_step_index(&mut self) -> u8 {
        self.write_byte(
            INTRO_STEP_INDEX,
            self.attract_scene.intro_step_index.wrapping_add(1),
        )
    }

    pub(crate) fn set_intro_step_timer(&mut self, value: u8) {
        self.write_byte(INTRO_STEP_TIMER, value);
    }

    pub(crate) fn increment_intro_step_timer(&mut self) -> u8 {
        self.write_byte(
            INTRO_STEP_TIMER,
            self.attract_scene.intro_step_timer.wrapping_add(1),
        )
    }

    pub(crate) fn decrement_intro_step_timer(&mut self) -> u8 {
        self.write_byte(
            INTRO_STEP_TIMER,
            self.attract_scene.intro_step_timer.wrapping_sub(1),
        )
    }

    pub(crate) fn increment_intro_frame_counter(&mut self) -> u8 {
        self.write_byte(
            INTRO_FRAME_CTR,
            self.attract_scene.intro_frame_counter.wrapping_add(1),
        )
    }

    pub(crate) fn set_intro_did_run_step(&mut self, value: u8) {
        self.write_byte(INTRO_DID_RUN_STEP, value);
    }

    pub(crate) fn clear_intro_did_run_step(&mut self) {
        self.write_byte(INTRO_DID_RUN_STEP, 0);
    }

    pub(crate) fn mark_intro_did_run_step(&mut self) {
        self.write_byte(INTRO_DID_RUN_STEP, 1);
    }

    pub(crate) fn set_intro_palette_flash_count(&mut self, value: u8) {
        self.write_byte(INTRO_TIMES_PAL_FLASH, value);
    }

    pub(crate) fn clear_intro_palette_flash_count(&mut self) {
        self.write_byte(INTRO_TIMES_PAL_FLASH, 0);
    }

    pub(crate) fn decrement_intro_palette_flash_count(&mut self) -> u8 {
        self.write_byte(
            INTRO_TIMES_PAL_FLASH,
            self.attract_scene.intro_palette_flash_count.wrapping_sub(1),
        )
    }

    pub(crate) fn increment_legend_flag(&mut self) {
        self.write_byte(
            ATTRACT_LEGEND_FLAG,
            self.attract_scene.legend_flag.wrapping_add(1),
        );
    }

    pub(crate) fn clear_legend_flag(&mut self) {
        self.write_byte(ATTRACT_LEGEND_FLAG, 0);
    }

    pub(crate) fn clear_next_legend_gfx(&mut self) {
        self.write_byte(ATTRACT_NEXT_LEGEND_GFX, 0);
    }

    pub(crate) fn advance_next_legend_gfx(&mut self) {
        self.write_byte(
            ATTRACT_NEXT_LEGEND_GFX,
            self.attract_scene.next_legend_gfx.wrapping_add(2),
        );
    }

    pub(crate) fn set_bg2_vofs_backup(&mut self, value: u16) {
        self.write_word(ATTRACT_BG2_VOFS_BACKUP, value);
    }

    pub(crate) fn set_throne_fade_timer(&mut self, value: u8) {
        self.write_byte(ATTRACT_THRONE_FADE_TIMER, value);
    }

    pub(crate) fn decrement_throne_fade_timer(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_THRONE_FADE_TIMER,
            self.attract_scene.throne_fade_timer.wrapping_sub(1),
        )
    }

    pub(crate) fn set_prison_zelda_y_base(&mut self, value: u8) {
        self.write_byte(ATTRACT_PRISON_ZELDA_Y_BASE, value);
    }

    pub(crate) fn decrement_prison_zelda_y_base(&mut self) {
        self.write_byte(
            ATTRACT_PRISON_ZELDA_Y_BASE,
            self.attract_scene.prison_zelda_y_base.wrapping_sub(1),
        );
    }

    pub(crate) fn set_anim_step_counter(&mut self, value: u8) {
        self.write_byte(ATTRACT_ANIM_STEP_COUNTER, value);
    }

    pub(crate) fn decrement_anim_step_counter(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_ANIM_STEP_COUNTER,
            self.attract_scene.anim_step_counter.wrapping_sub(1),
        )
    }

    pub(crate) fn increment_anim_step_counter(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_ANIM_STEP_COUNTER,
            self.attract_scene.anim_step_counter.wrapping_add(1),
        )
    }

    pub(crate) fn set_soldier_anim_step(&mut self, value: u8) {
        self.write_byte(ATTRACT_SOLDIER_ANIM_STEP, value);
    }

    pub(crate) fn increment_soldier_anim_step(&mut self) {
        self.write_byte(
            ATTRACT_SOLDIER_ANIM_STEP,
            self.attract_scene.soldier_anim_step.wrapping_add(1),
        );
    }

    pub(crate) fn set_prison_soldier_x_lo(&mut self, value: u8) {
        self.write_byte(ATTRACT_PRISON_SOLDIER_X_LO, value);
    }

    pub(crate) fn set_scene_frame_counter(&mut self, value: u8) {
        self.write_byte(ATTRACT_SCENE_FRAME_COUNTER, value);
    }

    pub(crate) fn increment_scene_frame_counter(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_SCENE_FRAME_COUNTER,
            self.attract_scene.scene_frame_counter.wrapping_add(1),
        )
    }

    pub(crate) fn decrement_scene_frame_counter(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_SCENE_FRAME_COUNTER,
            self.attract_scene.scene_frame_counter.wrapping_sub(1),
        )
    }

    pub(crate) fn increment_scene_done_flag(&mut self) {
        self.write_byte(
            ATTRACT_SCENE_DONE_FLAG,
            self.attract_scene.scene_done_flag.wrapping_add(1),
        );
    }

    pub(crate) fn set_legend_ctr(&mut self, value: u16) {
        self.write_word(ATTRACT_LEGEND_CTR, value);
    }

    pub(crate) fn decrement_legend_ctr(&mut self) -> u16 {
        self.write_word(
            ATTRACT_LEGEND_CTR,
            self.attract_scene.legend_ctr.wrapping_sub(1),
        )
    }

    pub(crate) fn set_fade_in_complete_flag(&mut self, value: u8) {
        self.write_byte(ATTRACT_FADE_IN_COMPLETE_FLAG, value);
    }

    pub(crate) fn increment_fade_in_complete_flag(&mut self) {
        self.write_byte(
            ATTRACT_FADE_IN_COMPLETE_FLAG,
            self.attract_scene.fade_in_complete_flag.wrapping_add(1),
        );
    }

    pub(crate) fn clear_fade_in_done_flag(&mut self) {
        self.write_byte(ATTRACT_FADE_IN_DONE_FLAG, 0);
    }

    pub(crate) fn increment_fade_in_done_flag(&mut self) {
        self.write_byte(
            ATTRACT_FADE_IN_DONE_FLAG,
            self.attract_scene.fade_in_done_flag.wrapping_add(1),
        );
    }

    pub(crate) fn clear_substep_delay_counter(&mut self) {
        self.write_byte(ATTRACT_SUBSTEP_DELAY_COUNTER, 0);
    }

    pub(crate) fn increment_substep_delay_counter(&mut self) {
        self.write_byte(
            ATTRACT_SUBSTEP_DELAY_COUNTER,
            self.attract_scene.substep_delay_counter.wrapping_add(1),
        );
    }

    pub(crate) fn set_maiden_warp_timer_a(&mut self, value: u8) {
        self.write_byte(ATTRACT_MAIDEN_WARP_TIMER_A, value);
    }

    pub(crate) fn decrement_maiden_warp_timer_a(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_MAIDEN_WARP_TIMER_A,
            self.attract_scene.maiden_warp_timer_a.wrapping_sub(1),
        )
    }

    pub(crate) fn set_maiden_warp_timer_b(&mut self, value: u8) {
        self.write_byte(ATTRACT_MAIDEN_WARP_TIMER_B, value);
    }

    pub(crate) fn decrement_maiden_warp_timer_b(&mut self) -> u8 {
        self.write_byte(
            ATTRACT_MAIDEN_WARP_TIMER_B,
            self.attract_scene.maiden_warp_timer_b.wrapping_sub(1),
        )
    }

    pub(crate) fn set_mode7_zoom_timer(&mut self, value: u8) {
        self.write_byte(TIMER_FOR_MODE7_ZOOM, value);
    }

    pub(crate) fn decrement_mode7_zoom_timer(&mut self) {
        self.write_byte(
            TIMER_FOR_MODE7_ZOOM,
            self.attract_scene.mode7_zoom_timer.wrapping_sub(1),
        );
    }
}

pub(crate) struct NativeIntroSceneBridgeMut<'a> {
    intro_scene: &'a mut IntroSceneState,
    ram: &'a mut [u8],
}

impl<'a> NativeIntroSceneBridgeMut<'a> {
    pub(crate) fn new(intro_scene: &'a mut IntroSceneState, ram: &'a mut [u8]) -> Self {
        Self { intro_scene, ram }
    }

    fn sync(&mut self) {
        self.intro_scene.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.intro_scene, IntroSceneState::load_from_ram(self.ram));
    }

    pub(crate) fn pause_triangle_motion(&mut self) {
        self.intro_scene.pause_triangle_motion();
        self.sync();
    }

    pub(crate) fn resume_triangle_motion(&mut self) {
        self.intro_scene.resume_triangle_motion();
        self.sync();
    }

    pub(crate) fn set_sprite_oam_cursor(&mut self, value: u16) {
        self.intro_scene.sprite_oam_cursor = value;
        self.sync();
    }

    pub(crate) fn allocate_oam_entries(&mut self, entry_count: usize) -> usize {
        let cursor = self.intro_scene.allocate_oam_entries(entry_count);
        self.sync();
        cursor
    }

    pub(crate) fn set_triforce_countdown(&mut self, value: u16) {
        self.intro_scene.triforce_countdown = value;
        self.sync();
    }

    pub(crate) fn decrement_triforce_countdown(&mut self) {
        self.intro_scene.decrement_triforce_countdown();
        self.sync();
    }
}

pub(crate) struct NativeIntroActorBridgeMut<'a> {
    state: &'a mut IntroActorState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeIntroActorBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut IntroActorState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn actor_mut(&mut self) -> Option<&mut IntroActorSlotState> {
        self.state.slots.get_mut(self.slot)
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, IntroActorState::load_from_ram(self.ram));
    }

    pub(crate) fn set_x(&mut self, value: i16) {
        if let Some(actor) = self.actor_mut() {
            actor.set_x(value);
            self.sync();
        }
    }

    pub(crate) fn set_y(&mut self, value: i16) {
        if let Some(actor) = self.actor_mut() {
            actor.set_y(value);
            self.sync();
        }
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.x_low = value;
            self.sync();
        }
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.y_low = value;
            self.sync();
        }
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.x_velocity = value;
            self.sync();
        }
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.y_velocity = value;
            self.sync();
        }
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.x_velocity = actor.x_velocity.wrapping_add(value);
            self.sync();
        }
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.y_velocity = actor.y_velocity.wrapping_add(value);
            self.sync();
        }
    }

    pub(crate) fn set_init_phase(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.init_phase = value;
            self.sync();
        }
    }

    pub(crate) fn increment_init_phase(&mut self) {
        if let Some(actor) = self.actor_mut() {
            actor.init_phase = actor.init_phase.wrapping_add(1);
            self.sync();
        }
    }

    pub(crate) fn set_subtype(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.subtype = value;
            self.sync();
        }
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        if let Some(actor) = self.actor_mut() {
            actor.state = value;
            self.sync();
        }
    }

    pub(crate) fn increment_state(&mut self) {
        if let Some(actor) = self.actor_mut() {
            actor.state = actor.state.wrapping_add(1);
            self.sync();
        }
    }

    pub(crate) fn move_x(&mut self) {
        if let Some(actor) = self.actor_mut() {
            actor.move_x();
            self.sync();
        }
    }

    pub(crate) fn move_y(&mut self) {
        if let Some(actor) = self.actor_mut() {
            actor.move_y();
            self.sync();
        }
    }
}

pub(crate) struct NativeEndingCreditBridgeMut<'a> {
    credits: &'a mut EndingCreditState,
    ram: &'a mut [u8],
}

impl<'a> NativeEndingCreditBridgeMut<'a> {
    pub(crate) fn new(credits: &'a mut EndingCreditState, ram: &'a mut [u8]) -> Self {
        Self { credits, ram }
    }

    fn sync(&mut self) {
        self.credits.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.credits, EndingCreditState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_palace_death_count_digit_step(&mut self) {
        self.credits.clear_palace_death_count_digit_step();
        self.sync();
    }

    pub(crate) fn set_palace_death_count_digit_step(&mut self, value: u16) {
        self.credits.palace_death_count_digit_step = value;
        self.sync();
    }

    pub(crate) fn advance_palace_death_count_digit_step(&mut self) {
        let value = self.credits.palace_death_count_digit_step.wrapping_add(1);
        self.set_palace_death_count_digit_step(value);
    }

    pub(crate) fn set_death_count_digit_tile_base(&mut self, value: u16) {
        self.credits.death_count_digit_tile_base = value;
        self.sync();
    }
}
