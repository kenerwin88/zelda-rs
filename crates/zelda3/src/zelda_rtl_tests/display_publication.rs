use super::*;

fn captured_display_snapshot() -> DisplaySnapshot {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    *state.display_snapshot.take().unwrap()
}

#[test]
fn parity_diagnostics_stay_under_the_repository_target() {
    let path = parity_trace_path("display.trace");
    assert!(path.ends_with("target/parity-traces/display.trace"));
    assert!(!path.starts_with("/tmp"));
}

#[test]
fn publication_plan_keeps_memory_domains_independent() {
    let mut snapshot = captured_display_snapshot();
    snapshot.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    snapshot.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    snapshot.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    snapshot.animated_bg_scanout_generation =
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            publish_live_overworld_transition_half_color: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert!(plan.compose_live_cgram);
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert_eq!(
        plan.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        plan.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::CapturedBeforeNmi
    );
    assert!(plan.publish_live_overworld_transition_half_color);
}

#[test]
fn dungeon_exit_publication_plan_promotes_only_live_boundary_domains() {
    let mut snapshot = captured_display_snapshot();
    snapshot.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    snapshot.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            dungeon_exit_crosses_nmi_boundary: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert_eq!(
        plan.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::LiveAfterNmi
    );
}

#[test]
fn attract_memory_retention_still_publishes_the_live_scroll_origin() {
    let mut snapshot = captured_display_snapshot();
    snapshot.vram_generation = DisplayVramGeneration::ComposeLiveAfterNmi;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            retain_previous_nmi_display_memory: true,
            attract_map_retains_display_memory: true,
            world_map_fade_display: true,
            world_map_mode7_brightness_is_early_published: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert!(!plan.compose_live_cgram);
    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::LiveAfterNmi
    );
    assert!(plan.world_map_fade_display);
    assert!(plan.world_map_mode7_brightness_is_early_published);
}

#[test]
fn bad_weather_publication_promotes_bg1_and_animated_tiles_together() {
    let mut snapshot = captured_display_snapshot();
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;
    snapshot.animated_bg_scanout_generation =
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            publish_live_overworld_bad_weather_scroll: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::LiveBg1AfterNmi
    );
    assert_eq!(
        plan.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
}

#[test]
fn post_nmi_bg_scroll_writes_target_the_following_scanout() {
    let mut state = ZeldaState::new();
    let current_scanout = [
        [0x0111, 0x0122],
        [0x0233, 0x0244],
        [0x0355, 0x0366],
        [0x0077, 0x0088],
    ];
    for (layer, [h_scroll, v_scroll]) in state.ppu.bg_layer.iter_mut().zip(current_scanout) {
        layer.h_scroll = h_scroll;
        layer.v_scroll = v_scroll;
    }
    state.capture_display_snapshot();

    let following_scanout = [
        [0x1111, 0x1122],
        [0x1233, 0x1244],
        [0x1355, 0x1366],
        [0x1077, 0x1088],
    ];
    state.publish_bg_scroll_for_following_scanout(BgScrollRegisterScanout {
        offsets: following_scanout,
    });

    let displayed = state.with_display_snapshot(|display| {
        display
            .ppu
            .bg_layer
            .map(|layer| [layer.h_scroll, layer.v_scroll])
    });
    assert_eq!(displayed, current_scanout);

    state.capture_display_snapshot();
    let displayed = state.with_display_snapshot(|display| {
        display
            .ppu
            .bg_layer
            .map(|layer| [layer.h_scroll, layer.v_scroll])
    });
    assert_eq!(displayed, following_scanout);
}

#[test]
fn display_snapshot_consumes_vram_once_and_retains_active_obj_generation() {
    let mut state = ZeldaState::new();
    let held_oam = vec![0x1234; state.ppu.oam.len()];
    let held_obj_vram = vec![0x5678; 0x400];
    state.next_display_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    state.next_display_bg_scroll_generation = DisplayBgScrollGeneration::ComposeLiveAfterNmi;
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.active_display_obj_generation = DisplayObjGeneration::RetainCapturedMemory {
        oam: held_oam.clone(),
        vram: held_obj_vram.clone(),
    };

    state.capture_display_snapshot();

    let snapshot = state.display_snapshot.as_ref().expect("display snapshot");
    assert_eq!(
        snapshot.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        snapshot.bg_scroll_generation,
        DisplayBgScrollGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        snapshot.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        snapshot.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert_eq!(
        snapshot.obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam.clone(),
            vram: held_obj_vram.clone(),
        },
    );
    assert_eq!(
        state.next_display_vram_generation,
        DisplayVramGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        state.next_display_bg_scroll_generation,
        DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(state.next_display_obj_scanout_generation, None);
    assert_eq!(
        state.active_display_obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam.clone(),
            vram: held_obj_vram.clone(),
        },
    );

    state.capture_display_snapshot();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .expect("second display snapshot")
            .obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam,
            vram: held_obj_vram,
        },
    );
}

#[test]
fn presented_vram_generation_combines_snapshot_and_domain_retention_once() {
    assert_eq!(
        DisplayVramGeneration::ComposeLiveAfterNmi.resolve_for_scanout(false),
        DisplayVramGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        DisplayVramGeneration::ComposeLiveAfterNmi.resolve_for_scanout(true),
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        DisplayVramGeneration::RetainCapturedBeforeNmi.resolve_for_scanout(false),
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
}

#[test]
fn interface_exit_stripe_upload_belongs_to_the_following_scanout() {
    // Ordinary dialogue and the save menu share this interface-exit boundary;
    // the destination module does not own the stripe until the next scanout.
    assert!(interface_exit_bg_upload_misses_current_scanout(
        0x0e, 9, true,
    ));
    assert!(interface_exit_bg_upload_misses_current_scanout(
        0x0e, 23, true,
    ));
    assert!(!interface_exit_bg_upload_misses_current_scanout(
        0x0e, 0x0e, true,
    ));
    assert!(!interface_exit_bg_upload_misses_current_scanout(9, 9, true));
    assert!(!interface_exit_bg_upload_misses_current_scanout(
        0x0e, 9, false,
    ));
}

#[test]
fn pre_main_nmi_resume_selects_display_domains_by_hardware_generation() {
    let dialogue_upload = PreMainNmiResume::DialogueVwfUpload;
    assert!(dialogue_upload.defers_current_trailing_nmi());
    assert_eq!(
        dialogue_upload.scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: None,
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: None,
        },
    );
    assert_eq!(
        PreMainNmiResume::OverworldAuxGraphicsReturn.scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: None,
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: None,
        },
    );
    for (return_phase, publication, bg_scroll) in [
        (
            NmiPhase::BeforeNmi,
            DisplaySnapshotPublication::PublishCaptured,
            DisplayBgScrollGeneration::ComposeLiveAfterNmi,
        ),
        (
            NmiPhase::AfterNmi,
            DisplaySnapshotPublication::RetainPublished,
            DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
        ),
    ] {
        assert_eq!(
            PreMainNmiResume::OverworldSpriteReloadReturn {
                scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(return_phase),
            }
            .scanout_generations(),
            PreMainNmiScanoutGenerations {
                publication,
                vram: DisplayVramGeneration::RetainCapturedBeforeNmi,
                animated_bg: None,
                bg_scroll,
                obj: None,
            },
        );
    }
    assert_eq!(
        PreMainNmiResume::OverworldSpriteReloadReturn {
            scanout: OverworldSpriteReloadResumeScanout::FollowingNmi,
        }
        .scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: Some(AnimatedBgScanoutGeneration::LiveAfterNmi),
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: None,
        },
    );
    assert_eq!(
        PreMainNmiResume::DungeonSupertileQuadrantUploads.scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: Some(AnimatedBgScanoutGeneration::LiveAfterNmi),
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: Some(ObjScanoutGenerations {
                oam: GraphicsDmaGeneration::LiveAfterMain,
                link_obj: GraphicsDmaGeneration::LiveAfterMain,
            }),
        },
    );
    let dungeon_phase = PreMainNmiResume::DungeonSupertileQuadrantUploads;
    for subsubmodule in 5..=15 {
        assert!(
            dungeon_phase.continues_after_main(crate::game_state::FrameState {
                main_module: 7,
                submodule: 2,
                subsubmodule,
                ..Default::default()
            })
        );
    }
    assert!(
        !dungeon_phase.continues_after_main(crate::game_state::FrameState {
            main_module: 7,
            submodule: 0,
            subsubmodule: 0,
            ..Default::default()
        })
    );
}
