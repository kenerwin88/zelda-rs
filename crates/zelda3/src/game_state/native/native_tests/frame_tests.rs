use super::*;

#[test]
fn frame_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MAIN_MODULE] = 7;
    ram[SUBMODULE] = 2;
    ram[SUBSUBMODULE] = 9;
    ram[FRAME_COUNTER] = 0x42;
    ram[SAVED_MODULE_FOR_MENU] = 5;
    ram[MODAL_PAUSE_FLAG] = 1;

    let mut frame = FrameState::load_from_ram(&ram);
    assert_eq!(frame.main_module, 7);
    assert_eq!(frame.main_module_word(), 0x0207);
    assert_eq!(frame.submodule, 2);
    assert_eq!(frame.subsubmodule, 9);
    assert_eq!(frame.frame_counter, 0x42);
    assert_eq!(frame.saved_module_for_menu, 5);
    assert_eq!(frame.modal_pause_flag, 1);

    frame.set_main_module_word(0x030e);
    frame.set_subsubmodule(1);
    frame.set_frame_counter(0x80);
    frame.set_saved_module_for_menu(7);
    frame.set_modal_pause_flag(2);
    frame.write_to_ram(&mut ram);

    assert_eq!(ram[MAIN_MODULE], 14);
    assert_eq!(ram[SUBMODULE], 3);
    assert_eq!(ram[SUBSUBMODULE], 1);
    assert_eq!(ram[FRAME_COUNTER], 0x80);
    assert_eq!(ram[SAVED_MODULE_FOR_MENU], 7);
    assert_eq!(ram[MODAL_PAUSE_FLAG], 2);
}

#[test]
fn frame_state_owns_module_and_pause_behavior() {
    let mut frame = FrameState {
        main_module: 0xfe,
        submodule: 0xff,
        subsubmodule: 0,
        frame_counter: 0xff,
        saved_module_for_menu: 0x44,
        modal_pause_flag: 0xff,
    };

    frame.increment_submodule();
    frame.decrement_submodule();
    frame.increment_subsubmodule();
    frame.decrement_subsubmodule();
    frame.increment_frame_counter();
    frame.clear_saved_module_for_menu();
    frame.save_main_module_for_menu();
    frame.save_submodule_for_menu();
    frame.clear_modal_pause_flag();
    let modal_pause_flag = frame.increment_modal_pause_flag();

    assert_eq!(frame.submodule, 0xff);
    assert_eq!(frame.subsubmodule, 0);
    assert_eq!(frame.frame_counter, 0);
    assert_eq!(frame.saved_module_for_menu, 0xff);
    assert_eq!(frame.modal_pause_flag, 1);
    assert_eq!(modal_pause_flag, 1);
}

#[test]
fn native_frame_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MAIN_MODULE] = 1;
    ram[SUBMODULE] = 2;
    ram[SUBSUBMODULE] = 3;
    ram[FRAME_COUNTER] = 4;
    ram[SAVED_MODULE_FOR_MENU] = 8;
    ram[MODAL_PAUSE_FLAG] = 1;

    let mut frame = FrameState::load_from_ram(&ram);
    {
        let mut bridge = NativeFrameStateBridgeMut::new(&mut frame, &mut ram);
        bridge.increment_submodule();
        bridge.set_subsubmodule(9);
        bridge.increment_frame_counter();
        bridge.save_main_module_for_menu();
        bridge.clear_saved_module_for_menu();
        bridge.save_submodule_for_menu();
        bridge.clear_modal_pause_flag();
        bridge.increment_modal_pause_flag();
        bridge.set_modal_pause_flag(6);
    }

    assert_eq!(frame.main_module, 1);
    assert_eq!(frame.submodule, 3);
    assert_eq!(frame.subsubmodule, 9);
    assert_eq!(frame.frame_counter, 5);
    assert_eq!(frame.saved_module_for_menu, 3);
    assert_eq!(frame.modal_pause_flag, 6);
    assert_eq!(ram[SUBMODULE], 3);
    assert_eq!(ram[SUBSUBMODULE], 9);
    assert_eq!(ram[FRAME_COUNTER], 5);
    assert_eq!(ram[SAVED_MODULE_FOR_MENU], 3);
    assert_eq!(ram[MODAL_PAUSE_FLAG], 6);
}

#[test]
fn native_frame_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut frame = FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 9,
        frame_counter: 0x42,
        saved_module_for_menu: 5,
        modal_pause_flag: 1,
    };
    frame.write_to_ram(&mut ram);

    ram[MAIN_MODULE] = 0xaa;
    ram[FRAME_COUNTER] = 0xbb;

    {
        let mut bridge = NativeFrameStateBridgeMut::new(&mut frame, &mut ram);
        bridge.set_submodule(3);
    }

    assert_eq!(frame.main_module, 7);
    assert_eq!(frame.submodule, 3);
    assert_eq!(frame.frame_counter, 0x42);
    assert_eq!(FrameState::load_from_ram(&ram), frame);
    assert_eq!(ram[MAIN_MODULE], 7);
    assert_eq!(ram[SUBMODULE], 3);
    assert_eq!(ram[FRAME_COUNTER], 0x42);
}
