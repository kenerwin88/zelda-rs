use platform::{DeveloperDestination, DeveloperThumbnail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeveloperRouteBookmark {
    pub id: &'static str,
    pub replay_path: &'static str,
    pub rom_path: &'static str,
    pub checkpoint_path: Option<&'static str>,
    pub target_frame: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperDestinationTarget {
    RouteBookmark(DeveloperRouteBookmark),
    SyntheticRoom(DeveloperSyntheticRoom),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeveloperSyntheticRoom {
    pub id: &'static str,
    pub room_id: u16,
    pub tilemap_json: &'static str,
    pub seed_checkpoint_path: &'static str,
    pub theme_checkpoint_path: &'static str,
    pub visual_theme: DeveloperSyntheticRoomTheme,
    pub music_track: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperSyntheticRoomTheme {
    Kakariko,
}

pub fn developer_destinations() -> Vec<DeveloperDestination> {
    vec![
        DeveloperDestination::curated_preset(
            "preset-dev-sandbox",
            "Dev Sandbox",
            "DEV SANDBOX",
            "ROOM 01FF",
            "DEV JSON",
            "SYNTH ROOM",
            DeveloperThumbnail::DevRoom,
        ),
        DeveloperDestination::curated_preset(
            "preset-route-start",
            "Route Start",
            "ROUTE START",
            "FRAME 0",
            "ROUTE SAVE",
            "FRAME 0",
            DeveloperThumbnail::RouteStart,
        ),
        DeveloperDestination::curated_preset(
            "preset-sanctuary",
            "Sanctuary",
            "SANCTUARY",
            "ROOM 0050",
            "ROUTE SAVE",
            "FRAME 12000",
            DeveloperThumbnail::Sanctuary,
        ),
        DeveloperDestination::curated_preset(
            "preset-late-dungeon",
            "Late Dungeon",
            "LATE DUNGEON",
            "FRAME 1045813",
            "REPLAY-BISECT",
            "CKPT READY",
            DeveloperThumbnail::LateDungeon,
        ),
        DeveloperDestination::route_bookmark(
            "route-start",
            "Route Start",
            "ROUTE START",
            "FRAME 0",
            "ROUTE SAVE",
            "FRAME 0",
            DeveloperThumbnail::RouteStart,
        ),
        DeveloperDestination::route_bookmark(
            "route-file-select",
            "File Select",
            "FILE SELECT",
            "ROOM 0050",
            "ROUTE SAVE",
            "FRAME 12000",
            DeveloperThumbnail::FileSelect,
        ),
        DeveloperDestination::route_bookmark(
            "route-late-checkpoint",
            "Late Route Checkpoint",
            "LATE CHECKPT",
            "FRAME 1045813",
            "REPLAY-BISECT",
            "CKPT READY",
            DeveloperThumbnail::LateDungeon,
        ),
        DeveloperDestination::locked_browser(
            "unverified-overworld-browser",
            "Overworld Browser",
            "OVERWORLD BROW",
            "OVERWORLD",
            "LOCKED",
            "NEEDS INIT TEST",
            DeveloperThumbnail::LockedOverworld,
        ),
        DeveloperDestination::locked_browser(
            "unverified-room-browser",
            "Dungeon Room Browser",
            "DUNGEON BROW",
            "DUNGEON",
            "LOCKED",
            "NEEDS INIT TEST",
            DeveloperThumbnail::LockedDungeon,
        ),
    ]
}

pub fn destination_target(id: &str) -> Option<DeveloperDestinationTarget> {
    route_bookmark(id)
        .map(DeveloperDestinationTarget::RouteBookmark)
        .or_else(|| synthetic_room(id).map(DeveloperDestinationTarget::SyntheticRoom))
}

pub fn synthetic_room(id: &str) -> Option<DeveloperSyntheticRoom> {
    const DEV_SANDBOX_ROOM_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_rooms/dev_sandbox_room.json"
    ));
    const DEV_SANDBOX_SEED_CHECKPOINT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.cache/replay-bisect/rust-frame-384852.sav"
    );
    const KAKARIKO_THEME_CHECKPOINT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.cache/replay-bisect/rust-frame-1034462.sav"
    );

    match id {
        "preset-dev-sandbox" => Some(DeveloperSyntheticRoom {
            id: "preset-dev-sandbox",
            room_id: 0x01ff,
            tilemap_json: DEV_SANDBOX_ROOM_JSON,
            seed_checkpoint_path: DEV_SANDBOX_SEED_CHECKPOINT_PATH,
            theme_checkpoint_path: KAKARIKO_THEME_CHECKPOINT_PATH,
            visual_theme: DeveloperSyntheticRoomTheme::Kakariko,
            music_track: 0x07,
        }),
        _ => None,
    }
}

pub fn route_bookmark(id: &str) -> Option<DeveloperRouteBookmark> {
    const REPLAY_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );
    const ROM_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    const SANCTUARY_CHECKPOINT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.cache/replay-bisect/rust-frame-12000.sav"
    );
    const LATE_CHECKPOINT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.cache/replay-bisect/rust-frame-1045813.sav"
    );
    match id {
        "preset-route-start" => Some(DeveloperRouteBookmark {
            id: "preset-route-start",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: None,
            target_frame: 0,
        }),
        "route-start" => Some(DeveloperRouteBookmark {
            id: "route-start",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: None,
            target_frame: 0,
        }),
        "preset-sanctuary" => Some(DeveloperRouteBookmark {
            id: "preset-sanctuary",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: Some(SANCTUARY_CHECKPOINT_PATH),
            target_frame: 12_000,
        }),
        "route-file-select" => Some(DeveloperRouteBookmark {
            id: "route-file-select",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: Some(SANCTUARY_CHECKPOINT_PATH),
            target_frame: 12_000,
        }),
        "preset-late-dungeon" => Some(DeveloperRouteBookmark {
            id: "preset-late-dungeon",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: Some(LATE_CHECKPOINT_PATH),
            target_frame: 1_045_813,
        }),
        "route-late-checkpoint" => Some(DeveloperRouteBookmark {
            id: "route-late-checkpoint",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            checkpoint_path: Some(LATE_CHECKPOINT_PATH),
            target_frame: 1_045_813,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use platform::{DeveloperDestinationKind, DeveloperDestinationStatus};

    #[test]
    fn destination_manifest_has_verified_route_bookmarks() {
        let destinations = developer_destinations();
        assert!(destinations.iter().any(|destination| {
            destination.id == "route-start"
                && destination.status == DeveloperDestinationStatus::Verified
        }));
        assert!(destinations.iter().any(|destination| {
            destination.id == "unverified-room-browser"
                && destination.status == DeveloperDestinationStatus::Locked
        }));
    }

    #[test]
    fn verified_route_destinations_resolve_to_bookmarks() {
        let destinations = developer_destinations();
        for destination in destinations
            .iter()
            .filter(|destination| destination.status == DeveloperDestinationStatus::Verified)
        {
            assert!(
                destination_target(destination.id).is_some(),
                "missing verified target for {}",
                destination.id
            );
        }
        assert!(route_bookmark("unverified-room-browser").is_none());
        assert!(destination_target("unverified-room-browser").is_none());
    }

    #[test]
    fn developer_sandbox_is_a_curated_synthetic_room_target() {
        let destinations = developer_destinations();
        let sandbox = destinations
            .iter()
            .find(|destination| destination.id == "preset-dev-sandbox")
            .expect("developer sandbox preset should be listed");

        assert_eq!(sandbox.kind, DeveloperDestinationKind::CuratedPreset);
        assert_eq!(sandbox.status, DeveloperDestinationStatus::Verified);
        assert_eq!(sandbox.location, "ROOM 01FF");
        assert_eq!(sandbox.thumbnail, DeveloperThumbnail::DevRoom);
        assert!(route_bookmark(sandbox.id).is_none());
        assert!(matches!(
            destination_target(sandbox.id),
            Some(DeveloperDestinationTarget::SyntheticRoom(room))
                if room.id == "preset-dev-sandbox"
                    && room.room_id == 0x01ff
                    && room.visual_theme == DeveloperSyntheticRoomTheme::Kakariko
                    && room.music_track == 0x07
        ));
    }

    #[test]
    fn every_curated_preset_has_thumbnail_and_short_detail_lines() {
        let destinations = developer_destinations();
        let presets: Vec<_> = destinations
            .iter()
            .filter(|destination| {
                destination.kind == platform::DeveloperDestinationKind::CuratedPreset
            })
            .collect();
        assert_eq!(presets.len(), 4);

        for preset in presets {
            assert!(destination_target(preset.id).is_some());
            assert_ne!(preset.menu_label, "");
            assert_ne!(preset.location, "");
            assert_ne!(preset.detail, "");
            assert_ne!(preset.provenance, "");
            assert!(
                preset.menu_label.len() <= 13,
                "{} menu label is too wide for the thumbnail detail panel",
                preset.id
            );
            assert!(
                preset.location.len() <= 13,
                "{} location is too wide for the thumbnail detail panel",
                preset.id
            );
            assert!(
                preset.detail.len() <= 13,
                "{} detail is too wide for the thumbnail detail panel",
                preset.id
            );
            assert!(
                preset.provenance.len() <= 13,
                "{} provenance is too wide for the thumbnail detail panel",
                preset.id
            );
        }
    }
}
