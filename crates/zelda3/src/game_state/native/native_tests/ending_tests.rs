use super::*;

#[test]
fn native_attract_scene_bridge_updates_only_targeted_ram_fields() {
    let mut ram = vec![0xff; WRAM_SIZE];
    let mut native_ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut native_ram, ATTRACT_STATE, 0x1203);
    write_le_u16(&mut native_ram, ATTRACT_X_BASE, 0x5678);
    native_ram[ATTRACT_SCENE_TIMER] = 9;
    native_ram[INTRO_STEP_INDEX] = 4;
    native_ram[INTRO_STEP_TIMER] = 5;
    native_ram[INTRO_FRAME_CTR] = 6;
    let mut attract = AttractSceneState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeAttractSceneBridgeMut::new(&mut attract, &mut ram);
        assert_eq!(bridge.increment_state(), 4);
        bridge.set_sequence(0x34);
        bridge.set_y_base(0x9a);
        assert_eq!(bridge.decrement_scene_timer(), 8);
        assert_eq!(bridge.increment_intro_frame_counter(), 7);
        bridge.clear_intro_step_state_block();
    }

    assert_eq!(attract.state_word(), 0x3404);
    assert_eq!(attract.sequence(), 0x34);
    assert_eq!(attract.x_base_word(), 0x9a78);
    assert_eq!(attract.y_base(), 0x9a);
    assert_eq!(attract.scene_timer(), 8);
    assert_eq!(attract.intro_step_index(), 0);
    assert_eq!(attract.intro_step_timer(), 0);
    assert_eq!(attract.intro_frame_counter(), 0);
    assert_eq!(read_le_u16(&ram, ATTRACT_STATE), 0x3404);
    assert_eq!(ram[ATTRACT_X_BASE], 0xff);
    assert_eq!(ram[ATTRACT_Y_BASE], 0x9a);
    assert_eq!(ram[ATTRACT_SCENE_TIMER], 8);
    assert_eq!(ram[INTRO_STEP_INDEX], 0);
    assert_eq!(ram[INTRO_STEP_TIMER], 0);
    assert_eq!(ram[INTRO_FRAME_CTR], 0);
    assert_eq!(ram[INTRO_TIMES_PAL_FLASH], 0xff);
}

#[test]
fn ending_credit_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ENDING_WHICH_DUNG, 5);
    write_le_u16(&mut ram, ENDING_CREDIT_DIGIT_CHAR, 0x3cf6);

    let mut credits = EndingCreditState::load_from_ram(&ram);
    assert_eq!(credits.palace_death_count_digit_step, 5);
    assert_eq!(credits.palace_death_count_index(), 2);
    assert_eq!(credits.digit_tile_base_index(), 1);
    assert!(credits.should_write_digit_for_scroll_y(0x200, 0x290));
    assert_eq!(credits.death_count_digit_tile_base, 0x3cf6);

    credits.clear_palace_death_count_digit_step();
    credits.death_count_digit_tile_base = 0x3ce6;
    credits.advance_palace_death_count_digit_step();
    credits.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, ENDING_WHICH_DUNG), 1);
    assert_eq!(read_le_u16(&ram, ENDING_CREDIT_DIGIT_CHAR), 0x3ce6);
}

#[test]
fn intro_scene_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[INTRO_WANT_DOUBLE_RET] = 1;
    write_le_u16(&mut ram, INTRO_SPRITE_ALLOC, 0x0800);
    write_le_u16(&mut ram, TRIFORCE_CTR, 0x01c0);

    let mut intro = IntroSceneState::load_from_ram(&ram);
    assert!(intro.triangle_motion_is_paused());
    assert_eq!(intro.sprite_oam_cursor, 0x0800);
    assert_eq!(intro.triforce_countdown, 0x01c0);
    assert_eq!(intro.allocate_oam_entries(3), 0x0800);
    assert_eq!(intro.sprite_oam_cursor, 0x080c);
    intro.resume_triangle_motion();
    intro.decrement_triforce_countdown();
    intro.write_to_ram(&mut ram);

    assert_eq!(ram[INTRO_WANT_DOUBLE_RET], 0);
    assert_eq!(read_le_u16(&ram, INTRO_SPRITE_ALLOC), 0x080c);
    assert_eq!(read_le_u16(&ram, TRIFORCE_CTR), 0x01bf);
}

#[test]
fn native_intro_scene_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[INTRO_WANT_DOUBLE_RET] = 0;
    write_le_u16(&mut ram, INTRO_SPRITE_ALLOC, 0x0800);
    write_le_u16(&mut ram, TRIFORCE_CTR, 0);

    let mut intro = IntroSceneState::load_from_ram(&ram);
    {
        let mut bridge = NativeIntroSceneBridgeMut::new(&mut intro, &mut ram);
        bridge.pause_triangle_motion();
        assert_eq!(bridge.allocate_oam_entries(2), 0x0800);
        bridge.set_triforce_countdown(0x0001);
        bridge.decrement_triforce_countdown();
        bridge.resume_triangle_motion();
        bridge.set_sprite_oam_cursor(0x0900);
    }

    assert_eq!(
        intro,
        IntroSceneState {
            triangle_motion_pause: 0,
            sprite_oam_cursor: 0x0900,
            triforce_countdown: 0,
        }
    );
    assert_eq!(ram[INTRO_WANT_DOUBLE_RET], 0);
    assert_eq!(read_le_u16(&ram, INTRO_SPRITE_ALLOC), 0x0900);
    assert_eq!(read_le_u16(&ram, TRIFORCE_CTR), 0);
}

#[test]
fn native_intro_scene_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[INTRO_WANT_DOUBLE_RET] = 1;
    write_le_u16(&mut ram, INTRO_SPRITE_ALLOC, 0x0800);
    write_le_u16(&mut ram, TRIFORCE_CTR, 0x0001);
    let mut intro = IntroSceneState {
        triangle_motion_pause: 0,
        sprite_oam_cursor: 0x0900,
        triforce_countdown: 0x0020,
    };

    {
        let mut bridge = NativeIntroSceneBridgeMut::new(&mut intro, &mut ram);
        assert_eq!(bridge.allocate_oam_entries(1), 0x0900);
    }

    assert_eq!(intro.sprite_oam_cursor, 0x0904);
    assert_eq!(intro.triforce_countdown, 0x0020);
    assert_eq!(ram[INTRO_WANT_DOUBLE_RET], 0);
    assert_eq!(read_le_u16(&ram, INTRO_SPRITE_ALLOC), 0x0904);
    assert_eq!(read_le_u16(&ram, TRIFORCE_CTR), 0x0020);
}

#[test]
fn native_intro_actor_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[INTRO_SPRITE_STATE + 1] = 0xaa;
    ram[INTRO_X_LO + 1] = 0xbb;
    ram[INTRO_X_HI + 1] = 0xcc;
    let mut native_ram = vec![0; WRAM_SIZE];
    native_ram[INTRO_SPRITE_STATE + 1] = 0x22;
    native_ram[INTRO_X_LO + 1] = 0x34;
    native_ram[INTRO_X_HI + 1] = 0x12;
    let mut actors = ending::IntroActorState::load_from_ram(&native_ram);

    {
        let mut bridge = NativeIntroActorBridgeMut::new(&mut actors, &mut ram, 1);
        bridge.increment_state();
        bridge.set_x_low(0x78);
    }

    let actor = IntroActorRead::new(&actors, 1);
    assert_eq!(actor.state(), 0x23);
    assert_eq!(actor.x(), 0x1278);
    assert_eq!(ram[INTRO_SPRITE_STATE + 1], 0x23);
    assert_eq!(ram[INTRO_X_LO + 1], 0x78);
    assert_eq!(ram[INTRO_X_HI + 1], 0x12);
}

#[test]
fn native_ending_credit_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ENDING_WHICH_DUNG, 0xffff);
    write_le_u16(&mut ram, ENDING_CREDIT_DIGIT_CHAR, 0x1111);

    let mut credits = EndingCreditState::load_from_ram(&ram);
    {
        let mut bridge = NativeEndingCreditBridgeMut::new(&mut credits, &mut ram);
        bridge.clear_palace_death_count_digit_step();
        bridge.advance_palace_death_count_digit_step();
        bridge.set_death_count_digit_tile_base(0x3cf6);
        bridge.set_palace_death_count_digit_step(4);
    }

    assert_eq!(credits.palace_death_count_digit_step, 4);
    assert_eq!(credits.death_count_digit_tile_base, 0x3cf6);
    assert_eq!(read_le_u16(&ram, ENDING_WHICH_DUNG), 4);
    assert_eq!(read_le_u16(&ram, ENDING_CREDIT_DIGIT_CHAR), 0x3cf6);
}

#[test]
fn native_ending_credit_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ENDING_WHICH_DUNG, 0xffff);
    write_le_u16(&mut ram, ENDING_CREDIT_DIGIT_CHAR, 0x1111);
    let mut credits = EndingCreditState {
        palace_death_count_digit_step: 2,
        death_count_digit_tile_base: 0x2222,
    };

    {
        let mut bridge = NativeEndingCreditBridgeMut::new(&mut credits, &mut ram);
        bridge.advance_palace_death_count_digit_step();
    }

    assert_eq!(credits.palace_death_count_digit_step, 3);
    assert_eq!(credits.death_count_digit_tile_base, 0x2222);
    assert_eq!(read_le_u16(&ram, ENDING_WHICH_DUNG), 3);
    assert_eq!(read_le_u16(&ram, ENDING_CREDIT_DIGIT_CHAR), 0x2222);
}
