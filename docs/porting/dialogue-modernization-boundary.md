# Dialogue Modernization Boundary

The dialogue modernization path keeps three boundaries separate:

- `zelda3-dialogue` is the pure modern text model. It owns message bytecode
  decoding, semantic IR, glyph labels, editable source text compilation, and
  semantic VWF layout. It must not depend on `zelda3`, live WRAM, PPU state,
  renderer types, or parity tooling.
- `zelda3-compat` is the legacy bridge. It may understand old decoded message
  buffers, dialogue flags, glyph-run offsets, and compatibility sentinels, but
  it remains independent of `ZeldaState`. Runtime code can use this crate while
  old message state still drives text.
- `parity` is the oracle/tool crate. It owns RAM/frame/checkpoint/scanning
  verification and must not become a production dependency for modern rendering
  or dialogue authoring.

The intended migration is:

1. Legacy bytecode and RAM state produce decoded message buffers.
2. `zelda3-compat` maps those legacy buffers and offsets to `zelda3-dialogue`
   IR for transitional rendering.
3. Capture attaches both the full decoded message IR and per-glyph
   `DialogueIrKind` to live frames.
4. Capture also attaches the active source message id and source-authored IR.
   This IR is expanded from the message bytecode/dictionary but not from runtime
   substitutions, so operations like `player_name` and `number` remain semantic
   commands at the source boundary.
5. `ModernFrame` carries both IR streams forward so independent text layout and
   future semantic command rendering do not have to reach back into `ZeldaState`,
   WRAM, or compatibility helpers.
6. `zelda3-dialogue` expands source-authored IR with explicit runtime
   substitutions (`player_name`, `number`) into the glyph-producing render IR
   used for message layout when source IR is available. Legacy decoded-text IR
   remains the fallback.
7. `zelda3-dialogue` lays out the render IR into message-buffer-local VWF glyph
   placements using the VWF width table supplied by the runtime/asset side.
   The layout also tracks active semantic text color commands per glyph. Capture
   forwards the BG3 VWF origin tile so extraction can convert those semantic
   placements into screen-space draw runs.
8. The renderer prefers semantic layout-derived VWF draw runs for PNG glyph
   rendering and stats. Legacy-emitted glyph runs remain capture/debug evidence
   and a fallback for old or synthetic non-semantic frames. Color command state
   is carried on the draw runs, and VWF rendering prefers exact
   `palette_bg3_text_color_xx` PNG atlas entries generated from HUD text palette
   rows, falling back to `palette_bg3_text_main` for old or custom atlases that
   only provide the main glyph sheet.
9. Editable source text is the build authority for `kDialogue`: extraction
   verifies generated source text against expanded bytecode, and
   `zelda3-dialogue` validates the source document, enforces contiguous message
   ids/counts, and packs it back into legacy bytecode as needed.
10. `parity` continues proving vanilla behavior without shaping the modern
   authoring/runtime APIs.
