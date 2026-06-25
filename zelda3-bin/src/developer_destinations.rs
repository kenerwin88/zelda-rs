use platform::DeveloperDestination;

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
}
