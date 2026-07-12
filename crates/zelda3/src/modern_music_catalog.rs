pub const PACKED_NOTE_BYTES: usize = 21;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedModernMusicTrack {
    pub track: u8,
    pub lead_in_frames: u16,
    pub notes: &'static [u8],
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
    })
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
}
