use super::*;
use crate::game_state::FrameState;
use snes::WRAM_SIZE;

#[test]
fn ram_frame_state_view_matches_native_frame_projection() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[MAIN_MODULE] = 7;
    ram[SUBMODULE] = 2;
    ram[SUBSUBMODULE] = 9;
    ram[FRAME_COUNTER] = 0x42;
    ram[SAVED_MODULE_FOR_MENU] = 5;
    ram[MODAL_PAUSE_FLAG] = 1;

    let native = FrameState::load_from_ram(&ram);
    let view = RamFrameStateView::new(&ram);

    assert_eq!(view.main_module(), native.main_module);
    assert_eq!(view.submodule(), native.submodule);
    assert_eq!(view.subsubmodule(), native.subsubmodule);
    assert_eq!(view.frame_counter(), native.frame_counter);
    assert_eq!(view.saved_module_for_menu(), native.saved_module_for_menu);
    assert_eq!(view.modal_pause_flag(), native.modal_pause_flag);
    assert_eq!(view.main_module_word(), native.main_module_word());
}

#[test]
fn ram_frame_state_view_mut_writes_byte_backed_projection() {
    let mut ram = vec![0; WRAM_SIZE];

    {
        let mut view = RamFrameStateViewMut::new(&mut ram);
        view.set_main_module(14);
        view.set_submodule(3);
        view.set_subsubmodule(1);
        view.set_frame_counter(0x80);
        view.set_saved_module_for_menu(7);
        view.set_modal_pause_flag(2);
    }

    assert_eq!(
        FrameState::load_from_ram(&ram),
        FrameState {
            main_module: 14,
            submodule: 3,
            subsubmodule: 1,
            frame_counter: 0x80,
            saved_module_for_menu: 7,
            modal_pause_flag: 2,
        }
    );
}
