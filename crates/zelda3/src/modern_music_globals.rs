#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModernMusicGlobalEvent {
    pub track: u8,
    /// Track-relative S-DSP clock at which the write reached the register file.
    /// One output sample is exactly 32 DSP cycles.
    pub dsp_cycle: u32,
    pub register: u8,
    pub value: u8,
}

include!(concat!(env!("OUT_DIR"), "/modern_music_global_assets.rs"));

pub fn events_in_cycle_range(
    track: u8,
    start_cycle: u64,
    end_cycle: u64,
) -> impl Iterator<Item = ModernMusicGlobalEvent> {
    let start_cycle = start_cycle.min(u64::from(u32::MAX)) as u32;
    let end_cycle = end_cycle.min(u64::from(u32::MAX) + 1);
    let start = MUSIC_GLOBAL_EVENTS.partition_point(|event| {
        event.track < track || (event.track == track && event.dsp_cycle < start_cycle)
    });
    let end = MUSIC_GLOBAL_EVENTS.partition_point(|event| {
        event.track < track || (event.track == track && u64::from(event.dsp_cycle) < end_cycle)
    });
    MUSIC_GLOBAL_EVENTS[start..end].iter().copied()
}

/// Convert a phase-0-aligned S-DSP write clock into the output-sample clock
/// consumed by the modern renderer. A write that reaches the register file
/// after a register's read phase becomes visible in the following sample.
pub const fn output_sample_for_write(dsp_cycle: u32, register: u8) -> u32 {
    let raw_sample = dsp_cycle / 32;
    let phase = (dsp_cycle & 31) as u8;
    let register = register & 0x7f;
    // The renderer interpolates a voice and then advances its BRR cursor in one
    // operation. S-DSP reads PITCHL/PITCHH earlier in the target voice's
    // staggered pipeline and applies that word to the cursor after the current
    // output. Schedule the write on the preceding renderer sample when it made
    // that pipeline read; otherwise schedule it on the current sample.
    if matches!(register & 0x0f, 0x02 | 0x03) {
        let voice = register >> 4;
        let base_phase = if register & 0x0f == 0x02 { 21 } else { 22 };
        let read_phase = (base_phase + voice * 3) % 24;
        let renderer_sample = if voice == 0 {
            raw_sample
        } else {
            raw_sample.saturating_sub(1)
        };
        return renderer_sample.saturating_add((phase > read_phase) as u32);
    }
    // EON is latched at phase 28, but that latch controls voice output which
    // is accumulated into the *following* phase-29 echo write. The renderer's
    // echo-send flag therefore becomes visible one sample after the latch.
    if register == 0x4d {
        return raw_sample.saturating_add(1 + (phase > 28) as u32);
    }
    // PMON/NON/DIR and the echo address/length registers are first
    // captured into internal S-DSP latches. Their effect belongs to the next
    // generated sample, or one sample later if the write missed that latch.
    let latch_phase = match register {
        0x2d => Some(27),
        0x3d | 0x5d => Some(28),
        0x6d | 0x7d => Some(29),
        _ => None,
    };
    if let Some(latch_phase) = latch_phase {
        return raw_sample.saturating_add((phase > latch_phase) as u32);
    }
    let read_phase = match register {
        register if register & 0x0f == 0x0f => match register >> 4 {
            0 => 22,
            1 | 2 => 23,
            3..=5 => 24,
            _ => 25,
        },
        // Left and right final outputs are calculated on cycles 26 and 27.
        0x0c | 0x2c | 0x0d => 26,
        0x1c | 0x3c => 27,
        // Voice/FIR writes are retained in the same absolute clock. Until the
        // renderer exposes every internal voice stage, the output boundary is
        // the conservative visibility point.
        _ => 27,
    };
    raw_sample.saturating_add((phase > read_phase) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_events_are_sorted_for_indexed_lookup() {
        assert!(MUSIC_GLOBAL_EVENTS.windows(2).all(|events| {
            (events[0].track, events[0].dsp_cycle) <= (events[1].track, events[1].dsp_cycle)
        }));
    }

    #[test]
    fn indexed_lookup_returns_only_the_requested_cycle_range() {
        let expected = MUSIC_GLOBAL_EVENTS
            .iter()
            .copied()
            .find(|event| event.track == 3)
            .expect("house music has global events");
        let events = events_in_cycle_range(
            expected.track,
            u64::from(expected.dsp_cycle),
            u64::from(expected.dsp_cycle) + 1,
        )
        .collect::<Vec<_>>();

        assert!(!events.is_empty());
        assert!(events
            .iter()
            .all(|event| event.track == expected.track && event.dsp_cycle == expected.dsp_cycle));
    }

    #[test]
    fn output_visibility_uses_the_register_read_cycle() {
        assert_eq!(output_sample_for_write(496 * 32 + 20, 0x3c), 496);
        assert_eq!(output_sample_for_write(498 * 32 + 6, 0x2c), 498);
        assert_eq!(output_sample_for_write(496 * 32 + 28, 0x3c), 497);
        assert_eq!(output_sample_for_write(105 * 32 + 27, 0x5f), 106);
        assert_eq!(output_sample_for_write(436 * 32 + 31, 0x2c), 437);
        // EON is latched at phase 28, then the enabled voice output must reach
        // the following phase-29 echo write. A phase-30 register write misses
        // both the current latch and the next echo write.
        assert_eq!(output_sample_for_write(245 * 32 + 30, 0x4d), 247);
        assert_eq!(output_sample_for_write(95 * 32 + 30, 0x6d), 96);
    }

    #[test]
    fn pitch_writes_follow_the_target_voices_pipeline_read() {
        // Voice 4 reads PITCHL at phase 9. A phase-0 write changes the pitch
        // applied after the preceding renderer sample; a phase-10 write misses
        // that read and applies one renderer sample later.
        assert_eq!(output_sample_for_write(399 * 32, 0x42), 398);
        assert_eq!(output_sample_for_write(399 * 32 + 10, 0x42), 399);
        // Voice 0 advances at phase 31, after the phase-27 output boundary, so
        // its phase-20 PITCHL write belongs to the current renderer sample.
        assert_eq!(output_sample_for_write(387 * 32 + 20, 0x02), 387);
        assert_eq!(output_sample_for_write(387 * 32 + 22, 0x02), 388);
    }
}
