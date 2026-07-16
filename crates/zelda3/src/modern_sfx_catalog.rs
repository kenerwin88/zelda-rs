use std::sync::OnceLock;

pub const MODERN_SFX_ASSET_FORMAT: &str = "zelda3_modern_sfx_assets_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModernSfxWaveform {
    Pulse,
    Saw,
    Triangle,
    Noise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernSfxEnvelope {
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernSfxPitchSlide {
    pub target_pitch: u8,
    pub frames: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernSfxStep {
    pub voice: u8,
    pub pitch: u8,
    pub instrument: u8,
    pub waveform: ModernSfxWaveform,
    pub volume: u8,
    pub pan: i8,
    pub echo: bool,
    pub envelope: ModernSfxEnvelope,
    pub duration_frames: u8,
    pub pitch_slide: Option<ModernSfxPitchSlide>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSfxProgram {
    pub bank: u8,
    pub id: u8,
    pub variant: u8,
    pub variant_hash: u32,
    pub name: &'static str,
    pub context: ModernSfxContextSignature,
    pub steps: &'static [ModernSfxStep],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernSfxContextSignature {
    pub source_slot: u8,
    pub voice_mask: u8,
    pub context_voice_mask: u8,
    pub step_count: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModernSfxRuntimeContext {
    pub source_slot: u8,
    pub active_voice_mask: u8,
}

#[derive(serde::Deserialize)]
struct SfxAssetsWire {
    format: String,
    programs: Vec<SfxProgramWire>,
    exact_dsp_steps: Vec<crate::modern_sfx_dsp_catalog::ExactSfxDspStep>,
    pitch_events: Vec<crate::modern_sfx_pitch_catalog::ExactSfxPitchEvent>,
}

#[derive(serde::Deserialize)]
struct SfxProgramWire {
    bank: u8,
    id: u8,
    variant: u8,
    variant_hash: u32,
    name: String,
    promotion_status: String,
    context: ModernSfxContextSignature,
    steps: Vec<SfxStepWire>,
}

#[derive(serde::Deserialize)]
struct SfxStepWire {
    voice: u8,
    pitch: u8,
    instrument: u8,
    waveform: String,
    volume: u8,
    pan: i8,
    echo: bool,
    envelope: ModernSfxEnvelope,
    duration_frames: u8,
    pitch_slide: Option<ModernSfxPitchSlide>,
}

struct ModernSfxAssets {
    programs: &'static [ModernSfxProgram],
    exact_dsp_steps: Box<[crate::modern_sfx_dsp_catalog::ExactSfxDspStep]>,
    pitch_events: Box<[crate::modern_sfx_pitch_catalog::ExactSfxPitchEvent]>,
}

static ASSETS: OnceLock<ModernSfxAssets> = OnceLock::new();
const PACKED_ASSETS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/modern_sfx_assets.bin"));

fn assets() -> &'static ModernSfxAssets {
    ASSETS.get_or_init(|| {
        let wire: SfxAssetsWire = bincode::deserialize(PACKED_ASSETS)
            .expect("build-validated modern SFX assets must decode");
        assert_eq!(wire.format, MODERN_SFX_ASSET_FORMAT);
        let programs = wire
            .programs
            .into_iter()
            .map(|program| {
                assert_eq!(program.promotion_status, "review_ready");
                let steps = Box::leak(
                    program
                        .steps
                        .into_iter()
                        .map(|step| ModernSfxStep {
                            voice: step.voice,
                            pitch: step.pitch,
                            instrument: step.instrument,
                            waveform: match step.waveform.as_str() {
                                "Pulse" => ModernSfxWaveform::Pulse,
                                "Saw" => ModernSfxWaveform::Saw,
                                "Triangle" => ModernSfxWaveform::Triangle,
                                "Noise" => ModernSfxWaveform::Noise,
                                _ => unreachable!("waveform validated by build script"),
                            },
                            volume: step.volume,
                            pan: step.pan,
                            echo: step.echo,
                            envelope: step.envelope,
                            duration_frames: step.duration_frames,
                            pitch_slide: step.pitch_slide,
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                );
                ModernSfxProgram {
                    bank: program.bank,
                    id: program.id,
                    variant: program.variant,
                    variant_hash: program.variant_hash,
                    name: Box::leak(program.name.into_boxed_str()),
                    context: program.context,
                    steps,
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        ModernSfxAssets {
            // The old catalog was static data. Retaining this decoded arena for
            // the process lifetime preserves that public reference contract.
            programs: Box::leak(programs),
            exact_dsp_steps: wire.exact_dsp_steps.into_boxed_slice(),
            pitch_events: wire.pitch_events.into_boxed_slice(),
        }
    })
}

pub(crate) fn exact_dsp_steps() -> &'static [crate::modern_sfx_dsp_catalog::ExactSfxDspStep] {
    &assets().exact_dsp_steps
}

pub(crate) fn exact_pitch_events() -> &'static [crate::modern_sfx_pitch_catalog::ExactSfxPitchEvent]
{
    &assets().pitch_events
}

fn programs() -> &'static [ModernSfxProgram] {
    assets().programs
}

pub fn lookup_sfx_program(bank: u8, id: u8) -> Option<&'static ModernSfxProgram> {
    if let Some(program) = programs().iter().find(|program| {
        program.bank == bank && program.id == id && program.context.context_voice_mask == 0
    }) {
        return Some(program);
    }
    lookup_sfx_program_for_context(
        bank,
        id,
        ModernSfxRuntimeContext {
            source_slot: bank,
            active_voice_mask: 0,
        },
    )
}

#[cfg(test)]
pub(crate) fn conformance_commands() -> Vec<(u8, u8)> {
    let mut commands = programs()
        .iter()
        .map(|program| (program.bank, program.id))
        .collect::<Vec<_>>();
    commands.sort_unstable();
    commands.dedup();
    commands
}

pub fn lookup_sfx_program_for_context(
    bank: u8,
    id: u8,
    context: ModernSfxRuntimeContext,
) -> Option<&'static ModernSfxProgram> {
    select_sfx_program_from(programs(), bank, id, context)
}

fn select_sfx_program_from<'a>(
    programs: &'a [ModernSfxProgram],
    bank: u8,
    id: u8,
    context: ModernSfxRuntimeContext,
) -> Option<&'a ModernSfxProgram> {
    programs
        .iter()
        .filter(|program| program.bank == bank && program.id == id)
        .max_by_key(|program| program_context_score(program, context))
}

fn program_context_score(program: &ModernSfxProgram, context: ModernSfxRuntimeContext) -> u16 {
    let mut score = 1u16;
    if program.context.source_slot == context.source_slot {
        score += 64;
    }
    if program.context.context_voice_mask == 0 {
        score += 2;
    } else if program.context.context_voice_mask == context.active_voice_mask {
        score += 32;
    } else if (program.context.context_voice_mask & context.active_voice_mask)
        == program.context.context_voice_mask
    {
        score += 16;
    }
    if program.context.voice_mask != 0
        && (program.context.voice_mask & context.active_voice_mask) == 0
    {
        score += 8;
    }
    if program.context.step_count != 0 {
        score += u16::from(program.context.step_count.min(8));
    }
    score
}

pub fn sfx_program_hash(program: &ModernSfxProgram) -> u32 {
    let mut hash = 2166136261;
    hash = fnv1a32_byte(hash, program.bank);
    hash = fnv1a32_byte(hash, program.id);
    for byte in program.name.as_bytes() {
        hash = fnv1a32_byte(hash, *byte);
    }
    for step in program.steps {
        for byte in [
            step.voice,
            step.pitch,
            step.instrument,
            waveform_tag(step.waveform),
            step.volume,
            step.pan as u8,
            u8::from(step.echo),
            step.envelope.attack,
            step.envelope.decay,
            step.envelope.sustain,
            step.envelope.release,
            step.duration_frames,
        ] {
            hash = fnv1a32_byte(hash, byte);
        }
        if let Some(slide) = step.pitch_slide {
            hash = fnv1a32_byte(hash, 1);
            hash = fnv1a32_byte(hash, slide.target_pitch);
            hash = fnv1a32_byte(hash, slide.frames);
        } else {
            hash = fnv1a32_byte(hash, 0);
        }
    }
    hash
}

pub fn catalog_program_count() -> usize {
    programs().len()
}
fn waveform_tag(w: ModernSfxWaveform) -> u8 {
    match w {
        ModernSfxWaveform::Pulse => 0,
        ModernSfxWaveform::Saw => 1,
        ModernSfxWaveform::Triangle => 2,
        ModernSfxWaveform::Noise => 3,
    }
}
fn fnv1a32_byte(hash: u32, byte: u8) -> u32 {
    (hash ^ u32::from(byte)).wrapping_mul(16777619)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ENVELOPE: ModernSfxEnvelope = ModernSfxEnvelope {
        attack: 1,
        decay: 2,
        sustain: 8,
        release: 2,
    };
    #[test]
    fn canonical_asset_catalog_is_complete() {
        assert_eq!(MODERN_SFX_ASSET_FORMAT, "zelda3_modern_sfx_assets_v1");
        assert_eq!(catalog_program_count(), 342);
        assert_eq!(exact_dsp_steps().len(), 573);
        assert_eq!(exact_pitch_events().len(), 170);
        for (bank, id, name) in [
            (0, 0x01, "menu_cursor"),
            (0, 0x22, "sword"),
            (0, 0x2b, "rupee_pickup"),
            (0, 0x34, "door_or_stairs"),
            (0, 0x88, "damage"),
        ] {
            assert_eq!(lookup_sfx_program(bank, id).unwrap().name, name);
        }
    }
    #[test]
    fn pinned_hashes_survive_asset_compilation() {
        assert_eq!(
            sfx_program_hash(lookup_sfx_program(0, 0x03).unwrap()),
            0x8a30d42c
        );
        assert_eq!(
            sfx_program_hash(lookup_sfx_program(2, 0x0a).unwrap()),
            0x71c0b04e
        );
        assert_eq!(
            sfx_program_hash(lookup_sfx_program(2, 0x24).unwrap()),
            0x308b7cca
        );
    }
    #[test]
    fn contextual_variants_survive_asset_compilation() {
        for (mask, hash) in [
            (0x1f, 0xe8a9cbf4),
            (0x11, 0x6b6a1520),
            (0x3f, 0xa1d0e0bb),
            (0x28, 0x83cc46a8),
        ] {
            assert_eq!(
                lookup_sfx_program_for_context(
                    1,
                    0x2c,
                    ModernSfxRuntimeContext {
                        source_slot: 1,
                        active_voice_mask: mask
                    }
                )
                .unwrap()
                .variant_hash,
                hash
            );
        }
    }

    #[test]
    fn exact_runtime_context_selects_the_route_2b_variant() {
        let selected = lookup_sfx_program_for_context(
            1,
            0x2b,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x7f,
            },
        )
        .unwrap();
        assert_eq!(selected.variant_hash, 0x78dac40b);
    }

    #[test]
    fn program_hash_changes_with_program_identity() {
        let sword = lookup_sfx_program(0, 0x22).unwrap();
        let rupee = lookup_sfx_program(0, 0x2b).unwrap();
        assert_ne!(sfx_program_hash(sword), 0);
        assert_ne!(sfx_program_hash(sword), sfx_program_hash(rupee));
    }

    #[test]
    fn repeating_bank_two_effect_keeps_full_slide_for_retrigger_interruption() {
        let step = lookup_sfx_program(2, 0x0c).unwrap().steps[0];
        assert_eq!(step.pitch, 42);
        assert_eq!(step.duration_frames, 3);
        assert_eq!(
            step.pitch_slide,
            Some(ModernSfxPitchSlide {
                target_pitch: 47,
                frames: 3,
            })
        );
    }

    #[test]
    fn route_echo_variant_requires_its_observed_active_voice_context() {
        let dry = lookup_sfx_program_for_context(
            1,
            0x09,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0,
            },
        )
        .unwrap();
        let echoed = lookup_sfx_program_for_context(
            1,
            0x09,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x06,
            },
        )
        .unwrap();
        assert!(!dry.steps[0].echo);
        assert!(echoed.steps[0].echo);
    }

    #[test]
    fn active_owner_selects_observed_suppressed_retrigger_variant() {
        let audible = lookup_sfx_program_for_context(
            1,
            0x45,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0,
            },
        )
        .unwrap();
        let suppressed = lookup_sfx_program_for_context(
            1,
            0x45,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x80,
            },
        )
        .unwrap();
        assert!(!audible.steps.is_empty());
        assert!(suppressed.steps.is_empty());
    }

    #[test]
    fn context_lookup_selects_best_matching_variant() {
        const STEPS: &[ModernSfxStep] = &[ModernSfxStep {
            voice: 1,
            pitch: 40,
            instrument: 2,
            waveform: ModernSfxWaveform::Pulse,
            volume: 64,
            pan: 0,
            echo: false,
            envelope: TEST_ENVELOPE,
            duration_frames: 3,
            pitch_slide: None,
        }];
        const TEST_PROGRAMS: &[ModernSfxProgram] = &[
            ModernSfxProgram {
                bank: 1,
                id: 0x2c,
                variant: 0,
                variant_hash: 0x11111111,
                name: "fallback",
                context: ModernSfxContextSignature {
                    source_slot: 1,
                    voice_mask: 2,
                    context_voice_mask: 0,
                    step_count: 1,
                },
                steps: STEPS,
            },
            ModernSfxProgram {
                bank: 1,
                id: 0x2c,
                variant: 1,
                variant_hash: 0x22222222,
                name: "contextual",
                context: ModernSfxContextSignature {
                    source_slot: 1,
                    voice_mask: 2,
                    context_voice_mask: 0x20,
                    step_count: 1,
                },
                steps: STEPS,
            },
        ];
        let selected = select_sfx_program_from(
            TEST_PROGRAMS,
            1,
            0x2c,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x20,
            },
        )
        .unwrap();
        assert_eq!(selected.variant_hash, 0x22222222);
    }

    #[test]
    fn exact_voice_context_beats_a_later_subset_variant() {
        let selected = lookup_sfx_program_for_context(
            1,
            0x2c,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x1f,
            },
        )
        .unwrap();
        assert_eq!(selected.variant_hash, 0xe8a9cbf4);
    }
}
