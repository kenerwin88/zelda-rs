use super::tests::{marker29_machine, marker29_state};

#[test]
fn prefix27_radial_projection_all_angles_radii_and_far_entry_match_rom() {
    for angle in 0..64 { for radius in [0, 1, 14, 127, 128, 255] { for far in [false, true] {
        let n = marker29_state();
        let mut s = marker29_machine(&n, if far {0x08fb23} else {0x08fadd}, 5).unwrap();
        s.cpu.a = u16::from(angle); s.ram[8] = radius;
        let value = std::cell::Cell::new(None);
        let source = crate::cycle_ledger::assert_rom_marker_work(0x08fadd, s,
            |s| s.cpu.k == 8 && (0xfadd..0xfb23).contains(&s.cpu.pc),
            || value.set(Some(if far { n.ancilla_get_radial_projection_long(angle, radius) }
                else { n.ancilla_get_radial_projection(angle, radius) })));
        let p = value.get().unwrap();
        for (address, actual) in [(0,p.r0),(2,p.r2),(4,p.r4),(6,p.r6)] {
            assert_eq!(actual, source.snes().ram[address]);
        }
    } } }
}

#[test]
fn prefix27_radial_oam_all_sign_and_coordinate_paths_match_rom() {
    for sign_y in [0, 1] { for sign_x in [0, 1] { for base in [0u16, 4, 0xff, 0x100, 0xffff] {
        let mut n = marker29_state();
        n.write_wram_word(0x1f810, base); n.write_wram_word(0x1f80e, base);
        n.sync_native_game_state_from_ram();
        let mut s = marker29_machine(&n, 0x08da17, 0).unwrap();
        s.ram[0..8].fill(0);
        let p = crate::types::AncillaRadialProjection { r0: 14, r2: sign_y, r4: 17, r6: sign_x };
        for (a,v) in [(0,p.r0),(2,p.r2),(4,p.r4),(6,p.r6)] { s.ram[a] = v; }
        let value = std::cell::Cell::new(None);
        let source = crate::cycle_ledger::assert_rom_marker_work(0x08da17, s,
            |s| s.cpu.k == 8 && (0xda17..0xda4c).contains(&s.cpu.pc),
            || value.set(Some(n.sparkle_prep_oam_from_radial(p))));
        let p = value.get().unwrap();
        assert_eq!(p.y, u16::from_le_bytes(source.snes().ram[0..2].try_into().unwrap()));
        assert_eq!(p.x, u16::from_le_bytes(source.snes().ram[2..4].try_into().unwrap()));
    } } }
}

#[test]
fn prefix27_sword_beam_axis_wrappers_all_slots_and_velocity_signs_match_rom() {
    for slot in 0..10 { for velocity in [0, 1, 127, 128, 255] { for horizontal in [false, true] {
        let mut n = marker29_state();
        n.write_wram_byte(0xc22 + slot, velocity); n.write_wram_byte(0xc2c + slot, velocity);
        n.write_wram_byte(0xc36 + slot, 0xff); n.write_wram_byte(0xc40 + slot, 0xff);
        n.write_wram_byte(0xfa0, slot as u8); n.sync_native_game_state_from_ram();
        let s = marker29_machine(&n, if horizontal {0x0ffa37} else {0x0ffa43}, slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x0ffa43, s,
            |s| s.cpu.k == 15 && (0xfa43..0xfa6f).contains(&s.cpu.pc),
            || n.ancilla_move_sword_beam_axis(slot, horizontal));
    } } }
}

#[test]
fn prefix27_sword_beam_all_caller_branches_exclude_children() {
    let edges = std::cell::RefCell::new(std::collections::BTreeSet::new());
    for slot in [0usize, 9] { for paused in [0, 1] { for g in [0, 1] {
     for (aux, work3, work1) in [(0,0,0),(1,1,1),(1,0,2),(1,0,3)] {
      for (x,y) in [(0x40u16,0x40u16),(0x1000,0x1000)] { for tile in [0, 1] {
        let mut n = marker29_state();
        for (a,v) in [(0xc4a,12),(0x394,g),(0x3b1,aux),(0x39f,work3),(0x3a4,work1),(0xc72,0)] { n.write_wram_byte(a+slot,v); }
        for a in 0x12000..0x14000 { n.write_wram_byte(a,tile); }
        n.write_wram_word(0x1f80e,x); n.write_wram_word(0x1f810,y);
        n.write_wram_byte(0x11,paused); n.write_wram_byte(0xfa0,slot as u8);
        n.sync_native_game_state_from_ram();
        n.ancilla_slot_view_mut(slot).set_g(g);
        n.ether_orbit_mut().set_swordbeam_temp(x,y);
        n.effect_angle_scratch_mut().set_radial_radius(14);
        n.effect_angle_scratch_mut().set_angles4(&[0,16,32,48],0);
        let mut s = marker29_machine(&n, 0x0ff764, slot).unwrap();
        s.cpu.sp = 0x1fc; s.ram[0x1fd..0x200].copy_from_slice(&[0xff,0x7f,0x0f]);
        crate::cycle_ledger::assert_rom_marker_work(0x0ff764, s, |s| {
            let own = s.cpu.k == 15 && (0xf764..0xf8eb).contains(&s.cpu.pc);
            if own { let taken = match s.cart.read(s.cpu.k,s.cpu.pc,0) {
                0xf0=>Some(s.cpu.z), 0xd0=>Some(!s.cpu.z), 0xb0=>Some(s.cpu.c),
                0x90=>Some(!s.cpu.c), 0x10=>Some(!s.cpu.n), _=>None };
                if let Some(t) = taken { edges.borrow_mut().insert((s.cpu.pc,t)); }
            }
            own
        }, || n.ancilla_sword_beam(slot));
      } }
     }
    } } }
    for pc in [0xf76e,0xf7b6,0xf7ce,0xf80e,0xf826,0xf86c,0xf872,0xf877,0xf889,0xf89f,0xf8db,0xf8e3] {
        for t in [false,true] { assert!(edges.borrow().contains(&(pc,t)),"uncovered {pc:x}/{t}"); }
    }
}

#[test]
fn prefix27_beam_hit_all_timer_clipping_sign_and_oam_crossing_paths_match_rom() {
    for slot in [0usize,9] { for timer in 0..8 { for floor in [0,1] {
        for (x,y) in [(0u16,0u16),(4,4),(0x40,0x40),(0xf3,0xef),(0xf4,0xf0),(0x100,0x100)] {
            for oam in [0x800u16,0x8fc] {
                let mut n=marker29_state();
                for(a,v)in[(0xc4a,4),(0xc68,timer),(0xc7c,floor)] {n.write_wram_byte(a+slot,v);}
                n.write_wram_byte(0xfa0,slot as u8);n.write_wram_word(0x90,oam);
                n.sync_native_game_state_from_ram();n.ancilla_slot_view_mut(slot).set_x(x);n.ancilla_slot_view_mut(slot).set_y(y);
                let s=marker29_machine(&n,0x088d19,slot).unwrap();
                crate::cycle_ledger::assert_rom_marker_work(0x088d19,s,|s| {
                    let pc=u32::from(s.cpu.k)*0x10000+u32::from(s.cpu.pc);
                    (0x088d19..0x088d68).contains(&pc)||(0x08fb44..0x08fba6).contains(&pc)
                        ||pc==0x08a752||pc==0x08a755||(0x08e5d8..0x08e5dc).contains(&pc)
                },||n.ancilla04_beam_hit(slot));
            }
        }
    } } }
}

#[test]
fn prefix27_initial_tile_all_directions_collision_and_clear_paths_exclude_children() {
    for slot in [0usize,9] { for direction in 0..4 { for tile in [0,1] { for (x,y) in [(0x40u16,0x40u16),(0xffff,0xffff)] {
        let mut n=marker29_state();n.write_wram_byte(0xc72+slot,direction);
        n.write_wram_word(0x22,x);n.write_wram_word(0x20,y);
        for a in 0x12000..0x14000 {n.write_wram_byte(a,tile);}
        n.sync_native_game_state_from_ram();let s=marker29_machine(&n,0x099dd3,slot).unwrap();
        crate::cycle_ledger::assert_rom_marker_work(0x099dd3,s,
            |s|s.cpu.k==9 && (0x9dd3..0x9e24).contains(&s.cpu.pc),||{n.ancilla_check_initial_tile_a(slot);});
    } } } }
}

#[test]
fn prefix27_add_sword_beam_all_spawn_facing_and_initial_collision_paths_exclude_children() {
    for free in 0..=6 { for facing in [0,2,4,6] { for tile in [0,1] {
        let mut n=marker29_state();
        for slot in 0..10 {n.write_wram_byte(0xc4a+slot,1);}
        if free<6 {n.write_wram_byte(0xc4a+free,0);}
        n.write_wram_byte(0x2f,facing);n.write_wram_word(0x22,0x40);n.write_wram_word(0x20,0x40);
        for a in 0x12000..0x14000 {n.write_wram_byte(a,tile);}
        n.sync_native_game_state_from_ram();let mut s=marker29_machine(&n,0x0ff67b,0).unwrap();s.cpu.y=0;
        s.cpu.sp=0x1fc;s.ram[0x1fd..0x200].copy_from_slice(&[0xff,0x7f,0x0f]);
        crate::cycle_ledger::assert_rom_marker_work(0x0ff67b,s,
            |s|s.cpu.k==15 && ((0xf67b..0xf74d).contains(&s.cpu.pc)||(0xf64d..0xf64f).contains(&s.cpu.pc)),||n.add_sword_beam(0));
    } } }
}
