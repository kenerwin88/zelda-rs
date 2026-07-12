#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExactSfxPitchEvent {
    pub bank: u8,
    pub id: u8,
    pub variant_hash: u32,
    pub step: u8,
    pub relative_sample: u16,
    pub pitch_word: u16,
}

include!(concat!(env!("OUT_DIR"), "/modern_sfx_pitch_assets.rs"));

pub fn pitch_events(
    bank: u8,
    id: u8,
    variant_hash: u32,
    step: usize,
) -> impl Iterator<Item = ExactSfxPitchEvent> {
    EXACT_SFX_PITCH_EVENTS.iter().copied().filter(move |event| {
        event.bank == bank
            && event.id == id
            && (event.variant_hash == 0 || event.variant_hash == variant_hash)
            && usize::from(event.step) == step
    })
}
