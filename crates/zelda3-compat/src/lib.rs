//! Legacy runtime compatibility bridges.
//!
//! Code in this crate may understand old byte streams, offsets, and sentinel
//! values, but it stays independent of the live `zelda3::ZeldaState`. That keeps
//! production modernization code separate from parity/oracle tooling while the
//! renderer transitions from legacy message state to semantic dialogue IR.

pub const UNKNOWN_DIALOGUE_OFFSET: u16 = u16::MAX;

pub use zelda3_dialogue::{DialogueIrKind, DialogueIrOp};

#[derive(Clone, Copy, Debug)]
pub struct LegacyDialogueView<'a> {
    pub dialogue_flags: u8,
    pub decoded: &'a [u8],
    pub glyph_run_dialogue_offsets: &'a [u16],
}

impl<'a> LegacyDialogueView<'a> {
    pub fn new(
        dialogue_flags: u8,
        decoded: &'a [u8],
        glyph_run_dialogue_offsets: &'a [u16],
    ) -> Self {
        Self {
            dialogue_flags,
            decoded,
            glyph_run_dialogue_offsets,
        }
    }

    pub fn ir(&self) -> Vec<DialogueIrOp> {
        zelda3_dialogue::parse_dialogue_ir(self.dialogue_flags, self.decoded)
    }

    pub fn ir_for_glyph_run(&self, run_index: usize) -> Option<DialogueIrOp> {
        let offset = *self.glyph_run_dialogue_offsets.get(run_index)?;
        if offset == UNKNOWN_DIALOGUE_OFFSET {
            return None;
        }
        zelda3_dialogue::dialogue_ir_op_at(self.dialogue_flags, self.decoded, usize::from(offset))
    }
}

pub fn legacy_dialogue_ir(dialogue_flags: u8, decoded: &[u8]) -> Vec<DialogueIrOp> {
    zelda3_dialogue::parse_dialogue_ir(dialogue_flags, decoded)
}

pub fn legacy_glyph_run_dialogue_ir(
    dialogue_flags: u8,
    decoded: &[u8],
    glyph_run_dialogue_offsets: &[u16],
    run_index: usize,
) -> Option<DialogueIrOp> {
    LegacyDialogueView::new(dialogue_flags, decoded, glyph_run_dialogue_offsets)
        .ir_for_glyph_run(run_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_legacy_glyph_run_offset_to_dialogue_ir() {
        let decoded = [
            0,
            1,
            zelda3_dialogue::TEXT_COMMAND_START_US + zelda3_dialogue::TEXT_CMD_2,
        ];
        let view = LegacyDialogueView::new(0, &decoded, &[1]);

        assert_eq!(
            view.ir_for_glyph_run(0).map(|op| op.kind),
            Some(DialogueIrKind::Glyph { code: 1 })
        );
    }

    #[test]
    fn ignores_unknown_legacy_glyph_run_offsets() {
        let decoded = [0, 1];
        let view = LegacyDialogueView::new(0, &decoded, &[UNKNOWN_DIALOGUE_OFFSET]);

        assert_eq!(view.ir_for_glyph_run(0), None);
    }
}
