//! Cartridge-frame input scripts, excluded from the shipped lagless game.
use std::fs;
use std::path::Path;

pub(crate) struct CartridgeInputRecording;

impl CartridgeInputRecording {
    pub(crate) fn finish(&self, dir: &Path, history: &[(u32, u16)]) -> Result<(), String> {
        fs::write(dir.join("input.txt"), crate::libretro_timeline::format_input_history(history))
            .map_err(|e| format!("write deterministic input script: {e}"))
    }
}
