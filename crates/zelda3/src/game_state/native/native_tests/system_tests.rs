use super::*;

#[test]
fn system_signals_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MUSIC_CONTROL] = 0xf2;
    ram[CURRENT_MUSIC_CONTROL] = 0x13;
    ram[LAST_MUSIC_CONTROL] = 0x12;
    ram[QUEUED_MUSIC_CONTROL] = 0x09;
    ram[SOUND_EFFECT_1] = 0x2d;
    ram[SOUND_EFFECT_2] = 0x1b;
    ram[SOUND_EFFECT_AMBIENT] = 0x05;
    ram[SOUND_EFFECT_AMBIENT_LAST] = 0x07;
    ram[MSU_VOLUME] = 0x80;
    ram[RAM_APUI00] = 1;
    ram[RAW_SFX_PAN_VALUE] = 0xc0;
    ram[FLAG_UPDATE_CGRAM_IN_NMI] = 2;
    ram[FLAG_UPDATE_HUD_IN_NMI] = 3;
    ram[GAME_OVER_CHECK_FLAG] = 4;
    ram[RESTART_CHECK_FLAG] = 5;
    ram[RAM_BUGS_FIXED] = 0x42;
    ram[DEATH_BACKUP_CURRENT_MUSIC] = 0x22;
    ram[DEATH_BACKUP_AMBIENT_SOUND] = 0x33;

    let system_signals = SystemSignalsState::load_from_ram(&ram);
    assert_eq!(system_signals.music_control(), 0xf2);
    assert_eq!(system_signals.current_music_control(), 0x13);
    assert_eq!(system_signals.last_music_control(), 0x12);
    assert_eq!(system_signals.queued_music_control(), 0x09);
    assert_eq!(system_signals.sound_effect_1(), 0x2d);
    assert_eq!(system_signals.sound_effect_2(), 0x1b);
    assert_eq!(system_signals.ambient_sound_effect(), 0x05);
    assert_eq!(system_signals.last_ambient_sound_effect(), 0x07);
    assert_eq!(system_signals.msu_volume(), 0x80);
    assert_eq!(system_signals.apui00(), 1);
    assert_eq!(system_signals.raw_sfx_pan_value(), 0xc0);
    assert!(system_signals.should_update_cgram());
    assert!(system_signals.should_update_hud());
    assert_eq!(system_signals.game_over_check_flag(), 4);
    assert_eq!(system_signals.restart_check_flag(), 5);
    assert_eq!(system_signals.bugs_fixed(), 0x42);
    assert_eq!(system_signals.death_backup_current_music(), 0x22);
    assert_eq!(system_signals.death_backup_ambient_sound(), 0x33);

    let primary_resume = MsuResumeInfoState {
        tag: 0x1122_3344,
        offset: 0x5566_7788,
        samples_until_repeat: 0x99aa_bbcc,
        range_cur: 0x1234,
        range_repeat: 0x5678,
        initial_packet_bytes: 0x1122_3344_5566_7788,
        orig_track: 0x12,
        actual_track: 0x34,
    };
    let alternate_resume = MsuResumeInfoState {
        tag: 0xaabb_ccdd,
        offset: 0xeeff_0011,
        samples_until_repeat: 0x2233_4455,
        range_cur: 0x6677,
        range_repeat: 0x8899,
        initial_packet_bytes: 0x8877_6655_4433_2211,
        orig_track: 0x56,
        actual_track: 0x78,
    };
    let mut system_signals = system_signals;
    system_signals.set_msu_resume_info(MsuResumeSlot::Primary, primary_resume);
    system_signals.set_msu_resume_info(MsuResumeSlot::Alternate, alternate_resume);

    let mut projected = vec![0; WRAM_SIZE];
    system_signals.write_to_ram(&mut projected);
    let reloaded = SystemSignalsState::load_from_ram(&projected);
    assert_eq!(
        reloaded.msu_resume_info(MsuResumeSlot::Primary),
        primary_resume
    );
    assert_eq!(
        reloaded.msu_resume_info(MsuResumeSlot::Alternate),
        alternate_resume
    );
    for offset in [
        MUSIC_CONTROL,
        CURRENT_MUSIC_CONTROL,
        LAST_MUSIC_CONTROL,
        QUEUED_MUSIC_CONTROL,
        SOUND_EFFECT_1,
        SOUND_EFFECT_2,
        SOUND_EFFECT_AMBIENT,
        SOUND_EFFECT_AMBIENT_LAST,
        MSU_VOLUME,
        RAM_APUI00,
        RAW_SFX_PAN_VALUE,
        FLAG_UPDATE_CGRAM_IN_NMI,
        FLAG_UPDATE_HUD_IN_NMI,
        GAME_OVER_CHECK_FLAG,
        RESTART_CHECK_FLAG,
        RAM_BUGS_FIXED,
        DEATH_BACKUP_CURRENT_MUSIC,
        DEATH_BACKUP_AMBIENT_SOUND,
    ] {
        assert_eq!(projected[offset], ram[offset]);
    }
}

#[test]
fn system_signals_state_owns_sound_and_update_behavior() {
    let mut system_signals = SystemSignalsState::default();
    system_signals.set_current_music_control(0x12);
    system_signals.set_ambient_sound_effect(0x05);

    assert!(system_signals.queue_sound_effect_1_if_empty(0x2d));
    assert!(!system_signals.queue_sound_effect_1_if_empty(0x33));
    assert!(system_signals.queue_sound_effect_2_if_empty(0x1b));

    system_signals.set_sound_effect_1_word(0x3412);
    system_signals.set_ambient_sound_effect_word(0x5607);
    system_signals.save_current_music_as_last();
    system_signals.save_ambient_sound_effect_as_last();
    system_signals.set_game_over_check_flag(0xff);
    system_signals.increment_game_over_check_flag();
    system_signals.set_restart_check_flag(0x44);
    system_signals.clear_restart_check_flag();
    system_signals.clear_sound_effect_2();
    system_signals.clear_ambient_sound_effect();
    system_signals.set_raw_sfx_pan_value(0x80);

    assert_eq!(system_signals.sound_effect_1(), 0x56);
    assert_eq!(system_signals.sound_effect_2(), 0);
    assert_eq!(system_signals.ambient_sound_effect(), 0);
    assert_eq!(system_signals.last_music_control(), 0x12);
    assert_eq!(system_signals.last_ambient_sound_effect(), 0x07);
    assert_eq!(system_signals.game_over_check_flag(), 0);
    assert_eq!(system_signals.restart_check_flag(), 0);
    assert_eq!(system_signals.raw_sfx_pan_value(), 0x80);
    assert_eq!(system_signals.increment_cgram_update_flag(), 1);
    assert_eq!(system_signals.increment_hud_update_flag(), 1);
    system_signals.clear_cgram_update_flag();
    system_signals.clear_hud_update_flag();
    assert!(!system_signals.should_update_cgram());
    assert!(!system_signals.should_update_hud());
}

#[test]
fn native_system_signals_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[CURRENT_MUSIC_CONTROL] = 0x12;
    ram[SOUND_EFFECT_AMBIENT] = 0x05;
    ram[SOUND_EFFECT_1] = 0;
    ram[SOUND_EFFECT_2] = 0;
    ram[FLAG_UPDATE_CGRAM_IN_NMI] = 0xff;
    ram[FLAG_UPDATE_HUD_IN_NMI] = 1;
    ram[GAME_OVER_CHECK_FLAG] = 7;

    let mut system_signals = SystemSignalsState::load_from_ram(&ram);
    {
        let mut bridge = NativeSystemSignalsBridgeMut::new(&mut system_signals, &mut ram);
        assert!(bridge.queue_sound_effect_1_if_empty(0x2d));
        assert!(bridge.queue_sound_effect_2_if_empty(0x1b));
        assert!(!bridge.queue_sound_effect_1_if_empty(0x33));
        bridge.set_sound_effect_1_word(0x3412);
        bridge.set_ambient_sound_effect_word(0x5607);
        bridge.save_current_music_as_last();
        bridge.save_ambient_sound_effect_as_last();
        bridge.increment_cgram_update_flag();
        bridge.increment_hud_update_flag();
        bridge.clear_game_over_check_flag();
        bridge.set_restart_check_flag(9);
        bridge.set_raw_sfx_pan_value(0x80);
        bridge.set_death_backup_current_music(0x21);
        bridge.set_death_backup_ambient_sound(0x22);
    }

    assert_eq!(system_signals.sound_effect_1(), 0x56);
    assert_eq!(system_signals.sound_effect_2(), 0x34);
    assert_eq!(system_signals.ambient_sound_effect(), 0x07);
    assert_eq!(system_signals.last_music_control(), 0x12);
    assert_eq!(system_signals.last_ambient_sound_effect(), 0x07);
    assert_eq!(system_signals.raw_sfx_pan_value(), 0x80);
    assert_eq!(system_signals.restart_check_flag(), 9);
    assert_eq!(system_signals.death_backup_current_music(), 0x21);
    assert_eq!(system_signals.death_backup_ambient_sound(), 0x22);
    assert_eq!(ram[SOUND_EFFECT_1], 0x56);
    assert_eq!(ram[SOUND_EFFECT_2], 0x34);
    assert_eq!(ram[SOUND_EFFECT_AMBIENT], 0x07);
    assert_eq!(ram[LAST_MUSIC_CONTROL], 0x12);
    assert_eq!(ram[SOUND_EFFECT_AMBIENT_LAST], 0x07);
    assert_eq!(ram[FLAG_UPDATE_CGRAM_IN_NMI], 0);
    assert_eq!(ram[FLAG_UPDATE_HUD_IN_NMI], 2);
    assert_eq!(ram[GAME_OVER_CHECK_FLAG], 0);
    assert_eq!(ram[RESTART_CHECK_FLAG], 9);
    assert_eq!(ram[RAW_SFX_PAN_VALUE], 0x80);
    assert_eq!(ram[DEATH_BACKUP_CURRENT_MUSIC], 0x21);
    assert_eq!(ram[DEATH_BACKUP_AMBIENT_SOUND], 0x22);
}

#[test]
fn native_system_work_area_bridge_clears_startup_work_ranges() {
    const STARTUP_LOW_MEMORY_START: usize = 0;
    const STARTUP_LOW_MEMORY_LEN: usize = 0x2000;
    const ATTRACT_LOW_WORK_AREA_START: usize = 0x20;
    const ATTRACT_LOW_WORK_AREA_LEN: usize = 0x51;
    const POLY_THREAD_WORK_AREA_START: usize = 0x1f00;
    const POLY_THREAD_WORK_AREA_LEN: usize = 0x100;

    let mut ram = vec![0xff; WRAM_SIZE];
    {
        let mut bridge = NativeSystemWorkAreaBridgeMut::new(&mut ram);
        bridge.clear_startup_low_memory();
    }

    assert!(
        ram[STARTUP_LOW_MEMORY_START..STARTUP_LOW_MEMORY_START + STARTUP_LOW_MEMORY_LEN]
            .iter()
            .all(|&value| value == 0)
    );
    assert_eq!(ram[STARTUP_LOW_MEMORY_START + STARTUP_LOW_MEMORY_LEN], 0xff);

    ram.fill(0xff);
    {
        let mut bridge = NativeSystemWorkAreaBridgeMut::new(&mut ram);
        bridge.clear_attract_low_work_area();
        bridge.clear_poly_thread_work_area();
    }

    assert!(ram
        [ATTRACT_LOW_WORK_AREA_START..ATTRACT_LOW_WORK_AREA_START + ATTRACT_LOW_WORK_AREA_LEN]
        .iter()
        .all(|&value| value == 0));
    assert_eq!(ram[ATTRACT_LOW_WORK_AREA_START - 1], 0xff);
    assert_eq!(
        ram[ATTRACT_LOW_WORK_AREA_START + ATTRACT_LOW_WORK_AREA_LEN],
        0xff
    );
    assert!(ram
        [POLY_THREAD_WORK_AREA_START..POLY_THREAD_WORK_AREA_START + POLY_THREAD_WORK_AREA_LEN]
        .iter()
        .all(|&value| value == 0));
    assert_eq!(ram[POLY_THREAD_WORK_AREA_START - 1], 0xff);
}

#[test]
fn native_system_work_area_bridge_writes_poly_thread_bootstrap() {
    const POLY_THREAD_BOOTSTRAP_BYTES_OFFSET: usize = 0x1f32;
    const POLY_THREAD_BOOTSTRAP_BYTES: [u8; 13] =
        [9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9];

    let mut ram = vec![0; WRAM_SIZE];
    NativeSystemWorkAreaBridgeMut::new(&mut ram).write_poly_thread_bootstrap_bytes();

    assert_eq!(
        &ram[POLY_THREAD_BOOTSTRAP_BYTES_OFFSET
            ..POLY_THREAD_BOOTSTRAP_BYTES_OFFSET + POLY_THREAD_BOOTSTRAP_BYTES.len()],
        &POLY_THREAD_BOOTSTRAP_BYTES
    );
}

#[test]
fn native_system_work_area_bridge_clears_intro_wram_block_columns() {
    const INTRO_CLEAR_BLOCK_BASE: usize = 0x2000;
    const INTRO_CLEAR_BLOCK_STRIDE: usize = 0x2000;

    let mut ram = vec![0xff; WRAM_SIZE];
    let result = NativeSystemWorkAreaBridgeMut::new(&mut ram).clear_intro_wram_block_columns(4, 0);

    assert_eq!(result, 0);
    for block in 0..15 {
        let base = INTRO_CLEAR_BLOCK_BASE + block * INTRO_CLEAR_BLOCK_STRIDE;
        assert_eq!(&ram[base + 2..base + 6], &[0, 0, 0, 0]);
        assert_eq!(ram[base], 0xff);
        assert_eq!(ram[base + 1], 0xff);
        assert_eq!(ram[base + 6], 0xff);
    }
}
