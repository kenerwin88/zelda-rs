//! Native animated-item execution from the existing caller timing handoff.
//! The upstream caller and its return tail still have separate owners.
use super::*;
use crate::game_state::constants::{SCRATCH_0};
use crate::cycle_models::{
    animated_item_graphics::AnimatedItemGraphics,
    cardinal_collision_prefix::CardinalCollisionPrefix,
    diagonal_collision_prefix::DiagonalCollisionPrefix,
    ground_item_apress_return::GroundItemApressReturn,
    ground_movement_prefix::GroundMovementPrefix,
    ground_sword_cape_return::GroundSwordCapeReturn,
    ground_velocity::GroundVelocity,
    ground_animation_return::GroundAnimationReturn,
    dungeon_main_return::DungeonMainReturn,
    ground_y_item_prefix::{GroundYItemPrefix, GroundYItemTail},
    intro_clear::Access,
    item_receipt_return::ItemReceiptReturnWork,
    link_receive_item_return::LinkReceiveItemReturn,
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    tile_classifier::{TileClassifier, TileClassifierTail},
    main_tile_probe::MainTileProbe,
    main_tile_response::MainTileResponse,
    vertical_cardinal_probe::{VerticalCardinalProbe, VerticalCardinalTail},
    vertical_collision_response::{VerticalCollisionResponse, VerticalResponseTail},
    vertical_slope_probe::VerticalSlopeProbe,
    vertical_surface_response::{VerticalSurfaceResponse, VerticalSurfaceTail},
};
#[cfg(test)]
use crate::cycle_models::main_tile_response::MainTileResponseTail;
use snes::{cpu_timeline::CpuSynchronousTimelineEvent, CpuMasterTimeline, HdmaTimingState};

// Two repeating 120-line groups consume the live scroll words at $1B00
// and $1BF0. This is immutable HDMA descriptor data, not CPU instructions.
pub(super) fn scroll_hdma_descriptor_byte(address: u32) -> Option<u8> {
    let offset = address.checked_sub(0x00f2fb)? as usize;
    [0xf8, 0x00, 0x1b, 0xf8, 0xf0, 0x1b, 0x00]
        .get(offset)
        .copied()
}

#[derive(Clone, Debug)]
pub(super) struct NativeItemGraphicsEntry {
    pub(super) context: NativeNmiContext,
    pub(super) timeline: CpuMasterTimeline,
    pub(super) hdma: HdmaTimingState,
    pub(super) saved_link_x: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn live_item_bus_matches_source_using_assets_without_a_runtime_rom() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x00f2fb..=0x00f301 {
            assert_eq!(
                scroll_hdma_descriptor_byte(address),
                Some(rom[address as usize - 0x8000])
            );
        }
        for (vertical_probe,submodule,cached_active) in [(false,17u8,false),(true,17,false),(true,0,false),(true,17,true)] {
            let mut state = ZeldaState::new();
            state
                .set_assets(
                    &std::fs::read(
                        std::env::var("ZELDA3_ASSETS")
                            .expect("live graphics fixture requires native assets"),
                    )
                    .unwrap(),
                )
                .unwrap();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1e2..0x1e5].copy_from_slice(&[0xa3, 0x87, 9]);
            state.ram[0x1e5..0x1ec].copy_from_slice(&[4, 0x1b, 0x70, 7, 0x0a, 0x9a, 7]);
            state.ram[0x72] = 0x14;
            state.ram[0x2d8] = 0x12;
            state.ram[0x2e9] = 1;
            state.ram[0x1ec..0x1f1].copy_from_slice(&[0x24, 7, 0xb9, 0xb5, 7]);
            state.ram[0x1f1..0x1f5].copy_from_slice(&[0x40, 0x9c, 0xb2, 0x81]);
            state.ram[0x1f5..0x1fd].copy_from_slice(&[0x1b, 0x80, 0, 0x4a, 0x89, 2, 0xae, 0x87]);
            state.write_wram_byte(0x11, submodule);
            state.ram[0xaaa]=0;
            state.ram[0xae..0xb0].fill(0);
            state.ram[0x4c7]=0;
            state.ram[0x436]=0x80;
            state.ram[0x454]=0;
            state.ram[0x2c3]=0; // Valid block animation for the newly owned drawing child.
            state.ram[0xfac]=0; // Inactive weapon tink; active-body handoffs are checked separately.
            state.ram[0xdd0..0xde0].fill(0); // Valid inactive sprites for the compound slot pass.
            state.ram[0x1d00..0x1d10].fill(0); // Valid cache guards for the compound return.
            if cached_active {state.ram[0x1d07]=3;state.ram[0x1dd7]=0;state.ram[0xee]=0;state.ram[0xffa]=1;}
            state.ram[0xc4a..0xc54].fill(0); // Empty valid ancilla slots for the compound return.
            state.ram[0xf3cc]=5; // Valid timed follower, matching the ordinary dungeon route.
            state.ram[0x2cd..0x2cf].copy_from_slice(&1u16.to_le_bytes());
            state.ram[0xff9]=0; // Inactive flash; palette children have independent handoff coverage.
            state.ram[0x1f800..0x1f81e].fill(0);
            state.ram[0x42c..0x42e].fill(0);
            state.ram[0x478..0x47a].copy_from_slice(&6u16.to_le_bytes());
            state.ram[0x500..0x506].fill(0x11);
            state.ram[0x36c] = 5;
            { let mut image = state.wram_image().into_owned(); image[0xf340..0xf37c].fill(0); state.import_wram_image(&image); }
            state.write_wram_byte(0xf34a, 1);
            for address in [
                0x308, 0x376, 0x377, 0x5d, 0x3c, 0x3a, 0x2e0, 0x3fc, 0x301, 0x37a, 0x6c, 0x46,
                0xf4, 0x2e1, 0x34a, 0x49, 0x46c, 0x372, 0x5e,
            ] {
                state.ram[address] = 0;
            }
            state.ram[0x2e]=8;
            state.ram[0x2f]=0;
            state.ram[0x304] = 9;
            state.ram[0xf0] = if vertical_probe { 8 } else { 1 };
            state.write_wram_byte(0xf3c5, 0);
            state.write_wram_byte(0xf36c, 24);
            state.write_wram_byte(0xf36d, 24);
            state.write_wram_byte(0xf36f, 255);
            { let mut image = state.wram_image().into_owned(); image[0x202..0x205].fill(0); state.import_wram_image(&image); }
            state.ram[0x12000..0x14000].fill(0);
            state.ram[0x1b] = 1;
            for address in [0xb7b, 0x5b, 0x57, 0x351, 0x3f3] {
                state.ram[address] = 0;
            }
            state.ram[0x322] = 3;
            state.ram[0xb7c..0xb80].fill(0);
            state.sync_native_game_state_from_ram();
            let mut context = NativeNmiContext {
                a: 0x1b14,
                x: 255,
                y: 18,
                d: 0,
                db: 9,
                bank: 0,
                pc: 0xd4ed,
                sp: 0x1e1,
                p: 0xb0,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = context.a;
            machine.cpu.x = context.x;
            machine.cpu.y = context.y;
            machine.cpu.db = context.db;
            machine.cpu.pc = context.pc;
            machine.cpu.sp = context.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(context.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4582,
                CpuRasterPosition::new(14, 966),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            let mut work = AnimatedItemGraphics::default();
            let mut receipt_work: Option<ItemReceiptReturnWork> = None;
            let mut link_work: Option<LinkReceiveItemReturn> = None;
            let mut apress_work: Option<GroundItemApressReturn> = None;
            let mut y_work: Option<GroundYItemPrefix> = None;
            let mut sword_work: Option<GroundSwordCapeReturn> = None;
            let mut movement_work: Option<GroundMovementPrefix> = None;
            let mut velocity_work: Option<GroundVelocity> = None;
            let mut cardinal_work: Option<CardinalCollisionPrefix> = None;
            let mut vertical_cardinal_work: Option<VerticalCardinalProbe> = None;
            let mut vertical_response_work: Option<VerticalCollisionResponse> = None;
            let mut vertical_surface_work: Option<VerticalSurfaceResponse> = None;
            let mut ground_animation_work: Option<GroundAnimationReturn> = None;
            let mut main_response_work: Option<MainTileResponse> = None;
            let mut main_tile_work: Option<MainTileProbe> = None;
            let mut main_tile_classifier: Option<TileClassifier> = None;
            let mut cardinal_classifiers: [Option<TileClassifier>; 3] = Default::default();
            let mut diagonal_work: Option<DiagonalCollisionPrefix> = None;
            let mut probe_work: Option<VerticalSlopeProbe> = None;
            let mut classifier_work: Option<TileClassifier> = None;
            let mut high_classifier_work: Option<TileClassifier> = None;
            let mut count = 0;
            loop {
                let ready = if let Some(work)=&ground_animation_work {work.complete()} else if let Some(work)=&main_response_work {work.tail().is_some_and(|t|t!=MainTileResponseTail::Returned)} else if let Some(work)=&main_tile_work {
                    work.child_ready() && main_tile_classifier.as_ref().is_some_and(|c|matches!(c.tail(),Some(TileClassifierTail::Effect {..}|TileClassifierTail::OutdoorAttribute)))
                } else if let Some(work) = &vertical_surface_work {
                    work.tail()
                        .is_some_and(|tail| tail != VerticalSurfaceTail::Returned)
                } else if let Some(work) = &vertical_response_work {
                    work.tail().is_some_and(|tail| {
                        !matches!(
                            tail,
                            VerticalResponseTail::Returned | VerticalResponseTail::SurfaceTail
                        )
                    })
                } else if let Some(probe) = &vertical_cardinal_work {
                    probe.child_ready()
                        && cardinal_classifiers[probe.sample()]
                            .as_ref()
                            .is_some_and(|c| {
                                matches!(
                                    c.tail(),
                                    Some(
                                        TileClassifierTail::Effect { .. }
                                            | TileClassifierTail::OutdoorAttribute
                                    )
                                )
                            })
                } else {
                    cardinal_work.as_ref().is_some_and(|w| w.tail().is_some())
                        && context.pc != 0xba0a && context.pc != 0xd077
                };
                if ready {
                    break;
                }

                if vertical_surface_work
                    .as_ref()
                    .is_some_and(|w| w.tail() == Some(VerticalSurfaceTail::Returned))
                {
                    cardinal_work
                        .as_mut()
                        .unwrap()
                        .resume_after_vertical_axis(context.pc);
                    vertical_surface_work = None;
                    vertical_response_work = None;
                    vertical_cardinal_work = None;
                    cardinal_classifiers = Default::default();
                }
                if vertical_response_work
                    .as_ref()
                    .is_some_and(|w| w.tail() == Some(VerticalResponseTail::SurfaceTail))
                    && vertical_surface_work.is_none()
                {
                    assert_eq!(context.pc, 0xbbe6);
                    vertical_surface_work = Some(VerticalSurfaceResponse::default());
                }
                if vertical_response_work
                    .as_ref()
                    .is_some_and(|w| w.tail() == Some(VerticalResponseTail::Returned))
                {
                    cardinal_work
                        .as_mut()
                        .unwrap()
                        .resume_after_vertical_axis(context.pc);
                    vertical_response_work = None;
                    vertical_cardinal_work = None;
                    cardinal_classifiers = Default::default();
                }
                if vertical_cardinal_work
                    .as_ref()
                    .is_some_and(|p| p.tail() == Some(VerticalCardinalTail::Response))
                    && vertical_response_work.is_none()
                {
                    assert_eq!(context.pc, 0xba2e);
                    vertical_response_work = Some(VerticalCollisionResponse::default());
                }
                if vertical_cardinal_work
                    .as_ref()
                    .is_some_and(|p| p.tail() == Some(VerticalCardinalTail::Returned))
                {
                    cardinal_work
                        .as_mut()
                        .unwrap()
                        .resume_after_vertical_axis(context.pc);
                    vertical_cardinal_work = None;
                    cardinal_classifiers = Default::default();
                }
                if cardinal_work.as_ref().is_some_and(|p|matches!(p.tail(),Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::SlopeChecks {..}))) && context.pc==0xba0a && vertical_cardinal_work.is_none() {
                vertical_cardinal_work=Some(VerticalCardinalProbe::default());
            }
                if let Some(probe) = &mut vertical_cardinal_work {
                    if probe.child_ready() {
                        let classifier = &mut cardinal_classifiers[probe.sample()];
                        if classifier.is_none() {
                            *classifier = Some(TileClassifier::default());
                        }
                        if classifier.as_ref().unwrap().tail() == Some(TileClassifierTail::Complete)
                        {
                            probe.begin_next();
                        }
                    }
                }
                if main_response_work.as_ref().is_some_and(|w|w.tail()==Some(MainTileResponseTail::Returned)) && ground_animation_work.is_none() {assert_eq!(context.pc,0x82c5);ground_animation_work=Some(GroundAnimationReturn::default().with_native_push_handler(state.native_push_handler_ready()).with_native_after_push(true).with_native_room_tags(!state.game_state.enhanced_features.has(4096)).with_native_interactions(true).with_native_edge(true).with_native_lamp(true).with_native_sprite_scroll(true).with_native_drawing(true).with_native_sprite_prefix(true).with_native_garnish(true).with_native_follower(true).with_native_ancilla(true));}
                if main_tile_work.as_ref().is_some_and(MainTileProbe::complete) && main_response_work.is_none() {assert_eq!(context.pc,0xd104);main_response_work=Some(MainTileResponse::default());}
                if cardinal_work.as_ref().is_some_and(|w|w.tail()==Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::MainHandler)) && main_tile_work.is_none() {
                    assert_eq!(context.pc,0xd077);main_tile_work=Some(MainTileProbe::default());
                }
                if let Some(work)=&mut main_tile_work {
                    if work.child_ready() {
                        if main_tile_classifier.is_none() {main_tile_classifier=Some(TileClassifier::default());}
                        if main_tile_classifier.as_ref().unwrap().tail()==Some(TileClassifierTail::Complete) {work.begin_response();}
                    }
                }
                if probe_work
                    .as_ref()
                    .is_some_and(VerticalSlopeProbe::complete)
                {
                    assert_eq!(context.pc, 0xb682);
                    diagonal_work
                        .as_mut()
                        .unwrap()
                        .resume_after_vertical_probe();
                    probe_work = None;
                    classifier_work = None;
                    high_classifier_work = None;
                }
                if work.complete() && receipt_work.is_none() {
                    assert_eq!((context.bank, context.pc, context.sp), (9, 0x87a4, 0x1e4));
                    receipt_work = Some(ItemReceiptReturnWork::default());
                }
                if receipt_work
                    .as_ref()
                    .is_some_and(ItemReceiptReturnWork::complete)
                    && link_work.is_none()
                {
                    assert_eq!((context.bank, context.pc, context.sp), (7, 0x9a0b, 0x1eb));
                    link_work = Some(LinkReceiveItemReturn::default());
                }
                if link_work
                    .as_ref()
                    .is_some_and(LinkReceiveItemReturn::complete)
                    && apress_work.is_none()
                {
                    assert_eq!((context.bank, context.pc, context.sp), (7, 0xb5ba, 0x1f0));
                    apress_work = Some(GroundItemApressReturn::default());
                }
                if apress_work
                    .as_ref()
                    .is_some_and(GroundItemApressReturn::complete)
                    && y_work.is_none()
                {
                    assert_eq!((context.bank, context.pc, context.sp), (7, 0x81b3, 0x1f4));
                    y_work = Some(GroundYItemPrefix::default());
                }
                if y_work
                    .as_ref()
                    .is_some_and(|work| work.tail() == Some(GroundYItemTail::AfterYItem))
                    && sword_work.is_none()
                {
                    sword_work = Some(GroundSwordCapeReturn::from_entry(context.pc));
                }
                if sword_work.as_ref().is_some_and(|work| work.tail() == Some(crate::cycle_models::ground_sword_cape_return::GroundSwordCapeTail::Movement)) && movement_work.is_none() {
                movement_work = Some(GroundMovementPrefix::default());
            }
                if movement_work.as_ref().is_some_and(|work| matches!(work.tail(), Some(crate::cycle_models::ground_movement_prefix::GroundMovementTail::Collision { .. }))) && diagonal_work.is_none() {
                diagonal_work = Some(DiagonalCollisionPrefix::default());
            }
                if velocity_work.as_ref().is_some_and(|w| {
                    w.tail()
                        == Some(crate::cycle_models::ground_velocity::GroundVelocityTail::Complete)
                }) && cardinal_work.is_none()
                {
                    assert_eq!((context.bank, context.pc), (7, 0x82c2));
                    cardinal_work = Some(CardinalCollisionPrefix::default());
                }
                if diagonal_work.as_ref().is_some_and(|work| matches!(work.tail(), Some(crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::VerticalProbe { .. }))) && probe_work.is_none() {
                probe_work = Some(VerticalSlopeProbe::default());
            }
                if probe_work
                    .as_ref()
                    .is_some_and(VerticalSlopeProbe::child_ready)
                    && classifier_work.is_none()
                {
                    classifier_work = Some(TileClassifier::default());
                }
                if let Some(probe) = &mut probe_work {
                    if probe.corner() == 0
                        && classifier_work
                            .as_ref()
                            .is_some_and(|w| w.tail() == Some(TileClassifierTail::Complete))
                    {
                        probe.begin_high_corner();
                    }
                    if probe.corner() == 1 && probe.child_ready() && high_classifier_work.is_none()
                    {
                        high_classifier_work = Some(TileClassifier::default());
                    }
                    if probe.corner() == 1
                        && high_classifier_work
                            .as_ref()
                            .is_some_and(|w| w.tail() == Some(TileClassifierTail::Complete))
                    {
                        probe.begin_return();
                    }
                }
                if diagonal_work.as_ref().is_some_and(|w| w.tail() == Some(crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::Complete)) && velocity_work.is_none() {
                    velocity_work = Some(GroundVelocity::default());
                }
                let pc = context.pc;
                source.step().unwrap();
                // Compare the production memory bus independently of the APU
                // scanline owner, which requires a full cold bootstrap.
                let publish_hud = link_work
                    .as_ref()
                    .is_some_and(LinkReceiveItemReturn::hud_active);
                let publish_link = link_work.is_some();
                let publish_probe = probe_work.is_some() || vertical_cardinal_work.is_some() || main_tile_work.is_some();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_hud,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_link,
                        publish_probe,
                    }
                    .access(at, access)
                };
                if let Some(work)=&mut ground_animation_work {work.step(&mut context,&mut clock,bus).unwrap();}
                else if let Some(work)=&mut main_response_work {work.step(&mut context,&mut clock,bus).unwrap();}
                else if let Some(work)=&mut main_tile_work {
                    if work.child_ready() {main_tile_classifier.as_mut().unwrap().step(&mut context,&mut clock,bus).unwrap();}
                    else {work.step(&mut context,&mut clock,bus).unwrap();}
                } else if let Some(work) = &mut vertical_surface_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut vertical_response_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(probe) = &mut vertical_cardinal_work {
                    if probe.child_ready() {
                        cardinal_classifiers[probe.sample()]
                            .as_mut()
                            .unwrap()
                            .step(&mut context, &mut clock, bus)
                            .unwrap();
                    } else {
                        probe.step(&mut context, &mut clock, bus).unwrap();
                    }
                } else if let Some(work) = &mut cardinal_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut velocity_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) =
                    high_classifier_work.as_mut().filter(|w| w.tail().is_none())
                {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = classifier_work.as_mut().filter(|w| w.tail().is_none()) {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut probe_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut diagonal_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut movement_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut sword_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut y_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut apress_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut link_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else if let Some(work) = &mut receipt_work {
                    work.step(&mut context, &mut clock, bus).unwrap();
                } else {
                    work.step(&mut context, &mut clock, bus).unwrap();
                }
                let c = &source.snes().cpu;
                assert_eq!(
                    context,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "pc={pc:04x}");
                if *state.wram_image() != source.snes().ram {
                    let differences = state
                        .wram_image()
                        .iter()
                        .zip(source.snes().ram.iter())
                        .enumerate()
                        .filter_map(|(offset, (&native, &source))| {
                            (native != source).then_some((offset, native, source))
                        })
                        .take(16)
                        .collect::<Vec<_>>();
                    panic!("pc={pc:04x} WRAM differences: {differences:x?}");
                }
                count += 1;
                assert!(count < 100000);
            }
            assert_eq!(y_work.unwrap().tail(), Some(GroundYItemTail::AfterYItem));
            assert_eq!(
                sword_work.unwrap().tail(),
                Some(crate::cycle_models::ground_sword_cape_return::GroundSwordCapeTail::Movement)
            );
            assert_eq!(
                movement_work.unwrap().tail(),
                Some(
                    crate::cycle_models::ground_movement_prefix::GroundMovementTail::Collision {
                        clear_vel_after: false
                    }
                )
            );
            assert!(probe_work.is_none());
            assert!(classifier_work.is_none());
            assert!(high_classifier_work.is_none());
            assert_eq!(
                diagonal_work.unwrap().tail(),
                Some(
                    crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::Complete
                )
            );
            assert_eq!(
                velocity_work.unwrap().tail(),
                Some(crate::cycle_models::ground_velocity::GroundVelocityTail::Complete)
            );
            assert!(vertical_cardinal_work.is_none());
            assert!(vertical_response_work.is_none());
            assert!(vertical_surface_work.is_none());
            if vertical_probe {
                assert_eq!((context.bank,context.pc,context.sp,context.db,context.p&0x30),if cached_active {(6,0x84eb,0x1cd,6,0x30)}else{(2,0x8842,0x1f4,0,0x30)});
                assert!(ground_animation_work.unwrap().complete());
                assert_eq!(main_response_work.unwrap().tail(),Some(MainTileResponseTail::Returned));
                assert!(main_tile_work.unwrap().complete());
                assert_eq!(main_tile_classifier.unwrap().tail(),Some(TileClassifierTail::Complete));
                assert_eq!(cardinal_work.unwrap().tail(),Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::MainHandler));
            } else {
                assert_eq!(context.pc, 0xc4d9);
                assert_eq!(cardinal_work.unwrap().tail(),Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::SlopeChecks {primary_layer:false,horizontal_first:false,vertical_completed:true}));
            }
            assert!(state.rom.is_empty());
            assert!(deadline.is_some());
            println!("live native asset bus vertical_probe={vertical_probe} submodule={submodule} matched {count} source instructions without a runtime ROM");
        }
    }

    #[test]
    fn live_ground_y_item_bus_publishes_lamp_activation_and_cleanup_once() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for variant in 0..4 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            state.ram.fill(0);
            state.ram[0x303] = 9;
            state.ram[0x304] = if variant == 3 { 8 } else { 9 };
            state.write_wram_byte(0xf34a, u8::from(variant != 1));
            state.ram[0xf4] = if variant == 0 { 0 } else { 0x40 };
            state.ram[0x50] = 0xa5;
            if variant == 3 {
                state.write_wram_byte(0xf34c, 2);
                state.ram[0x3a] = 0x40;
            }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x240a,
                x: 10,
                y: 254,
                d: 0,
                db: 7,
                bank: 7,
                pc: 0x81b3,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(21, 8),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundYItemPrefix::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "variant={variant} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "variant={variant} pc={pc:04x}");
                assert_eq!(
                    *state.wram_image(),
                    source.snes().ram,
                    "variant={variant} pc={pc:04x}"
                );
                assert_eq!(
                    state.game_state.player.follower_link.current_item_active(),
                    state.ram[0x304]
                );
                assert_eq!(
                    state.game_state.player.follower_link.button_mask_b_y(),
                    state.ram[0x3a]
                );
                assert_eq!(
                    state.game_state.player.follower_link.button_b_frames(),
                    state.ram[0x3c]
                );
                count += 1;
            }
            assert!(state.rom.is_empty());
            assert_eq!(
                work.tail(),
                Some(if variant < 2 {
                    GroundYItemTail::AfterYItem
                } else {
                    GroundYItemTail::LampMagic
                })
            );
            assert_eq!(state.ram[0x3a], if variant < 2 { 0 } else { 0x40 });
            assert_eq!(state.ram[0x50], if variant == 1 { 0 } else { 0xa5 });
        }
        println!(
            "live Y-item bus: 4 cases, {count} source instructions, typed lamp stores coherent"
        );
    }
}

#[cfg(test)]
mod sword_cape_bus_tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn live_ground_sword_cape_bus_publishes_each_timer_magic_and_velocity_store() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..8u8 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            state.ram.fill(0);
            state.ram[0x2e3] = if case == 0 { 1 } else { 0 };
            state.ram[0x372] = 1;
            state.write_wram_byte(0xf3c5, 1);
            state.ram[0x5d] = if case == 2 { 3 } else { 0 };
            state.ram[0x30] = 0xa5;
            state.ram[0x31] = 0x5a;
            let pc = if case < 3 { 0x81c9 } else { 0x81df };
            if case >= 3 {
                state.ram[0x308] = 0x80;
                state.ram[0x55] = 1;
                state.ram[0x303] = 19;
                state.ram[0x304] = 19;
                state.ram[0x4c] = if case == 3 { 2 } else { 1 };
                state.write_wram_byte(0xf37b, (case - 3) % 3);
                state.write_wram_byte(0xf36e, match case {
                    6 => 1,
                    7 => 0,
                    _ => 2,
                });
            }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundSwordCapeReturn::from_entry(pc);
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                assert_eq!(published[0x2e3], state.ram[0x2e3]);
                assert_eq!(link.cape_decrement_counter(), state.ram[0x4c]);
                assert_eq!(link.y_velocity(), state.ram[0x30]);
                assert_eq!(link.x_velocity(), state.ram[0x31]);
                assert_eq!(
                    state.game_state.inventory.player_resources.magic.amount(),
                    state.read_wram_byte(0xf36e)
                );
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live sword/cape bus: 8 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ground_movement_bus_publishes_reset_and_direction_stores() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..10u8 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            for a in [
                0x46, 0x377, 0x2e1, 0x376, 0x308, 0x309, 0x301, 0x37a, 0x3c, 0x3a, 0x34a, 0x49,
            ] {
                state.ram[a] = 0;
            }
            state.ram[0xf0] = if case == 0 { 0 } else { 8 };
            state.ram[0x26] = if case == 1 { 8 } else { 0 };
            match case {
                2 => state.ram[0x49] = 4,
                3 => state.ram[0x34a] = 1,
                4 => state.ram[0x377] = 1,
                5 => state.ram[0x2e1] = 1,
                6 => {
                    state.ram[0x46] = 1;
                    state.ram[0x3a] = 0x80;
                }
                7 => state.ram[0x46] = 1,
                8 => {
                    state.ram[0x376] = 2;
                    state.ram[0x49] = 8;
                }
                9 => {
                    state.ram[0x3c] = 8;
                    state.ram[0x3a] = 0x80;
                }
                _ => {}
            }
            let pc = 0x81e2;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundMovementPrefix::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x26, 0x2a, 0x2b, 0x2e, 0x30, 0x31, 0x3b, 0x48, 0x50, 0x67, 0x6b, 0x308, 0x309,
                    0x30a, 0x30d, 0x30e, 0x326, 0x327, 0x328, 0x329, 0x371, 0x375, 0x376,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed player case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(&published[0x32b..0x340], &state.wram_image()[0x32b..0x340]);
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live ground movement bus: 10 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_diagonal_collision_bus_publishes_room_relocation_and_direction_stores() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..16u8 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1f3..0x1f5].copy_from_slice(&[0xbd, 0x82]);
            state.ram[0x46c] = [0, 1, 2, 3, 4, 5, 128, 255][(case / 2) as usize];
            state.ram[0x67] = if case & 1 == 0 { 8 } else { 0 };
            state.ram[0x6b] = 0;
            let pc = 0xb64f;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f2,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = DiagonalCollisionPrefix::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x42, 0x43, 0x66, 0x67, 0x6a, 0xee, 0x318,
                    0x319, 0x31a, 0x31b,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live diagonal collision bus: 16 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_vertical_direction_return_bus_publishes_masks_and_swim_byte_resets() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            let primary_layer = case & 1 != 0;
            state.ram[0x1f1..0x1f5].copy_from_slice(&[0x56, 0xb6, 0xbd, 0x82]);
            state.ram[0xe] = (case / 2) as u8;
            state.ram[0xc] = ((case / 2) as u8).rotate_left(2);
            state.ram[0x62] = (case >> 6) as u8;
            state.ram[0x46c] = ((case >> 1) & 1) as u8;
            state.ram[0x67] = [8, 10, 9, 12][((case >> 2) & 3) as usize];
            state.ram[0x4d] = ((case >> 4) & 1) as u8;
            state.ram[0x34a] = ((case >> 5) & 1) as u8;
            state.ram[0x2e8] = ((case >> 6) & 3) as u8;
            state.ram[0x66] = ((case >> 3) & 1) as u8;
            state.ram[0x42] = 15;
            state.ram[0x43] = 15;
            state.ram[0x6a] = (case & 3) as u8;
            let pc = 0xb682;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: if primary_layer { 0x1f0 } else { 0x1f2 },
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = DiagonalCollisionPrefix::after_vertical_probe(primary_layer);
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x302, 0x6b, 0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x42, 0x43, 0x66, 0x67, 0x6a,
                    0xee, 0x318, 0x319, 0x31a, 0x31b,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live vertical direction return bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_cardinal_collision_bus_publishes_byte_resets_caches_and_layer_selection() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1280u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x6b] = [0, 0x10, 0x20, 0x30][(case & 3) as usize];
            state.ram[0x30] = [0, 1, 127, 128, 255][(case as usize / 4) % 5];
            state.ram[0x31] = [0, 1, 127, 128, 255][(case as usize / 20) % 5];
            state.ram[0x46c] = ((case / 100) % 5) as u8;
            state.ram[0x5d] = [0, 3, 4, 8, 9, 10, 19, 11][((case >> 4) & 7) as usize];
            for (n, address) in [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate() {
                state.ram[address..address + 2].copy_from_slice(
                    &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                        [((case as usize) + n) % 8]
                        .to_le_bytes(),
                );
            }
            state.ram[0xe] = (case & 15) as u8;
            state.ram[0x316] = ((case >> 4) & 15) as u8;
            state.ram[0x59] = (case & 15) as u8;
            state.ram[0x372] = (case & 1) as u8;
            state.ram[0x31f] = (case & 2) as u8;
            state.ram[0x67] = (case & 15) as u8;
            state.ram[0x310] = [0, 1, 127, 128, 255][(case as usize) % 5];
            state.ram[0x312] = [0, 1, 127, 128, 255][((case as usize) / 5) % 5];
            let pc = if case < 512 {
                0x82c2
            } else {
                [0xb7f8, 0xb892, 0xb879][((case - 512) / 256) as usize]
            };
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: if pc == 0x82c2 { 0x1f4 } else { 0x1f2 },
                p: if pc == 0xb7f8 { 0x10 } else { 0x30 },
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = if pc == 0x82c2 {
                CardinalCollisionPrefix::default()
            } else {
                CardinalCollisionPrefix::after_footprint_for_test(pc)
            };
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x6b, 0x6d, 0xee, 0x5d, 0x5e, 0x31f, 0x2dc,
                    0x2dd, 0x2de, 0x2df, 0x315, 0x318, 0x319, 0x31a, 0x31b,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                for address in [0x38, 0x39, 0x6e, 0x6f, 0x316] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "tile case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live cardinal collision bus: 1280 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_vertical_cardinal_bus_preserves_reset_neighbors_and_three_samples() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1ef..0x1f1].copy_from_slice(&[0x5e, 0xb9]);
            state.ram[0x30] = if case % 16 == 0 {
                0
            } else {
                if case & 1 == 0 {
                    1
                } else {
                    255
                }
            };
            state.ram[0x6c] = (case & 3) as u8;
            state.ram[0x1b] = u8::from(case < 256);
            state.ram[0xee] = (case & 1) as u8;
            state.ram[0x37f] = u8::from(case & 16 != 0);
            state.ram[0x12000..0x15000].fill(case as u8);
            for (n, address) in [0x20, 0x22].into_iter().enumerate() {
                state.ram[address..address + 2].copy_from_slice(
                    &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                        [((case as usize) + n) % 8]
                        .to_le_bytes(),
                );
            }
            state.ram[0xec..0xee].copy_from_slice(&0x1ffu16.to_le_bytes());
            let pc = 0xba0a;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1ee,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = VerticalCardinalProbe::default();
            let mut classifiers: [Option<TileClassifier>; 3] = Default::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                let sample = work.sample();
                if work.child_ready() && classifiers[sample].is_none() {
                    classifiers[sample] = Some(TileClassifier::default());
                }
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                if work.child_ready() {
                    classifiers[sample]
                        .as_mut()
                        .unwrap()
                        .step(&mut cpu, &mut clock, bus)
                        .unwrap();
                } else {
                    work.step(&mut cpu, &mut clock, bus).unwrap();
                }
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20, 0x21, 0x22, 0x23, 0x49, 0x4a, 0x66] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert_eq!(published, *state.wram_image(), "tile case={case} pc={pc:04x}");
                count += 1;
                classifiers = classifiers.clone();
                if work.child_ready() && classifiers[sample].is_some() {
                    match classifiers[sample].as_ref().unwrap().tail() {
                        Some(TileClassifierTail::Complete) => work.begin_next(),
                        Some(_) => break,
                        None => {}
                    }
                }
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live vertical cardinal bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_main_tile_probe_bus_preserves_reset_neighbors_and_coordinates() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1f1..0x1f3].copy_from_slice(&[0xd7,0xb8]);
            state.ram[0x2f]=((case&3)*2) as u8;
            state.ram[0x30] = if case % 16 == 0 {
                0
            } else {
                if case & 1 == 0 {
                    1
                } else {
                    255
                }
            };
            state.ram[0x6c] = (case & 3) as u8;
            state.ram[0x1b] = u8::from(case < 256);
            state.ram[0xee] = (case & 1) as u8;
            state.ram[0x37f] = u8::from(case & 16 != 0);
            state.ram[0x12000..0x15000].fill(case as u8);
            for (n, address) in [0x20, 0x22].into_iter().enumerate() {
                state.ram[address..address + 2].copy_from_slice(
                    &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                        [((case as usize) + n) % 8]
                        .to_le_bytes(),
                );
            }
            state.ram[0xec..0xee].copy_from_slice(&0x1ffu16.to_le_bytes());
            let pc = 0xd077;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: [0,4,5][case as usize%3],
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f0,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = MainTileProbe::default();
            let mut classifier: Option<TileClassifier> = None;
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                if work.child_ready() && classifier.is_none() {
                    classifier = Some(TileClassifier::default());
                }
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                if work.child_ready() {
                    classifier
                        .as_mut()
                        .unwrap()
                        .step(&mut cpu, &mut clock, bus)
                        .unwrap();
                } else {
                    work.step(&mut cpu, &mut clock, bus).unwrap();
                }
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20, 0x21, 0x22, 0x23, 0x49, 0x4a, 0x66] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert_eq!(published, *state.wram_image(), "tile case={case} pc={pc:04x}");
                count += 1;
                classifier = classifier.clone();
                if work.child_ready() && classifier.is_some() {
                    match classifier.as_ref().unwrap().tail() {
                        Some(TileClassifierTail::Complete) => work.begin_response(),
                        Some(_) => break,
                        None => {}
                    }
                }
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live main tile probe bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_main_tile_response_bus_preserves_typed_effects_and_swim_reset() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::main_tile_response::seed_case(&mut image,((case>>5)*17) as u8,(case&31) as u8); state.import_wram_image(&image); }
            let pc=0xd104;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f0,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = MainTileResponse::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu,&mut clock,bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20,0x21,0x22,0x23,0x26,0x30,0x31,0x49,0x4a,0x56,0x5d,0x66,0x67,0x6b,0x2cb,0x2db,0x2ed,0x2e0,0x31f,0x32a,0x340,0x345,0x34a,0x34f,0x351,0x373,0x3f5,0x3f6,0x3f7] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert!(published==*state.wram_image(),"tile case={case} pc={pc:04x} first_difference={:?}",published.iter().zip(state.wram_image().iter()).position(|(a,b)|a!=b));
                count += 1;

            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live main tile response bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ground_animation_return_bus_publishes_facing_steps_counters_and_cleanup() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::ground_animation_return::seed_case(&mut image,((case>>5)*17) as usize,(case&31) as usize); state.import_wram_image(&image); }
            let pc=0x82c5;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundAnimationReturn::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu,&mut clock,bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20,0x21,0x22,0x23,0x26,0x2d,0x2e,0x2f,0x30,0x31,0x302,0x303] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert!(published==*state.wram_image(),"tile case={case} pc={pc:04x} first_difference={:?}",published.iter().zip(state.wram_image().iter()).position(|(a,b)|a!=b));
                count += 1;

            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live ground animation return bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ground_camera_return_bus_publishes_page_deltas_and_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::ground_animation_return::seed_case(&mut image,((case>>5)*17) as usize,(case&31) as usize); state.import_wram_image(&image); }
            state.ram[0x1b]=(case&1) as u8;
            state.ram[0x6c]=((case/2)%3) as u8;
            state.ram[0x21]=case as u8;state.ram[0x23]=(case>>3) as u8;
            state.ram[0x40]=(case as u8).wrapping_add(((case>>4)%3) as u8);
            state.ram[0x41]=state.ram[0x23].wrapping_add([0,1,255][(case as usize/8)%3]);
            state.ram[0x1f3..0x1f7].copy_from_slice(&[0xd7,0x82,0x1b,0x80]);
            let pc=0xe8f0;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f2,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::ground_camera_return::GroundCameraReturn::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu,&mut clock,bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20,0x21,0x22,0x23,0x26,0x2d,0x2e,0x2f,0x30,0x31,0x302,0x303,0x68,0x69] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert!(published==*state.wram_image(),"tile case={case} pc={pc:04x} first_difference={:?}",published.iter().zip(state.wram_image().iter()).position(|(a,b)|a!=b));
                count += 1;

            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live ground camera return bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_player_ancilla_return_bus_preserves_slots_and_live_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::ground_animation_return::seed_case(&mut image,((case>>5)*17) as usize,(case&31) as usize); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::player_ancilla_return::seed_case(&mut image,case as usize,case as usize%32); state.import_wram_image(&image); }
            let pc=0x801c;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f6,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::player_ancilla_return::PlayerAncillaReturn::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu,&mut clock,bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20,0x21,0x22,0x23,0x26,0x2d,0x2e,0x2f,0x30,0x31,0x302,0x303,0x68,0x69] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert!(published==*state.wram_image(),"tile case={case} pc={pc:04x} first_difference={:?}",published.iter().zip(state.wram_image().iter()).position(|(a,b)|a!=b));
                count += 1;

            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live player ancilla return bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_push_caller_return_bus_preserves_records_and_publishes_each_index_byte() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1fb..0x1fd].copy_from_slice(&[0xae,0x87]);
            state.ram[0x42c]=case as u8;
            state.ram[0x42d]=if case & 3 == 0 {case as u8} else {0};
            let length=(case as usize % 17)*2;
            state.ram[0x478..0x47a].copy_from_slice(&(length as u16).to_le_bytes());
            for i in 0..16 {
                let record=[0u16,3,5,0x1111,0xffff,0x8000][(case as usize+i)%6];
                state.ram[0x500+i*2..0x502+i*2].copy_from_slice(&record.to_le_bytes());
            }
            if case & 3 == 1 && length != 0 {
                let record=[1u16,2,4][case as usize%3];
                state.ram[0x500..0x502].copy_from_slice(&record.to_le_bytes());
            }
            let pc=0x894b;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 0,
                bank: 2,
                pc,
                sp: 0x1fa,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let native_handler=state.native_push_loop_has_no_block_effects();
            let original_records=state.ram[0x500..0x600].to_vec();
            let mut work = crate::cycle_models::dungeon_push_caller_return::DungeonPushCallerReturn::with_handler_deferred(!native_handler);
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu,&mut clock,bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published=state.wram_image().into_owned();
                state.game_state.dungeon.write_to_ram(&mut published);
                assert_eq!(&published[0x42c..0x42e],&state.wram_image()[0x42c..0x42e],"index case={case} pc={pc:04x}");
                assert_eq!(&published[0x500..0x600],&state.wram_image()[0x500..0x600],"records case={case} pc={pc:04x}");
                count += 1;

            }
            assert_eq!(&state.wram_image()[0x500..0x600],&original_records);
            use crate::cycle_models::dungeon_push_caller_return::PushCallerTail;
            assert_eq!(work.tail(),Some(if native_handler {PushCallerTail::Returned} else {PushCallerTail::Handler}));
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon push caller return bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_garnish_bus_keeps_flash_counter_slot_index_and_saved_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_garnish::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 9,
                pc: 0xb06e,
                sp: 0x1ed,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(52, 1166),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_garnish::DungeonGarnish::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0xff9,0xffa),(0xfa0,0xfa1),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native garnish case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon garnish bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }    #[test]
    fn live_dungeon_follower_bus_keeps_timer_message_index_and_saved_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_follower::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8365,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(55, 1280),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_follower::DungeonFollower::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0x2cd,0x2cf),(0x1cf0,0x1cf2),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native follower case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon follower bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }    #[test]
    fn live_dungeon_ancilla_bus_keeps_setup_timers_oam_slots_and_saved_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_ancilla::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8369,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(56, 946),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_ancilla::DungeonAncilla::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"native priority case={case} pc={pc:04x}");
                for (start,end) in [(0x47,0x48),(0x314,0x315),(0x377,0x37a),(0xb7b,0xb7c),(0xfb2,0xfb3),(0xfa8,0xfac),(0xfdc,0xfdd),(0xc68,0xc72),(0xc86,0xc90),(0xfa0,0xfa1),(0x90,0x94),(0x800,0xaa0),(0xfe0,0xff8),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon ancilla bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_overlord_bus_keeps_slot_guards_archery_and_real_returns_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..2048usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_overlord::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8398,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(68, 1262),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_overlord::DungeonOverlord::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"native priority case={case} pc={pc:04x}");
                for (start,end) in [(0xb9a,0xb9b),(0xb00,0xb08),(0xffd,0xfff),(0x47,0x48),(0x314,0x315),(0x377,0x37a),(0xb7b,0xb7c),(0xfb2,0xfb3),(0xfa8,0xfac),(0xfdc,0xfdd),(0xc68,0xc72),(0xc86,0xc90),(0xfa0,0xfa1),(0x90,0x94),(0x800,0xaa0),(0xfe0,0xff8),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon overlord bus: 2048 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_lower_garnish_bus_keeps_countdown_kind_allocator_and_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_lower_garnish::seed_case(&mut image, case * 139); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83aa,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(73, 278),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_lower_garnish::DungeonLowerGarnish::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"native priority case={case} pc={pc:04x}");
                for (start,end) in [(0x1f800,0x1f81e),(0x1f90e,0x1f92c),(0xfa0,0xfa1),(0xb9a,0xb9b),(0xb00,0xb08),(0xffd,0xfff),(0x47,0x48),(0x314,0x315),(0x377,0x37a),(0xb7b,0xb7c),(0xfb2,0xfb3),(0xfa8,0xfac),(0xfdc,0xfdd),(0xc68,0xc72),(0xc86,0xc90),(0xfa0,0xfa1),(0x90,0x94),(0x800,0xaa0),(0xfe0,0xff8),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live lower garnish bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_cached_sprites_bus_keeps_scroll_cache_flag_index_and_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_cached_sprites::seed_case(&mut image, case * 7); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83ae,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(76, 54),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_cached_sprites::DungeonCachedSprites::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0x69e,0x6a0),(0xfa0,0xfa1),(0xffa,0xffb),(0x1ec,0x1fd)] {
                    assert_eq!(&published[start..end],&state.wram_image()[start..end],"case={case} pc={pc:04x} range={start:04x}");
                }
                for &base in &crate::game_state::constants::CACHED_SPRITE_LIVE_FIELDS {
                    assert_eq!(&published[base..base+16],&state.wram_image()[base..base+16],"case={case} pc={pc:04x} sprite field={base:05x}");
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live cached sprites bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_sprite_slots_bus_keeps_inactive_markers_current_slot_and_real_returns_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..2048usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_sprite_slots::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 15,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x83a1,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(69, 452),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_sprite_slots::DungeonSpriteSlots::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"native priority case={case} pc={pc:04x}");
                for (start,end) in [(0xbc0,0xbe0),(0xfa0,0xfa1),(0xb9a,0xb9b),(0xb00,0xb08),(0xffd,0xfff),(0x47,0x48),(0x314,0x315),(0x377,0x37a),(0xb7b,0xb7c),(0xfb2,0xfb3),(0xfa8,0xfac),(0xfdc,0xfdd),(0xc68,0xc72),(0xc86,0xc90),(0xfa0,0xfa1),(0x90,0x94),(0x800,0xaa0),(0xfe0,0xff8),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                for base in crate::game_state::constants::CACHED_SPRITE_LIVE_FIELDS.into_iter().chain([0xe00,0xe10,0xee0,0xef0,0xf10]) {
                    assert_eq!(&published[base..base+16],&state.wram_image()[base..base+16],"case={case} pc={pc:04x} field={base:05x}");
                }
                for (start,end) in [(0xfd8,0xfdc),(0x90,0x94),(0xfe0,0xff8)] {
                    assert_eq!(&published[start..end],&state.wram_image()[start..end],"case={case} pc={pc:04x} caller={start:05x}");
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon sprite slots bus: 2048 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ancilla_slot_return_bus_keeps_multiple_receipts_oam_and_real_returns_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            crate::cycle_models::dungeon_ancilla::seed_slot_return_case(&mut state.ram, case);
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8369,
                sp: 0x1f0,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(56, 946),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_ancilla::DungeonAncilla::default().with_native_receipt(true).with_native_oam(true).with_native_slot_return(true);
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"native priority case={case} pc={pc:04x}");
                for (start,end) in [(0,4),(0xbfa,0xc4a),(0xc54,0xc5e),(0x3b1,0x3bb),(0x3a4,0x3ae),(0x39f,0x3a9),(0xbf0,0xbfa),(0x47,0x48),(0x314,0x315),(0x377,0x37a),(0xb7b,0xb7c),(0xfb2,0xfb3),(0xfa8,0xfac),(0xfdc,0xfdd),(0xc68,0xc72),(0xc86,0xc90),(0xfa0,0xfa1),(0x90,0x94),(0x800,0xaa0),(0xfe0,0xff8),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert_eq!(work.tail(), Some(crate::cycle_models::dungeon_ancilla::AncillaTail::Returned));
            assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.sp),(6,0x8398,6,0x1f0));
            assert!(state.rom.is_empty());
        }
        println!(
            "live ancilla slot return bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_receipt_prefix_bus_keeps_modal_timer_follower_and_ancilla_bytes_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::receipt_prefix::seed_case(&mut image, case * 131); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: ((case*131)%10) as u16,
                y: 8,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc38a,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(58, 1334),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::receipt_prefix::ReceiptPrefix::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0x3b1,0x3bb),(0xc68,0xc72),(0xfc1,0xfc2),(0xf3cc,0xf3cd),(0x2cd,0x2cf),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live receipt prefix bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_receipt_oam_bus_keeps_shadow_extended_pointer_scratch_and_return_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::receipt_oam::seed_case(&mut image, case * 5003); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: ((case*5003)%10) as u16,
                y: 8,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc690,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(60, 1252),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::receipt_oam::ReceiptOam::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0,4),(0x90,0x94),(0x800,0xaa0),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live receipt OAM bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_receipt_drawing_bus_keeps_animation_priority_coordinates_and_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::receipt_drawing::seed_case(&mut image, case * 139); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: ((case*139)%10) as u16,
                y: 8,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc5f7,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(59, 1334),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::receipt_drawing::ReceiptDrawing::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0,4),(0x29e,0x2a8),(0x39f,0x3ae),(0xbf0,0xbfa),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                assert_eq!(state.game_state.oam.priority_word(),read_le_u16(&state.wram_image(),0x64),"priority case={case} pc={pc:04x}");
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live receipt drawing bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ancilla_axis_bus_keeps_coordinates_subpixels_velocity_and_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..1024usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::ancilla_axis::seed_case(&mut image, case * 41); state.import_wram_image(&image); }
            state.ram[0x1e9..0x1eb].copy_from_slice(&0x8035u16.to_le_bytes());
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: ((case*41)%10) as u16,
                y: 8,
                d: 0,
                db: 8,
                bank: 8,
                pc: if case%2==0 {0x9080}else{0x908b},
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(58, 1334),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::ancilla_axis::AncillaAxis::new(0x8036);
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0xbfa,0xc4a),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live ancilla_axis bus: 1024 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_receipt_movement_bus_keeps_coordinates_subpixels_velocity_and_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::receipt_movement::seed_case(&mut image, case * 563); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: ((case*563)%10) as u16,
                y: 8,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc5e0,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db=cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(58, 1334),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::receipt_movement::ReceiptMovement::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start,end) in [(0xbfa,0xc4a),(0x1f5,0x1fd)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native ancilla case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live receipt_movement bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_sprite_prefix_bus_keeps_oam_world_drag_and_ancilla_floors_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_sprite_prefix::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x883e,
                sp: 0x1f4,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(51, 832),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_sprite_prefix::DungeonSpritePrefix::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.write_to_ram(&mut published);
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .ancilla_slots
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start, end) in [
                    (0, 4),
                    (0xe, 0x10),
                    (0x90, 0x94),
                    (0x800, 0xaa0),
                    (0xfe0, 0xff8),
                    (0x1f5, 0x1fd),
                    (0xb7c, 0xb80),
                    (0xc7c, 0xc81),
                    (0xfff, 0x1000),
                ] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native sprite prefix case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon sprite prefix bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_push_drawing_bus_keeps_oam_allocator_and_saved_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_push_drawing::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x8838,
                sp: 0x1f4,
                p: 0x10 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(51, 478),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::dungeon_push_drawing::DungeonPushDrawing::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.oam.write_to_ram(&mut published);
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start, end) in [
                    (0, 4), (0xe,0x10),
                    (0x90, 0x94),
                    (0x800, 0xaa0),
                    (0xfe0, 0xff8),
                    (0x1f5, 0x1fd),
                ] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native push drawing case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon push drawing bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_sprite_scroll_bus_keeps_both_scroll_copies_and_saved_stack_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_sprite_scroll::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87e5,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(50, 954),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_sprite_scroll::DungeonSpriteScroll::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.saved().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start, end) in [(0, 4), (0xe0, 0xea), (0x11e, 0x126), (0x72, 0x76)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native sprite scroll case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon sprite scroll bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_lamp_bus_keeps_scroll_words_and_scratch_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_lamp_caller::seed_case(&mut image, case * 13); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87e1,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(50, 794),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::dungeon_lamp_caller::DungeonLampCaller::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.complete() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                for (start, end) in [(0, 4), (0xe0, 0xea), (0x72, 0x76)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native lamp case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon lamp bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_edge_bus_keeps_recoil_position_and_scratch_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_edge_caller::seed_case(&mut image, case); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87d1,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(50, 372),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_edge_caller::DungeonEdgeCaller::default(
                );
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                for (start,end) in [(0,4),(0x20,0x24),(0x27,0x29),(0x2c6,0x2c7),(0x72,0x76)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native edge case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon edge bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_interaction_bus_keeps_timers_target_and_door_flag_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_interaction_caller::seed_case(&mut image, case); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87c9,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(49, 1298),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_interaction_caller::DungeonInteractionCaller::default(
                );
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                for (start, end) in [
                    (0, 4),
                    (8, 10),
                    (0x72, 0x76),
                    (0x333, 0x334),
                    (0x4f0, 0x500),
                    (0x4b8, 0x4ba),
                ] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native interactions case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon interactions bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_dungeon_room_tags_bus_keeps_probe_scratch_flag_and_player_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_room_tags::seed_case(&mut image, case % 64, case / 64); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 6,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87c1,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(48, 896),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::dungeon_room_tags::DungeonRoomTags::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                state.game_state.dungeon.write_to_ram(&mut published);
                for (start, end) in [(0, 4), (0xe, 0x10), (0x72,0x76), (0x20, 0x24), (0x4c7, 0x4c8)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native room tags case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon room tags bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }

    #[test]
    fn live_dungeon_after_push_camera_bus_keeps_native_scroll_and_bounds_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512usize {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, b) in state.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            { let mut image = state.wram_image().into_owned();

            crate::cycle_models::dungeon_after_push::seed_case(&mut image, case * 17); state.import_wram_image(&image); }
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 6,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87b6,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(47, 894),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = crate::cycle_models::dungeon_after_push::DungeonAfterPush::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: true,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .display
                    .ppu_scroll_copy
                    .write_to_ram(&mut published);
                state
                    .game_state
                    .world
                    .camera_boundaries
                    .write_to_ram(&mut published);
                for (start, end) in [(0xe0, 0xe4), (0xe6, 0xea), (0x618, 0x624)] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "native camera case={case} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live dungeon after push camera bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }

    #[test]
    fn live_active_push_blocks_publish_source_tiles_motion_falling_collision_and_camera() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut addresses = std::collections::BTreeSet::<usize>::new();
        for fixture in 0..2304u16 {
            let case = fixture % 256;
            let mode = fixture as usize / 256;
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            crate::cycle_models::dungeon_push_blocks::seed_case(
                &mut state.ram,
                case as usize,
                if mode == 8 { 7 } else { mode.min(6) },
            );
            state.ram[0x1fb..0x1fd].copy_from_slice(&[0xae, 0x87]);
            state.ram[0x1b] = 1;
            state.ram[0x6c] = 0;
            state.ram[0x26] = [0, 1, 2, 4, 8, 12, 15, 255][case as usize % 8];
            state.ram[0xa0..0xa2].copy_from_slice(&(case as u16).to_le_bytes());
            state.ram[0xa6] = (case % 2 * 2) as u8;
            state.ram[0xa7] = (case / 2 % 2 * 2) as u8;
            state.ram[0xa9] = (case % 2) as u8;
            state.ram[0xaa] = (case / 2 % 2 * 2) as u8;
            state.ram[0x40e] = ((case % 8) * 4) as u8;
            if mode == 7 {
                state.ram[0x6c] = 2;
                state.ram[0x26] = 1;
                state.ram[0x67] = 1;
                state.ram[0x20..0x24].copy_from_slice(&[0, 0, 0xeb, 1]);
            }
            let pit_entry = mode == 8;
            if pit_entry {
                state.ram[0x1fc..0x1ff].copy_from_slice(&[0xb5, 0x87, 2]);
            }
            let pc = if pit_entry { 0xd7fd } else { 0x894b };
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 0,
                d: 0,
                db: 0,
                bank: if pit_entry { 1 } else { 2 },
                pc,
                sp: if pit_entry { 0x1fb } else { 0x1fa },
                p: if pit_entry { 0 } else { 0x30 },
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work =
                crate::cycle_models::dungeon_push_caller_return::DungeonPushCallerReturn::default();
            let mut pit =
                crate::cycle_models::dungeon_push_blocks::DungeonPushBlocks::pit_check_entry();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while if pit_entry {
                !pit.complete()
            } else {
                work.tail().is_none()
            } {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    if let Access::Write(address, _, width) = access {
                        let offset = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                            (address & 65535) as usize
                        } else {
                            (address - 0x7e0000) as usize
                        };
                        for byte in 0..usize::from(width) {
                            addresses.insert(offset + byte);
                        }
                    }

                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: true,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .access(at, access)
                };
                if pit_entry {
                    pit.step(&mut cpu, &mut clock, bus).unwrap();
                    pit = pit.clone();
                } else {
                    work.step(&mut cpu, &mut clock, bus).unwrap();
                    work = work.clone();
                }
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                assert_eq!(
                    state.game_state.display.bg_vram_load_mode, state.ram[0x14],
                    "VRAM request fixture={fixture} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state.game_state.dungeon.write_to_ram(&mut published);
                for (start, end) in [
                    (0xa8, 0xa9),
                    (0x408, 0x40a),
                    (0x42c, 0x42e),
                    (0x466, 0x468),
                    (0x500, 0x5e0),
                    (0x5fc, 0x5fe),
                    (0x641, 0x643),
                    (0x2000, 0x6000),
                    (0x12000, 0x14000),
                    (0xf940, 0xfacc),
                ] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "dungeon fixture={fixture} pc={pc:04x} range={start:05x}..{end:05x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .pushed_block
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x5e0..0x5fc],
                    &state.wram_image()[0x5e0..0x5fc],
                    "block banks fixture={fixture} pc={pc:04x}"
                );
                assert_eq!(
                    &published[0x2c3..0x2c5],
                    &state.wram_image()[0x2c3..0x2c5],
                    "block animation fixture={fixture} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .follower_link
                    .write_to_ram(&mut published);
                for (start, end) in [
                    (0x20, 0x24),
                    (0x27, 0x29),
                    (0x30, 0x32),
                    (0x40, 0x42),
                    (0x48, 0x51),
                    (0x67, 0x6a),
                    (0xa9, 0xab),
                    (0x308, 0x30c),
                    (0x376, 0x377),
                ] {
                    assert_eq!(
                        &published[start..end],
                        &state.wram_image()[start..end],
                        "player fixture={fixture} pc={pc:04x} range={start:04x}..{end:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .world
                    .room_bounds
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x600..0x610],
                    &state.wram_image()[0x600..0x610],
                    "bounds fixture={fixture} pc={pc:04x}"
                );
                state
                    .game_state
                    .sprites
                    .sprite_slots
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0xea0..0xeb0],
                    &state.wram_image()[0xea0..0xeb0],
                    "sprite recoil timer fixture={fixture} pc={pc:04x}"
                );
                assert_eq!(
                    &published[0xf30..0xf50],
                    &state.wram_image()[0xf30..0xf50],
                    "sprite recoil fixture={fixture} pc={pc:04x}"
                );
                count += 1;
            }
            use crate::cycle_models::dungeon_push_caller_return::PushCallerTail;
            if pit_entry {
                assert!(pit.complete());
            } else {
                assert!(matches!(
                    work.tail(),
                    Some(PushCallerTail::Returned | PushCallerTail::RoomTransition(_))
                ));
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live active push-block bus: 2304 cases, {count} source instructions, typed stores coherent"
        );
        let mut ranges = Vec::<(usize, usize)>::new();
        for address in addresses {
            if let Some((_, end)) = ranges.last_mut() {
                if address == *end + 1 {
                    *end = address;
                    continue;
                }
            }
            ranges.push((address, address));
        }
        let evidence = format!(
            "2304 active-block fixtures, {count} source instructions, physical store ranges {ranges:x?}\n"
        );
        std::fs::write(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/rea-native-active-push-block-store-ranges-v1.txt"),
            &evidence,
        )
        .unwrap();
        println!(
            "active push-block store domains: 2304 cases, {count} source instructions, {} physical store ranges",
            ranges.len()
        );
    }

    #[test]
    fn live_vertical_response_bus_keeps_typed_collision_resources_and_doors_coherent() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1ef..0x1f1].copy_from_slice(&[0x5e, 0xb9]);
            state.ram[0x1b] = u8::from(((case >> 4) as u8) & 8 == 0);
            state.ram[0x308] = if ((case >> 4) as u8) & 4 == 0 { 0 } else { 128 };
            state.ram[0x46] = ((case >> 4) as u8) & 2;
            state.ram[0x6c] = (case as u8) & 3;
            state.ram[0x6a] = ((case as u8) >> 2) & 3;
            state.ram[0x46c] = ((case as u8) >> 4) % 5;
            state.ram[0xee] = ((case >> 4) as u8) & 1;
            state.ram[0x315] = (((case >> 4) as u8) >> 1) & 3;
            state.ram[0x0e] = case as u8;
            state.ram[0x0f] = (case as u8).rotate_left(3);
            state.ram[0x0c] = if ((case >> 4) as u8) & 16 == 0 { 0 } else { 5 };
            state.ram[0x62] = ((case >> 4) as u8) & 1;
            state.ram[0x2e7] = if (case as u8) & 128 == 0 { 0 } else { 32 };
            state.ram[0x2f7] = if ((case >> 4) as u8) & 2 == 0 {
                0
            } else {
                0x22
            };
            state.ram[0x34c] = (case >> 4) as u8;
            state.ram[0x2e8] = ((case >> 4) as u8) & 7;
            state.ram[0x320] = (case >> 4) as u8;
            state.ram[0x3f1] = (case as u8) & 0x22;
            state.ram[0x3f2] = (case as u8).rotate_left(2) & 0x22;
            state.ram[0x66] = ((case as u8) >> 6) & 1;
            let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
            for (n, address) in [0x20, 0x22, 0xf360].into_iter().enumerate() {
                state.write_wram_word(address, values[(((case as u8) as usize) + n) % 8]);
            }
            let pc = 0xba2e;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1ee,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = VerticalCollisionResponse::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu, &mut clock, bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x20, 0x21, 0x22, 0x23, 0x49, 0x4a, 0x50, 0x6b, 0x6c, 0x3f3] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert_eq!(published, *state.wram_image(), "tile case={case} pc={pc:04x}");
                count += 1;
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                assert_eq!(
                    published[0xef], state.ram[0xef],
                    "world case={case} pc={pc:04x}"
                );
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live vertical response bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_vertical_surface_bus_publishes_pit_timer_damage_frame_and_snap_stores() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0x1ef..0x1f1].copy_from_slice(&[0x5e, 0xb9]);
            state.ram[0x36d] = if ((case >> 4) as u8) & 3 == 1 {
                7
            } else {
                (case as u8) & 6
            };
            state.ram[0x341] = if ((case >> 4) as u8) & 3 == 2 { 7 } else { 0 };
            state.ram[0x343] = if ((case >> 4) as u8) & 3 == 3 { 2 } else { 0 };
            state.ram[0x345] = (case as u8) & 1;
            state.ram[0x4d] = ((case as u8) >> 1) & 1;
            state.ram[0x58] = if ((case >> 4) as u8) & 4 != 0 && (case as u8) & 7 == 0 {
                7
            } else {
                0
            };
            state.ram[0x2c0] = (case as u8) & 0x77;
            state.ram[0x46] = ((case as u8) >> 3) & 1;
            state.ram[0x5e] = (case as u8) & 3;
            state.ram[0x57] = ((case as u8) >> 2) & 3;
            state.ram[0x372] = u8::from(((case >> 4) as u8) & 16 != 0);
            state.ram[0x59] = ((case as u8) >> 4) & 5;
            state.ram[0x5d] = (case as u8) & 7;
            state.ram[0x2e8] = ((case as u8) >> 5) & 7;
            state.ram[0x31f] = (case as u8) & 1;
            state.ram[0x55] = ((case >> 4) as u8) & 2;
            state.write_wram_byte(0xf35b, ((case as u8) >> 6) % 3);
            state.ram[0x46c] = (((case >> 4) as u8) >> 1) % 5;
            state.ram[0xee] = ((case >> 4) as u8) & 1;
            state.ram[0x5f] = if ((case >> 4) as u8) & 8 != 0 {
                (case as u8) | 1
            } else {
                0
            };
            state.ram[0x60] = if ((case >> 4) as u8) & 8 != 0 { 128 } else { 0 };
            state.ram[0x6a] = ((case >> 4) as u8) & 3;
            state.ram[0x310] = (case as u8) & 1;
            state.ram[0x0e] = case as u8;
            state.ram[0x30] = [0, 1, 127, 128, 255][((case as u8) as usize) % 5];
            state.ram[0x66] = ((case >> 4) as u8) & 1;
            state.ram[0x2f] = if (case as u8) & 2 == 0 {
                8
            } else {
                (((case >> 4) as u8) & 1) * 2
            };
            state.ram[0x315] = ((case as u8) >> 6) & 3;
            state.ram[0x3c] = (case as u8) & 7;
            state.ram[0x371] = case as u8;
            state.ram[0x2f6] = (case as u8) & 0x20;
            state.ram[0x2f1] = (case as u8) & 0x40;
            state.ram[0x2ef] = (case as u8) & 0x70;
            let values = [0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa];
            for (n, address) in [0x20, 0x22, 0x51].into_iter().enumerate() {
                state.write_wram_word(address, values[(((case as u8) as usize) + n) % 8]);
            }
            let pc = 0xbbe6;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1ee,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = VerticalSurfaceResponse::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                let bus = |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                };
                work.step(&mut cpu, &mut clock, bus).unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x20, 0x21, 0x22, 0x23, 0x49, 0x4a, 0x48, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x57,
                    0x61, 0x302, 0x371, 0x373,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                assert_eq!(published, *state.wram_image(), "tile case={case} pc={pc:04x}");
                let mut published = state.wram_image().into_owned();
                state.game_state.frame.encode_into_image(&mut published);
                assert_eq!(
                    &published[0x10..0x12],
                    &state.wram_image()[0x10..0x12],
                    "frame case={case} pc={pc:04x}"
                );
                count += 1;
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .world
                    .transient
                    .write_to_ram(&mut published);
                assert_eq!(
                    published[0xef], state.ram[0xef],
                    "world case={case} pc={pc:04x}"
                );
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live vertical surface bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_ground_velocity_bus_publishes_axis_bytes_safe_position_and_conveyor() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            state.ram[0xb7b] = 0;
            state.write_wram_byte(0x11, 0);
            state.ram[0x5d] = 0;
            state.ram[0x34a] = 0;
            state.ram[0x372] = 0;
            state.ram[0x316] = 0;
            state.ram[0x317] = 0;
            state.ram[0x5e] = ((case / 16) % 15) as u8 * 2;
            state.ram[0x351] = u8::from(case & 16 != 0);
            state.ram[0x57] = [0, 1, 2, 15, 16, 32, 48, 255][((case >> 3) & 7) as usize];
            state.ram[0x5b] = [0, 1, 2, 3][((case >> 1) & 3) as usize];
            state.ram[0x67] = (case & 15) as u8;
            state.ram[0x4d] = u8::from(case & 128 != 0);
            state.ram[0x2f5] = if case & 64 != 0 { 2 } else { 0 };
            state.ram[0x46c] = [0, 1, 2, 4][((case >> 5) & 3) as usize];
            state.ram[0x322] = 3;
            state.ram[0x376] = 0;
            state.ram[0x3f3] = (case % 5) as u8;
            for (i, address) in [0x20, 0x22, 0x310, 0x312].into_iter().enumerate() {
                state.ram[address..address + 2].copy_from_slice(
                    &[0u16, 0xffff, 0x7fff, 0x8000, 0xff, 0xff00, 0x5555, 0xaaaa]
                        [((case as usize >> 2) + i) % 8]
                        .to_le_bytes(),
                );
            }
            match case {
                0 => state.ram[0xb7b] = 1,
                1 => state.ram[0x5d] = 4,
                2 => {
                    state.ram[0x372] = 1;
                    state.ram[0x2f1] = 15;
                    state.ram[0xf0] = 8;
                }
                3 => {
                    state.ram[0x316] = 7;
                    state.ram[0x317] = 8;
                }
                _ => {}
            }
            let pc = 0x82be;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: 8,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1f4,
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = GroundVelocity::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: false,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [
                    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x3e, 0x3f,
                    0x40, 0x41, 0x57, 0x5e, 0x67, 0x68, 0x69, 0x6a,
                ] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .swim_acceleration
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x32b..0x340],
                    &state.wram_image()[0x32b..0x340],
                    "swim case={case} pc={pc:04x}"
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .bg1_movement_accumulator
                    .write_to_ram(&mut published);
                assert_eq!(
                    &published[0x41c..0x41e],
                    &state.wram_image()[0x41c..0x41e],
                    "conveyor case={case} pc={pc:04x}"
                );
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live ground velocity bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_vertical_slope_probe_bus_publishes_reset_neighbors_and_coordinates() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..16u8 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            let x = [0xffffu16, 0xfff8, 0x7fff, 0x1ff][(case >> 2) as usize];
            let y = x.rotate_left(case as u32);
            state.ram[0x20..0x22].copy_from_slice(&y.to_le_bytes());
            state.ram[0x22..0x24].copy_from_slice(&x.to_le_bytes());
            state.ram[0xec..0xee]
                .copy_from_slice(&[0xffffu16, 0x1ff, 0x1f8, 0][(case >> 2) as usize].to_le_bytes());
            let pc = 0xce85;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: u16::from(case & 1) * 2,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: if case & 2 == 0 { 0x1f0 } else { 0x1ee },
                p: 0x30,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = VerticalSlopeProbe::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while !work.child_ready() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                assert_eq!(published[0x5a], state.ram[0x5a]);
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                for address in [
                    0xc, 0xe, 0x38, 0x58, 0x59, 0x5f, 0x62, 0x2c0, 0x320, 0x341, 0x343, 0x348,
                    0x34c, 0x357, 0x359, 0x35b, 0x366, 0x36d, 0x36f, 0x2e5, 0x2e7, 0x2ee, 0x2f6,
                ] {
                    assert_eq!(
                        &published[address..address + 2],
                        &state.wram_image()[address..address + 2],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                assert_eq!(
                    state
                        .game_state
                        .dungeon
                        .environment
                        .moving_floor_check_flags(),
                    read_le_u16(&state.wram_image(), 0x3f1)
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..0x10], &state.wram_image()[0..0x10]);
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live vertical slope bus: 16 cases, {count} source instructions, typed stores coherent"
        );
    }
    #[test]
    fn live_tile_classifier_bus_publishes_lookup_neighbors_and_common_effects() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        for case in 0..512u16 {
            let mut state = ZeldaState::new();
            state.rom.clear();
            for (i, byte) in state.ram.iter_mut().enumerate() {
                *byte = (i ^ (i >> 7)) as u8;
            }
    { let image = state.ram.clone(); state.import_wram_image(&image); }
            let tile = (case / 2) as u8;
            let x = 16u16;
            let y = 24u16;
            let offset = 192 + 16 + if case & 1 != 0 { 0x1000 } else { 0 };
            state.ram[0..2].copy_from_slice(&y.to_le_bytes());
            state.ram[2..4].copy_from_slice(&x.to_le_bytes());
            state.ram[0x1b] = if case == 511 { 0 } else { 1 };
            state.ram[0xee] = (case & 1) as u8;
            state.ram[0x37f] = 0;
            state.ram[0x12000 + offset] = tile;
            state.ram[0x12000 + offset + 64] = if case & 1 != 0 { tile } else { 0 };
            state.ram[0x2f5..0x2f7].fill(0);
            state.ram[0xffc..0xffe].fill(0);
            state.ram[0x403..0x405].fill(0);
            state.ram[0xa..0xc]
                .copy_from_slice(&if case & 1 != 0 { 0xffffu16 } else { 1 }.to_le_bytes());
            state.ram[0x1ef..0x1f1].copy_from_slice(&[0xb9, 0xce]);
            let pc = 0xd9d8;
            state.sync_native_game_state_from_ram();
            let mut cpu = NativeNmiContext {
                a: 0x2400,
                x: 16,
                y: u16::from(case & 1) * 2,
                d: 0,
                db: 7,
                bank: 7,
                pc,
                sp: 0x1ee,
                p: 0,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.ram.copy_from_slice(&state.wram_image());
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut clock = CpuMasterTimeline::at_raster(
                4586,
                CpuRasterPosition::new(22, 26),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut work = TileClassifier::default();
            let mut hdma = HdmaTimingState::new(&state.dma);
            let mut deadline = None;
            while work.tail().is_none() {
                let pc = cpu.pc;
                source.step().unwrap();
                work.step(&mut cpu, &mut clock, |at, access| {
                    ItemBus {
                        state: &mut state,
                        hdma: &mut hdma,
                        nmi_deadline: &mut deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: true,
                        publish_probe: true,
                    }
                    .access(at, access)
                })
                .unwrap();
                work = work.clone();
                let c = &source.snes().cpu;
                assert_eq!(
                    cpu,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "case={case} pc={pc:04x}"
                );
                assert_eq!(clock, *source.timeline(), "case={case} pc={pc:04x}");
                assert_eq!(*state.wram_image(), source.snes().ram, "case={case} pc={pc:04x}");
                let link = &state.game_state.player.follower_link;
                let mut published = state.wram_image().into_owned();
                link.write_to_ram(&mut published);
                for address in [0x49, 0x4a, 0x114, 0x5a] {
                    assert_eq!(
                        published[address], state.ram[address],
                        "link case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .player
                    .tile_detection
                    .write_to_ram(&mut published);
                for address in [
                    0x6e, 0x76, 0xc, 0xe, 0x38, 0x58, 0x59, 0x5f, 0x62, 0x2c0, 0x320, 0x341, 0x343,
                    0x348, 0x34c, 0x357, 0x359, 0x35b, 0x366, 0x36d, 0x36f, 0x2e5, 0x2e7, 0x2ee,
                    0x2f6,
                ] {
                    assert_eq!(
                        &published[address..address + 2],
                        &state.wram_image()[address..address + 2],
                        "typed case={case} pc={pc:04x} address={address:04x}"
                    );
                }
                assert_eq!(
                    state
                        .game_state
                        .dungeon
                        .environment
                        .moving_floor_check_flags(),
                    read_le_u16(&state.wram_image(), 0x3f1)
                );
                let mut published = state.wram_image().into_owned();
                state
                    .game_state
                    .sprites
                    .workspace
                    .write_to_ram(&mut published);
                assert_eq!(&published[0..4], &state.wram_image()[0..4]);
                count += 1;
            }
            assert!(state.rom.is_empty());
        }
        println!(
            "live tile classifier bus: 512 cases, {count} source instructions, typed stores coherent"
        );
    }
}

#[derive(Clone, Debug)]
pub(super) struct NativeItemGraphicsOwner {
    entry: NativeItemGraphicsEntry,
    work: AnimatedItemGraphics,
    receipt_work: Option<ItemReceiptReturnWork>,
    link_work: Option<LinkReceiveItemReturn>,
    apress_work: Option<GroundItemApressReturn>,
    y_work: Option<GroundYItemPrefix>,
    sword_work: Option<GroundSwordCapeReturn>,
    movement_work: Option<GroundMovementPrefix>,
    velocity_work: Option<GroundVelocity>,
    cardinal_work: Option<CardinalCollisionPrefix>,
    vertical_cardinal_work: Option<VerticalCardinalProbe>,
    vertical_response_work: Option<VerticalCollisionResponse>,
    vertical_surface_work: Option<VerticalSurfaceResponse>,
    ground_animation_work: Option<GroundAnimationReturn>,
    dungeon_return_work: Option<DungeonMainReturn>,
    main_response_work: Option<MainTileResponse>,
    main_tile_work: Option<MainTileProbe>,
    main_tile_classifier: Option<TileClassifier>,
    cardinal_classifiers: [Option<TileClassifier>; 3],
    diagonal_work: Option<DiagonalCollisionPrefix>,
    probe_work: Option<VerticalSlopeProbe>,
    classifier_work: Option<TileClassifier>,
    high_classifier_work: Option<TileClassifier>,
    hdma: HdmaTimingState,
    nmi_deadline: Option<u64>,
    gfx: u8,
    receipt: ItemReceiptReturn,
}

impl NativeItemGraphicsOwner {
    fn return_tail_ready(&self) -> bool {
        if let Some(work) = &self.dungeon_return_work { return work.complete(); }
        if let Some(work)=&self.ground_animation_work {return work.complete();}
        if let Some(work)=&self.main_response_work {return work.tail().is_some();}
        if let Some(work)=&self.main_tile_work {
            return work.complete() || (work.child_ready() && self.main_tile_classifier.as_ref().is_some_and(|c| c.tail().is_some()));
        }
        if let Some(work) = &self.vertical_surface_work {
            return work.tail().is_some();
        }
        if let Some(work) = &self.vertical_response_work {
            return work.tail().is_some();
        }
        if let Some(probe) = &self.vertical_cardinal_work {
            if probe.tail() == Some(VerticalCardinalTail::Response) {
                return true;
            }
            return probe.child_ready()
                && self.cardinal_classifiers[probe.sample()]
                    .as_ref()
                    .is_some_and(|c| {
                        matches!(
                            c.tail(),
                            Some(
                                TileClassifierTail::Effect { .. }
                                    | TileClassifierTail::OutdoorAttribute
                            )
                        )
                    });
        }

        if let Some(work) = &self.cardinal_work {
            return work.tail().is_some();
        }
        if let Some(work) = &self.velocity_work {
            return work.tail().is_some();
        }
        if let Some(work) = &self.probe_work {
            if work.complete() {
                return true;
            }
            let classifier = self
                .high_classifier_work
                .as_ref()
                .or(self.classifier_work.as_ref());
            return classifier.is_some_and(|work| matches!(work.tail(), Some(crate::cycle_models::tile_classifier::TileClassifierTail::Effect { .. } | crate::cycle_models::tile_classifier::TileClassifierTail::OutdoorAttribute)));
        }
        if let Some(work) = &self.diagonal_work {
            return work.tail().is_some();
        }
        if let Some(work) = &self.movement_work {
            return work.tail().is_some();
        }
        match &self.sword_work {
            Some(work) => work.tail().is_some(),
            None => self
                .y_work
                .as_ref()
                .is_some_and(|work| work.tail().is_some()),
        }
    }
}

struct ItemBus<'a> {
    state: &'a mut ZeldaState,
    hdma: &'a mut HdmaTimingState,
    nmi_deadline: &'a mut Option<u64>,
    publish_hud: bool,
    publish_push: bool,
    publish_dungeon_camera: bool,
    publish_link: bool,
    publish_probe: bool,
}

pub(super) struct DungeonReturnBus<'a> {
    inner: ItemBus<'a>,
}

impl<'a> DungeonReturnBus<'a> {
    pub(super) fn new(state: &'a mut ZeldaState, hdma: &'a mut HdmaTimingState, nmi_deadline: &'a mut Option<u64>) -> Self {
        Self { inner: ItemBus { state, hdma, nmi_deadline, publish_push: false,
            publish_dungeon_camera: false, publish_hud: false, publish_link: false, publish_probe: false } }
    }
}

impl WorkBus for DungeonReturnBus<'_> {
    fn inactive_sprite_body_mask(&mut self) -> u16 { self.inner.inactive_sprite_body_mask() }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        let result = self.inner.access(at, access);
        if let Access::Write(address, value, width) = access {
            let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 { address & 65535 } else { address };
            self.inner.state.native_dungeon_main_return_store(address, value, width);
        }
        result
    }
    fn retire_event(&mut self, event: CpuSynchronousTimelineEvent, clock: &CpuMasterTimeline) -> u32 {
        self.inner.retire_event(event, clock)
    }
}

impl ItemBus<'_> {
    fn read(&self, address: u32) -> u8 {
        let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
            address & 65535
        } else {
            address
        };
        if address < 0x2000 {
            return self.state.read_wram_byte(address as usize);
        }
        if (0x7e0000..0x800000).contains(&address) {
            return self.state.read_wram_byte(address as usize - 0x7e0000);
        }
        if let Some(value) = crate::cycle_models::graphics_sources::table_byte(address)
            .or_else(|| load_gfx::native_animated_item_table_byte(address))
            .or_else(|| misc::native_item_receipt_table_byte(address))
            .or_else(|| ZeldaState::native_load_hud_table_byte(address))
            .or_else(|| crate::cycle_models::ground_item_apress_return::table_byte(address))
            .or_else(|| crate::cycle_models::y_item_dispatch::table_byte(address))
            .or_else(|| crate::cycle_models::ground_sword_cape_return::table_byte(address))
            .or_else(|| crate::cycle_models::vertical_slope_probe::table_byte(address))
            .or_else(|| crate::cycle_models::tile_classifier::table_byte(address))
            .or_else(|| crate::cycle_models::diagonal_collision_prefix::table_byte(address))
            .or_else(|| crate::cycle_models::ground_velocity::table_byte(address))
            .or_else(|| crate::cycle_models::vertical_cardinal_probe::table_byte(address))
            .or_else(|| crate::cycle_models::vertical_collision_response::table_byte(address))
            .or_else(|| crate::cycle_models::vertical_surface_response::table_byte(address))
            .or_else(|| crate::cycle_models::main_tile_probe::table_byte(address))
            .or_else(|| crate::cycle_models::main_tile_response::table_byte(address))
            .or_else(|| crate::cycle_models::player_animation::table_byte(address))
            .or_else(|| crate::cycle_models::dungeon_push_blocks::table_byte(address))
            .or_else(|| crate::cycle_models::dungeon_room_tags::table_byte(address))
            .or_else(|| crate::cycle_models::dungeon_interactions::table_byte(address))
            .or_else(|| crate::cycle_models::lamp_cone::table_byte(address))
            .or_else(|| crate::cycle_models::push_block_drawing::table_byte(address))
            .or_else(|| crate::cycle_models::sprite_main::table_byte(address))
            .or_else(|| crate::cycle_models::dungeon_main_return::table_byte(address))
            .or_else(|| ZeldaState::native_follower_table_byte(address))
            .or_else(|| ZeldaState::native_receipt_animation_table_byte(address))
        {
            return value;
        }
        let offset = |a: u32| ((a >> 16) & 127) * 0x8000 + (a & 0x7fff);
        for &(sprite, sheet, start) in crate::cycle_models::graphics_sources::SOURCES {
            if !sprite {
                continue;
            }
            let Some(at) = offset(address).checked_sub(offset(start)) else {
                continue;
            };
            let Some(data) = self.state.asset_bytes(64, sheet) else {
                continue;
            };
            if let Some(&byte) = data.get(at as usize) {
                return byte;
            }
        }
        panic!("native item graphics read outside its assets: {address:06x}");
    }
}

impl WorkBus for ItemBus<'_> {
    fn inactive_sprite_body_mask(&mut self) -> u16 {
        self.state.native_inactive_sprite_body_mask()
    }
    fn access(&mut self, _at: u64, access: Access) -> u16 {
        match access {
            Access::Read(address, width) => {
                if width == 1 {
                    u16::from(self.read(address))
                } else {
                    u16::from_le_bytes([self.read(address), self.read(address + 1)])
                }
            }
            Access::Write(address, value, width) => {
                let address = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                    address & 65535
                } else {
                    address
                };
                let offset = if address < 0x2000 {
                    address as usize
                } else if (0x7e0000..0x800000).contains(&address) {
                    address as usize - 0x7e0000
                } else {
                    panic!("native item graphics write outside WRAM: {address:06x}")
                };
                if self.publish_hud {
                    self.state.native_load_hud_store(address, value, width);
                    if address == 0x200 {
                        assert_eq!(width, 1);
                        self.state.set_overworld_map_state(value as u8);
                    }
                }
                if self.publish_link {
                    match offset {
                        0x42c | 0x42d => {
                            for (byte,v) in value.to_le_bytes()[..usize::from(width)].iter().enumerate() {
                                assert!(self.state.game_state.dungeon.write_room_parent_byte(offset+byte,*v));
                            }
                        }
                        0x2cb|0x32a|0x34f|0x340|0x345|0x34a|0x351|0x2db|0x2ed|0x56|0x2e0|0x3f5..=0x3f7 if width==1=>self.state.follower_link_state_mut().write_main_tile_response_store(offset,value as u8),
                        0x10 if width == 1 => self.state.set_main_module(value as u8),
                        0x11 if width == 1 => self.state.set_submodule(value as u8),
                        0x5a if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_pit_data_index(value as u8),
                        0x5b if width == 1 => {
                            assert_eq!(value, 1);
                            self.state.follower_link_state_mut().begin_pit_check();
                        }
                        0x5c if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_sprite_oam_state_timer(value as u8),
                        0x61 if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_gravestone_push_timeout(value as u8),
                        0x373 if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_given_damage(value as u8),
                        0x49 if width == 1 => {
                            let word =
                                (read_le_u16(&self.state.ram, 0x49) & 0xff00) | (value & 255);
                            self.state
                                .follower_link_state_mut()
                                .set_force_move_any_direction(word);
                        }
                        0x6c if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_doorway_state(value as u8),
                        0x3f3 if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_conveyor_belt_state(value as u8),
                        0xef if width == 1 => self.state.set_room_transitioning_flags(value as u8),
                        0xf360 if width == 2 => {
                            self.state.player_resources_mut().set_rupees_goal(value)
                        }
                        0x38 | 0x6e | 0x316 if width == 1 => self
                            .state
                            .tile_detect_position_mut()
                            .write_probe_setup_byte(offset, value as u8),
                        0x6d | 0x315 | 0x2dc..=0x2df if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .write_cardinal_collision_store(offset, value as u8),

                        0..=3 if width == 1 => self
                            .state
                            .game_state
                            .sprites
                            .workspace
                            .write_zero_page_byte(offset, value as u8),
                        0x20..=0x25 | 0x27..=0x29 | 0x3e..=0x41 | 0x57 | 0x68 | 0x69
                            if width == 1 =>
                        {
                            self.state
                                .follower_link_state_mut()
                                .write_ground_velocity_store(offset, value as u8)
                        }
                        0x41c | 0x41d => {
                            assert_eq!(width, 1);
                            if offset == 0x41c {
                                self.state.bg1_move_calc_mut().set_y_subpixel(value as u8);
                            } else {
                                self.state.bg1_move_calc_mut().set_x_subpixel(value as u8);
                            }
                        }
                        0x32b..=0x332 | 0x33c..=0x33f if width == 1 => {
                            let start = if offset < 0x32f {
                                0x32b
                            } else if offset < 0x33c {
                                0x32f
                            } else {
                                0x33c
                            };
                            let base = start + ((offset - start) & !1);
                            let old = read_le_u16(&self.state.ram, base);
                            let word = if offset == base {
                                (old & 0xff00) | value
                            } else {
                                (old & 255) | (value << 8)
                            };
                            assert!(self
                                .state
                                .native_load_player_initialization_store(base, word, 2));
                        }
                        0x302 => {
                            assert_eq!(width,1);assert!(value<=1);
                            if value==0 {self.state.follower_link_state_mut().clear_pit_correction();}
                            else {self.state.follower_link_state_mut().set_pit_correction_active();}
                        }
                        0 | 2 => {
                            assert!(self.state.native_opening_player_store(
                                offset as u32,
                                value,
                                width
                            ));
                        }
                        0x20 => {
                            assert_eq!(width, 2);
                            self.state.follower_link_state_mut().set_y(value);
                        }
                        0x22 => {
                            assert_eq!(width, 2);
                            self.state.follower_link_state_mut().set_x(value);
                        }
                        0x42 | 0x43 | 0x66 | 0x6a | 0xee | 0x318 | 0x31a => self
                            .state
                            .follower_link_state_mut()
                            .write_diagonal_collision_store(offset, value, width),
                        0x26
                        | 0x3b
                        | 0x48
                        | 0x67
                        | 0x308..=0x30a
                        | 0x30d..=0x30e
                        | 0x326
                        | 0x328
                        | 0x32d
                        | 0x32f
                        | 0x331
                        | 0x33c
                        | 0x33e
                        | 0x376 => {
                            assert!(self
                                .state
                                .native_load_player_initialization_store(offset, value, width));
                        }
                        0x32b if width == 2 => {
                            assert!(self
                                .state
                                .native_load_player_initialization_store(offset, value, width));
                        }
                        0x334..=0x337 => {
                            if width == 2 {
                                assert!(self
                                    .state
                                    .native_load_player_initialization_store(offset, value, width));
                            } else {
                                let base = offset & !1;
                                let old = read_le_u16(&self.state.ram, base);
                                let word = if offset & 1 == 0 {
                                    (old & 0xff00) | value
                                } else {
                                    (old & 255) | (value << 8)
                                };
                                self.state
                                    .swim_acceleration_mut()
                                    .set_max_speed(base - 0x334, word);
                            }
                        }
                        0x2a | 0x2b | 0x6b | 0x371 | 0x375 => {
                            assert_eq!(width, 1);
                            self.state
                                .follower_link_state_mut()
                                .write_ground_movement_store(offset, value as u8);
                        }
                        0x2d..=0x2f => {
                            assert!(self.state.native_player_drawing_store(
                                offset as u32,
                                value,
                                width
                            ));
                        }
                        _ => {}
                    }
                }
                if self.publish_link && width == 1 {
                    match address {
                        0x5d => self
                            .state
                            .follower_link_state_mut()
                            .set_handler_state(value as u8),
                        0x31f => self
                            .state
                            .follower_link_state_mut()
                            .set_blink_countdown(value as u8),
                        0x2e3 => self
                            .state
                            .follower_link_state_mut()
                            .set_sword_delay_timer(value as u8),
                        0x4c => self
                            .state
                            .follower_link_state_mut()
                            .set_cape_decrement_counter(value as u8),
                        0x30 => self
                            .state
                            .follower_link_state_mut()
                            .set_y_velocity(value as u8),
                        0x31 => self
                            .state
                            .follower_link_state_mut()
                            .set_x_velocity(value as u8),
                        0x7ef36e => self.state.player_magic_mut().set_magic_power(value as u8),
                        0x304 => self
                            .state
                            .follower_link_state_mut()
                            .set_current_item_active(value as u8),
                        0x307 => self
                            .state
                            .follower_link_state_mut()
                            .set_selected_rod(value as u8),
                        0x3a => self
                            .state
                            .follower_link_state_mut()
                            .set_button_mask_b_y(value as u8),
                        0x3c => self
                            .state
                            .follower_link_state_mut()
                            .set_button_b_frames(value as u8),
                        0x301 => {
                            assert_eq!(value, 0);
                            self.state.follower_link_state_mut().clear_item_in_hand();
                        }
                        0x306 => self
                            .state
                            .follower_link_state_mut()
                            .set_cached_tile_action_index(value as u8),
                        0x374 => self
                            .state
                            .follower_link_state_mut()
                            .set_dash_countdown(value as u8),
                        0x5e => self
                            .state
                            .follower_link_state_mut()
                            .set_speed_setting(value as u8),
                        0x372 => self
                            .state
                            .follower_link_state_mut()
                            .set_running_state(value as u8),
                        0x50 => self
                            .state
                            .follower_link_state_mut()
                            .write_initialization_store(0x50, value, 1),
                        0x32b => {
                            assert_eq!(value, 0);
                            // M8 STZ preserves the neighboring high mode byte.
                            let mode = read_le_u16(&self.state.ram, 0x32b) & 0xff00;
                            self.state.swim_acceleration_mut().set_mode(0, mode);
                        }
                        _ => {}
                    }
                }
                for (byte, value) in value.to_le_bytes()[..usize::from(width)].iter().enumerate() {
                    self.state.write_wram_byte(offset + byte, *value);
                }
                if self.publish_push {self.state.native_push_block_store(offset,value,width);}
                if self.publish_dungeon_camera {self.state.native_dungeon_camera_store(offset,value,width);}
                if self.publish_probe && width == 1 && matches!(offset, 0x0e | 0x59 | 0x322 | 0x2c2) {
                    self.state
                        .tile_detect_position_mut()
                        .write_probe_setup_byte(offset, value as u8);
                }
                if self.publish_probe && width == 2 {
                    for (i, byte) in value.to_le_bytes().into_iter().enumerate() {
                        let at = offset + i;
                        self.state
                            .tile_detect_position_mut()
                            .write_probe_setup_byte(at, byte);
                        if at < 4 {
                            self.state
                                .game_state
                                .sprites
                                .workspace
                                .write_zero_page_byte(at, byte);
                        }
                        if at == 0x5a {
                            self.state
                                .follower_link_state_mut()
                                .set_pit_data_index(byte);
                        }
                    }
                    match offset {
                        0x10 if width == 1 => self.state.set_main_module(value as u8),
                        0x11 if width == 1 => self.state.set_submodule(value as u8),
                        0x5a if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_pit_data_index(value as u8),
                        0x5b if width == 1 => {
                            assert_eq!(value, 1);
                            self.state.follower_link_state_mut().begin_pit_check();
                        }
                        0x5c if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_sprite_oam_state_timer(value as u8),
                        0x61 if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_gravestone_push_timeout(value as u8),
                        0x373 if width == 1 => self
                            .state
                            .follower_link_state_mut()
                            .set_given_damage(value as u8),
                        0x49 => self
                            .state
                            .follower_link_state_mut()
                            .set_force_move_any_direction(value),
                        0x114 => self
                            .state
                            .follower_link_state_mut()
                            .set_tile_below(value as u8),
                        0x3f1 => {
                            if value == 0 {
                                self.state
                                    .dungeon_environment_mut()
                                    .clear_moving_floor_check_flags();
                            } else {
                                self.state
                                    .dungeon_environment_mut()
                                    .or_moving_floor_check_flags(value);
                            }
                        }
                        _ => {}
                    }
                }
                // These are live game latches, not APU ports. Preserve their
                // typed ownership at the instruction's actual write.
                if width == 1 {
                    match offset {
                        0x012c => self.state.set_music_control(value as u8),
                        0x012e => self.state.set_sound_effect_1(value as u8),
                        0x012f => self.state.set_sound_effect_2(value as u8),
                        _ => {}
                    }
                }
                0
            }
            Access::Vblank { at, .. } => {
                *self.nmi_deadline = Some(at + 12);
                0
            }
        }
    }
    // The existing host NMI owns APU execution. This decoder has no APUI
    // accesses; transferring that separate device owner mid-window would
    // replay already rendered audio. Its synchronous clock still owns HMax.
    fn retire_event(
        &mut self,
        event: CpuSynchronousTimelineEvent,
        clock: &CpuMasterTimeline,
    ) -> u32 {
        match event {
            CpuSynchronousTimelineEvent::Bus(
                event @ (CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart),
            ) => self
                .hdma
                .stall(event, |address| {
                    let offset = if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                        (address & 65535) as usize
                    } else if (0x7e0000..0x800000).contains(&address) {
                        (address - 0x7e0000) as usize
                    } else {
                        return scroll_hdma_descriptor_byte(address);
                    };
                    (offset < self.state.ram.len()).then(|| self.state.read_wram_byte(offset))
                })
                .expect("native item graphics lost its HDMA operands"),
            _ => {
                self.timeline_event(event, clock.clock_master_cycles());
                0
            }
        }
    }
}

impl ZeldaState {
    pub(super) fn native_inactive_sprite_body_mask(&self) -> u16 {
        if self.game_state.oam.sprite_sorting_setting() != 0 {
            return 0;
        }
        let globally_inactive = self.game_state.frame.modal_pause_flag != 0
            || self.game_state.frame.submodule != 0;
        (0..16).fold(0u16, |mask, slot| {
            let sprite = self.game_state.sprites.sprite_slots.slot(slot);
            let supported = (sprite.sprite_type() == 0x73
                && (sprite.e() == 0 || sprite.e() == 2
                    || (sprite.e() == 1 && (sprite.a() != 0
                        || u16::from(sprite.direction()) * 2 + u16::from(sprite.graphics()) < 5))))
                || (sprite.sprite_type() == 0x76 && sprite.direction() < 4 && sprite.graphics() < 2);
            let inactive = globally_inactive
                || (sprite.deflection_bits() & 0x80 == 0 && sprite.pause() != 0);
            if sprite.state() == 9 && supported && inactive { mask | (1 << slot) } else { mask }
        })
    }
    pub(super) fn retain_native_dungeon_main_wait(
        &mut self, context: &mut NativeNmiContext, timeline: &CpuMasterTimeline,
        hdma: &mut HdmaTimingState, nmi_deadline: &mut Option<u64>,
    ) {
        use crate::cycle_models::main_wait::{MainWait, WaitAccess};
        use crate::cycle_models::nmi_stack::{CARRY, ZERO, NEGATIVE};
        let mut budget = CpuCycleBudget::from_native_timeline(timeline);
        let mut wait = MainWait::default();
        {
            let bus = std::cell::RefCell::new(DungeonReturnBus::new(
                self, hdma, nmi_deadline,
            ));
            loop {
                let (cycles, advance) = budget.retire_native_instruction(|clock| {
                    wait.step_clocked(context, clock,
                        |at, access| match access {
                            WaitAccess::ReadLatch => bus.borrow_mut().access(at, Access::Read(0x12, 1)) as u8,
                            WaitAccess::Vblank { .. } => 0,
                        },
                        |event, clock| bus.borrow_mut().retire_event(event, clock),
                    )
                });
                crate::cycle_ledger::charge_routine(0x008034, u64::from(cycles));
                if advance != CpuWorkAdvance::Complete { break; }
                assert!(!matches!(wait, MainWait::Complete));
            }
        }
        let cpu = *context;
        assert!(matches!(cpu.pc, 0x8034 | 0x8036));
        assert_eq!((cpu.bank, cpu.sp, cpu.db), (0, 0x1ff, 0));
        self.native_main_wait_cpu_phase = Some(NativeMainWaitCpuPhase {
            host: self.frame_ctr_dbg + 1,
            checkpoint: RomCpuCheckpoint {
                entry_pc: u32::from(cpu.pc), stop_pc: 0x00805d,
                a: cpu.a, x: cpu.x, y: cpu.y, sp: cpu.sp, dp: cpu.d, db: cpu.db,
                carry: cpu.p & CARRY != 0, zero: cpu.p & ZERO != 0,
                overflow: cpu.p & 0x40 != 0, negative: cpu.p & NEGATIVE != 0,
                interrupt_disable: cpu.p & 4 != 0, decimal: cpu.p & 8 != 0,
                accumulator_is_8_bit: cpu.p & 0x20 != 0, index_is_8_bit: cpu.p & 0x10 != 0,
                emulation: false, waiting: false, stack_address: 0, stack_bytes: &[],
            },
            budget,
        });
    }

    fn native_push_handler_ready(&self) -> bool {
        let count = self.game_state.dungeon.torch.torches_start_index();
        if self.game_state.dungeon.object_tracking.misc_object_index() & 0xff00 != 0
            || count > 32
            || count & 1 != 0
        {
            return false;
        }
        // Enhanced camera suppression is owned by the existing semantic lane.
        // The cartridge model owns the ordinary camera and real room child call.
        !self.game_state.enhanced_features.has(4096) || self.native_push_loop_has_no_block_effects()
    }

    fn native_push_loop_has_no_block_effects(&self) -> bool {
        use crate::game_state::ObjectRecord;
        let count = self.game_state.dungeon.torch.torches_start_index();
        self.game_state.dungeon.object_tracking.misc_object_index() & 0xff00 == 0
            && count <= 32
            && count & 1 == 0
            && (0..usize::from(count / 2)).all(|i| {
                !matches!(
                    self.game_state.dungeon.object_tracking.object_record(i),
                    ObjectRecord::PUSHED | ObjectRecord::SLIDING | ObjectRecord::FALLING
                )
            })
    }

    pub(super) fn begin_native_ground_item_graphics(
        &mut self,
        gfx: u8,
        receipt: ItemReceiptReturn,
    ) -> bool {
        let Some(entry) = self.native_item_graphics_entry.take() else {
            return false;
        };
        if matches!(gfx, 6 | 0x18) {
            return false;
        }
        let Some(saved_link_x) = entry.saved_link_x else {
            return false;
        };
        assert_eq!(self.game_state.player.follower_link.tile_action_index(), 5);
        assert_eq!(
            self.game_state.player.follower_link.item_receipt_method(),
            1
        );
        assert_eq!(
            entry.context.a as u8, gfx,
            "native receipt disagrees with caller's graphics selector"
        );
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "native-item-graphics-entry host={} context={:?} field={} raster={:?}",
                self.frame_ctr_dbg,
                entry.context,
                entry.timeline.field_index(),
                entry.timeline.raster_position()
            );
        }
        let Ok(timeline) = entry.timeline.try_native_callee_handoff() else {
            return false;
        };
        assert!(self.native_item_graphics_owner.is_none());
        assert_eq!(
            (
                entry.context.bank,
                entry.context.pc,
                entry.context.d,
                entry.context.p & 0x38
            ),
            (0, 0xd4ed, 0, 0x30)
        );
        // AncillaAdd_ItemReceipt's JSL owns these three stack bytes. Do not
        // import the timing probe's private WRAM or its predicted stores.
        let stack = usize::from(entry.context.sp) + 1;
        self.ram[stack..stack + 3].copy_from_slice(&[0xa3, 0x87, 9]);
        // The admitted ground-item caller owns the saved slot, optional
        // chest position, data bank, and Link_ReceiveItem return address.
        // Materialize those native caller inputs instead of copying shadow RAM.
        let mut parent_stack = vec![receipt.ancilla_slot];
        if self.game_state.player.follower_link.item_receipt_method() == 1 {
            parent_stack.extend(receipt.chest_position.to_be_bytes());
        }
        parent_stack.extend([7, 0x0a, 0x9a, 7]);
        parent_stack.extend([saved_link_x, 7, 0xb9, 0xb5, 7]);
        parent_stack.extend([0x40, 0x9c, 0xb2, 0x81]);
        // Link_Main's $07:8019 JSR owns the ground handler's return.
        // Include it now that the native continuation consumes that RTS.
        parent_stack.extend([0x1b, 0x80]);
        // Link_Main saved DB=0, then returns to the dungeon caller JSL at 028947.
        parent_stack.extend([0, 0x4a, 0x89, 2, 0xae, 0x87]);
        // Main's $8056 JSL returns through Module07's real far frame.
        parent_stack.extend([0x59, 0x80, 0]);
        self.ram[stack + 3..stack + 3 + parent_stack.len()].copy_from_slice(&parent_stack);
        self.ram[SCRATCH_0] = gfx;
        self.native_item_graphics_owner = Some(NativeItemGraphicsOwner {
            hdma: entry.hdma.clone(),
            entry: NativeItemGraphicsEntry {
                context: entry.context,
                timeline,
                hdma: entry.hdma,
                saved_link_x: entry.saved_link_x,
            },
            work: Default::default(),
            receipt_work: None,
            link_work: None,
            apress_work: None,
            y_work: None,
            sword_work: None,
            movement_work: None,
            velocity_work: None,
            cardinal_work: None,
            vertical_cardinal_work: None,
            vertical_response_work: None,
            vertical_surface_work: None,
            ground_animation_work: None,
            dungeon_return_work: None,
            main_response_work: None,
            main_tile_work: None,
            main_tile_classifier: None,
            cardinal_classifiers: Default::default(),
            diagonal_work: None,
            probe_work: None,
            classifier_work: None,
            high_classifier_work: None,
            nmi_deadline: None,
            gfx,
            receipt,
        });
        self.advance_native_ground_item_graphics();
        true
    }

    fn advance_native_ground_item_graphics(&mut self) {
        let mut owner = self.native_item_graphics_owner.take().unwrap();
        loop {
            if owner.dungeon_return_work.is_none()
                && owner.ground_animation_work.as_ref().is_some_and(|work| {
                    work.complete() && matches!(work.after_push_tail(),
                        Some(crate::cycle_models::dungeon_after_push::DungeonAfterPushTail::SpritePrefix {
                            tail: crate::cycle_models::dungeon_sprite_prefix::SpritePrefixTail::Returned, ..
                        }))
                })
                && !self.game_state.enhanced_features.has(u32::MAX)
                && !self.spiral_stair_return_player_control_pending
            {
                assert_eq!((owner.entry.context.bank, owner.entry.context.pc, owner.entry.context.sp), (2, 0x8842, 0x1f4));
                owner.dungeon_return_work = Some(DungeonMainReturn::default());
            }
            if owner
                .nmi_deadline
                .is_some_and(|at| owner.entry.timeline.clock_master_cycles() >= at)
                || owner.return_tail_ready()
            {
                break;
            }
            if let Some(work) = &mut owner.dungeon_return_work {
                let (cycles, routine) = work.step(
                    &mut owner.entry.context,
                    &mut owner.entry.timeline,
                    DungeonReturnBus::new(self, &mut owner.hdma, &mut owner.nmi_deadline),
                ).expect("active native dungeon return");
                crate::cycle_ledger::charge_routine(routine, cycles);
                continue;
            }
            let routine = if let Some(work)=&owner.ground_animation_work {work.routine()}
            else if owner.main_response_work.is_some() {0x07d104}
            else if let Some(work)=&owner.main_tile_work {
                if work.child_ready() {0x07d9d8} else {0x07d077}
            } else if owner.vertical_surface_work.is_some() {
                0x07bbe6
            } else if owner.vertical_response_work.is_some() {
                0x07ba2e
            } else if let Some(probe) = &owner.vertical_cardinal_work {
                if probe.child_ready() {
                    0x07d9d8
                } else {
                    0x07ba0a
                }
            } else if owner.cardinal_work.is_some() {
                0x07b7c7
            } else if owner.velocity_work.is_some() {
                0x07e245
            } else if owner
                .classifier_work
                .as_ref()
                .is_some_and(|w| w.tail().is_none())
                || owner
                    .high_classifier_work
                    .as_ref()
                    .is_some_and(|w| w.tail().is_none())
            {
                0x07d9d8
            } else if owner.probe_work.is_some() {
                0x07ce85
            } else if owner.diagonal_work.is_some() {
                0x07b64f
            } else if owner.movement_work.is_some() {
                0x0781e2
            } else if owner.sword_work.is_some() {
                0x0781c9
            } else if owner.y_work.is_some() {
                0x0781b3
            } else if owner.apress_work.is_some() {
                0x079c41
            } else if let Some(work) = &owner.link_work {
                work.routine()
            } else {
                owner
                    .receipt_work
                    .as_ref()
                    .map_or_else(|| owner.work.routine(), ItemReceiptReturnWork::routine)
            };
            let bus = ItemBus {
                state: self,
                hdma: &mut owner.hdma,
                nmi_deadline: &mut owner.nmi_deadline,
                publish_push: owner.ground_animation_work.as_ref().is_some_and(|w|(w.push_active() && !w.after_push_active()) || w.drawing_active() || w.sprite_prefix_active()),
                publish_dungeon_camera: owner.ground_animation_work.as_ref().is_some_and(|w|w.after_push_active() && !w.drawing_active() && !w.sprite_prefix_active()),
                publish_hud: owner
                    .link_work
                    .as_ref()
                    .is_some_and(LinkReceiveItemReturn::hud_active),
                publish_link: owner.link_work.is_some() && !owner.ground_animation_work.as_ref().is_some_and(GroundAnimationReturn::push_active),
                publish_probe: !owner.ground_animation_work.as_ref().is_some_and(GroundAnimationReturn::push_active) && (owner.probe_work.is_some() || owner.vertical_cardinal_work.is_some() || owner.main_tile_work.is_some()),
            };
            let cycles = if let Some(work)=&mut owner.ground_animation_work {work.step(&mut owner.entry.context,&mut owner.entry.timeline,bus)}
            else if let Some(work)=&mut owner.main_response_work {work.step(&mut owner.entry.context,&mut owner.entry.timeline,bus)}
            else if let Some(work)=&mut owner.main_tile_work {
                if work.child_ready() {owner.main_tile_classifier.as_mut().unwrap().step(&mut owner.entry.context,&mut owner.entry.timeline,bus)}
                else {work.step(&mut owner.entry.context,&mut owner.entry.timeline,bus)}
            } else if let Some(work) = &mut owner.vertical_surface_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.vertical_response_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(probe) = &mut owner.vertical_cardinal_work {
                if probe.child_ready() {
                    owner.cardinal_classifiers[probe.sample()]
                        .as_mut()
                        .unwrap()
                        .step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
                } else {
                    probe.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
                }
            } else if let Some(work) = &mut owner.cardinal_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.velocity_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = owner
                .high_classifier_work
                .as_mut()
                .filter(|w| w.tail().is_none())
            {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = owner
                .classifier_work
                .as_mut()
                .filter(|w| w.tail().is_none())
            {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.probe_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.diagonal_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.movement_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.sword_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.y_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.apress_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.link_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else if let Some(work) = &mut owner.receipt_work {
                work.step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            } else {
                owner
                    .work
                    .step(&mut owner.entry.context, &mut owner.entry.timeline, bus)
            }
            .unwrap();
            crate::cycle_ledger::charge_routine(routine, cycles);
            if owner.work.complete() && owner.receipt_work.is_none() {
                assert_eq!(
                    (owner.entry.context.bank, owner.entry.context.pc),
                    (9, 0x87a4)
                );
                if nmi::debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "native-item-graphics-return host={} field={} raster={:?}",
                        self.frame_ctr_dbg,
                        owner.entry.timeline.field_index(),
                        owner.entry.timeline.raster_position()
                    );
                }
                owner.receipt_work = Some(ItemReceiptReturnWork::default());
            }
            if owner
                .receipt_work
                .as_ref()
                .is_some_and(ItemReceiptReturnWork::complete)
                && owner.link_work.is_none()
            {
                assert_eq!(
                    (owner.entry.context.bank, owner.entry.context.pc),
                    (7, 0x9a0b)
                );
                self.game_state
                    .sprites
                    .ancilla_slots
                    .reload_from_ram(&self.ram);
                assert!(self.early_ground_item_receipt_tail.is_none());
                self.early_ground_item_receipt_tail = Some(owner.receipt);
                if nmi::debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "native-item-receipt-return host={} field={} raster={:?}",
                        self.frame_ctr_dbg,
                        owner.entry.timeline.field_index(),
                        owner.entry.timeline.raster_position()
                    );
                }
                owner.link_work = Some(LinkReceiveItemReturn::default());
            }
            if owner
                .link_work
                .as_ref()
                .is_some_and(LinkReceiveItemReturn::complete)
                && owner.apress_work.is_none()
            {
                assert_eq!(
                    (owner.entry.context.bank, owner.entry.context.pc),
                    (7, 0xb5ba)
                );
                assert!(self.early_ground_link_receive_item_tail.is_none());
                self.early_ground_link_receive_item_tail = Some(owner.receipt.item);
                if nmi::debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "native-item-link-return host={} field={} raster={:?}",
                        self.frame_ctr_dbg,
                        owner.entry.timeline.field_index(),
                        owner.entry.timeline.raster_position()
                    );
                }
                owner.apress_work = Some(GroundItemApressReturn::default());
            }
            if owner
                .apress_work
                .as_ref()
                .is_some_and(GroundItemApressReturn::complete)
                && owner.y_work.is_none()
            {
                assert_eq!(
                    (owner.entry.context.bank, owner.entry.context.pc),
                    (7, 0x81b3)
                );
                if nmi::debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "native-item-apress-return host={} field={} raster={:?}",
                        self.frame_ctr_dbg,
                        owner.entry.timeline.field_index(),
                        owner.entry.timeline.raster_position()
                    );
                }
                owner.y_work = Some(GroundYItemPrefix::default());
            }
            if owner.y_work.as_ref().is_some_and(|work| {
                matches!(
                    work.tail(),
                    Some(GroundYItemTail::AfterYItem | GroundYItemTail::SkipYItem)
                )
            }) && owner.sword_work.is_none()
            {
                if nmi::debug_frame_selection_env_matches(
                    "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                    self.frame_ctr_dbg,
                ) {
                    eprintln!(
                        "native-item-y-return host={} pc={:04x} field={} raster={:?}",
                        self.frame_ctr_dbg,
                        owner.entry.context.pc,
                        owner.entry.timeline.field_index(),
                        owner.entry.timeline.raster_position()
                    );
                }
                owner.sword_work = Some(GroundSwordCapeReturn::from_entry(owner.entry.context.pc)
                    .with_sword_swing_guards());
            }
            if owner.sword_work.as_ref().is_some_and(|work| work.tail() == Some(crate::cycle_models::ground_sword_cape_return::GroundSwordCapeTail::Movement)) && owner.movement_work.is_none() {
                assert_eq!(owner.entry.context.pc, 0x81e2);
                owner.movement_work = Some(GroundMovementPrefix::default());
            }
            if owner.movement_work.as_ref().is_some_and(|work| matches!(work.tail(), Some(crate::cycle_models::ground_movement_prefix::GroundMovementTail::Collision { .. }))) && owner.diagonal_work.is_none() {
                assert_eq!(owner.entry.context.pc, 0xb64f);
                owner.diagonal_work = Some(DiagonalCollisionPrefix::default());
            }
            if owner.main_response_work.as_ref().is_some_and(|w|w.tail()==Some(crate::cycle_models::main_tile_response::MainTileResponseTail::Returned)) && owner.ground_animation_work.is_none() {
                assert_eq!(owner.entry.context.pc,0x82c5);owner.ground_animation_work=Some(GroundAnimationReturn::with_camera_skip(self.apply_links_movement_to_camera_called && self.game_state.enhanced_features.has(4096)).with_native_push_handler(self.native_push_handler_ready()).with_native_after_push(true).with_native_room_tags(!self.game_state.enhanced_features.has(4096)).with_native_interactions(true).with_native_edge(true).with_native_lamp(true).with_native_sprite_scroll(true).with_native_drawing(true).with_native_sprite_prefix(true).with_native_garnish(true).with_native_follower(true).with_native_ancilla(true).with_live_inactive_slot_bodies(true));
            }
            if owner.main_tile_work.as_ref().is_some_and(MainTileProbe::complete) && owner.main_response_work.is_none() {
                assert_eq!(owner.entry.context.pc,0xd104);owner.main_response_work=Some(MainTileResponse::default());
            }
            if owner.cardinal_work.as_ref().is_some_and(|w|w.tail()==Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::MainHandler)) && owner.main_tile_work.is_none() {
                assert_eq!(owner.entry.context.pc,0xd077);
                owner.main_tile_work=Some(MainTileProbe::default());
            }
            if let Some(work)=&mut owner.main_tile_work {
                if work.child_ready() {
                    if owner.main_tile_classifier.is_none() {owner.main_tile_classifier=Some(TileClassifier::default());}
                    if owner.main_tile_classifier.as_ref().unwrap().tail()==Some(TileClassifierTail::Complete) {work.begin_response();}
                }
            }
            if owner
                .vertical_surface_work
                .as_ref()
                .is_some_and(|w| w.tail() == Some(VerticalSurfaceTail::Returned))
            {
                owner
                    .cardinal_work
                    .as_mut()
                    .unwrap()
                    .resume_after_vertical_axis(owner.entry.context.pc);
                owner.vertical_surface_work = None;
                owner.vertical_response_work = None;
                owner.vertical_cardinal_work = None;
                owner.cardinal_classifiers = Default::default();
            }
            if owner
                .vertical_response_work
                .as_ref()
                .is_some_and(|w| w.tail() == Some(VerticalResponseTail::SurfaceTail))
                && owner.vertical_surface_work.is_none()
            {
                assert_eq!(owner.entry.context.pc, 0xbbe6);
                owner.vertical_surface_work = Some(VerticalSurfaceResponse::default());
            }
            if owner
                .vertical_response_work
                .as_ref()
                .is_some_and(|w| w.tail() == Some(VerticalResponseTail::Returned))
            {
                owner
                    .cardinal_work
                    .as_mut()
                    .unwrap()
                    .resume_after_vertical_axis(owner.entry.context.pc);
                owner.vertical_response_work = None;
                owner.vertical_cardinal_work = None;
                owner.cardinal_classifiers = Default::default();
            }
            if owner
                .vertical_cardinal_work
                .as_ref()
                .is_some_and(|p| p.tail() == Some(VerticalCardinalTail::Response))
                && owner.vertical_response_work.is_none()
            {
                assert_eq!(owner.entry.context.pc, 0xba2e);
                owner.vertical_response_work = Some(VerticalCollisionResponse::default());
            }
            if owner
                .vertical_cardinal_work
                .as_ref()
                .is_some_and(|p| p.tail() == Some(VerticalCardinalTail::Returned))
            {
                owner
                    .cardinal_work
                    .as_mut()
                    .unwrap()
                    .resume_after_vertical_axis(owner.entry.context.pc);
                owner.vertical_cardinal_work = None;
                owner.cardinal_classifiers = Default::default();
            }
            if owner.cardinal_work.as_ref().is_some_and(|p|matches!(p.tail(),Some(crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail::SlopeChecks {..}))) && owner.entry.context.pc==0xba0a && owner.vertical_cardinal_work.is_none() {
                owner.vertical_cardinal_work=Some(VerticalCardinalProbe::default());
            }
            if let Some(probe) = &mut owner.vertical_cardinal_work {
                if probe.child_ready() {
                    let classifier = &mut owner.cardinal_classifiers[probe.sample()];
                    if classifier.is_none() {
                        *classifier = Some(TileClassifier::default());
                    }
                    if classifier.as_ref().unwrap().tail() == Some(TileClassifierTail::Complete) {
                        probe.begin_next();
                    }
                }
            }
            if owner.velocity_work.as_ref().is_some_and(|w| {
                w.tail() == Some(crate::cycle_models::ground_velocity::GroundVelocityTail::Complete)
            }) && owner.cardinal_work.is_none()
            {
                assert_eq!(
                    (owner.entry.context.bank, owner.entry.context.pc),
                    (7, 0x82c2)
                );
                owner.cardinal_work = Some(CardinalCollisionPrefix::default());
            }
            if owner.diagonal_work.as_ref().is_some_and(|work| matches!(work.tail(), Some(crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::VerticalProbe { .. }))) && owner.probe_work.is_none() {
                assert_eq!(owner.entry.context.pc, 0xce85);
                owner.probe_work = Some(VerticalSlopeProbe::default());
            }
            if owner
                .probe_work
                .as_ref()
                .is_some_and(VerticalSlopeProbe::child_ready)
                && owner.classifier_work.is_none()
            {
                assert_eq!(owner.entry.context.pc, 0xd9d8);
                owner.classifier_work = Some(TileClassifier::default());
            }
            if let Some(probe) = &mut owner.probe_work {
                if probe.corner() == 0
                    && owner.classifier_work.as_ref().is_some_and(|w| {
                        w.tail()
                            == Some(
                                crate::cycle_models::tile_classifier::TileClassifierTail::Complete,
                            )
                    })
                {
                    assert_eq!(owner.entry.context.pc, 0xceba);
                    probe.begin_high_corner();
                }
                if probe.corner() == 1
                    && probe.child_ready()
                    && owner.high_classifier_work.is_none()
                {
                    assert_eq!(owner.entry.context.pc, 0xd9d8);
                    owner.high_classifier_work = Some(TileClassifier::default());
                }
                if probe.corner() == 1
                    && owner.high_classifier_work.as_ref().is_some_and(|w| {
                        w.tail()
                            == Some(
                                crate::cycle_models::tile_classifier::TileClassifierTail::Complete,
                            )
                    })
                {
                    assert_eq!(owner.entry.context.pc, 0xcec6);
                    probe.begin_return();
                }
            }
            if owner
                .probe_work
                .as_ref()
                .is_some_and(VerticalSlopeProbe::complete)
            {
                assert_eq!(owner.entry.context.pc, 0xb682);
                owner
                    .diagonal_work
                    .as_mut()
                    .unwrap()
                    .resume_after_vertical_probe();
                owner.probe_work = None;
                owner.classifier_work = None;
                owner.high_classifier_work = None;
            }
            if owner.diagonal_work.as_ref().is_some_and(|w| w.tail() == Some(crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::Complete)) && owner.velocity_work.is_none() {
                assert_eq!(owner.entry.context.pc, 0x82be);
                owner.velocity_work = Some(GroundVelocity::default());
            }
        }
        if nmi::debug_frame_selection_env_matches(
            "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
            self.frame_ctr_dbg,
        ) {
            eprintln!(
                "native-item-graphics-boundary host={} pc={:06x} complete={} field={} raster={:?}",
                self.frame_ctr_dbg,
                (u32::from(owner.entry.context.bank) << 16) | u32::from(owner.entry.context.pc),
                owner.return_tail_ready(),
                owner.entry.timeline.field_index(),
                owner.entry.timeline.raster_position()
            );
        }
        let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: owner.gfx,
            ground_apress_tail: Some(owner.receipt),
        };
        let accepts_nmi = owner
            .nmi_deadline
            .is_some_and(|at| owner.entry.timeline.clock_master_cycles() >= at);
        if owner.return_tail_ready() && !accepts_nmi {
            let tail = owner.y_work.as_ref().unwrap().tail().unwrap();
            assert!(self.early_ground_y_item_tail.is_none());
            self.early_ground_y_item_tail = Some(tail);
            assert!(self.early_ground_sword_cape_tail.is_none());
            self.early_ground_sword_cape_tail = owner
                .sword_work
                .as_ref()
                .and_then(GroundSwordCapeReturn::tail);
            assert!(self.early_ground_movement_tail.is_none());
            self.early_ground_movement_tail = owner
                .movement_work
                .as_ref()
                .and_then(GroundMovementPrefix::tail);
            assert!(self.early_diagonal_collision_tail.is_none());
            self.early_diagonal_collision_tail = owner
                .diagonal_work
                .as_ref()
                .and_then(DiagonalCollisionPrefix::tail);
            assert!(!self.early_vertical_slope_probe);
            self.early_vertical_slope_probe = owner
                .probe_work
                .as_ref()
                .is_some_and(|w| w.child_ready() || w.complete());
            assert!(self.early_tile_classifier_tail.is_none());
            self.early_tile_classifier_tail = owner
                .classifier_work
                .as_ref()
                .and_then(TileClassifier::tail);
            assert!(self.early_high_tile_classifier_tail.is_none());
            self.early_high_tile_classifier_tail = owner
                .high_classifier_work
                .as_ref()
                .and_then(TileClassifier::tail);
            assert!(!self.early_ground_animation_return);
            self.early_ground_animation_return=owner.ground_animation_work.as_ref().is_some_and(GroundAnimationReturn::complete);
            self.early_ground_camera_tail=owner.ground_animation_work.as_ref().and_then(GroundAnimationReturn::camera_tail);
            self.early_player_ancilla_tail=owner.ground_animation_work.as_ref().and_then(GroundAnimationReturn::ancilla_tail);
            self.early_dungeon_push_caller_tail=owner.ground_animation_work.as_ref().and_then(GroundAnimationReturn::push_tail);
            self.early_dungeon_after_push_tail=owner.ground_animation_work.as_ref().and_then(GroundAnimationReturn::after_push_tail);
            if let Some(work) = &owner.dungeon_return_work {
                assert!(work.complete());
                assert_eq!((owner.entry.context.bank, owner.entry.context.pc, owner.entry.context.sp), (0, 0x8034, 0x1ff));
                self.early_dungeon_after_push_tail = Some(crate::cycle_models::dungeon_after_push::DungeonAfterPushTail::MainWait);
                self.main_loop_caller_phase = crate::cycle_models::main_wait::MainLoopCallerPhase::Waiting;
                assert!(!self.main_loop_sprite_preparation_completed);
                self.main_loop_sprite_preparation_completed = true;
                self.last_nmi_prepare_sprites_master_cycles = Some(u32::try_from(work.sprite_preparation_master_cycles()).unwrap());
                assert!(self.pending_main_loop_common_suffix.replace(MainLoopCommonSuffixContinuation::ReturnedToMainWait).is_none());
            }
            assert!(self.early_main_tile_response.is_none());
            self.early_main_tile_response=owner.main_response_work.as_ref().and_then(MainTileResponse::tail);
            assert!(self.early_main_tile_probe.is_none());
            self.early_main_tile_probe=owner.main_tile_work.as_ref().map(|w|w.handoff(owner.main_tile_classifier.as_ref().and_then(TileClassifier::tail).unwrap()));
            assert!(self.early_vertical_surface_tail.is_none());
            self.early_vertical_surface_tail = owner
                .vertical_surface_work
                .as_ref()
                .and_then(VerticalSurfaceResponse::tail);
            assert!(self.early_vertical_response_tail.is_none());
            self.early_vertical_response_tail = owner
                .vertical_response_work
                .as_ref()
                .and_then(VerticalCollisionResponse::tail);
            assert!(self.early_vertical_cardinal_probe.is_none());
            self.early_vertical_cardinal_probe =
                owner.vertical_cardinal_work.as_ref().map(|probe| {
                    probe.handoff(std::array::from_fn(|i| {
                        owner.cardinal_classifiers[i]
                            .as_ref()
                            .and_then(TileClassifier::tail)
                    }))
                });
            assert!(self.early_cardinal_collision_tail.is_none());
            self.early_cardinal_collision_tail = owner
                .cardinal_work
                .as_ref()
                .and_then(CardinalCollisionPrefix::tail);
            assert!(self.early_ground_velocity_tail.is_none());
            self.early_ground_velocity_tail =
                owner.velocity_work.as_ref().and_then(GroundVelocity::tail);
            self.game_state
                .sprites
                .ancilla_slots
                .reload_from_ram(&self.ram);
            self.native_item_graphics_return_after_nmi = true;
            let before = crate::cycle_ledger::master();
            // The player/module return remains with its existing
            // owner. It must not accept the decoder's final held NMI twice.
            self.lane_finish_item_receipt_graphics(0, continuation, None, None, None);
            if owner.dungeon_return_work.is_some() {
                self.retain_native_dungeon_main_wait(&mut owner.entry.context,
                    &owner.entry.timeline, &mut owner.hdma, &mut owner.nmi_deadline);
            }
            if nmi::debug_frame_selection_env_matches(
                "ZELDA3_DEBUG_DIALOGUE_CPU_PHASE",
                self.frame_ctr_dbg,
            ) {
                eprintln!(
                    "native-item-graphics-parent-return host={} retired={} end_phase={:?}",
                    self.frame_ctr_dbg,
                    crate::cycle_ledger::master() - before,
                    self.main_loop_caller_phase
                );
            }
        } else {
            self.native_item_graphics_owner = Some(owner);
            self.game_execution_scheduler
                .schedule_after_current_trailing_nmi(
                    GameWorkContinuation::FinishItemReceiptGraphics { continuation },
                );
        }
    }

    pub(super) fn resume_native_ground_item_graphics(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        let Some(mut owner) = self.native_item_graphics_owner.take() else {
            return false;
        };
        let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: owner.gfx,
            ground_apress_tail: Some(owner.receipt),
        };
        assert_eq!(
            self.game_execution_scheduler
                .take_after_current_trailing_nmi(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics { continuation })
        );
        assert!(owner.nmi_deadline.take().is_some());
        self.interrupt_nmi(input, oam_dma_source, false);
        let cycles = self
            .last_nmi_handler_master_cycles
            .expect("native graphics lost its NMI cost");
        owner
            .entry
            .timeline
            .advance_synchronous_after_semantics_with_clock(cycles, |event, clock| {
                Ok::<u32, String>(
                    ItemBus {
                        state: self,
                        hdma: &mut owner.hdma,
                        nmi_deadline: &mut owner.nmi_deadline,
                        publish_push: false,
                        publish_dungeon_camera: false,
                        publish_hud: false,
                        publish_link: false,
                        publish_probe: false,
                    }
                    .retire_event(event, clock),
                )
            })
            .unwrap();
        owner.hdma = HdmaTimingState::new(&self.dma_with_native_hdma_enable());
        self.capture_display_snapshot();
        self.native_item_graphics_owner = Some(owner);
        self.advance_native_ground_item_graphics();
        true
    }
}
