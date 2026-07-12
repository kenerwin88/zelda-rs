#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExactSfxDspStep {
    pub bank: u8,
    pub id: u8,
    pub variant_hash: u32,
    pub step: u8,
    pub voice: u8,
    pub pitch: u8,
    pub instrument: u8,
    pub volume: u8,
    pub pan: i8,
    pub duration_frames: u8,
    pub echo: bool,
    pub command_delay_frames: u8,
    pub scheduler_tick_index: u8,
    pub dsp_pitch: u16,
    pub volume_left: i8,
    pub volume_right: i8,
    pub adsr1: u8,
    pub adsr2: u8,
    pub gain: u8,
    pub sample_offset: u16,
    #[serde(default)]
    pub duration_samples: u32,
}

include!(concat!(env!("OUT_DIR"), "/modern_sfx_dsp_assets.rs"));

pub fn exact_sfx_dsp_step(
    bank: u8,
    id: u8,
    variant_hash: u32,
    step: usize,
    shape: crate::modern_sfx_catalog::ModernSfxStep,
) -> Option<ExactSfxDspStep> {
    if let Some(exact) = EXACT_SFX_DSP_STEPS.iter().copied().find(|candidate| {
        candidate.bank == bank
            && candidate.id == id
            && candidate.variant_hash == variant_hash
            && usize::from(candidate.step) == step
    }) {
        return Some(exact);
    }

    let mut matches = EXACT_SFX_DSP_STEPS.iter().copied().filter(|candidate| {
        candidate.bank == bank
            && candidate.id == id
            && candidate.voice == shape.voice
            && candidate.pitch == shape.pitch
            && candidate.instrument == shape.instrument
            && candidate.volume == shape.volume
            && candidate.pan == shape.pan
            && candidate.duration_frames == shape.duration_frames
            && candidate.echo == shape.echo
    });
    let first = matches.next()?;
    if matches.all(|candidate| candidate.same_render_parameters(first)) {
        Some(first)
    } else {
        None
    }
}

impl ExactSfxDspStep {
    fn same_render_parameters(self, other: Self) -> bool {
        self.dsp_pitch == other.dsp_pitch
            && self.command_delay_frames == other.command_delay_frames
            && self.scheduler_tick_index == other.scheduler_tick_index
            && self.volume_left == other.volume_left
            && self.volume_right == other.volume_right
            && self.adsr1 == other.adsr1
            && self.adsr2 == other.adsr2
            && self.gain == other.gain
            && self.sample_offset == other.sample_offset
            && self.duration_samples == other.duration_samples
    }
}
