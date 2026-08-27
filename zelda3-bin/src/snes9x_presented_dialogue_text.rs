//! Replaceable adapter for the dialogue character generation used by one
//! completed Snes9x host scanout.
//!
//! Pinned Snes9x returns immediately before the following VBlank NMI, so its
//! exposed VRAM contains the BG3 character words consumed by the completed
//! active field. Emulator memory layout and addresses stop in this adapter;
//! translated Zelda receives only [`PresentedDialogueText`].

use crate::libretro_core::{LibretroCore, RETRO_MEMORY_VIDEO_RAM};
use zelda3::PresentedDialogueText;

const BG3_TEXT_START_WORD: usize = 0x7c00;

pub(crate) fn snes9x_presented_dialogue_text(
    oracle: &LibretroCore,
) -> Result<PresentedDialogueText, String> {
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .ok_or("pinned Snes9x did not expose VRAM for dialogue publication")?;
    decode_presented_dialogue_text(vram)
}

fn decode_presented_dialogue_text(vram: &[u8]) -> Result<PresentedDialogueText, String> {
    let start = BG3_TEXT_START_WORD * 2;
    let byte_count = PresentedDialogueText::WORD_COUNT * 2;
    let bytes = vram
        .get(start..start + byte_count)
        .ok_or_else(|| format!("pinned Snes9x VRAM is only {} bytes", vram.len()))?;
    let words = bytes
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect();
    PresentedDialogueText::new(words)
        .ok_or_else(|| "pinned Snes9x dialogue-text receipt has an invalid shape".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_extracts_only_c_nmi_upload_bg3_text_words() {
        let mut vram = vec![0xaa; 0x1_0000];
        for word in 0..PresentedDialogueText::WORD_COUNT {
            let offset = (BG3_TEXT_START_WORD + word) * 2;
            vram[offset..offset + 2].copy_from_slice(&(word as u16).to_le_bytes());
        }

        let receipt = decode_presented_dialogue_text(&vram).unwrap();
        assert_eq!(receipt.words()[0], 0);
        assert_eq!(receipt.words()[0x2a5], 0x02a5);
        assert_eq!(receipt.words().last(), Some(&0x03ef));
    }

    #[test]
    fn decoder_rejects_truncated_video_ram() {
        assert!(decode_presented_dialogue_text(&vec![0; BG3_TEXT_START_WORD * 2]).is_err());
    }
}
