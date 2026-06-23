use super::*;

#[test]
fn guard_body_packs_skipped_oam_and_hides_type_46_blank_tiles() {
    let graphics = (0..SOLDIER_DRAW2_CHAR.len() / 4)
        .find(|&graphics| {
            let emitted: Vec<_> = (0..=3)
                .rev()
                .filter_map(|i| {
                    let j = graphics * 4 + i;
                    if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                        None
                    } else {
                        Some(j)
                    }
                })
                .collect();
            emitted.len() < 4 && emitted.iter().any(|&j| SOLDIER_DRAW2_CHAR[j] == 0x20)
        })
        .expect("soldier draw table should contain type-46 skipped blank body entries");
    let expected: Vec<_> = (0..=3)
        .rev()
        .filter_map(|i| {
            let j = graphics * 4 + i;
            if SOLDIER_DRAW2_BIG[j] == 0 || i == 3 && SOLDIER_DRAW2_CHAR[j] == 0x20 {
                None
            } else {
                Some(SOLDIER_DRAW2_CHAR[j])
            }
        })
        .collect();

    let mut s = ZeldaState::new();
    s.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    for i in 0..4 {
        let base = OAM_BUF + i * 4;
        s.oam_state_mut().set_entry_y(base, 0xee);
        s.oam_state_mut().set_entry_char(base, 0xee);
    }
    let k = 0;
    s.sprite_slot_view_mut(k).set_sprite_type(0x46);
    s.sprite_slot_view_mut(k).set_graphics(graphics as u8);

    s.guard_animate_body(
        k,
        0,
        &PrepOamCoordsRet {
            x: 0x40,
            y: 0x50,
            r4: 0,
            flags: 0,
        },
    );

    for (slot, &charnum) in expected.iter().enumerate() {
        assert_eq!(s.ram[OAM_BUF + slot * 4 + 2], charnum);
        if charnum == 0x20 {
            assert_eq!(s.ram[OAM_BUF + slot * 4 + 1], 0xf0);
        }
    }
    assert_eq!(s.ram[OAM_BUF + expected.len() * 4 + 2], 0xee);
}
