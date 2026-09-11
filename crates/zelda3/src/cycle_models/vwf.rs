//! Cycle model of the dialogue glyph renderer: `RenderText_Draw_MessageCharacters`
//! (`$0E:C984`), the per-command handlers it dispatches through
//! `RenderText_Draw_HandleNext` (`$0E:C9FD`), and the variable-width glyph
//! path `RenderText_Draw_RenderCharacter` (`$0E:CA6C`) → `VWF_RenderSingle`
//! (`$0E:CAB8`) → `VWF_RenderCharacter` (`$0E:CB5E`).
//!
//! Every constant is one instruction (or one basic block) of the disassembly
//! priced by `scripts/rom_block_costs.py`: slow ROM 8 master cycles per
//! fetched byte, WRAM 8, internal 6; `abs,X`/`abs,Y` reads add 6 with 16-bit
//! index registers or when an 8-bit index crosses a page. The routine is
//! entered at m8/x8 (`Text_Render` reaches it through `JumpTableLocal`) with
//! the data bank `$0E`.
//!
//! The phases the translated engine budgets (`messaging.rs`) map onto the
//! ROM as follows; each function documents the address range it covers.
//!
//! - **click**: `$0E:C984` (loop restart) through the click SFX store at
//!   `$0E:CAC9`, i.e. up to label `$0E:CACC`: [`glyph_click_master_cycles`].
//! - **post-click entry**: `$0E:CACC` through the drawing setup, up to the
//!   first row of `VWF_RenderCharacter` at `$0E:CBD1`:
//!   [`glyph_post_click_setup_master_cycles`].
//! - **drawing**: the sixteen glyph rows `$0E:CBD1..$0E:CCF7`, whose cost
//!   depends on the glyph width, the x position within the 8-pixel tile and
//!   the font bits themselves (set pixels take the `EOR` path, clear pixels
//!   the `AND` path): [`glyph_drawing_master_cycles`].
//! - **restart / return**: `VWF_RenderSingle`'s epilogue, the speed-0 loop
//!   restart in `RenderText_Draw_RenderCharacter_All` or the handler's exit
//!   at `$0E:C9F5`: [`RENDER_SINGLE_EPILOGUE_MASTER_CYCLES`],
//!   [`render_all_continuation_master_cycles`], [`HANDLER_EXIT_MASTER_CYCLES`].
//! - **caller suffix**: from the handler's `RTS` back through `RenderText`
//!   and `Module0E_Interface`'s scroll-register copies to the main loop's
//!   `JSR NMI_PrepareSprites`: [`caller_suffix_master_cycles`] and the
//!   `MAIN_LOOP_*` constants.
//!
//! The ROM keeps a fixed-width-era tile cursor in `$1CDD`
//! (`dialogue_msg_src_offs`, dead in the C port) and `$1CE6`
//! (`text_next_position`) that the dispatcher clamps on every iteration and
//! the speed-0 restart tests; both only change which comparisons are taken,
//! but they do change the count, so the model carries them as
//! [`DispatchCursor`].
//!
//! The test at the bottom runs the whole routine on the shadow CPU for every
//! glyph of the font at every tile column, every line speed, both line
//! transition states, the dispatch cursor's clamp classes, and each modeled
//! command, and demands equality with the composite model.

// Instruction pricing (master cycles).
const BRANCH_TAKEN: u64 = 22; // opcode 8 + operand 8 + 6 internal
const BRANCH_NOT_TAKEN: u64 = 16;
const BRA: u64 = 22;
const JSR_ABS: u64 = 46;
const JSL_LONG: u64 = 62;
const RTS: u64 = 42;
const IMM_8: u64 = 16;
const ABS_8: u64 = 32; // LDA/STA abs, 8-bit
const INC_DEC_ABS_8: u64 = 46;
const INC_ABS_16: u64 = 62;
const REP_SEP: u64 = 22;

/// `JumpTableLocal` (`$00:8781`) entered by `JSL` with 8-bit index
/// registers: `STY $03 : PLY : STY $00 : REP #$30 : AND #$00FF : ASL : TAY
/// : PLA : STA $01 : INY : LDA [$00],Y : STA $00 : SEP #$30 : LDY $03 :
/// JML [$00]` = 414. Every dispatch below pays `JSL_LONG + JUMP_TABLE_LOCAL`.
const JUMP_TABLE_LOCAL: u64 = 414;

/// The ROM's fixed-width text cursor state the dispatcher clamps:
/// `$1CDD` (`dialogue_msg_src_offs`, a 16-bit word) and `$1CE6`
/// (`text_next_position`). `Text_Initialize` zeroes both;
/// `Text_LoadCharacterBuffer` leaves the message's source byte count in
/// `$1CDD`; `VWF_SetLine` stores `0`/`$28`/`$50` and zeroes `$1CE6`; a
/// completed scroll stores `$50` and zeroes `$1CE6`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DispatchCursor {
    pub(crate) src_offs: u16,
    pub(crate) next_position: u8,
}

impl DispatchCursor {
    /// The cursor `VWF_SetLine` (`$0E:D0C9`) leaves for command byte
    /// `$74`/`$75`/`$76` (`kVWF_LinePositions[byte & 3]`).
    pub(crate) fn after_set_line(command: u8) -> Self {
        const LINE_POSITIONS: [u16; 4] = [0x0000, 0x0028, 0x0050, 0x0000];
        Self {
            src_offs: LINE_POSITIONS[usize::from(command & 3)],
            next_position: 0,
        }
    }

    /// The cursor a completed `RenderText_Draw_Scroll` leaves (`$0E:D08C`).
    pub(crate) fn after_scroll_completion() -> Self {
        Self {
            src_offs: 0x0050,
            next_position: 0,
        }
    }
}

/// `$0E:C984..$0E:C9F4` plus `RenderText_Draw_HandleNext` (`$0E:C9FD`): the
/// message-loop dispatch from the loop head through the `JSR HandleNext`,
/// its `JSL JumpTableLocal` and the jump-table jump, up to the handler's
/// first instruction. `buffer_byte` is the character-buffer byte at
/// `$7F:1200 + $1CD9` (a letter below `$66`, or a command). Returns the cost
/// and the cursor the dispatch stored back to `$1CDD`/`$1CE6`.
pub(crate) fn dispatch_master_cycles(
    cursor: DispatchCursor,
    buffer_byte: u8,
) -> (u64, DispatchCursor) {
    let mut master = 0u64;
    let DispatchCursor {
        mut src_offs,
        mut next_position,
    } = cursor;
    loop {
        // C984: REP #$30 : LDA $1CDD : LDY #$0000 : CMP #$0063 : BCC C999
        master += 22 + 40 + 24 + 24;
        if src_offs >= 0x63 {
            // C991: LDA #$0000 : STY $1CE6 : BRA C9BB
            master += BRANCH_NOT_TAKEN + 24 + 40 + BRA;
            src_offs = 0;
            next_position = 0;
        } else {
            // C999: CMP #$003B : BCC C9AB
            master += BRANCH_TAKEN + 24;
            let mut clamp_line_two = false;
            if src_offs < 0x3b {
                master += BRANCH_TAKEN;
            } else {
                // C99E: CMP #$0050 : BCS C9AB
                master += BRANCH_NOT_TAKEN + 24;
                if src_offs >= 0x50 {
                    master += BRANCH_TAKEN;
                } else {
                    // C9A3: LDA #$0050 : STY $1CE6 : BRA C9BB
                    master += BRANCH_NOT_TAKEN + 24 + 40 + BRA;
                    src_offs = 0x50;
                    next_position = 0;
                    clamp_line_two = true;
                }
            }
            if !clamp_line_two {
                // C9AB: CMP #$0013 : BCC C9BB
                master += 24;
                if src_offs < 0x13 {
                    master += BRANCH_TAKEN;
                } else {
                    // C9B0: CMP #$0028 : BCS C9BB
                    master += BRANCH_NOT_TAKEN + 24;
                    if src_offs >= 0x28 {
                        master += BRANCH_TAKEN;
                    } else {
                        // C9B5: LDA #$0028 : STY $1CE6
                        master += BRANCH_NOT_TAKEN + 24 + 40;
                        src_offs = 0x28;
                        next_position = 0;
                    }
                }
            }
        }
        // C9BB: STA $1CDD : CMP #$0012 : BEQ C9CD
        master += 40 + 24;
        let line_end_slot = if src_offs == 0x12 {
            master += BRANCH_TAKEN;
            true
        } else {
            // C9C3: CMP #$003A : BEQ C9CD
            master += BRANCH_NOT_TAKEN + 24;
            if src_offs == 0x3a {
                master += BRANCH_TAKEN;
                true
            } else {
                // C9C8: CMP #$0062 : BNE C9DD
                master += BRANCH_NOT_TAKEN + 24;
                if src_offs == 0x62 {
                    master += BRANCH_NOT_TAKEN;
                    true
                } else {
                    master += BRANCH_TAKEN;
                    false
                }
            }
        };
        if line_end_slot {
            // C9CD: LDA $1CE6 : AND #$0007 : CMP #$0006 : BCC C9DD
            master += 40 + 24 + 24;
            if next_position & 7 >= 6 {
                // C9D8: INC $1CDD : BRA C984
                master += BRANCH_NOT_TAKEN + INC_ABS_16 + BRA;
                src_offs = src_offs.wrapping_add(1);
                continue;
            }
            master += BRANCH_TAKEN;
        }
        break;
    }
    // C9DD: LDX $1CD9 : LDA $7F1200,X : AND #$007F : SEC : SBC #$0066 : BPL C9F0
    master += 40 + 48 + 24 + 14 + 24;
    if buffer_byte & 0x7f >= 0x66 {
        master += BRANCH_TAKEN;
    } else {
        // C9ED: LDA #$0000
        master += BRANCH_NOT_TAKEN + 24;
    }
    // C9F0: SEP #$30 : JSR RenderText_Draw_HandleNext
    master += REP_SEP + JSR_ABS;
    // C9FD: JSL JumpTableLocal, then the table jump to the handler.
    master += JSL_LONG + JUMP_TABLE_LOCAL;
    (
        master,
        DispatchCursor {
            src_offs,
            next_position,
        },
    )
}

/// `$0E:C9F5..$0E:C9FC`: the dispatcher's exit after a handler returns
/// (`LDA #$02 : STA $17 : STA $0710 : RTS`). Paid once per `JSR HandleNext`
/// the call made: a speed-0 glyph restart (`JMP $C984` in
/// `RenderText_Draw_RenderCharacter_All`) leaves the `JSR`'s return address
/// on the stack, so when the loop finally returns, the `RTS` chain runs this
/// block once for the last handler and once more for every restart before
/// it reaches `RenderText` (the shadow CPU confirms 114 per restart).
pub(crate) const HANDLER_EXIT_MASTER_CYCLES: u64 = IMM_8 + 24 + ABS_8 + RTS;

/// `RenderText_Draw_RenderCharacter` (`$0E:CA6C..$0E:CA78`) for a glyph whose
/// line delay has expired (`$1CD5 < 2`): the speed compare, the
/// `JSL JumpTableLocal` dispatch on the speed, and for speed 0 the
/// `JSR VWF_RenderSingle` of `RenderText_Draw_RenderCharacter_All`
/// (`$0E:CA99`). Ends at `VWF_RenderSingle`'s entry (`$0E:CAB8`).
pub(crate) fn letter_handler_prefix_master_cycles(speed_cur: u8) -> u64 {
    debug_assert!(
        speed_cur < 2,
        "a delayed glyph takes reduce_speed_master_cycles"
    );
    // CA6C: LDA $1CD5 : CMP #$02 : BCC CA75 (taken) : JSL JumpTableLocal
    let master = ABS_8 + IMM_8 + BRANCH_TAKEN + JSL_LONG + JUMP_TABLE_LOCAL;
    if speed_cur == 0 {
        // CA99: JSR VWF_RenderSingle
        master + JSR_ABS
    } else {
        master
    }
}

/// `RenderText_Draw_RenderCharacter` for a glyph whose line delay has not
/// expired (`$1CD5 >= 2`): `$0E:CA6C..$0E:CA78` with the delay clamped to 2,
/// then `RenderText_Draw_RenderCharacter_ReduceSpeed` (`$0E:CCF9`: `DEC
/// $1CD5 : RTS`), returning to `$0E:C9F5`.
pub(crate) fn reduce_speed_master_cycles() -> u64 {
    // CA6C: LDA $1CD5 : CMP #$02 : BCC (not taken) : LDA #$02 : JSL JumpTableLocal
    ABS_8 + IMM_8 + BRANCH_NOT_TAKEN + IMM_8 + JSL_LONG + JUMP_TABLE_LOCAL
        // CCF9: DEC $1CD5 : RTS
        + INC_DEC_ABS_8 + RTS
}

/// `VWF_RenderSingle` `$0E:CAB8..$0E:CACB`: the buffer read, the space
/// test and, for any glyph but `$59`, the click SFX store `STA $012F` at
/// `$0E:CAC9`. Ends at label `$0E:CACC`.
pub(crate) fn render_single_click_master_cycles(c: u8) -> u64 {
    // CAB8: REP #$10 : LDX $1CD9 : LDA $7F1200,X : CMP #$59 : BEQ CACC
    let master = 22 + 40 + 40 + 16;
    if c == 0x59 {
        master + BRANCH_TAKEN
    } else {
        // CAC5: SEP #$30 : LDA #$0C : STA $012F
        master + BRANCH_NOT_TAKEN + 22 + 16 + 32
    }
}

/// The engine's **click** phase: from the message-loop head `$0E:C984`
/// through the click store, up to `$0E:CACC`, for glyph `c` at line speed
/// `speed_cur` (0 or 1). Replaces `VWF_GLYPH_CLICK_MASTER_CYCLES`. Returns
/// the cost and the clamped dispatch cursor.
pub(crate) fn glyph_click_master_cycles(
    cursor: DispatchCursor,
    c: u8,
    speed_cur: u8,
) -> (u64, DispatchCursor) {
    let (dispatch, cursor) = dispatch_master_cycles(cursor, c);
    (
        dispatch
            + letter_handler_prefix_master_cycles(speed_cur)
            + render_single_click_master_cycles(c),
        cursor,
    )
}

/// The engine's **post-click entry** phase: `$0E:CACC..$0E:CAD7`
/// (`VWF_RenderSingle`'s `JSR VWF_RenderCharacter`) and
/// `VWF_RenderCharacter`'s prologue `$0E:CB5E..$0E:CBD0` (line transition,
/// width lookup, advance bookkeeping, font pointer and row index setup), up
/// to the first row at `$0E:CBD1`. `next_line_pending` is `$0720`
/// (`vwf_flag_next_line`) at entry. The width table `kVWF_RenderCharacter_widths`
/// at `$0E:CADF` is read with an 8-bit index, so glyph codes from `$21` up
/// cross into page `$CB` and pay 6 more.
pub(crate) fn glyph_post_click_setup_master_cycles(c: u8, next_line_pending: bool) -> u64 {
    // CACC: REP #$30 : LDA $1CDD : ASL : TAX : SEP #$30 : JSR VWF_RenderCharacter
    let mut master = 22 + 40 + 14 + 14 + 22 + JSR_ABS;
    // CB5E: SEP #$30 : PHB : PHK : PLB : REP #$20 : LDA $0720 : BEQ CB7C
    master += 22 + 22 + 22 + 28 + 22 + 40;
    if next_line_pending {
        // CB6A: LDY $0722 : LDA $CB4A,Y : STA $0726 : LDA $CB50,Y : STA $0724 : STZ $0720
        master += BRANCH_NOT_TAKEN + 32 + 40 + 40 + 40 + 40 + 40;
    } else {
        master += BRANCH_TAKEN;
    }
    // CB7C..CBD0: SEP #$20 : REP #$10 : STZ $03 : LDX $1CD9 : LDA $7F1200,X : SEP #$10
    // : TAY : LDA $CADF,Y : STA $02 : LDX $0724 : CLC : ADC $7EC230,X : STA $7EC231,X
    // : INX : STX $0724 : TYA : AND #$F0 : ASL : STA $00 : TYA : AND #$0F : ORA $00
    // : STA $0A : STZ $0B : REP #$20 : LDA #$8000 : STA $0D : LDY #$0E : STY $0F
    // : REP #$10 : LDA $7EC22F,X : AND #$00FF : ASL : STA $00 : LDX #$0000 : LDA $0A
    // : ASL : ASL : ASL : ASL : TAY
    master += 966;
    if c >= 0x21 {
        master += 6; // LDA $CADF,Y crosses into page $CB
    }
    master
}

/// The whole entry work of a glyph: click plus post-click setup, from
/// `$0E:C984` to `$0E:CBD1`. Replaces `VWF_GLYPH_ENTRY_MASTER_CYCLES`
/// together with the per-row fixed costs inside
/// [`glyph_drawing_master_cycles`] (the estimate folded those into the entry).
pub(crate) fn glyph_entry_master_cycles(
    cursor: DispatchCursor,
    c: u8,
    speed_cur: u8,
    next_line_pending: bool,
) -> (u64, DispatchCursor) {
    let (click, cursor) = glyph_click_master_cycles(cursor, c, speed_cur);
    (
        click + glyph_post_click_setup_master_cycles(c, next_line_pending),
        cursor,
    )
}

/// Font row words of glyph `c` in the order `VWF_RenderCharacter` reads
/// them: eight rows of the top tile (`$0E:8000 + r10 * 16 + 2i`) then
/// eight of the bottom tile (`(r10 + 16) * 16 + 2i`), with
/// `r10 = (c & $70) * 2 + (c & $0F)`. `font` holds the bytes at `$0E:8000`
/// (the engine's dialogue font memblk index 0 is the same data).
pub(crate) fn glyph_font_rows(font: &[u8], c: u8) -> [u16; 16] {
    let r10 = ((usize::from(c) & 0x70) * 2) + (usize::from(c) & 0x0f);
    let word = |offset: usize| {
        u16::from_le_bytes([
            font.get(offset).copied().unwrap_or(0),
            font.get(offset + 1).copied().unwrap_or(0),
        ])
    };
    let mut rows = [0u16; 16];
    for i in 0..8 {
        rows[i] = word(r10 * 16 + i * 2);
        rows[8 + i] = word((r10 + 16) * 16 + i * 2);
    }
    rows
}

/// Number of pixel columns the column loop draws for a glyph of `width`
/// starting at tile column `x & 7`: it stops at the glyph's width or at the
/// 8-pixel tile boundary, whichever comes first.
pub(crate) fn glyph_drawn_columns(width: u8, x: u8) -> u8 {
    width.min(8 - (x & 7))
}

/// One row's column loop (`$0E:CBF2..$0E:CC33`, identical at
/// `$0E:CC90..$0E:CCD1`): per column, `ASL $04 : BCC` selects the
/// `LDA/EOR/STA/BRA` path (140) for a set plane-0 pixel or the `LDA/AND/STA`
/// path (118) for a clear one, then the same for plane 1 via `ASL $05`;
/// `DEC $03 : BEQ` leaves when the width is exhausted, else `INY : CPY
/// #$0008 : BNE` leaves at the tile boundary. Returns the cost and the
/// shifted word left in `$04` (the seed for the next tile).
pub(crate) fn glyph_column_loop_master_cycles(word: u16, width: u8, x: u8) -> (u64, u16) {
    let mut master = 0u64;
    let mut word = word;
    let mut remaining = width;
    let mut column = x & 7;
    loop {
        // CBF2: ASL $04 : BCC CC03
        master += 38;
        if word & 0x0080 != 0 {
            // CBF6: LDA $7F0000,X : EOR $CB42,Y : STA $7F0000,X : BRA CC0E
            master += BRANCH_NOT_TAKEN + 40 + 38 + 40 + BRA;
        } else {
            // CC03: LDA $7F0000,X : AND $CB56,Y : STA $7F0000,X
            master += BRANCH_TAKEN + 40 + 38 + 40;
        }
        // CC0E: ASL $05 : BCC CC1F
        master += 38;
        if word & 0x8000 != 0 {
            master += BRANCH_NOT_TAKEN + 40 + 38 + 40 + BRA;
        } else {
            master += BRANCH_TAKEN + 40 + 38 + 40;
        }
        word = (word & 0x7f7f) << 1;
        // CC2A: DEC $03 : BEQ CC34
        master += 38;
        remaining = remaining.wrapping_sub(1);
        if remaining == 0 {
            master += BRANCH_TAKEN;
            break;
        }
        // CC2E: INY : CPY #$0008 : BNE CBF2
        master += BRANCH_NOT_TAKEN + 14 + 24;
        column += 1;
        if column == 8 {
            master += BRANCH_NOT_TAKEN;
            break;
        }
        master += BRANCH_TAKEN;
    }
    (master, word)
}

/// Which tile of the glyph a row belongs to; the two row loops differ in
/// their row prologue and loop-closing branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphHalf {
    /// Rows 0..8, the loop at `$0E:CBD1`.
    Top,
    /// Rows 8..16, the loop at `$0E:CC67`.
    Bottom,
}

/// One glyph row: the row prologue (`$0E:CBD1..$0E:CBF1` = 470 for the top
/// tile, `$0E:CC67..$0E:CC8F` = 556 for the bottom tile, which re-reads the
/// advance from `$7EC22F,X`), the column loop, the seed handling
/// (`$0E:CC34..$0E:CC43`: `REP #$20 : TXA : CLC : ADC #$0010 : TAX : LDA
/// $04 : BEQ`, plus `STA $7F0000,X` when the remaining font bits are not
/// zero) and the row epilogue (`PLY : INY : INY : LDX $06 : INX : INX :
/// CPX #$0010`, closed by `BNE CBD1` for the top tile and by `BEQ CCF1 :
/// BRL CC67` for the bottom one). `last_row` is the eighth row of the tile.
pub(crate) fn glyph_row_master_cycles(
    word: u16,
    width: u8,
    x: u8,
    half: GlyphHalf,
    last_row: bool,
) -> u64 {
    let mut master = match half {
        GlyphHalf::Top => 470,
        GlyphHalf::Bottom => 556,
    };
    let (columns, seed) = glyph_column_loop_master_cycles(word, width, x);
    master += columns;
    // CC34 / CCD2: REP #$20 : TXA : CLC : ADC #$0010 : TAX : LDA $04 : BEQ
    master += 22 + 14 + 14 + 24 + 14 + 32;
    if seed != 0 {
        // CC40 / CCDE: STA $7F0000,X (16-bit)
        master += BRANCH_NOT_TAKEN + 48;
    } else {
        master += BRANCH_TAKEN;
    }
    // CC44 / CCE2: PLY : INY : INY : LDX $06 : INX : INX : CPX #$0010
    master += 36 + 14 + 14 + 32 + 14 + 14 + 24;
    match (half, last_row) {
        // BNE CBD1
        (GlyphHalf::Top, false) => master + BRANCH_TAKEN,
        (GlyphHalf::Top, true) => master + BRANCH_NOT_TAKEN,
        // BEQ CCF1 : BRL CC67
        (GlyphHalf::Bottom, false) => master + BRANCH_NOT_TAKEN + 30,
        (GlyphHalf::Bottom, true) => master + BRANCH_TAKEN,
    }
}

/// `$0E:CC50..$0E:CC66` between the two row loops: `LDA $0726 : CLC : ADC
/// #$0150 : STA $08 : LDX #$0000 : LDA $0A : CLC : ADC #$0010 : ASL : ASL :
/// ASL : ASL : TAY`.
const BOTTOM_TILE_SETUP_MASTER_CYCLES: u64 = 274;

/// `$0E:CCF1..$0E:CCF7`, `VWF_RenderCharacter`'s epilogue: `INC $1CD9 : SEP
/// #$30 : PLB : RTS` (consumes the glyph and returns to `$0E:CAD8`).
const RENDER_CHARACTER_EPILOGUE_MASTER_CYCLES: u64 = INC_ABS_16 + REP_SEP + 28 + RTS;

/// The engine's **drawing** phase: `$0E:CBD1..$0E:CCF7`, the sixteen rows
/// of glyph `c` (`rows` from [`glyph_font_rows`]) of `width` pixels at
/// pixel position `x` (the advance `vwf_arr[i]`; only `x & 7` matters),
/// through `VWF_RenderCharacter`'s `RTS`. Replaces
/// `vwf_render_glyph_drawing_master_cycles` (`columns * 8_000`).
pub(crate) fn glyph_drawing_master_cycles(width: u8, x: u8, rows: [u16; 16]) -> u64 {
    let mut master = 0u64;
    for (i, word) in rows[..8].iter().enumerate() {
        master += glyph_row_master_cycles(*word, width, x, GlyphHalf::Top, i == 7);
    }
    master += BOTTOM_TILE_SETUP_MASTER_CYCLES;
    for (i, word) in rows[8..].iter().enumerate() {
        master += glyph_row_master_cycles(*word, width, x, GlyphHalf::Bottom, i == 7);
    }
    master + RENDER_CHARACTER_EPILOGUE_MASTER_CYCLES
}

/// `$0E:CAD8..$0E:CADE`, `VWF_RenderSingle`'s epilogue after
/// `VWF_RenderCharacter` returns: `LDA $1CD6 : STA $1CD5 : RTS` (reloads the
/// line delay). At speed 1 the `RTS` lands on `$0E:C9F5`; at speed 0 on
/// `$0E:CA9C`.
pub(crate) const RENDER_SINGLE_EPILOGUE_MASTER_CYCLES: u64 = ABS_8 + ABS_8 + RTS;

/// `RenderText_Draw_RenderCharacter_All` after its glyph (`$0E:CA9C..`):
/// `REP #$30 : LDA $1CDD : CMP #$0013 : BEQ : CMP #$003B : BEQ : CMP #$0063
/// : BEQ`, then either `SEP #$30 : JMP $C984` (the loop restart, `true`) or
/// `SEP #$30 : RTS` to `$0E:C9F5` (`false`). `cursor` is the dispatch
/// cursor as stored by the dispatch that preceded the glyph. A restart
/// also owes one more [`HANDLER_EXIT_MASTER_CYCLES`] when the call finally
/// returns (see that constant).
pub(crate) fn render_all_continuation_master_cycles(cursor: DispatchCursor) -> (u64, bool) {
    // CA9C: REP #$30 : LDA $1CDD : CMP #$0013 : BEQ CAB5
    let mut master = 22 + 40 + 24;
    for (index, limit) in [0x0013u16, 0x003b, 0x0063].into_iter().enumerate() {
        if index > 0 {
            master += 24; // CMP #$003B / CMP #$0063
        }
        if cursor.src_offs == limit {
            // CAB5: SEP #$30 : RTS
            return (master + BRANCH_TAKEN + REP_SEP + RTS, false);
        }
        master += BRANCH_NOT_TAKEN;
    }
    // CAB0: SEP #$30 : JMP RenderText_Draw_MessageCharacters
    (master + REP_SEP + 24, true)
}

/// The cost of one complete glyph from the loop head `$0E:C984`: entry,
/// drawing, `VWF_RenderSingle`'s epilogue and, at speed 0, the
/// `RenderText_Draw_RenderCharacter_All` continuation (which normally jumps
/// back to `$0E:C984`); at speed 1 the epilogue returns to `$0E:C9F5`, whose
/// [`HANDLER_EXIT_MASTER_CYCLES`] are not included. Returns the cost, the
/// stored dispatch cursor and whether the loop restarted.
pub(crate) fn glyph_master_cycles(
    cursor: DispatchCursor,
    c: u8,
    speed_cur: u8,
    next_line_pending: bool,
    width: u8,
    x: u8,
    rows: [u16; 16],
) -> (u64, DispatchCursor, bool) {
    let (entry, cursor) = glyph_entry_master_cycles(cursor, c, speed_cur, next_line_pending);
    let mut master =
        entry + glyph_drawing_master_cycles(width, x, rows) + RENDER_SINGLE_EPILOGUE_MASTER_CYCLES;
    if speed_cur == 0 {
        let (continuation, restarted) = render_all_continuation_master_cycles(cursor);
        master += continuation;
        (master, cursor, restarted)
    } else {
        (master, cursor, false)
    }
}

/// `VWF_SetLine` (`$0E:D0C9..$0E:D0F1`, commands `$74`-`$76`): `REP #$30 :
/// LDX $1CD9 : LDA $7F1200,X : AND #$0003 : ASL : TAX : LDA $D399,X : STA
/// $1CDD : LDA $D0C3,X : STA $0722 : LDA #$0001 : STA $0720 : INC $1CD9 :
/// SEP #$30 : STZ $1CE6 : RTS`. The end-of-line path; returns to `$0E:C9F5`.
pub(crate) const SET_LINE_MASTER_CYCLES: u64 = 556;

/// `RenderText_Draw_PlaySfx` (`$0E:D162`, command `$79`): `REP #$10 : LDX
/// $1CD9 : INX : LDA $7F1200,X : STA $012F : INX : STX $1CD9 : SEP #$30 : RTS`.
pub(crate) const PLAY_SFX_MASTER_CYCLES: u64 = 266;

/// `RenderText_Draw_SetSpeed` (`$0E:D176`, command `$7A`): as PlaySfx with
/// two stores (`STA $1CD6 : STA $1CD5`).
pub(crate) const SET_SPEED_MASTER_CYCLES: u64 = 298;

/// `$0E:CE6B` (commands `$6A`-`$6E`; only `$6E` reaches the render loop):
/// `REP #$10 : LDX $1CD9 : INX : LDA $7F1200,X : STA $1CEA : INX : STX
/// $1CD9 : SEP #$30 : RTS` (sets the scroll speed).
pub(crate) const SCROLL_SPEED_MASTER_CYCLES: u64 = 266;

/// `RenderText_Draw_NextImage` (`$0E:CCFE`, command `$67`) outside module
/// `$14`: `LDA $10 : CMP #$14 : BNE CD0E : REP #$30 : INC $1CD9 : SEP #$30
/// : RTS`. In module `$14` the handler calls `PaletteFilterHistory`
/// (priced by its own annotation): [`next_image_module_14_master_cycles`].
pub(crate) const NEXT_IMAGE_MASTER_CYCLES: u64 =
    24 + 16 + BRANCH_TAKEN + 22 + INC_ABS_16 + 22 + RTS;

/// `RenderText_Draw_NextImage` in module `$14`: `LDA $10 : CMP #$14 : BNE`
/// (not taken) `: JSL PaletteFilterHistory : LDA $7EC007 : BNE CD15`, then
/// `RTS` when the filter countdown is still running, else `REP #$30 : INC
/// $1CD9 : SEP #$30 : RTS`. The `JSL` is priced as the call only.
pub(crate) fn next_image_module_14_master_cycles(filter_countdown_running: bool) -> u64 {
    let master = 24 + 16 + BRANCH_NOT_TAKEN + JSL_LONG + 40;
    if filter_countdown_running {
        master + BRANCH_TAKEN + RTS
    } else {
        master + BRANCH_NOT_TAKEN + 22 + INC_ABS_16 + 22 + RTS
    }
}

/// `RenderText_Draw_Wait` (`$0E:D115`, command `$78`): the B-button test
/// (`LDA $F2 : AND #$80`), the countdown clamp, `JSL JumpTableLocal` on the
/// selector, then `Wait_Init` (`$0E:D138`, selector 0: table lookup and a
/// fall-through into the decrement), `Wait_End` (`$0E:D154`, selector 1:
/// consumes the two command bytes) or `VWF_WaitLoop_decCounter`
/// (`$0E:D14C`, selector 2). `b_held` is bit 7 of `$F2` (`joypad1L_last`),
/// `wait_countdown` the 16-bit `$1CE0`.
pub(crate) fn wait_master_cycles(b_held: bool, wait_countdown: u16) -> u64 {
    // D115: LDA $F2 : AND #$80 : BEQ D11F
    let mut master = 24 + 16;
    let selector = if b_held {
        // D11B: LDA #$01 : BRA D12C
        master += BRANCH_NOT_TAKEN + IMM_8 + BRA;
        1
    } else {
        // D11F: REP #$30 : LDA $1CE0 : CMP #$0002 : BCC D12C
        master += BRANCH_TAKEN + 22 + 40 + 24;
        if wait_countdown < 2 {
            master += BRANCH_TAKEN;
            wait_countdown
        } else {
            // D129: LDA #$0002
            master += BRANCH_NOT_TAKEN + 24;
            2
        }
    };
    // D12C: SEP #$30 : JSL JumpTableLocal
    master += REP_SEP + JSL_LONG + JUMP_TABLE_LOCAL;
    master
        + match selector {
            // D138: REP #$30 : LDX $1CD9 : LDA $7F1201,X : AND #$000F : ASL : TAX
            // : LDA $D3AF,X : STA $1CE0, falling into D14C.
            0 => 396,
            // D154: REP #$30 : INC $1CD9 : INC $1CD9 : SEP #$30 : STZ $1CE0 : RTS
            1 => 242,
            // D14C: REP #$30 : DEC $1CE0 : SEP #$30 : RTS
            _ => 148,
        }
}

/// `RenderText_Draw_PauseForInput` (`$0E:D230`, command `$7E`).
/// `countdown2` is `$1CE9` (`text_wait_countdown2`); `a_or_b_pressed` is
/// `(filtered_joypad_H | filtered_joypad_L) & $C0 != 0`, read only when the
/// countdown is zero.
pub(crate) fn pause_for_input_master_cycles(countdown2: u8, a_or_b_pressed: bool) -> u64 {
    // D230: LDA $1CE9 : BEQ D244
    let master = ABS_8;
    if countdown2 != 0 {
        // D235: DEC : STA $1CE9 : CMP #$01 : BNE D25A
        let master = master + BRANCH_NOT_TAKEN + 14 + ABS_8 + IMM_8;
        if countdown2 == 2 {
            // D23D: LDA #$24 : STA $012F : BRA D25A (the decremented value is 1)
            master + BRANCH_NOT_TAKEN + IMM_8 + ABS_8 + BRA + RTS
        } else {
            master + BRANCH_TAKEN + RTS
        }
    } else {
        // D244: LDA $F4 : ORA $F6 : AND #$C0 : BEQ D25A
        let master = master + BRANCH_TAKEN + ABS_8 + ABS_8 + IMM_8;
        if a_or_b_pressed {
            // D24E: REP #$30 : INC $1CD9 : SEP #$30 : LDA #$1C : STA $1CE9
            master + BRANCH_NOT_TAKEN + 22 + INC_ABS_16 + 22 + IMM_8 + ABS_8 + RTS
        } else {
            master + BRANCH_TAKEN + RTS
        }
    }
}

/// `RenderText_Draw_Terminate` (`$0E:D25B`, command `$7F`), the
/// end-of-message path. `countdown2` is `$1CE9`; `any_pressed` is
/// `(filtered_joypad_H | filtered_joypad_L) != 0`, read only when the
/// countdown is zero (the two joypad reads are direct page here).
pub(crate) fn terminate_master_cycles(countdown2: u8, any_pressed: bool) -> u64 {
    // D25B: LDA $1CE9 : BEQ D26F
    let master = ABS_8;
    if countdown2 != 0 {
        // D260: DEC : STA $1CE9 : CMP #$01 : BNE D27F
        let master = master + BRANCH_NOT_TAKEN + 14 + ABS_8 + IMM_8;
        if countdown2 == 2 {
            // D268: LDA #$24 : STA $012F : BRA D27F (the decremented value is 1)
            master + BRANCH_NOT_TAKEN + IMM_8 + ABS_8 + BRA + RTS
        } else {
            master + BRANCH_TAKEN + RTS
        }
    } else {
        // D26F: LDA $F4 : ORA $F6 : BEQ D27F
        let master = master + BRANCH_TAKEN + 24 + 24;
        if any_pressed {
            // D275: LDA #$04 : STA $1CD4 : LDA #$1C : STA $1CE9
            master + BRANCH_NOT_TAKEN + IMM_8 + ABS_8 + IMM_8 + ABS_8 + RTS
        } else {
            master + BRANCH_TAKEN + RTS
        }
    }
}

/// One scroll copy pass of `RenderText_Draw_Scroll` (`$0E:CFF9..$0E:D0B9`
/// without the outer branch): `REP #$30 : STZ $00`, the 126-iteration
/// 16-word shift loop at `$0E:CFFD` (910 per iteration, `BCC` taken 125
/// times), the 21 `STZ` column clears, and `SEP #$30 : LDA $1CDF : CLC :
/// ADC #$01 : STA $1CDF : AND #$0F` up to the `BNE`.
const SCROLL_PASS_MASTER_CYCLES: u64 =
    22 + 32 + 126 * 910 + 125 * 6 + 21 * 40 + 22 + 40 + 14 + 16 + 40 + 16;

/// What a `RenderText_Draw_Scroll` call did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollStep {
    pub(crate) master: u64,
    /// The `$1CDF` line counter after the call.
    pub(crate) line_counter: u8,
    /// The pass whose counter reached a multiple of 16 completed the
    /// scroll: the command byte was consumed, `$1CDD = $50`, `$1CE6 = 0`,
    /// `$0722 = 4`, `$0720 = 1`.
    pub(crate) completed: bool,
}

/// `RenderText_Draw_Scroll` (`$0E:CFE2..$0E:D0C2`, command `$73`): the
/// prologue (`PHB : LDA #$7F : PHA : PLB : LDA $F2 : AND #$80 : BEQ`, one
/// `LDA long $1CEA` on either side, `STA $02`), then `scroll_speed + 1`
/// copy passes at most (`DEC $02 : BMI` exits when the counter goes
/// negative, `JMP $CFF9` repeats), each pass ending with `BNE D0BA` when
/// the incremented `$1CDF` is not a multiple of 16; when it is, the
/// completion block `$0E:D08C` (498) runs and the call returns. Exit is
/// `PLB : RTS` to `$0E:C9F5`. `b_held` is bit 7 of `$F2`.
pub(crate) fn scroll_master_cycles(b_held: bool, scroll_speed: u8, line_counter: u8) -> ScrollStep {
    // CFE2: PHB : LDA #$7F : PHA : PLB : LDA $F2 : AND #$80 : BEQ CFF3
    let mut master = 22 + 16 + 22 + 28 + 24 + 16;
    if b_held {
        // CFED: LDA $7E1CEA : BRA CFF7
        master += BRANCH_NOT_TAKEN + 40 + BRA;
    } else {
        // CFF3: LDA $7E1CEA
        master += BRANCH_TAKEN + 40;
    }
    // CFF7: STA $02
    master += 24;
    let mut line_counter = line_counter;
    let mut passes_left = scroll_speed;
    loop {
        master += SCROLL_PASS_MASTER_CYCLES;
        line_counter = line_counter.wrapping_add(1);
        if line_counter & 0x0f == 0 {
            // D08C..D0B9: consume the command, reset the line cursor, STZ $02
            master += BRANCH_NOT_TAKEN + 498;
            // D0BA: DEC $02 : BMI D0C1 (taken) : PLB : RTS
            master += 38 + BRANCH_TAKEN + 28 + RTS;
            return ScrollStep {
                master,
                line_counter,
                completed: true,
            };
        }
        // D0BA: DEC $02 : BMI D0C1
        master += BRANCH_TAKEN + 38;
        if passes_left == 0 {
            master += BRANCH_TAKEN + 28 + RTS;
            return ScrollStep {
                master,
                line_counter,
                completed: false,
            };
        }
        // D0BE: JMP $CFF9
        master += BRANCH_NOT_TAKEN + 24;
        passes_left -= 1;
    }
}

/// The caller suffix after a handler returns through `$0E:C9F5`: the
/// dispatcher exit ([`HANDLER_EXIT_MASTER_CYCLES`], whose `RTS` returns to
/// `RenderText` since `Messaging_Text_Near` and `Text_Render` reached the
/// handler through `JumpTableLocal`), `RenderText`'s `PLB : RTL`
/// (`$0E:C446`, 72), and `Module0E_Interface`'s scroll-register copies
/// `$00:F84E..$00:F875` (`REP #$21 : LDA $E2 : ADC $011A : STA $011E : LDA
/// $E8 : CLC : ADC $011C : STA $0122 : LDA $E0 : CLC : ADC $011A : STA
/// $0120 : LDA $E6 : CLC : ADC $011C : STA $0124 : SEP #$20 : RTL`, 578),
/// whose `RTL` lands on the main loop's `JSR NMI_PrepareSprites` at
/// `$00:805A`. Replaces the fixed part of `VWF_CALLER_SUFFIX_MASTER_CYCLES`;
/// the rest of that estimate is `NMI_PrepareSprites` itself, priced by its
/// own annotation, plus [`MAIN_LOOP_PREPARE_SPRITES_CALL_MASTER_CYCLES`] and
/// [`MAIN_LOOP_TAIL_MASTER_CYCLES`].
pub(crate) fn caller_suffix_master_cycles() -> u64 {
    HANDLER_EXIT_MASTER_CYCLES + (28 + 44) + 578
}

/// `$00:805A`: `JSR NMI_PrepareSprites` (the call instruction only).
pub(crate) const MAIN_LOOP_PREPARE_SPRITES_CALL_MASTER_CYCLES: u64 = JSR_ABS;

/// `$00:805D..$00:8060` after `NMI_PrepareSprites` returns: `STZ $12 : BRA
/// $8034`.
pub(crate) const MAIN_LOOP_TAIL_MASTER_CYCLES: u64 = 24 + BRA;

/// `$00:8034`: one spin of the main loop's NMI wait (`LDA $12 : BEQ $8034`,
/// taken) until the interrupt arrives.
pub(crate) const MAIN_LOOP_IDLE_SPIN_MASTER_CYCLES: u64 = 24 + BRANCH_TAKEN;

/// The fixed call chain from `Module0E_Interface`'s `SEP #$30 : JSL
/// RunInterface` (`$00:F848`) to the handler: `RunInterface` (`$00:F89A`,
/// 264, long table jump), `RenderText` (`$0E:C440`: `PHB : PHK : PLB : JSR`,
/// 118), `Messaging_Text_Near` (`$0E:C448`: `LDA $1CD8 : JSL JumpTableLocal`,
/// 508) and `Text_Render` (`$0E:C8D9`: `LDA $1CD4 : JSL JumpTableLocal`,
/// 508), landing on `$0E:C984`. What precedes it in `Module0E_Interface`
/// (`Sprite_Main`, `LinkOam_Main`, ...) is data dependent and priced by
/// those routines' own annotations.
pub(crate) fn handler_entry_prefix_master_cycles() -> u64 {
    (REP_SEP + JSL_LONG)
        + 264
        + (22 + 22 + 28 + JSR_ABS)
        + (ABS_8 + JSL_LONG + JUMP_TABLE_LOCAL) * 2
}

/// The render-phase WRAM the routine reads and writes, for the composite
/// model of one `RenderText_Draw_MessageCharacters` call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MessageCharactersState {
    /// `$1CD9`: the character-buffer read position.
    pub(crate) read_pos: u16,
    pub(crate) cursor: DispatchCursor,
    /// `$1CD6` / `$1CD5`: the line speed and its countdown.
    pub(crate) speed: u8,
    pub(crate) speed_cur: u8,
    /// `$0720`: a line transition is pending.
    pub(crate) next_line_pending: bool,
    /// `$0722`: the current line (0, 2 or 4).
    pub(crate) current_line: u8,
    /// `$0724`: the glyph cursor `i` into `vwf_arr`.
    pub(crate) glyph_cursor: u8,
    /// `$7E:C230..`: `vwf_arr`, the pixel advance before each glyph.
    pub(crate) advance: [u8; 0x100],
    /// `$1CE9`: `text_wait_countdown2`.
    pub(crate) countdown2: u8,
    /// `$1CE0`: `text_wait_countdown`.
    pub(crate) wait_countdown: u16,
    /// `$1CEA`: `dialogue_scroll_speed`.
    pub(crate) scroll_speed: u8,
    /// `$1CDF`: the scroll line counter.
    pub(crate) scroll_line_counter: u8,
    /// `$F2` bit 7: B held on the last frame.
    pub(crate) b_held: bool,
    /// `$F4 | $F6`: the filtered joypad word.
    pub(crate) filtered_joypad: u8,
    /// `$10`: the main module.
    pub(crate) main_module: u8,
}

/// Composite model of one `RenderText_Draw_MessageCharacters` call from
/// `$0E:C984` to the `RTS` at `$0E:C9FC`, over the decoded character buffer
/// (`$7F:1200`), the font bytes at `$0E:8000` and the width table at
/// `$0E:CADF`. Speed-0 glyphs restart the loop as the ROM does; the state is
/// updated the way the handlers update WRAM. Returns `None` for a command
/// this model does not price (the choice menus, `$70` and the
/// `Text_LoadCharacterBuffer`-only commands), leaving `state` partially
/// advanced.
pub(crate) fn message_characters_call_master_cycles(
    state: &mut MessageCharactersState,
    buffer: &[u8],
    font: &[u8],
    widths: &[u8],
) -> Option<u64> {
    let mut master = 0u64;
    let mut handle_next_calls = 0u64;
    loop {
        handle_next_calls += 1;
        let byte = buffer
            .get(usize::from(state.read_pos))
            .copied()
            .unwrap_or(0x7f);
        let parameter = buffer
            .get(usize::from(state.read_pos) + 1)
            .copied()
            .unwrap_or(0);
        let (dispatch, cursor) = dispatch_master_cycles(state.cursor, byte);
        let c = byte & 0x7f;
        if c < 0x66 {
            if state.speed_cur >= 2 {
                master += dispatch + reduce_speed_master_cycles();
                state.cursor = cursor;
                state.speed_cur -= 1;
                break;
            }
            {
                // glyph_master_cycles prices the dispatch itself.
                let speed_cur = state.speed_cur;
                let next_line_pending = state.next_line_pending;
                if next_line_pending {
                    const LINE_POSITIONS: [u8; 3] = [0x00, 0x40, 0x80];
                    state.glyph_cursor = LINE_POSITIONS[usize::from(state.current_line >> 1)];
                    state.next_line_pending = false;
                }
                let width = widths.get(usize::from(c)).copied().unwrap_or(0);
                let i = usize::from(state.glyph_cursor);
                let x = state.advance[i];
                state.advance[(i + 1) & 0xff] = x.wrapping_add(width);
                state.glyph_cursor = state.glyph_cursor.wrapping_add(1);
                let (glyph, cursor, restarted) = glyph_master_cycles(
                    state.cursor,
                    c,
                    speed_cur,
                    next_line_pending,
                    width,
                    x,
                    glyph_font_rows(font, c),
                );
                master += glyph;
                state.cursor = cursor;
                state.read_pos = state.read_pos.wrapping_add(1);
                state.speed_cur = state.speed;
                if restarted {
                    continue;
                }
                break;
            }
        }
        master += dispatch;
        state.cursor = cursor;
        match c {
            0x67 => {
                if state.main_module == 0x14 {
                    return None;
                }
                master += NEXT_IMAGE_MASTER_CYCLES;
                state.read_pos = state.read_pos.wrapping_add(1);
                break;
            }
            0x6e => {
                master += SCROLL_SPEED_MASTER_CYCLES;
                state.scroll_speed = parameter;
                state.read_pos = state.read_pos.wrapping_add(2);
                break;
            }
            0x73 => {
                let step = scroll_master_cycles(
                    state.b_held,
                    state.scroll_speed,
                    state.scroll_line_counter,
                );
                master += step.master;
                state.scroll_line_counter = step.line_counter;
                if step.completed {
                    state.read_pos = state.read_pos.wrapping_add(1);
                    state.cursor = DispatchCursor::after_scroll_completion();
                    state.current_line = 4;
                    state.next_line_pending = true;
                }
                break;
            }
            0x74..=0x76 => {
                master += SET_LINE_MASTER_CYCLES;
                state.cursor = DispatchCursor::after_set_line(byte);
                state.current_line = (byte & 3) * 2;
                state.next_line_pending = true;
                state.read_pos = state.read_pos.wrapping_add(1);
                break;
            }
            0x78 => {
                master += wait_master_cycles(state.b_held, state.wait_countdown);
                if state.b_held || state.wait_countdown == 1 {
                    state.wait_countdown = 0;
                    state.read_pos = state.read_pos.wrapping_add(2);
                } else if state.wait_countdown == 0 {
                    // kText_WaitDurations ($0E:D3AF).
                    const WAIT_DURATIONS: [u16; 16] = [
                        0x001f, 0x003f, 0x005e, 0x007d, 0x009c, 0x00bc, 0x00db, 0x00fa, 0x0119,
                        0x0139, 0x0158, 0x0177, 0x0196, 0x01b6, 0x01d5, 0x01f4,
                    ];
                    state.wait_countdown =
                        WAIT_DURATIONS[usize::from(parameter & 0x0f)].wrapping_sub(1);
                } else {
                    state.wait_countdown -= 1;
                }
                break;
            }
            0x79 => {
                master += PLAY_SFX_MASTER_CYCLES;
                state.read_pos = state.read_pos.wrapping_add(2);
                break;
            }
            0x7a => {
                master += SET_SPEED_MASTER_CYCLES;
                state.speed = parameter;
                state.speed_cur = parameter;
                state.read_pos = state.read_pos.wrapping_add(2);
                break;
            }
            0x7e => {
                let pressed = state.filtered_joypad & 0xc0 != 0;
                master += pause_for_input_master_cycles(state.countdown2, pressed);
                if state.countdown2 != 0 {
                    state.countdown2 -= 1;
                } else if pressed {
                    state.read_pos = state.read_pos.wrapping_add(1);
                    state.countdown2 = 0x1c;
                }
                break;
            }
            0x7f => {
                let pressed = state.filtered_joypad != 0;
                master += terminate_master_cycles(state.countdown2, pressed);
                if state.countdown2 != 0 {
                    state.countdown2 -= 1;
                } else if pressed {
                    state.countdown2 = 0x1c;
                }
                break;
            }
            _ => return None,
        }
    }
    Some(master + handle_next_calls * HANDLER_EXIT_MASTER_CYCLES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{lorom_offset, RomCpuCheckpoint, RomCpuTimingRun};

    fn test_rom() -> Option<Vec<u8>> {
        let path = std::env::var_os("ZELDA3_ROM")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
            });
        let mut rom = std::fs::read(path).ok()?;
        if rom.len() % 0x400 == 0x200 {
            rom.drain(..0x200);
        }
        Some(rom)
    }

    /// The font at `$0E:8000` (`$4000` bytes: 256 tiles of 16 bytes) and
    /// the width table at `$0E:CADF` (`$63` glyphs).
    fn font_and_widths(rom: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let font = lorom_offset(0x0e_8000).unwrap();
        let widths = lorom_offset(0x0e_cadf).unwrap();
        (
            rom[font..font + 0x4000].to_vec(),
            rom[widths..widths + 0x63].to_vec(),
        )
    }

    fn state_with(buffer_read_pos: u16) -> MessageCharactersState {
        MessageCharactersState {
            read_pos: buffer_read_pos,
            cursor: DispatchCursor::default(),
            speed: 1,
            speed_cur: 1,
            next_line_pending: false,
            current_line: 0,
            glyph_cursor: 0,
            advance: [0; 0x100],
            countdown2: 0,
            wait_countdown: 0,
            scroll_speed: 0,
            scroll_line_counter: 0,
            b_held: false,
            filtered_joypad: 0,
            main_module: 0x07,
        }
    }

    /// Run `RenderText_Draw_MessageCharacters` on the shadow CPU from the
    /// given state and buffer, through the caller suffix, stopping at the
    /// main loop's `JSR NMI_PrepareSprites` (`$00:805A`).
    fn shadow_master_cycles(
        rom: &[u8],
        state: &MessageCharactersState,
        buffer: &[u8],
        line_ptr: u16,
    ) -> Option<u64> {
        // The stack RenderText_Draw_MessageCharacters sees: the JSR return into
        // RenderText ($0E:C446), RenderText's saved data bank, RunInterface's
        // JSL return into Module0E_Interface ($00:F84E) and Module_MainRouting's
        // JSL return into the main loop ($00:805A).
        const STACK: [u8; 9] = [0x45, 0xc4, 0x00, 0x4d, 0xf8, 0x00, 0x59, 0x80, 0x00];
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x0e_c984,
            stop_pc: 0x00_805a,
            a: 0,
            x: 0,
            y: 0,
            sp: 0x01f6,
            dp: 0,
            db: 0x0e,
            carry: false,
            zero: false,
            overflow: false,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01f7,
            stack_bytes: &STACK,
        };
        let mut ram = vec![0u8; 0x2_0000];
        let word = |ram: &mut [u8], address: usize, value: u16| {
            ram[address] = value as u8;
            ram[address + 1] = (value >> 8) as u8;
        };
        ram[0x11200..0x11200 + buffer.len()].copy_from_slice(buffer);
        word(&mut ram, 0x1cd9, state.read_pos);
        word(&mut ram, 0x1cdd, state.cursor.src_offs);
        ram[0x1ce6] = state.cursor.next_position;
        ram[0x1cd6] = state.speed;
        ram[0x1cd5] = state.speed_cur;
        word(&mut ram, 0x0720, u16::from(state.next_line_pending));
        word(&mut ram, 0x0722, u16::from(state.current_line));
        word(&mut ram, 0x0724, u16::from(state.glyph_cursor));
        word(&mut ram, 0x0726, line_ptr);
        ram[0xc230..0xc330].copy_from_slice(&state.advance);
        ram[0x1ce9] = state.countdown2;
        word(&mut ram, 0x1ce0, state.wait_countdown);
        ram[0x1cea] = state.scroll_speed;
        ram[0x1cdf] = state.scroll_line_counter;
        ram[0x00f2] = if state.b_held { 0x80 } else { 0 };
        ram[0x00f4] = state.filtered_joypad;
        ram[0x0010] = state.main_module;
        let sram = vec![0u8; 0x2000];
        let ppu = snes::PpuState::default();
        let dma = snes::DmaState::default();
        let mut run =
            RomCpuTimingRun::new(rom, &ram, &sram, &ppu, &dma, [0; 4], checkpoint).ok()?;
        let mut master = 0u64;
        for _ in 0..400_000 {
            if run.is_complete() {
                return Some(master);
            }
            master += u64::from(run.step().master_cycles);
        }
        eprintln!(
            "shadow run did not finish; pc={:06x} sp={:04x}",
            run.pc(),
            run.stack_pointer()
        );
        None
    }

    fn check(
        rom: &[u8],
        font: &[u8],
        widths: &[u8],
        state: MessageCharactersState,
        buffer: &[u8],
        line_ptr: u16,
        label: &str,
        mismatches: &mut Vec<String>,
    ) -> bool {
        let Some(measured) = shadow_master_cycles(rom, &state, buffer, line_ptr) else {
            mismatches.push(format!("{label}: shadow run did not finish"));
            return false;
        };
        let mut modeled_state = state;
        let modeled =
            message_characters_call_master_cycles(&mut modeled_state, buffer, font, widths)
                .map(|master| master + caller_suffix_master_cycles() - HANDLER_EXIT_MASTER_CYCLES);
        if modeled != Some(measured) {
            mismatches.push(format!("{label}: modeled {modeled:?} measured {measured}"));
        }
        true
    }

    #[test]
    fn glyph_model_matches_the_shadow_cpu_for_every_glyph_column_and_speed() {
        let Some(rom) = test_rom() else {
            eprintln!("no ROM available; skipping the VWF cycle model check");
            return;
        };
        let (font, widths) = font_and_widths(&rom);
        let mut checked = 0;
        let mut mismatches = Vec::new();
        let line_ptrs = [0x0000u16, 0x02a0, 0x0540];
        for c in 0..0x63u8 {
            for x in 0..8u8 {
                for speed in [0u8, 1] {
                    // A glyph at tile column x on line (c % 3), followed by the
                    // terminator so the speed-0 restart dispatches a command.
                    let mut state = state_with(0);
                    state.speed = speed;
                    state.speed_cur = speed;
                    state.current_line = (c % 3) * 2;
                    state.glyph_cursor = 0x10 + (c % 5);
                    state.advance[usize::from(state.glyph_cursor)] = x + 8 * (c % 4);
                    // Exercise the cursor's clamp classes across the sweep.
                    state.cursor.src_offs =
                        [0x0000, 0x0005, 0x0030, 0x0050, 0x0062, 0x0100, 0x0027]
                            [usize::from(c) % 7];
                    let line_ptr = line_ptrs[usize::from(c % 3)];
                    let label = format!(
                        "glyph c={c:#04x} x={x} speed={speed} src_offs={:#06x}",
                        state.cursor.src_offs
                    );
                    if check(
                        &rom,
                        &font,
                        &widths,
                        state,
                        &[c, 0x7f],
                        line_ptr,
                        &label,
                        &mut mismatches,
                    ) {
                        checked += 1;
                    }
                }
            }
        }
        // Pending line transitions, the delayed-glyph path and the cursor's
        // line-end slots (with and without the INC $1CDD restart) on a few
        // glyphs, at speed 0 so RenderCharacter_All's continuation runs.
        for (c, x) in [(0x00u8, 0u8), (0x20, 3), (0x21, 5), (0x59, 7), (0x62, 2)] {
            for (src_offs, next_position) in [
                (0x0012u16, 0u8),
                (0x0012, 6),
                (0x003a, 7),
                (0x0062, 6),
                (0x0013, 0),
                (0x003b, 0),
                (0x0063, 0),
                (0x004f, 0),
            ] {
                for next_line_pending in [false, true] {
                    for speed in [0u8, 1, 2, 5] {
                        let mut state = state_with(4);
                        state.speed = speed.min(1);
                        state.speed_cur = speed;
                        state.cursor = DispatchCursor {
                            src_offs,
                            next_position,
                        };
                        state.current_line = 2;
                        state.next_line_pending = next_line_pending;
                        state.glyph_cursor = 0x22;
                        state.advance[0x22] = x;
                        state.advance[0x40] = x + 8;
                        let buffer = [0x59, 0x59, 0x59, 0x59, c, c, 0x7f];
                        let label = format!(
                            "glyph c={c:#04x} x={x} speed={speed} src_offs={src_offs:#06x} next_position={next_position} next_line_pending={next_line_pending}"
                        );
                        if check(
                            &rom,
                            &font,
                            &widths,
                            state,
                            &buffer,
                            0x02a0,
                            &label,
                            &mut mismatches,
                        ) {
                            checked += 1;
                        }
                    }
                }
            }
        }
        eprintln!(
            "VWF glyph cycle model: checked={checked} mismatches={}",
            mismatches.len()
        );
        assert!(checked > 1_500, "only {checked} glyph runs were checked");
        assert!(
            mismatches.is_empty(),
            "VWF glyph cycle model differs from the shadow CPU on {} of {checked} runs: {:#?}",
            mismatches.len(),
            &mismatches[..mismatches.len().min(12)]
        );
    }

    #[test]
    fn command_model_matches_the_shadow_cpu_for_each_modeled_command() {
        let Some(rom) = test_rom() else {
            eprintln!("no ROM available; skipping the VWF command cycle model check");
            return;
        };
        let (font, widths) = font_and_widths(&rom);
        let mut checked = 0;
        let mut mismatches = Vec::new();
        let mut cases: Vec<(String, MessageCharactersState, Vec<u8>)> = Vec::new();
        for command in [0x74u8, 0x75, 0x76] {
            cases.push((
                format!("set_line {command:#04x}"),
                state_with(0),
                vec![command, 0x59],
            ));
        }
        cases.push(("play_sfx".to_string(), state_with(0), vec![0x79, 0x05]));
        cases.push(("set_speed".to_string(), state_with(0), vec![0x7a, 0x03]));
        cases.push(("scroll_speed".to_string(), state_with(0), vec![0x6e, 0x02]));
        cases.push(("next_image".to_string(), state_with(0), vec![0x67]));
        for b_held in [false, true] {
            for wait_countdown in [0u16, 1, 2, 9] {
                let mut state = state_with(0);
                state.b_held = b_held;
                state.wait_countdown = wait_countdown;
                cases.push((
                    format!("wait b_held={b_held} countdown={wait_countdown}"),
                    state,
                    vec![0x78, 0x03],
                ));
            }
            for scroll_speed in [0u8, 1, 2] {
                for line_counter in [0u8, 0x0e, 0x0f, 0x1d] {
                    let mut state = state_with(0);
                    state.b_held = b_held;
                    state.scroll_speed = scroll_speed;
                    state.scroll_line_counter = line_counter;
                    cases.push((
                        format!("scroll b_held={b_held} speed={scroll_speed} counter={line_counter:#04x}"),
                        state,
                        vec![0x73],
                    ));
                }
            }
        }
        for command in [0x7eu8, 0x7f] {
            for countdown2 in [0u8, 1, 2, 0x1c] {
                for joypad in [0u8, 0x10, 0x40, 0x80] {
                    let mut state = state_with(0);
                    state.countdown2 = countdown2;
                    state.filtered_joypad = joypad;
                    cases.push((
                        format!(
                            "command {command:#04x} countdown2={countdown2} joypad={joypad:#04x}"
                        ),
                        state,
                        vec![command],
                    ));
                }
            }
        }
        for (label, state, buffer) in cases {
            if check(
                &rom,
                &font,
                &widths,
                state,
                &buffer,
                0,
                &label,
                &mut mismatches,
            ) {
                checked += 1;
            }
        }
        eprintln!(
            "VWF command cycle model: checked={checked} mismatches={}",
            mismatches.len()
        );
        assert!(checked >= 60, "only {checked} command runs were checked");
        assert!(
            mismatches.is_empty(),
            "VWF command cycle model differs from the shadow CPU on {} of {checked} runs: {:#?}",
            mismatches.len(),
            &mismatches
        );
    }

    #[test]
    fn column_loop_prices_each_pixel_path_and_exit() {
        // All pixels clear, width 3 from column 0: three columns of the AND
        // path, the last leaving through DEC $03 : BEQ.
        let (master, seed) = glyph_column_loop_master_cycles(0x0000, 3, 0);
        assert_eq!(
            master,
            2 * (38 + 22 + 118 + 38 + 22 + 118 + 38 + 16 + 14 + 24 + 22)
                + (38 + 22 + 118 + 38 + 22 + 118 + 38 + 22)
        );
        assert_eq!(seed, 0);
        // Both planes set on the first column, then the tile boundary at
        // column 8 stops a wider glyph with the remaining bits as the seed.
        let (master, seed) = glyph_column_loop_master_cycles(0x80ff, 6, 6);
        assert_eq!(
            master,
            (38 + 16 + 140 + 38 + 16 + 140 + 38 + 16 + 14 + 24 + 22)
                + (38 + 16 + 140 + 38 + 22 + 118 + 38 + 16 + 14 + 24 + 16)
        );
        assert_eq!(seed, 0x00fc);
        assert_eq!(glyph_drawn_columns(6, 6), 2);
        assert_eq!(glyph_drawn_columns(3, 0), 3);
    }

    #[test]
    fn fixed_chain_costs_are_the_listing_totals() {
        // $00:F848 SEP/JSL (84) + RunInterface (264) + RenderText prologue (118)
        // + Messaging_Text_Near (508) + Text_Render (508).
        assert_eq!(
            handler_entry_prefix_master_cycles(),
            84 + 264 + 118 + 508 + 508
        );
        // C9F5 exit (114) + RenderText PLB/RTL (72) + Module0E scroll copies (578).
        assert_eq!(caller_suffix_master_cycles(), 114 + 72 + 578);
        assert_eq!(MAIN_LOOP_PREPARE_SPRITES_CALL_MASTER_CYCLES, 46);
        assert_eq!(MAIN_LOOP_TAIL_MASTER_CYCLES, 24 + 22);
        assert_eq!(MAIN_LOOP_IDLE_SPIN_MASTER_CYCLES, 24 + 22);
        // CCFE: LDA $10 : CMP #$14 : BNE (not taken) : JSL : LDA $7EC007 : BNE.
        assert_eq!(
            next_image_module_14_master_cycles(true),
            24 + 16 + 16 + 62 + 40 + 22 + 42
        );
        assert_eq!(
            next_image_module_14_master_cycles(false),
            24 + 16 + 16 + 62 + 40 + 16 + 22 + 62 + 22 + 42
        );
        assert_eq!(NEXT_IMAGE_MASTER_CYCLES, 24 + 16 + 22 + 22 + 62 + 22 + 42);
    }

    #[test]
    fn dispatch_clamps_the_cursor_like_the_rom() {
        let (_, cursor) = dispatch_master_cycles(
            DispatchCursor {
                src_offs: 0x100,
                next_position: 7,
            },
            0x00,
        );
        assert_eq!(cursor, DispatchCursor::default());
        let (_, cursor) = dispatch_master_cycles(
            DispatchCursor {
                src_offs: 0x15,
                next_position: 7,
            },
            0x00,
        );
        assert_eq!(
            cursor,
            DispatchCursor {
                src_offs: 0x28,
                next_position: 0
            }
        );
        let (_, cursor) = dispatch_master_cycles(
            DispatchCursor {
                src_offs: 0x40,
                next_position: 3,
            },
            0x00,
        );
        assert_eq!(
            cursor,
            DispatchCursor {
                src_offs: 0x50,
                next_position: 0
            }
        );
        // A line-end slot with a full next position steps to the next slot,
        // which the restarted dispatch clamps to the following line.
        let (_, cursor) = dispatch_master_cycles(
            DispatchCursor {
                src_offs: 0x12,
                next_position: 6,
            },
            0x00,
        );
        assert_eq!(
            cursor,
            DispatchCursor {
                src_offs: 0x28,
                next_position: 0
            }
        );
        let (_, cursor) = dispatch_master_cycles(
            DispatchCursor {
                src_offs: 0x62,
                next_position: 6,
            },
            0x00,
        );
        assert_eq!(cursor, DispatchCursor::default());
    }
}
