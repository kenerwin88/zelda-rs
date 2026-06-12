use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};

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
    pub(crate) intro_scene: IntroSceneState,
    pub(crate) credits: EndingCreditState,
}

impl EndingState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            intro_scene: IntroSceneState::load_from_ram(ram),
            credits: EndingCreditState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.intro_scene.write_to_ram(ram);
        self.credits.write_to_ram(ram);
    }
}

pub(crate) struct NativeIntroSceneBridgeMut<'a> {
    intro_scene: &'a mut IntroSceneState,
    ram: &'a mut [u8],
}

impl<'a> NativeIntroSceneBridgeMut<'a> {
    pub(crate) fn new(intro_scene: &'a mut IntroSceneState, ram: &'a mut [u8]) -> Self {
        *intro_scene = IntroSceneState::load_from_ram(ram);
        Self { intro_scene, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.intro_scene, IntroSceneState::load_from_ram(self.ram));
    }

    pub(crate) fn pause_triangle_motion(&mut self) {
        self.intro_scene.pause_triangle_motion();
        self.ram[INTRO_WANT_DOUBLE_RET] = self.intro_scene.triangle_motion_pause;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn resume_triangle_motion(&mut self) {
        self.intro_scene.resume_triangle_motion();
        self.ram[INTRO_WANT_DOUBLE_RET] = self.intro_scene.triangle_motion_pause;
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_sprite_oam_cursor(&mut self, value: u16) {
        self.intro_scene.sprite_oam_cursor = value;
        write_le_u16(self.ram, INTRO_SPRITE_ALLOC, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn allocate_oam_entries(&mut self, entry_count: usize) -> usize {
        let cursor = self.intro_scene.allocate_oam_entries(entry_count);
        write_le_u16(
            self.ram,
            INTRO_SPRITE_ALLOC,
            self.intro_scene.sprite_oam_cursor,
        );
        self.debug_assert_matches_ram();
        cursor
    }

    pub(crate) fn set_triforce_countdown(&mut self, value: u16) {
        self.intro_scene.triforce_countdown = value;
        write_le_u16(self.ram, TRIFORCE_CTR, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn decrement_triforce_countdown(&mut self) {
        self.intro_scene.decrement_triforce_countdown();
        write_le_u16(self.ram, TRIFORCE_CTR, self.intro_scene.triforce_countdown);
        self.debug_assert_matches_ram();
    }
}

pub(crate) struct NativeEndingCreditBridgeMut<'a> {
    credits: &'a mut EndingCreditState,
    ram: &'a mut [u8],
}

impl<'a> NativeEndingCreditBridgeMut<'a> {
    pub(crate) fn new(credits: &'a mut EndingCreditState, ram: &'a mut [u8]) -> Self {
        *credits = EndingCreditState::load_from_ram(ram);
        Self { credits, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.credits, EndingCreditState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_palace_death_count_digit_step(&mut self) {
        self.credits.clear_palace_death_count_digit_step();
        write_le_u16(self.ram, ENDING_WHICH_DUNG, 0);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn set_palace_death_count_digit_step(&mut self, value: u16) {
        self.credits.palace_death_count_digit_step = value;
        write_le_u16(self.ram, ENDING_WHICH_DUNG, value);
        self.debug_assert_matches_ram();
    }

    pub(crate) fn advance_palace_death_count_digit_step(&mut self) {
        let value = self.credits.palace_death_count_digit_step.wrapping_add(1);
        self.set_palace_death_count_digit_step(value);
    }

    pub(crate) fn set_death_count_digit_tile_base(&mut self, value: u16) {
        self.credits.death_count_digit_tile_base = value;
        write_le_u16(self.ram, ENDING_CREDIT_DIGIT_CHAR, value);
        self.debug_assert_matches_ram();
    }
}
