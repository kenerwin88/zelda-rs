use super::*;

pub(super) fn state() -> ZeldaState {
    let mut n = ZeldaState::new();
    n.set_assets(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../zelda3_assets.dat")).unwrap()).unwrap();
    n.set_lag_free(true);
    n
}

pub(super) fn machine(n: &ZeldaState, entry: u32, slot: usize) -> snes::Snes {
    let mut s = snes::Snes::new();
    s.cart.load(snes::CartType::LoRom, &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc")).unwrap(), 0x2000);
    s.ram.copy_from_slice(&n.wram_image());
    s.ram[0x1fb..0x200].copy_from_slice(&[0xff,0x7f,0xff,0x7f,0]);
    s.cpu.e = false; s.cpu.unpack_flags(0x30);
    s.cpu.k = (entry >> 16) as u8; s.cpu.db = s.cpu.k;
    s.cpu.pc = entry as u16; s.cpu.x = slot as u16; s.cpu.sp = 0x1fa;
    s
}

#[test]
fn prefix34_fireball_trail_all_frames_slots_allocations_and_coordinate_carries_match_rom() {
    let base = state();
    for k in [0,15] { for frame in 0..4 { for free in 0..=30 { for xy in [0,0xff,0xffff] {
        let mut n = base.clone();
        n.write_wram_byte(0x1a,frame); n.write_wram_byte(0xfa0,k as u8);
        n.write_wram_word(0xfd8,xy); n.write_wram_word(0xfda,xy);
        for i in 0..30 {n.write_wram_byte(0x1f800+i,u8::from(i != free));}
        n.sync_native_game_state_from_ram();
        let source=crate::cycle_ledger::assert_rom_marker_work(0x09b020,machine(&n,0x09b020,k),
            |s|s.cpu.k==9 && (0xb020..0xb06e).contains(&s.cpu.pc),||n.fireball_spawn_trail_garnish(k));
        assert_eq!(&n.ram[0x1f800..0x1f94a],&source.snes().ram[0x1f800..0x1f94a]);
    }}}}
}

#[test]
fn prefix34_direct_animation_all_pose_walk_timers_and_guards_exclude_full_entry_and_dash_child() {
    let base = state();
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for mode in 0..16 {for case in 0..256usize {
        let mut n=base.clone();
        for (a,v) in [(0x5d,if mode==3 {23} else {0}), (0x2f,(case%4*2) as u8),
            (0x2d,case as u8),(0x2e,(case%9) as u8),(0x5e,if mode==12 {6} else {0}),
            (0x34a,u8::from(mode==11 || mode==13)),(0x372,u8::from(mode==1)),
            (0xf0,if case&1==0 {0} else {15}),
            (0x11,match mode {4=>14,5=>18,6=>19,_=>15}),
            (0x308,if mode==7 {128} else {0}),(0x48,if mode==8 {case as u8} else {0}),
            (0x351,u8::from(mode==9)),(0x3c,if mode==10 {1} else {0}),
            (0x2f5,[0,1,2,255][case/16%4])] {n.write_wram_byte(a,v);}
        n.sync_native_game_state_from_ram();
        let source=crate::cycle_ledger::assert_rom_marker_work(0x07e704,machine(&n,0x07e704,0),
            |s| {
                let own=s.cpu.k==7 && (0xe704..0xe7fa).contains(&s.cpu.pc);
                if own {
                    let t=match s.cart.read(7,s.cpu.pc,0) {
                        0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0x90=>Some(!s.cpu.c),0xb0=>Some(s.cpu.c),_=>None};
                    if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                }
                own
            },
            ||n.link_handle_moving_animation_start_with_dash());
        for a in [0x2d,0x2e] {assert_eq!(n.ram[a],source.snes().ram[a],"mode {mode} case {case}");}
    }}
    for pc in [0xe70a,0xe717,0xe723,0xe729,0xe734,0xe73d,0xe741,0xe745,0xe750,0xe756,
        0xe75f,0xe763,0xe769,0xe770,0xe780,0xe789,0xe797,0xe7a1,0xe7ad,0xe7bd,0xe7c6,
        0xe7d2,0xe7d9,0xe7e9,0xe7f2] {for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}

#[test]
fn prefix34_ancilla_single_collision_all_guards_success_returns_and_deflected_arrows_exclude_children() {
    let base = state();
    let edges=std::cell::RefCell::new(std::collections::BTreeSet::new());
    for k in [0,5] { for j in [0,15] { for path in 0..18 { for direction in 0..4 {
        for bow in [0,3] { for overlap in [false,true] {
            let mut n=base.clone();
            let ty=match path {0..=3=>9,4..=7=>5,8..=11=>0x1f,_=>1};
            let sprite=match path {0|1=>0x1b,8|9=>0x8d,12|13=>0x92,_=>0x12};
            for (a,v) in [(0xc4a,ty),(0xc72,direction)] {n.write_wram_byte(a+k,v);}
            n.write_wram_byte(0xbfa+k,0x40); n.write_wram_byte(0xc04+k,0x40);
            for (a,v) in [(0xdd0,9),(0xe20,sprite),(0xd10,if overlap {0x40}else{0xc0}),
                (0xd00,0x40),(0xb6b,if path<4 || path==7 || path==15 {8}else{0}),
                (0xcaa,match path {4|8|14|15=>16,5|9=>2,_=>0}),
                (0xde0,if path&1==0 {direction}else{direction^2}),
                (0xef0,u8::from(path==6 || path==10)),(0xba0,u8::from(path==16)),
                (0xdb0,if path==12 {0}else{3}),(0xf50,8)] {n.write_wram_byte(a+j,v);}
            n.write_wram_byte(0x1a,j as u8&3); n.write_wram_byte(0xf340,bow);
            n.sync_native_game_state_from_ram();
            let source=crate::cycle_ledger::assert_rom_marker_owners_work(&[0x088dae,0x088ab9],machine(&n,0x088d68,k),
                |s| {
                    let own=s.cpu.k==8 && (0x8dae..0x8e75).contains(&s.cpu.pc);
                    if own {
                        let t=match s.cart.read(8,s.cpu.pc,0) {
                            0xf0=>Some(s.cpu.z),0xd0=>Some(!s.cpu.z),0x90=>Some(!s.cpu.c),0xb0=>Some(s.cpu.c),_=>None};
                        if let Some(t)=t {edges.borrow_mut().insert((s.cpu.pc,t));}
                    }
                    own || s.cpu.k==8 && (0x8ab9..0x8abf).contains(&s.cpu.pc)
                },||{n.ancilla_check_sprite_collision_marked(k);});
            for a in [0xc72+k,0xbb0+j,0xfdc] {
                assert_eq!(n.ram[a],source.snes().ram[a],"path {path}, direction {direction}, address {a:x}");
            }
        }}
    }}}}
    for pc in [0x8dbe,0x8dc8,0x8dcf,0x8dd6,0x8de4,0x8df3,0x8e09,0x8e10,0x8e14,
        0x8e1b,0x8e20,0x8e27,0x8e33,0x8e3a,0x8e41] { for t in [false,true] {
        assert!(edges.borrow().contains(&(pc,t)),"branch {pc:x}/{t}");
    }}
}
