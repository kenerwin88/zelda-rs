use platform::DeveloperDestination;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeveloperRouteBookmark {
    pub id: &'static str,
    pub replay_path: &'static str,
    pub rom_path: &'static str,
    pub target_frame: u32,
}

pub fn developer_destinations() -> Vec<DeveloperDestination> {
    vec![
        DeveloperDestination::verified(
            "route-start",
            "Route Start",
            "saves/zelda3-combined-route.sav frame 0",
        ),
        DeveloperDestination::verified(
            "route-file-select",
            "File Select",
            "standard route first menu segment",
        ),
        DeveloperDestination::verified(
            "route-late-checkpoint",
            "Late Route Checkpoint",
            "standard route checkpoint frame 1045813",
        ),
        DeveloperDestination::locked(
            "unverified-overworld-browser",
            "Overworld Browser",
            "requires verified overworld initializer",
        ),
        DeveloperDestination::locked(
            "unverified-room-browser",
            "Dungeon Room Browser",
            "requires verified dungeon room initializer",
        ),
    ]
}

pub fn route_bookmark(id: &str) -> Option<DeveloperRouteBookmark> {
    const REPLAY_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );
    const ROM_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    match id {
        "route-start" => Some(DeveloperRouteBookmark {
            id: "route-start",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            target_frame: 0,
        }),
        "route-file-select" => Some(DeveloperRouteBookmark {
            id: "route-file-select",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            target_frame: 12_000,
        }),
        "route-late-checkpoint" => Some(DeveloperRouteBookmark {
            id: "route-late-checkpoint",
            replay_path: REPLAY_PATH,
            rom_path: ROM_PATH,
            target_frame: 1_045_813,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use platform::DeveloperDestinationStatus;

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
                route_bookmark(destination.id).is_some(),
                "missing route bookmark for {}",
                destination.id
            );
        }
        assert!(route_bookmark("unverified-room-browser").is_none());
    }
}
