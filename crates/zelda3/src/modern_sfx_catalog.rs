#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernSfxWaveform {
    Pulse,
    Saw,
    Triangle,
    Noise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSfxEnvelope {
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSfxPitchSlide {
    pub target_pitch: u8,
    pub frames: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSfxStep {
    pub voice: u8,
    pub pitch: u8,
    pub instrument: u8,
    pub waveform: ModernSfxWaveform,
    pub volume: u8,
    pub envelope: ModernSfxEnvelope,
    pub duration_frames: u8,
    pub pitch_slide: Option<ModernSfxPitchSlide>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSfxProgram {
    pub bank: u8,
    pub id: u8,
    pub name: &'static str,
    pub steps: &'static [ModernSfxStep],
}

const QUICK: ModernSfxEnvelope = ModernSfxEnvelope {
    attack: 1,
    decay: 2,
    sustain: 8,
    release: 2,
};
const HIT: ModernSfxEnvelope = ModernSfxEnvelope {
    attack: 1,
    decay: 6,
    sustain: 7,
    release: 3,
};
const CHIME: ModernSfxEnvelope = ModernSfxEnvelope {
    attack: 1,
    decay: 3,
    sustain: 11,
    release: 4,
};

const MENU_CURSOR_STEPS: &[ModernSfxStep] = &[ModernSfxStep {
    voice: 1,
    pitch: 66,
    instrument: 1,
    waveform: ModernSfxWaveform::Pulse,
    volume: 82,
    envelope: QUICK,
    duration_frames: 4,
    pitch_slide: Some(ModernSfxPitchSlide {
        target_pitch: 72,
        frames: 3,
    }),
}];

const SWORD_STEPS: &[ModernSfxStep] = &[ModernSfxStep {
    voice: 1,
    pitch: 58,
    instrument: 3,
    waveform: ModernSfxWaveform::Noise,
    volume: 116,
    envelope: HIT,
    duration_frames: 7,
    pitch_slide: Some(ModernSfxPitchSlide {
        target_pitch: 47,
        frames: 6,
    }),
}];

const RUPEE_STEPS: &[ModernSfxStep] = &[
    ModernSfxStep {
        voice: 1,
        pitch: 76,
        instrument: 2,
        waveform: ModernSfxWaveform::Triangle,
        volume: 92,
        envelope: CHIME,
        duration_frames: 5,
        pitch_slide: None,
    },
    ModernSfxStep {
        voice: 2,
        pitch: 83,
        instrument: 2,
        waveform: ModernSfxWaveform::Triangle,
        volume: 70,
        envelope: CHIME,
        duration_frames: 5,
        pitch_slide: None,
    },
];

const DOOR_STEPS: &[ModernSfxStep] = &[ModernSfxStep {
    voice: 1,
    pitch: 42,
    instrument: 0,
    waveform: ModernSfxWaveform::Saw,
    volume: 100,
    envelope: ModernSfxEnvelope {
        attack: 1,
        decay: 7,
        sustain: 6,
        release: 5,
    },
    duration_frames: 10,
    pitch_slide: Some(ModernSfxPitchSlide {
        target_pitch: 38,
        frames: 8,
    }),
}];

const DAMAGE_STEPS: &[ModernSfxStep] = &[ModernSfxStep {
    voice: 1,
    pitch: 49,
    instrument: 3,
    waveform: ModernSfxWaveform::Noise,
    volume: 120,
    envelope: HIT,
    duration_frames: 8,
    pitch_slide: Some(ModernSfxPitchSlide {
        target_pitch: 53,
        frames: 4,
    }),
}];

const PROGRAMS: &[ModernSfxProgram] = &[
    ModernSfxProgram {
        bank: 0,
        id: 0x01,
        name: "menu_cursor",
        steps: MENU_CURSOR_STEPS,
    },
    ModernSfxProgram {
        bank: 0,
        id: 0x22,
        name: "sword",
        steps: SWORD_STEPS,
    },
    ModernSfxProgram {
        bank: 0,
        id: 0x2b,
        name: "rupee_pickup",
        steps: RUPEE_STEPS,
    },
    ModernSfxProgram {
        bank: 0,
        id: 0x34,
        name: "door_or_stairs",
        steps: DOOR_STEPS,
    },
    ModernSfxProgram {
        bank: 0,
        id: 0x88,
        name: "damage",
        steps: DAMAGE_STEPS,
    },
];

pub fn lookup_sfx_program(bank: u8, id: u8) -> Option<&'static ModernSfxProgram> {
    PROGRAMS
        .iter()
        .find(|program| program.bank == bank && program.id == id)
}

pub fn sfx_program_hash(program: &ModernSfxProgram) -> u32 {
    let mut hash = FNV1A32_OFFSET;
    hash = fnv1a32_byte(hash, program.bank);
    hash = fnv1a32_byte(hash, program.id);
    for byte in program.name.as_bytes() {
        hash = fnv1a32_byte(hash, *byte);
    }
    for step in program.steps {
        hash = fnv1a32_byte(hash, step.voice);
        hash = fnv1a32_byte(hash, step.pitch);
        hash = fnv1a32_byte(hash, step.instrument);
        hash = fnv1a32_byte(hash, waveform_tag(step.waveform));
        hash = fnv1a32_byte(hash, step.volume);
        hash = fnv1a32_byte(hash, step.envelope.attack);
        hash = fnv1a32_byte(hash, step.envelope.decay);
        hash = fnv1a32_byte(hash, step.envelope.sustain);
        hash = fnv1a32_byte(hash, step.envelope.release);
        hash = fnv1a32_byte(hash, step.duration_frames);
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
    PROGRAMS.len()
}

fn waveform_tag(waveform: ModernSfxWaveform) -> u8 {
    match waveform {
        ModernSfxWaveform::Pulse => 0,
        ModernSfxWaveform::Saw => 1,
        ModernSfxWaveform::Triangle => 2,
        ModernSfxWaveform::Noise => 3,
    }
}

const FNV1A32_OFFSET: u32 = 2166136261;
const FNV1A32_PRIME: u32 = 16777619;

fn fnv1a32_byte(hash: u32, byte: u8) -> u32 {
    (hash ^ u32::from(byte)).wrapping_mul(FNV1A32_PRIME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_initial_named_sfx_programs() {
        assert!(catalog_program_count() >= 5);
        assert_eq!(lookup_sfx_program(0, 0x01).unwrap().name, "menu_cursor");
        assert_eq!(lookup_sfx_program(0, 0x22).unwrap().name, "sword");
        assert_eq!(lookup_sfx_program(0, 0x2b).unwrap().name, "rupee_pickup");
        assert_eq!(lookup_sfx_program(0, 0x34).unwrap().name, "door_or_stairs");
        assert_eq!(lookup_sfx_program(0, 0x88).unwrap().name, "damage");
    }

    #[test]
    fn program_hash_changes_with_program_identity() {
        let sword = lookup_sfx_program(0, 0x22).unwrap();
        let rupee = lookup_sfx_program(0, 0x2b).unwrap();

        assert_ne!(sfx_program_hash(sword), 0);
        assert_ne!(sfx_program_hash(sword), sfx_program_hash(rupee));
    }
}
