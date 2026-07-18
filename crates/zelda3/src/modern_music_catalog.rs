pub const PACKED_NOTE_BYTES: usize = 23;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernMusicNote {
    pub voice: u8,
    pub pitch: u8,
    pub instrument: u8,
    pub volume: u8,
    pub pan: i8,
    pub start_frame: u16,
    pub duration_frames: u16,
    pub dsp_pitch: u16,
    pub sample_offset: u16,
    pub volume_left: i8,
    pub volume_right: i8,
    pub adsr1: u8,
    pub adsr2: u8,
    pub gain: u8,
    pub echo_send: bool,
    pub keyoff_sample_offset: u16,
    pub kon_phase: u8,
    pub keyoff_phase: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedModernMusicTrack {
    pub track: u8,
    pub lead_in_frames: u16,
    /// Native 32 kHz DSP-sample interval that repeats for looping tracks.
    /// Zeroes mean that the captured sequence is intentionally one-shot.
    pub loop_start_sample: u32,
    pub loop_end_sample: u32,
    pub notes: &'static [u8],
}

impl PackedModernMusicTrack {
    pub const fn loop_range(self) -> Option<std::ops::Range<u32>> {
        if self.loop_end_sample > self.loop_start_sample {
            Some(self.loop_start_sample..self.loop_end_sample)
        } else {
            None
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/modern_music_assets.rs"));

pub fn packed_track(track: u8) -> Option<&'static PackedModernMusicTrack> {
    PACKED_TRACKS
        .iter()
        .find(|candidate| candidate.track == track)
}

pub fn decode_note(bytes: &[u8]) -> Option<ModernMusicNote> {
    let bytes: &[u8; PACKED_NOTE_BYTES] = bytes.try_into().ok()?;
    Some(ModernMusicNote {
        voice: bytes[0],
        pitch: bytes[1],
        instrument: bytes[2],
        volume: bytes[3],
        pan: bytes[4] as i8,
        start_frame: u16::from_le_bytes([bytes[5], bytes[6]]),
        duration_frames: u16::from_le_bytes([bytes[7], bytes[8]]),
        dsp_pitch: u16::from_le_bytes([bytes[9], bytes[10]]),
        sample_offset: u16::from_le_bytes([bytes[11], bytes[12]]),
        volume_left: bytes[13] as i8,
        volume_right: bytes[14] as i8,
        adsr1: bytes[15],
        adsr2: bytes[16],
        gain: bytes[17],
        echo_send: bytes[18] != 0,
        keyoff_sample_offset: u16::from_le_bytes([bytes[19], bytes[20]]),
        kon_phase: bytes[21],
        keyoff_phase: bytes[22],
    })
}

/// Returns only notes whose source start frame is in the inclusive range.
///
/// Packed tracks are sorted by start frame at build time. Binary-searching the
/// packed bytes keeps long-running looping tracks from decoding every captured
/// note on every game frame.
pub fn notes_starting_in(
    track: &'static PackedModernMusicTrack,
    first_frame: u16,
    last_frame: u16,
) -> impl Iterator<Item = ModernMusicNote> {
    fn frame_at(notes: &[u8], index: usize) -> u16 {
        let offset = index * PACKED_NOTE_BYTES + 5;
        u16::from_le_bytes([notes[offset], notes[offset + 1]])
    }

    fn lower_bound(notes: &[u8], target: u32) -> usize {
        let mut low = 0usize;
        let mut high = notes.len() / PACKED_NOTE_BYTES;
        while low < high {
            let middle = low + (high - low) / 2;
            if u32::from(frame_at(notes, middle)) < target {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    let start = lower_bound(track.notes, u32::from(first_frame));
    let end = lower_bound(track.notes, u32::from(last_frame) + 1);
    track.notes[start * PACKED_NOTE_BYTES..end * PACKED_NOTE_BYTES]
        .chunks_exact(PACKED_NOTE_BYTES)
        .filter_map(decode_note)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_catalog_is_well_formed_and_time_sorted() {
        assert!(!PACKED_TRACKS.is_empty());
        for track in PACKED_TRACKS {
            assert_eq!(track.notes.len() % PACKED_NOTE_BYTES, 0);
            let notes: Vec<_> = track
                .notes
                .chunks_exact(PACKED_NOTE_BYTES)
                .map(|bytes| decode_note(bytes).unwrap())
                .collect();
            assert!(!notes.is_empty());
            assert!(notes
                .windows(2)
                .all(|pair| pair[0].start_frame <= pair[1].start_frame));
            assert!(notes.iter().all(|note| note.voice < 8));
        }
    }

    #[test]
    fn title_track_uses_snes9x_command_phase() {
        let track = packed_track(1).unwrap();
        let first = decode_note(&track.notes[..PACKED_NOTE_BYTES]).unwrap();

        assert_eq!(track.lead_in_frames, 7);
        assert_eq!(first.start_frame, 0);
        assert_eq!(first.sample_offset, 287);
        assert_eq!(first.duration_frames, 29);
        assert_eq!(first.keyoff_sample_offset, 16);
        assert!(!first.echo_send);
    }

    #[test]
    fn house_track_declares_a_sample_accurate_loop() {
        let track = packed_track(3).unwrap();
        let loop_range = track.loop_range().unwrap();

        assert_eq!(loop_range.start, 1_521_498);
        assert_eq!(loop_range.end, 2_172_314);
        assert_eq!(loop_range.end - loop_range.start, 650_816);
    }

    #[test]
    fn indexed_note_lookup_matches_a_full_catalog_filter() {
        let track = packed_track(3).unwrap();
        let expected = track
            .notes
            .chunks_exact(PACKED_NOTE_BYTES)
            .filter_map(decode_note)
            .filter(|note| (2840..=2850).contains(&note.start_frame))
            .collect::<Vec<_>>();

        assert!(!expected.is_empty());
        assert_eq!(
            notes_starting_in(track, 2840, 2850).collect::<Vec<_>>(),
            expected
        );
    }
}
