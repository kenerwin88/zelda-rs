#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModernMusicGlobalEvent {
    pub track: u8,
    pub start_frame: u16,
    pub sample_offset: u16,
    pub register: u8,
    pub value: u8,
}

include!(concat!(env!("OUT_DIR"), "/modern_music_global_assets.rs"));

pub fn events_at(track: u8, start_frame: u16) -> impl Iterator<Item = ModernMusicGlobalEvent> {
    let key = (track, start_frame);
    let start = MUSIC_GLOBAL_EVENTS.partition_point(|event| (event.track, event.start_frame) < key);
    let end = MUSIC_GLOBAL_EVENTS.partition_point(|event| (event.track, event.start_frame) <= key);
    MUSIC_GLOBAL_EVENTS[start..end].iter().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_events_are_sorted_for_indexed_lookup() {
        assert!(MUSIC_GLOBAL_EVENTS.windows(2).all(|events| {
            (events[0].track, events[0].start_frame) <= (events[1].track, events[1].start_frame)
        }));
    }

    #[test]
    fn indexed_lookup_returns_only_the_requested_frame() {
        let expected = MUSIC_GLOBAL_EVENTS
            .iter()
            .copied()
            .find(|event| event.track == 3)
            .expect("house music has global events");
        let events = events_at(expected.track, expected.start_frame).collect::<Vec<_>>();

        assert!(!events.is_empty());
        assert!(events.iter().all(|event| {
            event.track == expected.track && event.start_frame == expected.start_frame
        }));
    }
}
