use super::*;

pub(crate) struct EffectAngleScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> EffectAngleScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn angle(&self, slot: usize) -> u8 {
        byte(self.ram, EFFECT_ANGLE_SCRATCH + slot)
    }

    pub(crate) fn trailing_angle(&self) -> u8 {
        byte(self.ram, EFFECT_ANGLE_SCRATCH + 4)
    }

    pub(crate) fn radial_radius(&self) -> u8 {
        byte(self.ram, EFFECT_ANGLE_SCRATCH + 8)
    }
}

pub(crate) struct EffectAngleScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> EffectAngleScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        self.ram[EFFECT_ANGLE_SCRATCH + slot] = value;
    }

    pub(crate) fn set_angles4(&mut self, values: &[u8], start: usize) {
        for slot in 0..4 {
            self.set_angle(slot, values[start + slot]);
        }
    }

    pub(crate) fn add_angle_mod64(&mut self, slot: usize, value: u8) -> u8 {
        let angle = self.ram[EFFECT_ANGLE_SCRATCH + slot].wrapping_add(value) & 0x3f;
        self.ram[EFFECT_ANGLE_SCRATCH + slot] = angle;
        angle
    }

    pub(crate) fn set_trailing_angle(&mut self, value: u8) {
        self.ram[EFFECT_ANGLE_SCRATCH + 4] = value;
    }

    pub(crate) fn add_trailing_angle_mod64(&mut self, value: u8) -> u8 {
        let angle = self.ram[EFFECT_ANGLE_SCRATCH + 4].wrapping_add(value) & 0x3f;
        self.ram[EFFECT_ANGLE_SCRATCH + 4] = angle;
        angle
    }

    pub(crate) fn set_radial_radius(&mut self, value: u8) {
        self.ram[EFFECT_ANGLE_SCRATCH + 8] = value;
    }
}

pub(crate) struct QuakeBoltView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> QuakeBoltView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, QUAKE_BOLT_TIMER + self.slot)
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, QUAKE_BOLT_PHASE + self.slot)
    }
}

pub(crate) struct QuakeBoltViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> QuakeBoltViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[QUAKE_BOLT_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.ram[QUAKE_BOLT_TIMER + self.slot].wrapping_sub(1);
        self.ram[QUAKE_BOLT_TIMER + self.slot] = value;
        value
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[QUAKE_BOLT_PHASE + self.slot] = value;
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.ram[QUAKE_BOLT_PHASE + self.slot].wrapping_add(1);
        self.ram[QUAKE_BOLT_PHASE + self.slot] = value;
        value
    }
}

pub(crate) struct QuakeSpellScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> QuakeSpellScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn active_bolt_limit(&self) -> u8 {
        byte(self.ram, QUAKE_ACTIVE_BOLT_LIMIT)
    }

    pub(crate) fn pending_step(&self) -> u8 {
        byte(self.ram, QUAKE_PENDING_STEP)
    }

    pub(crate) fn origin_x(&self) -> u16 {
        read_le_u16(self.ram, QUAKE_ORIGIN_X)
    }

    pub(crate) fn origin_y(&self) -> u16 {
        read_le_u16(self.ram, QUAKE_ORIGIN_Y)
    }

    pub(crate) fn screen_shake_y(&self) -> u16 {
        read_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y)
    }
}

pub(crate) struct QuakeSpellScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> QuakeSpellScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_active_bolt_limit(&mut self, value: u8) {
        self.ram[QUAKE_ACTIVE_BOLT_LIMIT] = value;
    }

    pub(crate) fn set_pending_step(&mut self, value: u8) {
        self.ram[QUAKE_PENDING_STEP] = value;
    }

    pub(crate) fn set_origin(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, QUAKE_ORIGIN_X, x);
        write_le_u16(self.ram, QUAKE_ORIGIN_Y, y);
    }

    pub(crate) fn set_screen_shake_y(&mut self, value: u16) {
        write_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y, value);
    }

    pub(crate) fn invert_screen_shake_y(&mut self) -> u16 {
        let value = read_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y);
        write_le_u16(self.ram, QUAKE_SCREEN_SHAKE_Y, 0u16.wrapping_sub(value));
        value
    }
}

pub(crate) struct BombosFireColumnView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BombosFireColumnView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_TIMER + self.slot)
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_PHASE + self.slot)
    }

    pub(crate) fn radial_angle(&self) -> u8 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot)
    }

    pub(crate) fn x(&self) -> u16 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_X_LO + self.slot) as u16
            | ((byte(self.ram, BOMBOS_FIRE_COLUMN_X_HI + self.slot) as u16) << 8)
    }

    pub(crate) fn y(&self) -> u16 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_Y_LO + self.slot) as u16
            | ((byte(self.ram, BOMBOS_FIRE_COLUMN_Y_HI + self.slot) as u16) << 8)
    }
}

pub(crate) struct BombosFireColumnViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BombosFireColumnViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot].wrapping_sub(1);
        self.ram[BOMBOS_FIRE_COLUMN_TIMER + self.slot] = value;
        value
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot] = value;
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot].wrapping_add(1);
        self.ram[BOMBOS_FIRE_COLUMN_PHASE + self.slot] = value;
        value
    }

    pub(crate) fn add_radial_angle(&mut self, value: u8) -> u8 {
        let angle = self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot].wrapping_add(value);
        self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot] = angle;
        angle
    }

    pub(crate) fn set_radial_angle(&mut self, value: u8) {
        self.ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + self.slot] = value;
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[BOMBOS_FIRE_COLUMN_X_LO + self.slot] = x as u8;
        self.ram[BOMBOS_FIRE_COLUMN_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[BOMBOS_FIRE_COLUMN_Y_LO + self.slot] = y as u8;
        self.ram[BOMBOS_FIRE_COLUMN_Y_HI + self.slot] = (y >> 8) as u8;
    }
}

pub(crate) struct BombosBlastView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BombosBlastView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, BOMBOS_BLAST_PHASE + self.slot)
    }
}

pub(crate) struct BombosBlastViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BombosBlastViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[BOMBOS_BLAST_PHASE + self.slot] = value;
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.ram[BOMBOS_BLAST_PHASE + self.slot].wrapping_add(1);
        self.ram[BOMBOS_BLAST_PHASE + self.slot] = value;
        value
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[BOMBOS_BLAST_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.ram[BOMBOS_BLAST_TIMER + self.slot].wrapping_sub(1);
        self.ram[BOMBOS_BLAST_TIMER + self.slot] = value;
        value
    }
}

pub(crate) struct BombosSpellScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> BombosSpellScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn mode(&self) -> u8 {
        byte(self.ram, BOMBOS_MODE)
    }

    pub(crate) fn fire_column_radius(&self) -> u8 {
        byte(self.ram, BOMBOS_FIRE_COLUMN_RADIUS)
    }

    pub(crate) fn blast_release_locked(&self) -> bool {
        byte(self.ram, BOMBOS_BLAST_RELEASE_LOCKED) != 0
    }

    pub(crate) fn fire_column_seed_x(&self, slot: usize) -> u16 {
        read_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_X + slot * 2)
    }

    pub(crate) fn fire_column_seed_y(&self, slot: usize) -> u16 {
        read_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_Y + slot * 2)
    }

    pub(crate) fn blast_x(&self, slot: usize) -> u16 {
        read_le_u16(self.ram, BOMBOS_BLAST_X + slot * 2)
    }

    pub(crate) fn blast_y(&self, slot: usize) -> u16 {
        read_le_u16(self.ram, BOMBOS_BLAST_Y + slot * 2)
    }
}

pub(crate) struct BombosSpellScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> BombosSpellScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_mode(&mut self, value: u8) {
        self.ram[BOMBOS_MODE] = value;
    }

    pub(crate) fn set_fire_column_radius(&mut self, value: u8) {
        self.ram[BOMBOS_FIRE_COLUMN_RADIUS] = value;
    }

    pub(crate) fn grow_fire_column_radius(&mut self, value: u8, limit: u8) -> u8 {
        let next = self.ram[BOMBOS_FIRE_COLUMN_RADIUS].wrapping_add(value);
        let radius = if next >= limit { limit } else { next };
        self.ram[BOMBOS_FIRE_COLUMN_RADIUS] = radius;
        radius
    }

    pub(crate) fn set_blast_release_locked(&mut self, value: bool) {
        self.ram[BOMBOS_BLAST_RELEASE_LOCKED] = u8::from(value);
    }

    pub(crate) fn set_blast_release_countdown(&mut self, value: u8) {
        self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = value;
    }

    pub(crate) fn tick_blast_release_countdown(&mut self) -> u8 {
        let value = self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN].wrapping_sub(1);
        self.ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = value;
        value
    }

    pub(crate) fn set_fire_column_seed_position(&mut self, slot: usize, x: u16, y: u16) {
        write_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_X + slot * 2, x);
        write_le_u16(self.ram, BOMBOS_FIRE_COLUMN_SEED_Y + slot * 2, y);
    }

    pub(crate) fn set_blast_position(&mut self, slot: usize, x: u16, y: u16) {
        write_le_u16(self.ram, BOMBOS_BLAST_X + slot * 2, x);
        write_le_u16(self.ram, BOMBOS_BLAST_Y + slot * 2, y);
    }
}

pub(crate) struct TowerSealOrbitView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> TowerSealOrbitView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn angle(&self) -> u8 {
        byte(self.ram, TOWER_SEAL_ORBIT_ANGLE + self.slot)
    }
}

pub(crate) struct TowerSealOrbitViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> TowerSealOrbitViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_angle(&mut self, value: u8) {
        self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot] = value;
    }

    pub(crate) fn advance_angle_mod64(&mut self) -> u8 {
        let angle = self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot].wrapping_add(1) & 0x3f;
        self.ram[TOWER_SEAL_ORBIT_ANGLE + self.slot] = angle;
        angle
    }

    pub(crate) fn set_base_sparkle_position(&mut self, x: u16, y: u16) {
        self.ram[TOWER_SEAL_BASE_SPARKLE_X_LO + self.slot] = x as u8;
        self.ram[TOWER_SEAL_BASE_SPARKLE_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + self.slot] = y as u8;
        self.ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + self.slot] = (y >> 8) as u8;
    }
}

pub(crate) struct TowerSealSparkleView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> TowerSealSparkleView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, TOWER_SEAL_SPARKLE_PHASE + self.slot)
    }

    pub(crate) fn is_free(&self) -> bool {
        self.phase() == 0xff
    }

    pub(crate) fn x(&self) -> u16 {
        byte(self.ram, TOWER_SEAL_SPARKLE_X_LO + self.slot) as u16
            | ((byte(self.ram, TOWER_SEAL_SPARKLE_X_HI + self.slot) as u16) << 8)
    }

    pub(crate) fn y(&self) -> u16 {
        byte(self.ram, TOWER_SEAL_SPARKLE_Y_LO + self.slot) as u16
            | ((byte(self.ram, TOWER_SEAL_SPARKLE_Y_HI + self.slot) as u16) << 8)
    }
}

pub(crate) struct TowerSealSparkleViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> TowerSealSparkleViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot] = value;
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot].wrapping_sub(1);
        self.ram[TOWER_SEAL_SPARKLE_TIMER + self.slot] = timer;
        timer
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot].wrapping_add(1);
        self.ram[TOWER_SEAL_SPARKLE_PHASE + self.slot] = phase;
        phase
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[TOWER_SEAL_SPARKLE_X_LO + self.slot] = x as u8;
        self.ram[TOWER_SEAL_SPARKLE_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[TOWER_SEAL_SPARKLE_Y_LO + self.slot] = y as u8;
        self.ram[TOWER_SEAL_SPARKLE_Y_HI + self.slot] = (y >> 8) as u8;
    }

    pub(crate) fn base_sparkle_position(&self, base: usize) -> (u16, u16) {
        let x = self.ram[TOWER_SEAL_BASE_SPARKLE_X_LO + base] as u16
            | ((self.ram[TOWER_SEAL_BASE_SPARKLE_X_HI + base] as u16) << 8);
        let y = self.ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + base] as u16
            | ((self.ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + base] as u16) << 8);
        (x, y)
    }
}

pub(crate) struct TowerSealScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> TowerSealScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn ring_radius(&self) -> u8 {
        byte(self.ram, TOWER_SEAL_RING_RADIUS)
    }

    pub(crate) fn center_x(&self) -> u16 {
        read_le_u16(self.ram, TOWER_SEAL_CENTER_X)
    }

    pub(crate) fn center_y(&self) -> u16 {
        read_le_u16(self.ram, TOWER_SEAL_CENTER_Y)
    }
}

pub(crate) struct TowerSealScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> TowerSealScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_ring_radius(&mut self, value: u8) {
        self.ram[TOWER_SEAL_RING_RADIUS] = value;
    }

    pub(crate) fn set_center(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, TOWER_SEAL_CENTER_X, x);
        write_le_u16(self.ram, TOWER_SEAL_CENTER_Y, y);
    }

    pub(crate) fn tick_wait_countdown(&mut self) -> u8 {
        let value = self.ram[TOWER_SEAL_WAIT_COUNTDOWN].wrapping_sub(1);
        self.ram[TOWER_SEAL_WAIT_COUNTDOWN] = value;
        value
    }

    pub(crate) fn set_wait_countdown(&mut self, value: u8) {
        self.ram[TOWER_SEAL_WAIT_COUNTDOWN] = value;
    }
}

pub(crate) struct BlastWallExplosionView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BlastWallExplosionView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, BLAST_WALL_EXPLOSION_PHASE + self.slot)
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, BLAST_WALL_EXPLOSION_TIMER + self.slot)
    }
}

pub(crate) struct BlastWallExplosionViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BlastWallExplosionViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot] = value;
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot].wrapping_add(1);
        self.ram[BLAST_WALL_EXPLOSION_PHASE + self.slot] = phase;
        phase
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot].wrapping_sub(1);
        self.ram[BLAST_WALL_EXPLOSION_TIMER + self.slot] = timer;
        timer
    }
}

pub(crate) struct BlastWallFragmentView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BlastWallFragmentView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn y(&self) -> u16 {
        read_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2)
    }

    pub(crate) fn x(&self) -> u16 {
        read_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2)
    }
}

pub(crate) struct BlastWallFragmentViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BlastWallFragmentViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2, x);
        write_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2, y);
    }

    pub(crate) fn offset(&mut self, x_delta: i16, y_delta: i16) -> (u16, u16) {
        let x = read_le_u16(self.ram, BLAST_WALL_FRAGMENT_X + self.slot * 2)
            .wrapping_add(x_delta as u16);
        let y = read_le_u16(self.ram, BLAST_WALL_FRAGMENT_Y + self.slot * 2)
            .wrapping_add(y_delta as u16);
        self.set_position(x, y);
        (x, y)
    }
}

pub(crate) struct BlastWallFireballView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BlastWallFireballView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, BLAST_WALL_FIREBALL_TIMER + self.slot)
    }
}

pub(crate) struct BlastWallFireballViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BlastWallFireballViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot].wrapping_sub(1);
        self.ram[BLAST_WALL_FIREBALL_TIMER + self.slot] = timer;
        timer
    }
}

pub(crate) struct BlastWallScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> BlastWallScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, BLAST_WALL_DIRECTION)
    }

    pub(crate) fn center_x(&self) -> u16 {
        read_le_u16(self.ram, BLAST_WALL_CENTER_X)
    }

    pub(crate) fn center_y(&self) -> u16 {
        read_le_u16(self.ram, BLAST_WALL_CENTER_Y)
    }
}

pub(crate) struct BlastWallScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> BlastWallScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_entry_state(&mut self) {
        self.ram[BLAST_WALL_ENTRY_STATE] = 0;
    }

    pub(crate) fn clear_secondary_state(&mut self) {
        self.ram[BLAST_WALL_SECONDARY_STATE] = 0;
    }

    pub(crate) fn offset_center(&mut self, x_delta: i8, y_delta: i8) -> (u16, u16) {
        let y = read_le_u16(self.ram, BLAST_WALL_CENTER_Y).wrapping_add(y_delta as i16 as u16);
        let x = read_le_u16(self.ram, BLAST_WALL_CENTER_X).wrapping_add(x_delta as i16 as u16);
        write_le_u16(self.ram, BLAST_WALL_CENTER_Y, y);
        write_le_u16(self.ram, BLAST_WALL_CENTER_X, x);
        (x, y)
    }
}

pub(crate) struct SkullWoodsFireView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> SkullWoodsFireView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn phase(&self) -> u8 {
        byte(self.ram, SKULL_WOODS_FIRE_PHASE + self.slot)
    }

    pub(crate) fn is_finished(&self) -> bool {
        (self.phase() as i8).is_negative()
    }

    pub(crate) fn x(&self) -> u16 {
        read_le_u16(self.ram, SKULL_WOODS_FIRE_X + self.slot * 2)
    }

    pub(crate) fn y(&self) -> u16 {
        read_le_u16(self.ram, SKULL_WOODS_FIRE_Y + self.slot * 2)
    }
}

pub(crate) struct SkullWoodsFireViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> SkullWoodsFireViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.ram[SKULL_WOODS_FIRE_PHASE + self.slot] = value;
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.ram[SKULL_WOODS_FIRE_PHASE + self.slot].wrapping_add(1);
        self.ram[SKULL_WOODS_FIRE_PHASE + self.slot] = phase;
        phase
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[SKULL_WOODS_FIRE_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.ram[SKULL_WOODS_FIRE_TIMER + self.slot].wrapping_sub(1);
        self.ram[SKULL_WOODS_FIRE_TIMER + self.slot] = timer;
        timer
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, SKULL_WOODS_FIRE_X + self.slot * 2, x);
        write_le_u16(self.ram, SKULL_WOODS_FIRE_Y + self.slot * 2, y);
    }
}

pub(crate) struct SkullWoodsFireScratchView<'a> {
    ram: &'a [u8],
}

impl<'a> SkullWoodsFireScratchView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn has_started_entrance_opening(&self) -> bool {
        byte(self.ram, SKULL_WOODS_FIRE_STARTED) != 0
    }

    pub(crate) fn inner_x(&self) -> u16 {
        read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_X)
    }

    pub(crate) fn inner_y(&self) -> u16 {
        read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y)
    }
}

pub(crate) struct SkullWoodsFireScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SkullWoodsFireScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_entrance_opening_started(&mut self) {
        self.ram[SKULL_WOODS_FIRE_STARTED] = 0;
    }

    pub(crate) fn set_entrance_opening_started(&mut self) {
        self.ram[SKULL_WOODS_FIRE_STARTED] = 1;
    }

    pub(crate) fn set_inner_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_X, x);
        write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y, y);
    }

    pub(crate) fn set_outer_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, SKULL_WOODS_FIRE_OUTER_X, x);
        write_le_u16(self.ram, SKULL_WOODS_FIRE_OUTER_Y, y);
    }

    pub(crate) fn retreat_inner_y(&mut self, value: u16) -> u16 {
        let y = read_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y).wrapping_sub(value);
        write_le_u16(self.ram, SKULL_WOODS_FIRE_INNER_Y, y);
        y
    }
}

pub(crate) struct HappinessPondRupeeState {
    pub(crate) y_low: u8,
    pub(crate) y_high: u8,
    pub(crate) x_low: u8,
    pub(crate) x_high: u8,
    pub(crate) z: u8,
    pub(crate) y_velocity: u8,
    pub(crate) x_velocity: u8,
    pub(crate) z_velocity: u8,
    pub(crate) y_subpixel: u8,
    pub(crate) x_subpixel: u8,
    pub(crate) z_subpixel: u8,
    pub(crate) item_to_link: u8,
    pub(crate) timer: u8,
    pub(crate) step: u8,
}

pub(crate) struct HappinessPondRupeeView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> HappinessPondRupeeView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn is_active(&self) -> bool {
        byte(self.ram, HAPPINESS_POND_ACTIVE + self.slot) != 0
    }

    pub(crate) fn step(&self) -> u8 {
        byte(self.ram, HAPPINESS_POND_STEP + self.slot)
    }

    pub(crate) fn snapshot(&self) -> HappinessPondRupeeState {
        HappinessPondRupeeState {
            y_low: byte(self.ram, HAPPINESS_POND_Y_LO + self.slot),
            y_high: byte(self.ram, HAPPINESS_POND_Y_HI + self.slot),
            x_low: byte(self.ram, HAPPINESS_POND_X_LO + self.slot),
            x_high: byte(self.ram, HAPPINESS_POND_X_HI + self.slot),
            z: byte(self.ram, HAPPINESS_POND_Z + self.slot),
            y_velocity: byte(self.ram, HAPPINESS_POND_Y_VEL + self.slot),
            x_velocity: byte(self.ram, HAPPINESS_POND_X_VEL + self.slot),
            z_velocity: byte(self.ram, HAPPINESS_POND_Z_VEL + self.slot),
            y_subpixel: byte(self.ram, HAPPINESS_POND_Y_SUBPIXEL + self.slot),
            x_subpixel: byte(self.ram, HAPPINESS_POND_X_SUBPIXEL + self.slot),
            z_subpixel: byte(self.ram, HAPPINESS_POND_Z_SUBPIXEL + self.slot),
            item_to_link: byte(self.ram, HAPPINESS_POND_ITEM_TO_LINK + self.slot),
            timer: byte(self.ram, HAPPINESS_POND_TIMER + self.slot).saturating_sub(1),
            step: byte(self.ram, HAPPINESS_POND_STEP + self.slot),
        }
    }
}

pub(crate) struct HappinessPondRupeeViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> HappinessPondRupeeViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_active(&mut self, active: bool) {
        self.ram[HAPPINESS_POND_ACTIVE + self.slot] = u8::from(active);
    }

    pub(crate) fn clear(&mut self) {
        self.ram[HAPPINESS_POND_ACTIVE + self.slot] = 0;
    }

    pub(crate) fn initialize(
        &mut self,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
    ) {
        self.set_active(true);
        self.ram[HAPPINESS_POND_Z_VEL + self.slot] = z_velocity;
        self.ram[HAPPINESS_POND_Y_VEL + self.slot] = y_velocity;
        self.ram[HAPPINESS_POND_X_VEL + self.slot] = x_velocity;
        self.ram[HAPPINESS_POND_Z + self.slot] = 0;
        self.ram[HAPPINESS_POND_STEP + self.slot] = 0;
        self.ram[HAPPINESS_POND_TIMER + self.slot] = 16;
        self.ram[HAPPINESS_POND_ITEM_TO_LINK + self.slot] = 53;
        self.ram[HAPPINESS_POND_X_LO + self.slot] = x as u8;
        self.ram[HAPPINESS_POND_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[HAPPINESS_POND_Y_LO + self.slot] = y as u8;
        self.ram[HAPPINESS_POND_Y_HI + self.slot] = (y >> 8) as u8;
    }

    pub(crate) fn store_snapshot(&mut self, state: HappinessPondRupeeState) {
        self.ram[HAPPINESS_POND_Y_LO + self.slot] = state.y_low;
        self.ram[HAPPINESS_POND_Y_HI + self.slot] = state.y_high;
        self.ram[HAPPINESS_POND_X_LO + self.slot] = state.x_low;
        self.ram[HAPPINESS_POND_X_HI + self.slot] = state.x_high;
        self.ram[HAPPINESS_POND_Z + self.slot] = state.z;
        self.ram[HAPPINESS_POND_Y_VEL + self.slot] = state.y_velocity;
        self.ram[HAPPINESS_POND_X_VEL + self.slot] = state.x_velocity;
        self.ram[HAPPINESS_POND_Z_VEL + self.slot] = state.z_velocity;
        self.ram[HAPPINESS_POND_Y_SUBPIXEL + self.slot] = state.y_subpixel;
        self.ram[HAPPINESS_POND_X_SUBPIXEL + self.slot] = state.x_subpixel;
        self.ram[HAPPINESS_POND_Z_SUBPIXEL + self.slot] = state.z_subpixel;
        self.ram[HAPPINESS_POND_ITEM_TO_LINK + self.slot] = state.item_to_link;
        self.ram[HAPPINESS_POND_TIMER + self.slot] = state.timer;
        self.ram[HAPPINESS_POND_STEP + self.slot] = state.step;
    }
}

pub(crate) struct WeatherVaneStateView<'a> {
    ram: &'a [u8],
}

impl<'a> WeatherVaneStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn oam_offset(&self) -> u8 {
        byte(self.ram, WEATHERVANE_OAM_OFFSET)
    }
}

pub(crate) struct WeatherVaneStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> WeatherVaneStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
    }

    pub(crate) fn tick_countdown(&mut self) -> u16 {
        let value = read_le_u16(self.ram, WEATHERVANE_COUNTDOWN).wrapping_sub(1);
        write_le_u16(self.ram, WEATHERVANE_COUNTDOWN, value);
        value
    }

    pub(crate) fn set_music_latch(&mut self, value: u8) {
        self.ram[WEATHERVANE_MUSIC_LATCH] = value;
    }

    pub(crate) fn music_latch(&self) -> u8 {
        byte(self.ram, WEATHERVANE_MUSIC_LATCH)
    }

    pub(crate) fn set_source_slot(&mut self, slot: u8) {
        self.ram[WEATHERVANE_SOURCE_SLOT] = slot;
    }

    pub(crate) fn reset_oam_offset(&mut self) {
        self.ram[WEATHERVANE_OAM_OFFSET] = 0;
    }

    pub(crate) fn advance_oam_offset(&mut self, value: u8) {
        self.ram[WEATHERVANE_OAM_OFFSET] = self.ram[WEATHERVANE_OAM_OFFSET].wrapping_add(value);
    }
}

pub(crate) struct WeatherVaneDebrisState {
    pub(crate) y: u16,
    pub(crate) x: u16,
    pub(crate) z: u8,
    pub(crate) y_velocity: u8,
    pub(crate) x_velocity: u8,
    pub(crate) z_velocity: u8,
    pub(crate) draw_state: u8,
}

pub(crate) struct WeatherVaneDebrisView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> WeatherVaneDebrisView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn is_finished(&self) -> bool {
        byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot) == 0xff
    }

    pub(crate) fn snapshot(&self) -> WeatherVaneDebrisState {
        WeatherVaneDebrisState {
            y: packed_position(
                self.ram,
                WEATHERVANE_Y_LO + self.slot,
                WEATHERVANE_Y_HI + self.slot,
            ),
            x: packed_position(
                self.ram,
                WEATHERVANE_X_LO + self.slot,
                WEATHERVANE_X_HI + self.slot,
            ),
            z: byte(self.ram, WEATHERVANE_Z + self.slot),
            y_velocity: byte(self.ram, WEATHERVANE_Y_VELOCITY + self.slot),
            x_velocity: byte(self.ram, WEATHERVANE_X_VELOCITY + self.slot),
            z_velocity: byte(self.ram, WEATHERVANE_Z_VELOCITY + self.slot),
            draw_state: byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot),
        }
    }
}

pub(crate) struct WeatherVaneDebrisViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> WeatherVaneDebrisViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn initialize(
        &mut self,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
        z: u8,
        draw_state: u8,
    ) {
        self.ram[WEATHERVANE_Y_VELOCITY + self.slot] = y_velocity;
        self.ram[WEATHERVANE_X_VELOCITY + self.slot] = x_velocity;
        self.ram[WEATHERVANE_Z_VELOCITY + self.slot] = z_velocity;
        self.ram[WEATHERVANE_Y_LO + self.slot] = y as u8;
        self.ram[WEATHERVANE_Y_HI + self.slot] = (y >> 8) as u8;
        self.ram[WEATHERVANE_X_LO + self.slot] = x as u8;
        self.ram[WEATHERVANE_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[WEATHERVANE_Z + self.slot] = z;
        self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = 1;
        self.ram[WEATHERVANE_DRAW_STATE + self.slot] = draw_state;
    }

    pub(crate) fn tick_animation(&mut self) -> u8 {
        let timer = self.ram[WEATHERVANE_ANIM_TIMER + self.slot].wrapping_sub(1);
        self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = timer;
        if (timer as i8).is_negative() {
            self.ram[WEATHERVANE_ANIM_TIMER + self.slot] = 1;
            self.ram[WEATHERVANE_DRAW_STATE + self.slot] ^= 1;
        }
        self.ram[WEATHERVANE_DRAW_STATE + self.slot]
    }

    pub(crate) fn tick_z_velocity(&mut self) -> u8 {
        let z_velocity = self.ram[WEATHERVANE_Z_VELOCITY + self.slot].wrapping_sub(1);
        self.ram[WEATHERVANE_Z_VELOCITY + self.slot] = z_velocity;
        z_velocity
    }

    pub(crate) fn load_into_ancilla(&self, ancilla: &mut AncillaSlotViewMut<'_>) {
        let y = packed_position(
            self.ram,
            WEATHERVANE_Y_LO + self.slot,
            WEATHERVANE_Y_HI + self.slot,
        );
        let x = packed_position(
            self.ram,
            WEATHERVANE_X_LO + self.slot,
            WEATHERVANE_X_HI + self.slot,
        );
        ancilla.set_item_to_link(byte(self.ram, WEATHERVANE_DRAW_STATE + self.slot));
        ancilla.set_y(y);
        ancilla.set_x(x);
        ancilla.set_z(byte(self.ram, WEATHERVANE_Z + self.slot));
        ancilla.set_y_velocity(byte(self.ram, WEATHERVANE_Y_VELOCITY + self.slot));
        ancilla.set_x_velocity(byte(self.ram, WEATHERVANE_X_VELOCITY + self.slot));
        ancilla.set_z_velocity(byte(self.ram, WEATHERVANE_Z_VELOCITY + self.slot));
    }

    pub(crate) fn mark_finished_if_landed(&mut self, z: u8) {
        if z >= 0xf0 {
            self.ram[WEATHERVANE_DRAW_STATE + self.slot] = 0xff;
        }
    }

    pub(crate) fn save_position(&mut self, x: u16, y: u16, z: u8) {
        self.ram[WEATHERVANE_Y_LO + self.slot] = y as u8;
        self.ram[WEATHERVANE_Y_HI + self.slot] = (y >> 8) as u8;
        self.ram[WEATHERVANE_X_LO + self.slot] = x as u8;
        self.ram[WEATHERVANE_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[WEATHERVANE_Z + self.slot] = z;
    }
}

pub(crate) struct BirdTravelDestinationView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BirdTravelDestinationView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            BIRD_TRAVEL_X_LO + self.slot,
            BIRD_TRAVEL_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            BIRD_TRAVEL_Y_LO + self.slot,
            BIRD_TRAVEL_Y_HI + self.slot,
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ram[BIRD_TRAVEL_X_LO + self.slot]
            | self.ram[BIRD_TRAVEL_X_HI + self.slot]
            | self.ram[BIRD_TRAVEL_Y_LO + self.slot]
            | self.ram[BIRD_TRAVEL_Y_HI + self.slot]
            == 0
    }
}

pub(crate) struct BirdTravelDestinationViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BirdTravelDestinationViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[BIRD_TRAVEL_X_LO + self.slot] = x as u8;
        self.ram[BIRD_TRAVEL_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[BIRD_TRAVEL_Y_LO + self.slot] = y as u8;
        self.ram[BIRD_TRAVEL_Y_HI + self.slot] = (y >> 8) as u8;
    }

    pub(crate) fn clear(&mut self) {
        self.set_position(0, 0);
    }
}

pub(crate) struct BirdTravelStatusViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> BirdTravelStatusViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear(&mut self, slot: usize) {
        self.ram[BIRD_TRAVEL_STATUS + slot] = 0;
    }

    pub(crate) fn increment(&mut self, slot: usize) {
        self.ram[BIRD_TRAVEL_STATUS + slot] = self.ram[BIRD_TRAVEL_STATUS + slot].wrapping_add(1);
    }
}

pub(crate) struct MoldormHistoryView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> MoldormHistoryView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            MOLDORM_HISTORY_X_LO + self.slot,
            MOLDORM_HISTORY_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            MOLDORM_HISTORY_Y_LO + self.slot,
            MOLDORM_HISTORY_Y_HI + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, MOLDORM_HISTORY_X_LO + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, MOLDORM_HISTORY_Y_LO + self.slot)
    }
}

pub(crate) struct MoldormHistoryViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> MoldormHistoryViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[MOLDORM_HISTORY_X_LO + self.slot] = x as u8;
        self.ram[MOLDORM_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[MOLDORM_HISTORY_Y_LO + self.slot] = y as u8;
        self.ram[MOLDORM_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
    }

    pub(crate) fn set_low_position(&mut self, x_low: u8, y_low: u8) {
        self.ram[MOLDORM_HISTORY_X_LO + self.slot] = x_low;
        self.ram[MOLDORM_HISTORY_Y_LO + self.slot] = y_low;
    }
}

pub(crate) struct SwamolaTargetView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> SwamolaTargetView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            SWAMOLA_TARGET_X_LO + self.slot,
            SWAMOLA_TARGET_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            SWAMOLA_TARGET_Y_LO + self.slot,
            SWAMOLA_TARGET_Y_HI + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, SWAMOLA_TARGET_X_LO + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, SWAMOLA_TARGET_Y_LO + self.slot)
    }
}

pub(crate) struct SwamolaTargetViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> SwamolaTargetViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[SWAMOLA_TARGET_X_LO + self.slot] = x as u8;
        self.ram[SWAMOLA_TARGET_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[SWAMOLA_TARGET_Y_LO + self.slot] = y as u8;
        self.ram[SWAMOLA_TARGET_Y_HI + self.slot] = (y >> 8) as u8;
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[SWAMOLA_TARGET_X_LO + self.slot] = value;
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[SWAMOLA_TARGET_Y_LO + self.slot] = value;
    }
}

pub(crate) struct SwamolaHistoryView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> SwamolaHistoryView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            SWAMOLA_HISTORY_X_LO + self.slot,
            SWAMOLA_HISTORY_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            SWAMOLA_HISTORY_Y_LO + self.slot,
            SWAMOLA_HISTORY_Y_HI + self.slot,
        )
    }
}

pub(crate) struct SwamolaHistoryViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> SwamolaHistoryViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[SWAMOLA_HISTORY_X_LO + self.slot] = x as u8;
        self.ram[SWAMOLA_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[SWAMOLA_HISTORY_Y_LO + self.slot] = y as u8;
        self.ram[SWAMOLA_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
    }
}

pub(crate) struct BeamosLaserHistoryView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> BeamosLaserHistoryView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            BEAMOS_LASER_HISTORY_X_LO + self.slot,
            BEAMOS_LASER_HISTORY_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            BEAMOS_LASER_HISTORY_Y_LO + self.slot,
            BEAMOS_LASER_HISTORY_Y_HI + self.slot,
        )
    }
}

pub(crate) struct BeamosLaserHistoryViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> BeamosLaserHistoryViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.ram[BEAMOS_LASER_HISTORY_X_LO + self.slot] = x as u8;
        self.ram[BEAMOS_LASER_HISTORY_X_HI + self.slot] = (x >> 8) as u8;
        self.ram[BEAMOS_LASER_HISTORY_Y_LO + self.slot] = y as u8;
        self.ram[BEAMOS_LASER_HISTORY_Y_HI + self.slot] = (y >> 8) as u8;
    }
}

pub(crate) struct LanmolaSegmentMotionView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> LanmolaSegmentMotionView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn z_offset(&self) -> u8 {
        byte(self.ram, BEAMOS_LASER_HISTORY_X_HI + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, BEAMOS_LASER_HISTORY_Y_HI + self.slot)
    }
}

pub(crate) struct LanmolaSegmentMotionViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> LanmolaSegmentMotionViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_z_offset(&mut self, value: u8) {
        self.ram[BEAMOS_LASER_HISTORY_X_HI + self.slot] = value;
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.ram[BEAMOS_LASER_HISTORY_Y_HI + self.slot] = value;
    }
}

pub(crate) struct DoorDebrisView<'a> {
    ram: &'a [u8],
}

impl<'a> DoorDebrisView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn x(&self, slot: usize) -> u8 {
        byte(self.ram, DOOR_DEBRIS_X + slot)
    }

    pub(crate) fn y(&self, slot: usize) -> u8 {
        byte(self.ram, DOOR_DEBRIS_Y + slot)
    }

    pub(crate) fn direction(&self, slot: usize) -> u8 {
        byte(self.ram, DOOR_DEBRIS_DIRECTION + slot)
    }

    pub(crate) fn x_word(&self, slot: usize) -> u16 {
        word(self.ram, DOOR_DEBRIS_X + slot * 2)
    }

    pub(crate) fn y_word(&self, slot: usize) -> u16 {
        word(self.ram, DOOR_DEBRIS_Y + slot * 2)
    }
}

pub(crate) struct DoorDebrisViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DoorDebrisViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_direction(&mut self, slot: usize, value: u8) {
        self.ram[DOOR_DEBRIS_DIRECTION + slot] = value;
    }

    pub(crate) fn set_y_low_and_x_low_from_word(&mut self, slot: usize, value: u16) {
        self.ram[DOOR_DEBRIS_Y + slot] = value as u8;
        self.ram[DOOR_DEBRIS_X + slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_x_word(&mut self, slot: usize, value: u16) {
        write_le_u16(self.ram, DOOR_DEBRIS_X + slot * 2, value);
    }
}

pub(crate) struct DiggingGamePrizeView<'a> {
    ram: &'a [u8],
}

impl<'a> DiggingGamePrizeView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn attempts(&self) -> u8 {
        byte(self.ram, DIGGING_GAME_PRIZE_ATTEMPTS)
    }

    pub(crate) fn spawned_marker(&self) -> u8 {
        byte(self.ram, DIGGING_GAME_PRIZE_SPAWNED)
    }
}

pub(crate) struct DiggingGamePrizeViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DiggingGamePrizeViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn increment_attempts(&mut self) {
        self.ram[DIGGING_GAME_PRIZE_ATTEMPTS] =
            self.ram[DIGGING_GAME_PRIZE_ATTEMPTS].wrapping_add(1);
    }

    pub(crate) fn mark_spawned(&mut self) {
        self.ram[DIGGING_GAME_PRIZE_SPAWNED] = 0xeb;
    }

    pub(crate) fn clear_prize_spawned(&mut self) {
        self.ram[DIGGING_GAME_PRIZE_SPAWNED] = 0;
    }
}

pub(crate) struct DrawScratchPositionView<'a> {
    ram: &'a [u8],
}

impl<'a> DrawScratchPositionView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, DRAW_SCRATCH_POSITION_X)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, DRAW_SCRATCH_POSITION_Y)
    }

    pub(crate) fn low_position_word(&self) -> u16 {
        word(self.ram, DRAW_SCRATCH_POSITION_X)
    }
}

pub(crate) struct DrawScratchPositionViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DrawScratchPositionViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_low_position(&mut self, x: u8, y: u8) {
        self.ram[DRAW_SCRATCH_POSITION_X] = x;
        self.ram[DRAW_SCRATCH_POSITION_Y] = y;
    }

    pub(crate) fn set_low_position_word(&mut self, value: u16) {
        write_le_u16(self.ram, DRAW_SCRATCH_POSITION_X, value);
    }

    pub(crate) fn set_word_bytes(&mut self, low: u8, high: u8) {
        self.ram[DRAW_SCRATCH_POSITION_X] = low;
        self.ram[DRAW_SCRATCH_POSITION_Y] = high;
    }

    pub(crate) fn offset_low_position(&mut self, dx: u8, dy: u8) -> (u8, u8) {
        let x = self.ram[DRAW_SCRATCH_POSITION_X].wrapping_add(dx);
        let y = self.ram[DRAW_SCRATCH_POSITION_Y].wrapping_add(dy);
        self.set_low_position(x, y);
        (x, y)
    }

    pub(crate) fn set_flags_high(&mut self, value: u8) {
        self.ram[DRAW_SCRATCH_FLAGS_HI] = value;
    }
}

pub(crate) struct HitboxScratchOffsetView<'a> {
    ram: &'a [u8],
}

impl<'a> HitboxScratchOffsetView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn x_high_offset(&self) -> u8 {
        byte(self.ram, HITBOX_SCRATCH_X_OFFSET)
    }

    pub(crate) fn y_low_offset(&self) -> u8 {
        byte(self.ram, HITBOX_SCRATCH_Y_OFFSET)
    }
}

pub(crate) struct HitboxScratchOffsetViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> HitboxScratchOffsetViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_x_high_offset(&mut self, value: u8) {
        self.ram[HITBOX_SCRATCH_X_OFFSET] = value;
    }

    pub(crate) fn set_y_low_offset(&mut self, value: u8) {
        self.ram[HITBOX_SCRATCH_Y_OFFSET] = value;
    }

    pub(crate) fn set_offsets(&mut self, y_low: u8, x_high: u8) {
        self.ram[HITBOX_SCRATCH_Y_OFFSET] = y_low;
        self.ram[HITBOX_SCRATCH_X_OFFSET] = x_high;
    }
}

pub(crate) struct OverworldScrollDeltaView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldScrollDeltaView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn low(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCROLL_DELTA)
    }

    pub(crate) fn high(&self) -> u8 {
        byte(self.ram, OVERWORLD_SCROLL_DELTA + 1)
    }

    pub(crate) fn word(&self) -> u16 {
        word(self.ram, OVERWORLD_SCROLL_DELTA)
    }
}

pub(crate) struct OverworldScrollDeltaViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldScrollDeltaViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_high(&mut self, value: u8) {
        self.ram[OVERWORLD_SCROLL_DELTA + 1] = value;
    }

    pub(crate) fn set_low(&mut self, value: u8) {
        self.ram[OVERWORLD_SCROLL_DELTA] = value;
    }

    pub(crate) fn set_low_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA, value);
    }

    pub(crate) fn set_high_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA + 1, value);
    }

    pub(crate) fn set_y_delta(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_SCROLL_DELTA, value);
    }

    pub(crate) fn clear_low(&mut self) {
        self.ram[OVERWORLD_SCROLL_DELTA] = 0;
    }
}
