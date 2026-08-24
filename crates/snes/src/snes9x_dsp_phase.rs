//! Clock-by-clock port of the pinned Snes9x `SPC_DSP` pipeline.
//!
//! This module deliberately owns no SMP or APU scheduling.  A caller supplies
//! the shared 64 KiB SPC RAM and advances exactly one DSP clock at a time.
//! The phase order and intermediate latches mirror `SPC_DSP.cpp`'s
//! `GEN_DSP_TIMING` table, making the entire 32-clock pipeline checkpointable.

use crate::apu::dsp_gaussian_interpolate;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

const REGISTER_COUNT: usize = 128;
const VOICE_COUNT: usize = 8;
const BRR_BUFFER_SIZE: usize = 12;
const BRR_BLOCK_SIZE: u16 = 9;
const ECHO_HISTORY_SIZE: usize = 8;
const SIMPLE_COUNTER_RANGE: i32 = 2048 * 5 * 3;

const MVOLL: usize = 0x0c;
const MVOLR: usize = 0x1c;
const EVOLL: usize = 0x2c;
const EVOLR: usize = 0x3c;
const KON: usize = 0x4c;
const KOFF: usize = 0x5c;
const FLG: usize = 0x6c;
const ENDX: usize = 0x7c;
const EFB: usize = 0x0d;
const PMON: usize = 0x2d;
const NON: usize = 0x3d;
const EON: usize = 0x4d;
const DIR: usize = 0x5d;
const ESA: usize = 0x6d;
const EDL: usize = 0x7d;
const FIR: usize = 0x0f;

const VOLL: usize = 0x00;
const VOLR: usize = 0x01;
const PITCHL: usize = 0x02;
const PITCHH: usize = 0x03;
const SRCN: usize = 0x04;
const ADSR0: usize = 0x05;
const ADSR1: usize = 0x06;
const GAIN: usize = 0x07;
const ENVX: usize = 0x08;
const OUTX: usize = 0x09;

const INITIAL_REGISTERS: [u8; REGISTER_COUNT] = [
    0x45, 0x8b, 0x5a, 0x9a, 0xe4, 0x82, 0x1b, 0x78, 0x00, 0x00, 0xaa, 0x96, 0x89, 0x0e, 0xe0, 0x80,
    0x2a, 0x49, 0x3d, 0xba, 0x14, 0xa0, 0xac, 0xc5, 0x00, 0x00, 0x51, 0xbb, 0x9c, 0x4e, 0x7b, 0xff,
    0xf4, 0xfd, 0x57, 0x32, 0x37, 0xd9, 0x42, 0x22, 0x00, 0x00, 0x5b, 0x3c, 0x9f, 0x1b, 0x87, 0x9a,
    0x6f, 0x27, 0xaf, 0x7b, 0xe5, 0x68, 0x0a, 0xd9, 0x00, 0x00, 0x9a, 0xc5, 0x9c, 0x4e, 0x7b, 0xff,
    0xea, 0x21, 0x78, 0x4f, 0xdd, 0xed, 0x24, 0x14, 0x00, 0x00, 0x77, 0xb1, 0xd1, 0x36, 0xc1, 0x67,
    0x52, 0x57, 0x46, 0x3d, 0x59, 0xf4, 0x87, 0xa4, 0x00, 0x00, 0x7e, 0x44, 0x00, 0x4e, 0x7b, 0xff,
    0x75, 0xf5, 0x06, 0x97, 0x10, 0xc3, 0x24, 0xbb, 0x00, 0x00, 0x7b, 0x7a, 0xe0, 0x60, 0x12, 0x0f,
    0xf7, 0x74, 0x1c, 0xe5, 0x39, 0x3d, 0x73, 0xc1, 0x00, 0x00, 0x7a, 0xb3, 0xff, 0x4e, 0x7b, 0xff,
];

const COUNTER_RATES: [u32; 32] = [
    SIMPLE_COUNTER_RANGE as u32 + 1,
    2048,
    1536,
    1280,
    1024,
    768,
    640,
    512,
    384,
    320,
    256,
    192,
    160,
    128,
    96,
    80,
    64,
    48,
    40,
    32,
    24,
    20,
    16,
    12,
    10,
    8,
    6,
    5,
    4,
    3,
    2,
    1,
];

const COUNTER_OFFSETS: [u32; 32] = [
    1, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040,
    536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 0, 0,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Snes9xDspEnvelopeMode {
    #[default]
    Release = 0,
    Attack = 1,
    Decay = 2,
    Sustain = 3,
}

/// The pinned libretro oracle explicitly selects Snes9x's hardware Gaussian
/// interpolation (`DSP_INTERPOLATION_GAUSSIAN`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Snes9xDspInterpolationMode {
    #[default]
    Gaussian,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xDspVoiceState {
    /// Twelve decoded samples followed by the source's wrap-around copy.
    pub buffer: [i32; BRR_BUFFER_SIZE * 2],
    pub buffer_position: u8,
    pub interpolation_position: u16,
    pub brr_address: u16,
    pub brr_offset: u8,
    pub key_on_delay: u8,
    pub envelope_mode: Snes9xDspEnvelopeMode,
    pub envelope: i32,
    pub hidden_envelope: i32,
    pub envelope_output_latch: u8,
}

impl Default for Snes9xDspVoiceState {
    fn default() -> Self {
        Self {
            buffer: [0; BRR_BUFFER_SIZE * 2],
            buffer_position: 0,
            interpolation_position: 0,
            brr_address: 0,
            brr_offset: 1,
            key_on_delay: 0,
            envelope_mode: Snes9xDspEnvelopeMode::Release,
            envelope: 0,
            hidden_envelope: 0,
            envelope_output_latch: 0,
        }
    }
}

/// Complete mutable state of the pinned Snes9x 32-phase DSP pipeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xDspPhaseState {
    pub interpolation_mode: Snes9xDspInterpolationMode,
    /// Hardware pipeline registers. These intentionally differ from the
    /// externally readable register file during delayed ENVX/OUTX/ENDX writes.
    pub registers: Vec<u8>,
    pub external_registers: Vec<u8>,
    pub voices: [Snes9xDspVoiceState; VOICE_COUNT],
    pub echo_history: [[[i32; 2]; ECHO_HISTORY_SIZE]; 2],
    pub echo_history_position: u8,
    pub every_other_sample: u8,
    pub kon: u8,
    pub noise: i32,
    pub counter: i32,
    pub echo_offset: u16,
    pub echo_length: u16,
    pub phase: u8,
    pub kon_check: bool,
    pub new_kon: u8,
    pub endx_buffer: u8,
    pub envx_buffer: u8,
    pub outx_buffer: u8,
    pub pmon_latch: u8,
    pub non_latch: u8,
    pub eon_latch: u8,
    pub dir_latch: u8,
    pub koff_latch: u8,
    pub brr_next_address_latch: u16,
    pub adsr0_latch: u8,
    pub brr_header_latch: u8,
    pub brr_byte_latch: u8,
    pub srcn_latch: u8,
    pub esa_latch: u8,
    pub echo_enabled_latch: u8,
    pub dir_address: u16,
    pub pitch: i32,
    pub output: i32,
    pub looped: u8,
    pub echo_pointer: u16,
    pub main_output: [i32; 2],
    pub echo_output: [i32; 2],
    pub echo_input: [i32; 2],
    pub mute_mask: u8,
    pub stereo_switch: u16,
    pub separate_echo_buffer_enabled: bool,
    pub separate_echo_buffer: Vec<u8>,
}

/// Standalone source-exact DSP phase machine. It is not wired into `ApuState`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Snes9xDspPhaseMachine {
    state: Snes9xDspPhaseState,
    #[serde(skip)]
    observation: Option<Snes9xDspClockObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xDspSuboperation {
    pub kind: i8,
    pub voice: i8,
    pub argument: i8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Snes9xDspClockObservation {
    suboperations: Vec<Snes9xDspSuboperation>,
    branch_evaluated_mask: u64,
    branch_taken_mask: u64,
    branch_not_taken_mask: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xDspClockReceipt {
    pub phase: u8,
    pub suboperations: Vec<Snes9xDspSuboperation>,
    pub branch_evaluated_mask: u64,
    pub branch_taken_mask: u64,
    pub branch_not_taken_mask: u64,
    pub emitted_sample: Option<[i16; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xDspCompositeCheckpoint {
    dsp: Snes9xDspPhaseState,
    shared_apu_ram: Vec<u8>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Snes9xDspCheckpointError {
    #[error("DSP internal register file has length {actual}, expected 128")]
    InternalRegisterLength { actual: usize },
    #[error("DSP external register file has length {actual}, expected 128")]
    ExternalRegisterLength { actual: usize },
    #[error("DSP separate echo buffer has length {actual}, expected 65536")]
    SeparateEchoBufferLength { actual: usize },
    #[error("shared APU RAM has length {actual}, expected 65536")]
    SharedApuRamLength { actual: usize },
    #[error("DSP phase {phase} is outside 0..32")]
    Phase { phase: u8 },
    #[error("DSP echo history position {position} is outside 0..8")]
    EchoHistoryPosition { position: u8 },
    #[error("DSP voice {voice} BRR buffer position {position} is not 0, 4, or 8")]
    VoiceBufferPosition { voice: usize, position: u8 },
    #[error("DSP voice {voice} BRR offset {offset} is not 1, 3, 5, or 7")]
    VoiceBrrOffset { voice: usize, offset: u8 },
    #[error("DSP voice {voice} key-on delay {delay} is outside 0..=5")]
    VoiceKeyOnDelay { voice: usize, delay: u8 },
    #[error("DSP voice {voice} interpolation position {position:#x} exceeds $7fff")]
    VoiceInterpolationPosition { voice: usize, position: u16 },
    #[error("DSP voice {voice} BRR buffer mirror differs at sample {sample}")]
    VoiceBufferMirror { voice: usize, sample: usize },
    #[error("DSP echo history mirror differs at tap {tap}, channel {channel}")]
    EchoHistoryMirror { tap: usize, channel: usize },
}

impl Default for Snes9xDspPhaseMachine {
    fn default() -> Self {
        Self::power_on()
    }
}

impl<'de> Deserialize<'de> for Snes9xDspPhaseMachine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state = Snes9xDspPhaseState::deserialize(deserializer)?;
        Self::from_checkpoint(state).map_err(serde::de::Error::custom)
    }
}

impl Snes9xDspPhaseState {
    pub fn validate(&self) -> Result<(), Snes9xDspCheckpointError> {
        if self.registers.len() != REGISTER_COUNT {
            return Err(Snes9xDspCheckpointError::InternalRegisterLength {
                actual: self.registers.len(),
            });
        }
        if self.external_registers.len() != REGISTER_COUNT {
            return Err(Snes9xDspCheckpointError::ExternalRegisterLength {
                actual: self.external_registers.len(),
            });
        }
        if self.separate_echo_buffer.len() != 0x10000 {
            return Err(Snes9xDspCheckpointError::SeparateEchoBufferLength {
                actual: self.separate_echo_buffer.len(),
            });
        }
        if self.phase >= 32 {
            return Err(Snes9xDspCheckpointError::Phase { phase: self.phase });
        }
        if usize::from(self.echo_history_position) >= ECHO_HISTORY_SIZE {
            return Err(Snes9xDspCheckpointError::EchoHistoryPosition {
                position: self.echo_history_position,
            });
        }
        for (voice, state) in self.voices.iter().enumerate() {
            if !matches!(state.buffer_position, 0 | 4 | 8) {
                return Err(Snes9xDspCheckpointError::VoiceBufferPosition {
                    voice,
                    position: state.buffer_position,
                });
            }
            if !matches!(state.brr_offset, 1 | 3 | 5 | 7) {
                return Err(Snes9xDspCheckpointError::VoiceBrrOffset {
                    voice,
                    offset: state.brr_offset,
                });
            }
            if state.key_on_delay > 5 {
                return Err(Snes9xDspCheckpointError::VoiceKeyOnDelay {
                    voice,
                    delay: state.key_on_delay,
                });
            }
            if state.interpolation_position > 0x7fff {
                return Err(Snes9xDspCheckpointError::VoiceInterpolationPosition {
                    voice,
                    position: state.interpolation_position,
                });
            }
            for sample in 0..BRR_BUFFER_SIZE {
                if state.buffer[sample] != state.buffer[sample + BRR_BUFFER_SIZE] {
                    return Err(Snes9xDspCheckpointError::VoiceBufferMirror { voice, sample });
                }
            }
        }
        for tap in 0..ECHO_HISTORY_SIZE {
            for channel in 0..2 {
                if self.echo_history[0][tap][channel] != self.echo_history[1][tap][channel] {
                    return Err(Snes9xDspCheckpointError::EchoHistoryMirror { tap, channel });
                }
            }
        }
        Ok(())
    }
}

impl Snes9xDspCompositeCheckpoint {
    /// Captures the coherent emulation boundary: the DSP pipeline and the RAM
    /// it reads/writes. Whether an already-returned phase-27 sample has been
    /// published remains the audio sink owner's checkpoint responsibility.
    pub fn capture(machine: &Snes9xDspPhaseMachine, shared_apu_ram: &[u8; 0x10000]) -> Self {
        Self {
            dsp: machine.checkpoint(),
            shared_apu_ram: shared_apu_ram.to_vec(),
        }
    }

    pub fn restore(
        self,
    ) -> Result<(Snes9xDspPhaseMachine, Box<[u8; 0x10000]>), Snes9xDspCheckpointError> {
        let machine = Snes9xDspPhaseMachine::from_checkpoint(self.dsp)?;
        if self.shared_apu_ram.len() != 0x10000 {
            return Err(Snes9xDspCheckpointError::SharedApuRamLength {
                actual: self.shared_apu_ram.len(),
            });
        }
        let ram =
            self.shared_apu_ram
                .into_boxed_slice()
                .try_into()
                .map_err(
                    |value: Box<[u8]>| Snes9xDspCheckpointError::SharedApuRamLength {
                        actual: value.len(),
                    },
                )?;
        Ok((machine, ram))
    }
}

impl Snes9xDspPhaseMachine {
    pub fn state(&self) -> &Snes9xDspPhaseState {
        &self.state
    }

    pub fn interpolation_mode(&self) -> Snes9xDspInterpolationMode {
        self.state.interpolation_mode
    }

    pub fn checkpoint(&self) -> Snes9xDspPhaseState {
        self.state.clone()
    }

    pub fn from_checkpoint(state: Snes9xDspPhaseState) -> Result<Self, Snes9xDspCheckpointError> {
        state.validate()?;
        Ok(Self {
            state,
            observation: None,
        })
    }

    pub fn restore_checkpoint(
        &mut self,
        state: Snes9xDspPhaseState,
    ) -> Result<(), Snes9xDspCheckpointError> {
        state.validate()?;
        self.state = state;
        Ok(())
    }

    #[cfg(test)]
    fn from_snes9x_copy_state(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 642 {
            return Err(format!(
                "Snes9x DSP copy_state is {} bytes, expected 642",
                bytes.len()
            ));
        }
        let mut cursor = DspCopyStateReader { bytes, offset: 0 };
        let mut state = Self::power_on_unloaded();
        cursor.copy_into(&mut state.registers);
        for voice in &mut state.voices {
            for sample in 0..BRR_BUFFER_SIZE {
                let value = i32::from(cursor.i16());
                voice.buffer[sample] = value;
                voice.buffer[sample + BRR_BUFFER_SIZE] = value;
            }
            voice.interpolation_position = cursor.u16();
            voice.brr_address = cursor.u16();
            voice.envelope = i32::from(cursor.u16());
            voice.hidden_envelope = i32::from(cursor.i16());
            voice.buffer_position = cursor.u8();
            voice.brr_offset = cursor.u8();
            voice.key_on_delay = cursor.u8();
            voice.envelope_mode = match cursor.u8() {
                0 => Snes9xDspEnvelopeMode::Release,
                1 => Snes9xDspEnvelopeMode::Attack,
                2 => Snes9xDspEnvelopeMode::Decay,
                3 => Snes9xDspEnvelopeMode::Sustain,
                value => return Err(format!("invalid Snes9x envelope mode {value}")),
            };
            voice.envelope_output_latch = cursor.u8();
            if cursor.u8() != 0 {
                return Err("nonempty Snes9x voice copy_state extension".to_string());
            }
        }
        for tap in 0..ECHO_HISTORY_SIZE {
            for channel in 0..2 {
                let value = i32::from(cursor.i16());
                state.echo_history[0][tap][channel] = value;
                state.echo_history[1][tap][channel] = value;
            }
        }
        state.echo_history_position = 0;
        state.every_other_sample = cursor.u8();
        state.kon = cursor.u8();
        state.noise = i32::from(cursor.u16());
        state.counter = i32::from(cursor.u16());
        state.echo_offset = cursor.u16();
        state.echo_length = cursor.u16();
        state.phase = cursor.u8();
        state.new_kon = cursor.u8();
        state.endx_buffer = cursor.u8();
        state.envx_buffer = cursor.u8();
        state.outx_buffer = cursor.u8();
        state.pmon_latch = cursor.u8();
        state.non_latch = cursor.u8();
        state.eon_latch = cursor.u8();
        state.dir_latch = cursor.u8();
        state.koff_latch = cursor.u8();
        state.brr_next_address_latch = cursor.u16();
        state.adsr0_latch = cursor.u8();
        state.brr_header_latch = cursor.u8();
        state.brr_byte_latch = cursor.u8();
        state.srcn_latch = cursor.u8();
        state.esa_latch = cursor.u8();
        state.echo_enabled_latch = cursor.u8();
        for channel in 0..2 {
            state.main_output[channel] = i32::from(cursor.i16());
        }
        for channel in 0..2 {
            state.echo_output[channel] = i32::from(cursor.i16());
        }
        for channel in 0..2 {
            state.echo_input[channel] = i32::from(cursor.i16());
        }
        state.dir_address = cursor.u16();
        state.pitch = i32::from(cursor.u16());
        state.output = i32::from(cursor.i16());
        state.echo_pointer = cursor.u16();
        state.looped = cursor.u8();
        cursor.copy_into(&mut state.external_registers);
        if cursor.u8() != 0 || cursor.offset != bytes.len() {
            return Err("invalid Snes9x DSP terminal copy_state extension".to_string());
        }
        Self::from_checkpoint(state).map_err(|error| error.to_string())
    }

    #[cfg(test)]
    fn to_snes9x_copy_state(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(642);
        bytes.extend_from_slice(&self.state.registers);
        for voice in &self.state.voices {
            for &sample in &voice.buffer[..BRR_BUFFER_SIZE] {
                push_i16(&mut bytes, sample as i16);
            }
            push_u16(&mut bytes, voice.interpolation_position);
            push_u16(&mut bytes, voice.brr_address);
            push_u16(&mut bytes, voice.envelope as u16);
            push_i16(&mut bytes, voice.hidden_envelope as i16);
            bytes.extend_from_slice(&[
                voice.buffer_position,
                voice.brr_offset,
                voice.key_on_delay,
                voice.envelope_mode as u8,
                voice.envelope_output_latch,
                0,
            ]);
        }
        for tap in 0..ECHO_HISTORY_SIZE {
            let position =
                (usize::from(self.state.echo_history_position) + tap) % ECHO_HISTORY_SIZE;
            for channel in 0..2 {
                push_i16(
                    &mut bytes,
                    self.state.echo_history[0][position][channel] as i16,
                );
            }
        }
        bytes.extend_from_slice(&[self.state.every_other_sample, self.state.kon]);
        push_u16(&mut bytes, self.state.noise as u16);
        push_u16(&mut bytes, self.state.counter as u16);
        push_u16(&mut bytes, self.state.echo_offset);
        push_u16(&mut bytes, self.state.echo_length);
        bytes.extend_from_slice(&[
            self.state.phase,
            self.state.new_kon,
            self.state.endx_buffer,
            self.state.envx_buffer,
            self.state.outx_buffer,
            self.state.pmon_latch,
            self.state.non_latch,
            self.state.eon_latch,
            self.state.dir_latch,
            self.state.koff_latch,
        ]);
        push_u16(&mut bytes, self.state.brr_next_address_latch);
        bytes.extend_from_slice(&[
            self.state.adsr0_latch,
            self.state.brr_header_latch,
            self.state.brr_byte_latch,
            self.state.srcn_latch,
            self.state.esa_latch,
            self.state.echo_enabled_latch,
        ]);
        for values in [
            self.state.main_output,
            self.state.echo_output,
            self.state.echo_input,
        ] {
            for value in values {
                push_i16(&mut bytes, value as i16);
            }
        }
        push_u16(&mut bytes, self.state.dir_address);
        push_u16(&mut bytes, self.state.pitch as u16);
        push_i16(&mut bytes, self.state.output as i16);
        push_u16(&mut bytes, self.state.echo_pointer);
        bytes.push(self.state.looped);
        bytes.extend_from_slice(&self.state.external_registers);
        bytes.push(0);
        assert_eq!(bytes.len(), 642);
        bytes
    }

    pub fn power_on() -> Self {
        let mut machine = Self {
            state: Snes9xDspPhaseState {
                interpolation_mode: Snes9xDspInterpolationMode::Gaussian,
                registers: vec![0; REGISTER_COUNT],
                external_registers: vec![0; REGISTER_COUNT],
                voices: std::array::from_fn(|_| Snes9xDspVoiceState::default()),
                echo_history: [[[0; 2]; ECHO_HISTORY_SIZE]; 2],
                echo_history_position: 0,
                every_other_sample: 0,
                kon: 0,
                noise: 0,
                counter: 0,
                echo_offset: 0,
                echo_length: 0,
                phase: 0,
                kon_check: false,
                new_kon: 0,
                endx_buffer: 0,
                envx_buffer: 0,
                outx_buffer: 0,
                pmon_latch: 0,
                non_latch: 0,
                eon_latch: 0,
                dir_latch: 0,
                koff_latch: 0,
                brr_next_address_latch: 0,
                adsr0_latch: 0,
                brr_header_latch: 0,
                brr_byte_latch: 0,
                srcn_latch: 0,
                esa_latch: 0,
                echo_enabled_latch: 0,
                dir_address: 0,
                pitch: 0,
                output: 0,
                looped: 0,
                echo_pointer: 0,
                main_output: [0; 2],
                echo_output: [0; 2],
                echo_input: [0; 2],
                mute_mask: 0,
                stereo_switch: 0xffff,
                separate_echo_buffer_enabled: false,
                separate_echo_buffer: vec![0; 0x10000],
            },
            observation: None,
        };
        machine.load_registers(INITIAL_REGISTERS);
        machine
    }

    /// Implements `SPC_DSP::load`: external registers retain the supplied
    /// values while the internal register pipeline starts with FLG=$e0.
    pub fn load_registers(&mut self, registers: [u8; REGISTER_COUNT]) {
        let mute_mask = self.state.mute_mask;
        let stereo_switch = self.state.stereo_switch;
        let separate_echo_buffer_enabled = self.state.separate_echo_buffer_enabled;
        let mut replacement = Self::power_on_unloaded();
        replacement.external_registers.copy_from_slice(&registers);
        replacement.registers[FLG] = 0xe0;
        replacement.new_kon = replacement.registers[KON];
        replacement.dir_latch = replacement.registers[DIR];
        replacement.esa_latch = replacement.registers[ESA];
        replacement.mute_mask = mute_mask;
        replacement.stereo_switch = stereo_switch;
        replacement.separate_echo_buffer_enabled = separate_echo_buffer_enabled;
        self.state = replacement;
    }

    fn power_on_unloaded() -> Snes9xDspPhaseState {
        Snes9xDspPhaseState {
            interpolation_mode: Snes9xDspInterpolationMode::Gaussian,
            registers: vec![0; REGISTER_COUNT],
            external_registers: vec![0; REGISTER_COUNT],
            voices: std::array::from_fn(|_| Snes9xDspVoiceState::default()),
            echo_history: [[[0; 2]; ECHO_HISTORY_SIZE]; 2],
            echo_history_position: 0,
            every_other_sample: 1,
            kon: 0,
            noise: 0x4000,
            counter: 0,
            echo_offset: 0,
            echo_length: 0,
            phase: 0,
            kon_check: false,
            new_kon: 0,
            endx_buffer: 0,
            envx_buffer: 0,
            outx_buffer: 0,
            pmon_latch: 0,
            non_latch: 0,
            eon_latch: 0,
            dir_latch: 0,
            koff_latch: 0,
            brr_next_address_latch: 0,
            adsr0_latch: 0,
            brr_header_latch: 0,
            brr_byte_latch: 0,
            srcn_latch: 0,
            esa_latch: 0,
            echo_enabled_latch: 0,
            dir_address: 0,
            pitch: 0,
            output: 0,
            looped: 0,
            echo_pointer: 0,
            main_output: [0; 2],
            echo_output: [0; 2],
            echo_input: [0; 2],
            mute_mask: 0,
            stereo_switch: 0xffff,
            separate_echo_buffer_enabled: false,
            separate_echo_buffer: vec![0; 0x10000],
        }
    }

    pub fn soft_reset(&mut self) {
        self.state.registers[FLG] = 0xe0;
        self.state.noise = 0x4000;
        self.state.echo_history_position = 0;
        self.state.every_other_sample = 1;
        self.state.echo_offset = 0;
        self.state.phase = 0;
        self.state.counter = 0;
        self.state.separate_echo_buffer.fill(0);
    }

    pub fn read_register(&self, address: u8) -> u8 {
        assert!(usize::from(address) < REGISTER_COUNT);
        self.state.external_registers[usize::from(address)]
    }

    pub fn write_register(&mut self, address: u8, data: u8) {
        assert!(usize::from(address) < REGISTER_COUNT);
        let address = usize::from(address);
        self.state.registers[address] = data;
        self.state.external_registers[address] = data;
        match address & 0x0f {
            ENVX => self.state.envx_buffer = data,
            OUTX => self.state.outx_buffer = data,
            0x0c => {
                if address == KON {
                    self.state.new_kon = data;
                }
                if address == ENDX {
                    self.state.endx_buffer = 0;
                    self.state.registers[ENDX] = 0;
                }
            }
            _ => {}
        }
    }

    pub fn check_kon(&mut self) -> bool {
        let occurred = self.state.kon_check;
        self.state.kon_check = false;
        occurred
    }

    /// Execute exactly one of the 32 source phases. A stereo sample is returned
    /// only by phase 27, exactly where Snes9x sends the pair to its resampler.
    pub fn run_clock(&mut self, ram: &mut [u8; 0x10000]) -> Option<[i16; 2]> {
        self.run_clock_inner(ram)
    }

    /// Executes one clock and records every source-declared suboperation and
    /// branch outcome. The receipt is produced by the actual execution path,
    /// not a separately reconstructed phase table.
    pub fn run_clock_with_receipt(&mut self, ram: &mut [u8; 0x10000]) -> Snes9xDspClockReceipt {
        let phase = self.state.phase;
        assert!(self.observation.is_none());
        self.observation = Some(Snes9xDspClockObservation::default());
        let emitted_sample = self.run_clock_inner(ram);
        let observation = self.observation.take().unwrap();
        Snes9xDspClockReceipt {
            phase,
            suboperations: observation.suboperations,
            branch_evaluated_mask: observation.branch_evaluated_mask,
            branch_taken_mask: observation.branch_taken_mask,
            branch_not_taken_mask: observation.branch_not_taken_mask,
            emitted_sample,
        }
    }

    fn run_clock_inner(&mut self, ram: &mut [u8; 0x10000]) -> Option<[i16; 2]> {
        let sample = match self.state.phase {
            0 => {
                self.voice_v5(0);
                self.voice_v2(1, ram);
                None
            }
            1 => {
                self.voice_v6(0);
                self.voice_v3(1, ram);
                None
            }
            2 => {
                self.voice_v7(0);
                self.voice_v1(3);
                self.voice_v4(1, ram);
                None
            }
            3 => {
                self.voice_v8(0);
                self.voice_v5(1);
                self.voice_v2(2, ram);
                None
            }
            4 => {
                self.voice_v9(0);
                self.voice_v6(1);
                self.voice_v3(2, ram);
                None
            }
            5 => {
                self.voice_v7(1);
                self.voice_v1(4);
                self.voice_v4(2, ram);
                None
            }
            6 => {
                self.voice_v8(1);
                self.voice_v5(2);
                self.voice_v2(3, ram);
                None
            }
            7 => {
                self.voice_v9(1);
                self.voice_v6(2);
                self.voice_v3(3, ram);
                None
            }
            8 => {
                self.voice_v7(2);
                self.voice_v1(5);
                self.voice_v4(3, ram);
                None
            }
            9 => {
                self.voice_v8(2);
                self.voice_v5(3);
                self.voice_v2(4, ram);
                None
            }
            10 => {
                self.voice_v9(2);
                self.voice_v6(3);
                self.voice_v3(4, ram);
                None
            }
            11 => {
                self.voice_v7(3);
                self.voice_v1(6);
                self.voice_v4(4, ram);
                None
            }
            12 => {
                self.voice_v8(3);
                self.voice_v5(4);
                self.voice_v2(5, ram);
                None
            }
            13 => {
                self.voice_v9(3);
                self.voice_v6(4);
                self.voice_v3(5, ram);
                None
            }
            14 => {
                self.voice_v7(4);
                self.voice_v1(7);
                self.voice_v4(5, ram);
                None
            }
            15 => {
                self.voice_v8(4);
                self.voice_v5(5);
                self.voice_v2(6, ram);
                None
            }
            16 => {
                self.voice_v9(4);
                self.voice_v6(5);
                self.voice_v3(6, ram);
                None
            }
            17 => {
                self.voice_v1(0);
                self.voice_v7(5);
                self.voice_v4(6, ram);
                None
            }
            18 => {
                self.voice_v8(5);
                self.voice_v5(6);
                self.voice_v2(7, ram);
                None
            }
            19 => {
                self.voice_v9(5);
                self.voice_v6(6);
                self.voice_v3(7, ram);
                None
            }
            20 => {
                self.voice_v1(1);
                self.voice_v7(6);
                self.voice_v4(7, ram);
                None
            }
            21 => {
                self.voice_v8(6);
                self.voice_v5(7);
                self.voice_v2(0, ram);
                None
            }
            22 => {
                self.voice_v3a(0);
                self.voice_v9(6);
                self.voice_v6(7);
                self.echo_22(ram);
                None
            }
            23 => {
                self.voice_v7(7);
                self.echo_23(ram);
                None
            }
            24 => {
                self.voice_v8(7);
                self.echo_24();
                None
            }
            25 => {
                self.voice_v3b(0, ram);
                self.voice_v9(7);
                self.echo_25();
                None
            }
            26 => {
                self.echo_26();
                None
            }
            27 => {
                self.misc_27();
                Some(self.echo_27())
            }
            28 => {
                self.misc_28();
                self.echo_28();
                None
            }
            29 => {
                self.misc_29();
                self.echo_29(ram);
                None
            }
            30 => {
                self.misc_30();
                self.voice_v3c(0);
                self.echo_30(ram);
                None
            }
            31 => {
                self.voice_v4(0, ram);
                self.voice_v1(2);
                None
            }
            _ => unreachable!("DSP phase is always masked to five bits"),
        };
        self.state.phase = (self.state.phase + 1) & 31;
        sample
    }

    fn voice_register(&self, voice: usize, register: usize) -> u8 {
        self.state.registers[voice * 0x10 + register]
    }

    fn set_voice_register(&mut self, voice: usize, register: usize, value: u8) {
        let address = voice * 0x10 + register;
        self.state.registers[address] = value;
        self.state.external_registers[address] = value;
    }

    fn voice_bit(voice: usize) -> u8 {
        1 << voice
    }

    fn observe_suboperation(&mut self, kind: i8, voice: i8, argument: i8) {
        if let Some(observation) = &mut self.observation {
            observation.suboperations.push(Snes9xDspSuboperation {
                kind,
                voice,
                argument,
            });
        }
    }

    fn observe_branch(&mut self, branch: u8, taken: bool) {
        if let Some(observation) = &mut self.observation {
            let bit = 1u64 << branch;
            observation.branch_evaluated_mask |= bit;
            if taken {
                observation.branch_taken_mask |= bit;
            } else {
                observation.branch_not_taken_mask |= bit;
            }
        }
    }

    fn voice_v1(&mut self, voice: usize) {
        self.observe_suboperation(1, voice as i8, 0);
        self.state.dir_address = (u16::from(self.state.dir_latch) << 8)
            .wrapping_add(u16::from(self.state.srcn_latch) * 4);
        self.state.srcn_latch = self.voice_register(voice, SRCN);
    }

    fn voice_v2(&mut self, voice: usize, ram: &[u8; 0x10000]) {
        self.observe_suboperation(2, voice as i8, 0);
        let mut address = self.state.dir_address;
        if self.state.voices[voice].key_on_delay == 0 {
            address = address.wrapping_add(2);
        }
        self.state.brr_next_address_latch = read_le_u16(ram, address);
        self.state.adsr0_latch = self.voice_register(voice, ADSR0);
        self.state.pitch = i32::from(self.voice_register(voice, PITCHL));
    }

    fn voice_v3a(&mut self, voice: usize) {
        self.observe_suboperation(10, voice as i8, 0);
        self.state.pitch += i32::from(self.voice_register(voice, PITCHH) & 0x3f) << 8;
    }

    fn voice_v3b(&mut self, voice: usize, ram: &[u8; 0x10000]) {
        self.observe_suboperation(11, voice as i8, 0);
        let v = &self.state.voices[voice];
        self.state.brr_byte_latch =
            ram[usize::from(v.brr_address.wrapping_add(u16::from(v.brr_offset)))];
        self.state.brr_header_latch = ram[usize::from(v.brr_address)];
    }

    fn voice_v3(&mut self, voice: usize, ram: &[u8; 0x10000]) {
        self.voice_v3a(voice);
        self.voice_v3b(voice, ram);
        self.voice_v3c(voice);
    }

    fn voice_v3c(&mut self, voice: usize) {
        self.observe_suboperation(12, voice as i8, 0);
        let vbit = Self::voice_bit(voice);
        let pitch_modulation = self.state.pmon_latch & vbit != 0;
        self.observe_branch(8, pitch_modulation);
        if pitch_modulation {
            self.state.pitch += ((self.state.output >> 5) * self.state.pitch) >> 10;
        }

        let key_on_delay = self.state.voices[voice].key_on_delay != 0;
        self.observe_branch(9, key_on_delay);
        if key_on_delay {
            let v = &mut self.state.voices[voice];
            if v.key_on_delay == 5 {
                v.brr_address = self.state.brr_next_address_latch;
                v.brr_offset = 1;
                v.buffer_position = 0;
                self.state.brr_header_latch = 0;
                self.state.kon_check = true;
            }
            v.envelope = 0;
            v.hidden_envelope = 0;
            v.interpolation_position = 0;
            v.key_on_delay -= 1;
            if v.key_on_delay & 3 != 0 {
                v.interpolation_position = 0x4000;
            }
            self.state.pitch = 0;
        }

        let v = &self.state.voices[voice];
        let index = usize::from(v.interpolation_position >> 12) + usize::from(v.buffer_position);
        let offset = (v.interpolation_position >> 4) as u8;
        let mut output = match self.state.interpolation_mode {
            Snes9xDspInterpolationMode::Gaussian => i32::from(dsp_gaussian_interpolate(
                v.buffer[index] as i16,
                v.buffer[index + 1] as i16,
                v.buffer[index + 2] as i16,
                v.buffer[index + 3] as i16,
                offset,
            )),
        };
        let noise = self.state.non_latch & vbit != 0;
        self.observe_branch(10, noise);
        if noise {
            output = i32::from((self.state.noise * 2) as i16);
        }
        self.state.output = ((output * self.state.voices[voice].envelope) >> 11) & !1;
        self.state.voices[voice].envelope_output_latch =
            (self.state.voices[voice].envelope >> 4) as u8;

        let reset_or_end =
            self.state.registers[FLG] & 0x80 != 0 || self.state.brr_header_latch & 3 == 1;
        self.observe_branch(11, reset_or_end);
        if reset_or_end {
            self.state.voices[voice].envelope_mode = Snes9xDspEnvelopeMode::Release;
            self.state.voices[voice].envelope = 0;
        }
        if self.state.every_other_sample != 0 {
            let key_off = self.state.koff_latch & vbit != 0;
            self.observe_branch(12, key_off);
            if key_off {
                self.state.voices[voice].envelope_mode = Snes9xDspEnvelopeMode::Release;
            }
            let key_on = self.state.kon & vbit != 0;
            self.observe_branch(13, key_on);
            if key_on {
                self.state.voices[voice].key_on_delay = 5;
                self.state.voices[voice].envelope_mode = Snes9xDspEnvelopeMode::Attack;
            }
        }
        if self.state.voices[voice].key_on_delay == 0 {
            self.run_envelope(voice);
        }
    }

    fn voice_v4(&mut self, voice: usize, ram: &[u8; 0x10000]) {
        self.observe_suboperation(4, voice as i8, 0);
        self.state.looped = 0;
        let decode = self.state.voices[voice].interpolation_position >= 0x4000;
        self.observe_branch(15, decode);
        if decode {
            self.decode_brr(voice, ram);
            self.state.voices[voice].brr_offset += 2;
            if u16::from(self.state.voices[voice].brr_offset) >= BRR_BLOCK_SIZE {
                self.state.voices[voice].brr_address = self.state.voices[voice]
                    .brr_address
                    .wrapping_add(BRR_BLOCK_SIZE);
                let looped = self.state.brr_header_latch & 1 != 0;
                self.observe_branch(16, looped);
                if looped {
                    self.state.voices[voice].brr_address = self.state.brr_next_address_latch;
                    self.state.looped = Self::voice_bit(voice);
                }
                self.state.voices[voice].brr_offset = 1;
            }
        }
        let position =
            i32::from(self.state.voices[voice].interpolation_position & 0x3fff) + self.state.pitch;
        self.state.voices[voice].interpolation_position = position.min(0x7fff) as u16;
        self.voice_output(voice, 0);
    }

    fn voice_v5(&mut self, voice: usize) {
        self.observe_suboperation(5, voice as i8, 0);
        self.voice_output(voice, 1);
        let mut endx = self.state.registers[ENDX] | self.state.looped;
        if self.state.voices[voice].key_on_delay == 5 {
            endx &= !Self::voice_bit(voice);
        }
        self.state.endx_buffer = endx;
    }

    fn voice_v6(&mut self, voice: usize) {
        self.observe_suboperation(6, voice as i8, 0);
        self.state.outx_buffer = (self.state.output >> 8) as u8;
    }

    fn voice_v7(&mut self, voice: usize) {
        self.observe_suboperation(7, voice as i8, 0);
        self.state.registers[ENDX] = self.state.endx_buffer;
        self.state.external_registers[ENDX] = self.state.endx_buffer;
        self.state.envx_buffer = self.state.voices[voice].envelope_output_latch;
    }

    fn voice_v8(&mut self, voice: usize) {
        self.observe_suboperation(8, voice as i8, 0);
        self.set_voice_register(voice, OUTX, self.state.outx_buffer);
    }

    fn voice_v9(&mut self, voice: usize) {
        self.observe_suboperation(9, voice as i8, 0);
        self.set_voice_register(voice, ENVX, self.state.envx_buffer);
    }

    fn voice_output(&mut self, voice: usize, channel: usize) {
        let volume = self.voice_register(voice, VOLL + channel) as i8 as i32;
        let mut amplitude = (self.state.output * volume) >> 7;
        if self.state.stereo_switch & (1 << (voice + channel * VOICE_COUNT)) == 0 {
            amplitude = 0;
        }
        self.state.main_output[channel] = clamp_i16(self.state.main_output[channel] + amplitude);
        let echo_enabled = self.state.eon_latch & Self::voice_bit(voice) != 0;
        self.observe_branch(14, echo_enabled);
        if echo_enabled {
            self.state.echo_output[channel] =
                clamp_i16(self.state.echo_output[channel] + amplitude);
        }
    }

    fn run_envelope(&mut self, voice: usize) {
        let mut env = self.state.voices[voice].envelope;
        let release = self.state.voices[voice].envelope_mode == Snes9xDspEnvelopeMode::Release;
        self.observe_branch(0, release);
        if release {
            env = (env - 8).max(0);
            self.state.voices[voice].envelope = env;
            return;
        }

        let mut rate;
        let mut env_data = self.voice_register(voice, ADSR1);
        let adsr_enabled = self.state.adsr0_latch & 0x80 != 0;
        self.observe_branch(1, adsr_enabled);
        if adsr_enabled {
            if matches!(
                self.state.voices[voice].envelope_mode,
                Snes9xDspEnvelopeMode::Decay | Snes9xDspEnvelopeMode::Sustain
            ) {
                env -= 1;
                env -= env >> 8;
                rate = usize::from(env_data & 0x1f);
                if self.state.voices[voice].envelope_mode == Snes9xDspEnvelopeMode::Decay {
                    rate = usize::from((self.state.adsr0_latch >> 3 & 0x0e) + 0x10);
                }
            } else {
                rate = usize::from((self.state.adsr0_latch & 0x0f) * 2 + 1);
                env += if rate < 31 { 0x20 } else { 0x400 };
            }
        } else {
            env_data = self.voice_register(voice, GAIN);
            let mode = env_data >> 5;
            let direct = mode < 4;
            self.observe_branch(2, direct);
            if direct {
                env = i32::from(env_data) * 0x10;
                rate = 31;
            } else {
                rate = usize::from(env_data & 0x1f);
                let linear_decrease = mode == 4;
                self.observe_branch(3, linear_decrease);
                if linear_decrease {
                    env -= 0x20;
                } else {
                    let exponential_decrease = mode < 6;
                    self.observe_branch(4, exponential_decrease);
                    if exponential_decrease {
                        env -= 1;
                        env -= env >> 8;
                    } else {
                        self.observe_branch(5, mode == 6);
                        env += 0x20;
                        let two_slope =
                            mode > 6 && self.state.voices[voice].hidden_envelope as u32 >= 0x600;
                        self.observe_branch(6, two_slope);
                        if two_slope {
                            env += 0x8 - 0x20;
                        }
                    }
                }
            }
        }
        if env >> 8 == i32::from(env_data >> 5)
            && self.state.voices[voice].envelope_mode == Snes9xDspEnvelopeMode::Decay
        {
            self.state.voices[voice].envelope_mode = Snes9xDspEnvelopeMode::Sustain;
        }
        self.state.voices[voice].hidden_envelope = env;
        if env as u32 > 0x7ff {
            env = if env < 0 { 0 } else { 0x7ff };
            if self.state.voices[voice].envelope_mode == Snes9xDspEnvelopeMode::Attack {
                self.state.voices[voice].envelope_mode = Snes9xDspEnvelopeMode::Decay;
            }
        }
        let counter_tick = self.read_counter(rate) == 0;
        self.observe_branch(7, counter_tick);
        if counter_tick {
            self.state.voices[voice].envelope = env;
        }
    }

    fn decode_brr(&mut self, voice: usize, ram: &[u8; 0x10000]) {
        let v = &self.state.voices[voice];
        let second = ram[usize::from(
            v.brr_address
                .wrapping_add(u16::from(v.brr_offset))
                .wrapping_add(1),
        )];
        let mut nybbles = i32::from(self.state.brr_byte_latch) * 0x100 + i32::from(second);
        let header = i32::from(self.state.brr_header_latch);
        let start = usize::from(v.buffer_position);
        let next_position = if start + 4 >= BRR_BUFFER_SIZE {
            0
        } else {
            start + 4
        };
        for position in start..start + 4 {
            let mut sample = (nybbles as i16 as i32) >> 12;
            let shift = header >> 4;
            if shift <= 12 {
                sample = (sample << shift) >> 1;
            } else {
                sample &= !0x7ff;
            }
            let previous1 = self.state.voices[voice].buffer[position + BRR_BUFFER_SIZE - 1];
            let previous2 = self.state.voices[voice].buffer[position + BRR_BUFFER_SIZE - 2] >> 1;
            let filter = header & 0x0c;
            if filter >= 8 {
                sample += previous1;
                sample -= previous2;
                if filter == 8 {
                    sample += previous2 >> 4;
                    sample += (previous1 * -3) >> 6;
                } else {
                    sample += (previous1 * -13) >> 7;
                    sample += (previous2 * 3) >> 4;
                }
            } else if filter != 0 {
                sample += previous1 >> 1;
                sample += (-previous1) >> 5;
            }
            sample = i32::from((clamp_i16(sample) * 2) as i16);
            self.state.voices[voice].buffer[position] = sample;
            self.state.voices[voice].buffer[position + BRR_BUFFER_SIZE] = sample;
            nybbles <<= 4;
        }
        self.state.voices[voice].buffer_position = next_position as u8;
    }

    fn read_counter(&self, rate: usize) -> u32 {
        (self.state.counter as u32 + COUNTER_OFFSETS[rate]) % COUNTER_RATES[rate]
    }

    fn misc_27(&mut self) {
        self.observe_suboperation(27, -1, 27);
        self.state.pmon_latch = self.state.registers[PMON] & 0xfe;
    }

    fn misc_28(&mut self) {
        self.observe_suboperation(28, -1, 28);
        self.state.non_latch = self.state.registers[NON];
        self.state.eon_latch = self.state.registers[EON];
        self.state.dir_latch = self.state.registers[DIR];
    }

    fn misc_29(&mut self) {
        self.observe_suboperation(29, -1, 29);
        self.state.every_other_sample ^= 1;
        let every_other = self.state.every_other_sample != 0;
        self.observe_branch(21, every_other);
        if every_other {
            self.state.new_kon &= !self.state.kon;
        }
    }

    fn misc_30(&mut self) {
        self.observe_suboperation(30, -1, 30);
        let latch_keys = self.state.every_other_sample != 0;
        self.observe_branch(22, latch_keys);
        if latch_keys {
            self.state.kon = self.state.new_kon;
            self.state.koff_latch = self.state.registers[KOFF] | self.state.mute_mask;
        }
        self.state.counter -= 1;
        if self.state.counter < 0 {
            self.state.counter = SIMPLE_COUNTER_RANGE - 1;
        }
        let noise_tick = self.read_counter(usize::from(self.state.registers[FLG] & 0x1f)) == 0;
        self.observe_branch(23, noise_tick);
        if noise_tick {
            let feedback = (self.state.noise << 13) ^ (self.state.noise << 14);
            self.state.noise = (feedback & 0x4000) ^ (self.state.noise >> 1);
        }
    }

    fn echo_fir(&self, tap: usize, channel: usize) -> i32 {
        let position = usize::from(self.state.echo_history_position) + tap + 1;
        let coefficient = self.state.registers[FIR + tap * 0x10] as i8 as i32;
        (self.state.echo_history[position / ECHO_HISTORY_SIZE][position % ECHO_HISTORY_SIZE]
            [channel]
            * coefficient)
            >> 6
    }

    fn echo_read(&mut self, channel: usize, ram: &[u8; 0x10000]) {
        let address = self
            .state
            .echo_pointer
            .wrapping_add((channel as u16).wrapping_mul(2));
        let sample = if self.state.separate_echo_buffer_enabled {
            read_le_i16_slice(&self.state.separate_echo_buffer, address)
        } else {
            read_le_i16_slice(ram, address)
        };
        let value = i32::from(sample) >> 1;
        let position = usize::from(self.state.echo_history_position);
        self.state.echo_history[0][position][channel] = value;
        self.state.echo_history[1][position][channel] = value;
    }

    fn echo_22(&mut self, ram: &[u8; 0x10000]) {
        self.observe_suboperation(50, -1, 22);
        self.state.echo_history_position =
            (self.state.echo_history_position + 1) % ECHO_HISTORY_SIZE as u8;
        self.state.echo_pointer =
            (u16::from(self.state.esa_latch) << 8).wrapping_add(self.state.echo_offset);
        self.echo_read(0, ram);
        self.state.echo_input = [self.echo_fir(0, 0), self.echo_fir(0, 1)];
    }

    fn echo_23(&mut self, ram: &[u8; 0x10000]) {
        self.observe_suboperation(50, -1, 23);
        self.state.echo_input[0] += self.echo_fir(1, 0) + self.echo_fir(2, 0);
        self.state.echo_input[1] += self.echo_fir(1, 1) + self.echo_fir(2, 1);
        self.echo_read(1, ram);
    }

    fn echo_24(&mut self) {
        self.observe_suboperation(50, -1, 24);
        self.state.echo_input[0] += self.echo_fir(3, 0) + self.echo_fir(4, 0) + self.echo_fir(5, 0);
        self.state.echo_input[1] += self.echo_fir(3, 1) + self.echo_fir(4, 1) + self.echo_fir(5, 1);
    }

    fn echo_25(&mut self) {
        self.observe_suboperation(50, -1, 25);
        for channel in 0..2 {
            let mut value = self.state.echo_input[channel] + self.echo_fir(6, channel);
            value = i32::from(value as i16);
            value += i32::from(self.echo_fir(7, channel) as i16);
            self.state.echo_input[channel] = clamp_i16(value) & !1;
        }
    }

    fn echo_mix_output(&self, channel: usize) -> i32 {
        let main_volume =
            self.state.registers[if channel == 0 { MVOLL } else { MVOLR }] as i8 as i32;
        let echo_volume =
            self.state.registers[if channel == 0 { EVOLL } else { EVOLR }] as i8 as i32;
        clamp_i16(
            i32::from(((self.state.main_output[channel] * main_volume) >> 7) as i16)
                + i32::from(((self.state.echo_input[channel] * echo_volume) >> 7) as i16),
        )
    }

    fn echo_26(&mut self) {
        self.observe_suboperation(50, -1, 26);
        self.state.main_output[0] = self.echo_mix_output(0);
        let feedback = self.state.registers[EFB] as i8 as i32;
        for channel in 0..2 {
            let value = self.state.echo_output[channel]
                + i32::from(((self.state.echo_input[channel] * feedback) >> 7) as i16);
            self.state.echo_output[channel] = clamp_i16(value) & !1;
        }
    }

    fn echo_27(&mut self) -> [i16; 2] {
        self.observe_suboperation(50, -1, 27);
        let mut left = self.state.main_output[0];
        let mut right = self.echo_mix_output(1);
        self.state.main_output = [0; 2];
        let muted = self.state.registers[FLG] & 0x40 != 0;
        self.observe_branch(17, muted);
        if muted {
            left = 0;
            right = 0;
        }
        [left as i16, right as i16]
    }

    fn echo_28(&mut self) {
        self.observe_suboperation(50, -1, 28);
        self.state.echo_enabled_latch = self.state.registers[FLG];
    }

    fn echo_write(&mut self, channel: usize, ram: &mut [u8; 0x10000]) {
        let enabled = self.state.echo_enabled_latch & 0x20 == 0;
        self.observe_branch(18, enabled);
        if enabled {
            let address = self
                .state
                .echo_pointer
                .wrapping_add((channel as u16).wrapping_mul(2));
            let value = self.state.echo_output[channel] as i16;
            if self.state.separate_echo_buffer_enabled {
                write_le_i16_slice(&mut self.state.separate_echo_buffer, address, value);
            } else {
                write_le_i16_slice(ram, address, value);
            }
        }
        self.state.echo_output[channel] = 0;
    }

    fn echo_29(&mut self, ram: &mut [u8; 0x10000]) {
        self.observe_suboperation(50, -1, 29);
        self.state.esa_latch = self.state.registers[ESA];
        let latch_length = self.state.echo_offset == 0;
        self.observe_branch(19, latch_length);
        if latch_length {
            self.state.echo_length = u16::from(self.state.registers[EDL] & 0x0f) * 0x800;
        }
        self.state.echo_offset = self.state.echo_offset.wrapping_add(4);
        let ring_wrap = self.state.echo_offset >= self.state.echo_length;
        self.observe_branch(20, ring_wrap);
        if ring_wrap {
            self.state.echo_offset = 0;
        }
        self.echo_write(0, ram);
        self.state.echo_enabled_latch = self.state.registers[FLG];
    }

    fn echo_30(&mut self, ram: &mut [u8; 0x10000]) {
        self.observe_suboperation(50, -1, 30);
        self.echo_write(1, ram);
    }
}

fn clamp_i16(value: i32) -> i32 {
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX))
}

fn read_le_u16(ram: &[u8], address: u16) -> u16 {
    let low = ram[usize::from(address)];
    let high = ram[usize::from(address.wrapping_add(1))];
    u16::from_le_bytes([low, high])
}

fn read_le_i16_slice(ram: &[u8], address: u16) -> i16 {
    read_le_u16(ram, address) as i16
}

fn write_le_i16_slice(ram: &mut [u8], address: u16, value: i16) {
    let [low, high] = value.to_le_bytes();
    ram[usize::from(address)] = low;
    ram[usize::from(address.wrapping_add(1))] = high;
}

#[cfg(test)]
struct DspCopyStateReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

#[cfg(test)]
impl DspCopyStateReader<'_> {
    fn u8(&mut self) -> u8 {
        let value = self.bytes[self.offset];
        self.offset += 1;
        value
    }

    fn u16(&mut self) -> u16 {
        u16::from_le_bytes([self.u8(), self.u8()])
    }

    fn i16(&mut self) -> i16 {
        self.u16() as i16
    }

    fn copy_into(&mut self, output: &mut [u8]) {
        let end = self.offset + output.len();
        output.copy_from_slice(&self.bytes[self.offset..end]);
        self.offset = end;
    }
}

#[cfg(test)]
fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
fn push_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::{fs, path::Path};

    const DSP_LEDGER_PATH: &str =
        "../../external/snes9x-libretro/fixtures/snes9x-spc-dsp-phase-ledger.jsonl";

    fn decode_base64_bytes(encoded: &str) -> Vec<u8> {
        assert_eq!(encoded.len() % 4, 0);
        let value = |byte: u8| match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 0,
            _ => panic!("invalid fixture base64 digit"),
        };
        let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
        for chunk in encoded.as_bytes().chunks_exact(4) {
            let bits = u32::from(value(chunk[0])) << 18
                | u32::from(value(chunk[1])) << 12
                | u32::from(value(chunk[2])) << 6
                | u32::from(value(chunk[3]));
            decoded.push((bits >> 16) as u8);
            if chunk[2] != b'=' {
                decoded.push((bits >> 8) as u8);
            }
            if chunk[3] != b'=' {
                decoded.push(bits as u8);
            }
        }
        decoded
    }

    fn read_unsigned_varint(bytes: &[u8], index: &mut usize) -> u64 {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = bytes[*index];
            *index += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
            shift += 7;
            assert!(shift < 64);
        }
    }

    fn expand_fixture_sequence(sequence: &Value) -> Vec<Vec<i64>> {
        assert_eq!(
            sequence["encoding"],
            "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
        );
        let field_count = sequence["fields"].as_array().unwrap().len();
        let record_count = sequence["record_count"].as_u64().unwrap() as usize;
        let compressed = decode_base64_bytes(sequence["data_base64"].as_str().unwrap());
        let bytes = zstd::stream::decode_all(compressed.as_slice()).unwrap();
        let mut columns = Vec::with_capacity(field_count);
        let mut byte_index = 0;
        for _ in 0..field_count {
            let column_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
            let column_end = byte_index + column_length;
            let mut column = Vec::with_capacity(record_count);
            let mut previous = 0i64;
            while byte_index < column_end {
                let code = read_unsigned_varint(&bytes, &mut byte_index);
                if code == 0 {
                    let run_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
                    column.extend(std::iter::repeat_n(previous, run_length));
                } else {
                    let zigzag = code - 1;
                    let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                    previous += delta;
                    column.push(previous);
                }
            }
            assert_eq!(column.len(), record_count);
            columns.push(column);
        }
        assert_eq!(byte_index, bytes.len());
        let mut digest_bytes = Vec::with_capacity(record_count * field_count * 8);
        let rows = (0..record_count)
            .map(|row| {
                (0..field_count)
                    .map(|field| {
                        let value = columns[field][row];
                        digest_bytes.extend_from_slice(&value.to_le_bytes());
                        value
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            format!("{:x}", Sha256::digest(digest_bytes)),
            sequence["expanded_sha256"].as_str().unwrap()
        );
        rows
    }

    fn fixture_rows() -> (Value, Value) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DSP_LEDGER_PATH);
        let bytes = fs::read(&path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "dec2ee495692d8b3dc110fac7db82efdd47c6982d0a99ebb0c807afd2b7b2f10"
        );
        let records = std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        (records[0].clone(), records[1].clone())
    }

    fn seeded_ram(profile: u8) -> Box<[u8; 0x10000]> {
        let mut ram = Box::new([0; 0x10000]);
        for (address, value) in ram.iter_mut().enumerate() {
            *value = (address as u32 * 37
                + ((address as u32) >> 8) * 13
                + u32::from(profile) * 97
                + 0x5a) as u8;
        }
        for voice in 0..8usize {
            let directory = 0x2000 + voice * 4;
            let brr = 0x3000 + voice * 0x20;
            let [low, high] = (brr as u16).to_le_bytes();
            ram[directory..directory + 4].copy_from_slice(&[low, high, low, high]);
            ram[brr] = if voice & 1 != 0 { 0 } else { 3 };
        }
        ram
    }

    fn combine_mask(row: &[i64], low: usize) -> u64 {
        u64::from(row[low] as u32) | (u64::from(row[low + 1] as u32) << 32)
    }

    #[test]
    fn canonical_snes9x_dsp_phase_ledger_matches_every_case_exactly() {
        let (provenance, fixture) = fixture_rows();
        assert_eq!(
            provenance["schema"],
            "pinned-snes9x-spc-dsp-phase-ledger-v1"
        );
        assert_eq!(provenance["source"]["copy_state_size"], 642);
        assert_eq!(provenance["source"]["case_count"], 5_440);
        assert_eq!(provenance["source"]["checkpoint_count"], 514);
        assert_eq!(provenance["source"]["interpolation_method"], 2);
        assert_eq!(provenance["source"]["separate_echo_buffer"], 0);

        let checkpoint_xors = expand_fixture_sequence(&fixture["checkpoint_state_xor_sequence"]);
        let mut current = vec![0u8; 642];
        let mut checkpoint_states = Vec::with_capacity(514);
        for chunk in checkpoint_xors.chunks_exact(642) {
            let checkpoint_id = checkpoint_states.len();
            for (byte_index, row) in chunk.iter().enumerate() {
                assert_eq!(row[0], checkpoint_id as i64);
                assert_eq!(row[1], byte_index as i64);
                current[byte_index] ^= row[2] as u8;
            }
            checkpoint_states.push(current.clone());
        }
        assert_eq!(checkpoint_states.len(), 514);

        let cases = expand_fixture_sequence(&fixture["case_sequence"]);
        assert_eq!(cases.len(), 5_440);
        let case_xors = expand_fixture_sequence(&fixture["copy_state_xor_sequence"]);
        let mut current = vec![0u8; 642];
        let mut case_states = Vec::with_capacity(cases.len());
        for chunk in case_xors.chunks_exact(642) {
            let case_id = case_states.len();
            for (byte_index, row) in chunk.iter().enumerate() {
                assert_eq!(row[0], case_id as i64);
                assert_eq!(row[1], byte_index as i64);
                current[byte_index] ^= row[2] as u8;
            }
            case_states.push(current.clone());
        }
        assert_eq!(case_states.len(), cases.len());

        let mut ram_diffs = vec![Vec::<(usize, u8)>::new(); cases.len()];
        for row in expand_fixture_sequence(&fixture["ram_diff_sequence"]) {
            ram_diffs[row[0] as usize].push((row[1] as usize, row[2] as u8));
        }
        let mut expected_samples = vec![Vec::<[i16; 2]>::new(); cases.len()];
        for row in expand_fixture_sequence(&fixture["sample_sequence"]) {
            let samples = &mut expected_samples[row[0] as usize];
            assert_eq!(row[1], samples.len() as i64);
            samples.push([row[2] as i16, row[3] as i16]);
        }
        let mut expected_suboperations = vec![Vec::<Snes9xDspSuboperation>::new(); cases.len()];
        for row in expand_fixture_sequence(&fixture["suboperation_sequence"]) {
            let operations = &mut expected_suboperations[row[0] as usize];
            assert_eq!(row[1], operations.len() as i64);
            operations.push(Snes9xDspSuboperation {
                kind: row[2] as i8,
                voice: row[3] as i8,
                argument: row[4] as i8,
            });
        }

        let mut initial_ram_diffs = [Vec::<(usize, u8)>::new(), Vec::new()];
        for row in expand_fixture_sequence(&fixture["profile_initial_ram_diff_sequence"]) {
            initial_ram_diffs[row[0] as usize].push((row[1] as usize, row[2] as u8));
        }
        let mut checkpoint_rams = Vec::with_capacity(514);
        for profile in 0..=1usize {
            let mut ram = seeded_ram(profile as u8);
            for &(address, value) in &initial_ram_diffs[profile] {
                ram[address] = value;
            }
            for tick in 0..=256usize {
                checkpoint_rams.push(ram.clone());
                if tick < 256 {
                    for &(address, value) in &ram_diffs[profile * 256 + tick] {
                        ram[address] = value;
                    }
                }
            }
        }

        for (case_id, row) in cases.iter().enumerate() {
            assert_eq!(row[0], case_id as i64);
            let start_checkpoint = row[2] as usize;
            let mut dsp =
                Snes9xDspPhaseMachine::from_snes9x_copy_state(&checkpoint_states[start_checkpoint])
                    .unwrap_or_else(|error| panic!("case {case_id} start codec: {error}"));
            assert_eq!(
                i64::from(dsp.state.phase),
                row[3],
                "case {case_id} start phase"
            );
            assert_eq!(
                dsp.to_snes9x_copy_state(),
                checkpoint_states[start_checkpoint],
                "case {case_id} start codec roundtrip"
            );
            let mut ram = checkpoint_rams[start_checkpoint].clone();
            let mut samples = Vec::new();
            let mut suboperations = Vec::new();
            let mut evaluated = 0u64;
            let mut taken = 0u64;
            let mut not_taken = 0u64;
            let operation = row[5];
            let mut remaining_debt = row[4] as usize;
            if operation != 3 {
                for _ in 0..remaining_debt {
                    let receipt = dsp.run_clock_with_receipt(&mut ram);
                    if let Some(sample) = receipt.emitted_sample {
                        samples.push(sample);
                    }
                    suboperations.extend(receipt.suboperations);
                    evaluated |= receipt.branch_evaluated_mask;
                    taken |= receipt.branch_taken_mask;
                    not_taken |= receipt.branch_not_taken_mask;
                }
                remaining_debt = 0;
            }
            let result = match operation {
                0 => 0,
                1 => i64::from(dsp.read_register((row[6] as u8) & 0x7f)),
                2 => {
                    dsp.write_register(row[6] as u8, row[7] as u8);
                    row[7]
                }
                3 => row[7],
                _ => panic!("case {case_id} invalid operation {operation}"),
            };
            assert_eq!(result, row[8], "case {case_id} result");
            assert_eq!(i64::from(dsp.state.phase), row[9], "case {case_id} phase");
            assert_eq!(row[10], 0, "case {case_id} initial debt");
            assert_eq!(remaining_debt as i64, row[11], "case {case_id} debt");
            assert_eq!(evaluated, combine_mask(row, 12), "case {case_id} evaluated");
            assert_eq!(taken, combine_mask(row, 14), "case {case_id} taken");
            assert_eq!(not_taken, combine_mask(row, 16), "case {case_id} not-taken");
            assert_eq!(row[18], 642, "case {case_id} state size");
            assert_eq!(ram_diffs[case_id].len() as i64, row[19]);
            assert_eq!(expected_samples[case_id].len() as i64, row[20]);
            assert_eq!(expected_suboperations[case_id].len() as i64, row[21]);
            assert_eq!(
                dsp.to_snes9x_copy_state(),
                case_states[case_id],
                "case {case_id} full 642-byte state"
            );
            let mut expected_ram = checkpoint_rams[start_checkpoint].clone();
            for &(address, value) in &ram_diffs[case_id] {
                expected_ram[address] = value;
            }
            assert_eq!(ram[..], expected_ram[..], "case {case_id} shared RAM");
            assert_eq!(samples, expected_samples[case_id], "case {case_id} samples");
            assert_eq!(
                suboperations, expected_suboperations[case_id],
                "case {case_id} suboperations"
            );
        }
    }

    #[test]
    fn source_reset_keeps_external_power_on_registers_but_internal_flg_is_e0() {
        let dsp = Snes9xDspPhaseMachine::power_on();
        assert_eq!(
            dsp.interpolation_mode(),
            Snes9xDspInterpolationMode::Gaussian
        );
        assert_eq!(dsp.state.external_registers, INITIAL_REGISTERS);
        assert_eq!(dsp.state.registers[FLG], 0xe0);
        assert_eq!(dsp.state.external_registers[FLG], INITIAL_REGISTERS[FLG]);
        assert_eq!(dsp.state.noise, 0x4000);
        assert_eq!(dsp.state.every_other_sample, 1);
        assert_eq!(dsp.state.phase, 0);
    }

    #[test]
    fn endx_write_has_the_source_internal_external_split() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        dsp.state.endx_buffer = 0xaa;
        dsp.write_register(ENDX as u8, 0x5a);
        assert_eq!(dsp.state.registers[ENDX], 0);
        assert_eq!(dsp.state.endx_buffer, 0);
        assert_eq!(dsp.read_register(ENDX as u8), 0x5a);
    }

    #[test]
    fn exact_32_phase_schedule_wraps_and_emits_only_at_phase_27() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        let mut ram = [0; 0x10000];
        let emitted: Vec<_> = (0..32).filter_map(|_| dsp.run_clock(&mut ram)).collect();
        assert_eq!(emitted.len(), 1);
        assert_eq!(dsp.state.phase, 0);
    }

    #[test]
    fn phase_18_is_only_the_source_v8_v5_v2_composite() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        let mut ram = [0; 0x10000];
        dsp.state.phase = 18;
        dsp.state.endx_buffer = 0x5a;
        dsp.state.registers[ENDX] = 0;
        dsp.state.outx_buffer = 0x33;
        dsp.state.dir_address = 0x4000;
        ram[0x4002] = 0x78;
        ram[0x4003] = 0x56;

        dsp.run_clock(&mut ram);

        // V8(5) publishes OUTX, V2(7) reads the directory pointer, while the
        // V7(5) from phase 17 must not be replayed.
        assert_eq!(dsp.state.registers[5 * 0x10 + OUTX], 0x33);
        assert_eq!(dsp.state.brr_next_address_latch, 0x5678);
        assert_eq!(dsp.state.registers[ENDX], 0);
    }

    #[test]
    fn v7_v1_v4_composites_report_the_exact_source_order() {
        let cases = [
            (2, 0, 3, 1),
            (5, 1, 4, 2),
            (8, 2, 5, 3),
            (11, 3, 6, 4),
            (14, 4, 7, 5),
        ];
        for (phase, v7, v1, v4) in cases {
            let mut dsp = Snes9xDspPhaseMachine::power_on();
            let mut ram = [0; 0x10000];
            dsp.state.phase = phase;
            let receipt = dsp.run_clock_with_receipt(&mut ram);
            assert_eq!(receipt.phase, phase);
            assert_eq!(
                receipt.suboperations,
                [
                    Snes9xDspSuboperation {
                        kind: 7,
                        voice: v7,
                        argument: 0
                    },
                    Snes9xDspSuboperation {
                        kind: 1,
                        voice: v1,
                        argument: 0
                    },
                    Snes9xDspSuboperation {
                        kind: 4,
                        voice: v4,
                        argument: 0
                    },
                ]
            );
        }
    }

    #[test]
    fn counter_rates_and_offsets_match_spc_dsp_source() {
        let dsp = Snes9xDspPhaseMachine::power_on();
        assert_eq!(dsp.read_counter(0), 1);
        assert_eq!(dsp.read_counter(1), 0);
        assert_eq!(dsp.read_counter(2), 1040);
        assert_eq!(dsp.read_counter(3), 536);
        assert_eq!(dsp.read_counter(31), 0);
    }

    #[test]
    fn clone_and_serde_preserve_every_pipeline_phase() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        let mut ram = [0; 0x10000];
        for expected_phase in 0..32 {
            assert_eq!(dsp.state.phase, expected_phase);
            assert_eq!(dsp.clone(), dsp);
            let encoded = serde_json::to_vec(&dsp).unwrap();
            let decoded: Snes9xDspPhaseMachine = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(decoded, dsp);
            dsp.run_clock(&mut ram);
        }
    }

    #[test]
    fn malformed_checkpoint_is_rejected_before_it_can_enter_the_machine() {
        let dsp = Snes9xDspPhaseMachine::power_on();
        let mut checkpoint = dsp.checkpoint();
        checkpoint.registers.pop();
        assert_eq!(
            Snes9xDspPhaseMachine::from_checkpoint(checkpoint.clone()),
            Err(Snes9xDspCheckpointError::InternalRegisterLength { actual: 127 })
        );

        let encoded = serde_json::to_vec(&checkpoint).unwrap();
        let error = serde_json::from_slice::<Snes9xDspPhaseMachine>(&encoded).unwrap_err();
        assert!(error.to_string().contains("length 127"));

        let mut checkpoint = dsp.checkpoint();
        checkpoint.phase = 32;
        assert_eq!(
            Snes9xDspPhaseMachine::from_checkpoint(checkpoint),
            Err(Snes9xDspCheckpointError::Phase { phase: 32 })
        );

        let mut checkpoint = dsp.checkpoint();
        checkpoint.voices[2].buffer[BRR_BUFFER_SIZE] = 1;
        assert_eq!(
            Snes9xDspPhaseMachine::from_checkpoint(checkpoint),
            Err(Snes9xDspCheckpointError::VoiceBufferMirror {
                voice: 2,
                sample: 0
            })
        );
    }

    #[test]
    fn composite_checkpoint_resumes_ram_effect_and_phase_27_sample_once() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        let ram = [0; 0x10000];
        dsp.state.phase = 29;
        dsp.state.echo_pointer = 0x1234;
        dsp.state.echo_output = [-0x1234, 0x2345];
        dsp.state.echo_enabled_latch = 0;
        dsp.state.registers[FLG] = 0;

        let checkpoint = Snes9xDspCompositeCheckpoint::capture(&dsp, &ram);
        let encoded = serde_json::to_vec(&checkpoint).unwrap();
        let decoded: Snes9xDspCompositeCheckpoint = serde_json::from_slice(&encoded).unwrap();
        let (mut resumed, mut resumed_ram) = decoded.restore().unwrap();

        let emitted: Vec<_> = (0..31)
            .filter_map(|_| resumed.run_clock(&mut resumed_ram))
            .collect();
        assert_eq!(emitted.len(), 1);
        assert_eq!(read_le_i16_slice(&resumed_ram[..], 0x1234), -0x1234);
        assert_eq!(read_le_i16_slice(&resumed_ram[..], 0x1236), 0x2345);
        assert!(resumed.run_clock(&mut resumed_ram).is_none());
    }

    #[test]
    fn phase_29_echo_write_uses_latched_disable_then_captures_current_flg() {
        let mut dsp = Snes9xDspPhaseMachine::power_on();
        let mut ram = [0; 0x10000];
        dsp.state.phase = 29;
        dsp.state.echo_pointer = 0x1234;
        dsp.state.echo_output[0] = -0x1234;
        dsp.state.echo_enabled_latch = 0;
        dsp.state.registers[FLG] = 0x20;
        dsp.run_clock(&mut ram);
        assert_eq!(read_le_i16_slice(&ram, 0x1234), -0x1234);
        assert_eq!(dsp.state.echo_enabled_latch, 0x20);
        assert_eq!(dsp.state.echo_output[0], 0);
    }
}
