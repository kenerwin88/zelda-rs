use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RomRandomSample {
    pub execution_frame: u32,
    pub value: u8,
}

impl RomRandomSample {
    pub const fn new(execution_frame: u32, value: u8) -> Self {
        Self {
            execution_frame,
            value,
        }
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
        if let Some(extra) = fields.next() {
            return Err(format!(
                "line {line_number}: unexpected third field {extra:?}"
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
        samples.push(RomRandomSample::new(execution_frame, value));
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

#[derive(Clone, Debug, Default)]
pub(crate) struct RomRandomReplay {
    enabled: bool,
    current_execution_frame: Option<u32>,
    next_execution_frame: u32,
    samples: VecDeque<RomRandomSample>,
}

impl RomRandomReplay {
    pub(crate) fn install(&mut self, samples: Vec<RomRandomSample>, start_execution_frame: u32) {
        self.enabled = true;
        self.current_execution_frame = None;
        self.next_execution_frame = start_execution_frame;
        self.samples = samples.into();
    }

    pub(crate) fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_execution_frame = Some(self.next_execution_frame);
        self.next_execution_frame = self.next_execution_frame.wrapping_add(1);
    }

    pub(crate) fn take_next(&mut self) -> Option<u8> {
        if !self.enabled {
            return None;
        }
        let execution_frame = self
            .current_execution_frame
            .expect("ROM random replay consumed outside a host frame");
        let sample = self.samples.pop_front().unwrap_or_else(|| {
            panic!(
                "unexpected ROM random call during execution frame {execution_frame}: replay is exhausted"
            )
        });
        assert_eq!(
            sample.execution_frame, execution_frame,
            "ROM random call order diverged: replay expected execution frame {}, Rust called during {execution_frame}",
            sample.execution_frame
        );
        Some(sample.value)
    }

    pub(crate) fn remaining(&self) -> Option<usize> {
        self.enabled.then_some(self.samples.len())
    }

    pub(crate) fn finish(&self) -> Result<(), String> {
        let Some(remaining) = self.remaining() else {
            return Ok(());
        };
        if remaining == 0 {
            return Ok(());
        }
        let next = self.samples.front().unwrap();
        Err(format!(
            "{remaining} ROM random sample(s) were not consumed; next sample is for execution frame {}",
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
            4976 0x45 # second call in the same frame
            0x13a2 216
            ",
        )
        .unwrap();

        assert_eq!(
            samples,
            vec![
                RomRandomSample::new(4976, 0x22),
                RomRandomSample::new(4976, 0x45),
                RomRandomSample::new(5026, 0xd8),
            ]
        );
    }

    #[test]
    fn replay_validates_host_frame_and_preserves_call_order() {
        let mut replay = RomRandomReplay::default();
        replay.install(
            vec![
                RomRandomSample::new(12, 0x34),
                RomRandomSample::new(12, 0x56),
                RomRandomSample::new(13, 0x78),
            ],
            12,
        );

        replay.begin_frame();
        assert_eq!(replay.take_next(), Some(0x34));
        assert_eq!(replay.take_next(), Some(0x56));
        replay.begin_frame();
        assert_eq!(replay.take_next(), Some(0x78));
        assert_eq!(replay.remaining(), Some(0));
    }
}
