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
    MUSIC_GLOBAL_EVENTS
        .iter()
        .copied()
        .filter(move |event| event.track == track && event.start_frame == start_frame)
}
