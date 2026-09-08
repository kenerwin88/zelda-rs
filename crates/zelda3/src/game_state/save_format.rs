//! Original cartridge save format. This borrows the published compatibility
//! image; it is not another live owner of inventory or progress.

use super::constants::SAVE_DUNG_INFO;
use crate::types::{read_le_u16, write_le_u16};
use std::ops::Range;

pub(crate) const SAVE_BLOCK_BYTES: usize = 0x500;
pub(crate) const SAVE_CHECKSUM_OFFSET: usize = SAVE_BLOCK_BYTES - 2;
pub(crate) const SAVE_CHECKSUM_WORDS: u16 = (SAVE_CHECKSUM_OFFSET / 2) as u16;
const BACKUP_OFFSET: usize = 0xf00;

pub(crate) struct LiveSave<'a> {
    bytes: &'a [u8],
}

impl<'a> LiveSave<'a> {
    pub(crate) fn from_wram(ram: &'a [u8]) -> Self {
        Self {
            bytes: &ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + SAVE_BLOCK_BYTES],
        }
    }

    pub(crate) fn copy_to_sram(&self, sram: &mut [u8], slot_offset: usize) {
        for offset in [slot_offset, slot_offset + BACKUP_OFFSET] {
            if let Some(destination) = sram.get_mut(offset..offset + SAVE_BLOCK_BYTES) {
                destination.copy_from_slice(self.bytes);
            }
        }
    }

    /// Read this portion now. Callers retain the accumulated sum across NMI;
    /// later portions must observe the then-current image, not an earlier copy.
    pub(crate) fn sum_words(&self, words: Range<u16>, mut sum: u16) -> u16 {
        assert!(words.start <= words.end && words.end <= SAVE_CHECKSUM_WORDS);
        for word in words {
            sum = sum.wrapping_add(read_le_u16(self.bytes, usize::from(word) * 2));
        }
        sum
    }

    pub(crate) fn checksum(sum: u16) -> u16 {
        0x5a5au16.wrapping_sub(sum)
    }

    /// The source room-word index can cross into other save domains. Keep
    /// that addressing contract at the format boundary, not in native rooms.
    pub(crate) fn indexed_word(&self, index: usize) -> u16 {
        if index < SAVE_BLOCK_BYTES / 2 {
            read_le_u16(self.bytes, index * 2)
        } else {
            0
        }
    }
}

pub(crate) fn write_indexed_word(ram: &mut [u8], index: usize, value: u16) {
    if index < SAVE_BLOCK_BYTES / 2 {
        write_le_u16(ram, SAVE_DUNG_INFO + index * 2, value);
    }
}

pub(crate) fn replace_live_save(ram: &mut [u8], source: &[u8]) {
    ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + SAVE_BLOCK_BYTES].copy_from_slice(source);
}

pub(crate) fn clear_live_save(ram: &mut [u8]) {
    ram[SAVE_DUNG_INFO..SAVE_DUNG_INFO + SAVE_BLOCK_BYTES].fill(0);
}

pub(crate) fn write_sram_checksum(sram: &mut [u8], slot_offset: usize, checksum: u16) {
    for offset in [slot_offset, slot_offset + BACKUP_OFFSET] {
        if let Some(save) = sram.get_mut(offset..offset + SAVE_BLOCK_BYTES) {
            write_le_u16(save, SAVE_CHECKSUM_OFFSET, checksum);
        }
    }
}
