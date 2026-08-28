use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RomRandomSample {
    pub execution_frame: u32,
    pub value: u8,
    /// Carry left by the cartridge RNG routine's final `ADC`. Logical
    /// operations such as `AND` preserve this flag, so a few callers consume
    /// it in their next arithmetic instruction.
    pub carry: bool,
}

impl RomRandomSample {
    pub const fn new(execution_frame: u32, value: u8) -> Self {
        Self::with_carry(execution_frame, value, false)
    }

    pub const fn with_carry(execution_frame: u32, value: u8, carry: bool) -> Self {
        Self {
            execution_frame,
            value,
            carry,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RomRandomResult {
    value: u8,
    carry: bool,
}

impl RomRandomResult {
    pub(crate) const fn new(value: u8, carry: bool) -> Self {
        Self { value, carry }
    }

    pub(crate) const fn value(self) -> u8 {
        self.value
    }

    pub(crate) const fn carry(self) -> bool {
        self.carry
    }

    /// Models the common ROM sequence `AND #mask; ADC #operand`: `AND`
    /// changes the accumulator but deliberately leaves RNG's carry intact.
    pub(crate) const fn masked_adc(self, mask: u8, operand: u8) -> u8 {
        (self.value & mask)
            .wrapping_add(operand)
            .wrapping_add(self.carry as u8)
    }
}

pub fn parse_rom_random_script(text: &str) -> Result<Vec<RomRandomSample>, String> {
    let mut samples: Vec<RomRandomSample> = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let content = line.split('#').next().unwrap_or_default().trim();
        if content.is_empty() {
            continue;
        }
        let mut fields = content.split_whitespace();
        let execution_frame = fields
            .next()
            .ok_or_else(|| format!("line {line_number}: missing execution frame"))
            .and_then(|value| parse_u32(value, "execution frame", line_number))?;
        let value = fields
            .next()
            .ok_or_else(|| format!("line {line_number}: missing random value"))
            .and_then(|value| parse_u8(value, line_number))?;
        let carry = fields
            .next()
            .map(|value| parse_carry(value, line_number))
            .transpose()?
            .unwrap_or(false);
        if let Some(extra) = fields.next() {
            return Err(format!(
                "line {line_number}: unexpected fourth field {extra:?}"
            ));
        }
        if let Some(previous) = samples.last() {
            if execution_frame < previous.execution_frame {
                return Err(format!(
                    "line {line_number}: execution frame {execution_frame} precedes {}",
                    previous.execution_frame
                ));
            }
        }
        samples.push(RomRandomSample::with_carry(execution_frame, value, carry));
    }
    Ok(samples)
}

fn parse_u32(value: &str, label: &str, line_number: usize) -> Result<u32, String> {
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u32::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|error| format!("line {line_number}: invalid {label} {value:?}: {error}"))
}

fn parse_u8(value: &str, line_number: usize) -> Result<u8, String> {
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u8::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|error| format!("line {line_number}: invalid random value {value:?}: {error}"))
}

fn parse_carry(value: &str, line_number: usize) -> Result<bool, String> {
    match value {
        "carry=0" => Ok(false),
        "carry=1" => Ok(true),
        _ => Err(format!(
            "line {line_number}: invalid RNG carry {value:?}; expected carry=0 or carry=1"
        )),
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RomRandomReplay {
    enabled: bool,
    current_execution_frame: Option<u32>,
    next_execution_frame: u32,
    samples: VecDeque<RomRandomSample>,
    first_frame_drift: Option<(u32, u32)>,
}

impl RomRandomReplay {
    pub(crate) fn install(&mut self, samples: Vec<RomRandomSample>, start_execution_frame: u32) {
        self.enabled = true;
        self.current_execution_frame = None;
        self.next_execution_frame = start_execution_frame;
        self.first_frame_drift = None;
        self.samples = samples
            .into_iter()
            .skip_while(|sample| sample.execution_frame < start_execution_frame)
            .collect();
    }

    pub(crate) fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_execution_frame = Some(self.next_execution_frame);
        self.next_execution_frame = self.next_execution_frame.wrapping_add(1);
    }

    #[track_caller]
    pub(crate) fn take_next(&mut self) -> Option<RomRandomResult> {
        if !self.enabled {
            return None;
        }
        let execution_frame = self
            .current_execution_frame
            .expect("ROM random replay consumed outside a host frame");
        let caller = std::panic::Location::caller();
        let Some(sample) = self.samples.pop_front() else {
            if std::env::var_os("ZELDA3_DEBUG_ALLOW_UNEXPECTED_ROM_RANDOM").is_some() {
                eprintln!(
                    "unexpected_rom_random frame={execution_frame} callsite={}:{} fallback=00 carry=0",
                    caller.file(),
                    caller.line(),
                );
                return Some(RomRandomResult::new(0, false));
            }
            panic!(
                "unexpected ROM random call during execution frame {execution_frame}: replay is exhausted; callsite={}:{}; set ZELDA3_DEBUG_ALLOW_UNEXPECTED_ROM_RANDOM=1 to continue with a zero-valued diagnostic sample",
                caller.file(),
                caller.line(),
            )
        };
        if sample.execution_frame != execution_frame {
            if std::env::var_os("ZELDA3_DEBUG_ROM_RANDOM_FRAME_DRIFT").is_none() {
                panic!(
                    "ROM random call order diverged: replay expected execution frame {}, Rust called during {execution_frame}; callsite={}:{}",
                    sample.execution_frame,
                    caller.file(),
                    caller.line(),
                );
            }
            if self.first_frame_drift.is_none() {
                self.first_frame_drift = Some((sample.execution_frame, execution_frame));
                eprintln!(
                    "rom_random_frame_drift expected={} actual={} delta={} value={:02x} carry={}",
                    sample.execution_frame,
                    execution_frame,
                    i64::from(execution_frame) - i64::from(sample.execution_frame),
                    sample.value,
                    u8::from(sample.carry),
                );
            }
        }
        Some(RomRandomResult::new(sample.value, sample.carry))
    }

    pub(crate) fn remaining(&self) -> Option<usize> {
        self.enabled.then_some(self.samples.len())
    }

    pub(crate) fn finish(&self) -> Result<(), String> {
        match self.remaining() {
            None | Some(0) => Ok(()),
            Some(remaining) => Err(format!(
                "{remaining} ROM random sample(s) were not consumed"
            )),
        }
    }

    pub(crate) fn finish_through(&self, end_execution_frame: u32) -> Result<(), String> {
        let Some(remaining) = self.remaining() else {
            return Ok(());
        };
        if remaining == 0 {
            return Ok(());
        }
        let next = self.samples.front().unwrap();
        if next.execution_frame >= end_execution_frame {
            return Ok(());
        }
        let overdue = self
            .samples
            .iter()
            .take_while(|sample| sample.execution_frame < end_execution_frame)
            .count();
        Err(format!(
            "{overdue} ROM random sample(s) through execution frame {} were not consumed; next sample is for execution frame {} ({remaining} total sample(s) remain)",
            end_execution_frame.saturating_sub(1),
            next.execution_frame
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_parser_accepts_repeated_frames_and_hex_values() {
        let samples = parse_rom_random_script(
            "
            # Outputs written by ROM $8dba71.
            4976 0x22
            4976 0x45 carry=1 # second call in the same frame
            0x13a2 216 carry=0
            ",
        )
        .unwrap();

        assert_eq!(
            samples,
            vec![
                RomRandomSample::new(4976, 0x22),
                RomRandomSample::with_carry(4976, 0x45, true),
                RomRandomSample::new(5026, 0xd8),
            ]
        );
    }

    #[test]
    fn replay_validates_host_frame_and_preserves_call_order() {
        let mut replay = RomRandomReplay::default();
        replay.install(
            vec![
                RomRandomSample::with_carry(12, 0x34, true),
                RomRandomSample::with_carry(12, 0x56, false),
                RomRandomSample::with_carry(13, 0x78, true),
            ],
            12,
        );

        replay.begin_frame();
        assert_eq!(replay.take_next(), Some(RomRandomResult::new(0x34, true)));
        assert_eq!(replay.take_next(), Some(RomRandomResult::new(0x56, false)));
        replay.begin_frame();
        assert_eq!(replay.take_next(), Some(RomRandomResult::new(0x78, true)));
        assert_eq!(replay.remaining(), Some(0));
    }

    #[test]
    fn resumed_replay_discards_samples_before_its_execution_frame() {
        let mut replay = RomRandomReplay::default();
        replay.install(
            vec![
                RomRandomSample::new(10, 0x12),
                RomRandomSample::new(11, 0x34),
                RomRandomSample::new(12, 0x56),
            ],
            12,
        );

        replay.begin_frame();
        assert_eq!(replay.take_next(), Some(RomRandomResult::new(0x56, false)));
        assert_eq!(replay.remaining(), Some(0));
    }

    #[test]
    fn prefix_replay_allows_future_samples_but_rejects_missed_calls() {
        let mut replay = RomRandomReplay::default();
        replay.install(
            vec![
                RomRandomSample::new(12, 0x12),
                RomRandomSample::new(15, 0x34),
            ],
            10,
        );

        assert_eq!(replay.finish_through(12), Ok(()));
        assert!(replay.finish_through(13).unwrap_err().contains("frame 12"));
    }

    #[test]
    fn masked_adc_preserves_rng_carry_across_the_mask() {
        assert_eq!(
            RomRandomResult::new(0x03, false).masked_adc(0x3f, 0x30),
            0x33
        );
        assert_eq!(
            RomRandomResult::new(0x03, true).masked_adc(0x3f, 0x30),
            0x34
        );
    }
}
