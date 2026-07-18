# Manual 1:1 Parity Audit

This ledger tracks manual C-to-Rust parity review. It is intentionally separate from
`scripts/signature_drift.py`; entries require direct comparison against
`../zelda3/src`.

For a scan-friendly covered/open/next view, see
`docs/porting/manual_parity_status.md`.

Verdicts:

- `verified`: control flow, RAM/global writes, table offsets, asset offsets, and loop bounds were compared directly.
- `fixed`: mismatch found during manual review and corrected in the Rust tree.
- `unverified`: not yet manually reviewed.

## Method

For each C function:

1. Read the C source and neighboring constants/macros in `../zelda3/src`.
2. Read the Rust implementation in this checkout.
3. Compare side effects, pointer destinations, table indexing, byte-vs-word writes, loop bounds, and helper calls.
4. Record exact function verdicts here.
5. Run compile/hygiene checks after fixes.

## 2026-05-31 Name-Entry END Tile and Audio Oracle Split Pass

Scope: manually compare the name-entry finalize path in
`crates/zelda3/src/select_file.rs` against
`../zelda3/src/select_file.c`, and separate C
lockstep state parity from sample-exact external audio parity.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| name-entry END tile finalize | `crates/zelda3/src/select_file.rs` | fixed | `NameFile_DoTheNaming` now matches C's `t == 0x6f` behavior. C does not return when the selected tile is END; it falls through into the save-slot initialization, checksum, `ReturnToFileSelect`, `irq_flag = 0xff`, and `sound_effect_1 = 0x2c`. Rust previously returned after the character-selection branch and stayed in `main=4/sub=3`. |
| lockstep trace clarity | `zelda3-bin/src/main.rs` | fixed | `TraceState` now prints `grid_col` (`selectfile_var3`) separately from `name_col` (`selectfile_var4`) and no longer reports a misleading SRAM-name value from WRAM-only data. |
| C lockstep vs sample-exact audio oracle | `zelda3-bin/src/main.rs` | fixed | Playable C lockstep and lockstep-render comparison no longer fail on APUI command-port mismatch. C lockstep remains the RAM/PPU/SRAM/render oracle; sample-exact audio comparison is reserved for `--compare-snes9x-oracle`, which records the snes9x WAV/reference artifacts. |

Checks after this pass:

- `cargo fmt -p zelda3 -p zelda3-bin --check`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 1896 --input-script target/parity-failures/1780231020-16427/input.txt --trace-state` completes through the previously failing frame and returns to `main=1/sub=1` at frame 1895.

## 2026-05-31 Ending Intro/Credits Scene Load Pass

Scope: manually compare the first ending/intro scene-load cluster in
`crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c` and `ending.h`, without using
the progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| intro setup and background registers | `crates/zelda3/src/ending.rs` | fixed | `Intro_SetupScreen`, `Intro_LoadTextPointersAndPalettes`, and `Intro_InitializeBackgroundSettings` match C's NMI disable, force blank, TM/TS, half-slot CHR load, overworld music load, 17 palette/VRAM clears, R16/R18 setup, message pointer generation, and overworld palette load. This pass restored the missing C `zelda_ppu_write(BG1SC/BG2SC/BG3SC, 0x13/0x03/0x63)` side effects in the background-settings helper. |
| credits overworld scene-load trio | `crates/zelda3/src/ending.rs` | verified | `Credits_LoadScene_Overworld_PrepGFX`, `Credits_LoadScene_Overworld_Overlay`, and `Credits_LoadScene_Overworld_LoadMap` match C's force blank/tilemap erase, CGWSEL, ending table lookup, normal-vs-special overworld load, music/ambient clears, animated-tile selection, sprite graphics/palette selection, tileset/palette/HUD/font/fixed-color setup, overlay decrement, map build, sprite prep, R16 clear, and subsubmodule reset. |
| credits scroll/cool-background/dungeon scene load | `crates/zelda3/src/ending.rs` | verified | `Credits_OperateScrollingAndTileMap`, `Credits_LoadCoolBackground`, and `Credits_LoadScene_Dungeon` match C's camera-scroll/map-scroll gate, cool-background tileset/palette/overlay setup, BG1 scroll clears, submodule decrement, dungeon entrance load, torch/darkness clears, room draw, animated dungeon tiles, sprite/palette setup, BGMODE/INIDISP/R16 writes, and sprite prep. |
| Ganon-emerges state machine | `crates/zelda3/src/ending.rs` | verified | `Module18_GanonEmerges` matches C's temporary BG scroll offsetting around `Sprite_Main`, restoration order, map-state cases 0..8, duck/key save handoff, pyramid location transition, fade-out/force-blank/HUD rebuild, pyramid area load, overlay load, bat-crash spawn handoff, player drop-off delay, and final `LinkOam_Main`. |

Checks after this pass:

- `cargo test -q -p zelda3 intro_background_settings_write_ppu_tilemap_regs`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Ending Triforce Room Poly Pass

Scope: manually compare `Module19_TriforceRoom` and the Triforce/polyhedral
helper cluster in `crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| Triforce room state machine | `crates/zelda3/src/ending.rs` | verified | `Module19_TriforceRoom` matches C cases 0..14: Link reset and reset-interface handoff, mosaic/palette filtering, force blank/credits song load, special-area/overlay setup, tileset and palette selection, `Module08_02_LoadAndAdvance` with restored submodule state, Link coordinate/direction setup, mosaic fade, message/poly setup, text render, approach/fade handoff, scroll-copy tail, velocity/moving-animation gate, and `LinkOam_Main`. Byte-only writes such as `BYTE(R16)`, `BYTE(link_y_coord)`, `BYTE(link_x_coord)`, `BYTE(darkening_or_lightening_screen)`, and `HIBYTE(BG1HOFS_copy2)` were checked directly. |
| Triforce/credits poly setup helpers | `crates/zelda3/src/ending.rs` | verified | `TriforceRoom_PrepGFXSlotForPoly`, `Credits_InitializePolyhedral`, `AdvancePolyhedral`, and `Credits_AnimateTheTriangles` match C's graphics-index load, common-sprite init, subtype/is-inited arrays, `INIDISP_copy`, `poly_config1`, submodule advance, helper call order, frame counter, NMI-thread flag, and one-step guard behavior. |
| Triforce poly state machine | `crates/zelda3/src/ending.rs` | fixed | `TriforceRoom_HandlePoly` now matches C's case 0 fallthrough into case 1. The previous Rust shape advanced `poly_b`/`poly_a` twice in step 0; C subtracts `poly_config1`, optionally advances the step/subsubmodule, then executes the shared case-1 rotation exactly once. Cases 2 and 3 match C's `triforce_ctr`, config increment, `(poly_b - 10 & 0x7f)`/`(uint8)(poly_a - 11)` gate, palette word writes, CGRAM flag, timer, final step bookkeeping, and double-return clear. |

Checks after this pass:

- `cargo test -q -p zelda3 triforce_poly_step0_falls_through_once_like_c`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Ending Credits Scene Dispatch/Sprite Prep Pass

Scope: manually compare `Module1A_Credits`,
`Credits_LoadNextScene_Overworld`, `Credits_LoadNextScene_Dungeon`, and
`Credits_PrepAndLoadSprites` in `crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| credits module dispatch and OAM region bases | `crates/zelda3/src/ending.rs` | fixed | `Module1A_Credits` matches C's `kEndSequence_Funcs` dispatch table for submodules 0..38. This pass fixed `oam_region_base` writes to use C's `uint16` indexing: base words `0x0030`, `0x01d0`, and `0x0000` at `$0fe0/$0fe2/$0fe4` instead of an overlapping byte/word write. |
| next-scene loader dispatch | `crates/zelda3/src/ending.rs` | verified | `Credits_LoadNextScene_Overworld` matches C's three-entry `kEndSequence0_Funcs` dispatch followed by `Credits_AddEndingSequenceText`; `Credits_LoadNextScene_Dungeon` matches C's dungeon scene-load call followed by the same ending text helper. |
| ending sprite reset and scene-specific prep | `crates/zelda3/src/ending.rs` | fixed | `Credits_PrepAndLoadSprites` now matches C's per-sprite `SpritePrep_ResetProperties(k)` before clearing `sprite_state`, `sprite_flags5`, and `sprite_defl_bits`. Scene cases 0..15 were compared against C's gotos: overworld/dungeon coordinate base math, delay/velocity/head-dir/type/oam/flags/AI/Z mutations, and reverse loops match. |

Checks after this pass:

- `cargo test -q -p zelda3 credits_`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Ending Credits Scroll/Fade Draw Pass

Scope: manually compare `Credits_ScrollScene_Overworld`,
`Credits_ScrollScene_Dungeon`, `Credits_HandleSceneFade`, and the immediate
shadow/draw helper handoff in `crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| credits scene scroll wrappers | `crates/zelda3/src/ending.rs` | verified | `Credits_ScrollScene_Overworld` and `Credits_ScrollScene_Dungeon` match C's reverse delay decrement, `submodule_index >> 1` scene lookup, `R16 >= 0x40 && !(R16 & 1)` movement gate, overworld Link velocity assignment, dungeon BG2 scroll mutation, and follow-up helper call order. |
| credits fade/draw scene cases | `crates/zelda3/src/ending.rs` | verified | `Credits_HandleSceneFade` cases 0..15 were compared against C's OAM flag tables, sprite type/velocity/delay/graphics mutations, sound effects, sparkle draws, active/preexisting sprite calls, shadow calls, random sparkle offsets, Link DMA writes, and case-specific helper ordering. |
| credits fade tail timing | `crates/zelda3/src/ending.rs` | fixed | Rust now matches C's `if (!(R16 & 1) && !--INIDISP_copy) submodule_index++; else R16++;` behavior. The previous Rust branch decremented `INIDISP_copy` on even `R16` but failed to increment `R16` when the fade was not complete, stretching the fade state. |

Checks after this pass:

- `cargo test -q -p zelda3 credits_scene_fade`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Ending Intro Module/Sprite Animation Pass

Scope: manually compare the intro module dispatch, intro memory setup, Triforce
poly setup, intro run-step state machine, intro sprite animation helpers, and
sword/flash OAM helper in `crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| intro module and memory setup | `crates/zelda3/src/ending.rs` | verified-open | `Polyhedral_InitializeThread`, `Module00_Intro`, `Intro_Init`, `Intro_Init_Continue`, `Intro_Clear1kbBlocksOfWRAM`, `Intro_InitializeMemory_darken`, `FadeMusicAndResetSRAMMirror`, `Intro_InitializeTriforcePolyThread`, `Intro_InitGfx_Helper`, and `LoadTriforceSpritePalette` match C's module dispatch, skip-input gate, force blank/tilemap/palette/theme setup, 1KB WRAM clearing stride, R16/R18 updates, poly thread stack bytes, common-sprite load, sprite init/subtype writes, poly registers, thread flags, and palette writes. Runtime-open because the playable host can enable the Rust-only `rom_startup_timing` delay shim in `Intro_Init_Continue`; the C-shaped path remains the default core path when that flag is false. |
| intro run-step and sprite animation | `crates/zelda3/src/ending.rs` | verified | `Intro_HandleAllTriforceAnimations`, `Scene_AnimateEverySprite`, `Intro_AnimateTriforce`, `Intro_RunStep`, `Intro_AnimOneObj`, `Intro_SpriteType_A_0`, `Intro_SpriteType_B_0`, `AnimateSceneSprite_DrawTriangle`, `Intro_CopySpriteType4ToOam`, `InitializeSceneSprite_Copyright`, `AnimateSceneSprite_Copyright`, `InitializeSceneSprite_Sparkle`, `AnimateSceneSprite_Sparkle`, `AnimateSceneSprite_AddObjectsToOamBuffer`, `AnimateSceneSprite_MoveTriangle`, and `Intro_DisplayLogo` match C dispatch, reverse sprite loop, frame counters, step-index/timer/poly mutations, music handoff, NMI upload flag, signed coordinate initialization, signed velocity/subpixel movement, OAM allocation increments, left/right triangle tables, copyright/sparkle tables, and logo OAM. |
| intro fade/sword helper | `crates/zelda3/src/ending.rs` | fixed | `IntroZeldaFadein`, `Intro_FadeInBg`, `Intro_SwordComingDown`, `Intro_WaitPlayer`, `Intro_SetupSwordAndIntroFlash`, and `Intro_PeriodicSwordAndIntroFlash` match C's Triforce animation calls, frame-gated palette fades, TM/TS writes, submodule/subsubmodule transitions, thread flags, final module handoff, sword position/OAM tables, sparkle sub-states, palette-flash counters, and SFX writes. This pass fixed the `DimFlashes` feature side effect so the sword flash ORs `$05` into `COLDATA_copy*` when `kFeatures0_DimFlashes` is set, matching C instead of always OR-ing `$1f`. |
| verification | ending intro module/sprite slice | verified | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 1900 --input-script scripts/inputs/title-start.txt` pass; the lockstep route completes with `WRAM fnv1a64 = a2104f594c71a7ed`. |

## 2026-05-31 Ending Triforce/Credits Sprite Helper Pass

Scope: manually compare the remaining Triforce-room triangle helpers, credits
triangle helpers, credits sprite draw helpers, and credits camera-scroll helper
in `crates/zelda3/src/ending.rs` against
`../zelda3/src/ending.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| Triforce-room triangle helpers | `crates/zelda3/src/ending.rs` | verified | `InitializeSceneSprite_TriforceRoomTriangle`, `Intro_SpriteType_B_456`, and `AnimateTriforceRoomTriangle_HandleContracting` match C's initial X/Y/velocity tables, OAM-copy early return, movement-before-step dispatch, frame-gated acceleration, velocity zeroing, contraction clamps at `$10/$f0`, `triforce_ctr` countdown, and final Y snap. |
| credits triangle helper | `crates/zelda3/src/ending.rs` | verified | `InitializeSceneSprite_CreditsTriangle` and `AnimateSceneSprite_CreditsTriangle` match C's triangle coordinate tables, palette load, OAM copy, movement call, non-credits-state sprite clear, 80-frame state cap, and signed X/Y acceleration. |
| credits sprite draw helpers | `crates/zelda3/src/ending.rs` | verified | `Credits_SpriteDraw_DrawShadow`, `EndSequence_DrawShadow2`, `Ending_Func2`, `Credits_SpriteDraw_ActivateAndRunSprite`, `Credits_SpriteDraw_PreexistingSpriteDraw`, `Credits_SpriteDraw_Single`, `Credits_SpriteDraw_SetShadowProp`, `Credits_SpriteDraw_AddSparkle`, `Credits_SpriteDraw_WalkLinkAwayFromPedestal`, `Credits_SpriteDraw_MoveSquirrel`, and `Credits_SpriteDraw_CirclingBirds` match C's OAM/shadow property writes, delay and graphics tables, active-sprite submodule override, preexisting sprite draw path, direct draw-multiple table indexing, sparkle reset/delay behavior, Link DMA sequence, squirrel velocity cycling, and bird acceleration direction flips. |
| credits camera scroll helper | `crates/zelda3/src/ending.rs` | verified | `Credits_HandleCameraScrollControl` matches C's signed BG2 scroll mutation, overworld accumulator/transition-bit updates, `byte_7E069E` word writes, BG1 subpixel divisor cases for special overlays, overlay `$9c/$97/$9d` special movement, and room `$0181` BG1 mirror behavior. |

Checks after this pass:

- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 1900 --input-script scripts/inputs/title-start.txt`

## 2026-05-31 CPU Addressing/RTS Hook Pass

Scope: manually compare the 65816 addressing helpers and selected
control-transfer paths in `crates/snes/src/cpu_step.rs` against
`../zelda3/snes/cpu.c`, plus the Rust
`RunEmulatedFuncSilent` split in `crates/zelda3/src/zelda_cpu_infra.rs`
against `../zelda3/src/zelda_cpu_infra.c`. This pass
did not use the progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| 65816 addressing helpers | `crates/snes/src/cpu_step.rs` | verified | `adr_imm`, direct-page, indexed direct-page, indirect direct-page, stack-relative, absolute, indexed absolute, absolute-long, absolute-long-X, and indexed indirect jump helpers match C's pointer fetch order, bank composition, 16-bit wrapping, page-cross cycle conditions, and write/read high-address behavior. |
| JSR/JSL/RTI/RTS/RTL/JMP/branch control flow | `crates/snes/src/cpu_step.rs` | fixed | Rust matched C's return-address pushes, pull order, PC increment after RTS/RTL, RTI flag/bank restoration, branch signed offsets, and indirect JMP wrapping. This pass tightened the RTS/RTL breakpoint overshoot check from debug-only to a real `assert_eq!`, matching C's `assert(cpu->sp == cpu->spBreakpoint)`. |
| BRK patch table and emulated-function return hook | `crates/snes/src/cpu_step.rs`, `crates/zelda3/src/zelda_cpu_infra.rs` | verified | The Rust BRK patch table mirrors C's patched addresses and restart opcodes for the Zelda-specific breakpoints. C's `HookedFunctionRts` clears `g_calling_asm_from_c`; Rust keeps `snes` independent from `zelda3` by clearing `sp_breakpoint`, which `run_emulated_func_silent` polls before restoring CPU state and syncing the game side. |

Checks after this pass:

- `cargo test -q -p snes cpu_step`
- `cargo fmt -p snes -p zelda3 --check`
- `cargo check -q -p snes -p zelda3 -p zelda3-bin`

## 2026-05-31 CPU Shift/Rotate/Inc-Dec Opcode Pass

Scope: manually compare the 65816 shift, rotate, increment, and decrement
opcode helpers plus their dispatch cases in `crates/snes/src/cpu_step.rs`
against `../zelda3/snes/cpu.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| shift/rotate memory helpers | `crates/snes/src/cpu_step.rs` | verified | `op_asl`, `op_lsr`, `op_rol`, and `op_ror` match C `cpu_asl`, `cpu_lsr`, `cpu_rol`, and `cpu_ror` for 8-bit vs 16-bit width, carry source/update order, Z/N flag computation, 16-bit extra cycles, and reversed write order for 16-bit read-modify-write memory ops. |
| accumulator shift/rotate opcodes | `crates/snes/src/cpu_step.rs` | verified | Dispatch cases `0x0a`, `0x2a`, `0x4a`, and `0x6a` match C accumulator ASL/ROL/LSR/ROR behavior, including preserving A high byte in 8-bit mode, rotating in the previous carry where applicable, setting carry from the original shifted-out bit, and calling `cpu_set_zn` with `mf`. |
| increment/decrement opcodes | `crates/snes/src/cpu_step.rs` | verified | Memory `op_inc`/`op_dec` and dispatch cases `0x1a`, `0x3a`, `0xc6`, `0xce`, `0xd6`, `0xde`, `0xe6`, `0xee`, `0xf6`, and `0xfe` match C wrapping/truncation behavior, accumulator high-byte preservation in 8-bit mode, 16-bit extra cycles, reversed memory write order, and Z/N flag updates. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 CPU Logical/BIT/TRB/TSB Opcode Pass

Scope: manually compare the 65816 logical accumulator operations, BIT, TRB, and
TSB helpers plus their dispatch cases in `crates/snes/src/cpu_step.rs` against
`../zelda3/snes/cpu.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| AND/ORA/EOR helpers and dispatch | `crates/snes/src/cpu_step.rs` | verified | `op_and`, `op_ora`, and `op_eor` match C `cpu_and`, `cpu_ora`, and `cpu_eor` for 8-bit A high-byte preservation, 16-bit extra cycle, read-word behavior, accumulator mutation, and Z/N flag updates. Dispatch cases `0x01/03/05/07/09/0d/0f/11/12/13/15/17/19/1d/1f`, `0x21/23/25/27/29/2d/2f/31/32/33/35/37/39/3d/3f`, and `0x41/43/45/47/49/4d/4f/51/52/53/55/57/59/5d/5f` use the same addressing helper variants as C. |
| BIT memory and immediate forms | `crates/snes/src/cpu_step.rs` | verified | `op_bit` matches C `cpu_bit` for Z from `A & value`, N/V copied from memory bits, 16-bit extra cycle, and no accumulator mutation. Immediate BIT case `0x89` matches C's immediate-only Z update and deliberately does not touch N/V. |
| TRB/TSB read-modify-write helpers | `crates/snes/src/cpu_step.rs` | verified | `op_trb` and `op_tsb` match C `cpu_trb` and `cpu_tsb`: Z is computed from `A & value`, 8-bit mode masks to the low accumulator byte, 16-bit mode adds two cycles, and the 16-bit memory write uses reversed read-modify-write ordering. Dispatch cases `0x04/0x0c` and `0x14/0x1c` use the same direct-page and absolute addressing forms as C. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 CPU Load/Store/Transfer Opcode Pass

Scope: manually compare the 65816 load, store, zero-store, and immediate
register-transfer opcode cluster in `crates/snes/src/cpu_step.rs` against
`../zelda3/snes/cpu.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| load helpers | `crates/snes/src/cpu_step.rs` | verified | `op_lda`, `op_ldx`, and `op_ldy` match C `cpu_lda`, `cpu_ldx`, and `cpu_ldy`: 8-bit A preserves the accumulator high byte, 8-bit X/Y load into the low register value, 16-bit mode adds one cycle, and Z/N flag width follows `mf` or `xf`. |
| store helpers | `crates/snes/src/cpu_step.rs` | verified | `op_sta`, `op_stx`, `op_sty`, and `op_stz` match C store behavior for 8-bit truncation, 16-bit extra cycle, normal word write ordering, and no flag updates. |
| store and transfer dispatch | `crates/snes/src/cpu_step.rs` | verified | Store cases `0x64/0x74/0x81/0x83/0x84/0x85/0x86/0x87/0x8c/0x8d/0x8e/0x8f/0x91/0x92/0x93/0x94/0x95/0x96/0x97/0x99/0x9c/0x9d/0x9e/0x9f` use the same addressing helper variants as C. Transfer cases `0x8a/0x98/0x9a/0x9b/0xa8/0xaa/0xba/0xbb` match C's width masking, stack-pointer assignment, and Z/N updates. |
| load dispatch | `crates/snes/src/cpu_step.rs` | verified | Load cases `0xa0/0xa1/0xa2/0xa3/0xa4/0xa5/0xa6/0xa7/0xa9/0xac/0xad/0xae/0xaf/0xb1/0xb2/0xb3/0xb4/0xb5/0xb6/0xb7/0xb9/0xbc/0xbd/0xbe/0xbf` match C addressing forms, including immediate X-width selection for LDX/LDY and M-width selection for LDA. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 CPU Arithmetic/Compare Opcode Pass

Scope: manually compare the 65816 ADC, SBC, CMP, CPX, and CPY helper bodies
plus direct dispatch cases in `crates/snes/src/cpu_step.rs` against
`../zelda3/snes/cpu.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| ADC helper | `crates/snes/src/cpu_step.rs` | verified | `op_adc` matches C `cpu_adc` for 8-bit and 16-bit widths, decimal-mode nibble adjustment order, V flag calculation before final decimal correction, C flag threshold, accumulator high-byte preservation in 8-bit mode, 16-bit extra cycle, and final Z/N update. |
| SBC helper | `crates/snes/src/cpu_step.rs` | verified | `op_sbc` matches C `cpu_sbc` for one's-complement value handling, decimal-mode borrow adjustment masks, V/C flag behavior, 8-bit accumulator preservation, 16-bit extra cycle, and final Z/N update. |
| compare helpers | `crates/snes/src/cpu_step.rs` | verified | `op_cmp`, `op_cpx`, and `op_cpy` match C `cpu_cmp`, `cpu_cpx`, and `cpu_cpy`: comparisons are implemented as add with inverted operand plus one, set C from the width-specific threshold, add the 16-bit extra cycle, and update Z/N using M width for A and X width for X/Y. |
| arithmetic and compare dispatch | `crates/snes/src/cpu_step.rs` | verified | Direct ADC cases `0x61/0x63/0x65/0x67/0x69/0x6d/0x6f/0x71/0x72/0x73/0x75/0x77/0x79/0x7d/0x7f`, CMP/CPX/CPY cases `0xc0/0xc1/0xc3/0xc4/0xc5/0xc7/0xc9/0xcc/0xcd/0xcf/0xd1/0xd2/0xd3/0xd5/0xd7/0xd9/0xdd/0xdf/0xe0/0xe4/0xec`, and SBC cases `0xe1/0xe3/0xe5/0xe7/0xe9/0xed/0xef/0xf1/0xf2/0xf3/0xf5/0xf7/0xf9/0xfd/0xff` use the same addressing helper variants as C. The C `goto` labels used by Zelda-specific BRK patches remain covered by the earlier BRK patch-table pass. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 CPU Stack/Flag/Block/Misc Opcode Pass

Scope: manually compare the remaining stack, flag, block-move, wait/stop, and
miscellaneous opcode cluster in `crates/snes/src/cpu_step.rs` against
`../zelda3/snes/cpu.c`, without using the
progress/signature scripts. JSR/JSL/RTI/RTS/RTL/JMP/branch control-flow details
remain covered by the earlier CPU addressing/RTS hook pass.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| stack push/pull opcodes | `crates/snes/src/cpu_step.rs` | verified | `0x08/0x0b/0x28/0x2b/0x48/0x4b/0x5a/0x68/0x7a/0x8b/0xab/0xd4/0xda/0xf4/0xfa` match C for pushed values, pull destinations, width-dependent extra cycles, A high-byte preservation on PLA, X/Y width handling, DB/DP flag updates, and PEI/PEA word sources. |
| flag and processor-state opcodes | `crates/snes/src/cpu_step.rs` | verified | `0x18/0x38/0x58/0x78/0xb8/0xc2/0xd8/0xe2/0xf8/0xfb` match C for direct flag mutation, REP/SEP via packed flags, and XCE's carry/emulation swap followed by `cpu_set_flags` so M/X width side effects are applied. |
| index increments and DP/SP transfers | `crates/snes/src/cpu_step.rs` | verified | `0x88/0xc8/0xca/0xe8` match C's X-width masking and Z/N updates for DEY/INY/DEX/INX. `0x1b/0x3b/0x5b/0x7b` match C's TCS/TSC/TCD/TDC transfer targets and full-width Z/N updates where C sets them. |
| block move and signed PC-relative opcodes | `crates/snes/src/cpu_step.rs` | verified | MVP `0x44` and MVN `0x54` match C's source/destination bank operand order, DB assignment, byte copy, A decrement, X/Y decrement-or-increment direction, PC rewind while A has not underflowed to `$ffff`, and 8-bit index masking. BRA/BRL/PER `0x80/0x82/0x62` match C's signed offset handling and pushed PC-relative word. |
| miscellaneous immediate/control opcodes | `crates/snes/src/cpu_step.rs` | verified | COP `0x02`, WDM `0x42`, WAI `0xcb`, STP `0xdb`, NOP `0xea`, XBA `0xeb`, JML indirect-long `0xdc`, and JSR indexed-indirect `0xfc` match C's operand consumption, status flags, wait/stop booleans, byte swap/Z/N update, and bank-byte wrap. Rust's `u16::wrapping_add(2)` in `0xdc` is equivalent to C's `(adr + 2) & 0xffff`. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 DSP Register Write Surface Pass

Scope: manually compare `DspState::read` and `DspState::write` in
`crates/snes/src/apu.rs` against `dsp_read` and `dsp_write` in
`../zelda3/snes/dsp.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| DSP register reads | `crates/snes/src/apu.rs` | verified | `DspState::read` matches C `dsp_read`: reads return the raw 0x80-byte DSP register RAM entry. |
| voice parameter writes | `crates/snes/src/apu.rs` | verified | Per-channel volume, pitch low/high, SRCN, ADSR1, ADSR2, and GAIN register writes match C's channel index, signed volume storage, 14-bit pitch masking, ADSR rate-table indexes, sustain-level calculation, direct-gain flag, gain mode, and gain value scaling. |
| key/reset/noise/echo writes | `crates/snes/src/apu.rs` | verified | KON, KOF, FLG, PMON, NON, EON, DIR, ESA, EDL, EFB, echo volumes, and FIR writes match C's side effects. KON clears `keyOn` after immediately restarting the sample, resets previous BRR flags/decode buffer/gain, loads the BRR pointer from DIR/SRCN with 16-bit wrapping, and chooses ADSR state 3 for gain mode or 0 otherwise. KOF enters release. FLG updates reset/mute/echo-write/noise-rate fields. |
| ENDX clear and backing RAM write | `crates/snes/src/apu.rs` | verified | ENDX writes match C's `val = 0` before the final `dsp->ram[adr] = val`: Rust writes zero to `ram[ENDX]` and returns before the shared backing-RAM store. All other handled registers fall through to the same backing-register write as C. |

Checks after this pass:

- `cargo check -q -p snes`
- `cargo test -q -p snes dsp_write_records_register_value`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 DSP Cycle/Mix/Pitch Pass

Scope: manually compare `DspState::cycle`, `handle_echo`,
`cycle_channel`, `handle_gain`, `get_sample`, `decode_brr`, and
`handle_noise` in `crates/snes/src/apu.rs` against the corresponding
`dsp_cycle`, `dsp_handleEcho`, `dsp_cycleChannel`, `dsp_handleGain`,
`dsp_getSample`, `dsp_decodeBrr`, and `dsp_handleNoise` functions in
`../zelda3/snes/dsp.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| DSP frame sample generation | `crates/snes/src/apu.rs` | verified | `DspState::cycle` matches C's eight-channel accumulation, per-add 16-bit clamp, master-volume scaling, echo-before-mute ordering, noise update timing, 534 stereo-sample buffer cap, sample offset increment, and even-cycle toggle. |
| echo FIR/mix/write path | `crates/snes/src/apu.rs` | verified | `handle_echo` matches C's echo buffer address math, signed half-sample reads, FIR index ordering, cast-to-int16 before the final FIR tap, output echo-volume mix, echo-input accumulation, feedback mix, even-sample masking, conditional echo RAM writes, FIR index wrap, echo-buffer index increment, and echo-delay reset. |
| channel pitch/sample/envelope path | `crates/snes/src/apu.rs` | fixed | `cycle_channel` matches C's pitch modulation, BRR decode trigger on counter overflow, pitch-counter truncation, noise-vs-Gaussian sample selection, reset-to-release behavior, direct-gain gate, rate-counter update/reset, gain handling, ENVX/OUTX writes, gain-scaled sample output, and sample_out update. This pass fixed pitch modulation for negative products: C assigns `(pitch * factor) >> 10` into a `uint16_t` before clamping, so negative products wrap and then clamp to `$3fff`; Rust previously clamped the signed product to zero. |
| gain, interpolation, BRR decode, noise | `crates/snes/src/apu.rs` | verified | `handle_gain`, `get_sample`, `decode_brr`, and `handle_noise` match C's attack/decay/sustain/gain/release math, exponential decrease promotion behavior, Gaussian interpolation and cast/clamp order, BRR loop/end flag behavior, header shift/filter decoding, 15-bit clipping, old/older sample updates, and noise LFSR update/reset. |

Checks after this pass:

- `cargo fmt -p snes --check`
- `cargo test -q -p snes dsp_pitch_modulation_wraps_negative_product_like_c`
- `cargo check -q -p snes`
- `git diff --check -- crates/snes/src/apu.rs docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 DSP Sample Extraction Pass

Scope: manually compare `DspState::get_samples` in `crates/snes/src/apu.rs`
against `dsp_getSamples` in `../zelda3/snes/dsp.c`,
without using the progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| frame-buffer resampling | `crates/snes/src/apu.rs` | verified-open | For normal callers with a correctly sized destination buffer and nonzero `samples_per_frame`, Rust matches C's `534.0 / samplesPerFrame` float adder, float location truncation to a source sample index, mono `(L + R) >> 1` mix, stereo L/R copy, and `sampleOffset = 0` tail. Rust also has defensive zero-sample and output-slice bounds guards; these are compatibility extensions around the C-shaped path rather than behavior used by the C caller. |
| exact 534-sample fast path | `crates/snes/src/apu.rs` | verified | Rust's 534-sample branch is equivalent to C's generic loop because the adder is exactly 1.0 and each integer-truncated location maps to the same source frame. Mono averaging and stereo copy match C before resetting `sample_offset`. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 SPC Core Helper Pass

Scope: manually compare SPC reset/run shell, flag/stack helpers, addressing
helpers, and shared opcode helper bodies in `crates/snes/src/apu.rs` against
`../zelda3/snes/spc.c`, without using the
progress/signature scripts. Individual opcode dispatch groups remain separate
audit slices.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| SPC reset/run shell and flags | `crates/snes/src/apu.rs` | verified | `spc_reset`, `spc_run_opcode`, opcode fetch, packed flag get/set, Z/N helpers, and word Z/N helper match C's reset state, reset-vector fetch through APU reads, stopped return, cycle-table seeding, PC increment, and flag bit layout. |
| SPC stack and word helpers | `crates/snes/src/apu.rs` | verified | `spc_pull_byte`, `spc_push_byte`, `spc_pull_word`, `spc_push_word`, `spc_read_word`, and `spc_write_word` match C's stack pre/post increment order, `$0100 | SP` address formation, high-before-low word push, low-then-high word pull, and low/high memory access order. |
| SPC addressing helpers | `crates/snes/src/apu.rs` | verified | Direct-page, absolute, immediate, indirect, indexed-indirect, direct-page indexed, absolute indexed, indirect-Y, direct-page word, direct-page/direct-page, direct-page/immediate, indirect/indirect, post-increment indirect, and absolute-bit helpers match C's operand fetch order, page-bit handling, 8-bit wraparound, 16-bit wraparound, source/destination tuple order, and bit extraction. |
| SPC shared operation helpers | `crates/snes/src/apu.rs` | verified | MOV loads/stores, OR/AND/EOR and memory forms, CMP/CPX/CPY/CMPM, ADC/ADCM, SBC/SBCM, ASL/LSR/ROL/ROR, INC, and DEC match C's read/write order, result truncation, carry/half-carry/overflow flag calculations, and Z/N updates. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 SPC Dispatch Low/Logic/Compare Pass

Scope: manually compare the first SPC opcode dispatch families in
`crates/snes/src/apu.rs` against `../zelda3/snes/spc.c`,
without using the progress/signature scripts. This pass covers dispatch and
inline opcode bodies that are not already fully represented by the shared
helper pass.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| low-nibble bit/control dispatch | `crates/snes/src/apu.rs` | verified | NOP, TCALL, SET1/CLR1, BBS/BBC, PUSH P/A/X/Y, TSET1/TCLR1, BRK, page flag clear/set, PCALL, RET/RETI, branches through `0x70`, JMP/CALL forms, CBNE, DBNZ direct-page, DECW/INCW, ASL/ROL/LSR/ROR accumulator and memory forms, and X inc/dec/transfer cases match C's operand order, stack order, bit selection, branch timing helper use, direct-page wrapping, and word Z/N updates. |
| OR/AND/EOR dispatch families | `crates/snes/src/apu.rs` | verified | OR, AND, and EOR accumulator and memory forms through direct-page, absolute, indirect, indexed-indirect, immediate, direct-page indexed, absolute indexed, indirect-Y, direct-page/direct-page, direct-page/immediate, and indirect/indirect modes dispatch to the same helper/addressing combinations as C. OR1/AND1/EOR1 and inverted forms match C's absolute-bit fetch and boolean carry mutation semantics. |
| CMP and word arithmetic dispatch through ADDW | `crates/snes/src/apu.rs` | verified | CMP A/X/Y, CMPM, CMPW, ADC dispatch through the first ADC family, and ADDW `0x7a` match C's addressing helpers, flag updates, and result register writes. This includes C's `ADDW` half-carry expression with `+ 1`, which Rust preserves. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 SPC Dispatch ADC/SBC/MOV Tail Pass

Scope: manually compare the remaining SPC opcode dispatch families in
`crates/snes/src/apu.rs` against `../zelda3/snes/spc.c`,
without using the progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| ADC/SBC and word arithmetic tail | `crates/snes/src/apu.rs` | verified | ADC/ADCM cases `0x84..0x99`, SUBW `0x9a`, SBC/SBCM cases `0xa4..0xb9`, INC/DEC A and memory tail cases, DIV `0x9e`, MUL `0xcf`, and DAA/DAS match C's addressing helpers, result truncation, carry/half-carry/overflow/Z/N flags, divide-by-zero fallback, Y/A result split, and wrapping decimal adjust behavior. |
| MOV and bit-store tail | `crates/snes/src/apu.rs` | verified | MOV1/MOV1S/NOT1, MOV load/store A/X/Y forms, MOVW load/store, MOVM direct-page copies, post-increment indirect MOV, and transfer opcodes match C's source/destination order, read-before-write behavior, direct-page and indexed addressing, post-increment X behavior, and Z/N updates where C applies them. |
| branch/control tail | `crates/snes/src/apu.rs` | verified | Carry/Z branches, POP P/A/X/Y, EI/DI, CLRV, NOTC, SLEEP, STOP, DBNZ direct-page, DBNZ Y, and stack-pointer transfers match C's flag mutations, pop side effects, stopped-state behavior, and decrement-before-branch ordering. |

Checks after this pass:

- `cargo check -q -p snes`
- `git diff --check -- docs/porting/manual_parity_audit.md docs/porting/manual_parity_status.md`

## 2026-05-31 NMI Full-File Pass

Scope: manually compare `crates/zelda3/src/nmi.rs` against
`../zelda3/src/nmi.c` and `nmi.h`, without using the
progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| NMI entry, audio, joypad, and PPU registers | `crates/zelda3/src/nmi.rs` | verified | `Interrupt_NMI`, `Interrupt_NMI_AudioParts_Locked`, `NMI_ReadJoypads`, and `WritePpuRegisters` match C call order, music/SFX APUI writes, joypad bit reversal/filtering, thread-stack toggle, window/color math, scroll, Mode 7 center, and BG tile-base register writes. |
| core NMI update dispatcher | `crates/zelda3/src/nmi.rs` | fixed | Rust matches C's link/tile DMA, HUD/CGRAM/OAM copies, BG stripe selection, tilemap DMA, copy-packet handling, subroutine-index clear, and 25-entry subroutine dispatch. This pass restored C-shaped direct behavior for animated-tile source `0`, invalid `nmi_load_bg_from_vram` asserts, invalid subroutine-index panics, invalid copy-packet `vmain` panics, and odd vertical packet length asserts. |
| tilemap, stripe, and graphics upload helpers | `crates/zelda3/src/nmi.rs` | verified | `NMI_UploadTilemap`, BG3 text, OW scroll, subscreen overlay former/latter, arbitrary tilemap upload, BG1 wall vertical upload, light/dark world map uploads, BG char/object char uploads, game-over/peg/star/IRQ graphics, `HandleStripes14`, and CopyToVram helpers match the C VRAM destinations, byte-vs-word copies, low-byte-only map copies, vertical increment rules, terminators, and NMI-disable clearing behavior. |

Checks after this pass:

- `cargo test -q -p zelda3 nmi::tests`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Sprite Main Guard Body OAM Pass

Scope: `Guard_HandleAllAnimation`, `Guard_AnimateHead`, `Guard_AnimateBody`,
and `Guard_AnimateWeapon` in `src/sprite_main.c:4812..4864`, checked directly
against `crates/zelda3/src/sprite_main.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| guard animation dispatch/head/weapon | `crates/zelda3/src/sprite_main.rs` | verified | Rust matches C's prep/early-return dispatch, body/head/weapon call ordering, optional shadow draw, head OAM offset, head Y subtraction, weapon OAM index/table selection, `dungmap_var8` byte packing, tile adjustment for sprite types below `0x43`, and two-entry weapon emission. |
| guard body OAM emission | `crates/zelda3/src/sprite_main.rs` | fixed | C only increments `OamEnt *oam` after an emitted body tile and forces `oam->y = 0xf0` for tile `0x20` on soldier type `0x46`. Rust now packs emitted entries through skipped body pieces and applies the same hidden-Y override. |

Checks after this pass:

- `cargo test -q -p zelda3 sprite_main::tests::guard_body_packs_skipped_oam_and_hides_type_46_blank_tiles`
- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Overlay/Fixed Scroll Pass

Scope: `OverworldLoad_LoadSubOverlayMap32`, `LoadOverworldOverlay`,
`Overworld_ReadTileAttribute`, and `Overworld_SetFixedColAndScroll` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| sub-overlay load and map8 upload | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's overlay quadrant load into `0x4000`, `Map16ToMap8(&g_ram[0x4000], 0x1000)`, `nmi_subroutine_index = nmi_disable_core_updates = 4`, and `submodule_index++`. |
| overworld tile-attribute read | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's wrapped X/Y offset calculation, Y mask shifted by 3, `dung_bg2[t >> 1]` word read, and direct `kSomeTileAttr` lookup. |
| fixed color and scroll setup | `crates/zelda3/src/overworld.rs` | fixed | Rust already matched the palette word selection, COLDATA defaults, special fixed-color cases, early CGRAM-update return, and final `TS_copy`/CGRAM flag update. This pass restored the missing `(si & 0x3f) == 0x1b` BG1 parallax branch, including the signed half-offset from `BG2HOFS_copy2 - 0x778`, vertical clamp/wrap rules, and the submodule-4 transition special case. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Memorized Tile/Damage Data Pass

Scope: `MirrorBonk_RecoverChangedTiles`,
`DecompressEnemyDamageSubclasses`, `Overworld_Memorize_Map16_Change`, and
`HandlePegPuzzles` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| mirror-bonk tile recovery | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `num_memorized_tiles >> 1` loop, memorized address/value word reads, and restore into `dung_bg2[pos >> 1]`. |
| enemy damage subclass decompression | `crates/zelda3/src/overworld.rs` | fixed | C copies `kEnemyDamageData` directly into `g_ram[0x14000]` and decodes a fixed 0x1000-byte nibble table. Rust now requires the asset and copies it directly instead of returning on missing data or truncating the copy. |
| map16-change memorization | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's skip for values `0xdc5` and `0xdc9`, stores value/address at `num_memorized_tiles >> 1`, and advances `num_memorized_tiles` by 2. |
| peg puzzle handler | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's light-world Turtle Rock three-position sequence, event bit gate, success/failure sound writes, `word_7E04C8` increments/reset, submodule 47 handoff, and dark-world screen `98` counter/event/door update behavior. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Special Switch/Ganon Entrance Pass

Scope: `GanonTowerEntrance_Func1`, `Overworld_CheckSpecialSwitchArea`, and
`Overworld_GetMap16OfLink_Mult8` in `src/overworld.c`, checked directly
against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| Ganon Tower entrance palette step | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule-zero sound effect and `Palette_AnimGetMasterSword2` call, then the blinding-white branch that sets `palette_filter_countdown = 0xff` and increments `subsubmodule_index` once the filter reaches `0xff`, otherwise calling `Palette_AnimGetMasterSword3`. |
| special switch area detection | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's reverse scan of the four map8/screen entries, direct dungeon-room exit write, chained direction writes to `overworld_screen_trans_dir_bits`, `overworld_screen_trans_dir_bits2`, and `link_direction`, `DirToEnum` writes to `byte_7E069C` and `overworld_screen_transition`, and module/submodule handoff to `main_module_index = 11`, `submodule_index = 23`. |
| link map16-to-map8 lookup | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `(link_x + 8) >> 3`, `link_y + 12`, wrapped offset/mask calculation, `dung_bg2[pos >> 1] * 4`, and direct `kMap16ToMap8` four-word return. |

Checks after this pass:

- Direct C/Rust source comparison only; no Rust code changed in this tranche.

## 2026-05-31 Overworld Entrance Use Pass

Scope: `Overworld_UseEntrance` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| overworld entrance interaction | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's Link coordinate-to-`dung_bg2` position calculation, forward-facing door tile checks, `0xe9` map16 replacement, `0x149/0x169` door animation setup, progress-gated `0x169` fallthrough, `LookupInOwEntranceTab`/`LookupInOwEntranceTab2` checks, tagalong text gate, and module handoff to entrance load. This pass replaced silent `asset_u16`/`asset_u8` fallback reads with required direct reads of `kMap16ToMap8` and `kOverworld_Entrance_Id`, matching C's direct table indexing. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Map Load Stripe Pass

Scope: `TriggerAndFinishMapLoadStripe_Y`, `TriggerAndFinishMapLoadStripe_X`,
`SomeTileMapChange`, `CreateInitialNewScreenMapToScroll`, the eight
`CreateInitialOWScreenView_*` helpers, `OverworldTransitionScrollAndLoadMap`,
the four `BuildFullStripeDuringTransition_*` helpers, `Map16ToMap8`, and
`OverworldCopyMap16ToBuffer` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| map-load stripe finishers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's forced transition-direction byte writes (`8` for Y, `2` for X), `nmi_subroutine_index = 3`, initial `uvram` header words `0x80`/`0x8040`, repeated stripe build calls, source/destination/var wrap updates, and `0xffff` terminator. |
| initial new-screen view builders | `crates/zelda3/src/overworld.rs` | fixed | Big and small north/south/west/east helpers match the C source offset rewrites, `map16_load_var2`/destination setup, small-map backup words, and post-load wrap adjustments. Invalid direction arms now also reset `submodule_index = 0` before panicking, matching the C assert path side effect. |
| transition scroll stripe builder | `crates/zelda3/src/overworld.rs` | fixed | The four directional `BuildFullStripeDuringTransition_*` helpers match C's one-stripe header, buffer call, and map16 load cursor update. `OverworldTransitionScrollAndLoadMap` now mirrors the C invalid-direction assert side effect by clearing `submodule_index` before aborting, then writes the double `0xffff` terminator and only enables NMI upload when the stripe pointer advanced. |
| map16-to-map8 full buffer copy | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `map16_load_src_off += 0x1000`, 32-row loop, `r14 += 0x100`, word-pointer `r10 += 2` represented as byte `+4`, source decrement, and `map16_load_var2` wrap. `OverworldCopyMap16ToBuffer` preserves the temporary 32-word ring buffer, `yr/xr` masking, map8 table lookup, `r0` high-half offset, and BG2 attr-table writes at `r14`, `r14 + 64`, `r14 + 2`, and `r14 + 66`. |
| `SomeTileMapChange` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's full quadrant decompression, first 64 BG1 words set to `0x0dc4`, overlay/bomb-door handling, and `submodule_index++`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Map Scroll Stripe Pass

Scope: `OverworldHandleMapScroll`,
`CheckForNewlyLoadedMapAreas_North`,
`CheckForNewlyLoadedMapAreas_South`, `CheckForNewlyLoadedMapAreas_West`,
`CheckForNewlyLoadedMapAreas_East`, `BufferAndBuildMap16Stripes_X`, and
`BufferAndBuildMap16Stripes_Y` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| map-scroll dispatcher | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's direction cases `1/2/4/5/6/8/9/10`, clears or masks `overworld_screen_trans_dir_bits2` the same way, writes the double `0xffff` terminator, conditionally sets `nmi_subroutine_index = 3`, and copies the resulting direction byte into `overworld_screen_transition`. The invalid-direction path now resets `submodule_index = 0` and panics instead of silently continuing. |
| newly-loaded area checks | `crates/zelda3/src/overworld.rs` | verified | North uses the same signed `map16_load_src_off - 0x80` boundary; south uses `>= 0x1800`; west/east reduce the source offset modulo `0x80` before checking `0` or `>= 0x60`. All four helpers skip stripe emission on small maps but still advance the same source/destination/var cursor words when within bounds. |
| X/Y stripe buffer builders | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's draw-strip table index, 32-entry temporary tile-state ring buffer, out-of-range `pos >= 0x2000` zero fill, map8 table lookup, `r0` high-half selection, and two-half stripe layout. The X helper preserves C's `dst[33] = s[1]` / `dst[1] = s[2]` ordering; the Y helper preserves `dst[32] = s[2]` / `dst[1] = s[1]`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Quadrant Decompression Pass

Scope: `GetOverworldHibytes`, `GetOverworldLobytes`,
`Decompress_bank02`, `Overworld_DecompressAndDrawAllQuadrants`,
`Overworld_DecompressAndDrawOneQuadrant`,
`Overworld_ParseMap32Definition`, `OverworldLoad_LoadSubOverlayMap32`, and
`LoadOverworldOverlay` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| overworld compressed block accessors | `crates/zelda3/src/overworld.rs` | fixed | C returns direct `kOverworld_Hibytes_Comp(i).ptr` and `kOverworld_Lobytes_Comp(i).ptr`. Rust now requires those memblocks instead of returning an empty vector for missing blocks. |
| bank02 decompressor | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's command decoding, extended-length calculation, literal/copy/fill/alternating/incrementing modes, and destination-relative backcopy. Source reads now index directly instead of returning early or substituting zero when compressed data is short. |
| quadrant draw and Map32 parse | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's four quadrant destinations, hi-byte then lo-byte decompression into the interleaved `0x14000` source buffer, `map16_decode_last = 0xffff`, 16x16 map32 loop, `dst += 2` word-pointer stride represented as byte `+4`, and row stride represented as byte `+192`. Map32-to-Map16 tables are now required/directly indexed, preserving the C decode cache layout and final writes to `dst[0]`, `dst[64]`, `dst[1]`, and `dst[65]`. |
| sub-overlay load | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's single-quadrant overlay load into `0x4000`, `Map16ToMap8(&g_ram[0x4000], 0x1000)`, `nmi_subroutine_index = nmi_disable_core_updates = 4`, and `submodule_index++`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Table Accessor/Sprite Property Pass

Scope: `GetMap8toTileAttr`, `GetMap16toMap8Table`,
`LookupInOwEntranceTab`, `LookupInOwEntranceTab2`, `CanEnterWithTagalong`,
`DirToEnum`, `Overworld_GetSignText`, `GetOverworldSpritePtr`,
`GetOverworldBgPalette`, `Sprite_LoadGraphicsProperties`, and
`Sprite_LoadGraphicsProperties_light_world_only` in `src/overworld.c`, checked
directly against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| map8/map16 table accessors | `crates/zelda3/src/overworld.rs` | fixed | C returns direct pointers to `kMap8DataToTileAttr` and `kMap16ToMap8`. Rust previously returned an empty vector if the asset was missing; it now requires the corresponding asset and returns the full table bytes. |
| overworld entrance lookup helpers | `crates/zelda3/src/overworld.rs` | fixed | `LookupInOwEntranceTab` already matched the reverse scan of `kOverworld_Entrance_Tab0/1`. `LookupInOwEntranceTab2` now requires the position/area assets and directly reads the 129 entries from high to low, matching C instead of allowing zero fallback reads. |
| tagalong and direction helpers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's follower allow-list, `(t == 7 || t == 8) && e >= 59` precedence, and `DirToEnum` shift/count loop. |
| sign text, sprite pointer, and BG palette helpers | `crates/zelda3/src/overworld.rs` | fixed | Rust now requires `kOverworld_SignText`, `kOverworldSpriteOffs`, `kOverworldSprites`, and `kOverworldBgPalettes` instead of returning zero/empty fallback data. The sprite pointer base selection for progress values `3`, `2`, and default already matched C. |
| sprite graphics property loads | `crates/zelda3/src/overworld.rs` | fixed | Rust now mirrors the two C `memcpy` calls for dark-world sprite graphics/palettes at `+64`, then delegates to the light-world loader. The light-world loader requires both source assets, computes the same progress-dependent index, and performs exact 64-byte copies. The shared copy helper now panics on short source/destination ranges like C direct `memcpy` would fail under bad data. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Mirror HDMA/Pit Damage Pass

Scope: `InitializeMirrorHDMA`, `MirrorWarp_BuildWavingHDMATable`,
`MirrorWarp_BuildDewavingHDMATable`, and `TakeDamageFromPit` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| mirror HDMA initialization | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's HDMA disable, mirror var clears/initial constants, signed `var1`/`var3` setup, `HdmaSetup(0xf2fb, 0xf2fb, 0x42, BG1HOFS, BG2HOFS, 0)`, 240-word HDMA table fill from `BG2HOFS_copy2`, and final `HDMAEN_copy = 0xc0`. |
| mirror waving table build | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's animation submodule call, odd-frame return, 8-line table shift, `mirror_vars.var0 >> 1` selection, signed target crossing clamp, var5/var7 reset, var0 toggle, fractional accumulator update, `swap16` addition into var5, late palette-filter threshold that narrows var1 to `+-0x100`, subsubmodule advance, and first four HDMA entries set to `t + BG2HOFS_copy2`. |
| mirror dewaving table build | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's animation submodule call, odd-frame return, 8-line table shift, OR of HDMA entries `0xc0/0xc8/0xd0/0xd8`, HDMA disable on BG2 scroll match, subsubmodule advance, fixed-color/scroll reset call, and non-`0x1b` screen BG1/BG2 scroll-copy restore. |
| `TakeDamageFromPit` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's visibility status, indoor/outdoor submodule selection, wrapping health subtract by 8, and overflow/death clamp when the result is `>= 0xa8`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Preload/Exit Data Pass

Scope: `Module08_OverworldLoad`, `PreOverworld_LoadProperties`,
`AdjustLinkBunnyStatus`, `ForceNonbunnyStatus`, `RecoverPositionAfterDrowning`,
`LoadOverworldFromDungeon`, `Overworld_LoadNewScreenProperties`, and
`LoadCachedEntranceProperties` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| pre-overworld module dispatch | `crates/zelda3/src/overworld.rs` | fixed | C indexes the three-entry `kModule_PreOverworld` table directly. Rust now panics for invalid `submodule_index` instead of ignoring it, while valid cases dispatch to the same load-properties, overlay, and load/advance handlers. |
| `PreOverworld_LoadProperties` setup | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's CGWSEL/dungeon flag setup, bunny adjustment, dungeon-vs-special overworld load path, song-list setup, key/HUD reset, overworld animated tile/music selection including the `ow_anim_tiles = 0x5a` comma-expression branch behavior, palette/tile initialization, fixed color reset, follower/sprite/mirror-portal setup, ambient SFX choice, doorway/button/movement clears, dark-world bunny transform branch, BGMODE/lower-level clears, submodule/HUD increments, savegame-state clear, and final music load. |
| bunny status helpers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's moon-pearl gate, forced ground handler state, temp-bunny/poof/bunny flag clears, mirror-bunny clear, and enhanced turn-while-dashing `link_is_running` clear. |
| drowning recovery | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's cached coordinate, scroll-bound, target, quadrant, direction, lower-level, doorway, floor, visibility/blink, damage reset, follower, water puzzle, map state, transition, and submodule restores, including the indoor `+2` and outdoor `-2` camera high-word behavior and the death handoff to module 18. |
| dungeon exit-data load | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's initial overworld/dungeon flag clears, cached-entrance gate, reverse `kExitDataRooms` scan from entry 78 down, scroll/player/map16/camera/direction/door/screen/unk writes, signed `unk1/unk3` byte extension, negated scroll unk fields, and final screen-property load. Exit-data assets are now required and indexed directly instead of using zero fallback reads. |
| new-screen and cached-entrance properties | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's tilemap mask, GFX/screen-size load, bottom-bound byte write, high-byte clear for `overworld_area_is_big`, camera-boundary reload from current screen, quadrant/OAM/direction/z resets, and cached entrance restoration of TM, BG scroll copies, Link coords, direction/fancy-door adjustment, map16 offsets, camera lows/highs, scroll bounds/targets, unk fields, byte `7E0AA0`, tile themes, and sprite graphics index. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Spotlight/Pyramid Warp Pass

Scope: `Module0F_SpotlightClose`, `Dungeon_PrepExitWithSpotlight`,
`Spotlight_ConfigureTableAndControl`, `OpenSpotlight_Next2`,
`Module10_SpotlightOpen`, `Module10_00_OpenIris`,
`SetTargetOverworldWarpToPyramid`, and `ResetAncillaAndCutscene` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| spotlight close module | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's sprite call, submodule 0 prep-vs-config dispatch, outdoor waterfall/velocity handling, special outdoor direction choice for entrance `0x43`, direction/last-direction table writes from `{8,4,2,1}`, moving animation call, and Link OAM call. |
| dungeon spotlight exit prep | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's NMI/polyhedral clears, outdoor waterfall termination and exit-Y cache, `ZeldaGetEntranceMusicTrack` handling including the short-circuited `m != 3 || (m = sram_progress_indicator) >= 2` behavior, fade/music-control values, floor HUD update, HUD NMI increment, spotlight close, and submodule advance. |
| spotlight table/open handoff | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's iris-table configure, NMI/polyhedral clears, early return when submodule is nonzero, module-6 exit-Y restore, force blank/item reset outside module 9, module-9 dungeon-room direction submodule choice, transition countdown, low-byte entrance/door plus high-byte fancy-door test, door counter high-bit behavior, big-rock high-bit clear, door animation/submodule/subsubmodule/SFX writes, window mask clears, sword-hold clear, and special screen fixed-color triplets. |
| spotlight open and pyramid warp reset | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's sprite/open-vs-config/OAM sequence, `Spotlight_open` plus submodule advance, pyramid warp module-21 gate, overworld reload, animated tile decompress `0x5a`, ancilla termination, damage/button/sword/immobilization clears. |

Checks after this pass:

- Direct C/Rust source comparison only; no Rust code changed in this tranche.

## 2026-05-31 Overworld Module Dispatcher/Rain Pass

Scope: `Module09_Overworld` and `OverworldOverlay_HandleRain` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| overworld submodule dispatch | `crates/zelda3/src/overworld.rs` | fixed | Rust dispatches all 48 valid `kOverworldSubmodules` entries to the same handlers as C. Invalid submodule values now panic instead of being silently ignored, matching C's direct table-index behavior. |
| scroll offset wrapping around sprite draw | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's BG2/BG1 scroll copy snapshots, temporary addition of BG1 X/Y offsets into both BG scroll copies, `Sprite_Main()`, restore of copy2 scroll words, `LinkOam_Main()`, HUD refill, and rain overlay call. |
| rain overlay | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's screen/progress/event early-return precedence, frame-counter fixed-color/SFX cases `3/88`, `5/44/90`, and `36`, 4-frame movement cadence, overlay counter increment-and-mask, and BG1 X/Y offset additions from the `{1,0,1,0}` and `{0,17,0,17}` tables shifted by 8. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Player Control/Transition Wrapper Pass

Scope: `Module09_00_PlayerControl`, `OverworldHandleTransitions`,
`ScrollAndCheckForSOWExit`, `Module09_LoadAuxGFX`,
`Module09_LoadNewMapAndGFX`, `Overworld_RunScrollTransition`,
`Module09_LoadNewSprites`, `Overworld_StartScrollTransition`,
`Overworld_EaseOffScrollTransition`, `Module09_0A_WalkFromExiting_FacingDown`,
`Module09_0B_WalkFromExiting_FacingUp`,
`Module09_09_OpenBigDoorFromExiting`, `Overworld_DoMapUpdate32x32_B`,
`Module09_0C_OpenBigDoor`, `Overworld_DoMapUpdate32x32_conditional`, and
`Overworld_DoMapUpdate32x32` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| player-control module | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's menu/blocking gates, map/select menu handoffs, item-switch handling, special-entrance animation call, Link main, super-bomb HUD indicator gate, current-area calculation from Link coordinates, CHR half-slot load, camera scroll, entrance/palette/transition path for normal overworld, and SOW exit path for module 11. |
| edge transition detection | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's pending map-scroll handling, Y/X velocity edge tests, expected direction compare, edge-screen transition veto, special-switch fallback, map16 source mask/add using `kSwitchAreaTab0/1/3`, old-screen ambient special case, area-head plus dark-world bit, ambient/music-control checks, GFX/screen-size reload, transition direction bytes, `DirToEnum`, low-byte entrance/door clears, transition counter clear, mosaic handoff for area heads, and palette/cache refresh otherwise. |
| SOW exit and transition wrappers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's SOW map-scroll gate and special-switch-B scan, aux-GFX event/dungeon flag clears, NMI/submodule updates, new-map tile/decompress/GFX sequence, scroll-transition animation/VRAM/map-load wrapper, new-sprite vertical correction on transition `1`, sprite reload, memorized-tile clear, fixed-color reset gate, start-scroll diagonal map-load gate, ease-off timing including small-map restores and follower disable. |
| exit walking and big-door update helpers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's facing-down animation, per-frame Y movement, transition countdown, final Y nudge/velocity/camera/map-scroll behavior, facing-up countdown behavior, open-big-door gate on door animation step `3`, countdown/submodule/trans-dir writes, conditional door counter increment-vs-map update, four persisted door-tile writes, VRAM terminator at current upload offset, memorized-tile byte-count increment, door animation step increment including the counter-32 skip, NMI BG reload, and low-byte door counter increment. |

Checks after this pass:

- Direct C/Rust source comparison only; no Rust code changed in this tranche.

## 2026-05-31 Overworld Mosaic/Overlay/Mirror Warp Pass

Scope: `Overworld_StartMosaicTransition`, `Overworld_LoadOverlays`,
`PreOverworld_LoadOverlays`, `Overworld_LoadOverlays2`,
`Module09_FadeBackInFromMosaic`, `Overworld_Func1C`,
`OverworldMosaicTransition_LoadSpriteGraphicsAndSetMosaic`,
`Overworld_Func22`, `Overworld_Func18`, `Overworld_Func19`,
`Module09_MirrorWarp`, `MirrorWarp_FinalizeAndLoadDestination`,
`Overworld_DrawScreenAtCurrentMirrorPosition`, and
`MirrorWarp_LoadSpritesAndColors` in `src/overworld.c`, checked directly
against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| mosaic transition start/fade | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's conditional mosaic control, subsubmodule cases, music fade-to-zero gate excluding screen `0x80`, transition reset, palette bounce, forced blank, animated sprite tile decode on area heads, overlay window setup for nonzero overworld areas outside module 11, special-overworld reload for submodule 36, fade-back palette/sprite/GFX upload sequence, ambient/music restore excluding screens `0x80/0x2a`, and SOW handoff to module 9 submodule 31. |
| overlay loaders | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's sprite slot/reload and Link throw-state clears, ambient SFX setup, full previous-state snapshot, overlay/BG1 subpixel clears, high-screen room-specific overlay selection including the master-sword getout case, forest overlay choice, dark-world death mountain overlay choice, rain/progress overlay choice, load-overlay map16 setup, transition-dir clears, window/subscreen/color math setup, ambient SFX from overlay music, CGADSUB/TS cases, load call, overlay `0x94` BG1 vertical high-bit adjustment, and restoration of the previous overworld/map16/transition state. |
| mosaic helper and special-area wrappers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's sprite GFX load, INIDISP/HDMA setup, palette countdown from target minus one, target clear, lightening mode, subsubmodule advance, fade-in INIDISP increment until 15, special-area swim reset, module/submodule preservation, and `Module08_02_LoadAndAdvance` wrapper behavior. |
| mirror warp module/finalize | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's NMI disable increment, high-screen abort, music/control/blink/HDMA setup, dark-world bit toggle, `word_7E04C8` clear, screen/area recompute, map-state clear, white filter, GFX/screen-size reload, case-1 fallthrough into waving-table build, dewaving build, HDMA finalize setup, iris reset, palette/lightening clears, sheet reload, song-list reload, HDMA enable, music/ambient selection with no-pearl bunny music override, menu/submodule/map-state/core-update resets. |
| mirror draw/load helpers | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's backup/restore of map16 source/destination/var2 around small-map temporary `0x390` positioning, quadrant/overlay draw, mirror-bonk tile restore for submodule 44, Map16-to-map8 refresh, palette loads, special overworld palette, fixed col/scroll reset, special `TS_copy` for screens `0x1b/0x5b`, white palette fill, screen `0x5b` palette zeroes, sprite reset/reload, Link item reset, torch/player reset, mirror player handler state, and mirror portal initialization outside dark-world screens. |

Checks after this pass:

- Direct C/Rust source comparison only; no Rust code changed in this tranche.

## 2026-05-31 Whirlpool/Drowning/Bird Travel Pass

Scope: `Overworld_WeathervaneExplosion`, `Module09_2E_Whirlpool`,
`Overworld_Func2F`, `Module09_2A_RecoverFromDrowning`,
`Module09_2A_00_ScrollToLand`, `FluteMenu_LoadTransport`,
`Overworld_LoadBirdTravelPos`, `FluteMenu_LoadSelectedScreenPalettes`, and
`FindPartnerWhirlpoolExit` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| whirlpool transition module | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's NMI-disable increment, subsubmodule cases `0..=12`, SFX/ambient/map-state/palette setup, whirlpool palette filters, color/palette/HUD setup, partner exit lookup, draw-width clear, overlay reload and submodule decrement, NMI/CGRAM/INIDISP/core-update writes, aux-GFX and finish-GFX wrappers with submodule correction, palette reload case, restore-red/green double-call behavior when countdown remains nonzero, incremental VRAM plus blue restore, and final reload/music/map-state/core-update reset. |
| pyramid hole and drowning recovery | `crates/zelda3/src/overworld.rs` | verified | `Overworld_WeathervaneExplosion` is intentionally empty. Rust matches C's `Overworld_Func2F` map16 write/memorize/draw/NMI/submodule behavior and drowning recovery module dispatch. The scroll-to-land helper matches C's two-pixel-at-most X/Y approach to cached coordinates, signed velocity bytes from wrapping deltas, subsubmodule/incapacitation/damage clears on arrival, camera scroll, and pending map-scroll handling. |
| bird/flute travel position load | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's memorized-tile clear, low-byte travel index, word shift of `birdtravel_var1`, scroll/player/unk/screen/map16/camera/door field setup, new-screen property reload, sprite reset/reload, doorway clear, and torch/player reset. Bird-travel assets are now required and indexed directly instead of using zero fallback reads. |
| selected travel palettes and partner whirlpool exit | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's selected-screen palette reload sequence and `FindInWordArray(kWhirlpoolAreas, overworld_screen_index, size)` behavior followed by memorized-tile clear and travel slot `j + 9`. The whirlpool-area asset is now required and read directly instead of treating a missing table as empty. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Pit/Tool/Lift Interaction Pass

Scope: `Overworld_GetPitDestination`, `Overworld_ToolAndTileInteraction`,
`Overworld_PickHammerSfx`, `Overworld_GetLinkMap16Coords`,
`Overworld_HandleLiftableTiles`, `Overworld_LiftingSmallObj`,
`Overworld_SmashRockPile`, and `SmashRockPile_fromLift` in `src/overworld.c`,
checked directly against `crates/zelda3/src/overworld.rs` and the shared
Rust helpers in `crates/zelda3/src/player.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| pit destination lookup | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's aligned Link coordinate math, map16 position calculation, reverse scan of the 19 fall-hole entries, Chris Houlihan fallback, entrance write, and `byte_7E010F` clear. Fall-hole assets are now required and indexed directly instead of using zero fallback reads. |
| tool/tile interaction | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's scratch/index clears, map16 attribute lookup, shovelable and thick-grass cases, bush/small-rock cases, secret reveal and map16 update path, smashed-terrain spawn and poof, hammer peg case, hammer SFX fallback, and final quadrant map8-to-attribute return. The map16-to-map8 and map8-attribute tables are now required and directly indexed. |
| hammer SFX and liftable tile dispatch | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's first-quadrant map8 attribute thresholds for hammer sounds, big-rock quadrant tile groups, small-object tile groups, and fallback quadrant attribute lookup. The fallback map8/attribute lookup now uses required direct table reads. |
| link map16 coordinate and smash helpers | `crates/zelda3/src/player.rs` | fixed | Rust matches C's direction-indexed action offsets, aligned x/y output, map16 position calculation, optional one-tile-down smash probe, rock-pile/small-object dispatch, small-object secret/map16 update behavior, rock-pile base-position adjustment, door counter setup, `0xffff` secret event behavior, quadrant coordinate adjustment, 32x32 door update call, and final quadrant attribute lookup. Rust no longer clamps the direction index and no longer uses fallback table reads for the final quadrant attribute. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Secret/Event Opening Helpers Pass

Scope: `Overworld_BombTiles32x32`, `Overworld_BombTile`,
`Overworld_AlterWeathervane`, `OpenGargoylesDomain`, `CreatePyramidHole`,
`Overworld_RevealSecret`, `AdjustSecretForPowder`, and
`Overworld_HandleOverlaysAndBombDoors` in `src/overworld.c`, checked directly
against `crates/zelda3/src/overworld.rs` and shared smash helpers in
`crates/zelda3/src/player.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| bomb tile scan and reveal behavior | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's 3x3 bomb scan origin, 16-pixel scan increments, final `word_7E0486/0488` writes, follower-13 bypass, bush/grass tile replacement choices, reveal-secret fallback tile behavior, smashed-terrain spawn coordinates, NMI flag, and secondary `0xdb4/0xdb5` overlay behavior including the original `pos` memorize quirk for `0xdb5`. |
| weather vane, gargoyle, and pyramid openings | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's door counter/start position for the weathervane, 32x32 update call, all persisted map16 tile positions/values for the weathervane, Gargoyle's Domain, and Pyramid hole, event bits, ambient/SFX writes, and NMI BG reload flags. |
| secret reveal/powder | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's high-screen fail path, secret-offset/table scan, `0xffff` fail sentinel, data-bit writes to `dung_secrets_unk1`, powder override, `0xff` secret marker, discovery SFX conditions including Ice Temple bug-fix gate, and `kTileBelow[(data & 0xf) >> 1]` return mapping. |
| overlay/bomb-door wrapper | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's screen `0x33` and `0x2f` hardcoded `0x20f` writes, event-overlay load gate for low screens with bit `0x20`, and secondary overlay bit `2` direct lookup through `kSecondaryOverlayPerOw`. Rust previously skipped the secondary overlay when the screen exceeded the local table length; it now indexes directly like C. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Camera Scroll Pass

Scope: `Overworld_OperateCameraScroll` and `OverworldCameraBoundaryCheck` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`
and the C/Rust RAM layout constants. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_OperateCameraScroll` Y-axis camera movement | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's optional Z subtraction, `link_y_coord - z + 12` threshold position, signed per-pixel velocity loop, low/high camera-bound checks, `WORD(byte_7E069E[0])` write, overlay `0xb5/0xbe` quarter-scroll handling, default half-scroll handling, overlay `0x97/0x9d` skip, screen `0x1b` BG1 vertical clamp, and BG1 subpixel/copy carry behavior. |
| `Overworld_OperateCameraScroll` X-axis camera movement | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `link_x_coord + 8` threshold position, signed per-pixel velocity loop, low/high camera-bound checks, unaligned `WORD(byte_7E069E[1])` write at `$69f`, overlay `0x95/0x9e` quarter-scroll handling, default half-scroll handling, overlay `0x97/0x9d` skip, and BG1 horizontal subpixel/copy carry behavior. |
| `Overworld_OperateCameraScroll` overlay drift and room override | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `overworld_screen_index != 0x47` gate, overlay `0x9c` vertical drift with `WORD(byte_7E069E[0])` added to BG1 copy and BG1 horizontal copy set from BG2, overlay `0x97/0x9d` diagonal BG1 drift, and dungeon room `0x181` BG1/BG2 copy override. |
| `OverworldCameraBoundaryCheck` scroll-boundary helper | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `ya >>= 1` / `r8 >>= 1` pointer math, BG2 vertical-vs-horizontal scroll target selection, `ow_scroll_vars0` boundary comparison at `$600`, paired `overworld_unk1` zeroing at a hard boundary, camera high/low word updates, 16-pixel accumulator wrap, transition-direction bit OR from `kOverworld_Func2_Tab`, paired negative accumulator write, and signed `vd` return. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Scroll Transition Boundary Pass

Scope: `OverworldScrollTransition` and `Overworld_SetCameraBoundaries` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`
and the C/Rust overworld scroll tables. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `OverworldScrollTransition` vertical transition branch | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's transition-counter increment, `kOverworld_Func6B_Tab1/2/3` indexing, `byte_7E069E[0]` low-byte write, BG2 vertical scroll update, conditional BG1 vertical copy skip for screens `0x1b/0x5b`, delayed Link Y movement, target comparison through `up_down_scroll_target[y]`, upward-transition BG2 two-pixel correction, Link Y alignment, camera high/low setup, and `overworld_unk1` pair clear. |
| `OverworldScrollTransition` horizontal transition branch | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `byte_7E069E[1]` low-byte write, BG2 horizontal scroll update, conditional BG1 horizontal copy skip for screens `0x1b/0x5b`, delayed Link X movement, target comparison through the same scroll-target word array at index `y`, Link X alignment, camera high/low setup, and `overworld_unk3` pair clear. |
| `OverworldScrollTransition` completion side effects | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's area calculation from `(current_area_of_player >> 1) + kOverworld_Func6B_AreaDelta[y]`, camera-boundary reload, `flag_overworld_area_did_change = 1`, submodule advance, subsubmodule clear, transition-counter clear, sprite slot reset, and final `rv` return. |
| `Overworld_SetCameraBoundaries` table indexing | `crates/zelda3/src/overworld.rs` | fixed | C indexes the overworld offset, size, and scroll-target tables directly with `area` and `big`. Rust had masked `area & 0x3f` and collapsed any nonzero `big` to `1`, which could hide bad state and load a different boundary set than C. Rust now release-asserts the valid C call range and indexes directly. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Finalize Entry Scroll Pass

Scope: `Overworld_FinalizeEntryOntoScreen` in `src/overworld.c`, checked
directly against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_FinalizeEntryOntoScreen` entry nudge and completion | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `Link_HandleMovingAnimation_FullLongEntry()` call, signed two-pixel movement chosen from `byte_7E069C`, X-vs-Y coordinate update from bit 1, completion test using `(d & 0xfe)` against `kOverworld_Func8_tab[byte_7E069C]`, submodule/subsubmodule reset, `overworld_music[BYTE(overworld_screen_index)]` lookup, ambient SFX write, conditional `music_control` update when `music_unk1 == 0xf1`, `Overworld_OperateCameraScroll()` call, and conditional `OverworldHandleMapScroll()` call from the low byte of `overworld_screen_trans_dir_bits2`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Entrance Dispatch/PoD Pass

Scope: `Overworld_AnimateEntrance`, `Overworld_AnimateEntrance_PoD`,
`OverworldEntrance_AdvanceAndBoom`, and `OverworldEntrance_PlayJingle` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| entrance animation dispatcher | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's copy of `trigger_special_entrance` into `flag_is_link_immobilized`, `flag_unk1`, and `nmi_disable_core_updates`, and dispatches trigger values `1..=5` to the same PoD, Skull, Mire, Turtle Rock, and Ganon's Tower handlers. Rust previously ignored invalid trigger values; it now panics like C's direct table index would fail. |
| `Overworld_AnimateEntrance_PoD` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule cases, entrance counter thresholds `0x40/0x20`, `OverworldEntrance_AdvanceAndBoom()` calls, save event bit `0x5e |= 0x20`, PoD tile writes including the intentional duplicate `0x02ea` update in case 0, NMI BG reload flag, and final jingle case. |
| `OverworldEntrance_AdvanceAndBoom` / `OverworldEntrance_PlayJingle` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule advance, entrance counter clear, SFX writes `12` and `7`, final jingle SFX `27`, trigger clear, subsubmodule clear, core-update/immobilization flag clears, and BG1 offset word clears. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Skull Woods Entrance Pass

Scope: `Overworld_AnimateEntrance_Skull` in `src/overworld.c`, checked
directly against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_AnimateEntrance_Skull` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule cases `0..=4`, entrance-counter thresholds `4/12/12/12/12`, counter clear, subsubmodule advance, case-0 save event bit for the current screen, each `Overworld_DrawMap16_Persist` tile batch, NMI BG reload flag, `sound_effect_2 = 0x16`, and final `OverworldEntrance_PlayJingle()` call after the last tile batch. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Misery Mire Entrance Pass

Scope: `Overworld_AnimateEntrance_Mire` in `src/overworld.c`, checked directly
against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| shake and TS bit phase | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's BG1 X/Y shake when `subsubmodule_index >= 2`, frame-counter parity choosing `-1/+1`, first-phase counter increment, 32-frame delay, `j -= 32`, completion at `j == 207`, counter clear, subsubmodule set to 1, and `TS_copy` bit lookup from the 26-byte Misery Mire bit table. |
| staged tile reveal sequence | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's cases `1|2`, `3`, `4`, and `5`: counter thresholds `16/72/72/80/128`, subsubmodule advance at counter 16, ambient SFX 7 then final 5, `OverworldEntrance_AdvanceAndBoom()` calls, current-screen event bit, tile batches starting at `0xe48`, `0xe54`, and `0xe64`, NMI BG reload flag, and final `OverworldEntrance_PlayJingle()`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Turtle Rock Entrance Pass

Scope: `Overworld_AnimateEntrance_TurtleRock`, `OverworldEntrance_DrawManyTR`,
and `turtle_rock_vram_common` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| initial VRAM upload phases | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's BG1 shake every call, event bit set in case 0, fixed-color approach call, four VRAM upload header phases `0x10/0x14/0x18/0x1c`, common upload words `0xfe47` and `0x01e3`, low-byte `0xff` terminator marker, subsubmodule advance, and NMI BG reload flag. |
| palette/window and TR tile draw phases | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's case-4 palette clears at main `0x58` and aux `0x68`, BG1 scroll copies from BG2, CGRAM update flag, `OverworldEntrance_DrawManyTR()` 16-tile batch, `TS/CGWSEL/CGADSUB` changes, VRAM upload packet high-bit rewrite, `0x08aa` to `0x01e3` substitutions, counter clear, and subsubmodule advance. |
| Turtle Rock counter phases | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `turtlerock_ctr` byte alias behavior through `overworld_entrance_sequence_counter`, even-frame palette restore cadence, SFX `2`, wrap/decrement behavior from zero through `0xff`, reset to `0x30`, final draw-many call, window/subscreen restore, ambient SFX `5`, and final `OverworldEntrance_PlayJingle()` case. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Ganon Tower Entrance Pass

Scope: `Overworld_AnimateEntrance_GanonsTower` in `src/overworld.c`, checked
directly against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| initial flash cycle | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's cases `0` and `1` event bit set for the current screen, repeated `GanonTowerEntrance_Func1()` calls, case-2 `TS_copy` gate, `TS_copy = 1`, `ganonentrance_ctr` byte increment, reset at count 3 with ambient SFX 7, and otherwise returning to subsubmodule 0. |
| tower tile reveal sequence | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's cases `3..=11`, counter thresholds `48/48/52/32/32/32/32/32/32`, `OverworldEntrance_AdvanceAndBoom()` calls, every persisted tile position/value batch from `0x45e..0x5e0`, NMI BG reload through `entrance_draw_tiles`, and ambient SFX 5 in case 11. |
| final handoff | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's case-12 counter threshold 72, `OverworldEntrance_PlayJingle()`, counter clear, `music_control = 13`, and `sound_effect_ambient = 9`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Event Overlay Pass

Scope: `Overworld_LoadEventOverlay` in `src/overworld.c`, checked directly
against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| screen-group dispatch | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's switch groupings for screens `0..=127`, including the shared 2x2 overlay helper cases, screen `107`, `119`, `123`, and the assert-only invalid screens `114..=118`, `122`, and `124..=127`. |
| overlay tile writes | `crates/zelda3/src/overworld.rs` | verified | The reviewed Rust writes target the same `dung_bg2[XY(x,y)]` positions and values as C for the castle, single-tile, 2x2, desert/pond/entrance, waterfall, grove, pyramid/entrance, and multi-row overlay groups. The helper `write_bg2_xy` uses the same `XY(x,y) = y * 64 + x` address math. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Map16 Draw Upload Pass

Scope: `Overworld_DrawMap16_Persist`, `Overworld_DrawMap16`,
`Overworld_AlterTileHardcore`, and `Overworld_FindMap16VRAMAddress` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_DrawMap16_Persist` / `Overworld_AlterTileHardcore` BG2 update | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's persistent map16 write to `dung_bg2[pos >> 1]` before preparing the VRAM upload packet. |
| `Overworld_DrawMap16` VRAM upload packet | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's map16 VRAM address conversion, upload destination at `vram_upload_data[vram_upload_offset >> 1]`, swapped top-row VRAM address, `0x300` transfer sizes, two top map8 words, swapped lower-row address `pos + 0x20`, two lower map8 words, `0xffff` terminator, and `vram_upload_offset += 16`. Rust now requires asset 70 and indexes the map16-to-map8 table directly instead of using a zero-fallback asset helper. |
| `Overworld_FindMap16VRAMAddress` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's nametable quadrant bits from `addr & 0x3f` and `addr & 0xfff`, low column bits `addr & 0x1f`, and row bits `(addr & 0x780) >> 1`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Bomb Tile/Smash Helper Pass

Scope: `Overworld_BombTiles32x32` and `Overworld_BombTile` in
`src/overworld.c`, with the shared Rust smash helpers in
`crates/zelda3/src/player.rs`, checked directly against
`crates/zelda3/src/overworld.rs` and `crates/zelda3/src/player.rs`. This pass
did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_BombTiles32x32` scan | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `(x - 23) & ~7`, `(y - 20) & ~7`, three-by-three nested scan with 16-pixel increments, and final `word_7E0486/word_7E0488` writes after `y` has advanced through the loop. |
| `Overworld_BombTile` terrain/secret behavior | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's map16 position math, follower-13 bypass, smashable tile IDs `0x36/0x72a/0x37e`, terrain spawn IDs and fallback tiles, reveal-secret replacement, `dung_bg2` writes, map16 memorize/draw calls, smashed-terrain spawn coordinates, NMI BG reload flag, secondary `0xdb4/0xdb5` overlay handling, event bit set, and the intentional C quirk that memorizes `0xdb5` at `pos` while drawing it at `pos + 2`. |
| shared smash secret/map16 helpers | `crates/zelda3/src/player.rs` | fixed | The shared helpers now match the direct table access audited for the overworld versions: secret offset/data assets and map16-to-map8 asset 70 are required and indexed directly, rather than using zero-fallback asset helpers. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Waterfall Splash Termination Pass

Scope: `Ancilla_TerminateWaterfallSplashes` in `src/overworld.c`, checked
directly against `crates/zelda3/src/overworld.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla_TerminateWaterfallSplashes` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's low-byte screen check for overworld screen `0x0f`, reverse scan of ancilla slots `4..=0`, and clearing only `ancilla_type[i] == 0x41`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Death Mountain Palette Animation Pass

Scope: `Overworld_DwDeathMountainPaletteAnimation` in `src/overworld.c`,
checked directly against `crates/zelda3/src/overworld.rs`. This pass did not
use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_DwDeathMountainPaletteAnimation` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's `trigger_special_entrance` early return, screen filter for `0x43/0x45/0x47`, frame-counter restore frames `5/44/90`, apply frames `3/36/88`, SFX on frame `36`, five seven-color palette block copies from aux or `kDwPaletteAnim`, unconditional CGRAM update flag increment after the screen/frame work, event-bit early return for screens `0x43/0x45`, `(frame_counter & 0x0c) * 2` offset, default offset `32`, and eight-word write from `kDwPaletteAnim2` to palette index `0x68`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Entry Countdown/Music Setup Pass

Scope: `Overworld_Func1F`, `ConditionalMosaicControl`,
`Overworld_ResetMosaic_alwaysIncrease`, and `Overworld_SetSongList` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_Func1F` entry countdown movement | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's moving-animation call, signed one-pixel velocity from `byte_7E069C`, X-vs-Y coordinate and velocity update from bit 1, pre-decrement countdown test, main-module handoff to 9, submodule/subsubmodule clear, and final `Overworld_OperateCameraScroll()` call. |
| `ConditionalMosaicControl` / `Overworld_ResetMosaic_alwaysIncrease` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C: conditional mosaic increment on odd `palette_filter_countdown`, unconditional increment in the reset helper, `BGMODE_copy = 9`, and `MOSAIC_copy = mosaic_level | 7`. |
| `Overworld_SetSongList` progress/sword selection | `crates/zelda3/src/overworld.rs` | fixed | The progress and sword-type branch matches C's `r0` and source offset selection. Rust previously skipped or partially copied if the extracted music assets were missing/truncated, unlike C's fixed-size `memcpy` from static tables. Rust now requires assets 111/112 and performs exact 64-byte and 96-byte copies before writing `overworld_music[128]`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Map16 Secret/Attribute Pass

Scope: `Overworld_Memorize_Map16_Change`, `Overworld_ReadTileAttribute`,
`Overworld_RevealSecret`, and `AdjustSecretForPowder` in `src/overworld.c`,
checked directly against `crates/zelda3/src/overworld.rs`. This pass did not
use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_Memorize_Map16_Change` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's skip for values `0xdc5/0xdc9`, word index from `num_memorized_tiles >> 1`, value/address writes, and `num_memorized_tiles = x + 2` update. |
| `Overworld_ReadTileAttribute` | `crates/zelda3/src/overworld.rs` | fixed | Coordinate masking and `dung_bg2[t >> 1]` lookup match C. Rust previously read `kSomeTileAttr` through a helper that returned zero for missing/out-of-range asset data; it now requires asset 164 and indexes it directly like C. |
| `Overworld_RevealSecret` | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's byte clear of `dung_secrets_unk1`, screen `>= 0x80` fail path, three-byte secret entry scan, low-data bit OR, fail/return behavior, `0xff` reveal marker, follower gate for screen `0x5b`, enhanced-feature discovery chime case, tile-below table, and powder adjustment. Rust now requires the secret offset/data assets and indexes them directly instead of treating missing asset bytes as zero. |
| `AdjustSecretForPowder` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's word write of `dung_secrets_unk1 = 4` when `link_item_in_hand & 0x40` is set. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Peg Puzzle Pass

Scope: `HandlePegPuzzles` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| light-world Turtle Rock peg sequence | `crates/zelda3/src/overworld.rs` | fixed | Rust matches C's screen-7 event-bit early return, `word_7E04C8 != 0xffff` gate, table lookup by `word_7E04C8 >> 1`, correct-hit SFX, `word_7E04C8 += 2`, completion SFX/event bit/submodule handoff, wrong-hit SFX, and reset to `0xffff`. Rust had an extra bounds check before the three-entry table lookup; it now indexes directly like C. |
| dark-world peg counter event | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's screen-98 pre-increment of `word_7E04C8`, completion at 22, event bit set, `sound_effect_2 = 27`, door counter `0x50`, big-rock address `0xd20`, and `Overworld_DoMapUpdate32x32_B()` call. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Overworld Special Switch/Ganon Tower Pass

Scope: `GanonTowerEntrance_Func1`, `Overworld_CheckSpecialSwitchArea`,
`ScrollAndCheckForSOWExit`, and `Overworld_GetMap16OfLink_Mult8` in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `GanonTowerEntrance_Func1` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule-zero branch, `sound_effect_1 = 0x2e`, `Palette_AnimGetMasterSword2()` call, blinding-white palette filter, darkening completion check against `0xff`, palette countdown set to `0xff`, subsubmodule increment, and `Palette_AnimGetMasterSword3()` fallback. |
| `Overworld_CheckSpecialSwitchArea` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's four-entry reverse scan, `map8[0] & 0x1ff` comparison, full-word screen comparison, dungeon-room exit write, byte writes to both transition-direction fields and `link_direction`, word writes to `byte_7E069C` and `overworld_screen_transition` from `DirToEnum`, submodule `23`, and main module `11`. |
| `ScrollAndCheckForSOWExit` special-switch B path | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's conditional `OverworldHandleMapScroll()` call, three-entry reverse scan, `map8[0] & 0x1ff` comparison, screen comparison, link-direction write, `DirToEnum` transition writes, submodule `36`, subsubmodule clear, and low-byte clear of `dungeon_room_index`. |
| `Overworld_GetMap16OfLink_Mult8` map16 lookup | `crates/zelda3/src/overworld.rs` | fixed | Coordinate bias/masking and `dung_bg2[pos >> 1] * 4` table offset match C. Rust previously read the four map8 words through `asset_u16`, which returned zero on missing/out-of-range asset bytes; it now requires asset 70 and indexes the map16-to-map8 table directly like C. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Master Sword Palette Animation Pass

Scope: `Palette_AnimGetMasterSword`, `Palette_AnimGetMasterSword2`, and
`Palette_AnimGetMasterSword3` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Palette_AnimGetMasterSword` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule-zero handoff to `Palette_AnimGetMasterSword2`, blinding-white filter path, completion check against `darkening_or_lightening_screen == 0xff`, eight-word zeroing in main and aux palette buffers at palette index `0x58`, palette countdown clear, darkening state clear, submodule reset, and fallback to `Palette_AnimGetMasterSword3`. |
| `Palette_AnimGetMasterSword2` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's 512-byte copy from aux palette to map backup, 256-word aux palette fill with `0x7fff`, `main_palette_buffer[32] = main_palette_buffer[0]`, palette countdown clear, darkening state set to 2, and subsubmodule increment. |
| `Palette_AnimGetMasterSword3` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's early return unless `darkening_or_lightening_screen == 0` and `palette_filter_countdown == 31`, 512-byte restore from map backup into aux palette, and `TS_copy = 0`. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Drowning Recovery Cached Position Pass

Scope: `RecoverPositionAfterDrowning`, `AdjustLinkBunnyStatus`, and
`ForceNonbunnyStatus` in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module09_2A_RecoverFromDrowning` / `Module09_2A_00_ScrollToLand` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's subsubmodule dispatch, two-pixel step toward cached Link coordinates, low-byte velocity writes, completion transition, incapacitated/enemy-damage clears, `Overworld_OperateCameraScroll()` call, and conditional `OverworldHandleMapScroll()` call. |
| `RecoverPositionAfterDrowning` cached position/scroll restore | `crates/zelda3/src/overworld.rs` | verified | Rust restores `link_x_coord`, `link_y_coord`, the four cached `OwScrollVars` fields at `$600/$604/$608/$60c`, the four scroll target words, indoor camera low/high words, quadrant words, outdoor camera high-word adjustment, Link direction/floor/doorway/lower-level state, visibility/blink/sprite-damage state, tagalong state, water-puzzle flag, module/submodule state, and zero-health death-module handoff in the same order and with the same byte/word widths as C. |
| `AdjustLinkBunnyStatus` / `ForceNonbunnyStatus` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C: only force non-bunny status when Moon Pearl is owned, reset player handler/temp-bunny/poof/bunny mirror state, and clear `link_is_running` only under the turn-while-dashing enhanced feature. |
| `TakeDamageFromPit` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C: set Link visibility to 12, choose submodule 20 indoors or 42 outdoors, subtract 8 health with byte wrap semantics, and clamp wrapped/underflowed health values `>= 0xa8` to zero. |
| `Overworld_GetPitDestination` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C's fall-hole destination lookup: align Link coordinates to 8 pixels, derive the masked overworld position from area offsets, scan `kFallHole_*` entries from index 18 down, set `which_entrance` and clear `byte_7E010F` on match, otherwise clear dark-world state and route to entrance 130. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Exit Restore/Music Pass

Scope: `LoadOverworldFromDungeon`, `LoadCachedEntranceProperties`,
`PreOverworld_LoadProperties`, `LoadOWMusicIfNeeded`, and the spotlight
close/open handoff in `src/overworld.c`, checked directly against
`crates/zelda3/src/overworld.rs` and `crates/zelda3/src/dungeon.rs`. This pass
did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LoadCachedEntranceProperties` cached overworld restore | `crates/zelda3/src/overworld.rs` | verified | Rust restores the same cached area, TM, scroll, Link position, direction, screen index, map16 source/destination derived fields, camera scroll, scroll bounds, scroll targets, overworld unk values, and tile/sprite theme bytes as C. The `dungeon_room_index < 0x124` Y adjustment and `ow_entrance_value == 0xffff` doorway adjustment also match. |
| `LoadOverworldFromDungeon` normal/special exit dispatch | `crates/zelda3/src/overworld.rs` | fixed | The branch predicate matches C: cached entrance data for rooms `0x100..0x17f` except `0x104`, exit-data table otherwise. Rust had silently fallen back to exit table slot 0 if no matching special-exit room was found; C has no valid fallback. Rust now panics with the missing room instead of loading the wrong overworld exit data. |
| `PreOverworld_LoadProperties` overworld music/theme selection | `crates/zelda3/src/overworld.rs` | verified | Rust matches the C screen/dungeon-room decision tree for queued music, animated overworld tile set, dark-world/bunny overrides, palette/theme loads, mirror portal reload, ambient sound, state clears, and final `LoadOWMusicIfNeeded()` call. |
| `AdjustLinkBunnyStatus` / `ForceNonbunnyStatus` | `crates/zelda3/src/overworld.rs` | verified | Rust matches C: only force non-bunny status when Moon Pearl is owned, reset player handler/temp-bunny/poof/bunny mirror state, and clear `link_is_running` only under the turn-while-dashing enhanced feature. |
| `LoadOWMusicIfNeeded` | `crates/zelda3/src/dungeon.rs` | verified | Rust matches C: return when `flag_which_music_type` is zero, otherwise clear it and call the overworld song-list loader. |
| `Module0F_SpotlightClose` / `Dungeon_PrepExitWithSpotlight` / `OpenSpotlight_Next2` | `crates/zelda3/src/overworld.rs` | verified | The reviewed spotlight close/open handoff matches C for sprite/update calls, entrance music fade logic via `ZeldaGetEntranceMusicTrack`, HUD/spotlight state updates, overworld velocity handling, Link direction mask selection, forced blank/item reset, big-rock door handling, window mask clears, sword-hold clear, and special COLDATA values. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Dungeon Room Quadrant Upload Pass

Scope: `Dungeon_LoadAndDrawRoom`, `Dungeon_UploadRoomQuadrants`,
`Dungeon_PrepareNextRoomQuadrantUpload`, and `TileMapPrep_NotWaterOnTag`,
checked directly against `src/dungeon.c` and `src/nmi.c`. This pass did not use
the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_LoadAndDrawRoom` wrapper | `crates/zelda3/src/dungeon.rs` | verified | Rust preserves the C save/disable/restore of `HDMAEN_copy`, calls `Dungeon_LoadRoom()`, clears `overworld_screen_transition` and `overworld_map_state`, uploads all quadrants, then clears `nmi_subroutine_index`, `overworld_map_state`, and `subsubmodule_index`. |
| quadrant upload loop state | `crates/zelda3/src/dungeon.rs` | fixed | C initializes `dung_cur_quadrant_upload = 0` and loops until it reaches 16. Rust had a direct loop over literal quadrant values and did not maintain the same intermediate `dung_cur_quadrant_upload` state. Rust now follows the C loop shape. |
| `TileMapPrep_NotWaterOnTag` source layer | `crates/zelda3/src/dungeon.rs` | fixed | C copies from `dung_bg1` (`g_ram+0x4000`) and targets `kUploadBgDsts[ofs] + 0x10`. Rust had this helper reading the BG2 buffer. It now reads `DUNG_BG1`. |
| `Dungeon_PrepareNextRoomQuadrantUpload` source layer | `crates/zelda3/src/dungeon.rs` | fixed | C copies from `dung_bg2` (`g_ram+0x2000`), increments `dung_cur_quadrant_upload` by 4, and targets `kUploadBgDsts[ofs]`. Rust had this helper reading the BG1 buffer. It now reads `DUNG_BG2`. |
| immediate tilemap upload | `crates/zelda3/src/dungeon.rs` | verified | Rust `upload_tilemap_now()` mirrors `NMI_UploadTilemap()` for the lockstep path: copy the prepared packet into VRAM, clear the first word at `$1000`, and clear `nmi_disable_core_updates`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- A first literal-loop attempt diverged at frame 2203 because the two Rust helper source layers were swapped; after fixing the helpers, the same route passes.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Dungeon Entrance Load Branch/Music Pass

Scope: `Dungeon_LoadEntrance` in `src/dungeon.c`, checked directly against
`crates/zelda3/src/dungeon.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_LoadEntrance` BG/death-state clear | `crates/zelda3/src/dungeon.rs` | fixed | C clears `bg1_y_offset`, `bg1_x_offset`, and `WORD(death_var5)` after saving overworld exit state. Rust had been clearing `0x110` (`dung_index_x3`) instead of the BG1 offset/death-var words. Rust now clears the same words as C. |
| `Dungeon_LoadEntrance` starting-point branch | `crates/zelda3/src/dungeon.rs` | fixed | C uses the `kStartingPoint_*` asset tables when `WORD(follower_indicator) == 4` or `WORD(death_var4) != 0`, updates `which_entrance`, applies starting-point room/scroll/camera/bounds/floor/quadrant/music state, and clears `death_var4`. Rust previously always loaded `kEntranceData_*`. Rust now has the same starting-point branch using assets 28..45. |
| `Dungeon_LoadEntrance` normal entrance music | `crates/zelda3/src/dungeon.rs` | fixed | C calls `ZeldaGetEntranceMusicTrack(i)` instead of reading `kEntranceData_musicTrack` directly, preserving MSU Deluxe fade-track remapping. Rust now calls the same helper and keeps the `track == 3 && sram_progress_indicator >= 2` rewrite. |
| `Dungeon_LoadEntrance` normal entrance fields | `crates/zelda3/src/dungeon.rs` | verified | The normal branch still loads the same entrance assets for room, scroll, player position when progress is nonzero, camera, room bounds, tile theme, floor, palace, doorway orientation, BG level, quadrant, and the `room >= 0x100` floor override. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 House Re-entry Pre-Dungeon Load/Music Pass

Scope: Link's house re-entry path from overworld into `Module_PreDungeon`,
checked directly against `src/dungeon.c` and the Rust pre-dungeon loader in
`crates/zelda3/src/dungeon.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module_PreDungeon` room load sequence | `crates/zelda3/src/dungeon.rs` | fixed | C calls `Dungeon_LoadAndDrawRoom()` and then `Dungeon_LoadCustomTileAttr()` before animated-tile decompression and attribute-table load. Rust had been doing the lower-level `Dungeon_LoadRoom()` + quadrant upload path and skipped custom tile attributes, which could leave an entered room partially initialized. Rust now follows the C sequence. |
| `Module_PreDungeon` song-bank setup | `crates/zelda3/src/dungeon.rs` | fixed | C calls `Dungeon_LoadSongBankIfNeeded()` immediately before `Module_PreDungeon_setAmbientSfx()`. Rust skipped that call, leaving entrance music bank/control state stale on indoor entry/re-entry. Rust now calls the song-bank loader at the same point as C. |
| overworld entrance handoff | `crates/zelda3/src/overworld.rs` | verified | `Overworld_LoadEntrance` still matches the C handoff for detected entrances: set `which_entrance`, clear Link auxiliary/incapacitated state, switch to main module `15`, set saved module `6`, and clear submodule/subsubmodule. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 7a85b12394133e02`.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` completes with `WRAM fnv1a64 = 3b99d54bddde282e`.
- Attempted `scripts/inputs/sram-starting-house-exit.txt` still diverges at frame 5898 on two PPU register bytes before it can prove house re-entry, so it was not used as passing evidence for this fix.
- Attempted a `--trace-state` run with `scripts/inputs/file-select-enter-game-exit-house.txt`; the route was still in the starting-house/message flow past frame 100k and was stopped as non-actionable evidence for re-entry.

## 2026-05-31 Messaging/Ancilla/Player Assert Parity Pass

Scope: remaining release-vs-debug assertion gaps in player movement, ancilla
draw, and message rendering paths, checked directly against the matching C
assertions. This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleVelocity` dash counter invariant | `crates/zelda3/src/player.rs` | fixed | C asserts `link_dash_ctr >= 32` when Link is running but not in the moving branch. Rust now uses a release assertion at the same point instead of silently continuing. |
| `SpinSpark_Draw` OAM table index invariant | `crates/zelda3/src/ancilla.rs` | fixed | C asserts the computed spin-spark OAM table index is below 32. Rust now uses a release assertion instead of a debug-only check. |
| `RenderText_Draw_MessageCharacters` unsupported command cases | `crates/zelda3/src/messaging.rs` | fixed | C asserts for command types that should have been handled during text-buffer expansion or are unused. Rust no longer drops unhandled decoded commands silently; it now panics with the decoded command/param. |
| `VWF_RenderSingle` font width invariant | `crates/zelda3/src/messaging.rs` | fixed | C asserts font glyph width is at most 8. Rust now asserts this instead of clamping wider widths to 8. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45000 --input-script scripts/inputs/opening-uncle-message-diagonal-sweeps.txt` completes with `WRAM fnv1a64 = 7c8361d4699796d7`.

## 2026-05-31 Gameplay Invariant Assert Pass

Scope: debug-only gameplay invariants in `player.rs` and `dungeon.rs`, checked
directly against the matching C assertions. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HoppingHorizontally_FindTile_X` direction invariant | `crates/zelda3/src/player.rs` | fixed | C asserts that the horizontal hop offset selector is `0` or `2`. Rust now uses a release `assert!` instead of a debug-only check. |
| `WaterFlood_BuildOneQuadrantForVRAM` tag invariant | `crates/zelda3/src/dungeon.rs` | fixed | C asserts `dung_hdr_tag[0] != 25` before the non-water tag tile prep path. Rust now uses a release `assert_ne!` instead of a debug-only check. |
| `ChangeDoorToSwitch` door state invariant | `crates/zelda3/src/dungeon.rs` | fixed | C asserts `dung_unk5 == 0`. Rust now uses a release `assert_eq!` instead of a debug-only check. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Dungeon Object Decode Assert Pass

Scope: dungeon type-1 object subtype decoder assert/default paths in
`src/dungeon.c`, checked directly against `crates/zelda3/src/dungeon.rs`. This
pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LoadType1ObjectSubtype1` unsupported/default object IDs | `crates/zelda3/src/dungeon.rs` | fixed | C explicitly asserts for several unused object IDs and asserts in the default case. Rust no longer silently ignores unhandled subtype-1 object IDs; invalid or unhandled IDs now panic with the object id. |
| `LoadType1ObjectSubtype2` unsupported/default object IDs | `crates/zelda3/src/dungeon.rs` | fixed | C asserts for unused north-stairs object `0x30` and the default case. Rust now panics for invalid or unhandled subtype-2 object IDs instead of treating them as no-ops. |
| `LoadType1ObjectSubtype3` unsupported/default object IDs | `crates/zelda3/src/dungeon.rs` | fixed | C asserts for unused submerged stair object IDs `0x34..=0x39` and the default case. Rust now panics for invalid or unhandled subtype-3 object IDs instead of silently dropping them. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` reports `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 Dungeon Door Assert Dispatch Pass

Scope: dungeon entrance-door assert helpers and edge-transition direction
dispatch in `src/dungeon.c`, checked directly against
`crates/zelda3/src/dungeon.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Door_Up_EntranceDoor` / `Door_Down_EntranceDoor` / `Door_Left_EntranceDoor` / `Door_Right_EntranceDoor` | `crates/zelda3/src/dungeon.rs` | verified | All four C helpers are intentional `assert(0)` slots for entrance-door draw types. Rust keeps all four as unconditional panics, so these are not missing door renderer implementations. |
| `Dungeon_HandleEdgeTransitionMovement` invalid direction | `crates/zelda3/src/dungeon.rs` | verified | C masks `link_direction` through the four-entry direction table and asserts in the default switch case. Rust uses the same four direction entries and a release `assert!` before indexing, then dispatches the same right/left/down/up transition starters. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Gameplay Assert Release Parity Pass

Scope: remaining gameplay-state debug-only assert fallbacks found in
`crates/zelda3/src`, checked directly against the matching C `assert(0)`
defaults. This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleYItem` default item slot | `crates/zelda3/src/player.rs` | fixed | C asserts if `current_item_active` is outside item slots `0..=21`. Rust now uses an unconditional panic with the bad item slot instead of a debug-only assert, so release/playable builds retain the same invalid-state stop. |
| `Dungeon_LoadAttribute_Selectable` default state | `crates/zelda3/src/dungeon.rs` | fixed | C asserts if `overworld_map_state` is outside states `0..=5`. Rust now uses an unconditional panic with the bad state instead of a debug-only assert. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Misc Assert/Item DMA Guard Pass

Scope: selected `src/misc.c` assert-only dispatch slots and the Link sprite DMA
table guard around `NMI_PrepareSprites`, checked directly against
`crates/zelda3/src/misc.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module_Unknown0` / `Module_Unknown1` | `crates/zelda3/src/misc.rs` | fixed | Both C functions are unconditional `assert(0)` main-module slots. Rust now uses unconditional panics instead of debug-only asserts, so release builds preserve the same unreachable-slot behavior. |
| `NMI_PrepareSprites` Link DMA static table indexes | `crates/zelda3/src/misc.rs` | verified-limited | C indexes the Link DMA source/count tables directly from the computed DMA variables. Rust keeps the same index math at the reviewed call sites and routes the table reads through `link_dma_table_value` so an out-of-range state reports the exact table/index instead of producing a vague bounds panic. Full byte-for-byte table audit remains separate. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`

## 2026-05-31 Overworld Assert/Event Overlay Pass

Scope: overworld assert-only dispatch slots and event-overlay screen dispatch in
`src/overworld.c`, checked directly against `crates/zelda3/src/overworld.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_Func1D` / `Overworld_Func1E` | `crates/zelda3/src/overworld.rs` | verified | Both C functions are `assert(0)` dispatch slots. Rust keeps these as panic-only paths, so they are not missing overworld implementations. |
| `Overworld_LoadEventOverlay` invalid screens | `crates/zelda3/src/overworld.rs` | verified-limited | The reviewed overlay screen groups match C for the named case ranges, and Rust keeps the explicit C assert range `114..=118`, `122`, and `124..=127` as a panic. Full tile-pair parity for every overlay group still needs a broader table-by-table audit. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` reports `WRAM fnv1a64 = 3b99d54bddde282e`.
- Attempted `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 42000 --input-script scripts/inputs/sram-starting-house-exit.txt --load-sram saves/sram.dat`; it diverged at frame 5898 on two PPU register bytes and emitted the existing frame-logic skeleton warning, so this route was not used as passing evidence for this pass.

## 2026-05-31 Messaging Dispatch Assert Pass

Scope: messaging/interface dispatch assert slots and map sub-state dispatch in
`src/messaging.c`, checked directly against `crates/zelda3/src/messaging.rs`.
This pass did not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module_Messaging_0` / `Module_Messaging_6` | `crates/zelda3/src/messaging.rs` | verified | Both C functions are `assert(0)` dispatch slots. Rust keeps them as panic-only paths, so they are not missing interface implementations. |
| `RunInterface` dispatch | `crates/zelda3/src/messaging.rs` | verified | Rust dispatches submodules 0..=11 to the same `kMessagingSubmodules` targets as C. The out-of-range panic is the Rust bounds guard for C's direct table index. |
| `Module0E_0A_FluteMenu` | `crates/zelda3/src/messaging.rs` | verified | Flute/world-map states 0..=9 match C: fade out, load light map, sprite GFX, brighten, delay setup, selection, graphics restore, selected-screen load, overlay/map rebuild, and fade-in/quack. The default panic mirrors C `assert(0)`. |
| `Module0E_03_01_DrawMap` | `crates/zelda3/src/messaging.rs` | verified | Dungeon-map init states 0..=4 dispatch to the same five `kDungMapInit` entries as C. The default panic is the Rust bounds guard for C's direct table index. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45000 --input-script scripts/inputs/opening-uncle-message-diagonal-sweeps.txt` reports `WRAM fnv1a64 = 7c8361d4699796d7`.

## 2026-05-31 Ancilla Assert Slot Pass

Scope: unused ancilla dispatch entries in `src/ancilla.c`, checked directly
against `crates/zelda3/src/ancilla.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla_Unused_14` / `Ancilla_Unused_25` | `crates/zelda3/src/ancilla.rs` | verified | Both C functions are `assert(0)` dispatch slots. Rust keeps these as panic-only paths, so they are not missing gameplay implementations. |

Checks after this pass:

- `cargo check -q -p zelda3`

## 2026-05-31 HUD Menu State Pass

Scope: `Hud_Module_Run`, the inventory menu state machine, and
`Hud_GetItemBoxPtr` / item-switch helpers in `src/hud.c`, checked directly
against `crates/zelda3/src/hud.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Hud_Module_Run` dispatch | `crates/zelda3/src/hud.rs` | verified | Rust increments `byte_7E0206` and dispatches `overworld_map_state` 0..=12 to the same clear/init/menu/update/close/bottle-menu helpers as C. The default panic mirrors C `assert(0)`. |
| inventory navigation helpers | `crates/zelda3/src/hud.rs` | verified | `Hud_HaveAnyItems`, `Hud_DoWeHaveThisItem`, `Hud_GotoPrevItem`, `Hud_GotoNextItem`, `Hud_EquipPrevItem`, `Hud_EquipNextItem`, above/below movement, current-button selection, and reorder behavior match C for the compiled old-style inventory mode (`kNewStyleInventory = 0`). |
| normal and bottle menu states | `crates/zelda3/src/hud.rs` | verified | `Hud_Init`, `Hud_BringMenuDown`, `Hud_ChooseNextMode`, `Hud_NormalMenu`, `Hud_UpdateHud`, `Hud_CloseMenu`, `Hud_GotoBottleMenu`, `Hud_InitBottleMenu`, `Hud_ExpandBottleMenu`, `Hud_BottleMenu`, `Hud_EraseBottleMenu`, and `Hud_RestoreNormalMenu` match the C state transitions, BG3 scroll deltas, NMI upload flags, SFX writes, bottle index behavior, and close-menu restoration paths reviewed in this pass. |
| bottle menu selected item draw | `crates/zelda3/src/hud.rs` | fixed | `Hud_DrawBottleMenu` now matches C's direct `link_bottle_info[link_item_bottle_index - 1]` lookup and unconditional draw into the selected Y-item slot. Rust previously guarded `link_item_bottle_index == 0` and skipped the draw/flashing circle, which could hide invalid state and leave stale menu tile data instead of behaving like C's direct table access. |
| floor and super-bomb indicators | `crates/zelda3/src/hud.rs` | fixed | `Hud_RefreshIcon`, `CheckPalaceItemPosession`, `Hud_FloorIndicator`, `Hud_RemoveSuperBombIndicator`, `Hud_SuperBombIndicator`, and `MaxRupees` were compared against C. Rust now matches C's byte read of `hud_floor_changed_timer` followed by the word write back through `WORD(hud_floor_changed_timer)`; it previously read the word and could let the adjacent high byte affect the timer. The floor tile writes, ambient SFX choices, signed floor/super-bomb counter checks, digit table indices, and super-bomb removal path match this reviewed slice. |
| refill/update logic | `crates/zelda3/src/hud.rs` | verified | `Hud_RefillLogic` matches C's `overworld_map_state` gate, magic filler cap/increment/SFX timing, rupee goal stepping and signed underflow clamp, `rupee_sfx_sound_delay++ & 7` behavior, bomb/arrow filler increments, bow odd-item refresh path, low-health beep gate, heart-refill animation goto shape, HUD update helper calls, and `flag_update_hud_in_nmi` increments. |
| refill helper exports and torch restore | `crates/zelda3/src/hud.rs` | verified | `Hud_RefillHealth`, `Hud_AnimateHeartRefill`, `Hud_RefillMagicPower`, and `Hud_RestoreTorchBackground` match C's health-cap/filler writes, countdown predecrement return, `(uint16)((link_health_current & ~7) - 1)` tile index math, partial-heart tile table, subposition wrap/rebuild/animation clear, magic-filler return contract, lantern/dark-room early returns, and `TS_copy` restore gate. |
| top-level HUD constants and icon tables | `crates/zelda3/src/hud.rs` | verified | `kMaxBombsForLevel`, `kMaxArrowsForLevel`, `kMaxHealthForLevel`, old-style `kHudItemInVramPtr`, all primary `ItemBoxGfx` item-art tables, `kUpdateMagicPowerTilemap`, and `kDungFloorIndicator_Gfx0/Gfx1` match C values. Rust represents C's partially initialized `kHudItemArmor[5]` with the same two explicit zero-filled tail entries. The C new-style inventory VRAM table is inactive under `kNewStyleInventory = 0`; this row covers the runtime old-style table that C selects. |
| Y-button and ability draw helpers | `crates/zelda3/src/hud.rs` | verified | `CopyTilesForSwitchLR`, `Hud_DrawYButtonItems`, and `Hud_DrawAbilityBox` match C's LR tile PV bit packing, VRAM tile destinations, switch palettes, old-style item grid destinations, button-letter tiles, header tile writes, ability/gloves text tables, flag-shift loop shape, A/DO text writes, and gloves/boots/flippers/moon-pearl draw slots. |
| progress, selected-item, and equipment draw tables | `crates/zelda3/src/hud.rs` | verified | `Hud_DrawProgressIcons`, pendant/crystal progress backgrounds, pendant item tables, crystal bit placements, `Hud_DrawSelectedYButtonItem` text tables and text-source selection, and `Hud_DrawEquipmentBox` dungeon labels, heart-piece, palace-item, map, and compass tables match C values and destinations for the compiled old-style inventory path. This pass rechecked the selected-item special text sources for bottles, mushroom, mirror, flute, bow, shovel, unassigned buttons, bottle slots, and default item names, plus equipment box dotted lines, dungeon labels, heart-piece fallback, sword/shield/armor slots, and big-key/map/compass mask tests. |
| HUD rebuild/update helpers | `crates/zelda3/src/hud.rs` | fixed | `Hud_RebuildIndoor`, `Hud_Rebuild`, `DrawHudComponents`, `Hud_UpdateItemBox`, `Hud_Update_Hearts`, `Hud_Update_Magic`, `Hud_Update_Inventory`, and `Hud_IntToDecimal` were compared against C. Rust now matches C's promoted `(link_health_current + 3) & ~3` and `(link_magic_power + 7) >> 3` math instead of wrapping those additions in the source byte before indexing/drawing. The tilemap constants, 165-word initialization guard, HUD component copy destinations, inventory digit offsets, bow mutation, key blanking, and NMI HUD flag increment match this reviewed slice. |
| `Hud_GetItemBoxPtr` / item box pointer table | `crates/zelda3/src/hud.rs` | verified | Rust maps item IDs 0..31 to the same item-box art tables C indexes through `kHudItemBoxGfxPtrs`; the out-of-range panic is the Rust bounds guard for C's direct table index. |

Checks after this pass:

- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 120000 --input-script scripts/inputs/file-select-enter-game-button-taps.txt --load-sram saves/sram.dat` reports `WRAM fnv1a64 = ccce839e9d72a8ee`.

## 2026-05-31 HUD Full Source Closure Pass

Scope: close the remaining explicit `hud.c` function-name gaps in the manual
ledger by rechecking the already-covered helper clusters directly against
`crates/zelda3/src/hud.rs`, without using the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| menu dispatch helpers | `crates/zelda3/src/hud.rs` | verified | `Hud_ClearTileMap`, `GetCurrentItemButtonIndex`, `Hud_LookupInventoryItem`, `Hud_UpdateEquippedItem`, and `Hud_SearchForEquippedItem` match C's tilemap clear fill, SFX/NMI/map-state writes, enhanced LR/X/L/R button priority, old-style inventory ID table selected under `kNewStyleInventory = 0`, bottle-index side effect, `current_item_y` update, no-item clear, default item initialization, and invalid-state assertions/bounds guards. |
| menu draw primitives and bottle update wrapper | `crates/zelda3/src/hud.rs` | verified | `Hud_DrawBottleMenu_Update`, `Hud_DrawBox`, and `Hud_DrawFlashingCircle` match C's redraw-selected-item/NMI writes, box corner/edge/interior tile values with palette bits, and the twelve flashing-circle tile offsets and flip bits. |
| item movement helpers | `crates/zelda3/src/hud.rs` | verified | `Hud_EquipItemAbove` and `Hud_EquipItemBelow` match C's old-style `5`-slot vertical movement count, repeated prev/next search loops, and `Hud_DoWeHaveThisItem` termination condition. |
| progress subdraw helpers | `crates/zelda3/src/hud.rs` | verified | `Hud_DrawProgressIcons_Pendants` and `Hud_DrawProgressIcons_Crystals` match C's old-style destination, 10x9 background tilemaps, pendant tile tables, crystal bit positions, and tile values. |
| heart inner update helper | `crates/zelda3/src/hud.rs` | verified | `Hud_UpdateHearts_Inner` matches C's row wrap after ten hearts, source-table index selection from the remaining health count, destination stride, and decrement-by-eight loop. |
| HUD source coverage | `crates/zelda3/src/hud.rs` | source-covered/runtime-open | Every function in `../zelda3/src/hud.c`, including static helpers, now has direct manual C/Rust comparison evidence in this ledger. Runtime-open because menu edge states, enhanced LR item switching, bottle submenus, and HUD indicator variants still need broader route/oracle coverage. |

Checks after this pass:

- `cargo fmt -p zelda3 --check`
- `cargo check -q -p zelda3-bin`
- a direct extraction of C function names from `hud.c` against this ledger reports no missing entries.
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 120000 --input-script scripts/inputs/file-select-enter-game-button-taps.txt --load-sram saves/sram.dat` completes with `WRAM fnv1a64 = 697132d012debe48` for the current `saves/sram.dat`.

## 2026-05-31 Player A-Press Interaction Pass

Scope: `Link_HandleAPress`, `Link_APress_PerformBasic`, and the directly
dispatched A-button interaction helpers in `src/player.c`, checked directly
against `crates/zelda3/src/player.rs`. This pass did not use the
progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleAPress` / `Link_CheckNewAPress` | `crates/zelda3/src/player.rs` | verified | Early exits, B-button gating, cached sprite pickup flag, liftable fallback, sword/item reset, boomerang cleanup, ability-mask gate, action dispatch, and continuing pull/lift/statue action switch match C. Rust bounds-checks the ability table before clearing `bitfield_for_a_button`; valid C action slots still follow the same table. |
| `Link_APress_PerformBasic` | `crates/zelda3/src/player.rs` | verified | Action slots 0..=7 dispatch to the same desert-prayer, throw, dash, grab, read, open-chest, statue-drag, and rupee-pull helpers. The out-of-range default remains an assert-style panic like C. |
| lift/throw/pull/statue/read/chest helpers | `crates/zelda3/src/player.rs` | verified | `Link_PerformThrow`, `Link_APress_LiftCarryThrow`, `Link_APress_PullObject`, `Link_PerformStatueDrag`, `Link_APress_StatueDrag`, `Link_PerformRead`, and `Link_PerformOpenChest` match the C timers, animation-step tables, pickup flag handling, liftable tile replacement, throwable terrain spawn, SRAM/item alternate checks, dialogue setup, and release cleanup paths reviewed in this cluster. |
| assert-only player states | `crates/zelda3/src/player.rs` | verified | `LinkState_0F` and `LinkState_OnIce` remain panic-only paths because the C functions are assert-only/unreachable states. |

Checks after this pass:

- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` reports `WRAM fnv1a64 = 3b99d54bddde282e`.

## 2026-05-31 SNES DMA Full-File Pass

Scope: `snes/dma.c` / `snes/dma.h`, with DMA register call sites in
`snes/snes.c`, checked directly against `crates/snes/src/dma.rs` and
`crates/snes/src/snes.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| reset/register layout | `crates/snes/src/dma.rs` | verified | `DmaChannel` reset values, `$43x0..$43xf` read/write behavior, open-bus default, mode/fixed/decrement/indirect/from-B bit packing, and unused-byte mirrors match C. |
| general DMA transfer | `crates/snes/src/dma.rs` | verified | `bAdrOffsets`, first-active-channel selection, B-bus/A-bus direction, off-index wrap, fixed/decrement address update, size decrement, per-byte/channel timer increments, busy clearing, and `$420b` activation match C. |
| HDMA init/scanline transfer | `crates/snes/src/dma.rs` | verified | `dma_initHdma` and `dma_doHdma` match C table-address setup, rep-count fetch, indirect address fetch, transfer lengths, do-transfer bit, termination, per-channel/per-byte timing, and `$420c` activation. |
| saveload ABI | `crates/snes/src/dma.rs` | verified | `save_c_saveload` / `load_c_saveload` serialize from `Dma.channel` through the end of the C struct, including ABI padding after `hdmaTimer` and `dmaBusy`, matching the C `dma_saveload` range. |

Checks after this pass:

- `cargo test -q -p snes dma`
- `cargo check -q -p snes -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.

## 2026-05-31 SNES Tracing Full-File Pass

Scope: `snes/tracing.c` / `snes/tracing.h`, checked directly against
`crates/snes/src/tracing.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| CPU trace state line | `crates/snes/src/tracing.rs` | verified | `cpu_trace_line` matches C `getProcessorStateCpu`: bank/PC, 13-character disassembly field, A/X/Y/SP/DP/DB formatting, and `E/e N/n V/v M/m X/x D/d I/i Z/z C/c` flag letters. |
| CPU disassembly | `crates/snes/src/tracing.rs` | verified | Rust uses the same 256 opcode-name table, 8/16-bit immediate override table, opcode type table, four side-effecting bus reads, word/long assembly, 8-bit and 16-bit relative target math, and byte-order argument order for the two-byte `mvp/mvn` formatter cases. |
| SPC trace state line | `crates/snes/src/tracing.rs` | verified | `spc_trace_line` matches C `getProcessorStateSpc`: PC, 17-character disassembly field, A/X/Y/SP formatting, and `N/n V/v P/p B/b H/h I/i Z/z C/c` flag letters. |
| SPC disassembly | `crates/snes/src/tracing.rs` | verified | Rust uses the same 256 SPC opcode-name table and opcode type table, reads opcode plus two operand bytes through `cpu_read` like C `apu_cpuRead`, and preserves word, relative, bit-address, and two-byte argument formatting. The fixed-size Rust arrays enforce the same 256-entry coverage at compile time. |

Checks after this pass:

- `cargo test -q -p snes tracing` reports no matching tests; this is a filter check only.

## 2026-05-31 Register Name Menu Pass

Scope: the name-registration path in `src/select_file.c`, checked directly
against `crates/zelda3/src/select_file.rs` and the RAM symbol definitions in
`crates/zelda3/src/zelda_rtl.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `FileSelect_TriggerNameStripesAndAdvance` | `crates/zelda3/src/select_file.rs` | verified | The generated 253-byte stripe upload matches C `kSelectFile_Func3_Data`: six rows, identical VRAM addresses, first/second glyphs, blank fill count, terminator, display enable, core update enable, submodule increment, and `nmi_load_bg_from_vram = 6`. |
| `Module04_NameFile` / setup steps | `crates/zelda3/src/select_file.rs` | fixed | `NameFile_EraseSave`, `Module_NamePlayer_1`, and `Module_NamePlayer_2` match the C control flow and side effects. The `selectfile_var6` clear was already behaviorally at `$00cc`, but now uses the named `SELECTFILE_VAR6` symbol so the Rust mapping is unambiguous against C. |
| `NameFile_DoTheNaming` and helpers | `crates/zelda3/src/select_file.rs` | verified | Cursor X/Y easing, wrap tables, blocked top/bottom row behavior, OAM bars, name slot cursor, glyph lookup, back/forward commands, SRAM name writes, blank-name rejection, save initialization, checksum, and return-to-file-select side effects match C. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2600 --input-script <register-name cursor movement and character-entry script>` reports `mismatched_pixels=0`.

## 2026-05-31 SNES Cart/Loader Pass

Scope: `snes/cart.c`, `snes/cart.h`, and `snes/snes_other.c`, checked directly
against `crates/snes/src/cart.rs` and `crates/snes/src/loader.rs`. This pass did
not use the progress/signature scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| cart init/reset/ROM mapping | `crates/snes/src/cart.rs` | verified | LoROM/HiROM address masks, SRAM enable checks, and ROM/RAM indexing match C, including the preserved LoROM write condition using `bank > 0xf0`. |
| ROM header scoring and loader selection | `crates/snes/src/loader.rs` | verified | `read_header` matches C `readHeader`: 21-byte printable-name mapping, speed/type/coprocessor/chips nibbles, `0x400 << n` size fields, v2/v3 maker/game-code fields, PAL test, LoROM-vs-HiROM cart-type derivation from header location, checksum/reset-vector/opcode score additions, four candidate header offsets including 512-byte copier-header slots, highest-score selection, copier-header skip, power-of-two ROM expansion by tail mirroring, and post-load hard reset. |
| cart load/save RAM handling | `crates/snes/src/cart.rs`, `crates/snes/src/loader.rs` | fixed | Rust loader no longer smooths the SRAM size to `0x2000` before `cart.load`; it now passes the exact C-derived header RAM size. The fake LoROM loader fixture was adjusted to declare SRAM so it still exercises the C-equivalent loader path. |

Checks after this pass:

- `cargo test -q -p snes cart`
- `cargo test -q -p snes loader`
- `cargo check -q -p snes -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 300` reports `WRAM fnv1a64 = 320afd6b3f615cb7`.

## 2026-05-31 Tile Detect Full-File Pass

Scope: `src/tile_detect.c`, checked directly against
`crates/zelda3/src/tile_detect.rs` and the RAM symbol definitions in
`crates/zelda3/src/zelda_rtl.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| tile attribute lookup and movement probes | `crates/zelda3/src/tile_detect.rs` | verified | `Overworld_GetTileAttributeAtLocation`, `TileDetect_Movement_Y`, `TileDetect_Movement_X`, vertical/horizontal slope probes, and `Player_TileDetectNearby` match the C offset math, masks, scratch writes, and `TileDetection_Execute` bit calls. |
| hookshot, door nudge, mirror bonk, sword doorway probes | `crates/zelda3/src/tile_detect.rs` | verified | `Hookshot_CheckTileCollision`, `Hookshot_CheckSingleLayerTileCollision`, `HandleNudgingInADoor`, `TileCheckForMirrorBonk`, and `TileDetect_SwordSwingDeepInDoor` match the C backup/restore behavior, lower-layer toggle, detection tables, and coordinate nudges. |
| `TileDetect_ResetState` | `crates/zelda3/src/tile_detect.rs` | verified | Reset list matches C, including preserving `tiledetect_diag_state`; this is required for the diagonal/house-exit behavior previously checked by lockstep. |
| `TileDetection_Execute` / `TileDetect_ExecuteInner` | `crates/zelda3/src/tile_detect.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Found a C parity mismatch for `cheatWalkThroughWalls`: C zeros the indoor tile before writing `link_tile_below` and again before behavior dispatch. Rust now uses `CHEAT_WALK_THROUGH_WALLS` at `$037f` and applies the same zeroing in both locations. All reviewed tile behavior cases match the C side effects, bit shifts, table lookups, and indoor/outdoor branches after this fix. |

Checks after this pass:

- `cargo check -q -p zelda3 -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 32613 --input-script scripts/inputs/tas-us-rta-ace.txt` reports `lockstep completed 32613 frame(s) from frame 0; WRAM fnv1a64 = f01523b0bba49471`.

## 2026-05-31 SNES Input Full-File Pass

Scope: `snes/input.c` / `snes/input.h`, with the call sites in `snes/snes.c`
checked directly against `crates/snes/src/input.rs` and
`crates/snes/src/snes.rs`. This pass did not use the progress/signature
scripts.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `input_init` / `input_reset` | `crates/snes/src/input.rs` | verified | `InputState::default/new` matches C `type = 1`, `currentState = 0`, and reset clears only `latchLine`/`latchedState`. |
| `input_cycle` / `input_read` | `crates/snes/src/input.rs` | verified | Latch-high copies `current_state` to `latched_state`; reads return bit 0, shift right, and fill bit 15 with `1`, matching the C serial controller protocol. |
| SNES call sites | `crates/snes/src/snes.rs` | verified | `$4016/$4017` reads preserve the same open-bus masks, `$4016` writes update both latch lines from bit 0, and auto joypad read latches both inputs then shifts 16 bits into the four port words exactly like `snes_doAutoJoypad`. |

Checks after this pass:

- `cargo test -q -p snes input`

## 2026-05-30 Sprite Wrapper Stub/No-op Pass

Scope: empty-body scan follow-up in sprite/world/NPC and nearby intentional
no-op handlers, checked directly against `src/sprite_main.c`, `src/load_gfx.c`,
`src/overworld.c`, `src/dungeon.c`, and `src/player.c`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PaletteFilter_SP5F` / `PaletteFilter_RestoreSP5F` call sites | `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/load_gfx.rs` | fixed | The Flute Kid vanish path had local empty wrapper stubs even though the canonical palette ports already existed in `load_gfx.rs`. The wrappers now delegate to those ports so the C palette filtering/restoration side effects run. |
| `Sprite_SpawnSparkleGarnish` call sites | `crates/zelda3/src/sprite_main_npcs.rs`, `crates/zelda3/src/sprite_main_prep.rs` | fixed | Bee/Good Bee handlers had a local empty wrapper even though `sprite_spawn_sparkle_garnish` already matches the C garnish allocation/random-offset/countdown body. The wrapper now delegates to the canonical port. |
| intentional empty handlers | `crates/zelda3/src/player.rs`, `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/overworld.rs` | verified | Confirmed C bodies are empty for `PlayerHandler_15_HoldItem`, `Dung_TagRoutine_0x00`, `Dung_TagRoutine_0x1B`, `LayerEffect_Nothing`, and `Overworld_WeathervaneExplosion`; the Rust empty bodies are correct no-ops. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` reports `mismatched_pixels=0`.

## 2026-05-30 Sprite Prep No-op Classification Pass

Scope: empty sprite prep handlers and one Hinox helper from `src/sprite_main.c`,
checked directly against `crates/zelda3/src/sprite_main_prep.rs` and
`crates/zelda3/src/sprite_main_hinox_shop.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| early prep no-ops | `crates/zelda3/src/sprite_main_prep.rs` | verified | Confirmed C-empty bodies for `SpritePrep_ThrowableScenery`, `SpritePrep_SwitchFacingUp`, `SpritePrep_DoNothingA`, and `SpritePrep_DoNothingC`; Rust empty bodies are correct. |
| mid/late prep no-ops | `crates/zelda3/src/sprite_main_prep.rs` | verified | Confirmed C-empty bodies for `SpritePrep_DoNothingD`, `SpritePrep_Zazakku`, `SpritePrep_ShieldPickup`, `SpritePrep_DoNothingG`, `SpritePrep_DoNothingH`, and `SpritePrep_FakeSword`; Rust empty bodies are correct. |
| `Hinox_ThrowBomb` | `crates/zelda3/src/sprite_main_hinox_shop.rs` | verified | Confirmed the C function body is empty; Rust keeps it as an explicit no-op and already has a parity assertion for unchanged RAM. |

Checks after this pass:

- `git diff --check`

## 2026-05-30 Flute Boy Draw Wrapper Pass

Scope: `src/sprite_main.c` `FluteKid_Human` / `FluteBoy_Draw`, checked directly
against `crates/zelda3/src/sprite_main_world.rs` and
`crates/zelda3/src/sprite_main_draw.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `FluteBoy_Draw` wrapper | `crates/zelda3/src/sprite_main_world.rs` | fixed | `flute_boy_draw_for_world` returned `0` even though the canonical `flute_boy_draw` port exists. The wrapper now delegates to that port, restoring C-equivalent OAM allocation/draw and the offscreen return used by `FluteKid_Human` to gate ambient sound. |
| `FluteBoy_Draw` body | `crates/zelda3/src/sprite_main_draw.rs` | verified | Existing canonical body matches the C four-entry draw table, region-B OAM allocation, `sprite_D * 8 + sprite_graphics * 4` selection, `Sprite_DrawMultiple`, and `(info.x \| info.y) >> 8` return contract. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` reports `mismatched_pixels=0`.

## 2026-05-30 Smithy Cluster Classification Pass

Scope: `src/sprite_main.c` Smithy cluster, checked directly against
`crates/zelda3/src/sprite_main_dungeon_npcs.rs`,
`crates/zelda3/src/sprite_main_draw.rs`, and the stale skip notes in
`crates/zelda3/src/sprite_main_npcs.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_1A_Smithy` dispatch | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_draw.rs` | verified | Sprite type `0x1a` dispatches through the draw/main table to `sprite_1_a_smithy`, which matches C subtype dispatch for main, spark, frog, and homecoming. |
| Smithy state machines | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | verified | `Smithy_Homecoming`, `Smithy_Frog`, `Smithy_Main`, `Smithy_ListenForHammer`, `Smithy_SpawnDwarfPal`, `Smithy_Spark`, `Smithy_SpawnSpark`, and the dumb barrier spawn helper match the C control flow, state writes, message IDs, follower/sword/progress mutations, spawn coordinates, and animation tables reviewed in this pass. |
| Smithy draw helpers | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_draw.rs` | verified | `ReturningSmithy_Draw`, `SmithyFrog_Draw`, `Smithy_Draw`, and `SmithySpark_Draw` use the C draw tables, DMA byte, `Sprite_DrawMultiplePlayerDeferred`, and `SpriteDraw_Shadow` equivalent (`sprite_draw_shadow_custom(..., 10)`). Removed stale `sprite_main_npcs.rs` skip notes that still claimed these helpers were missing. |

Checks after this pass:

- `git diff --check`

## 2026-05-30 Dispatch No-op Classification Pass

Scope: empty dispatch targets in `src/nmi.c`, `src/ancilla.c`, and
`src/ending.c`, checked directly against `crates/zelda3/src/nmi.rs`,
`crates/zelda3/src/ancilla.rs`, and `crates/zelda3/src/ending.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| NMI empty tilemap slots | `crates/zelda3/src/nmi.rs` | verified | Confirmed C-empty bodies for `NMI_UploadTilemap_doNothing` and `NMI_TileMapNothing`; Rust keeps both as explicit no-op dispatch targets. |
| `Ancilla_Empty` | `crates/zelda3/src/ancilla.rs` | verified | Confirmed the C function body is empty and used as an ancilla dispatch table slot; Rust `ancilla_empty` is the correct no-op. |
| `EXIT_0CCA90` | `crates/zelda3/src/ending.rs` | verified | Confirmed the C function body is empty and is selected by intro/ending scene sprite dispatch; Rust `exit_0_cca90` is the correct no-op. |

Checks after this pass:

- `git diff --check`

## 2026-05-30 Load GFX Dead Shim Cleanup Pass

Scope: `src/load_gfx.c` `Palette_LoadSingle` and `LoadSpriteGraphics`, checked
directly against `crates/zelda3/src/load_gfx.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Palette_LoadSingle` | `crates/zelda3/src/load_gfx.rs` | verified | The real Rust port is the `ZeldaState::palette_load_single` method and all local call sites use that method. Removed the unused top-level C-signature placeholder so the scan no longer reports a false empty stub. |
| `LoadSpriteGraphics` | `crates/zelda3/src/load_gfx.rs` | verified | The real Rust port is the `ZeldaState::load_sprite_graphics` method and all local call sites use that method. Removed the unused top-level C-signature placeholder so the scan no longer reports a false empty stub. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Config Dead Shim Cleanup Pass

Scope: `src/config.c` parser helpers, checked directly against
`crates/zelda3/src/config.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| key/gamepad array parsing | `crates/zelda3/src/config.rs` | verified | The real Rust ports are `ConfigContext::parse_key_array` and `ConfigContext::parse_gamepad_array`; removed unused top-level C-signature no-op shims that were not called by the runtime or tests. |
| config file parser entry | `crates/zelda3/src/config.rs`, `crates/zelda3/src/main.rs` | verified | Runtime config loading uses `parse_config_file_context` / `ConfigContext::parse_config_file`, matching the C parser flow with explicit context instead of globals. Removed unused top-level no-op shims for `handle_ini_config`, `parse_one_config_file`, and `parse_config_file`. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Sprite Shadow Dead Shim Cleanup Pass

Scope: `src/sprite.c` `SpriteDraw_Shadow` / `SpriteDraw_Shadow_custom`, checked
directly against `crates/zelda3/src/sprite.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpriteDraw_Shadow_custom` | `crates/zelda3/src/sprite.rs` | verified | The real Rust port is `ZeldaState::sprite_draw_shadow_custom`, matching the C pause/state guards, Y offset/subtraction, OAM slot calculation, `sprite_flags3 & 0x20` branch, char/flags/ext writes, and call sites. |
| `SpriteDraw_Shadow` wrapper | `crates/zelda3/src/sprite.rs` | verified | Rust call sites either use `sprite_draw_shadow_custom(..., 10)` for the C wrapper or the simplified `sprite_draw_shadow` helper where the C call passes only the prepped X. Removed the unused top-level pointer-signature placeholder. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Util Dead Shim Cleanup Pass

Scope: utility compatibility shims in `crates/zelda3/src/util.rs`, checked
against `../zelda3/src`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `str_set` shim | `crates/zelda3/src/util.rs` | verified | No matching C symbol exists in the source tree and the Rust shim had no callers. Removed it so the placeholder scan no longer reports a non-parity empty function. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 SPC Debug Player Shim Cleanup Pass

Scope: `src/spc_player.c` `RunAudioPlayer`, checked directly against
`crates/zelda3/src/spc_player.rs` and the playable Rust audio path in
`crates/zelda3/src/audio.rs` / `zelda3-bin/src/main.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `RunAudioPlayer` | `crates/zelda3/src/spc_player.rs` | verified | The C body is inside `WITH_SPC_PLAYER_DEBUGGING` and is a standalone SDL/lightworld.spc debug loop, not the game runtime path. Rust runtime audio goes through `AudioState`, `spc_player_generate_samples`, and frontend `push_audio`; removed the unused public no-op shim. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Main Host Dead Shim Cleanup Pass

Scope: `src/main.c` SDL host helpers, checked directly against
`crates/zelda3/src/main.rs`, `crates/platform/src/lib.rs`, and
`zelda3-bin/src/main.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| unused SDL host shims | `crates/zelda3/src/main.rs` | verified | Removed unused Rust no-op shims for `sdl_renderer_destroy`, `zelda_apu_lock`, `zelda_apu_unlock`, and `open_one_gamepad`. The C functions perform SDL cleanup/mutex/gamepad work, but the playable Rust host uses `NativeFrontend` and the Rust audio queue instead of those SDL paths. |
| retained C-shaped draw hook | `crates/zelda3/src/main.rs` | verified | Kept `sdl_renderer_end_draw` because the legacy `zelda_main` draw wrapper still calls it after filling the in-memory video buffer; presentation happens in the native host, so the empty hook is intentional in this compatibility layer. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Cucco Helper Comment Cleanup Pass

Scope: top-level Cucco helper comments in `crates/zelda3/src/sprite_main_draw.rs`,
checked directly against the local implementations and the C Cucco flow in
`src/sprite_main.c`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| Cucco local helper duplicates | `crates/zelda3/src/sprite_main_draw.rs` | verified | `chicken_incr_subtype2_for_draw` and `bawk_bawk_for_draw` are implemented local duplicates used by the top-level Cucco actor. Renamed stale comments from "Deferred duplicate" to "Local duplicate" so placeholder scans no longer report them as deferred work. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Deferred-Draw Shadow Wrapper Pass

Scope: C draw helpers that call `Sprite_DrawMultiplePlayerDeferred` followed by
`SpriteDraw_Shadow`, checked directly against `src/sprite_main.c`,
`crates/zelda3/src/sprite_main_npcs.rs`,
`crates/zelda3/src/sprite_main_dungeon_npcs.rs`, and
`crates/zelda3/src/sprite_main_draw.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `BottleVendor_Draw` | `crates/zelda3/src/sprite_main_npcs.rs` | fixed | Draw table and deferred draw selection already matched C, but the shadow call used the simplified X-only helper. It now calls `sprite_draw_shadow_custom(..., 10)`, matching C `SpriteDraw_Shadow` mutation of `info.y` before the `(info.x | info.y) >> 8` return. |
| `Bully_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Draw table and base calculation already matched C. The shadow call now uses `sprite_draw_shadow_custom(..., 10)` instead of the simplified helper, preserving C's `sprite_flags3 & 0x20` branch and state/pause guards. |
| `Priest_Draw` / `FakeSword_Draw` | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_draw.rs` | verified | `Priest_Draw` already used the exact custom shadow wrapper. `FakeSword_Draw` has no shadow call in C and the Rust deferred draw call matches the two-entry table path. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Shared Shadow Helper Branch Pass

Scope: `src/sprite.c` `SpriteDraw_Shadow_custom`, checked directly against the
shared simplified Rust helper in `crates/zelda3/src/sprite.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| X-only shadow helper | `crates/zelda3/src/sprite.rs` | fixed | Existing X-only callers still use `sprite_draw_shadow` when they do not need `PrepOamCoordsRet` mutation. The helper now matches C's pause/state guard and `sprite_flags3 & 0x20` alternate shadow branch instead of always drawing `0x6c`; flags continue to come from the same `sprite_oam_flags ^ sprite_obj_prio` source used by prepared draw info. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Direct-Draw Shadow Wrapper Pass

Scope: direct/manual draw helpers that call C `SpriteDraw_Shadow`, checked
directly against `src/sprite_main.c` and `crates/zelda3/src/sprite_main_draw.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `WalkingZora_Draw` / `Crab_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Both functions draw manually from a prepped coordinate and C passes the mutable `PrepOamCoordsRet` into `SpriteDraw_Shadow`. Rust now reconstructs/passes the full info struct to `sprite_draw_shadow_custom(..., 10)` instead of using only X. |
| `RedBari_Draw` / `HardHatBeetle_Draw` / `Armos_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | These functions already carried full draw info after `Sprite_DrawMultiple`; their shadow calls now use the exact custom wrapper, preserving C's info mutation and alternate shadow branch. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`

## 2026-05-30 Multi-Draw Shadow Wrapper Pass

Scope: additional `Sprite_DrawMultiple` draw helpers that call C
`SpriteDraw_Shadow(k, &info)`, checked directly against `src/sprite_main.c`,
`crates/zelda3/src/sprite_main_draw.rs`, and
`crates/zelda3/src/sprite_main_hinox_shop.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ropa_Draw` / `Zazak_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Draw tables, body selection, pause/head patch behavior, and OAM char/flag writes already matched C. Their shadow calls now pass the mutable draw info to `sprite_draw_shadow_custom(..., 10)`, matching C `SpriteDraw_Shadow(k, &info)` instead of using only X. |
| `Pengator_Draw` / `Vulture_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Multi-tile body/extra-foot draw selection already matched C. The final shadow calls now use the exact custom wrapper, preserving C's `info.y` mutation, pause/state guard, and alternate shadow branch. |
| `Hinox_Draw` | `crates/zelda3/src/sprite_main_hinox_shop.rs` | fixed | The indexed draw-table count/offset path already matched C. Replaced the X-only shadow call with `sprite_draw_shadow_custom(..., 10)`, matching C `SpriteDraw_Shadow(k, &info)`. |
| remaining full-`info` sprite draw helpers | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Converted the remaining draw helpers that held a mutable `PrepOamCoordsRet` but still called the X-only shadow helper: `Bomber_Draw`, `Stalfos_Draw`, `Stal_Draw`, `Gibdo_Draw`, `FlyingTile_Draw`, `Lady_Draw`, `YoungSnitchLady_Draw`, `SnapDragon_Draw`, `Lynel_Draw`, `Goriya_Draw`, `RunningMan_Draw`, `Elder_Draw`, `Shopkeeper_Draw`, `FluteBoyFather_Draw`, `InnKeeper_Draw`, `MiddleAgedMan_Draw`, `BlindHideoutGuy_Draw`, `SweepingLady_Draw`, `MazeGameGuy_Draw`, `DrinkingGuy_Draw`, `DiggingGameGuy_Draw`, `BombShopEntity_Draw`, `StoryTeller_1_Draw`, `SmithyFrog_Draw`, `QuarrelBros_Draw`, `BigFaerie_Draw`, `TroughBoy_Draw`, `Moblin_Draw`, `Tektite_Draw`, and `ArcheryGameGuy_Draw`. |
| `Pikit_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | C reuses `PrepOamCoordsRet info` for tongue draw, body draw, `SpriteDraw_Shadow`, and loot draw. Rust has a file-local Pikit info copy for tongue/loot helpers, so the body/shadow path now mirrors the same coordinates in a canonical sprite prep struct and calls `sprite_draw_shadow_custom(..., 10)` instead of the X-only helper. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.

## 2026-05-30 Manual-OAM Shadow Wrapper Pass

Scope: remaining manual OAM draw helpers that prepare coordinates into
individual `info_x`/`info_y`/`info_flags` locals but call C
`SpriteDraw_Shadow(k, &info)`, checked directly against `src/sprite_main.c` and
`crates/zelda3/src/sprite_main_draw.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `BuzzBlob_Draw` / `Hokbok_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Manual OAM writes matched the C tables and loops. Rust now reconstructs the full prep info and calls `sprite_draw_shadow_custom(..., 10)`, matching C's final `SpriteDraw_Shadow(k, &info)` instead of passing only X. |
| `Recruit_Draw` / `Octoballoon_Draw` | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Manual body/head OAM writes and Octoballoon state-6 baby spawn gate matched C. Their shadow calls now use the full reconstructed prep info so the C state guard, Y mutation, and alternate shadow branch are preserved. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.

## 2026-05-30 Assert-Only Dispatch Classification Pass

Scope: Rust panic paths that still look like stubs in text scans, checked
directly against C `assert(0)` or C `Not_Implemented()` bodies.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| SPC `Not_Implemented` fallback | `crates/zelda3/src/spc_player.rs` | verified | C `Not_Implemented()` is an `assert(0)` helper used by unsupported music effects, tremolo helpers, and unreachable new-music/SFX fallback paths. Rust's `not_implemented()` remains a panic to preserve that C assert behavior; added an explicit comment so it no longer reads as an accidental Rust-only stub. |
| `Ancilla_Unused_14` / `Ancilla_Unused_25` | `crates/zelda3/src/ancilla.rs` | verified | C dispatch slots assert immediately. Rust panics are the matching assert behavior; added comments documenting the classification. |
| `Overworld_Func1D` / `Overworld_Func1E` | `crates/zelda3/src/overworld.rs` | verified | C module dispatch slots are `assert(0)`. Rust panics are the matching assert behavior; added comments documenting the classification. |
| `LinkState_0F`, `LinkState_OnIce`, `Link_APress_PerformBasic` default | `crates/zelda3/src/player.rs` | verified | `LinkState_0F` and `LinkState_OnIce` were already commented as C assert-only states. Confirmed `Link_APress_PerformBasic` C asserts outside action slots `0..=7`; added a matching Rust comment on the default panic. |
| follower, overlord, and entrance-door asserts | `crates/zelda3/src/tagalong.rs`, `crates/zelda3/src/overlord.rs`, `crates/zelda3/src/dungeon.rs` | verified | Confirmed `Follower_CheckGameMode` follower types `5/14`, `Follower_OldMan`'s undefined-X recoil edge, unused `Overlord_StalfosFactory`, and all four `Door_*_EntranceDoor` helpers are C `assert(0)` paths. Rust panics are the matching assert behavior; added comments documenting the classifications. |
| messaging/map invalid states | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/hud.rs` | verified | Confirmed C asserts for `Module_Messaging_0`, `Module_Messaging_6`, invalid `Module0E_0A_FluteMenu` states, invalid `Module0E_03_01_DrawMap` init states, and invalid `Hud_Module_Run` states. Rust panics are matching assert/bounds guard behavior; added comments documenting the classification. |
| overworld transition/event assert paths | `crates/zelda3/src/overworld.rs` | verified | Confirmed C asserts for invalid overworld transition directions in `OverworldTransitionScrollAndLoadMap`, invalid initial screen-scroll directions in `CreateInitialNewScreenMapToScroll`, and invalid event-overlay screens `114..=118`, `122`, `124..=127`. Rust panics are matching assert behavior; added comments documenting the classification. |
| sprite assert-only slots | `crates/zelda3/src/sprite_main_draw.rs` | fixed | Confirmed `Sprite_B8_DialogueTester`, `Sprite_2D_TelepathicTile`, and non-cannonball `Sprite_6B_CannonTrooper` are C `assert(0)` paths. Also fixed `Sprite_B9_BullyAndPinkBall` so invalid subtypes now panic like C's default `assert(0)` instead of silently doing nothing. |
| debug-only assert defaults | `crates/zelda3/src/player.rs`, `crates/zelda3/src/misc.rs`, `crates/zelda3/src/dungeon.rs` | verified | Confirmed `Link_HandleYItem` invalid item slots, `Module_Unknown0`, `Module_Unknown1`, and `Dungeon_LoadAttribute_Selectable` invalid states are C `assert(0)` defaults. Rust `debug_assert!(false)` paths are intentional debug-only stop points; added comments documenting the classification. |
| defensive direct-table bounds guards | `crates/zelda3/src/hud.rs`, `crates/zelda3/src/misc.rs`, `crates/zelda3/src/spc_player.rs` | verified-limited | C directly indexes `kMessagingSubmodules`, `kHudItemBoxGfxPtrs`, Link DMA source tables, and `kEffectByteLength`; Rust keeps explicit panic guards for corrupt/out-of-range translated indexes so invalid data fails loudly instead of reading past the safe slice. |
| infrastructure fatal paths | `crates/snes/src/cpu_step.rs`, `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/zelda_cpu_infra.rs`, `crates/zelda3/src/main.rs` | verified | Confirmed unknown CPU BRKs, unknown replay commands, unexpected oracle RTS hooks, and `Die`/asset-loader fatal paths all map to C `assert(0)` or `Die(...)` stops. Added comments documenting why these remain fatal rather than normal gameplay implementations. |
| config map capacity/duplicate edge behavior | `crates/zelda3/src/config.rs` | fixed | C `KeyMapHash_Add` checks the key limit only on 256-entry allocation boundaries and increments the backing array before detecting duplicate keys; Rust now mirrors that shape. C `GamepadMap_Add` similarly checks the joypad limit only on its allocation boundary; Rust now mirrors that threshold instead of checking before every insert. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `cargo test -p zelda3 config`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.

## 2026-05-30 Dungeon Map Outer State/Recovery Pass

Scope: `src/messaging.c` dungeon-map outer submodules and initialization states,
checked directly against `crates/zelda3/src/messaging.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| dungeon-map init dispatch | `crates/zelda3/src/messaging.rs` | fixed | `Module0E_03_01_DrawMap` now dispatches by `dungmap_init_state` like C instead of calling prep once and advancing `overworld_map_state`. This connects the existing room/layout/marker ports to the actual multi-frame map initialization flow. |
| prep/LEVEL/backdrop/room init states | `crates/zelda3/src/messaging.rs` | fixed | Ported `Module0E_03_01_00_PrepMapGraphics`, `01_DrawLEVEL`, `02_DrawFloorsBackdrop`, and `03_DrawRooms`: tile-theme backups, HDMA disable/restore, tilemap erase, tileset/palette/HUD setup, VRAM upload packet writers, floor backdrop box list, current-floor initialization, messaging-buffer room draws, scratch words, NMI target, and init-state increments now match C. |
| dungeon-map fade/backup/scroll helpers | `crates/zelda3/src/messaging.rs` | fixed | `DungMap_Backup`, `DungMap_LightenUpMap`, `DungMap_4`, `DungMap_FadeMapToBlack`, and `DungMap_RestoreOld` now match C brightness gates, force blank, palette/scroll/register backups, messaging-buffer clear, `dungmap_var4` scroll delta, `dungmap_var5` subtraction, lamp cone orientation, module restore, and HDMA restore. |
| dungeon-map graphics recovery | `crates/zelda3/src/messaging.rs` | fixed | `DungeonMap_RecoverGFX` now follows C recovery instead of the generic default-graphics shortcut: erase normal tilemaps, restore backed-up tileset/TM/TS state, rebuild HUD, rebuild/upload room quadrants via the existing immediate tilemap uploader, restore palette/COLDATA, recover peg graphics, queue SFX/music, and clear NMI core updates. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200` completes without a panic.

## 2026-05-30 Flute Menu Travel/World Map OAM Pass

Scope: `src/messaging.c` flute travel menu and shared overworld-map OAM helpers,
checked directly against `crates/zelda3/src/messaging.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module0E_0A_FluteMenu` state flow | `crates/zelda3/src/messaging.rs` | fixed | The top-level state dispatch already matched C, but the state bodies were placeholders. `FluteMenu_HandleSelection`, `FluteMenu_LoadSelectedScreen`, `Overworld_LoadOverlayAndMap`, and `FluteMenu_FadeInAndQuack` are now ported instead of just advancing `overworld_map_state`. |
| `FluteMenu_HandleSelection` | `crates/zelda3/src/messaging.rs` | fixed | Restored C timer gating, cancel-bird-travel feature handling, filtered d-pad selection wrap, selection SFX, current-location marker OAM, eight bird destination coordinate table writes, selected marker animation, and link special-exit coordinate restore. |
| `FluteMenu_LoadSelectedScreen` | `crates/zelda3/src/messaging.rs` | fixed | Restored save-event/dungeon-bit clears, optional transport load based on cancel state, palette load, animated overworld tile decompression selector, fixed color/scroll setup, tileset init, overlay load, submodule decrement, SFX, ambient, and music-control writes. |
| map transition/fade helpers | `crates/zelda3/src/messaging.rs` | fixed | `Overworld_LoadOverlayAndMap` now preserves `WORD(main_module_index)` and advances `WORD(overworld_map_state)` after `Overworld_LoadAndBuildScreen`, matching C. `FluteMenu_FadeInAndQuack` now increments brightness, calls `BirdTravel_Finish_Doit` at 15, and otherwise runs sprites. |
| `WorldMap_CalculateOamCoordinates` / `WorldMap_AddSprite` | `crates/zelda3/src/messaging.rs` | fixed | Replaced the stub-style coordinate predicate with the C coordinate calculation wrapper, including the extend-screen dark-map bounds check. `WorldMap_AddSprite` now matches C's `ch == 100` table substitution, `x/y -= 4` behavior, extend-screen high-X bit, and `SetOamPlain` write contract. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` reports `mismatched_pixels=0`.

## 2026-05-30 World Map Control/Exit Width Pass

Scope: `src/messaging.c` overworld-map player control, restore, exit, and HDMA
setup helpers, checked directly against `crates/zelda3/src/messaging.rs` and
`crates/zelda3/src/load_gfx.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| map button and player-control loop | `crates/zelda3/src/messaging.rs` | verified | `DidPressButtonForMap` and `WorldMap_PlayerControl` match the C select/X gate, zoom cooldown byte, zoom SFX, map-flag toggle, timer table, link-centered mode7 offsets, directional scroll tables, and final sprite handling call. |
| map restore/exit helpers | `crates/zelda3/src/messaging.rs` | verified | `WorldMap_RestoreGraphics`, `Attract_SetUpConclusionHDMA`, and `WorldMap_ExitMap` match the C fade gate, force blank, palette copy, backup register restores, HDMA setup, module/submodule restoration, VRAM upload reset, ambient/SFX/music writes, and byte-vs-word writes. |
| world-map HDMA setup | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/load_gfx.rs` | verified | `OverworldMap_SetupHdma` and `WorldMap_SetUpHDMA` match the C table address selection, HDMA parameters, initial mode7 register setup, map flag/timer branches, and submodule 10 special case. |
| flute selected-screen palette mode width | `crates/zelda3/src/messaging.rs` | fixed | Corrected `FluteMenu_LoadSelectedScreen` to clear `overworld_palette_aux_or_main` as a 16-bit variable, matching C `overworld_palette_aux_or_main = 0`; the previous Rust write cleared only the low byte. |

Checks after this pass:

- `cargo fmt -p zelda3`
- `cargo check -p zelda3-bin`
- `git diff --check`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` reports `mismatched_pixels=0`.

## 2026-05-30 snes9x Oracle Harness Pass

Scope: playable host parity tooling, checked against a local snes9x libretro core
at `/private/tmp/snes9x_libretro/snes9x_libretro.dylib` and the source ROM at
`/path/to/zelda3.sfc`.

| Area | Rust location | Verdict | Notes |
|---|---:|---|---|
| native host dependency surface | `crates/platform`, `zelda3-bin/src/main.rs` | verified | Confirmed the playable frontend is `winit` + `pixels` + `cpal`; no `sdl2`/`rust-sdl2` dependency remains in the workspace. Updated the stale binary comment from SDL host to native host. |
| native host pixel presentation | `crates/platform/src/lib.rs` | fixed | Fixed the `pixels` presentation copy to convert the PPU renderer's C-shaped little-endian `0x00RRGGBB` bytes (`BB GG RR 00`) into RGBA (`RR GG BB FF`). The dump-frame path already performed this conversion, which is why headless PNGs were visible while the live window could remain black/transparent. |
| snes9x libretro input wiring | `zelda3-bin/src/main.rs` | fixed | Added libretro joypad state injection using the same SNES button bit layout as the Rust input path, so scripted frames can drive both Rust and snes9x. |
| snes9x strict first-diff command | `zelda3-bin/src/main.rs` | fixed | Added `--compare-snes9x-oracle <core> <rom> [frames] [--input-script <path>] [--ignore-video] [--ignore-audio]`. It runs Rust and snes9x side-by-side, compares snes9x video RGB after pixel-format conversion, renders Rust audio at snes9x' captured per-frame sample count, and exits on the first enabled video/audio divergence with frame/input/port/audio summaries. |
| startup snes9x parity | `zelda3-bin/src/main.rs`, startup playable path | unverified | Current evidence contradicts startup parity: video diverges at frame 0 because Rust has 253 non-black Nintendo Presents pixels while snes9x is still black; audio diverges at frame 3 with Rust peak 2969 from the `$0a` SFX path while snes9x remains silent. This supports the current working theory that the playable Rust entry path is ahead of real ROM reset timing for startup audio/video. |
| startup snes9x video offset | `zelda3-bin/src/main.rs`, startup playable path | verified | Added `--skip-snes9x-frames <n>` to warm up snes9x before comparison. With `--skip-snes9x-frames 85 --ignore-audio`, the first 10 startup video frames match exactly, so startup video is aligned by a fixed 85-frame real-ROM boot offset. With the same offset and `--ignore-video`, audio still diverges at compared frame 0, so startup audio remains a separate APU timing/latency mismatch. |
| playable frame wrapper | `zelda3-bin/src/main.rs`, `crates/zelda3/src/zelda_rtl.rs`, `../zelda3/src/main.c`, `../zelda3/src/zelda_rtl.c` | fixed | The default playable host and snes9x compare now step via `ZeldaState::zelda_run_frame`, matching C `main.c`'s `ZeldaRunFrame(inputs)` call. The prior helper called `run_frame_internal` plus `zelda_push_apu_state`, which is the oracle-internal path and skipped the C wrapper's APUI00/music-playing mirror update. |
| startup Rust audio lead probe | `zelda3-bin/src/main.rs` | unverified | Added `--lead-rust-audio-blocks <n>` to test whether the startup audio mismatch is a fixed high-level audio latency. Probes with 0..4 lead blocks at snes9x skip 85 still diverge: 0..2 blocks leave Rust silent where snes9x has samples; 3..4 blocks overshoot and produce different Rust samples at the block start. This points away from a simple host queue delay and toward real-ROM APU command timing or SFX interpreter state. |
| raw ROM APU trace trigger | `zelda3-bin/src/main.rs` | fixed | `--trace-rom-apu-upload` now treats APUI input-port changes as interesting only once the SPC player has left the IPL/upload phase, avoiding per-byte upload spam while still allowing post-bootstrap command transitions to be captured. A 1,600,000-opcode trace confirms bootstrap readiness at opcode 1,438,619 and no post-bootstrap command in the standalone raw-opcode runner by opcode 1,600,000; this is useful bootstrap evidence but not yet a frame-level snes9x command-timing proof. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3-bin`
- `cargo run -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 5` exits with the expected frame-0 video divergence.
- `cargo run -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 120 --ignore-video` exits with the expected frame-3 audio divergence.
- `cargo run -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 10 --skip-snes9x-frames 85 --ignore-audio` passes.
- `cargo run -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 20 --skip-snes9x-frames 85 --ignore-video` exits with the expected compared-frame-0 audio divergence.
- `target/debug/zelda3 --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 4 --skip-snes9x-frames 85 --ignore-video --lead-rust-audio-blocks <0..4>` confirms the audio mismatch is not corrected by a small fixed Rust audio lead.
- `cargo run -p zelda3-bin -- --trace-rom-apu-upload /path/to/zelda3.sfc 1600000 0.286` confirms the narrowed raw trace reaches bootstrap-ready state without flooding on upload byte writes.

## 2026-05-30 HUD/Menu Renderer Pass

Scope: `src/hud.c` HUD/menu renderer and the messaging Y-item icon path,
checked directly against `crates/zelda3/src/hud.rs` and
`crates/zelda3/src/messaging.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| HUD item search/navigation/reorder | `crates/zelda3/src/hud.rs` | verified | `Hud_GetItemPosition`, `Hud_GotoPrevItem`, `Hud_GotoNextItem`, equip up/down/left/right, `Hud_NormalMenu`, `Hud_HandleItemSwitchInputs`, and `Hud_ReorderItem` match the C item loops, wrap bounds, order initialization length, SFX writes, and HUD redraw calls. |
| HUD box/icon/menu drawing helpers | `crates/zelda3/src/hud.rs` | fixed | `Hud_GetIconForItem` now indexes the same zero-based `kHudItemBoxGfxPtrs[i - 1][item_val]` table shape without Rust-only clamping; the C zero-initialized trailing armor entries are represented explicitly. `Hud_DrawYButtonItems`, ability/progress/equipment/bottle boxes, flashing circle, `Hud_DrawItem`, `Hud_DrawNxN`, and `Hud_Copy2x2` were compared for tile destinations and palette bits. |
| HUD refill/inventory counters | `crates/zelda3/src/hud.rs` | fixed | Bomb/arrow refill and inventory display now index `kMaxBombsForLevel[link_bomb_upgrades]` and `kMaxArrowsForLevel[link_arrow_upgrades]` directly like C instead of masking upgrades with `& 7`. Rupee/key/magic/heart update loops match C byte-vs-word writes and digit placement. |
| messaging Y-item icon preview | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/hud.rs` | fixed | `RenderText_DrawSelectedYItem` now matches C: `choice_in_multiselect_box` is a zero-based table index for `Hud_GetItemBoxPtr`, then the variant offset comes from `(&link_item_bow)[item]` except bombs/item 32. The previous Rust path incorrectly reused one-based `hud_get_icon_for_item`. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3-bin`
- `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` reports `mismatched_pixels=0`.

## 2026-05-30 Messaging Multiselect/Y-Item Pass

Scope: `src/messaging.c` multiselect message handlers around
`RenderText_Draw_ChooseItem`, `RenderText_FindYItem_Previous`,
`RenderText_FindYItem_Next`, `RenderText_DrawSelectedYItem`,
`RenderText_Draw_Choose2LowOr3`, `RenderText_Draw_Choose2HiOr3`,
`RenderText_Draw_Choose3`, and `RenderText_Draw_Choose1Or2`, checked directly
against `crates/zelda3/src/messaging.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| Y-item choice message state machine | `crates/zelda3/src/messaging.rs` | verified | `RenderText_Draw_ChooseItem`, previous/next search, accept/cancel gates, item-index wrapping, `RenderText_Refresh` calls, and selected item tile writes match C. This depends on the fixed `Hud_GetItemBoxPtr` table semantics from the HUD/Menu Renderer pass. |
| two- and three-choice message handlers | `crates/zelda3/src/messaging.rs` | verified | `RenderText_Draw_Choose2LowOr3`, `RenderText_Draw_Choose2HiOr3`, `RenderText_Draw_Choose3`, and `RenderText_Draw_Choose1Or2` match C countdown handling, sound effects, input masks, choice wrapping, dialogue-message index writes, and VWF reinitialization. |

Checks after this pass:

- `cargo check -p zelda3-bin`

## 2026-05-29 Dungeon Torch/Door Runtime Pass

Scope: remaining real dungeon gameplay stub in `src/dungeon.c`, checked directly
against `Dungeon_ProcessTorchesAndDoors`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_ProcessTorchesAndDoors` | `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Ported torch timer expiry, front-facing door checks, breakable wall dash debris, big-key/small-key door opening, invisible eye-watch doors, B-button curtain/cuttable-tile handling, overlay DMA, attr writes, and sound effects. Corrected byte-vs-word handling for `byte_7E0333`; exposed the existing door-debris ancilla allocator so the C breakable-wall path can be called from dungeon code. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Player Item Side-Effect Pass

Scope: partial player item paths in `src/player.c`, checked directly against
`LinkItem_Lamp`, `LinkItem_Powder`, `LinkItem_Shovel`, and `LinkItem_Flute`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkItem_Lamp` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Restored the C-shaped lamp side effects by calling `AncillaAdd_MagicPowder`, `Dungeon_LightTorch`, and `AncillaAdd_LampFlame` instead of generic ancilla allocation. The existing C-shaped powder/flame ancilla constructors are now visible to the player item path. |
| `LinkItem_Powder` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Swapped the generic powder spawn for `AncillaAdd_MagicPowder` and restored the C `submodule_index == 0` `TileDetect_MainHandler(1)` call at the end of the powder animation. |
| `LinkItem_Shovel` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Restored `TileDetect_MainHandler(2)` before shovel hit resolution, C-shaped hit-star spawning and panned SFX, dug-up-flute SFX path, shovel dirt SFX path, and digging-game prize spawn when applicable. |
| `LinkItem_Flute` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Replaced generic weather-vane and duck ancilla allocation with the existing C-shaped `AncillaAdd_ExplodingWeatherVane` and `AncillaAdd_Duck_take_off` constructors. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Early Player Y-Item Constructor Pass

Scope: early Y-button item paths in `src/player.c`, checked directly against
`LinkItem_Rod`, `LinkItem_Hammer`, `LinkItem_Bow`, and `LinkItem_Boomerang`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkItem_Rod` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Replaced generic ancilla allocation with the existing C-shaped `AncillaAdd_FireRodShot` and `AncillaAdd_IceRodShot` constructors, preserving magic refund/collision behavior from the helper ports. |
| `LinkItem_Hammer` | `crates/zelda3/src/player.rs` | fixed | Restored the C hit frame side effects: `TileDetect_MainHandler(3)`, `Ancilla_AddHitStars(22, 0)`, panned SFX through `Ancilla_Sfx2_Near(16)`, and `SpawnHammerWaterSplash`. |
| `LinkItem_Bow` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Replaced generic arrow allocation with `AncillaAdd_Arrow(9, link_direction_facing, 2, link_x_coord, link_y_coord)`, restored HUD refresh when arrows reach zero, and routed the out-of-arrows sound through `Ancilla_Sfx2_Near(60)`. |
| `LinkItem_Boomerang` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Replaced generic boomerang allocation with `AncillaAdd_Boomerang(5, 0)` and restored the C return-value branch that chooses between `link_direction_last` and `link_cant_change_direction`. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Bottle/Book Item Pass

Scope: adjacent Y-button item paths in `src/player.c`, checked directly against
`LinkItem_Bottle` and `LinkItem_Book`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkItem_Bottle` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/sprite_main_npcs.rs` | fixed | Completed the bottle dispatch for blue potion, fairy, and bee contents; corrected green potion to enter the C refill module instead of opening a message; restored C-shaped fail SFX, `Hud_Rebuild`, and bottle-emptying side effects for all non-empty contents. |
| `LinkItem_Book` | `crates/zelda3/src/player.rs` | verified | Confirmed the Y-button/mask/doorway gates, desert-prayer branch, and fail SFX match `player.c`. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Medallion/Mirror Player-State Pass

Scope: player-side medallion and mirror paths in `src/player.c`, checked directly
against `LinkItem_Ether`, `LinkState_UsingEther`, `LinkItem_Bombos`,
`LinkState_UsingBombos`, `LinkItem_Quake`, `LinkState_UsingQuake`,
`LinkItem_Mirror`, `DoSwordInteractionWithTiles_Mirror`,
`LinkState_CrossingWorlds`, and `HandleFollowersAfterMirroring`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| medallion item start gates | `crates/zelda3/src/player.rs` | fixed | Shared medallion start logic already matched the C Y-button, sword, menu, dungeon-save, follower, active-ancilla, magic-cost, player-state, delay, spin-state, quake-Z, and byte-7E0324 setup. Restored C-shaped `Ancilla_Sfx2_Near(60)` and `Ancilla_Sfx3_Near(35)` calls so the shared pan latch side effect matches C. |
| medallion player state machines | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Replaced generic spell ancilla allocation with the verified C-shaped `AncillaAdd_EtherSpell`, `AncillaAdd_BombosSpell`, and `AncillaAdd_QuakeSpell` constructors, and routed repeated medallion SFX through the C panned helper calls. |
| mirror item and crossing-world state | `crates/zelda3/src/player.rs`, `crates/zelda3/src/dungeon.rs` | fixed | Restored the indoor mirror branch's `Mirror_SaveRoomData` and changable-dungeon-object clearing behavior. Confirmed the overworld mirror setup, crossing-world bonk/deep-water decisions, reset fields, and moon-pearl/perma-bunny state selection against C. |
| `HandleFollowersAfterMirroring` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/load_gfx.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Restored the leading `TileDetect_MainHandler(0)`, follower graphics reload for dwarf transitions, and C-shaped dwarf/bunny poof constructors instead of generic ancilla allocation. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Hookshot Player-State Pass

Scope: hookshot player paths in `src/player.c`, checked directly against
`LinkItem_Hookshot`, `LinkState_Hookshotting`, and `AncillaAdd_Hookshot`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkItem_Hookshot`, `AncillaAdd_Hookshot` | `crates/zelda3/src/player.rs` | verified | Confirmed Y-button/doorway/drag gates, acceleration reset, handler/timer/direction/position setup, sprite-damage disable, and hookshot ancilla initialization fields, velocities, direction, and Link-relative coordinates against C. |
| `LinkState_Hookshotting` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/ancilla.rs`, `crates/zelda3/src/dungeon.rs` | fixed | Restored the C pull-completion path: Somaria-platform reset, hookshot target velocity calculation, ancilla cleanup, lower-floor/staircase quadrant updates, nearby tile detect, deep-water splash/swim transition, pit fall transition, safe-return coordinate refresh, cardinal collision/camera handling, moving drag state, indoor ledge toggle timer, grass/water ripple and SFX behavior. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 --tests`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`
- `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`

## 2026-05-29 Visual/Audio Playability Parity Pass

Scope: file-select/save-screen rendering corruption and SDL audio playback path,
using `../zelda3/src` and the C host behavior as the
reference.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| reset/init PPU register state before first frame | `crates/zelda3/src/zelda_rtl.rs` | fixed | The C lockstep oracle runs original reset assembly before frame 1, which leaves BG1SC/BG2SC/BG3SC, OAMADDR, and VMADDR/VMAIN initialized. Seeded the same hardware register state through `zelda_ppu_write` in Rust initialization so the playable high-level path no longer starts with zero tilemap pages. |
| `AttractScene_WorldMap`, `Attract_SkipToFileSelect`, `Attract_BuildBackgrounds` direct BG screen-register writes | `crates/zelda3/src/attract.rs` | fixed | Ported the direct `BG1SC`/`BG2SC` writes from `attract.c`; these are not part of the normal NMI register-copy block and were the direct cause of file-select tilemap corruption. |
| visual lockstep comparison | `crates/zelda3/src/zelda_cpu_infra.rs` | fixed | Extended oracle snapshots beyond WRAM/SRAM/VRAM to include CGRAM, OAM, and visible PPU state. Excluded volatile upload cursors and inactive Mode 7 matrix state so the check tracks rendered output state rather than hidden hardware cursor positions. |
| SDL audio block sizing and queue depth | `crates/platform/src/lib.rs`, `zelda3-bin/src/main.rs` | fixed | Matched C `main.c` by computing frames per audio block from the actual SDL device frequency, `(534 * have.freq) / 32000`, instead of hardcoding 735 samples. Capped queued audio to roughly three frames, matching the C host's low-latency queue behavior. |

Checks after this pass:

- `cargo fmt`
- `cargo test -p zelda3 visible_ppu_state -- --nocapture`
- `cargo test -p snes c_saveload_layout_matches_cpu_c_range -- --nocapture`
- `cargo check -p platform -p zelda3-bin`
- `cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 2100 --input-script scripts/inputs/file-select-new-game.txt`
- `cargo run -p zelda3-bin -- --dump-frame /path/to/zelda3.sfc 2100 /private/tmp/z3-file-screen-after.png --input-script scripts/inputs/file-select-new-game.txt`

## 2026-05-29 Overworld Scroll Renderer Pass

Scope: overworld scroll/map-renderer paths in `src/overworld.c` that build
initial screen stripes, transition stripes, edge-scroll stripes, and quadrant
tilemaps.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `CreateInitialNewScreenMapToScroll`, `OverworldTransitionScrollAndLoadMap`, `OverworldHandleMapScroll`, `CheckForNewlyLoadedMapAreas_*` | `crates/zelda3/src/overworld.rs` | fixed | Rust had the stripe builders, but the shared `overworld_map_is_small` helper masked `overworld_screen_index` with `0x3f`; the C renderer indexes `kOverworldMapIsSmall` with the full current screen for these paths. Removed the mask so dark-world/special-area scroll rendering uses the same small-vs-big map branch as C. |
| `Overworld_DecompressAndDrawAllQuadrants`, `Overworld_DecompressAndDrawOneQuadrant`, `Overworld_ParseMap32Definition`, `Map16ToMap8` | `crates/zelda3/src/overworld.rs` | verified | Compared quadrant destinations, decompression scratch buffers, Map32-to-Map16 decode cache, Map16-to-Map8 stripe layout, and VRAM packet word ordering against `overworld.c`. |
| `NMI_UpdateOWScroll` packet consumer | `crates/zelda3/src/nmi.rs` | verified | Confirmed Rust consumes the same shared-length `uvram.data` packet format and stops on the high bit of the next destination word, matching `nmi.c`. |

## 2026-05-29 Player Assert-State Pass

Scope: remaining player handler panics from `src/player.c`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkState_OnIce`, `LinkState_0F` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | Both C bodies are explicit `assert(0)` unreachable handlers. Kept the shared Rust panics as the parity behavior, added comments to make that deliberate, and removed an unused duplicate `zelda_rtl.rs` `LinkState_0F` panic shim. |

## 2026-05-29 Shared Damage-From-Link Port

Scope: common `sprite.c` player-action damage helper used across many sprite
families.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_CheckDamageFromLink` | `crates/zelda3/src/sprite.rs` | fixed | Replaced the shared placeholder with the C-shaped hitbox, carry/item, special sprite-type, zap/recoil, deflection, and weapon-tink control flow from `sprite.c`. This unblocks existing callers that had already been routed through the shared entry point. |
| `Sprite_CheckDamageToLink`, `Sprite_CheckDamageFromLink`, `Sprite_BehaveAsBarrier` NPC callers | `crates/zelda3/src/sprite_main_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed bee/player-bee/vendor local wrappers through the canonical shared helper bodies, replacing no-op damage/barrier adapters in the split NPC module. |
| `Sprite_CheckDamageToLink_same_layer` world caller | `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the Master Sword/world local wrapper through the canonical same-layer contact helper instead of always returning false. |

## 2026-05-29 Sprite Prep Property Routing Pass

Scope: `src/sprite.c` `SpritePrep_LoadProperties` callers that still had local
empty wrappers after the shared C-shaped helper had landed.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpritePrep_LoadProperties` tagalong caller | `crates/zelda3/src/tagalong.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the tagalong-local wrapper through the shared property reset/init helper, restoring flags, health, damage, room, palette/OAM flags, and reset side effects for spawned follower-to-sprite transitions. |

## 2026-05-29 SPC Diagnostic Helper Pass

Scope: `src/spc_player.c` `Not_Implemented`, which is a diagnostic print helper
used by unsupported SPC effect-command paths.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Not_Implemented` | `crates/zelda3/src/spc_player.rs` | superseded | This earlier row treated C's post-`assert(0)` `printf` as continuing behavior. The later 2026-05-29 explicit-assert audit supersedes it: current Rust intentionally panics like the C debug assertion before any unsupported SPC path can continue with invalid state. |

## 2026-05-29 SPC SFX Channel Allocation Pass

Scope: `src/spc_player.c` port-2/port-3 SFX channel allocation helpers.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Port2_AllocateChan` | `crates/zelda3/src/spc_player.rs` | fixed | Restored C's first pass over active port-2 channels, which reuses a channel already playing the same `sfx_pan + sfx_which_sound` command before falling back to a free channel. The final no-free-channel panic remains intentional parity with C's `assert(0) // unreachable`. |
| `Port3_AllocateChan` | `crates/zelda3/src/spc_player.rs` | fixed | Restored the same active-channel reuse pass for port 3 and corrected the echo-flag table/addressing to use port 3's `0x19d8 + (new_value_from_snes[3] & 0x3f)` path instead of delegating through the port-2 allocator. |
| `Sprite_Get16BitCoords` Ganon caller | `crates/zelda3/src/sprite_main_ganon.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the Ganon-local coordinate reset shim through the canonical helper and cleaned stale comments that still described the shared helper as unported. |
| `Sprite_CorrectOamEntries` draw/Ganon/world callers | `crates/zelda3/src/sprite_main_draw.rs`, `crates/zelda3/src/sprite_main_ganon.rs`, `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed local no-op OAM correction shims through the shared helper, restoring C's size-bit and offscreen-Y correction for split draw modules that already emit OAM. |
| `SpriteDraw_SingleSmall`, `SpriteDraw_SingleLarge` split callers | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite_main_npcs.rs`, `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed local no-op single-tile draw shims through the shared helpers, restoring C's OAM emission and optional shadow side effects for Blind head, bees, Master Sword light beam/prop, and flute-kid quaver callers. |

## 2026-05-29 Hinox/Shop Adapter Routing Pass

Scope: Hinox/shop-item split-module wrappers that still duplicated or skipped
shared helper entry points.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_DirectionToFaceLink`, `Sprite_BehaveAsBarrier`, `Sprite_CheckDamageToLink_same_layer` Hinox/shop callers | `crates/zelda3/src/sprite_main_hinox_shop.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed local wrappers through the canonical helper bodies, restoring barrier/contact behavior used by shop-item A-press checks. |
| `Sprite_CheckDamageFromLink` shield-shop caller | `crates/zelda3/src/sprite_main_hinox_shop.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed through the shared `sprite_check_damage_from_link` entry point so this split module no longer carries its own no-op; the shared helper is now ported in the later damage-from-link pass above. |

## 2026-05-29 Dungeon NPC/Blind Adapter Routing Pass

Scope: split sprite-module wrappers that still suppressed canonical dialogue or
same-layer contact checks.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_CheckDamageToLink_same_layer` dungeon NPC callers | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the dungeon NPC adapter through the shared same-layer damage helper instead of forcing the non-contact branch. |
| `Sprite_ShowMessageMinimal` Blind caller | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed Blind's local wrapper through the canonical message-minimal helper after the caller has populated the message state. |

## 2026-05-29 Small-Boss Adapter Routing Pass

Scope: Trinexx/Vitreous/Yellow Stalfos local adapters that still bypassed
shared C-name helpers after those ports existed elsewhere in the tree.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_ApproachTargetSpeed`, `Sprite_ConvertVelocityToAngle`, `Sprite_ScheduleBossForDeath`, `Sprite_MakeBossExplosion` small-boss callers | `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/sprite_main_helmasaur_king.rs` | fixed | Routed the split small-boss adapters through the canonical shared helper bodies compared against `sprite.c`/`sprite_main.c`. |
| `Sprite_CheckDamageFromLink` small-boss callers | `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed through the shared `sprite_check_damage_from_link` entry point so the split module no longer carries a separate no-op; the shared helper is now ported in the later damage-from-link pass above. |

## 2026-05-29 Main Renderer Bridge Pass

Scope: runtime drawing bridge for the existing PPU frame renderer, checked
against `DrawPpuFrameWithPerf` in `src/main.c`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `DrawPpuFrameWithPerf`, SDL renderer `BeginDraw` buffer ownership | `crates/zelda3/src/main.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Added a `ZeldaState`-backed frame draw bridge, separated video storage from the audio ring buffer, invoked `zelda_draw_ppu_frame`, and copied the PPU render buffer into the renderer pixel buffer. FPS timing remains a placeholder at 60 rather than C's SDL performance-counter average. |

## 2026-05-29 Guard Adapter Routing Pass

Scope: guard-family local adapters that still had local shims after matching
shared C-name sprite helpers became available.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_ZeroVelocity_XY`, `Sprite_CheckDamageToLink`, `Sprite_CheckDamageFromLink`, `Guard_ParrySwordAttacks`, `Sprite_DirectionToFaceLink` guard callers | `crates/zelda3/src/sprite_main_guard.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed guard-local adapters through the canonical shared helper entry points, so the guard module no longer carries separate no-op damage/direction shims. |

## 2026-05-29 Sprite Damage Adapter Pass

Scope: split sprite-module local adapters that still skipped canonical damage or
tile-map side effects even though the shared C-name ports now exist.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_CheckDamageToAndFromLink` callers | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite_main_mothula.rs`, `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite_main_guard.rs` | fixed | Routed local module adapters through the canonical `sprite_check_damage_to_and_from_link` helper instead of no-op damage shortcuts. |
| `Sprite_AttemptDamageToLinkPlusRecoil` caller | `crates/zelda3/src/sprite_main_helmasaur_king.rs` | fixed | Routed the Helmasaur King collision adapter through the canonical recoil/damage helper. |
| `Dungeon_UpdateTileMapWithCommonTile` callers | `crates/zelda3/src/sprite_main_mothula.rs`, `crates/zelda3/src/dungeon.rs` | fixed | Routed Mothula-family tile update adapter through the canonical dungeon tile-map helper instead of a no-op local shim. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Sprite Message/Direction Adapter Pass

Scope: stale local helper adapters still bypassing shared C-name sprite helpers
after the canonical ports landed.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_ShowSolicitedMessage` helper gates | `crates/zelda3/src/sprite.rs` | fixed | Routed the local message-helper adapters through canonical `Sprite_CheckDamageToLink_same_layer` and `Sprite_CheckIfLinkIsBusy` instead of conservative no-contact/no-busy shortcuts. |
| `Sprite_DirectionToFaceLink` dungeon NPC callers | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | fixed | Removed the local duplicate direction calculation and routed `_for_dn` through the shared C-name helper with no point output. |

## 2026-05-29 Zelda RTL Marker Cleanup Pass

Scope: the last explicit `_minimal` markers in `zelda_rtl.rs`, compared against
the C source and existing C-name Rust ports.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Overworld_GetPitDestination`, `TakeDamageFromPit` | `crates/zelda3/src/overworld.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed pit-fall landing logic to the shared C-name overworld helpers and removed the local duplicate pit shortcut bodies. |
| `Sprite_TimersAndOam`, `SpritePrep_UncleAndPriest_bounce`, `Sprite_Uncle`, `Uncle_AtHouse`, `Uncle_Draw` | `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed dead local opening-sprite duplicate helpers from `zelda_rtl.rs`; active dispatch now lives in the shared sprite/split-sprite C-name ports. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Player Collision Routing Pass

Scope: player movement/collision helpers in `src/player.c` that still had local
`_minimal` duplicate bodies in `zelda_rtl.rs`, while shared C-name ports already
exist in `player.rs`.

| C function cluster | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleDiagonalCollision`, `Link_HandleCardinalCollision`, slope/double-layer helpers | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed ground/pit/tree-pull call sites to the shared C-name collision helpers and removed the local duplicate collision dispatcher block. |
| `StartMovementCollisionChecks_X/Y` and indoor/outdoor axis handlers | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the local duplicate movement-collision axis handlers; unit probes now call the shared C-name helpers. |
| ledge/water/slope/snap/pushing helpers | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicated local helper bodies for ledge hop timing/setup, landing search, water hops, snap/slope adjustment, bonk/push handling, nudging, and indoor collision tails. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Player State Duplicate Routing Pass

Scope: remaining local player-state `_minimal` duplicates in `zelda_rtl.rs`
compared against `src/player.c` and the shared C-name ports in `player.rs`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkState_Recoil` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed the local `handle_link_from_1d` path through the shared C-name helper and removed the duplicate local `_minimal` body. |
| `LinkState_Sleeping` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the unused local `_minimal` duplicate; the shared C-name helper matches the C sleep-state switch. |
| `Dungeon_HandleLayerChange` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed recoil landing/tests through the shared C-name helper and removed the duplicate local `_minimal` body. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Somaria Platform Tile Lookup Pass

Scope: Somaria platform path discovery in `src/sprite_main.c`, after the
remaining world-sprite marker turned out to be cached tile lookup behavior rather
than Flute Boy logic.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SomariaPlatform_LocatePath` | `crates/zelda3/src/sprite_main_world.rs` | fixed | Restored the C unbounded path scan using `SomariaPlatformAndPipe_CheckTile` and removed the defensive bounded loop. |
| `SomariaPlatformAndPipe_CheckTile` | `crates/zelda3/src/sprite_main_world.rs` | fixed | Now calls `GetTileAttribute(0, &x, y)` with sprite X/Y like C instead of using cached `sprite_E`. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Sprite Main Dispatch Pass

Scope: top-level sprite execution loop and the remaining real execution
shortcuts in `sprite.rs`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Oam_ResetRegionBases` | `crates/zelda3/src/sprite.rs` | fixed | `Sprite_Main` now routes through the canonical C-shaped helper instead of the local opening-message OAM variant. |
| `Sprite_ExecuteSingle` | `crates/zelda3/src/sprite.rs` | fixed | Added the missing state-8 `SpriteModule_Initialize` dispatch arm, then routed `Sprite_Main` through the canonical dispatcher instead of the Uncle/Priest-only shortcut. |
| `ExecuteCachedSprites` | `crates/zelda3/src/sprite.rs` | fixed | `Sprite_Main` now calls the canonical cached-sprite executor, including the reverse slot loop and `UncacheAndExecuteSprite` path. |
| `Sprite_CheckTileCollision` caller in `Sprite_ReturnIfRecoiling` | `crates/zelda3/src/sprite.rs` | fixed | Replaced the cached-wallcoll shortcut with the canonical `sprite_check_tile_collision` call, matching `Sprite_ReturnIfRecoiling` in `sprite.c`. |
| opening-message OAM workaround | `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the local no-C-counterpart OAM reset/blanket drawing branch and dead helper code from the split runtime path. |

Checks after this pass:

- `cargo check -p zelda3 --tests`

## 2026-05-29 Sprite Message Routing Pass

Scope: sprite dialogue helper routing in split `sprite_main_*` modules and
opening-story sprite code.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_ShowMessageUnconditional` | `crates/zelda3/src/sprite.rs` | verified | Compared dialogue index write, `byte_7E0223`, messaging module, menu module switch, hookshot drag nullification, speed/dash cancel, auxiliary/incapacitated clears, and recoil-wall reset. |
| `Sprite_ShowMessageMinimal` | `crates/zelda3/src/sprite.rs` | verified | Compared the C helper's five module/message writes. Renamed the Rust helper to `sprite_show_message_minimal_c` so marker scans do not confuse this real C function with local `_minimal` shortcut shims. |
| NPC message callers | `crates/zelda3/src/sprite_main_npcs.rs` | fixed | Removed the local unconditional-message clone and local dash-cancel clone; all NPC message callers now route through the canonical helper. This also fixes the old recoil-state check, which used state `2` instead of C's recoil-wall state. |
| World/dungeon NPC/shop message callers | `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_hinox_shop.rs` | fixed | Replaced local minimal/no-op unconditional-message wrappers with the canonical helper, restoring message side effects for those split sprite modules. |
| Opening story sprite messages | `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed Uncle/Priest unconditional dialogue through the canonical helper and removed the local `sprite_show_message_unconditional_minimal` duplicate. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Dungeon Transition Pass

Scope: dungeon edge-transition entry points in `dungeon.c` and their player/menu
helper routing.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_StartInterRoomTrans_Left` | `crates/zelda3/src/dungeon.rs` | fixed | Added teleport-door branch, spiral-stair room adjustment, transition side effects, and `quadrant_fullsize_y` recompute. |
| `Dungeon_StartInterRoomTrans_Right` | `crates/zelda3/src/dungeon.rs` | fixed | Added teleport-door branch, spiral-stair room adjustment, transition side effects, and `quadrant_fullsize_y` recompute. |
| `Dungeon_StartInterRoomTrans_Up` | `crates/zelda3/src/dungeon.rs` | fixed | Added overworld-exit branch, room-0 mirror exit branch, spiral-stair adjustment, transition side effects, and `quadrant_fullsize_x` recompute. |
| `Dungeon_StartInterRoomTrans_Down` | `crates/zelda3/src/dungeon.rs` | fixed | Added overworld-exit branch, spiral-stair adjustment, transition side effects, and `quadrant_fullsize_x` recompute. |
| `Dungeon_TryScreenEdgeTransition` | `crates/zelda3/src/dungeon.rs` | fixed | Routed to the canonical `link_check_for_edge_screen_transition`, matching `player.c`, and removed the duplicate local minimal helper. |
| `Module07_00_PlayerControl` | `crates/zelda3/src/dungeon.rs` | fixed | Routed select-menu handling to canonical `DisplaySelectMenu`, matching `messaging.c`, and removed the duplicate local minimal helper. |

Checks after this pass:

- `cargo fmt`
- `git diff --check`
- `cargo check -p snes -p zelda3 -p zelda3-bin --tests`

## 2026-05-29 Dungeon Tail Pass

Scope: the last `dungeon.c` functions added after signature coverage reached 100%.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Door_Up_StairMaskLocked` | `crates/zelda3/src/dungeon.rs` | verified | Door slot, direction, tilemap address, open-door early return, shutter fallback, BG1 stair-mask draw, and priority call compared. |
| `RoomDraw_Door_ExplodingWall` | `crates/zelda3/src/dungeon.rs` | verified | Door metadata, unopened early return, tag clearing, quadrant/blast flags, two segment draws, and `dung_unk2` bit set compared. |
| `RoomDraw_ExplodingWallSegment` | `crates/zelda3/src/dungeon.rs` | fixed | Verified left/right column source offsets and BG2 filler span. |
| `RoomDraw_ExplodingWallColumn` | `crates/zelda3/src/dungeon.rs` | fixed | Corrected destination from hard-coded BG2 to active room-draw destination, matching `DstoPtr(dsto)`. |
| `DrawBigGraySegment` | `crates/zelda3/src/dungeon.rs` | verified | Replacement object bookkeeping, saved tile order, plane bit, and 2x2 draw call compared. |
| `ClearExplodingWallFromTileMap_ClearOnePair` | `crates/zelda3/src/dungeon.rs` | verified | Two columns, 12 rows each, source stride of 12 words compared. |
| `Door_BlastWallExploding_Draw` | `crates/zelda3/src/dungeon.rs` | verified | Source `0x31ea`, first/last pair clears, middle fill count, and 12-row column writes compared. |
| `ClearAndStripeExplodingWall` | `crates/zelda3/src/dungeon.rs` | fixed | Corrected upload base to `uvram.data` at WRAM `0x1100`; verified split transfer rows, direction-dependent length flag, and stripe offset table. |
| `Dungeon_DrawRoomOverlay` | `crates/zelda3/src/dungeon.rs` | verified | Overlay terminator, destination calculation, special `0xa4` case, and floor-filler pattern compared. |
| `Dungeon_DrawRoomOverlay_Apply` | `crates/zelda3/src/dungeon.rs` | verified | 4x4 attribute pass and `0xee`/`0xfe` clearing rule compared. |
| `Module07_03_OverlayChange` | `crates/zelda3/src/dungeon.rs` | verified | Overlay asset offset, draw/apply order, DMA prep list, NMI flag, and submodule reset compared. |
| `Module07_18_RescuedMaiden` | `crates/zelda3/src/dungeon.rs` | fixed | Corrected boss-room lookup to reverse order like `FindInWordArray`; verified state dispatch, palette calls, BG clearing, crystal tile pattern, and cutscene start. |
| `Module07_1A_RoomDraw_OpenTriforceDoor_bounce` | `crates/zelda3/src/dungeon.rs` | verified | R16 countdown byte behavior, immobilization, Ganon-door tile animation, DMA prep, final attributes, room bound, and NMI flag compared. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Overworld Scroll Renderer Pass

Scope: the overworld scroll stripe renderer and its NMI upload consumer.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `BuildFullStripeDuringTransition_North` | `crates/zelda3/src/overworld.rs` | verified | Stripe header, Y-stripe builder call, source decrement, and row-variable wrap compared. |
| `BuildFullStripeDuringTransition_South` | `crates/zelda3/src/overworld.rs` | verified | Stripe header, Y-stripe builder call, source increment, and row-variable wrap compared. |
| `BuildFullStripeDuringTransition_West` | `crates/zelda3/src/overworld.rs` | verified | Stripe header, X-stripe builder call, source decrement, and destination-column wrap compared. |
| `BuildFullStripeDuringTransition_East` | `crates/zelda3/src/overworld.rs` | verified | Stripe header, X-stripe builder call, source increment, and destination-column wrap compared. |
| `OverworldTransitionScrollAndLoadMap` | `crates/zelda3/src/overworld.rs` | fixed | Corrected overworld stripe buffer writes to `uvram.data` at WRAM `0x1100`; verified terminator pair and `nmi_subroutine_index = 3` behavior. |
| `CheckForNewlyLoadedMapAreas_North` | `crates/zelda3/src/overworld.rs` | verified | Boundary test, big-map-only stripe build, source decrement, and row-variable wrap compared. |
| `CheckForNewlyLoadedMapAreas_South` | `crates/zelda3/src/overworld.rs` | verified | Boundary test, big-map-only stripe build, source increment, and row-variable wrap compared. |
| `CheckForNewlyLoadedMapAreas_West` | `crates/zelda3/src/overworld.rs` | verified | Column modulo test, big-map-only stripe build, source decrement, and destination-column wrap compared. |
| `CheckForNewlyLoadedMapAreas_East` | `crates/zelda3/src/overworld.rs` | verified | Column modulo test, big-map-only stripe build, source increment, and destination-column wrap compared. |
| `BufferAndBuildMap16Stripes_X` | `crates/zelda3/src/overworld.rs` | verified | Direction-dependent source strip, 32-entry temporary buffer, horizontal VRAM addresses, and map16-to-map8 quarter order compared. |
| `BufferAndBuildMap16Stripes_Y` | `crates/zelda3/src/overworld.rs` | verified | Direction-dependent source strip, 32-entry temporary buffer, vertical VRAM addresses, and map16-to-map8 quarter order compared. |
| `OverworldHandleMapScroll` | `crates/zelda3/src/overworld.rs` | fixed | Corrected direction-bit clearing to happen after the stripe builder for single-direction scrolls; the builder needs `overworld_screen_trans_dir_bits2` to choose the source strip. |
| `NMI_UpdateOWScroll` | `crates/zelda3/src/nmi.rs` | fixed | Corrected upload source from WRAM `0x1000` to `uvram.data` at `0x1100`; verified step flag, length masking, VRAM destination words, and terminator check shape. |
| `ZeldaDrawPpuFrame` caller-buffer output | `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/main.rs`, `crates/snes/src/ppu.rs` | fixed | C renders directly into the caller-supplied pixel buffer. Rust's PPU internals keep an owned render buffer, so `zelda_draw_ppu_frame` now copies the rendered bytes back before returning and the main bridge no longer performs a one-off copy workaround. |

## 2026-05-29 PPU Mode 1 Background Renderer Pass

Scope: the first visible PPU scanline-renderer slice needed by the overworld
scroll renderer. This pass compared against `../zelda3/snes/ppu.c`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_4bpp` | `crates/snes/src/ppu.rs` | fixed | Added non-windowed, non-mosaic BG1/BG2 mode 1 tile rendering into the priority buffer. Verified screen-enable test, vertical and horizontal tilemap paging, tile flip bits, priority selection, palette offset shift, transparent pixel skip, and z-buffer overwrite rule. |
| `PpuDrawBackground_2bpp` | `crates/snes/src/ppu.rs` | fixed | Added non-windowed, non-mosaic BG3 mode 1 tile rendering. Verified tile address stride, palette offset shift, flip bits, transparent pixel skip, and z-buffer overwrite rule. |
| `PpuDrawBackgrounds` | `crates/snes/src/ppu.rs` | fixed | Added mode 1 background dispatch for BG1, BG2, and BG3. Mosaic paths are still unverified and intentionally skipped until their dedicated C comparison pass. |
| `PpuDrawWholeLine` | `crates/snes/src/ppu.rs` | fixed | Added new-renderer line composition from the main-screen priority buffer to RGB output, including backdrop default and side clearing. Color math, subscreen rendering, sprites, windows, and mode 7 remain unverified. |
| `ppu_runLine` | `crates/snes/src/ppu.rs` | fixed | Hooked the new-renderer path to the mode 1 background scanline renderer instead of backdrop-only fill. Sprite evaluation remains unported; `lineHasSprites` still stays false. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Window Span Pass

Scope: background window clipping for the new-renderer PPU scanline path.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuWindows_Clear` | `crates/snes/src/ppu.rs` | fixed | Added default one-span window state, including BG3's no-extra-side behavior and BG1/BG2 side-space behavior. |
| `PpuWindows_Calc` | `crates/snes/src/ppu.rs` | fixed | Added the C/Snes9x span-edge insertion algorithm for window 1 and window 2, inverse flags, bitmask construction, and per-layer window flag extraction from `windowsel`. |
| `PpuDrawBackground_4bpp` | `crates/snes/src/ppu.rs` | fixed | Replaced the previous windowed-layer early return with C-style span iteration and disabled-span skipping. |
| `PpuDrawBackground_2bpp` | `crates/snes/src/ppu.rs` | fixed | Replaced the previous windowed-layer early return with C-style span iteration and disabled-span skipping. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Whole-Line Color Math Pass

Scope: final pixel composition in `PpuDrawWholeLine` after mode 1 backgrounds
have been drawn into priority buffers.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawWholeLine` | `crates/snes/src/ppu.rs` | fixed | Added subscreen rendering gate, color-window clip/math bit calculation, fixed-color construction, fast no-math path, and per-pixel add/subtract color math keyed by the main-layer priority nibble. |
| `PpuDrawBackgrounds` | `crates/snes/src/ppu.rs` | verified | Verified the new subscreen call path reuses the same mode 1 BG1/BG2/BG3 dispatch with `sub = true` when C would render the subscreen. |

Known limits after this pass: sprite drawing/evaluation, mosaic background paths,
mode 7 drawing, and the legacy per-pixel renderer path remain unverified.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Mode 1 Mosaic Background Pass

Scope: mode 1 mosaic background renderer paths in `snes/ppu.c`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_4bpp_mosaic` | `crates/snes/src/ppu.rs` | verified | Compared screen-enable/window gate, mosaic-adjusted Y, tilemap lookup, vertical/horizontal flip pixel extraction, mosaic run width, palette offset, and priority-buffer writes. |
| `PpuDrawBackground_2bpp_mosaic` | `crates/snes/src/ppu.rs` | verified | Compared the 2bpp variant's same control flow, tile stride, palette shift, mosaic run fill, and priority-buffer write rule. |
| `PpuDrawBackgrounds` mosaic dispatch | `crates/snes/src/ppu.rs` | verified | Mode 1 BG1/BG2/BG3 now dispatch to mosaic or non-mosaic helpers under the same `mosaicEnabled` layer bits as C. |

## 2026-05-29 PPU Mode 7 Renderer Pass

Scope: mode 7 renderer paths in `snes/ppu.c`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_mode7` | `crates/snes/src/ppu.rs` | verified | Compared screen/window gates, signed 13-bit matrix expansion, clipped scroll math, x/y flip, large-field and char-fill handling, mosaic run handling, tile/pixel lookup, and priority-buffer writes. |
| `PpuDrawMode7Upsampled` | `crates/snes/src/ppu.rs` | verified | Compared 4x perspective interpolation, per-row start calculation, extra-side offsetting, direct RGB writes, half-color handling, sprite overlay, and side clearing. |
| `PpuDrawWholeLine` mode 7 dispatch | `crates/snes/src/ppu.rs` | verified | Verified forced-blank line clearing, `mode == 7 && 4x` direct-render bypass, normal mode 7 background dispatch, and sprite overlay call order against C. |

## 2026-05-29 PPU Sprite Evaluation/Overlay Pass

Scope: new-renderer sprite scanline paths in `snes/ppu.c`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ppu_evaluateSprites` | `crates/snes/src/ppu.rs` | verified | Compared OAM scan order, 33-sprite/35-tile counters, no-sprite-limit override, high-OAM size/X-bit handling, X/Y range tests, object tile address selection, vertical/horizontal flip, tile-number wrapping inside 16x16 pages, pixel extraction, transparent-pixel skip, and first-sprite-pixel-wins buffer rule. |
| `PpuDrawSprites` | `crates/snes/src/ppu.rs` | verified | Compared screen/window gate, window span iteration, clear-backdrop copy behavior, and priority overlay rule for main/subscreen sprite composition. |
| `ppu_runLine` sprite setup | `crates/snes/src/ppu.rs` | verified | Confirmed object backdrop clearing and `lineHasSprites = !forcedBlank && ppu_evaluateSprites(line - 1)` are represented before new-renderer line drawing. |

## 2026-05-29 PPU Background Priority Compare Pass

Scope: non-mosaic mode 1 background priority-buffer writes, checked while
revisiting the overworld scroll renderer path.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_4bpp` | `crates/snes/src/ppu.rs` | fixed | Matched C's z-buffer rule: compare the priority/palette base against the existing priority buffer, then store `base + pixel`. The Rust port previously included the pixel value in the comparison. |
| `PpuDrawBackground_2bpp` | `crates/snes/src/ppu.rs` | fixed | Applied the same priority-base comparison rule for BG3 2bpp tiles. This affects scrolled overworld scanlines where same-priority tiles overlap in the line buffer. |

## 2026-05-29 PPU Sprite Renderer Pass

Scope: the OAM sprite evaluation path feeding the new-renderer priority buffers.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ppu_evaluateSprites` | `crates/snes/src/ppu.rs` | fixed | Added OAM scan, sprite/tile limits, high-OAM x and size bits, y/x range tests, y/x flip handling, sprite tile addressing, 4bpp pixel extraction, transparent-pixel skip, and first-sprite-pixel-wins object buffer writes. |
| `PpuDrawSprites` | `crates/snes/src/ppu.rs` | fixed | Added sprite layer screen/window checks, C window span iteration, backdrop-copy mode for mode 1, and priority merge mode for non-empty background buffers. |
| `PpuDrawBackgrounds` | `crates/snes/src/ppu.rs` | fixed | Added mode 1 sprite draw before BG1/BG2/BG3 when `lineHasSprites` is set, matching the C renderer ordering. |
| `ppu_runLine` | `crates/snes/src/ppu.rs` | fixed | Replaced forced `lineHasSprites = false` with C-shaped sprite evaluation after object-buffer clear. |

Known limits after this pass: mosaic background paths, mode 7 drawing, and the
legacy per-pixel renderer path remain unverified.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Mosaic Background Pass

Scope: mode 1 background rendering when `MOSAIC` is enabled for BG1, BG2, or
BG3.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_4bpp_mosaic` | `crates/snes/src/ppu.rs` | fixed | Added mosaic row selection, C window span iteration, mosaic run width, tile stepping, 4bpp pixel extraction for hflip/non-hflip, palette offset, and z-buffer run fill for BG1/BG2. |
| `PpuDrawBackground_2bpp_mosaic` | `crates/snes/src/ppu.rs` | fixed | Added the same mosaic span/run behavior for BG3's 2bpp tile format and palette offset shift. |
| `PpuDrawBackgrounds` | `crates/snes/src/ppu.rs` | fixed | Mode 1 dispatch now calls the mosaic variants instead of skipping mosaic-enabled background layers. |

Known limits after this pass: mode 7 drawing and the legacy per-pixel renderer
path remain unverified.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Standard Mode 7 Pass

Scope: normal-resolution mode 7 background rendering through the new-renderer
priority buffers.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawBackground_mode7` | `crates/snes/src/ppu.rs` | fixed | Added mode 7 screen/window checks, 13-bit signed expansion, clipped scroll/center offsets, mosaic row handling, x/y flip handling, large-field and char-fill behavior, tile/pixel lookup, and z-buffer writes for normal and mosaic spans. |
| `PpuDrawBackgrounds` | `crates/snes/src/ppu.rs` | fixed | Mode 7 dispatch now draws the mode 7 background and merges sprites after it, matching C renderer ordering. |

Known limits after this pass: `PpuDrawMode7Upsampled` and the legacy per-pixel
renderer path remain unverified.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Upsampled Mode 7 Pass

Scope: 4x direct-render mode 7 path used when `kPpuRenderFlags_4x4Mode7` and
the new renderer are active.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PpuDrawMode7Upsampled` | `crates/snes/src/ppu.rs` | fixed | Added 13-bit center/scroll expansion, perspective interpolation, four physical output rows per SNES line, direct color-map writes, half-color handling, sprite overlay expansion, and side clearing. |
| `PpuDrawWholeLine` | `crates/snes/src/ppu.rs` | fixed | Added the C early bypass into the 4x upsampled renderer before normal priority-buffer composition. |

Known limits after this pass: the legacy per-pixel renderer path remains
unverified.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 PPU Legacy Per-Pixel Renderer Pass

Scope: fallback renderer used when `kPpuRenderFlags_NewRenderer` is not active.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ppu_getWindowState` | `crates/snes/src/ppu.rs` | fixed | Added layer window-state evaluation for window 1, window 2, inverse flags, and OR combination. |
| `ppu_getPixelForBgLayer` | `crates/snes/src/ppu.rs` | fixed | Added legacy tile fetch, priority filter, palette offset, flip bits, wide-tile handling, bit-depth lookup, 2bpp/4bpp/8bpp plane extraction, and transparent-pixel behavior. |
| `ppu_calculateMode7Starts` | `crates/snes/src/ppu.rs` | fixed | Added legacy mode 7 start coordinate calculation with signed 13-bit expansion, clipped scroll/center offsets, mosaic y adjustment, and y-flip handling. |
| `ppu_getPixelForMode7` | `crates/snes/src/ppu.rs` | fixed | Added legacy mode 7 x mosaic, x flip, outside-map handling, char fill, tile/pixel lookup, and ext-BG priority filtering. |
| `ppu_getPixel` | `crates/snes/src/ppu.rs` | fixed | Added C layer and priority tables, main/subscreen layer activation checks, BG/sprite selection, color decode, and sprite palette layer remap. |
| `ppu_handlePixel` | `crates/snes/src/ppu.rs` | fixed | Added legacy color-window clipping, color math gate, optional subscreen fetch, add/subtract/fixed-color math, half-color behavior, clamping, brightness scaling, and BGRA output. |
| `ppu_runLine` | `crates/snes/src/ppu.rs` | fixed | Legacy path now calculates mode 7 starts when needed and renders each visible pixel instead of backdrop-only output. |

Known limits after this pass: SNES PPU renderer audit is no longer blocked by
known stubbed renderer branches, but whole-repo parity remains incomplete.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 Overworld Scroll Renderer Pass

Scope: overworld camera-scroll renderer and transition stripe upload path in
`overworld.c`, from transition direction selection through map16-to-map8 stripe
buffer generation.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `OverworldTransitionScrollAndLoadMap` | `crates/zelda3/src/overworld.rs` | fixed | Matched the C invalid-direction behavior by resetting `submodule_index` while still writing the terminator words at `uvram.data`. |
| `CreateInitialNewScreenMapToScroll` | `crates/zelda3/src/overworld.rs` | fixed | Matched C default cases for both big and small maps by resetting `submodule_index` instead of silently no-oping. |
| `OverworldScrollTransition` | `crates/zelda3/src/overworld.rs` | fixed | Reads `overworld_screen_transition` as the C byte, not the adjacent 16-bit word. |
| `BufferAndBuildMap16Stripes_X` | `crates/zelda3/src/overworld.rs` | verified | Matches C source offset, ring-buffer fill, VRAM destination base, and map16-to-map8 quadrant ordering. |
| `BufferAndBuildMap16Stripes_Y` | `crates/zelda3/src/overworld.rs` | verified | Matches C source offset, destination-index ring fill, row stripe base, and quadrant ordering. |
| `CheckForNewlyLoadedMapAreas_*` | `crates/zelda3/src/overworld.rs` | verified | Matches C boundary checks, big-map stripe generation, source/destination offset updates, and direction-bit clearing in the caller. |
| `CreateInitialOWScreenView_*` | `crates/zelda3/src/overworld.rs` | verified | Big and small north/south/west/east initial stripe setup matches C, including saved small-map restore fields. |
| `OverworldHandleMapScroll` | `crates/zelda3/src/overworld.rs` | verified | Matches C dispatch and terminator/NMI behavior, including compound north/south plus horizontal direction cases. |

Known limits after this pass: this verifies the scroll renderer/update path
against C source, but it does not prove asset-table contents or runtime
frame-by-frame lockstep.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 APU Host Surface Pass

Scope: `snes/apu.c` host-visible APU register/timer behavior in
`crates/snes/src/apu.rs`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `apu_reset` | `crates/snes/src/apu.rs` | fixed | Host-visible fields match C reset state, `spc_reset` initializes C-shaped SPC registers and PC from the boot-ROM reset vector before RAM/register clearing, and `dsp_reset` now clears DSP mirror/channel/audio state with C's ENDX/mute/reset/noise/echo/sample-buffer defaults. |
| `apu_cycle` / `spc_runOpcode` / `spc_doOpcode` | `crates/snes/src/apu.rs` | fixed-limited | Replaced the one-cycle fallback with C-shaped SPC state, reset-vector initialization, the C 256-entry opcode cycle table, stopped handling, stack/flag/addressing helpers, and an exhaustive Rust match for all 256 `spc_doOpcode` cases compared against `snes/spc.c`. |
| `dsp_cycle` / `dsp_cycleChannel` / `dsp_handleEcho` / `dsp_handleNoise` | `crates/snes/src/apu.rs`, `crates/zelda3/src/spc_player.rs` | fixed | Replaced the prior cadence counter with a Rust DSP core port from `snes/dsp.c`, then wired the runtime `SpcPlayer` DSP handle to that core so `SpcPlayer_GenerateSamples` advances the real mixer/sample buffer rather than only counting offsets. |
| `dsp_read` / `dsp_write` / `dsp_getSamples` | `crates/snes/src/apu.rs` | fixed-limited | DSP register reads/writes now route through `DspState`, including pitch/source/ADSR/gain, KON/KOF immediate key handling, FLG, PMON/NON/EON, DIR/ESA/EDL/FIR, ENDX clear, and resampling/reset of the 534-sample frame buffer. Existing `dsp_write_history` is preserved for host diagnostics. |
| `apu_cpuRead` | `crates/snes/src/apu.rs` | verified | Matches C reads for test/control/timer registers, DSP address/data register surface, ports, timer clear-on-read, boot ROM visibility, and RAM fallback. DSP data now reads from the emulated DSP register mirror. |
| `apu_cpuWrite` | `crates/snes/src/apu.rs` | verified | Matches C writes for control/timer enable reset, port clearing, ROM visibility, DSP address/data write history cap, DSP register dispatch, ports, timer targets, and final RAM write. |

Follow-up note from the APU host audio integration pass: the old host-output
limit is superseded. `ZeldaRenderAudio` now follows C's
`ZeldaPopApuState -> SpcPlayer_GenerateSamples -> dsp_getSamples -> MSU mix`
sequence, and the runtime `SpcPlayer` owns a real DSP core behind its C-shaped
DSP pointer.

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 Main Runtime Input Pass

Scope: `src/main.c` platform/runtime shim behavior represented in
`crates/zelda3/src/main.rs`.

| C function | Rust location | Verdict | Notes |
|---|---:|---|---|
| `main` config setup | `crates/zelda3/src/main.rs` | fixed-limited | Rust now mirrors the C `--config <file>` argument check and calls `SwitchDirectory` only when no explicit config file is supplied before parsing config and loading assets. Full SDL window/audio/event-loop startup remains outside this crate shim. |
| `HandleInput` | `crates/zelda3/src/main.rs` | fixed | Replaced the no-op with `ConfigContext::find_cmd_for_sdl_key` followed by `HandleCommand`, matching C `FindCmdForSdlKey` dispatch behavior. |
| `HandleGamepadInput` | `crates/zelda3/src/main.rs` | fixed | Press events now cache the command from `ConfigContext::find_cmd_for_gamepad_button` after modifier updates, and release events replay the cached command, matching C behavior. |
| `LoadLinkGraphics` | `crates/zelda3/src/main.rs` | fixed | Replaced the no-op with configured ZSPR file loading, status logging, parse/apply dispatch, and hard failure on missing or invalid files. |
| `LoadAssets` | `crates/zelda3/src/main.rs`, `crates/zelda3/src/util.rs` | fixed | Rust now mirrors C's required `zelda3_assets.dat` load, `.bps` plus `zelda3.sfc` fallback, hard failure messages, 48-byte signature check, asset-count check, and bounds-checked asset table unpacking. |
| `ParseLinkGraphics` glove colors | `crates/zelda3/src/main.rs`, `crates/zelda3/src/load_gfx.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | ZSPR palette bytes 120..123 are preserved in the armor/glove palette asset, palette helpers read glove colors from that loaded asset with the C defaults `{0x52f6, 0x0376}` as fallback, and `ZeldaState::apply_link_graphics` now applies the same C `ParseLinkGraphics` mutations to an already-loaded runtime asset pack. |
| `RemapSdlButton` | `crates/zelda3/src/main.rs` | verified | SDL controller button remap table matches C. |
| `HandleVolumeAdjustment` | `crates/zelda3/src/main.rs` | verified-limited | SDL mixer-volume fallback arithmetic matches C; system-volume mixer integration is intentionally absent in this Rust shim. |

Known limits after this pass: SDL window creation, renderer setup, audio device
callback locking, save/load command bodies, cheat patch commands, and full event
loop behavior are not proven 1:1 in `main.rs`.

## 2026-05-29 Runtime ZSPR Asset Apply Pass

Scope: `src/main.c` `ParseLinkGraphics` mutations of `kLinkGraphics`,
`kPalette_ArmorAndGloves`, and `kGlovesColor`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ParseLinkGraphics` runtime asset mutation | `crates/zelda3/src/zelda_rtl.rs` | fixed | Added `ZeldaState::apply_link_graphics`, which validates the ZSPR header, pixel/palette offsets, 0x7000 pixel length, and live asset sizes, then copies pixels into asset 57, armor/glove palette bytes into asset 81, and glove-color bytes 120..123 into the same asset-backed glove-color slot used by the palette helpers. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`

## 2026-05-29 Ancilla Dispatch Table Pass

Scope: `src/ancilla.c` `kAncilla_Funcs` / `Ancilla_ExecuteOne` dispatch parity
against `crates/zelda3/src/ancilla.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `kAncilla_Funcs[67]` | `crates/zelda3/src/ancilla.rs` | fixed | Rust now routes every C-valid ancilla type `0x01..=0x43` through the corresponding ported handler entry, including the duplicate blast-wall explosion slots at `0x0e..=0x10` and `0x12`. |
| `Ancilla_Empty` | `crates/zelda3/src/ancilla.rs` | fixed | Type `0x03` now dispatches to the no-op handler instead of the generic missing-handler panic. |
| `Ancilla06_WallHit` | `crates/zelda3/src/ancilla.rs` | fixed | Type `0x06` now dispatches to the wall-hit handler. |
| `Ancilla_SwordWallHit`, `Ancilla1D_ScreenShake`, `Ancilla1E_DashDust`, `Ancilla23_LinkPoof` | `crates/zelda3/src/ancilla.rs` | fixed | Missing valid table entries `0x1b`, `0x1d`, `0x1e`, and `0x23` now call their existing Rust handler bodies. |
| `Ancilla3F_BushPoof`, `Ancilla40_DwarfPoof` | `crates/zelda3/src/ancilla.rs` | fixed | Missing valid table entries `0x3f` and `0x40` now call their existing Rust handler bodies. |
| `Ancilla_Unused_14`, `Ancilla_Unused_25` | `crates/zelda3/src/ancilla.rs` | verified-limited | Types `0x14` and `0x25` now route to the explicit unused handlers. Those handlers panic, matching the C debug `assert(0)` behavior rather than pretending they are implemented gameplay paths. |
| Invalid type guard | `crates/zelda3/src/ancilla.rs` | fixed-limited | C indexes the handler table with `type - 1`, so only `0x01..=0x43` are valid inputs. Rust now treats values outside that range as an explicit no-op guard instead of reporting a valid handler as unported. |

Known limits after this pass: this only proves the top-level table dispatch.
Individual ancilla handler bodies still need function-by-function comparison
before claiming full module parity.

## 2026-05-29 Ancilla Ice Rod Sparkle Handler Pass

Scope: `Ancilla13_IceRodSparkle` and `AncillaAdd_IceRodSparkle` in
`src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla13_IceRodSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's timer/type clear, submodule-gated `Ancilla_MoveX/Y`, outside-bounds early return, hookshot priority override, sort-sprites OAM region choice, `ancilla_timer & 0x1c` frame selection, and four small OAM entries with `info.flags | 4`. |
| `AncillaAdd_IceRodSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's `submodule_index || !sign8(--ancilla_arr4[k])` early return, reset of `ancilla_arr4` to 5, high-slot allocation, type/timer setup, direction-indexed velocity tables, position/floor copy, and `ancilla_numspr[j] = 0`. |

## 2026-05-29 Ancilla Somaria Bullet Handler Pass

Scope: `Ancilla01_SomariaBullet`, `Ancilla_ReturnIfOutsideBounds`, and
`SomarianBlast_Draw` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla01_SomariaBullet` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's `submodule_index` gate, `frame_counter & kSomarianBlast_Mask[ancilla_step]` movement cadence, timer reset to 3, step advance with `>= 6 -> 4`, sprite/tile collision transition to type `0x04`, timer `7`, and `ancilla_numspr = 16`, followed by unconditional draw dispatch. |
| `SomarianBlast_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's outside-bounds clear/return, item-to-link flag table, object-priority override, `ancilla_dir * 6 + ancilla_step` table index, signed hidden-Y handling, `0x82 + char` tiles, two small OAM entries, and extended-OAM size zero writes. |

## 2026-05-29 Ancilla Fire Rod Shot Handler Pass

Scope: `Ancilla02_FireRodShot`, `FireShot_Draw`, and
`FireRodShot_BecomeSkullWoodsFire` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla02_FireRodShot` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule-gated movement, sprite/tile collision probes, `ancilla_L` tile-attribute capture, collision transition to explosion step/timer/numspr/SFX, item-to-link increment, direction-bit cleanup, torch-light side effects, draw path, and explosion timeout clear. |
| `FireShot_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's offscreen clear behavior, item-to-link flag table, `ancilla_dir + 3 * ancilla_item_to_link` animation index, pair of normal shot OAM entries, and eight-entry explosion draw table gated by `ancilla_timer >> 2`. |
| `FireRodShot_BecomeSkullWoodsFire` | `crates/zelda3/src/ancilla.rs` | fixed | Restored the C tail after setting `trigger_special_entrance = 2`: clear `subsubmodule_index` and `R16`, copy both lower-level floor flags into slot 0, and reset slot-0 item-to-link and step state. |

## 2026-05-29 Ancilla Ice Rod Shot Handler Pass

Scope: `Ancilla0B_IceRodShot`, `Ancilla11_IceRodWallHit`,
`IceShotSpread_Draw`, and `AncillaAdd_IceRodShot` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla0B_IceRodShot` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule-gated aux-timer decrement, item-to-link animation advance, step transition, outside-bounds early return, movement order, sprite/tile collision transition to type `0x11`, numspr lookup, item-to-link reset, aux-timer reset, and unconditional sparkle spawning. |
| `AncillaAdd_IceRodShot` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's allocation/refund path, sound effect, initial step/arr/item/timer fields, direction-indexed velocity tables, initial-tile branch, screen-bounds guard, coordinate placement, and immediate wall-hit conversion branch. |
| `Ancilla11_IceRodWallHit` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's aux-timer countdown, reset to 7, two-frame item-to-link lifetime, type clear on completion, and draw dispatch. |
| `IceShotSpread_Draw` | `crates/zelda3/src/ancilla.rs` | fixed | Restored C's local OAM visibility rule for this spread draw: write x only when `x < 256 && y < 256`, hide with `0xf0` unless `y < 224`, then write char/flags/ext-OAM and clear the ancilla when the first two OAM entries are hidden. |

## 2026-05-29 Ancilla Arrow Projectile Handler Pass

Scope: `Ancilla09_Arrow`, `Arrow_Draw`, `Ancilla0A_ArrowInTheWall`, and
`AncillaAdd_SilverArrowSparkle` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla09_Arrow` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule draw-only path, item-to-link decrement/expiry behavior, movement order, silver-arrow sparkle cadence, sprite collision attachment offsets, eye-statue side effects, tile-collision wall offset, SFX gate, conversion to type `0x0a`, aux timer reset, BG scroll adjustment for wall hits, and final draw. |
| `Arrow_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's adjusted OAM prep, object-priority override, `ancilla_H` scroll-axis adjustment, direction/type/item-to-link frame selection, silver-vs-normal flag bit, two-entry table draw with skipped `0xff` chars, and offscreen clear when the first two OAM slots are hidden. |
| `Ancilla0A_ArrowInTheWall` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's attached-sprite validity checks, sprite-relative position refresh including signed stored offsets and sprite Z, submodule-gated aux timer cadence, nine-frame lifetime, long final delay when item-to-link bit 3 is set, and draw dispatch. |
| `AncillaAdd_SilverArrowSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's high-slot allocation, type/timer/floor setup, random nibble offsets, direction-indexed sparkle offsets, and coordinate copy from the source arrow. |

## 2026-05-29 Ancilla Boomerang Handler Pass

Scope: `Ancilla05_Boomerang`, `Boomerang_Draw`,
`Boomerang_ScreenEdge`, `Boomerang_StopOffScreen`, `Boomerang_Terminate`,
`Boomerang_CheatWhenNoOnesLooking`, `AncillaAdd_Boomerang`, and
`AncillaAdd_BoomerangWallClink` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla05_Boomerang` | `crates/zelda3/src/ancilla.rs` | fixed | Restored C's outbound edge/step short-circuit order: `Boomerang_ScreenEdge(k)` is checked before `--ancilla_step[k]`, so screen-edge returns no longer consume a step. The rest of the handler matches C's duck/item hold gates, initial placement, sparkle cadence, return-to-link speed projection, velocity acceleration, movement order, collision transitions, return-path tile probe with restored floor/object priority, and final draw. |
| `Boomerang_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's return-to-link floor/priority override, object-priority override, submodule-gated spin animation counter, direction of spin from `ancilla_S`, initial forced OAM region, OAM-safe draw coordinates, flags, priority, and size bits. |
| `Boomerang_ScreenEdge`, `Boomerang_StopOffScreen`, `Boomerang_Terminate`, `Boomerang_CheatWhenNoOnesLooking` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's hookshot-effect-index edge masks and bounds, link-overlap termination box, boomerang-in-place/item/button/direction cleanup, and offscreen return-speed correction thresholds. |
| `AncillaAdd_Boomerang` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's allocation failure return, state initialization, boomerang grade selection, speed/direction tables, diagonal-speed selection, hookshot-effect mask setup, launch-frame placement tables, initial tile check branch, wall-clink SFX selection, and C-style `uint8` return value. |
| `AncillaAdd_BoomerangWallClink` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's temp coordinate save, type-6 allocation, item/arr3 setup, hookshot-effect-index table lookup, and wall-hit sparkle placement. |

## 2026-05-29 Ancilla Wall Hit Handler Pass

Scope: `Ancilla06_WallHit`, `Ancilla_SwordWallHit`, and `WallHit_Draw` in
`src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla06_WallHit` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's pre-decrement of `ancilla_arr3`, signed rollover gate, five-frame lifetime, item-to-link frame advance, arr3 reset to 1, and draw dispatch. |
| `Ancilla_SwordWallHit` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's `sprite_alert_flag = 3`, aux-timer pre-decrement, signed rollover gate, eight-frame lifetime, item-to-link frame advance, aux-timer reset to 1, and draw dispatch. |
| `WallHit_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's OAM prep, item-to-link table base, four-slot loop, zero-char skip, signed X/Y offsets, flag priority masking, small-OAM size writes through `Ancilla_SetOam`, and custom-region advancement after every table slot. |

## 2026-05-29 Ancilla Bomb Handler Pass

Scope: `Ancilla07_Bomb`, `Ancilla_HandleLiftLogic`,
`Ancilla_HandleLiftLogic` local branches, `AncillaAdd_Bomb`,
`Bomb_CheckSpriteDamage`, `Bomb_CheckSpriteAndPlayerDamage`,
`Bomb_GetDisplacementFromLink`, `Bomb_Draw`, and
`Bomb_CheckUndersideSpriteStatus` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla07_Bomb` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule lift/carry branches, normal lift update, Y-to-Z latch, class-2 collision response, bounce velocity reversal, dungeon pit/conveyor/water/spike tile cases, restored Y/dir/priority, sprite/player damage timing, explosion frame advance, carried-bomb release, destructible-door debris check, transmute-to-debris behavior, and draw dispatch. |
| `Ancilla_HandleLiftLogic` and lift-local helpers | `crates/zelda3/src/ancilla.rs` | fixed | Restored C's pickup short-circuit order in the clear-pickup branch: `ancilla_item_to_link` and `link_state_bits` now return before `Ancilla_CheckLinkCollision`, avoiding collision-helper side effects when C would skip that call. The rest matches C's bounce/release label, pickup candidate gating, lift countdown, carried position latch, throw velocities, landing bounces, floor transfer, and speed reset. |
| `AncillaAdd_Bomb` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's allocation and no-bomb clear path, bomb count decrement/HUD refresh, initial state fields, door-debris scratch initialization, direction setup, class-2 initial tile branch, direction-indexed placement tables, and placement SFX. |
| `Bomb_CheckSpriteDamage`, `Bomb_CheckSpriteAndPlayerDamage`, `Bomb_GetDisplacementFromLink` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's staggered sprite scan, floor/state/projectile filters, 48x48 bomb hitbox, sprite recoil projection, player damage gate, link hitbox, displacement bucket, projected knockback, blink/menu guard, Z velocity/incapacitation timer, blink timer, and armor-scaled damage. |
| `Bomb_Draw`, `Bomb_CheckUndersideSpriteStatus` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's adjusted OAM prep, signed Z handling, priority override, bomb frame/r11 selection, low/sort OAM-region choices, tile-attr OAM skip, explosion draw, shadow placement fallback, underside tile animation, sound replacement, carried-bomb shadow suppression, and shadow offset output. |

## 2026-05-29 Ancilla Door Debris Handler Pass

Scope: `Ancilla08_DoorDebris` and `DoorDebris_Draw` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla08_DoorDebris` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's draw-before-timer order, pre-decrement of `ancilla_arr26`, signed rollover gate, arr26 reset to 7, arr25 frame advance, and type clear at frame 4. |
| `DoorDebris_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's adjusted OAM prep call, use of `door_debris_x/y` minus BG2 scroll, `ancilla_arr25 + door_debris_direction * 4` table base, two-entry draw loop, char/flag word split, priority high-byte merge, small-OAM size, and custom-region advance after each entry. |

## 2026-05-29 Ancilla Small Effect Handler Pass

Scope: `Ancilla15_JumpSplash`, `Ancilla16_HitStars`,
`HitStars_UpdateOamBufferPosition`, and `Ancilla17_ShovelDirt` in
`src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla15_JumpSplash` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule-gated aux-timer countdown, item-to-link transition, shared X/Y velocity decay, termination threshold, bunny/swimming deep-water ability check, movement order, mirror X calculation from Link and splash X, two mirrored splash OAM entries, and final center OAM entry size bit. |
| `Ancilla16_HitStars`, `HitStars_UpdateOamBufferPosition` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's arr3 pre-decrement early return, aux-timer/item transition, shared X/Y velocity decay, movement order, mirrored X low-byte replacement, region B/E allocation for step 2, two-entry OAM draw with flag flip, and unsorted OAM wrap to `0x820`/`0xa28`. |
| `Ancilla17_ShovelDirt` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's OAM prep, timer reset to 8, two-frame lifetime, facing-dependent XY table index, signed offsets, two-entry draw loop, consecutive chars, priority merge, and custom-region advance after each entry. |

## 2026-05-29 Ancilla Blast Wall Effect Pass

Scope: `Ancilla32_BlastWallFireball`, `Ancilla33_BlastWallExplosion`,
`AncillaDraw_BlastWallBlast`, and `AncillaDraw_Explosion` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla32_BlastWallFireball` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule-gated item-to-link increment by 2, Y velocity accumulation, Y/X movement order, signed `blastwall_var12` countdown and type clear, sort-dependent OAM region allocation, fireball frame selection from bits 3/2, tile char table, flags, and size. |
| `Ancilla33_BlastWallExplosion` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's active-side var6 countdown, var5 frame advance, fireball spawn for frames 1..8, frame-11 reset, inactive-side trigger via `k ^ 1`, uint8 item-to-link cap check, four-position blast offset update, screen-X sound gate, active-side draw loop over slots 3..0 or 7..4, and final dual-type/custom-spell clear. |
| `AncillaDraw_BlastWallBlast`, `AncillaDraw_Explosion` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's priority override, sort-dependent OAM allocation, frame table lookup through `kBomb_Draw_Tab0/Tab2`, BG2-relative coordinates, do-while style explosion frame loop, skipped `0xff` tiles, signed XY offsets, flag priority/r11 merge, ext-OAM size table, and returned OAM pointer advance. |

## 2026-05-29 Ancilla Ether Spell Handler Pass

Scope: `Ancilla18_EtherSpell`, `EtherSpell_HandleLightningStroke`,
`EtherSpell_HandleOrbPulse`, `EtherSpell_HandleRadialSpin`,
`AncillaDraw_EtherBlitzBall`, `AncillaDraw_EtherBlitzSegment`,
`AncillaDraw_EtherBlitz`, `AncillaDraw_EtherOrb`, and
`AncillaAdd_EtherSpell` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_EtherSpell` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's allocation guard, state initialization, custom spell flag, aux/arr timers, initial Y velocity, Ether radius/timer fields, CHR halfslot request, SFX pan, eight radial seeds, Link-relative Ether coordinate cache, BG2-relative starting Y, adjusted-Y low-byte setup, and initial ancilla placement. |
| `Ancilla18_EtherSpell` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule early return, palette flash cadence for step/nonzero spin counters, step-2 aux timer/item transition into step 3, X velocity growth, general aux timer toggle, dispatch for lightning/orb/radial/hold/release phases, Ether var1 countdown to step 5, and signed X velocity clamp to `0x7f`. |
| `EtherSpell_HandleLightningStroke`, `EtherSpell_HandleOrbPulse`, `EtherSpell_HandleRadialSpin` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's lightning Y movement and segment-count tracking, transition when reaching cached Ether Y, orb pulse arr3/arr25 countdown sequence, medallion sprite damage gate, radial sound cadence, radius update through `ether_var2`, per-orb angle stepping, offscreen completion gate, entrance trigger side effects, player/spin cleanup, speed reset, and palette/HUD restore. |
| Ether draw helpers | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's blitz-ball radial sign handling, BG2-relative positions, tile/flag tables, segment split-pair OAM writes, custom-region advancement, vertical blitz segment loop with alternating chars, optional orb draw in step 1, and four-piece orb layout/flags. |

## 2026-05-29 Ancilla Bombos Spell Handler Pass

Scope: `AncillaAdd_BombosSpell`, `Ancilla19_BombosSpell`,
`BombosSpell_ControlFireColumns`, `BombosSpell_FinishFireColumns`,
`BombosSpell_ControlBlasting`, `AncillaDraw_BombosFireColumn`, and
`AncillaDraw_BombosBlast` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_BombosSpell` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's bank-08 allocation guard, fire-column and blast-array initialization ranges, spell globals, CHR halfslot request, custom spell flag, step/item reset, SFX, generated asset lookup, masked initial blast coordinate, first radial fire-column seed, and initial x/y split-byte writes. |
| `Ancilla19_BombosSpell` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's three-phase dispatch on `bombos_var4`, submodule draw-only fallbacks for fire columns and blasts, reverse draw loops, and active control returns. |
| `BombosSpell_ControlFireColumns`, `BombosSpell_FinishFireColumns` | `crates/zelda3/src/ancilla.rs` | fixed | Restored C's full-ring fallback in the `sb == 9` branch: if no completed fire-column slot is found, the spawn slot remains 9 instead of falling through to slot 0. The rest matches C's arr1/arr2 countdowns, frame-13 skip, ring expansion, radius cap at 207, angle advance, radial projection, screen-X SFX gate, transition to finish phase, finish countdowns, all-complete scan, medallion damage, and blast-phase step reset. |
| `BombosSpell_ControlBlasting` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's reverse active blast scan, arr4 countdown, new-blast slot selection, frame-counter table coordinate generation, BG2-relative storage, SFX pan from blast X, completion scan, player/spin cleanup, speed reset, and delayed `bombos_var2/var3` throttle update. |
| Bombos draw helpers | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's fire-column OAM region allocation, arr2 frame-to-table conversion, three-slot reverse draw, skipped `0xff` chars, split x/y coordinate reads, BG2-relative offsets, custom-region advance, blast frame table, arr3==8 early return, four-slot reverse draw, flags/chars, and OAM size bits. |

## 2026-05-29 Ancilla Quake Spell Handler Pass

Scope: `AncillaAdd_QuakeSpell`, `Ancilla1C_QuakeSpell`,
`QuakeSpell_ShakeScreen`, `QuakeSpell_ControlBolts`,
`AncillaDraw_QuakeInitialBolts`, and `QuakeSpell_SpreadBolts` in
`src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_QuakeSpell` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's allocation guard, step/item reset, CHR halfslot request, SFX, five-entry quake-arr initialization, custom spell flag, timer, Link-relative quake origin, and initial shake amplitude. |
| `Ancilla1C_QuakeSpell`, `QuakeSpell_ShakeScreen`, `QuakeSpell_ControlBolts` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's submodule draw-only branch, shake/update/control/spread dispatch, completion cleanup, medallion damage and rumble calls, spin/player/button/speed cleanup, entrance trigger side effects, BG1 offset reset, shake sign flip, link Y velocity addition, bolt countdowns, var5/var4 transitions, SFX gate, and step update. |
| `AncillaDraw_QuakeInitialBolts`, `QuakeSpell_SpreadBolts` | `crates/zelda3/src/ancilla.rs` | verified | Current Rust matches C's table-offset selection, item-position range lookup, raw OAM writes with C's preserved offscreen x byte behavior, char/flag/size writes, OAM pointer increments, spread timer cadence, 55-frame completion, and custom-region advancement. |

Checks after this pass:

- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Zelda RTL Player Helper Consolidation Pass

Scope: `src/player.c` movement/dash helpers that had local `_minimal` copies in
`crates/zelda3/src/zelda_rtl.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_CancelDash` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Runtime paths in `zelda_rtl.rs` now call the shared `link_cancel_dash` port instead of a local duplicate. |
| `RepelDash` | `crates/zelda3/src/player.rs` | fixed | Restored the C side effects that were missing from Rust: `AncillaAdd_DashTremor(29, 1)` and `Prepare_ApplyRumbleToSprites()` now run before sound/rebound handling. |
| `Sprite_RepelDash`, `LinkApplyTileRebound`, `Flag67WithDirections` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate zelda_rtl copies and routed runtime/test callers through the shared player implementations. |
| `Link_ResetSwimmingState`, `ResetAllAcceleration` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate zelda_rtl copies and routed callers through the shared player implementations, matching the C swim-acceleration pair clears. |
| `Link_HandleVelocityAndSandDrag`, `Link_HandleMovingFloor`, `Link_ApplyMovingFloorVelocity`, `Link_ApplyConveyor` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate zelda_rtl copies and routed callers through the shared player implementations after direct C comparison. |
| `Link_HandleChangeInZVelocity`, `Player_ChangeZ` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate zelda_rtl copies and routed recoil/z-motion callers through the shared player implementations. |

Follow-up note from the 2026-05-29 marker recheck: the old `_minimal` runtime
duplicates are no longer present in `zelda_rtl.rs`; the earlier hit count is
superseded by the whole-repo marker scan below.

Checks after this pass:

- `cargo check -p zelda3`
- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Zelda RTL Moving Animation Pass

Scope: `src/player.c` moving-animation helpers that had local `_minimal` copies
in `crates/zelda3/src/zelda_rtl.rs`, plus a caller in `ending.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleMovingAnimation_FullLongEntry` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/ending.rs` | fixed | Runtime and ending paths now call the shared player implementation instead of the duplicate zelda_rtl copy. |
| `Link_HandleMovingAnimation_StartWithDash` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the duplicate zelda_rtl copy and routed callers through the shared C-name helper. The shared helper also carries the C enhanced temp-bunny bugfix branch that the local minimal copy lacked. |
| `Link_HandleMovingAnimationSwimming`, `Link_HandleMovingAnimation_Dash` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate zelda_rtl copies and routed callers through the shared implementations after direct C comparison. |
| Link animation step helpers | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate local step helpers; animation stepping is now centralized under the shared player port. |

Known limits after this pass: this does not prove the surrounding movement and
collision routines in `zelda_rtl.rs`; it only closes the moving-animation
helper group. The explicit marker scan for `zelda_rtl.rs` is down from 305 to
284 hits, and `ending.rs` is down from 4 to 3 hits.

Checks after this pass:

- `cargo check -p zelda3`
- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Zelda RTL Tile Detect Pass

Scope: `src/player.c` tile-detection, slosh-sound, Byrna-spark, item-tile, and
push-block helpers that had local `_minimal` copies in
`crates/zelda3/src/zelda_rtl.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `TileDetect_MainHandler` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Runtime paths now call the shared player implementation instead of the local minimal copy. The shared implementation includes C side effects missing from the duplicate, including dungeon quadrant flagging, destination room/layer updates, and overworld mirror sword-tile handling. |
| `TileBehavior_HandleItemAndExecute` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the duplicate zelda_rtl copy and routed callers through the shared player implementation. |
| `Link_PermissionForSloshSounds`, `SearchForByrnaSpark` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate local copies and routed tile detection through the shared C-compared helpers. |
| `PushBlock_GetTargetTileFlag`, `PushBlock_AttemptToPushTheBlock` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed duplicate local copies and routed tests/runtime through the shared player implementations. This also removes the minimal copy's index clamping, which did not match the C table indexing. |

Known limits after this pass: `HandleItemTileAction_Dungeon`,
`OverworldToolAndTileInteraction`, and related hammer SFX helpers still have
local minimal copies and need a separate C comparison/port. The explicit marker
scan for `zelda_rtl.rs` is down from 284 to 261 hits.

Checks after this pass:

- `cargo check -p zelda3`
- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Overworld Tool Tile Interaction Pass

Scope: `src/misc.c` `HandleItemTileAction_Overworld`,
`src/dungeon.c` `HandleItemTileAction_Dungeon`, and `src/overworld.c`
`Overworld_ToolAndTileInteraction` / `Overworld_PickHammerSfx`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `HandleItemTileAction_Overworld` | `crates/zelda3/src/misc.rs` | fixed | Outdoor routing now calls the C-name `Overworld_ToolAndTileInteraction`; indoor routing already called the full dungeon helper. |
| `HandleItemTileAction_Dungeon` | `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the unused local minimal copy from `zelda_rtl.rs`. The active dungeon helper includes hammer peg handling, enhanced sword pot breaking, pot item reveal, terrain spawn, and bush poof side effects. |
| `Overworld_ToolAndTileInteraction` | `crates/zelda3/src/overworld.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Replaced the wrapper around the local minimal copy with the C-shaped implementation: shovelables, thick grass, bushes, hammer pegs, secret reveal, map16 memoization/draw, NMI upload flag, terrain spawn, and bush poof are now handled. |
| `Overworld_PickHammerSfx` | `crates/zelda3/src/overworld.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the local duplicate hammer SFX helper; hammer fallback now calls the shared C-name helper using the interacted map16 tile. |

Known limits after this pass: this covers item/tool tile interaction only; other
overworld map16 mutation paths still need their own C comparison. The explicit
marker scan is down to `zelda_rtl.rs` 257 hits and `misc.rs` 1 hit.

Checks after this pass:

- `cargo check -p zelda3`
- `cargo fmt`
- `cargo check -p snes -p zelda3 -p zelda3-bin`
- `git diff --check`

## 2026-05-29 Zelda RTL Velocity/Swim Movement Pass

Scope: `src/player.c` `Link_HandleVelocity`, `Link_MovePosition`,
`HandleSwimStrokeAndSubpixels`, and
`Player_SomethingWithVelocity_TiredOrSwim`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleVelocity` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/ending.rs` | fixed | Runtime and ending callers now use the shared C-name implementation. Removed the local `_minimal` duplicate, including its drifted `link_flag_moving && !link_is_running` branch. |
| `Link_MovePosition` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Runtime tree-pull and movement tests now call the shared method, which snapshots current safe-return coordinates internally like C. The low-level coordinate helpers remain in `zelda_rtl.rs` because the shared player implementation still depends on them. |
| `HandleSwimStrokeAndSubpixels` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the local duplicate and routed tests through the shared implementation. |
| `Player_SomethingWithVelocity_TiredOrSwim` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the local duplicate; the shared method handles subpixel movement, actual velocity derivation, moving-floor application, and sand-drag/page-delta bookkeeping. |

Checks after this pass:

- `cargo check -p zelda3`

## 2026-05-29 Overworld Scroll Renderer Pass

Scope: `src/overworld.c` `OverworldTransitionScrollAndLoadMap`,
`BuildFullStripeDuringTransition_*`, `OverworldHandleMapScroll`,
`CheckForNewlyLoadedMapAreas_*`, `BufferAndBuildMap16Stripes_*`,
`CreateInitialNewScreenMapToScroll`, `Overworld_OperateCameraScroll`,
`OverworldScrollTransition`, `Overworld_DoMapUpdate32x32_B`, and
`Overworld_DrawMap16`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| transition stripe renderer | `crates/zelda3/src/overworld.rs` | verified | The Rust builders emit the same stripe headers, map16-to-map8 tile ordering, source/destination cursor updates, terminators, and NMI subroutine trigger as the C renderer. |
| initial scroll screen renderer | `crates/zelda3/src/overworld.rs` | verified | Big/small north/south/east/west setup mirrors the C map16 source/destination offsets, temporary small-map backup fields, and stripe counts. |
| camera scroll and transition | `crates/zelda3/src/overworld.rs` | verified | Camera boundary accounting, parallax overlay handling, transition counter movement, final camera bounds, area-change flag, and sprite-slot reset match the C path. |
| map16 update/draw upload | `crates/zelda3/src/overworld.rs`, `crates/zelda3/src/ancilla.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed the dead local `overworld_do_map_update32x32_b_minimal` stub and routed the remaining ancilla call to `Overworld_DoMapUpdate32x32_B`, so door/rock map updates use the full C-name VRAM upload path. |

Checks after this pass:

- `cargo check -p zelda3`

Current explicit marker scan after this pass:

| Rust file | Hit count |
|---|---:|
| `crates/zelda3/src/zelda_rtl.rs` | 242 |
| `crates/zelda3/src/sprite_main_draw.rs` | 14 |
| `crates/zelda3/src/sprite.rs` | 12 |
| `crates/zelda3/src/player.rs` | 9 |
| `crates/zelda3/src/dungeon.rs` | 6 |
| `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | 5 |
| `crates/zelda3/src/sprite_main_mothula.rs` | 5 |
| `crates/zelda3/src/messaging.rs` | 4 |
| `crates/zelda3/src/sprite_main_prep.rs` | 4 |
| `crates/zelda3/src/sprite_main_small_bosses.rs` | 3 |
| `crates/zelda3/src/ending.rs` | 2 |
| `crates/zelda3/src/sprite_main_blind.rs` | 2 |
| `crates/zelda3/src/sprite_main_ganon.rs` | 2 |
| `crates/zelda3/src/sprite_main_npcs.rs` | 2 |
| `crates/zelda3/src/sprite_main_world.rs` | 2 |
| `crates/zelda3/src/ancilla.rs` | 1 |
| `crates/zelda3/src/misc.rs` | 1 |
| `crates/zelda3/src/sprite_main_guard.rs` | 1 |
| `crates/zelda3/src/sprite_main_helmasaur_king.rs` | 1 |
| `crates/zelda3/src/sprite_main_hinox_shop.rs` | 1 |

## 2026-05-29 Main/PPU/APU Stub Pass

Scope: `src/main.c`, `snes/ppu.c`, `snes/apu.c` compared against
`crates/zelda3/src/main.rs`, `crates/snes/src/ppu.rs`, and
`crates/snes/src/apu.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| host main helpers | `crates/zelda3/src/main.rs` | fixed-limited | Tightened `LoadAssets` to match C asset signature validation and fallback loading. Remaining SDL/audio/window/save-load event-loop limits are tracked in the Main Runtime Input pass. |
| PPU register/render surface | `crates/snes/src/ppu.rs`, `crates/zelda3/src/zelda_rtl.rs` | superseded | The public frame-rendering API writes back into the caller's pixel buffer like C `ZeldaDrawPpuFrame`. Later PPU renderer passes cover mode 1 backgrounds, sprites, windows, color math, mosaic, mode 7, upsampled mode 7, and the legacy per-pixel fallback, so the old line-renderer warning has been removed from the file header. |
| APU/SPC/DSP execution | `crates/snes/src/apu.rs`, `crates/zelda3/src/audio.rs`, `crates/zelda3/src/spc_player.rs` | fixed | The host-visible APU register/timer surface mirrors `snes/apu.c`, `cycle` calls a Rust `spc_run_opcode` with C reset-vector state, cycle timing, stopped behavior, and an exhaustive `spc_doOpcode` match for all 256 opcodes. The runtime audio path now also routes `SpcPlayer_GenerateSamples` through the C-shaped DSP core and resamples it in `ZeldaRenderAudio` before MSU mixing, matching `src/audio.c`. |

## 2026-05-29 A-Button Interaction Stub Pass

Scope: `src/player.c` A-button interaction cluster compared against
`crates/zelda3/src/player.rs` and the older local copies in
`crates/zelda3/src/zelda_rtl.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_HandleAPress` and new-press gate | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed unused local `link_handle_apress_minimal`, `link_check_new_apress_minimal`, and `can_handle_ground_direction_input_minimal`; runtime already routes through the fuller C-name `link_handle_a_press` path in `player.rs`. |
| A-button basic actions | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Removed local minimal copies for lift/carry/throw, dash, grab, read, open chest, rupee pull, pull object, and statue drag; test probes now call the shared C-name implementations. |
| liftable terrain helper shims | `crates/zelda3/src/player.rs`, `crates/zelda3/src/overworld.rs`, `crates/zelda3/src/sprite.rs` | fixed | Replaced the local shims with the C-name `Overworld_HandleLiftableTiles` and `sprite_spawn_throwable_terrain` paths, so lifted overworld objects now use map16 mutation, secret reveal, throwable sprite property seeding, pickup flags, and SFX behavior from the full ports. |
| ending bat-crash spawn shim | `crates/zelda3/src/ending.rs`, `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Replaced `sprite_spawn_dynamically_minimal` with the C-name `sprite_spawn_dynamically` call in `Sprite_SpawnBatCrashCutscene` and removed the local allocator shim. |

## 2026-05-29 Player Helper Routing Pass

Scope: small remaining `player.rs`/`ending.rs` helper calls compared against
`src/player.c`, `src/load_gfx.c`, and `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkState_Zapped` mosaic calls | `crates/zelda3/src/player.rs`, `crates/zelda3/src/load_gfx.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed `LinkState_Zapped` to C-name `LinkZap_HandleMosaic` and `Player_SetCustomMosaicLevel`; removed the duplicate local mosaic helpers. |
| inter-room transition player reset | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | Inlined the same four C assignments before horizontal and vertical room-transition branches (`some_animation_timer`, `link_state_bits`, `link_picking_throw_state`, `link_grabbing_wall`) and removed the local helper extraction. |
| `CallForDuckIndoors` | `crates/zelda3/src/ending.rs`, `crates/zelda3/src/ancilla.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed the ending wrapper through the fuller ancilla.c port, which uses `Ancilla_Sfx2_Near` and `AncillaAdd_Duck_take_off`; removed the allocation-only shortcut. |
| Byrna spark local helper | `crates/zelda3/src/player.rs` | fixed | Removed unused `ancilla_add_byrna_spark_minimal`; Cane of Byrna already uses the C-shaped init-spark path and `SearchForByrnaSpark` port. |

## 2026-05-29 Sprite Helper Routing Pass

Scope: single-purpose sprite reset/prep helpers compared against `src/sprite.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_DisableAll` | `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/misc.rs` | fixed | Routed `dungeon_reset_sprites` to the C-shaped `sprite_disable_all` implementation and removed the duplicate local `sprite_disable_all_minimal` copy. |
| `SpritePrep_LoadProperties` | `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/ancilla.rs`, `crates/zelda3/src/sprite_main_prep.rs`, `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed remaining callers to `sprite_prep_load_properties`, which performs the C reset plus init-table load; removed the older local partial prep helper. |

## 2026-05-29 Helmasaur King Adapter Routing Pass

Scope: `src/sprite_main.c` Helmasaur King boss collision/draw helper calls that
still used local split-module no-ops despite canonical C-name ports being
available elsewhere in the tree.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Player_SetupActionHitBox` caller in `HelmasaurKing_CheckMaskDamageFromHammer` | `crates/zelda3/src/sprite_main_helmasaur_king.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the boss-local hitbox adapter through the canonical player action hitbox helper instead of leaving Link's action rectangle zeroed, restoring hammer-mask collision behavior. |
| `HelmasaurKing_Draw` helper chain | `crates/zelda3/src/sprite_main_helmasaur_king.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed `Sprite_PrepOamCoordOrDoubleRet`, `KingHelmasaur_OperateTail`, `SpriteDraw_KingHelmasaur_Eyes`, `KingHelmasaurMask`, body, legs, and mouth adapters through the existing C-compared draw helpers instead of local no-ops. This restores tail collision side effects and OAM emission for the split boss module. |

## 2026-05-29 Dungeon NPC Barrier Routing Pass

Scope: dungeon-NPC split-module adapter for the shared sprite barrier helper.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_BehaveAsBarrier` dungeon NPC caller | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the local `_for_dn` adapter through the canonical helper, restoring C's temporary `sprite_flags4` clear, same-layer contact check, movement halt, and flag restore. Remaining dungeon-NPC marker hits are OAM draw/shadow adapters or header text. |

## 2026-05-29 World Sprite Draw Routing Pass

Scope: split world-sprite draw adapters that still carried local deferred draw
comments despite matching C-name draw ports already existing in
`sprite_main_draw.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpriteDraw_LightFountain` world callers | `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed Master Sword light-fountain calls through the canonical draw helper, restoring C's Region C OAM allocation and one-entry `Sprite_DrawMultiple` atlas selection. |
| `FluteAardvark_Draw` world caller | `crates/zelda3/src/sprite_main_world.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the flute aardvark adapter through the canonical deferred-player draw helper and two-entry atlas selection instead of an empty local shim. |

## 2026-05-29 Small-Boss OAM Adapter Routing Pass

Scope: split Trinexx/Vitreous/Yellow Stalfos adapters that still skipped
shared OAM helpers even though C-name ports exist in `sprite.rs` and
`sprite_main_draw.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpriteDraw_TrinexxRockHeadAndBody` caller | `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the small-boss adapter through the existing C-compared Trinexx head/body draw helper instead of a no-op. |
| `Sprite_DrawMultiple`, `Sprite_DrawLargeShadow2`, `SpriteDraw_Shadow`, `SetOamHelper0` small-boss callers | `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Vitreous and Yellow Stalfos now use canonical OAM emission, large-shadow, shadow, and helper0 writes instead of returning only plausible `PrepOamCoordsRet` coordinates. |
| `Sprite_TrinexxD_Draw`, `Sprite_InitializedSegmented` | `crates/zelda3/src/sprite_main_small_bosses.rs` | fixed | Superseded by the Trinexx segmented renderer pass below; final-phase draw now emits body/head OAM from the Moldorm history ring and the phase-transition setup seeds all 128 history entries. |

## 2026-05-29 Trinexx Segmented Renderer Pass

Scope: `src/sprite_main.c` Trinexx final-phase body renderer, segmented-history
initializer, and the rock-head shell transition leading into that phase.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_TrinexxD_Draw` | `crates/zelda3/src/sprite_main_small_bosses.rs`, `crates/zelda3/src/sprite_main_draw.rs`, `crates/zelda3/src/sprite.rs` | fixed | Replaced the scratch-only stub with the C-shaped 24-segment history renderer: head draw, per-segment Moldorm history lookup, Link body-contact damage, OAM pointer advancement, flashing-segment damage check, `Sprite_DrawSingleLarge`, and rock-head segment redraw. |
| `Sprite_InitializedSegmented` | `crates/zelda3/src/sprite_main_small_bosses.rs` | fixed | Local adapter now seeds `moldorm_x/y_lo/hi[0..128]` from Trinexx's current sprite coordinates, matching the shared C initializer used before final phase. |
| `Sprite_CB_TrinexxRockHead` shell transition | `crates/zelda3/src/sprite_main_small_bosses.rs` | fixed | Restored `TM_copy = 0x17`, `TS_copy = 0`, the `sprite_delay_main >= 0xe0` rising-shell movement/collision-mirror branch, and `HIBYTE(dung_floor_y_vel) = 255` on final-phase entry. |

## 2026-05-29 Overworld Scroll Renderer Recheck

Scope: fresh C/Rust drift pass over `src/overworld.c` scroll-renderer paths:
initial screen stripes, transition stripes, newly loaded edge stripes, camera
boundary scrolling, scroll transitions, and the shared map16-to-map8 stripe
emitters.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `CreateInitialNewScreenMapToScroll`, `CreateInitialOWScreenView_*` | `crates/zelda3/src/overworld.rs` | verified | Rechecked big/small north/south/east/west setup against C: source/destination offsets, temporary small-map backup fields, stripe counts, terminator writes, and invalid-direction `submodule_index = 0` behavior match. |
| `OverworldTransitionScrollAndLoadMap`, `BuildFullStripeDuringTransition_*`, `BufferAndBuildMap16Stripes_X/Y` | `crates/zelda3/src/overworld.rs` | verified | Rechecked uvram word output, header words, map16 history buffer fill, map8 quarter ordering, source/destination cursor updates, terminator pair, and `nmi_subroutine_index = 3` trigger against C. |
| `OverworldHandleMapScroll`, `CheckForNewlyLoadedMapAreas_*` | `crates/zelda3/src/overworld.rs` | verified | Rechecked bounds guards, small-map no-stripe behavior, compound north/south plus horizontal direction preservation, final `overworld_screen_transition` copy, and terminator/NMI behavior. |
| `Overworld_OperateCameraScroll`, `OverworldCameraBoundaryCheck`, `OverworldScrollTransition` | `crates/zelda3/src/overworld.rs` | verified | Rechecked z-adjusted link scroll tests, parallax overlay subpixel math, boundary counters/direction bits, transition target comparisons, camera bound reset, area-change flag, transition counter reset, and sprite-slot initialization. |

## 2026-05-29 Messaging Interface Pass

Scope: `src/messaging.c` `Module0E_Interface` and the called rain overlay helper
from `src/overworld.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module0E_Interface` rain overlay call | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/overworld.rs` | fixed | Routed the outdoors branch from an empty local shim to C-name `OverworldOverlay_HandleRain`, matching `messaging.c` and restoring the rain color/window scroll side effects. |
| `Module0E_Interface` tail | `crates/zelda3/src/messaging.rs` | fixed | Removed the local `restore_opening_message_oam_minimal` tail hook; `src/messaging.c` ends after BG scroll-copy writes and has no equivalent OAM patch. |

## 2026-05-29 Mothula OAM Adapter Routing Pass

Scope: Mothula split-module draw adapters for `Mothula_Draw` in
`src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_DrawMultiple` Mothula body draw | `crates/zelda3/src/sprite_main_mothula.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the local Mothula draw-multiple adapter through canonical OAM prep and `sprite_draw_multiple_with_info`, converting `kMothula_Dmd` into the shared `DrawMultipleData` shape instead of returning only cursor coordinates. |
| `SetOamHelper0` Mothula wing strip | `crates/zelda3/src/sprite_main_mothula.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed | Routed the helper0 adapter through the canonical OAM writer, restoring the nine wing-strip OAM entries emitted after the body draw. |

## 2026-05-29 Hinox/Shop Draw Adapter Routing Pass

Scope: split Hinox/shop module adapter for `SpriteDraw_ShopItem` in
`src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpriteDraw_ShopItem` shop item handlers | `crates/zelda3/src/sprite_main_hinox_shop.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed all split-module shop item draw calls through the existing C-compared `sprite_draw_shop_item` helper instead of an empty local shim. |
| `ShopKeeper_RapidTerminateReceiveItem` | `crates/zelda3/src/sprite_main_hinox_shop.rs` | superseded | Deferred at the time of this draw pass; covered by the follow-up Hinox/Shop Receipt Termination Routing Pass below. |

## 2026-05-29 Hinox/Shop Receipt Termination Routing Pass

Scope: split Hinox/shop adapter for `ShopKeeper_RapidTerminateReceiveItem` in
`src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ShopKeeper_RapidTerminateReceiveItem` after shop item receipt | `crates/zelda3/src/sprite_main_hinox_shop.rs`, `crates/zelda3/src/sprite_main_prep.rs` | fixed | Routed the local adapter through the existing C-compared helper, restoring the five-slot scan for item-receipt ancilla type `0x22` and forcing matching `ancilla_aux_timer` values to `1`. |

## 2026-05-29 Blind Draw OAM Routing Pass

Scope: Blind boss draw helpers for `Sprite_Blind_Head` and `Blind_Draw` in
`src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_Blind_Head` OAM head patch | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite.rs` | fixed | After the canonical single-large draw, the split-module adapter now patches `GetOamCurPtr()` char/flags from `kBlindHead_Draw_*` instead of dropping the write. |
| `Blind_Draw` `kBlindPoof_Dmd` / `kBlind_Dmd` body draw | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved the C draw tables into the Blind module and routed the local draw adapter through canonical `sprite_draw_multiple`, restoring body/poof OAM emission. |
| `Blind_Draw` OAM hide/head tile patches | `crates/zelda3/src/sprite_main_blind.rs` | fixed | Implemented the C `oam[6].y = 0xf0` suppression and the post-body head char/flag patch against `GetOamCurPtr() + kBlind_OamIdx[gfx]`. |
| boss poof and audio probe spawning | `crates/zelda3/src/sprite_main_blind.rs` | superseded | Deferred at the time of this draw pass; covered by the follow-up Blind Spawn Adapter Routing Pass below. |

## 2026-05-29 Blind Spawn Adapter Routing Pass

Scope: remaining Blind split-module spawn adapters for `SpawnBossPoof` and
`Sprite_SpawnProbeAlways` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpawnBossPoof` during Blind reveal/death phase | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite_main_prep.rs` | fixed | Routed the Blind-local adapter through the existing C-compared `spawn_boss_poof`, restoring dynamic sprite `0xce` allocation, position offsets, graphics/state seeding, projectile flags, and SFX write. |
| `Sprite_SpawnProbeAlways` during Blind oscillation | `crates/zelda3/src/sprite_main_blind.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the Blind-local adapter through canonical `sprite_spawn_probe_always`, restoring probe allocation, offset placement, direction velocity tables, owner fields, flags, and deflection bits. |

## 2026-05-29 Ganon DrawMultiple Adapter Routing Pass

Scope: split Ganon module adapter for C `Sprite_DrawMultiple` callers, compared
against `PhantomGanon_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `PhantomGanon_Draw` `Sprite_DrawMultiple` call | `crates/zelda3/src/sprite_main_ganon.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the Ganon-local draw-multiple adapter through canonical `sprite_draw_multiple`, converting the existing Phantom Ganon tuple table into `DrawMultipleData` and restoring its eight-entry OAM emission. |
| `Ganon_Draw` body/trident/shadow OAM | `crates/zelda3/src/sprite_main_ganon.rs` | fixed | Superseded by the Ganon body renderer pass below; the deferred body, head-pair, trident, overlay, and shadow OAM paths now emit through canonical or local C-shaped helpers. |

## 2026-05-29 Ganon Body Renderer Pass

Scope: `src/sprite_main.c` `Ganon_Draw` and its direct draw helpers.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Trident_Draw` call from `Ganon_Draw` | `crates/zelda3/src/sprite_main_ganon.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the Ganon-local adapter through the existing canonical `trident_draw` port, preserving the C cur-sprite offset, temporary priority mask, five-entry trident draw, and coordinate restoration. |
| `Ganon_Draw` 12-entry body OAM | `crates/zelda3/src/sprite_main_ganon.rs` | fixed | Ported the 204-entry X/Y/char/flags tables and emitted the same `GetOamCurPtr() + 5` body entries with C's palette/priority flag mask rule. |
| `Ganon_Draw` head-pair patch | `crates/zelda3/src/sprite_main_ganon.rs` | fixed | Ported `kGanon_SprOffs`, `kGanon_Draw_Char2`, and `kGanon_Draw_Flags2`, then patched the same two OAM body entries from `sprite_head_dir` and `sprite_D`. |
| `Ganon_Draw` G==9 overlay and large shadow | `crates/zelda3/src/sprite_main_ganon.rs`, `crates/zelda3/src/sprite.rs` | fixed | Restored the two-entry overlay draw at OAM `0x828/0xa2a` and the three-entry large-shadow draw at OAM `0x9f4/0xa9d`, including the temporary `sprite_oam_flags` and `sprite_obj_prio` mutations. |

## 2026-05-29 Giant Moldorm Segment Renderer Pass

Scope: `src/sprite_main.c` Giant Moldorm draw path and the stale draw-module
deferred marker.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `GiantMoldorm_DrawSegment_C_OrTail` | `crates/zelda3/src/sprite_main_draw.rs`, `crates/zelda3/src/sprite.rs` | verified | Current Rust already matches the C segment-history renderer: it looks back through the Moldorm history ring, writes `cur_sprite_x/y`, applies the same four-way OAM flag rotation, emits `SpriteDraw_SingleLarge`, and restores `sprite_oam_flags`. Removed the now-dead no-op `_for_draw` shim and stale deferred header text. |
| `SpriteDraw_Moldorm_SegmentC`, `SpriteDraw_Moldorm_Tail`, `Moldorm_HandleTail` | `crates/zelda3/src/sprite_main_draw.rs` | verified | Rechecked pointer bumps, `sprite_graphics` writes, tail OAM flag setup, tail damage hitbox swap to `cur_sprite_x/y`, deflection/flags restoration, and final coordinate restoration against C. |

## 2026-05-29 Dungeon NPC Smithy Draw Routing Pass

Scope: split dungeon-NPC adapters for Smithy-family draw and OAM prep helpers
compared against `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SmithyFrog_Draw` | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the local dungeon-NPC adapter through the existing C-compared draw helper, restoring `Sprite_DrawMultiplePlayerDeferred` plus shadow emission for the Smithy frog handler. |
| `SmithySpark_Draw` | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite_main_draw.rs` | fixed | Routed the local adapter through the canonical draw helper, restoring Region B OAM allocation and two-entry spark OAM emission. |
| `Sprite_PrepOamCoord` in Kiki lying-in-wait | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Routed the no-op prep shim through canonical `sprite_prep_oam_coord`, restoring C's pause/out-of-bounds side effects even though the returned coordinates are not consumed by the state machine. |
| remaining dungeon-NPC draw/shadow adapters | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | superseded | Thief was deferred at the time of this pass and is covered below; Uncle, Priest, Kiki, Returning Smithy, and the shared fallback draw/shadow shims remain unverified. |

## 2026-05-29 Dungeon NPC Thief Draw Port Pass

Scope: `Thief_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Thief_Draw` table OAM draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved `kThief_Dmd`, `kThief_DrawChar`, and `kThief_DrawFlags` into the dungeon-NPC module and routed the draw through canonical `sprite_draw_multiple`, preserving the `PrepOamCoordsRet` out-parameter. |
| `Thief_Draw` head patch and shadow | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Implemented the `GetOamCurPtr()` char/flag override and routed the shadow through canonical `sprite_draw_shadow_custom` with the same `info` value C passes to `SpriteDraw_Shadow`. |
| remaining dungeon-NPC generic draw/shadow adapters | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | superseded | Thief was covered by this pass and Returning Smithy is covered below; Uncle, Priest, Kiki, and the shared fallback draw/shadow shims remain unverified. |

## 2026-05-29 Dungeon NPC Returning Smithy Draw Port Pass

Scope: `ReturningSmithy_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ReturningSmithy_Draw` table OAM draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved `kReturningSmithy_Dmd` and `kReturningSmithy_Dma` into the dungeon-NPC module, restored the `dma_var7` write, and routed the one-entry draw through canonical `sprite_draw_multiple_player_deferred`. |
| `ReturningSmithy_Draw` shadow | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved the `PrepOamCoordsRet` out-parameter and routed the shadow through canonical `sprite_draw_shadow_custom`, matching C's `SpriteDraw_Shadow(k, &info)`. |

## 2026-05-29 Dungeon NPC Priest Draw Port Pass

Scope: `Priest_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Priest_Draw` table OAM draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved `kPriest_Dmd` into the dungeon-NPC module and routed the two-entry draw through canonical `sprite_draw_multiple_player_deferred` using the same `sprite_D * 2 + sprite_graphics` index C uses. |
| `Priest_Draw` shadow | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved the `PrepOamCoordsRet` out-parameter and routed the shadow through canonical `sprite_draw_shadow_custom`, replacing the local no-op draw/shadow shims for this path. |
| remaining dungeon-NPC generic draw/shadow adapters | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | superseded | Uncle still called the shared fallback at the time of this pass and is covered below; Kiki still needs its own C comparison pass. |

## 2026-05-29 Dungeon NPC Uncle Draw Port Pass

Scope: `Uncle_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Uncle_Draw` Region B allocation and DMA side variables | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved `Oam_AllocateFromRegionB(0x18)` and moved the C `kUncleDraw_Dma3`/`kUncleDraw_Dma4` tables to module constants used by the same `sprite_D * 2 + sprite_graphics` index. |
| `Uncle_Draw` six-entry body draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved the 48-entry `kUncleDraw_Table` into the dungeon-NPC module and routed the draw through canonical `sprite_draw_multiple` using the C `sprite_D * 12 + sprite_graphics * 6` base. |
| `Uncle_Draw` conditional shadow | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved the `PrepOamCoordsRet` out-parameter and routed the shadow through canonical `sprite_draw_shadow_custom` only when C calls `SpriteDraw_Shadow`, i.e. `sprite_D != 0 && sprite_D != 3`. |
| remaining dungeon-NPC generic draw/shadow adapters | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | superseded | Kiki still called the shared fallback at the time of this pass and is covered below. |

## 2026-05-29 Dungeon NPC Kiki Draw Port Pass

Scope: `Kiki_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Kiki_Draw` normal two-entry draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved `kKiki_Dmd1` and `kKikiDma` into the dungeon-NPC module, restored `dma_var6`/`dma_var7` writes, and routed the `sprite_D < 8` branch through canonical `sprite_draw_multiple` using the C `sprite_D * 2 + sprite_graphics` index. |
| `Kiki_Draw` carried/flee six-entry draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved `kKiki_Dmd2` into the dungeon-NPC module and routed the `sprite_D >= 8` branch through canonical `sprite_draw_multiple` using `sprite_graphics * 6`. |
| `Kiki_Draw` shadow and return flag | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved C's `sprite_pause` shadow gate, passed the same `PrepOamCoordsRet` to `sprite_draw_shadow_custom`, and restored the off-screen return expression `((info.x | info.y) & 0xff00) != 0`. |

## 2026-05-29 Dungeon NPC Smithy Draw Port Pass

Scope: `Smithy_Draw` in `src/sprite_main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Smithy_Draw` table OAM draw | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Moved the 20-entry `kSmithy_Dmd` table into the dungeon-NPC module and routed the two-entry draw through canonical `sprite_draw_multiple_player_deferred` using the C `sprite_graphics * 4 + sprite_D * 2` base. |
| `Smithy_Draw` shadow | `crates/zelda3/src/sprite_main_dungeon_npcs.rs`, `crates/zelda3/src/sprite.rs` | fixed | Preserved the `PrepOamCoordsRet` out-parameter and routed the shadow through canonical `sprite_draw_shadow_custom`, removing the last live call to the local dungeon-NPC draw/shadow no-op shims. |

## 2026-05-29 Dungeon NPC Dead OAM Shim Cleanup

Scope: split dungeon-NPC `_for_dn` helper section after the Priest, Uncle,
Kiki, Returning Smithy, and Smithy draw-routing passes.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sprite_DrawMultiple` / `Sprite_DrawMultiplePlayerDeferred` local no-op fallback | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | removed | Verified by current `rg` scan that `sprite_draw_multiple_for_dn` had no call sites after the draw-routing passes, then removed the dead no-op fallback so future regressions cannot silently drop OAM. |
| `SpriteDraw_Shadow` local no-op fallback | `crates/zelda3/src/sprite_main_dungeon_npcs.rs` | removed | Verified by current `rg` scan that `sprite_draw_shadow_for_dn` had no call sites, then removed the dead no-op fallback and updated the module header to describe only live canonical-helper adapters. |

## Whole-Repo Risk Scan

The first non-signature scan looked for names and comments that explicitly mark
non-1:1 behavior:

```bash
rg -n "_minimal\b|stub|TODO|todo!\(|unimplemented!\(" crates/zelda3/src -g '*.rs' --count-matches
```

Current result: no hits. A broader wording scan for
`stub|TODO|todo!|unimplemented!|deferred|no-op|minimal` still finds legitimate
C helper names and explanatory comments, especially
`Sprite_DrawMultiplePlayerDeferred` and `Sprite_ShowMessageMinimal`; those are
not local shortcut implementations. Direct spot checks also confirmed:

| Rust file | Marker | Verdict | Notes |
|---|---|---|---|
| `crates/zelda3/src/sprite_main_hinox_shop.rs` | `Hinox_ThrowBomb` empty body | verified C-empty | `sprite_main.c:9032` has an empty `Hinox_ThrowBomb` body, so the Rust empty body is intentional parity. |
| `crates/zelda3/src/sprite_main_small_bosses.rs` | `SpritePrep_LoadProperties` adapter wording | verified canonical helper call | `Vitreous_SpawnSmallerEyes` calls `SpritePrep_LoadProperties(j)` in `sprite_main.c:18164`; the Rust adapter calls the shared `sprite_prep_load_properties` port. |

## Remaining Manual Coverage

Manual whole-repo parity audit is not complete. The next pass should continue
module by module from `docs/PORTING_MAP.md` and the C source files, updating this
ledger with reviewed function ranges and fixes.

## 2026-05-29 Ancilla Magic Powder Dust Pass

Scope: `AncillaAdd_MagicPowder`, `Ancilla1A_PowderDust`, `Ancilla_MagicPowder_Draw`,
and `Powder_ApplyDamageToSprites` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_MagicPowder` setup | `crates/zelda3/src/ancilla.rs` | verified | Matches C slot allocation, link-facing offsets, initial tile-collision probe, `byte_7E0333` mirror, mushroom active-item early clear, SFX pan, and final effect position. |
| `Ancilla1A_PowderDust` state progression | `crates/zelda3/src/ancilla.rs` | verified | Matches C's submodule gate, damage application cadence, signed decrement of `ancilla_aux_timer`, item-to-link frame advance through `kMagicPowder_Tab0`, slot clear at frame 9, OAM allocation, and draw call. |
| `Ancilla_MagicPowder_Draw` four-entry OAM renderer | `crates/zelda3/src/ancilla.rs` | verified | Tables and indexing match C; Rust preserves `Ancilla_PrepOamCoord`, direct OAM cursor iteration, char table, flag priority masking with `HIBYTE(oam_priority_value)`, and size 0 entries. |
| `Powder_ApplyDamageToSprites` collision/damage path | `crates/zelda3/src/ancilla.rs` | verified | Matches C frame/sprite-state/bump gates, hitbox overlap path, preset damage fallback, mushroom/bat special-case checks, head-dir write, and poof garnish spawn. |

## 2026-05-29 Ancilla Hookshot Handler Pass

Scope: `AncillaAdd_Hookshot`, `Ancilla1F_Hookshot`, `Hookshot_CheckTileCollision`,
`Hookshot_CheckProximityToLink`, `Hookshot_ShouldIEvenBotherWithTiles`, and
`AncillaAdd_HookshotWallClink` in `src/player.c`, `src/ancilla.c`, and
`src/tile_detect.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_Hookshot` setup | `crates/zelda3/src/player.rs` | verified | Matches C allocation, timers/state reset, hookshot globals, direction-derived velocity tables, and initial Link-relative position. |
| `Ancilla1F_Hookshot` movement/retract state | `crates/zelda3/src/ancilla.rs` | verified | Matches C timer/SFX cadence, global drag gate, forward/retract transitions, sprite-collision bounce, hard range cap, and hookshot latch activation/short-shot clear. |
| overworld ledge-contact `r0` handling | `crates/zelda3/src/ancilla.rs` | fixed | Split the C contact gate from the C `r0` value so outdoors contact decrements `ancilla_G` while preserving `r0 == 0`; the previous Rust port forced `r0 = 1`, changing the later `ancilla_K && ((r0 & 3) || ...)` branch. |
| `Hookshot_CheckTileCollision` layer setup | `crates/zelda3/src/tile_detect.rs` | verified | Matches C room/floor backup, `ancilla_arr1` layer flip, pit reset, dual-layer collision for collision header 2, single-layer collision, and state restore. |
| Hookshot head/chain OAM renderer | `crates/zelda3/src/ancilla.rs` | verified | Matches C head tables, priority override when latched, chain length/r10 calculation, direction offsets, proximity-to-Link suppression, frame-counter chain flip, and size 0 OAM entries. |
| helper checks and wall clink | `crates/zelda3/src/ancilla.rs` | verified | `Hookshot_CheckProximityToLink`, `Hookshot_ShouldIEvenBotherWithTiles`, and `AncillaAdd_HookshotWallClink` match the C coordinate math, bounds checks, and clink spawn offsets. |

## 2026-05-29 Ancilla Blanket and Snore Pass

Scope: `Ancilla20_Blanket` and `Ancilla21_Snore` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla20_Blanket` renderer | `crates/zelda3/src/ancilla.rs` | verified | Matches C OAM coordinate prep, Region B/A allocation based on `link_pose_during_opening`, table index base, four-entry OAM layout, 16-pixel x stride, second-row wrap, priority bits, and size 2 entries. |
| `Ancilla21_Snore` motion and draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C aux-timer animation advance, x-velocity oscillation via `ancilla_step`, movement calls, y-position self-clear against `link_y_coord - 24`, DMA table write, and single size 0 OAM entry. |

## 2026-05-29 Ancilla Item Receipt and Link Poof Pass

Scope: `Ancilla22_ItemReceipt`, `Ancilla_ReceiveItem_Draw`,
`ItemReceipt_TransmuteToRisingCrystal`, `Ancilla_AddRupees`,
`AncillaAdd_CapePoof`, `Ancilla23_LinkPoof`, and `MorphPoof_Draw` in
`src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla22_ItemReceipt` state labels | `crates/zelda3/src/ancilla.rs` | verified | Rust's split helpers preserve C's immobilization/submodule gates, `flag_unk1` increment, aux-timer finish/message/move branches, follower trigger, item finish side effects, boss-exit condition, and immobilization clear rule. |
| item message and rupee handling | `crates/zelda3/src/ancilla.rs` | verified | `ancilla22_item_receipt_show_message` and `ancilla_add_rupees` match C's exceptional room skips, pendant-dependent message table, heart-piece message table, ambient sound for message `0x70`, rupee item ranges, and goal increments. |
| item draw/update and crystal transmute | `crates/zelda3/src/ancilla.rs` | verified | Matches C's crystal sparkle/music handoff, Bottle/Mushroom frame table update, rupee animated tile decode, adjusted OAM coordinate prep, `Ancilla_ReceiveItem_Draw`, and `ItemReceipt_TransmuteToRisingCrystal` velocity/subpixel reset. |
| `Ancilla_ReceiveItem_Draw` OAM helper | `crates/zelda3/src/ancilla.rs` | verified | Matches C OAM cursor use, dynamic flag fallback through `ancilla_arr4`, `kReceiveItem_Tab1` ext selection, lower tile emission for ext 0, and returned cursor. |
| `AncillaAdd_CapePoof` setup | `crates/zelda3/src/ancilla.rs` | verified | Matches C allocation, transform/cant-turn flags, direction reset, frame/timer initialization, and Link-relative poof position. |
| `Ancilla23_LinkPoof` and `MorphPoof_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Matches C timer frame advance, transform cleanup, bunny mirror state/palette selection, sort-sprites high-floor OAM cursor override, morph tables, ext-2 early break, and priority flag composition. |

## 2026-05-29 Ancilla 0x24-0x28 Local Effects Pass

Scope: `Ancilla24_Gravestone`, `Ancilla_Unused_25`,
`Ancilla26_SwordSwingSparkle`, `Ancilla27_Duck`, `Ancilla28_WishPondItem`,
`WishPondItem_Draw`, and `AncillaAdd_GraveStone` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla24_Gravestone` draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C adjusted OAM coordinate prep, Region B allocation, four 16x16 OAM entries, char/flag tables, x stride, second-row wrap, and priority/size values. |
| `AncillaAdd_GraveStone` setup/map update | `crates/zelda3/src/ancilla.rs` | verified | Matches C row snap from Link Y, row candidate scan, run/non-run gate for grave index 13 versus other graves, big-rock/door counter writes, save-event side effect, debris address bytes, map update, SFX selection, drag state, hookshot flag, and final ancilla position. |
| `Ancilla_Unused_25` | `crates/zelda3/src/ancilla.rs` | verified | Rust panic matches C `assert(0)` for the unused dispatch slot. |
| `Ancilla26_SwordSwingSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Matches C timer decrement, frame advance/clear at frame 4, Link-relative XY reset, frame/direction table index, skipped `0xff` chars, and three-entry OAM emission. |
| `Ancilla27_Duck` behavior and draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C waiting-position update, periodic SFX, z/move updates, pickup/drop state transitions, follower/tagalong cleanup, Link immobilization/visibility flags, bird DMA animation, OAM body/shadows, and exit-module handoff at screen edge. |
| `Ancilla28_WishPondItem` and `WishPondItem_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Matches C OAM allocation, thrown-item movement, z-velocity/splash transmute branch, generated-item offset, draw through `Ancilla_ReceiveItem_Draw`, and conditional shadow parameters. |

## 2026-05-29 Ancilla 0x29-0x2B Milestone and Spin Spark Pass

Scope: `Ancilla29_MilestoneItemReceipt`, `Ancilla2A_SpinAttackSparkleA`,
`SpinAttackSparkleA_TransmuteToNextSpark`,
`Ancilla2B_SpinAttackSparkleB`, `SpinAttackSparkleB_Closer`,
`SpinSpark_Draw`, `Sparkle_PrepOamFromRadial`, `Ancilla_GetRadialProjection`,
and `AddSwordBeam` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla29_MilestoneItemReceipt` | `crates/zelda3/src/ancilla.rs` | verified | Matches C save-bit gates, delayed animated-tile decode, crystal palette side effects, sparkle addition, Link collision pickup handoff, rising z motion, receive-item OAM draw, aux-timer shadow animation, and room-6 shadow variant. |
| `SpinSpark_Draw` | `crates/zelda3/src/ancilla.rs` | verified | Matches C table contents, `(ancilla_item_to_link + offs) * 4` indexing, skipped `0xff` chars, coordinate offsets, priority mask, and size 0 entries. |
| `Ancilla2A_SpinAttackSparkleA` | `crates/zelda3/src/ancilla.rs` | verified | Matches C submodule/timer gate, initial spark timer table, sword-beam branch when `ancilla_step` is set, transmute branch otherwise, early return before frame 1, and draw offset `-1`. |
| spin sparkle transmute to 0x2B | `crates/zelda3/src/ancilla.rs` | verified | Matches C swordbeam angular table initialization, aux/item/arr/step state, temp center coordinates, direction-specific XY offsets, and immediate call into the 0x2B handler. |
| `Ancilla2B_SpinAttackSparkleB` and closer | `crates/zelda3/src/ancilla.rs` | verified | Matches C radial contraction, close-state transition at item counter below 13, step selection from counter values, aux-timer flag flip, radial OAM loop, secondary sparkle update, ext-OAM override when counter is 7, and closer draw offset `4`. |
| radial projection and sword beam helper | `crates/zelda3/src/ancilla.rs` | verified | `Ancilla_GetRadialProjection`, `Sparkle_PrepOamFromRadial`, and `AddSwordBeam` preserve C lookup tables, rounding math, signed radial offsets, sword-beam velocities, tile check, SFX, timer, and OAM sprite count setup. |

## 2026-05-29 Ancilla Somaria Block Cluster Pass

Scope: `AncillaAdd_SomariaBlock`, `SomariaBlock_CheckForTransitTile`,
`Ancilla2C_SomariaBlock`, `AncillaDraw_SomariaBlock`,
`SomariaBlock_CheckForSwitch`, `SomariaBlock_FizzleAway`,
`Ancilla2D_SomariaBlockFizz`, `Ancilla2E_SomariaBlockFission`,
`SomariaBlock_SpawnBullets`, `SomarianBlock_CheckEmpty`,
`AncillaAdd_SomariaPlatformPoof`, `Ancilla39_SomariaPlatformPoof`, and
`AncillaAdd_ExplodingSomariaBlock` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| Somaria block add/setup | `crates/zelda3/src/ancilla.rs` | verified | Matches C bank-08 allocation, replacement/exploding existing block path, drag-speed cleanup, initial state fields, initial collision fallback XY, facing-specific XY, and transit-tile probe. |
| transit/platform poof helpers | `crates/zelda3/src/ancilla.rs` | verified | `SomariaBlock_CheckForTransitTile`, `AncillaAdd_SomariaPlatformPoof`, and `Ancilla39_SomariaPlatformPoof` match C's targeted tile checks, `0xb6`/`0xbc` handoff, platform sprite cleanup/spawn, BG2 attribute direction selection, and fallback block draw. |
| `Ancilla2C_SomariaBlock` main state | `crates/zelda3/src/ancilla.rs` | verified | Matches C `ancilla_G` throttle, lift/carry handling, indoor switch/transit checks, class-2 collision loopback for tile `0x26`, floor/layer behavior, hazard/splash/conveyor branches, sprite-collision fizzle counter, and restore of Y/dir/objprio before draw. |
| Somaria block draw and empty check | `crates/zelda3/src/ancilla.rs` | verified | Matches C OAM region override cases, adjusted coordinates, z/priority rule, `ancilla_arr1 * 4` table base, safe OAM writes, `SomarianBlock_CheckEmpty` ext-OAM scan, pickup clear, and lifted Link-state reset. |
| switch coverage and fizzle away | `crates/zelda3/src/ancilla.rs` | verified | Matches C switch-cover targeted checks, `ancilla_arr24` count, switch flag reset/increment semantics, speed cleanup, 0x2D transmute fields, pickup clear, and immediate fizz draw call. |
| `Ancilla2D_SomariaBlockFizz` | `crates/zelda3/src/ancilla.rs` | verified | Matches C timer/frame progression, clear at frame 3, adjusted OAM coordinates, `0xff` z normalization, two-entry table indexing, skipped `0xff` char, and priority masking. |
| `Ancilla2E_SomariaBlockFission` and bullets | `crates/zelda3/src/ancilla.rs` | verified | Matches C fission timer/frame progression, clear and bullet spawn at frame 2, carried-Link z addition, eight-entry divide draw tables, and `SomariaBlock_SpawnBullets` allocation/state/velocity/floor setup plus `tmp_counter = 0xff`. |

## 2026-05-29 Ancilla 0x2F-0x31 Lamp and Byrna Pass

Scope: `AncillaAdd_LampFlame`, `Ancilla2F_LampFlame`,
`Ancilla30_ByrnaWindupSpark`, `ByrnaWindupSpark_TransmuteToNormal`,
`Ancilla31_ByrnaSpark`, and the Byrna kill path in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `AncillaAdd_LampFlame` setup | `crates/zelda3/src/ancilla.rs` | verified | Matches C allocation, item/aux/timer initialization, facing-derived direction, Link-relative flame offsets, and SFX pan write. |
| `Ancilla2F_LampFlame` draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C adjusted OAM coordinate prep, timer-zero clear, `(timer & 0xf8) >> 1` table base, four-entry group loop, skipped `0xff` chars, priority bits, and size 0 entries. |
| `Ancilla30_ByrnaWindupSpark` | `crates/zelda3/src/ancilla.rs` | verified | Matches C submodule/timer frame advance, transmute at frame 17, initial hidden frame, player-handler timer adjustment via `ancilla_arr3`, Link-relative position tables, frame-table draw selection, and OAM priority masking. |
| Byrna windup transmute | `crates/zelda3/src/ancilla.rs` | verified | Matches C type switch to 0x31, facing-derived swordbeam angle table initialization, aux/G/item/arr/step/L/timer state, `swordbeam_var2`, SFX, and immediate call into `Ancilla31_ByrnaSpark`. |
| `Ancilla31_ByrnaSpark` sustain/draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C current-item kill gate, sprite-damage disable, magic consumption cadence, `ancilla_G` magic drain period, filtered Y-button kill, step growth, flag flicker via `ancilla_arr1`, Link-z adjusted radial center, timer/SFX refresh, radial OAM loop, per-spark XY/collision probe, and direction reset. |
| Byrna kill path | `crates/zelda3/src/ancilla.rs` | verified | Matches C cleanup of `link_disable_sprite_damage`, ancilla type, and `link_give_damage`. |

## 2026-05-29 Ancilla 0x34-0x38 Cutscene and Item Effects Pass

Scope: `Ancilla34_SkullWoodsFire`, `Ancilla35_MasterSwordReceipt`,
`Ancilla36_Flute`, `AncillaAdd_ExplodingWeatherVane`,
`Ancilla37_WeathervaneExplosion`,
`AncillaDraw_WeathervaneExplosionWoodDebris`,
`AncillaAdd_CutsceneDuck`, and `Ancilla38_CutsceneDuck` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla34_SkullWoodsFire` | `crates/zelda3/src/ancilla.rs` | verified | Matches C aux-timer flame-column animation, four falling fire lanes, sound-trigger thresholds, `var0 == 128` inactive lanes, primary and secondary draw tables, extra tile for ext 0, global flame completion clear, and final skull-fire burst gate. |
| `Ancilla35_MasterSwordReceipt` | `crates/zelda3/src/ancilla.rs` | verified | Matches C timer-zero clear, signed aux decrement frame cycle, item frame wrap `2 -> 0`, early return for frame 0, OAM coordinate prep, four-entry ceremony tables, priority mask, and size 0 entries. |
| `Ancilla36_Flute` | `crates/zelda3/src/ancilla.rs` | verified | Matches C z-velocity bounce table, x/z movement, z reset at negative/high values, pickup collision gate with hookshot/aux state checks, item receipt handoff, adjusted OAM draw, and offscreen clear. |
| weathervane explosion setup | `crates/zelda3/src/ancilla.rs` | verified | `AncillaAdd_ExplodingWeatherVane` matches C allocation, timers/state, music/ambient sound, `weathervane_var1/var2`, and initialization of all 12 debris arrays. |
| `Ancilla37_WeathervaneExplosion` and debris draw | `crates/zelda3/src/ancilla.rs` | verified | Matches C var2/`ancilla_G` throttles, first SFX, delayed weathervane alteration and cutscene duck spawn, per-piece frame flip, temporary ancilla state load, movement, z termination to `0xff`, debris OAM offset via `weathervane_var14`, and final clear after all pieces finish. |
| cutscene duck setup and flight | `crates/zelda3/src/ancilla.rs` | verified | `AncillaAdd_CutsceneDuck` and `Ancilla38_CutsceneDuck` match C presence gate, initial state/position, periodic SFX, wing animation, initial bob countdown, horizontal velocity orbit, z velocity derivation, bird DMA/OAM/shadow draw, and flute item unlock when exiting screen. |

## 2026-05-29 Ancilla 0x3A-0x42 Tail Effects Pass

Scope: `Ancilla3A_BigBombExplosion`, `Ancilla3B_SwordUpSparkle`,
`Ancilla3C_SpinAttackChargeSparkle`, `Ancilla3D_ItemSplash`,
`ObjectSplash_Draw`, `Ancilla_TransmuteToSplash`,
`ItemReceipt_TransmuteToRisingCrystal`, `Ancilla_RisingCrystal`,
`Ancilla3F_BushPoof`, `Ancilla40_DwarfPoof`,
`Ancilla41_WaterfallSplash`, `Ancilla42_HappinessPondRupees`,
`AddHappinessPondRupees`, and setup helpers through
`AncillaAdd_WaterfallSplash` in `src/ancilla.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Ancilla3A_BigBombExplosion` | `crates/zelda3/src/ancilla.rs` | fixed | Matches C frame/timer progression, SFX, clear timing, destructible-tile branch, and follower-indicator handling. Fixed the OAM cursor read so each visible explosion chunk uses `OAM_CUR_PTR` after `Ancilla_AllocateOamFromRegion_A_or_D_or_F`, matching C's per-chunk allocation. |
| `Ancilla3B_SwordUpSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Matches C aux-timer gate, signed `ancilla_arr3` decrement/reset, item-frame advance and clear at frame 4, Link-relative coordinates, skipped `0xff` sprites, and priority flag composition. |
| `Ancilla3C_SpinAttackChargeSparkle` | `crates/zelda3/src/ancilla.rs` | verified | Matches C submodule/timer gate, timer reload, frame advance, clear at frame 3, OAM allocation call, single-entry draw table, and priority/size writes. |
| splash transmute and draw | `crates/zelda3/src/ancilla.rs` | verified | `Ancilla_TransmuteToSplash`, `Ancilla3D_ItemSplash`, and `ObjectSplash_Draw` match C type/timer/XY/SFX mutation, immediate splash call, timer-frame clear path, OAM allocation, skipped entries, char/flag/ext tables, and two-entry frame draw. |
| rising crystal handoff | `crates/zelda3/src/ancilla.rs` | verified | `ItemReceipt_TransmuteToRisingCrystal` and `Ancilla_RisingCrystal` match C type/velocity reset, sparkle cadence, Y velocity clamp, BG2 scroll stop threshold, crystal save bit, module/submodule handoff, palette-buffer clear, and receive-item OAM draw. |
| poof effects | `crates/zelda3/src/ancilla.rs` | verified | `Ancilla3F_BushPoof`, `Ancilla40_DwarfPoof`, `AncillaAdd_BushPoof`, and `AncillaAdd_DwarfPoof` match C timer/aux frame stepping, clear side effects, setup coordinates, SFX selection, floor/state fields, OAM region C allocation, and morph-poof draw tables. |
| `Ancilla41_WaterfallSplash` | `crates/zelda3/src/ancilla.rs` | fixed | Matches C entrance-trigger gate, ripple flag, Link animation step decrement, timer/frame cycle, indoor Y override, Link-relative XY sync, z-adjusted draw, and setup helper. Fixed periodic SFX to call `ancilla_sfx2_near(0x1c)`, matching C `Ancilla_Sfx2_Near(0x1c)`. |
| `Ancilla42_HappinessPondRupees` | `crates/zelda3/src/ancilla.rs` | verified | Matches C Link throw-state reset, active-rupee loops, step-2 clear rule, per-rupee state load/save, splash transition, movement/z threshold, fallback draw, and final clear when all rupees are inactive. |
| `AddHappinessPondRupees` | `crates/zelda3/src/ancilla.rs` | verified | Matches C allocation, Link-panned SFX, receive-item gfx value, Link state reset, rupee count memset, start/end/velocity table selection, active rupee initialization, and descending slot fill. |

## 2026-05-29 Overworld Scroll Renderer Pass

Scope: `TriggerAndFinishMapLoadStripe_X/Y`,
`CreateInitialNewScreenMapToScroll`, the big/small initial screen-view helpers,
`OverworldTransitionScrollAndLoadMap`,
`BuildFullStripeDuringTransition_*`, `OverworldHandleMapScroll`,
`CheckForNewlyLoadedMapAreas_*`, `BufferAndBuildMap16Stripes_X/Y`,
`Overworld_DoMapUpdate32x32*`, and the PPU BG scroll/tilemap renderer against
`src/overworld.c` and `snes/ppu.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| initial scroll stripe setup | `crates/zelda3/src/overworld.rs` | verified | `TriggerAndFinishMapLoadStripe_X/Y`, `CreateInitialNewScreenMapToScroll`, and all big/small initial view helpers match C direction selection, UVRAM header words, repeated stripe build counts, source/destination/var updates, small-map saved-state fields, and final `0xffff` terminator writes. |
| transition stripe builders | `crates/zelda3/src/overworld.rs` | verified | `OverworldTransitionScrollAndLoadMap` and `BuildFullStripeDuringTransition_*` match C dispatch, `0x0080`/`0x8040` headers, X/Y stripe construction, source offset movement, destination/var wrap masks, double terminator, and NMI trigger only when data was emitted. |
| active map-scroll loader | `crates/zelda3/src/overworld.rs` | verified | `OverworldHandleMapScroll` and `CheckForNewlyLoadedMapAreas_*` match C edge guards, small-map no-upload behavior, direction-bit clearing/masking for diagonal transitions, source/destination/var updates, UVRAM terminators, NMI trigger, and `overworld_screen_transition` update. |
| map16-to-map8 stripe renderer | `crates/zelda3/src/overworld.rs` | verified | `BufferAndBuildMap16Stripes_X/Y` match C draw-strip offsets, temporary replacement-tile ring fill, out-of-range tile zeroing, map16-to-map8 table indexing, VRAM row/column destination bases, quadrant word ordering, and final destination increments. |
| 32x32 map updates | `crates/zelda3/src/overworld.rs` | verified | `Overworld_DoMapUpdate32x32`, conditional update, and forced-B wrapper match C door animation tile table use, memorized tile address/value writes, persistent map16 draw calls, upload terminator, memorized-tile count increment, animation-step increment rule, NMI flag, and door counter reset/increment behavior. |
| PPU BG scroll/tilemap renderer | `crates/snes/src/ppu.rs` | verified | BG scroll register writes and tilemap lookup match C `ppu_write` and `PpuDrawBackground_*` formulas for tilemap base, wider/higher map selection, h/v scroll composition, tile row flip, char data address, palette/prio shifts, mosaic source rows, and mode-1 4bpp/2bpp layer draw behavior. |

## 2026-05-29 Main Runtime Helper Pass

Scope: render/input/audio/config helpers in `crates/zelda3/src/main.rs`
against `src/main.c`. This pass intentionally does not mark the desktop host
runtime complete; it records both fixed helper parity and remaining non-1:1
host-shell gaps.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `RenderDigit` / `RenderNumber` | `crates/zelda3/src/main.rs` | fixed | `RenderDigit` already matched C font bits, small/big scaling, pixel color writes, and digit table. Fixed `RenderNumber` to include C's first dark shadow pass at `(pitch + i + 4) << big` before the white foreground pass. |
| command/input mapping | `crates/zelda3/src/main.rs` | verified | `HandleCommand`, `HandleInput`, SDL button remap, gamepad modifier tracking, trigger hysteresis, volume clamp, and analog-stick segment mapping match the C control flow and constants for the non-SDL side effects implemented in Rust. |
| link graphics and asset parsing | `crates/zelda3/src/main.rs` / `crates/zelda3/src/zelda_rtl.rs` | verified | ZSPR parsing, pixel/palette bounds checks, Link graphics asset 57 replacement, armor/glove palette asset 81 replacement, asset-pack signature/count validation, aligned asset extraction, `SwitchDirectory`, and `FindInAssetArray` match C behavior. |
| `DrawPpuFrameWithPerf` | `crates/zelda3/src/main.rs` | partial | Frame draw path calls into `zelda_draw_ppu_frame` with the selected render flags and now renders the FPS number with the same shadow/foreground helper. It does not yet reproduce C's performance-counter rolling average or title-bar display-perf mode. |
| `AudioCallback` / `audio_callback_for_game` | `crates/zelda3/src/main.rs` | fixed-limited | The game-backed helper now matches C's audio callback shape for the portable pieces: it loops until the requested byte count is filled, renders `zelda_render_audio` blocks into an internal buffer, applies the SDL mixer-volume scalar, advances the buffer cursor, and calls `zelda_discard_unused_audio_frames` after servicing the request. The raw `audio_callback` without a `ZeldaState` still zeros the stream because this crate shim has no global game pointer. |
| desktop `main` / host loop | `crates/zelda3/src/main.rs`, `zelda3-bin/src/main.rs`, `crates/platform/src/lib.rs` | superseded | The C `main.c` SDL loop is not implemented inside the library shim; the live playable host is `zelda3-bin` using `NativeFrontend` (`winit`/`pixels`/`cpal`). Host-loop parity is tracked by the later native-host and snes9x-oracle sections rather than this legacy SDL-shell row. |

## 2026-05-29 APU Shell and DSP Register Pass

Scope: `apu_reset`, `apu_cycle`, `apu_cpuRead`, `apu_cpuWrite`,
SNES/APU communication ports, `dsp_reset`, `dsp_read`, `dsp_write`,
`dsp_getSamples`, and the SPC run/flag helpers against `snes/apu.c`,
`snes/dsp.c`, and `snes/spc.c`. This pass does not claim every SPC opcode arm
has been audited; the opcode table remains a separate large surface.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| APU reset/cycle shell | `crates/snes/src/apu.rs` | verified | Matches C ROM-readable-before-SPC-reset ordering, SPC reset-vector read, DSP reset, RAM clear, DSP address/cycle clear, in/out port clear, timer reset, `cpuCyclesLeft = 7`, opcode-run cadence, DSP cycle every 32 APU cycles, and timer divider/counter behavior. |
| APU CPU read/write registers | `crates/snes/src/apu.rs` | verified | Matches C read zeros for control/timer targets, DSP address/data reads, port reads, timer counter read-and-clear, boot ROM visibility at `0xffc0..`, control-register timer enable/reset behavior, port clears, ROM visibility bit, DSP write history cap, out/in port writes, timer target writes, and final backing `ram[adr] = val` mirror after writes. |
| SNES communication ports | `crates/snes/src/apu.rs` | verified | `read_snes_port` and `write_snes_port` match the C-side cross-CPU port behavior by exposing APU `outPorts[0..3]` to the SNES side and writing SNES values into APU `inPorts[0..3]`. |
| DSP reset/register writes | `crates/snes/src/apu.rs` | verified | `dsp_reset`, `dsp_read`, and `dsp_write` match C RAM reset with `ENDX = 0xff`, channel/default state, master/echo/noise/FIR fields, pitch/source/ADSR/gain register decoding, KON/KOF sample restart and release behavior, FLG/ENDX semantics, echo/noise/pitch/dir/ESA/EDL/FIR writes, and final DSP register RAM mirror. |
| DSP cycle/sample output | `crates/snes/src/apu.rs` | verified | Matches C per-channel mix and clipping, master-volume scaling, echo FIR buffering, echo feedback/writeback, mute handling, noise LFSR update, BRR decode/filter/loop/end handling, ADSR/gain state transitions, Gaussian interpolation, 534 stereo samples per frame, mono/stereo resampling, and sample-offset reset. |
| SPC run and status helpers | `crates/snes/src/apu.rs` | partial | `spc_runOpcode`, opcode fetch, word fetch, stack push/pull, flag pack/unpack, and Z/N helpers match C structure. Full 256-opcode behavioral parity was not audited in this pass and remains pending. |

## 2026-05-29 SPC Opcode Table 0x00-0x3F Pass

Scope: SPC opcode arms `0x00..=0x3f` in `spc_doOpcode`, their addressing
helpers, and the ALU/memory helpers they exercise against `snes/spc.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| addressing helpers used by `0x00..=0x3f` | `crates/snes/src/apu.rs` | verified | `dp`, `abs`, `imm`, `ind`, `idx`, `dpx`, `abx`, `aby`, `idy`, `dp-word`, `dp/dp`, `dp/imm`, `ind/ind`, and `abs.bit` match C PC increments, direct-page flag application, 8-bit pointer wrapping, 16-bit address wrapping, operand ordering, and abs-bit split. |
| branch, stack, and flag helpers | `crates/snes/src/apu.rs` | verified | `spc_do_branch`, push byte/word use, `spc_set_zn`, and `spc_set_zn_word` match C taken-branch cycle increment, signed relative PC update, stack-side call ordering, and 8-bit/16-bit Z/N flag semantics. |
| opcodes `0x00..=0x1f` | `crates/snes/src/apu.rs` | verified | Matches C `NOP`, `TCALL`, `SET1`, `CLR1`, `BBS`, `BBC`, OR/OR1 forms, ASL forms, `PUSHP`, `TSET1`, `BRK`, `BPL`, `DECW`, `ASLA`, `DECX`, `CMPX abs`, and `JMP (abs+X)` behavior including read/write order and wrapping arithmetic. |
| opcodes `0x20..=0x3f` | `crates/snes/src/apu.rs` | verified | Matches C `CLRP`, AND/AND1 forms, `OR1N`, ROL forms, `PUSHA`, `CBNE`, `BRA`, `BMI`, `INCW`, `ROLA`, `INCX`, `CMPX dp`, and `CALL abs` behavior including boolean carry updates, DP/absolute addressing, 16-bit word flag updates, and call stack ordering. |
| helper operations used by this range | `crates/snes/src/apu.rs` | verified | `spc_or`, `spc_orm`, `spc_and`, `spc_andm`, `spc_asl`, `spc_rol`, and `spc_cmp_x` match C accumulator/memory mutation, memory writeback, carry assignment, compare carry, and Z/N updates. |

## 2026-05-29 SPC Opcode Table 0x40-0x7F Pass

Scope: SPC opcode arms `0x40..=0x7f` in `spc_doOpcode`, stack/flag helpers
used by return opcodes, and the EOR/CMP/LSR/ROR helper operations against
`snes/spc.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| opcodes `0x40..=0x5f` | `crates/snes/src/apu.rs` | verified | Matches C `SETP`, EOR/EOR1 forms, `AND1`, LSR forms, `PUSHX`, `TCLR1`, `PCALL`, `BVC`, `CMPW`, `LSRA`, `MOV X,A`, `CMPY abs`, and `JMP abs` behavior including PC-relative reads, stack push order, boolean carry logic, 16-bit compare flags, and accumulator/register Z/N updates. |
| opcodes `0x60..=0x7f` | `crates/snes/src/apu.rs` | verified | Matches C `CLRC`, CMP/CMPM forms, `AND1N`, ROR forms, `PUSHY`, `DBNZ`, `RET`, `BVS`, `ADDW`, `RORA`, `MOV A,X`, `CMPY dp`, and `RETI` behavior including memory decrement/writeback before branch, pull-word return order, flag pull before PC pull, carry/overflow/half-carry calculations, and YA result splitting. |
| helper operations used by this range | `crates/snes/src/apu.rs` | verified | `spc_eor`, `spc_eorm`, `spc_lsr`, `spc_ror`, `spc_cmp_a`, `spc_cmp_y`, `spc_cmpm`, `spc_pull_byte`, `spc_pull_word`, and `spc_set_flags` match C read/write order, carry assignment, compare carry, Z/N updates, stack pointer increments, and flag bit unpacking. |

## 2026-05-29 SPC Opcode Table 0x80-0xBF Pass

Scope: SPC opcode arms `0x80..=0xbf` in `spc_doOpcode`, ADC/SBC helper
families, increment/decrement helpers, indirect-postincrement addressing, and
YA word operations against `snes/spc.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| opcodes `0x80..=0x9f` | `crates/snes/src/apu.rs` | verified | Matches C `SETC`, ADC/ADCM forms, `EOR1`, DEC forms, `MOV Y,imm`, `POPP`, `MOV dp,imm`, `BCC`, `SUBW`, `DECA`, `MOV X,SP`, `DIV`, and `XCN` behavior including source/destination operand order, dummy destination read before write, division-by-zero fallback, and accumulator/register Z/N updates. |
| opcodes `0xa0..=0xbf` | `crates/snes/src/apu.rs` | verified | Matches C `EI`, SBC/SBCM forms, `MOV1`, INC forms, `CMPY imm`, `POPA`, `MOV (X)+,A`, `BCS`, `MOVW`, `INCA`, `MOV SP,X`, `DAS`, and `MOV A,(X)+` behavior including indirect postincrement, carry/half-carry handling, decimal-adjust wrapping subtraction, YA split/merge, and word Z/N updates. |
| helper operations used by this range | `crates/snes/src/apu.rs` | verified | `spc_adc`, `spc_adcm`, `spc_sbc`, `spc_sbcm`, `spc_inc`, `spc_dec`, `spc_mov_y`, `spc_cmp_y`, `spc_pull_byte`, and `spc_adr_ind_p` match C read/write order, carry/overflow/half-carry formulas, flag updates, stack pull behavior, and X postincrement semantics. |

## 2026-05-29 SPC Opcode Table 0xC0-0xFF Pass

Scope: SPC opcode arms `0xc0..=0xff` in `spc_doOpcode`, MOV load/store
helper families, direct-page indexed address selection, final branch/control
opcodes, and full opcode-table coverage against `snes/spc.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| opcodes `0xc0..=0xdf` | `crates/snes/src/apu.rs` | verified | Matches C `DI`, store-A forms, `CMPX imm`, store-X/Y forms, `MOV1 C->mem`, `POPX`, `MUL`, `BNE`, `MOVW dp,YA`, `DECY`, `MOV A,Y`, `CBNE dpx`, and `DAA` behavior including dummy read before store, abs-bit merge, multiplication Z/N from high byte, branch compare arithmetic, and decimal-adjust carry rules. |
| opcodes `0xe0..=0xff` | `crates/snes/src/apu.rs` | fixed | Matches C `CLRV`, load-A/X/Y forms, `NOT1`, `NOTC`, `POPY`, `SLEEP`, `BEQ`, `MOV dp,dp`, `INCY`, `MOV Y,A`, `DBNZ Y`, and `STOP` behavior. Fixed opcode `0xf9` to use direct-page Y indexing (`spc_adr_dpy`) for `MOV X,dpy`, matching C `spc_adrDpy`. |
| helper operations used by this range | `crates/snes/src/apu.rs` | verified | `spc_mov_a`, `spc_mov_x`, `spc_mov_y`, `spc_movs_a`, `spc_movsx`, `spc_movsy`, `spc_cmp_x`, `spc_adr_dpy`, `spc_pull_byte`, and direct word stores match C read/write order, flag updates, stack pull behavior, DP wrap rules, and low/high byte write ordering. |
| full SPC opcode table | `crates/snes/src/apu.rs` | verified | All opcode arms `0x00..=0xff` have now been manually compared against C `spc_doOpcode` across the four opcode-table passes. Remaining APU/SPC work, if any, is outside the 256-opcode dispatch table itself. |

## 2026-05-29 PPU Register and Full Renderer Pass

Scope: `PpuState` construction/reset, `ppu_read`, `ppu_write`,
`PpuBeginDrawing`, `ppu_runLine`, window helpers, background/sprite/mode7
render paths, color math, legacy fallback pixels, and public frame-buffer
handoff against `snes/ppu.c` plus the Rust `zelda_draw_ppu_frame` wrapper.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| reset and register data paths | `crates/snes/src/ppu.rs` | verified | `reset`, `read`, and `write` match C VRAM/CGRAM/OAM clearing, brightness/mosaic caches, OBJ defaults, forced blank, BG scroll latch behavior, Mode7 matrix writes, VRAM increment-on-low/high behavior, CGRAM/OAM second-write toggles, window registers, screen enables, color math registers, and fixed-color updates. |
| draw setup and frame-buffer handoff | `crates/snes/src/ppu.rs` / `crates/zelda3/src/zelda_rtl.rs` | verified | Brightness cache, color-map cache, current render scale, side-space configuration, HDMA line stepping, IRQ line-128 scroll write, and line loop match C behavior. Rust keeps an owned render buffer inside `PpuState` and `zelda_draw_ppu_frame` copies it back to the caller after all lines, which is the Rust equivalent of C storing a raw render-buffer pointer. |
| window helpers and background renderers | `crates/snes/src/ppu.rs` | verified | `PpuWindows_Clear/Calc`, main/sub screen window routing, 4bpp/2bpp renderers, mosaic variants, BG tilemap addressing, tile flip, palette shifts, priority-buffer writes, and Mode7 normal renderer match C control flow and formulas. |
| sprites and Mode7 upsampled path | `crates/snes/src/ppu.rs` | verified | Sprite evaluation/draw matches C sprite-size selection, high-OAM X/size bits, sprite/tile limits including no-limit override, x/y flip, tile selection, first-pixel-wins object buffer, sprite priority encoding, window clipping, and 4x Mode7 direct pixel rendering with side clears and sprite overlay. |
| color math and legacy renderer | `crates/snes/src/ppu.rs` | verified | New-renderer color window, clip/prevent math modes, fixed color, add/subscreen behavior, half-color lookup selection, extra side clears, legacy `ppu_handlePixel`, `ppu_getPixel`, BG pixel lookup, Mode7 start/pixel helpers, and sprite layer remap for palette `< 0xc0` match C behavior. |
| `ppu_saveload` byte-layout routine | `crates/snes/src/ppu.rs` / `crates/zelda3/src/zelda_rtl.rs` | fixed | Added `PpuState::save_c_saveload` / `load_c_saveload` with C's exact 67,305-byte layout: VRAM, 10-byte padding, CGRAM, 556-byte padding, 520-byte padding, four 12-byte BG-layer snapshot records, and final 123-byte padding. `InternalSaveLoad` now uses this C layout instead of the previous oversized bincode slot. |

## 2026-05-29 SNES Shell and Bus Pass

Scope: `Snes` construction/reset, B-bus reads/writes, internal register
reads/writes, 24-bit bus reads/writes, CPU bus access timing, APU catchup,
auto-joypad reads, DMA dispatch, cart fallthrough, and C helper surfaces against
`snes/snes.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| construction and reset | `crates/snes/src/snes.rs` | verified | Matches C component initialization/reset ordering, hard-reset WRAM clear, RAM address reset, H/V/frame counters, CPU cycle/mem-op counters, APU catchup accumulator, IRQ/NMI/vblank flags, joypad auto-read fields, multiply/divide defaults, fast-mem flag, and open-bus reset. |
| B-bus and internal registers | `crates/snes/src/snes.rs` | verified | `read_b_bus`, `write_b_bus`, `read_reg`, and `write_reg` match C PPU/APU/WRAM data-port routing, APU catchup before SNES-to-APU port writes, open-bus bit preservation, NMI/IRQ read-and-clear behavior, HV/vblank status bits, auto-joypad result reads, joypad latch, multiplication/division including divide-by-zero, IRQ enable clearing, timers, DMA/HDMA starts, and fast-mem writes. |
| 24-bit bus and CPU access timing | `crates/snes/src/snes.rs` | verified | `raw_read`, `read`, `write`, `cpu_read`, and `cpu_write` match C low-bank/hi-bank WRAM mirrors, B-bus range, controller ports, internal register range, DMA register range, banks `0x7e/0x7f`, cart read/write fallthrough, open-bus update on public read/write, CPU mem-op increment, and flat 6-cycle access-time optimization. |
| auto joypad and APU catchup helpers | `crates/snes/src/snes.rs` | verified | `do_auto_joypad` matches C latch-line pulse, two-controller cycle calls, 16 serial reads, and port bit packing. `catchup_apu` matches integer truncation of accumulated cycles, repeated `apu_cycle`, and fractional carry-forward. |
| `snes_saveload` and debug print helpers | `crates/snes/src/snes.rs` / `crates/snes/src/apu.rs` / `crates/snes/src/cart.rs` / `crates/snes/src/tracing.rs` / `crates/zelda3/src/zelda_cpu_infra.rs` | fixed-limited | Added a standalone `Snes::save_c_saveload` / `load_c_saveload` aggregate matching C call order: CPU, APU RAM-through-`cpuCyclesLeft` prefix, DSP, SPC, DMA, PPU, cart SRAM, contiguous `hPos..openBus` block, WRAM, and `ramAdr`; it also clears `disable_hpos` after save/load like C. Added C-layout APU prefix, SPC, and cart SRAM helpers. The debug print path now emits the C-shaped CPU register/flag line before each opcode when `debug_cycles` is set, including `tracing.c`'s CPU opcode templates, 8-bit immediate substitutions, address-type table, relative target math, and C-style operand formatting. `Snes::print_cpu_line` covers the C `snes_printCpuLine` VRAM word suffix. `ApuState::spc_trace_line` now ports `getProcessorStateSpc` / `getDisassemblySpc`, including SPC opcode templates, address-type table, relative target math, bit-address formatting, and the same side-effecting `apu_cpuRead` behavior as C. |

## 2026-05-29 DMA Register and Transfer Pass

Scope: `DmaState` construction/reset, DMA register reads/writes,
`dma_startDma`, `dma_doDma`, `dma_initHdma`, `dma_doHdma`,
`dma_transferByte`, `dma_cycle`, and C helper surfaces against `snes/dma.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| reset and register data paths | `crates/snes/src/dma.rs` | verified | `DmaState::reset`, `dma_read`, and `dma_write` match C per-channel defaults, active/terminated/doTransfer flags, mode flag packing/unpacking, B-bus/A-bus address registers, indirect-bank/table/repeat/unused registers, channel selection from `adr & 0x70`, and open-bus/no-op behavior for unmapped DMA register offsets. |
| DMA start and byte transfer | `crates/snes/src/dma.rs` | verified | `dma_start_real` matches C `dma_startDma` for DMA versus HDMA activation, busy flag setting, and initial 16-cycle DMA cost. `dma_transfer_byte` matches C transfer direction behavior for A-to-B and B-to-A bus moves. |
| DMA transfer loop | `crates/snes/src/dma.rs` | verified | `dma_do` matches C `dma_doDma`: first active channel selection, transfer length/mode address offset lookup, per-byte 6-cycle cost, fixed/increment/decrement A-address update semantics, 16-bit size countdown, off-index reset, active-bit clear, and final 8-cycle channel completion cost. |
| HDMA init and line transfer | `crates/snes/src/dma.rs` | verified | `dma_init_hdma` and `dma_do_hdma` match C table address setup, repeat-count reads, indirect address fetches, per-line transfer lengths, direct versus indirect source addresses, repeat-count decrement/high-bit transfer rules, termination on zero count, and the same 8/16-cycle accounting points. |
| DMA scheduler | `crates/snes/src/dma.rs` | verified | `dma_cycle` matches C priority: drain HDMA timer first by two cycles, run DMA while busy, and report no DMA/APU-cycle hold when both timers and active DMA are idle. |
| init/free/back-pointer and `dma_saveload` | `crates/snes/src/dma.rs` / `crates/zelda3/src/zelda_rtl.rs` | fixed-limited | Added `DmaState::save_c_saveload` / `load_c_saveload` with C's exact 192-byte ABI layout from `Dma.channel` through tail padding after `dmaBusy`; `InternalSaveLoad` now uses that block. Rust still stores DMA state inside `Snes` instead of exposing C's `dma_init`/`dma_free` pointer ownership shape, but the saveload surface is now C-shaped. |

## 2026-05-29 Playable Host Wiring Pass

Scope: executable host surface needed to run a playable window/audio loop from
the Rust tree, compared against the responsibilities of `src/main.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| platform window/input/video/audio host | `crates/platform/src/lib.rs` | superseded | This pass originally introduced an SDL frontend, but that host was later removed. The current live dependency surface is the native Rust host in `crates/platform` using `winit`, `pixels`, and `cpal`; see the later Native Host Removal Pass. |
| default playable binary path | `zelda3-bin/src/main.rs` | superseded | Default `zelda3 <rom.sfc>` still loads the ROM/assets/SRAM, runs `zelda_run_frame`, renders PPU pixels, queues audio, and writes SRAM on quit, but it now opens the `NativeFrontend` rather than SDL. |
| per-frame audio output | `zelda3-bin/src/main.rs` / `crates/platform/src/lib.rs` | superseded | The playable loop still calls `zelda_render_audio`, but samples now flow into the cpal-backed queue rather than SDL audio. |
| C SDL feature parity | `zelda3-bin/src/main.rs` | superseded | The project intentionally removed SDL from the playable host. Remaining host parity work is tracked as native-host scheduling/input/audio parity, not C SDL API parity. |
| game-logic runtime coverage | `crates/zelda3/src/*.rs` | gap | The host can start and drive frames, but playability still depends on unported or partially ported game-logic paths not panicking during live gameplay. Continue manual parity passes against the first runtime panic/divergence reached by `zelda3 <rom.sfc>`. |

## 2026-05-29 Playable Black-Screen Fix Pass

Scope: first visible-frame blocker in the playable host path, covering intro
OAM helper parity and NMI PPU register upload against `src/ending.c` and
`src/nmi.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| intro scene OAM helper | `crates/zelda3/src/ending.rs` | fixed | `AnimateSceneSprite_AddObjectsToOamBuffer` now routes each object through the same `SetOamHelper0` clamp used by C, hiding offscreen Y positions as `0xf0` and updating bytewise extended OAM consistently. This removed the frame-31 startup divergence where Rust wrote raw offscreen Y bytes into OAM. |
| NMI PPU register upload | `crates/zelda3/src/nmi.rs` | fixed | Ported the C `WritePpuRegisters` batch for window/color math, TM/TS/TMW/TSW, BG scrolls, INIDISP, MOSAIC, BGMODE, Mode7 center writes, and BG tile base registers. This unblocked the renderer from reset/forced-blank state. |
| smoke-render visibility | `zelda3-bin/src/main.rs` | verified | `--smoke-render /path/to/zelda3.sfc 1200` now reports `nonzero_pixels=23397`, `forced_blank=false`, `brightness=15`, `mode=1`, and screen enables `15/02`; before this pass it stayed at zero visible pixels with forced blank still set. |

## 2026-05-29 Playable Menu-to-Gameplay Smoke Pass

Scope: scripted input through intro skip, file-select, new-file naming,
file load, and the first room spawn path. This is a playability boundary check,
not a full game-logic parity claim.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| intro/file-select input route | `zelda3-bin/src/main.rs` / `crates/platform/src/lib.rs` | verified | A lockstep input script using `START`, `B`, and `START` advances from intro (`main=0`) to file select (`main=1`), enters name-file (`main=4`), creates a non-empty save, returns to file select, and selects the new file. This confirms the playable keyboard/input word mapping is sufficient for the start/create/load path. |
| first gameplay spawn | `crates/zelda3/src/*.rs` | verified boundary | The same lockstep run reaches gameplay at frame 2302 with `main=7 sub=15`, room `$0104`, Link at `$0940,$215a`, and uncle sprite coordinates `$09a8,$2170`. This proves the black-screen/menu-only blocker is past and the host can drive into a playable scene. |
| first remaining gameplay divergence | `crates/zelda3/src/sprite.rs` / sprite prep surfaces | superseded | This historical frame-2304 `sprite_flags4[0]` divergence was fixed by the later Uncle Gameplay Parity Pass below. Re-verified current head with `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, which completes 7600 frames from frame 0 with WRAM hash `685840c2154c2a94`. |

## 2026-05-29 Playable Host Fix Pass

Scope: fixes for the reported default SDL run symptoms: silent audio, slow
runtime, and a 256-wide frame shifted far to the right/cut off inside the
window.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| normal play loop | `zelda3-bin/src/main.rs` | fixed | Default `zelda3 <rom.sfc>` now runs `ZeldaState` directly instead of calling `LockstepOracle::run_frame_with_compare` every frame. `--lockstep` remains the parity/debug path, but the playable host no longer pays the C-oracle comparison cost. |
| default render width/side-space | `crates/snes/src/ppu.rs` / `crates/platform/src/lib.rs` | fixed | `PpuState::new` now matches C's default `extended_aspect_ratio=0` behavior instead of always enabling `kPpuExtraLeftRight`. This removes the 96-pixel side-space offset on the default 256-wide frame. Later host passes superseded the old SDL presentation details with the current `pixels` frontend. |
| song-bank and APU-port wiring | `crates/zelda3/src/misc.rs`, `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/nmi.rs`, `crates/zelda3/src/audio.rs` | fixed | Ported the intro/overworld/dungeon/credits song-bank loaders, restored the startup `Sound_LoadIntroSongBank` call, restored NMI writes to APUI00-03 for queued music and SFX, and made `load_song_bank` upload into the live `SpcPlayer` like C. |
| smoke verification | `zelda3-bin/src/main.rs` | verified | `--smoke-render /path/to/zelda3.sfc 1200` reports visible pixels (`nonzero_pixels=37301`) with the default 256-wide render path and audible generated samples (`audio_nonzero=1319446`, `audio_peak=23539`). |

## 2026-05-29 SPC Music Sequencer Audio Pass

Scope: the remaining silent-audio blocker after host, song-bank, and APUI port
wiring were restored. Compared `Port0_HandleMusic`, phrase loading, channel
pattern advancement, note scheduling, and pitch write arithmetic against
`src/spc_player.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Port0_HandleMusic` command and tick path | `crates/zelda3/src/spc_player.rs` | fixed | Rust now includes the C `a == 0` music tick path instead of only handling new track commands. This restores pause/continue commands, delayed reset via `counter_sf0c`, phrase-table loading, channel pattern-pointer setup, note/effect parsing, note gate-off timing, tempo/echo/master-volume fades, and per-channel `Chan_HandleTick` calls. |
| phrase reload control flow | `crates/zelda3/src/spc_player.rs` | fixed | Matched C's two `goto next_phrase` cases: decrementing `counter_sf0c` to zero falls through into phrase load, and stream command `0` with no subroutine loop loads the next phrase instead of scanning phrase-pointer table bytes as commands. |
| C unsigned pitch arithmetic | `crates/zelda3/src/spc_player.rs` | fixed | `WritePitch` now uses explicit wrapping for the low-note adjustment expression that C evaluates through unsigned casts. This avoids debug-build overflow while preserving C semantics. |
| debug visibility | `crates/zelda3/src/spc_player.rs`, `crates/zelda3/src/audio.rs`, `zelda3-bin/src/main.rs` | added | Added an SPC debug summary surfaced in `--smoke-render` output so future audio smoke runs show port state, active channel patterns, sample-buffer status, and DSP flags alongside pixel/audio sample counts. |
| smoke verification | `zelda3-bin/src/main.rs` | verified | After this pass the same 1200-frame smoke run reports all 8 music channels active, nonzero DSP samples, `audio_nonzero=1319446`, and `audio_peak=23539`; the previous run reported `audio_nonzero=0`. |

## 2026-05-29 Uncle Gameplay Parity Pass

Scope: first live-gameplay lockstep divergences after file creation and room
`$0104` entry, compared against `src/sprite.c`, `src/sprite_main.c`, and
`src/dungeon.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| sprite init property tables | `crates/zelda3/src/sprite.rs` | fixed | The packed Rust `SPRITE_INIT_TABLES_HEX` data had drifted by four bytes after `BumpDamage`, shifting `Flags3`, `Flags4`, `Flags`, `Flags5`, and `DeflBits` for later sprite types. Added a regenerated C-derived packed table and routed `sprite_init_value` through it; all eight 243-byte C init tables now compare byte-for-byte, including `kSpriteInit_Flags4[0x73] == 0x0a`. |
| live sprite dispatch | `crates/zelda3/src/sprite.rs` | fixed | `Sprite_ExecuteSingle` now dispatches state `9` to `sprite_active_main` like C instead of the temporary death-path stub. The death helper also forwards to the real active dispatcher. This restores Uncle's first-frame draw/OAM allocation path. |
| dungeon room tag handler | `crates/zelda3/src/dungeon.rs` | fixed | Replaced the empty `dungeon_handle_room_tags` stub with the C-shaped skip-flag check, `Dungeon_DetectStaircase` call, tag-slot dispatch, and final `flag_skip_call_tag_routines = 0`. This fixed the `$04c7` divergence after entrance load. |
| lockstep verification | `zelda3-bin/src/main.rs` | verified | The scripted start/create-file/enter-room path now completes `7600` lockstep frames from frame 0 with `/path/to/zelda3.sfc`, passing the previous frame-2304 sprite-flags divergence, the frame-2305 Uncle OAM divergence, and the frame-2321 tag-skip divergence. |
| playable smoke verification | `zelda3-bin/src/main.rs` | verified | `--smoke-render /path/to/zelda3.sfc 1200` still reports visible pixels and audible samples after the dispatch changes: `nonzero_pixels=37301`, `audio_nonzero=1319446`, `audio_peak=23539`. |

## 2026-05-29 PPU Internal-Buffer Offset Pass

Scope: the default 256-wide SDL render path after enabling the C-shaped
`PpuDrawWholeLine` renderer. Compared internal z-buffer/object-buffer indexing
and final scanline copy behavior against `snes/ppu.c` and `src/types.h`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| fixed internal PPU buffer center | `crates/snes/src/ppu.rs` | fixed | C always indexes `bgBuffers` and `objBuffer` at `x + kPpuExtraLeftRight`, even when the runtime output margin `extraLeftRight` is zero. Rust now uses `PPU_EXTRA_LEFT_RIGHT` for mosaic BG writes, mode7 BG writes, sprite evaluation writes, sprite compositing reads, legacy sprite reads, mode7 upsample sprite overlay, and color-window source reads. |
| runtime output scanline pointer | `crates/snes/src/ppu.rs` | fixed | `PpuDrawWholeLine` now separates the 448-wide internal read index from the output write pointer. The final copy starts at `extra_left_right - extra_left_cur` and advances sequentially like C instead of writing at the internal 96-pixel source index, which prevented the default 256-wide frame from being shifted/cut off. |
| verification | `zelda3-bin/src/main.rs` | verified | `cargo check -p snes -p zelda3 -p zelda3-bin -p platform --tests` and `git diff --check` pass. `--smoke-render /path/to/zelda3.sfc 1200` reports `nonzero_pixels=37301`, `audio_nonzero=1319446`, `audio_peak=23539`. The scripted lockstep path completes `12000` frames with WRAM hash `6415022923f1e5ac`. |

## 2026-05-29 Playable Host Runtime Pass

Scope: historical fixes for the default `cargo run -p zelda3-bin --
/path/to/zelda3.sfc` path. These entries predate the
later native-host removal of SDL but still describe renderer/audio issues that
were fixed in the playable loop.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| default renderer path | `zelda3-bin/src/main.rs` | fixed | The playable host and smoke-render path now enable `PpuDrawWholeLine` via `PpuRenderFlags::NEW_RENDERER`, avoiding the slower legacy per-pixel fallback while keeping the default 256-wide output surface. |
| per-frame audio drain | `zelda3-bin/src/main.rs` | superseded | This historical SDL-queue row was later superseded by the native `cpal` host. The relevant parity point remains that the playable loop drains stale APU writes after rendering audio blocks; exact host scheduling is tracked in the later native-host/snes9x audio rows. |
| default cargo-run speed | `Cargo.toml` | fixed-limited | Dev builds now use `opt-level = 1` so the exact `cargo run` command is less likely to fall below realtime. Release remains the preferred performance profile for parity/perf checks. |

## 2026-05-29 C SaveLoad Layout Pass

Scope: the manual parity gaps for `dma_saveload`, `ppu_saveload`, and the
Zelda runtime `InternalSaveLoad` bridge. C layout sizes were confirmed from the
source headers with a local ABI probe: `DmaChannel=22`, `Dma` save block `192`,
`BgLayer` snapshot starts at `tilemapWider`, and PPU save block `67305`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `dma_saveload` | `crates/snes/src/dma.rs` | fixed | Added exact save/load helpers for the 192-byte C block: eight 22-byte channels, `hdmaTimer`, two ABI padding bytes, `dmaTimer`, `dmaBusy`, and seven tail padding bytes. |
| `ppu_saveload` | `crates/snes/src/ppu.rs` | fixed | Added exact save/load helpers for C's VRAM/CGRAM/BG-layer snapshot layout with explicit zero padding. Load intentionally restores only the C-saved fields, so transient renderer/register fields such as brightness are not carried by this path. |
| Zelda runtime state bridge | `crates/zelda3/src/zelda_rtl.rs` | fixed | Replaced the previous bincode DMA/PPU slots in `InternalSaveLoad` with the C-layout blocks and updated the roundtrip test to assert C-saved PPU fields rather than bincode-only fields. |
| verification | `crates/snes/src/dma.rs`, `crates/snes/src/ppu.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | `cargo test -p snes c_saveload_layout_matches -- --nocapture`, `cargo test -p zelda3 save_load -- --nocapture`, `cargo check -p snes -p zelda3 -p zelda3-bin -p platform --tests`, `cargo fmt`, and `git diff --check` pass for this slice. |

## 2026-05-29 Overworld Scroll Renderer Audit Pass

Scope: overworld screen-boundary scrolling and map-strip upload renderer
surface, compared against `src/overworld.c` around `OverworldHandleTransitions`,
`OverworldScrollTransition`, `OverworldTransitionScrollAndLoadMap`,
`CheckForNewlyLoadedMapAreas_*`, `BufferAndBuildMap16Stripes_*`, and the
initial big/small screen-view builders.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| scroll target and boundary constants | `crates/zelda3/src/overworld.rs` | verified | Compared the Rust constants against C for `kOverworld_OffsetBaseX/Y`, up/down and left/right scroll targets, scroll sizes, strip offsets, `kOverworld_Func6B_*`, `kOverworld_Func8_tab`, and `kOverworldAreaHeads`; all table lengths and values match. |
| transition state machine | `crates/zelda3/src/overworld.rs` | verified | Current Rust matches C's start/run/ease-off sequence: moving animation, incremental VRAM upload, scroll-step return low-nibble check, temporary `overworld_screen_trans_dir_bits2` map-load trigger, small-screen immediate reload during ease-off, subsubmodule countdown gates, small-screen restore of saved map16 cursors, and follower disable. |
| camera scroll and screen-boundary detection | `crates/zelda3/src/overworld.rs` | verified | `OverworldHandleTransitions`, `Overworld_OperateCameraScroll`, `OverworldCameraBoundaryCheck`, `OverworldScrollTransition`, and `Overworld_FinalizeEntryOntoScreen` match C's direction tests, edge transition rejection, area-head promotion, music/ambient updates, scroll-register arithmetic, camera-boundary counters, target comparison, Link coordinate snapping, camera boundary reload, slot reinitialization, and entry-finalization music resume. |
| map strip upload renderer | `crates/zelda3/src/overworld.rs` | verified | `OverworldTransitionScrollAndLoadMap`, `OverworldHandleMapScroll`, `CheckForNewlyLoadedMapAreas_*`, `BuildFullStripeDuringTransition_*`, and `BufferAndBuildMap16Stripes_X/Y` match C's direction dispatch, small-vs-big suppression, source/destination cursor mutation, terminator writes, NMI subroutine trigger, VRAM stripe headers, ring-buffer staging in `dung_replacement_tile_state`, and map16-to-map8 word ordering for X and Y stripes. |
| runtime coverage boundary | `zelda3-bin/src/main.rs` | verified-limited | The current scripted lockstep path completes `13000` frames with WRAM hash `9da3828ae5625e64`, but it remains in Uncle's message box (`main=14 sub=2`) and does not yet exercise overworld scroll transitions. This pass is source parity plus existing lockstep safety, not a runtime overworld-scroll traversal proof. |
| playable binary description | `zelda3-bin/src/main.rs` | superseded | This row fixed the old headless-only file header during the SDL-host period. The current binary description is covered by the later native-host pass and now reflects the default `winit`/`pixels`/`cpal` playable host. |

## 2026-05-29 Visual Gameplay and Audio Continuity Pass

Scope: follow-up on playable-host corruption/static reports using the scripted
file-select-to-Uncle route, with C lockstep as the visual/state oracle.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| indoor Y collision snap | `crates/zelda3/src/player.rs` | fixed | `Link_AddInVelocityYFalling` now matches C signed arithmetic for `(tiledetect_which_y_pos[0] & 7) - (sign8(link_y_vel) ? 8 : 0)`. The previous Rust wrapping-`u8` subtraction could convert a small signed snap into a large unsigned coordinate adjustment, which caused the first live gameplay divergence when pressing Up in room `$0104`. |
| visual gameplay lockstep | `zelda3-bin/src/main.rs` | verified | `cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 11163 --input-script scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt` now passes the former frame-11162 divergence. The wider `120820`-frame route also passes with WRAM hash `399e3c1ee062788a`, covering WRAM/SRAM/VRAM/CGRAM/OAM/visible PPU-reg parity for the route. |
| visual frame dump | `zelda3-bin/src/main.rs` | verified | `--dump-frame ... 120820 /private/tmp/z3-wander-120820.png --input-script scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt` produces a correct Uncle room/message frame: `main=0e`, `sub=02`, `mode=1`, `screen=16/00`, `cgram_nonzero=221`, `oam_nonzero=233`. |
| native audio continuity | `crates/platform/src/lib.rs` | superseded | This originally fixed the SDL queue host by preventing silent sample-block drops. The current `cpal` queue carries the same continuity requirement, while exact callback/frame timing is tracked in the later native-host and snes9x startup-audio rows. |
| build hygiene | `crates/platform/src/lib.rs`, `crates/zelda3/src/player.rs` | verified | `cargo check -p platform -p zelda3-bin` and `git diff --check` pass after the collision and audio-host fixes. |

## 2026-05-29 Playable Host Audio and Pixel-Compare Pass

Scope: follow-up on reports that the playable host was slow, audio was staticy,
and the save/file-select screen had visible corruption.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| playable host frame pacing | `zelda3-bin/src/main.rs` | superseded | Removed the explicit `16.667ms` sleep from `run_play` during the SDL-host period because the canvas already used vsync. The current `winit`/`pixels` host has its own pacing path; native timing parity is tracked in later host rows. |
| native audio queue fill | `crates/platform/src/lib.rs`, `zelda3-bin/src/main.rs` | superseded | This was an SDL queue-fill tuning row. The current host uses `cpal`; buffer-fill continuity remains relevant, but exact C callback equivalence is now tracked as native-host audio scheduling rather than SDL queue parity. |
| visual comparison tooling | `zelda3-bin/src/main.rs` | added | Added `--compare-renderers <rom> <frames> [diff.png] [--input-script <path>] [--load-state <path>]`, plus `--legacy-renderer` and `--load-state` support for `--dump-frame`. This gives a direct pixel-diff harness for the optimized `PpuDrawWholeLine` path against the legacy per-pixel renderer at exact checkpoints. |
| save/file-select visual check | `zelda3-bin/src/main.rs` | verified | `--dump-frame ... 2050 /private/tmp/z3-file-select-2050.png --input-script scripts/inputs/file-select-enter-game.txt` produced a coherent player-select screen. `--compare-renderers ... 2050 /private/tmp/z3-render-diff-2050.png --input-script scripts/inputs/file-select-enter-game.txt` reported `mismatched_pixels=0`. |
| scripted message visual check | `zelda3-bin/src/main.rs` | verified | `--compare-renderers ... 2400 /private/tmp/z3-render-diff-2400.png --input-script scripts/inputs/file-select-enter-game.txt` reported `mismatched_pixels=0`. A checkpointed one-frame compare at frame `120820` from `/private/tmp/z3-120819.z3rs` also reported `mismatched_pixels=0`. |

## 2026-05-29 APU/DSP SaveLoad Parity Pass

Scope: the remaining placeholder audio slots in `InternalSaveLoad`, compared
against `src/zelda_rtl.c` and `snes/dsp.{c,h}`. C writes the SPC-player APU RAM
and the 3024-byte `dsp_saveload` block before DMA/PPU/SRAM/WRAM.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `dsp_saveload` layout | `crates/snes/src/apu.rs` | fixed | Added exact 3024-byte C-layout save/load helpers for `DspState`: 128-byte mirror RAM, eight 86-byte channel records, scalar DSP state at ABI-derived offsets, FIR buffers, 1068-sample stereo buffer, and `sampleOffset` at save offset `3020`. The old Rust runtime had no C-shaped DSP snapshot bridge. |
| SPC-player APU RAM snapshot | `crates/zelda3/src/spc_player.rs`, `crates/zelda3/src/audio.rs` | fixed | Added wrappers to copy the live `SpcPlayer::ram` into the C APU-RAM slot and restore it on load while re-pointing the DSP RAM pointer. This replaces the previous zero-filled `apu_ram` placeholder in `InternalSaveLoad`. |
| runtime `InternalSaveLoad` bridge | `crates/zelda3/src/zelda_rtl.rs` | fixed | `InternalSaveLoad` now saves/loads real SPC-player RAM and real DSP state in the same positions as C, instead of consuming those slots as junk. DMA, PPU, SRAM, WRAM ordering remains unchanged. |
| verification | `crates/snes/src/apu.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | `cargo test -p snes dsp_c_saveload_layout_roundtrips_saved_fields -- --nocapture`, `cargo test -p zelda3 save_load -- --nocapture`, `cargo check -p snes -p zelda3 -p zelda3-bin -p platform --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass. |

## 2026-05-29 SPC Variable RAM Mirror Pass

Scope: the empty Rust `spc_player_copy_variables_to_ram` and
`spc_player_copy_variables_from_ram` stubs, compared against
`src/spc_player.c`'s `kChannel_Maps`, `kSpcPlayer_Maps`,
`SpcPlayer_CopyVariablesToRam`, and `SpcPlayer_CopyVariablesFromRam`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| channel variable mirror maps | `crates/zelda3/src/spc_player.rs` | fixed | Ported the C channel map addresses for all eight channels, including byte fields at `org_off + channel * 2` and word fields using the C `0x8000` size bit with low `0x7fff` RAM address. This covers pattern pointers, note timers, envelopes, volumes, pan, pitch, and SFX channel fields. |
| SPC player variable mirror maps | `crates/zelda3/src/spc_player.rs` | fixed | Ported the C player-level RAM mirror addresses for SNES port state, tempo, master volume, echo, SFX-port state, and the `$03c0-$03ff` control fields. The previous Rust functions were no-op stubs. |
| music save/restore hooks | `crates/zelda3/src/audio.rs` | fixed | `zelda_save_music_state_to_ram_locked` now calls the copy-to-RAM mirror before writing `$0410-$0413`, and `zelda_restore_music_after_load_locked` now calls copy-from-RAM, resets `timer_cycles`, restores input ports from `$0410-$0413`, and then follows the C reset/MSU path. |
| verification | `crates/zelda3/src/spc_player.rs`, `crates/zelda3/src/audio.rs` | verified | `cargo test -p zelda3 copy_variables_to_and_from_spc_ram_uses_c_addresses -- --nocapture`, `cargo test -p zelda3 save_load -- --nocapture`, `cargo test -p snes dsp_c_saveload_layout_roundtrips_saved_fields -- --nocapture`, `cargo check -p snes -p zelda3 -p zelda3-bin -p platform --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass. |

## 2026-05-29 Historical Audio Underrun and Save-Screen Visual Probe

Scope: follow-up on the report that live audio is staticy and save/file-select
graphics looked corrupt in the playable host. The audio-host rows in this
section predate the later SDL removal and are retained as historical evidence.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| audio queue sizing | `crates/platform/src/lib.rs`, `zelda3-bin/src/main.rs` | superseded | This SDL queue-sizing fix predates the native-host replacement. The current `cpal` queue has a different device-buffer surface; remaining exact audio parity work is tracked in the native-host scheduling and snes9x startup-audio rows. |
| save/file-select rendered frame | `zelda3-bin/src/main.rs` | verified-limited | `--dump-frame ... 2050 /private/tmp/z3-save-2050.png --input-script scripts/inputs/file-select-enter-game.txt` produced a coherent player-select screen using the current `saves/sram.dat`. `--compare-renderers ... 2050 ...` reported `mismatched_pixels=0`. |
| save/file-select state parity | `zelda3-bin/src/main.rs` | verified | `--lockstep ... 2050 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completed, so WRAM/SRAM/VRAM/CGRAM/OAM and visible PPU register state match the ROM oracle at that checkpoint. |
| transition/message visual probes | `zelda3-bin/src/main.rs` | verified-limited | `--dump-frame` at frames `2205`, `2400`, `2505`, and `3005` covered the black transition and Uncle room/message frames. The `2400` frame is coherent, and `--compare-renderers ... 2205 ...` reported `mismatched_pixels=0`. A C-renderer PNG oracle is still needed for strict visual identity beyond Rust new-vs-legacy renderer agreement. |
| verification | `crates/platform/src/lib.rs`, `zelda3-bin/src/main.rs` | verified | `cargo check -p platform -p zelda3-bin`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and the save-screen renderer/lockstep probes above pass. |

## 2026-05-29 Legacy Renderer Removal Pass

Scope: remove the Rust-only legacy per-pixel renderer fallback now that
`PpuDrawWholeLine` is the playable renderer path.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| PPU scanline renderer selection | `crates/snes/src/ppu.rs` | fixed | Removed `PpuRenderFlags::NEW_RENDERER` and the `run_line` branch that selected `draw_legacy_line`. `run_line` now always uses the ported whole-line renderer, while `MODE7_4X4`, `HEIGHT_240`, and `NO_SPRITE_LIMITS` remain as independent options. |
| legacy per-pixel code | `crates/snes/src/ppu.rs` | removed | Deleted the unused fallback helpers for per-pixel BG/mode7 lookup, legacy color math, and legacy scanline drawing. This leaves one visual renderer to audit and prevents future checks from accidentally comparing Rust against itself. |
| CLI surface | `zelda3-bin/src/main.rs` | fixed | Removed `--compare-renderers` and `--legacy-renderer` because they depended on the deleted fallback. `--dump-frame`, `--smoke-render`, and `--lockstep` remain the deterministic visual/state probes. |
| lockstep render comparison | `zelda3-bin/src/main.rs` | added | Added `--compare-lockstep-render`, which runs the ROM lockstep oracle, renders both the ported `ZeldaState` and the emulated SNES oracle state through the single remaining renderer, and fails on pixel mismatches. This replaces the old Rust-new-vs-Rust-legacy comparison with a state-oracle-based visual check. |
| config shim | `crates/zelda3/src/main.rs` | fixed | `NewRenderer` config is no longer mapped into runtime flags; the whole-line renderer is always active. |
| verification | `crates/snes/src/ppu.rs`, `zelda3-bin/src/main.rs` | verified | `cargo check -p snes -p zelda3 -p zelda3-bin -p platform --tests`, `--smoke-render /path/to/zelda3.sfc 1200`, `--dump-frame ... 2050 /private/tmp/z3-single-renderer-2050.png --input-script scripts/inputs/file-select-enter-game.txt`, `--lockstep ... 2050 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `--compare-lockstep-render` at frames `2050` and `2400` with the same script/SRAM all pass after the removal. |

## 2026-05-29 Playable Crash Diagnostics and Pot DMA Fix

Scope: follow-up on the interactive playable crash when picking up a pot:
`crates/zelda3/src/misc.rs:1144` indexed `LINK_DMA_SOURCES3[21]` while the
Rust table had length 1.

| Surface | Rust file(s) | Status | Notes |
|---|---:|---|---|
| Link DMA source table 3 | `crates/zelda3/src/misc.rs` | fixed | C `src/misc.c` defines `kLinkDmaSources3[27]`; Rust only had `[0x9a40]`. Ported all 27 C entries so valid item/pot carry graphics DMA indices no longer panic. |
| playable crash reports | `zelda3-bin/src/main.rs` | added | The SDL playable loop now catches frame/audio/present panics long enough to write `/tmp/zelda3-rs-crash-*.txt` with host frame, `frame_ctr_dbg`, WRAM/SRAM hashes, PPU state, input/run selector, and the existing `TraceState` module/player/sprite/DMA summary. It also writes `/tmp/zelda3-rs-crash-*.z3play`, a bincode snapshot of the current `ZeldaState`, so future reports can include reproducible state rather than only a Rust panic location. |
| verification | `crates/zelda3/src/misc.rs`, `zelda3-bin/src/main.rs` | verified | `cargo check -p zelda3 -p zelda3-bin --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, `--compare-lockstep-render ... 2400 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `git diff --check` pass. |

## 2026-05-29 SPC Unsupported Command Abort Parity Pass

Scope: follow-up on remaining explicit `Not_Implemented` markers in the SPC
player compared against `src/spc_player.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Not_Implemented()` behavior | `crates/zelda3/src/spc_player.rs` | fixed | C calls `assert(0)` before printing `Not Implemented`; Rust previously printed and continued. `not_implemented()` now has return type `!` and panics, so unsupported effect/tremolo/music-port commands fail hard like C debug builds instead of silently continuing with invalid audio state. |
| call sites | `crates/zelda3/src/spc_player.rs` | reviewed | Rust call sites match the C unsupported-command surfaces: default effect command, `HandleTremolo`, `CalcTremolo`, and the `port0 == 0xff` music command branch. |
| verification | `crates/zelda3/src/spc_player.rs` | verified | `cargo check -p zelda3 -p zelda3-bin --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the hard-stop change. |

## 2026-05-29 Dungeon Map Floor Control Parity Pass

Scope: explicit empty-body scan found dungeon map input/scroll routines in
`crates/zelda3/src/messaging.rs` that were not empty in C.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `DungeonMap_HandleMovementInput` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C call sequence: handle floor select first, then scroll active floor transitions while `dungmap_var2` is nonzero. |
| `DungeonMap_HandleFloorSelect` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C floor-count gating, up/down input checks, current-floor arithmetic, redraw calls, scroll target setup, `WORD(g_ram[6])`/`WORD(g_ram[10])` scratch writes, and `nmi_subroutine_index = 8`. |
| `DungeonMap_ScrollFloors` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C signed scroll deltas for `dungmap_var5`, `dungmap_var8`, and `BG2VOFS_copy2`, including clearing `dungmap_var2` when the target offset is reached. |
| confirmed C no-ops | `crates/zelda3/src/player.rs`, `crates/zelda3/src/sprite_main_hinox_shop.rs`, `crates/zelda3/src/sprite_main_prep.rs` | reviewed | Empty-body scan confirmed `PlayerHandler_15_HoldItem`, `Hinox_ThrowBomb`, and `SpritePrep_ThrowableScenery` are empty in C by design, so Rust no-ops are parity-preserving. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests` and `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200` pass after the port. |

## 2026-05-29 Dungeon Map Buffer Writer Parity Pass

Scope: continue the dungeon map empty-body audit in `crates/zelda3/src/messaging.rs`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `DungeonMap_BuildFloorListBoxes` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C VRAM upload writer using `kDungMap_Tab9`, swapped VRAM addresses, `$0700` block lengths, row split at four tiles, and `vram_upload_offset` word-to-byte accounting. |
| `DungeonMap_DrawBorderForRooms` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C messaging-buffer border writes using `kDungMap_Tab10` through `kDungMap_Tab15`, including `$0fff` wrapping and caller-provided mask application. |
| `DungeonMap_DrawFloorNumbersByRoom` | `crates/zelda3/src/messaging.rs` | fixed | Ported the C floor-label writer: clears the repeated `$0f00` floor slots, selects `kDungMap_Tab16` for positive and negative floor numbers, applies the caller mask, and writes the two current-floor label tiles into `messaging_buf`. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the buffer-writer port. |

## 2026-05-29 Dungeon Map Room Marker Parity Pass

Scope: continue the dungeon map empty-body audit with the marker setup routine
that follows room layout drawing.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `DungeonMap_DrawRoomMarkers` | `crates/zelda3/src/messaging.rs` | fixed | Ported room remapping via `kDungMap_Tab21/22`, current-room scan over `GetDungmapFloorLayout`, Link-position offsets into `dungmap_var3/5/6`, boss/target marker search with `kDungMap_Tab24/25/28`, signed floor offset into `dungmap_var8`, and the C state updates `overworld_map_state++`, `INIDISP_copy = 0`, `dungmap_init_state = 0`. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the marker setup port. |

## 2026-05-29 Dungeon Map Layout and Sprite Parity Pass

Scope: finish the dungeon map renderer stubs that consume the marker state and
write the visible room layout plus map OAM objects.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `DungeonMap_DrawDungeonLayout` / `DungeonMap_DrawSingleRowOfRooms` | `crates/zelda3/src/messaging.rs` | fixed | Ported the five-row layout renderer, full `kDungMap_Tab23` 2x2-room tile table, map/compass upper-bit checks, saved room edge-mask lookup, `GetOtherDungmapInfo` count scan, and the four tile writes per room into `messaging_buf`. |
| `DungeonMap_DrawSprites` group | `crates/zelda3/src/messaging.rs` | fixed | Ported Link pointer, blinking room indicator, location markers, floor number OAM, floor blinker, boss icon, and boss-floor icon using the C `kDungMap_Tab29` through `kDungMap_Tab38` data and `SetOamPlain` semantics. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the layout/sprite port. |

## 2026-05-29 Unsupported Path and Garnish DMA Parity Pass

Scope: classify the remaining empty-body scan hits and take the small real
parity fixes without changing confirmed C no-ops.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module_Messaging_6` | `crates/zelda3/src/messaging.rs` | fixed | C is `assert(0)`, so Rust now panics with a named unsupported-path message instead of silently returning. |
| `Dungeon_UpdateTileMapWithCommonTile` garnish callers | `crates/zelda3/src/sprite.rs` | fixed | Rewired the garnish-local wrapper to the canonical `Dungeon_UpdateTileMapWithCommonTile` port, covering Trinexx ice breath and crumble-tile map updates. |
| verification | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/sprite.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the unsupported-path and garnish-DMA port. |

## 2026-05-29 Dungeon Push Block Parity Pass

Scope: follow the remaining dungeon empty-body scan into the movable-block path,
which is playability-critical in dungeon rooms.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_PushBlock_Handler` | `crates/zelda3/src/dungeon.rs` | fixed | Replaced the placeholder index assignment with the C loop from `dung_misc_objs_index` to `dung_index_of_torches_start`, including state 1 draw/move/start-slide setup, state 2 slide/pit transition, state 4 falling animation, and `dung_misc_objs_index += 2`. |
| `index_of_changable_dungeon_objs` users | `crates/zelda3/src/dungeon.rs` | fixed | Corrected push-block helper reads/writes to treat `index_of_changable_dungeon_objs` as the C `uint8[2]` array instead of reading word pairs at `+0` and `+2`. |
| `Sprite_Dungeon_DrawAllPushBlocks` / `OrientLampLightCone` wrappers | `crates/zelda3/src/dungeon.rs` | fixed | Wired the lowercase module-path wrappers to the already-ported push-block draw and lamp-cone routines, matching the C module call sites. |
| verification | `crates/zelda3/src/dungeon.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the push-block port. |

## 2026-05-29 Exploding Wall Cleanup Parity Pass

Scope: continue the dungeon empty-body scan with the blast-wall cleanup routine
called from the main dungeon module while `dung_unk_blast_walls_2` is nonzero.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Dungeon_ClearAwayExplodingWall` | `crates/zelda3/src/dungeon.rs` | fixed | Ported the C state machine around `messaging_buf[0] == 6`: immobilize Link, clear scratch words, set `dung_cur_door_idx`, decrement the door tilemap address, call the already-ported blast-wall draw/stripe helpers, force NMI core update disable, advance `dung_unk_blast_walls_2`, mark opened door bits at terminal count 21, set quadrant blast flags, reload blast-wall attrs, flag room quadrants, and clear immobilization. |
| verification | `crates/zelda3/src/dungeon.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `git diff --check` pass after the exploding-wall cleanup port. |

## 2026-05-29 Cape, Byrna, and Net Item Effects Pass

Scope: continue player Y-item parity after the bottle, medallion, mirror, and
hookshot passes, focusing on visible/sound side effects that affect playable
feedback.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LinkItem_Cape` / `Link_ForceUnequipCape` | `crates/zelda3/src/player.rs` | fixed | Restored the C cape poof ancilla on equip and forced unequip, and routed equip/fail/unequip sounds through `Ancilla_Sfx2_Near` instead of direct pan writes. |
| `Player_CheckHandleCapeStuff` / passive lift check | `crates/zelda3/src/player.rs` | reviewed | Rust already matched the active-item gate, depletion counter reset, magic drain, forced-unequip behavior, and misc-bugfix grab-wall passive call. |
| `LinkItem_CaneOfSomaria` | `crates/zelda3/src/player.rs` | reviewed | Rust already matched the C doorway/platform gate, existing-block magic bypass, bugfix-specific button-mask behavior, Somaria block constructor, refund semantics, and animation cleanup. |
| `LinkItem_CaneOfByrna` | `crates/zelda3/src/player.rs` | fixed | Corrected the startup spark constructor from type `$31` to the C `$30`, and routed the activation sound through `Ancilla_Sfx3_Near(42)`. |
| `LinkItem_Net` | `crates/zelda3/src/player.rs` | fixed | Routed the initial swing sound through `Ancilla_Sfx2_Near(50)` while preserving the C animation-table and cleanup behavior. |

## 2026-05-29 SFX Interpreter and Render Oracle Pass

Scope: respond to audible title-screen SFX corruption and make visual parity
checks fail at the first differing rendered frame instead of only checking the
last requested frame.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sfx_ChannelTick` | `crates/zelda3/src/spc_player.rs` | fixed | Replaced the one-byte SFX stub with the C parser for note length, optional per-note volume, SFX instrument command `$e0`, pitch-slide commands `$f9/$f1`, loop command `$ff`, delayed key-off, and note-continuation pitch fading. |
| `Port1_Play_Inner` / `Port1_StartNewSound` | `crates/zelda3/src/spc_player.rs` | fixed | Restored the C two-channel port-1 fanout through channels 7 and 6, channel-67 volume fade counter, echo-mask behavior, and port2/port3 channel masking during port1 playback. |
| `Port2_StartNewSound` / `Port3_StartNewSound` | `crates/zelda3/src/spc_player.rs` | fixed | Restored the C three-frame SFX countdown before loading pointer tables `$1820` and `$191c`, including per-channel DSP register index setup and continuation ticks. |
| `--compare-lockstep-render` | `zelda3-bin/src/main.rs` | fixed | The render oracle now draws and compares both lockstep PPU states after each frame, exits on the first pixel mismatch, and reports frame, pixel coordinate, RGBA bytes, input, run target, and compact trace state for both sides. |
| verification | `crates/zelda3/src/spc_player.rs`, `zelda3-bin/src/main.rs` | verified | `cargo check -p snes -p zelda3 -p zelda3-bin --tests` and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass after the port. |

## 2026-05-29 SPC Debug Compare Pass

Scope: continue audio-oracle readiness by replacing a placeholder that would
otherwise make SPC implementation comparison silently pass.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `CompareSpcImpls` | `crates/zelda3/src/spc_player.rs` | fixed | Ported the C debug comparator shape: copy sequencer variables into SPC RAM, normalize the known APU-owned scratch/DSP/stack ranges, compare `$0000..$bfff`, compare DSP write-history streams, print bounded RAM evidence plus DSP write deltas, and clear histories on success. |
| verification | `crates/zelda3/src/spc_player.rs` | verified | `cargo check -p zelda3 --tests` passes after the comparator port. |

## 2026-05-29 Playable Lockstep Diff Harness Pass

Scope: add a usable "run it and stop at the first divergence" path for
interactive parity debugging.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| playable lockstep host | `zelda3-bin/src/main.rs` | added | Added `--play-lockstep <rom> [frames]`, which opens the playable frontend, polls real input, advances the SNES oracle and Rust game together, exits on state/render divergence, presents the verified Rust frame, and queues Rust audio. The frontend is now the native `winit`/`pixels`/`cpal` host. |
| audio command oracle | `crates/zelda3/src/audio.rs`, `zelda3-bin/src/main.rs` | superseded | The earlier C-lockstep APUI command-port comparison was removed from playable/render lockstep because the C harness is not a sample-producing full SPC/DSP oracle. Exact command/sample audio parity now belongs to the external snes9x oracle path. |
| SDL dummy verification | `crates/platform/src/lib.rs` | superseded | This headless SDL verification hook applied before the later native-host replacement. Current headless checks use non-windowed smoke/lockstep commands or the snes9x oracle path. |
| verification | `zelda3-bin/src/main.rs`, `crates/platform/src/lib.rs`, `crates/zelda3/src/audio.rs` | verified | `cargo check -p platform -p snes -p zelda3 -p zelda3-bin --tests`, `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run -p zelda3-bin -- --play-lockstep /path/to/zelda3.sfc 5`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass. |

## 2026-05-29 Link DMA Source Table Parity Pass

Scope: remove a graphics-path shortcut in `NMI_PrepareSprites` that could
return the wrong player/item tile DMA source for most Link animation indices.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `kLinkDmaSources1[303]` / `kLinkDmaSources2[303]` | `crates/zelda3/src/misc.rs` | fixed | Replaced sparse match/default helpers with the full 303-entry C tables from `misc.c`, and now indexes them through the same bounded table helper used by the rest of the DMA sources. |
| verification | `crates/zelda3/src/misc.rs` | verified | A direct extraction check reports `LINK_DMA_SOURCES1: c=303 rust=303 equal=True` and `LINK_DMA_SOURCES2: c=303 rust=303 equal=True`; `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass. |

## 2026-05-29 Overworld Scroll Assert-Path Pass

Scope: keep impossible overworld scroll directions from silently resetting the
submodule when C would assert.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `CreateInitialNewScreenMapToScroll` default branches | `crates/zelda3/src/overworld.rs` | fixed | The small-map and big-map invalid-direction branches now panic with the bad direction value, matching the C `assert(0)` behavior instead of silently setting `submodule_index = 0`. |
| `OverworldTransitionScrollAndLoadMap` default branch | `crates/zelda3/src/overworld.rs` | fixed | The transition stripe loader now panics on invalid `overworld_screen_trans_dir_bits2`, matching the C `assert(0)` path. |
| verification | `crates/zelda3/src/overworld.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass. |

## 2026-05-29 Interface and Flute Menu Dispatch Parity Pass

Scope: remove messaging/interface dispatch shortcuts that could run a different
state machine than C.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Module_Messaging_0` / `RunInterface` | `crates/zelda3/src/messaging.rs` | fixed | `Module_Messaging_0` now panics like the C `assert(0)` path, `RunInterface` dispatches submodule 0 to that assert path, and out-of-range submodules now panic instead of being silently ignored. |
| `Module0E_0A_FluteMenu` dispatch | `crates/zelda3/src/messaging.rs` | fixed | Restored the C state dispatch order for states 0 through 9: fade out, reset bird travel slot and load light-world map, load sprite graphics, brighten, initialize `some_menu_ctr`, handle selection, restore graphics, load selected screen, load overlay/map, and fade in/quack. Invalid states now panic like C. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass. |

## 2026-05-29 Overworld Map Dispatch Timing Pass

Scope: remove overworld-map dispatch and fade/brighten timing shortcuts that
could skip the C restore state or advance on the wrong frame.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Messaging_OverworldMap` dispatch | `crates/zelda3/src/messaging.rs` | fixed | State 6 now runs `WorldMap_RestoreGraphics`, state 7 exits, and out-of-range states no-op like the C switch with no default instead of collapsing into exit. |
| `WorldMap_FadeOut` / `WorldMap_Brighten` timing | `crates/zelda3/src/messaging.rs` | fixed | Restored the C pre-decrement and pre-increment behavior for `INIDISP_copy`, so fade-out advances when the decrement reaches zero and brighten advances when the increment reaches 15. |
| verification | `crates/zelda3/src/messaging.rs` | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass. |

## 2026-05-29 Overworld Map Side-Effect Pass

Scope: replace the remaining simplified overworld-map state-machine bodies with
the C register, palette, HDMA, audio-command, and module handoff side effects.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `WorldMap_FadeOut` | `crates/zelda3/src/messaging.rs` | fixed | Restored the HDMA/TM/TS/BG scroll/CGWSEL backups, force blank, mosaic, Link DMA index, special-exit coordinate backup, early-progress color math, SFX, music, and mode-7 register writes. |
| `WorldMap_PlayerControl` / `DidPressButtonForMap` | `crates/zelda3/src/messaging.rs` | fixed | Restored the HUD-item dependent X/select input gate, zoom toggle debounce, HDMA table reset, mode-7 zoom timer, BG1/M7 scroll calculations, directional pan behavior, and sprite handling order. |
| `WorldMap_RestoreGraphics` / `Attract_SetUpConclusionHDMA` | `crates/zelda3/src/messaging.rs` | fixed | Replaced the default-graphics shortcut with the C fade-down, force blank, palette restore, CGWSEL/TM/TS/BG scroll restore, conclusion HDMA setup, BGMODE 9, and NMI core-update release. |
| `WorldMap_ExitMap` / `BirdTravel_Finish_Doit` | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/ancilla.rs` | fixed | Restored the C module/submodule handoff, map state reset, HDMA restore, VRAM upload reset, palette flags, map ambient/music commands, bird-travel ancilla spawn, and sprite tick. |
| `WorldMap_SetUpHDMA` | `crates/zelda3/src/load_gfx.rs` | fixed | Restored the C main-module, map submodule, zoomed/unzoomed branch behavior including dynamic map flags, timers, map-center scroll math, and HDMA table selection. |
| verification | map render path | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass after the side-effect port. |

## 2026-05-29 Overworld Map Load Helper Pass

Scope: finish the small load-state helpers adjacent to the overworld-map
state-machine body port.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `WorldMap_LoadDarkWorldMap` | `crates/zelda3/src/messaging.rs` | fixed | Restored the C dark-world screen gate, copies asset 68 into the `uvram` staging buffer only for dark-world screens, sets NMI subroutine 21, and otherwise only advances the map state. |
| `WorldMap_LoadSpriteGFX` | `crates/zelda3/src/messaging.rs` | fixed | Replaced the common-sprite shortcut with the C `load_chr_halfslot_even_odd = $10`, `Graphics_LoadChrHalfSlot`, clear-halfslot, and state-advance sequence. |
| verification | map render path | verified | `cargo check -p zelda3 --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200`, and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 300` pass after the helper port. |

## 2026-05-29 Opening Render and Audio Oracle Pass

Scope: make the early title/opening route compare rendered audio samples and fix
the first render mismatch found in the attract sequence.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Attract_InitGraphics` HDMA setup | `crates/zelda3/src/attract.rs` | fixed | Restored the missing C `HdmaSetup(0xCFA87, 0xCFA94, 1, WH0, WH2, 0)` before setting `HDMAEN_copy = $c0`. This was the root cause of a render mismatch at frame 1748: Rust had HDMA enabled but DMA channels 6/7 still pointed at `$ff:ffff`. |
| render divergence diagnostics | `zelda3-bin/src/main.rs` | added | Render diffs now include PPU/DMA summaries for both Rust and the SNES oracle, including HDMA state, DMA6/7 descriptors, key BG state, color math, CGRAM samples, and VRAM samples. |
| rendered audio sample oracle | `zelda3-bin/src/main.rs` | superseded | The C-lockstep sample comparator was later removed because the SNES oracle APU buffer in this harness is not a true sample reference. Sample-exact comparison now lives in `--compare-snes9x-oracle`. |
| host audio discard timing | `zelda3-bin/src/main.rs` | fixed | Moved `ZeldaDiscardUnusedAudioFrames` equivalent out of the per-block prefill loop so startup queue prefill mirrors the C SDL callback: render one or more audio blocks, then discard stale APU writes once at the end of that audio service pass. This avoids dropping queued startup writes between prefill blocks. |
| verification | opening route | verified | `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2500 --input-script scripts/inputs/title-start.txt`, `... 12000 --input-script scripts/inputs/opening-uncle-message-advance.txt`, and `... 45610 --input-script scripts/inputs/opening-uncle-message-extended-move.txt` pass with zero pixel mismatches and no audio command/sample divergence. The exploratory pot/exit route was not kept because it was still in the attract/telepathy state, not free gameplay. |

## 2026-05-30 Startup Audio and Dungeon Exit Assert Parity Pass

Scope: respond to the reported one-time startup scratch and continue the
playable dungeon pot/exit slice by checking liftable replacement and indoor
edge-transition helpers directly against `dungeon.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| audio startup timing | `crates/platform/src/lib.rs` | superseded | This SDL startup-timing fix predates the native-host replacement. The current startup mismatch is not treated as solved by this row; it is tracked in the later snes9x startup-audio timing rows. |
| `Dungeon_LiftAndReplaceLiftable` | `crates/zelda3/src/dungeon.rs` | fixed | Restored the C `assert((attr & 0x70) == 0x70)` before masking liftable tile attributes to the low nibble. This keeps invalid pot/liftable replacement state from being silently treated as a normal replacement object. |
| `Dungeon_StartInterRoomTrans_Left/Right/Up/Down` | `crates/zelda3/src/dungeon.rs` | fixed | Restored the C `assert(submodule_index == 0)` guard on each inter-room transition starter. |
| `Dungeon_HandleEdgeTransitionMovement` | `crates/zelda3/src/dungeon.rs` | fixed | Removed Rust's `dir & 3` masking and no-op default branch; invalid directions now assert like C's `assert(0)` path instead of being coerced to a valid transition. |
| verification | startup/dungeon slice | verified | `cargo check -p platform -p zelda3 -p zelda3-bin --tests`, `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2500 --input-script scripts/inputs/title-start.txt`, and `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo run -p zelda3-bin -- --play-lockstep /path/to/zelda3.sfc 5` pass after this slice. |

## 2026-05-30 Player Liftable Action Parity Pass

Scope: continue the reported pot pickup/carry/throw path by checking
`Link_HandleLiftables`, `Link_PerformThrow`, and `Link_APress_LiftCarryThrow`
directly against `player.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Link_PerformThrow` / `Link_APress_LiftCarryThrow` | `crates/zelda3/src/player.rs` | verified | Confirmed the sprite-slot search, indoor/overworld lift replacement calls, `kLink_Lift_tab` lookup, delayed large-object pickup path, animation timers, carry/throw state bits, A-button clear, and stop-animation cleanup against C. |
| `Link_HandleLiftables` glove/action table | `crates/zelda3/src/player.rs` | fixed | Removed Rust's extra bounds guard around `kGetBestActionToPerformOnTile_a`; liftable indices now index the table like C instead of silently leaving action unchanged for impossible values. |
| verification | player liftable slice | verified | `cargo check -p zelda3 -p zelda3-bin --tests` and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2500 --input-script scripts/inputs/title-start.txt` pass after this slice. |

## 2026-05-30 Indoor Camera and Door Transition Pass

Scope: continue the playable indoor exit path by checking camera movement,
door-page transitions, edge-screen transitions, and the exit-to-overworld
handoff directly against `player.c` and `dungeon.c`.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `HandleIndoorCameraAndDoors` | `crates/zelda3/src/player.rs` | verified | Confirmed the indoor-only gate, doorway branch to `HandleDoorTransitions`, and non-doorway camera movement branch match C. |
| `HandleDoorTransitions` | `crates/zelda3/src/player.rs` | verified | Confirmed page-delta clearing, misc-bugfix module/submodule guard, vertical/horizontal doorway delta math, state clears, and left/right/up/down dispatch match C. |
| `ApplyLinksMovementToCamera` and ground-handler double-call guard | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | Confirmed the camera-applied flag reset at ground handler entry, flag set in `ApplyLinksMovementToCamera`, high-byte safe-return deltas, quadrant adjust dispatch, and misc-bugfix early return before a second camera pass match C. |
| `Link_CheckForEdgeScreenTransition` / `Dungeon_TryScreenEdgeTransition` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/dungeon.rs` | verified | Confirmed incapacitated-state exclusion, recoil reset/coordinate restore, edge coordinate thresholds, direction selection, main-module guard, transition dispatch, and post-dispatch `submodule_index = 2` behavior match C. |
| `Dung_HandleExitToOverworld` | `crates/zelda3/src/dungeon.rs` | verified | Confirmed dungeon key/quadrant saves, saved-module handoff, `main_module_index = 15`, submodule resets, and torch/player reset helper call match C. |
| verification | indoor door/camera slice | verified | `cargo check -p zelda3 -p zelda3-bin --tests` and `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2500 --input-script scripts/inputs/title-start.txt` pass after this no-code parity pass. |

## 2026-05-30 Room Door Draw Dispatch and Host Audio Resample Pass

Scope: continue the indoor-exit visual path by checking room-door drawing
dispatch against `dungeon.c`, and reduce the first title-logo SFX scratch in
host playback without changing native 534-sample lockstep output.

| C surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| door position table indexing | `crates/zelda3/src/dungeon.rs` | fixed | Removed Rust's silent `.get(...).return` guards from the four cardinal door draw helpers. The C code indexes `kDoorPosition_*[pos]` directly, so invalid object data should fail loudly instead of skipping a door. |
| `RoomDraw_Door_North` | `crates/zelda3/src/dungeon.rs` | fixed | Restored the missing `kDoorType_LgExplosion` branch and the stair-mask locked door branch before high-range door dispatch. |
| `RoomDraw_Door_South` | `crates/zelda3/src/dungeon.rs` | fixed | Restored the C special cases for `EntranceLarge`, `EntranceLarge2`, `EntranceCave`, `EntranceCave2`, and `DoorType_4`, including BG2 priority copies and high-priority door part writes. |
| `RoomDraw_Door_West` / `RoomDraw_Door_East` | `crates/zelda3/src/dungeon.rs` | fixed | Verified the dispatch bodies and aligned table indexing behavior with C. |
| host DSP resampling | `crates/snes/src/apu.rs` | superseded | This row changed non-native host output from nearest-sample stepping to linear interpolation. The 2026-05-30 DSP host resampling re-audit below found that this was not C parity and restored nearest-sample stepping. |
| verification | door/audio slice | verified | `cargo check -p zelda3 -p zelda3-bin --tests`, `cargo run -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2500 --input-script scripts/inputs/title-start.txt`, and `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 1200` pass after this slice. |

## 2026-05-30 DSP Host Resampling Re-Audit

Scope: follow up on title-screen SFX still sounding garbled after music became
acceptable in the native host, comparing the host sample extraction path back
to C instead of tuning by ear.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `dsp_getSamples` non-534 output | `crates/snes/src/apu.rs` | fixed | C `../zelda3/snes/dsp.c` resamples by taking `sampleBuffer[(int)location]` for both mono and stereo output. Rust had drifted to linear interpolation for non-native output sizes, which changes short SFX transients while leaving exact 534-sample oracle blocks untouched. Restored nearest-sample stepping to match C for host output too. |
| startup SFX envelope | `zelda3-bin/src/main.rs` diagnostics | open | Re-running `--trace-startup-audio` after the C resampling fix still shows port-3 SFX `$0a` beginning at frame 2 with peak about 19k around frames 20-25. This fix removes a proven host-output parity drift, but the remaining title SFX mismatch still points to startup command timing / oracle alignment rather than the non-534 sample extraction formula alone. |

## 2026-05-30 Native Host Audio Queue Re-Audit

Scope: compare the native `cpal` queue depth against C's SDL audio callback,
because pre-rendering too many audio blocks after one video frame can advance
SFX playback ahead of the APUI command's visual frame.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| audio queue target depth | `crates/platform/src/lib.rs` | fixed | C opens SDL with a 2048-frame callback buffer and renders internal `g_frames_per_block = (534 * have.freq) / 32000` blocks only when the callback needs more bytes. Rust had targeted the larger of four internal blocks or two cpal device buffers, which could pre-render roughly 4-6 game-audio blocks on an empty queue. Reduced the target to one device callback (or one internal block, whichever is larger) and the limit to one additional callback/block. |
| verification | native host audio queue | verified-open | `cargo check -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 120`, and `cargo run -q -p zelda3-bin -- --trace-startup-audio /path/to/zelda3.sfc 40` pass. The startup trace still shows the known high-level `$0a` SFX envelope, so this fixes host over-buffering drift but does not close the external-oracle startup audio mismatch. |

## 2026-05-30 Mesen2 Trace Oracle Scaffold

Scope: preserve the Mesen2 oracle work in-repo instead of leaving it in `/tmp`,
so APUI/DSP timing traces can be repeated while working through audio parity.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| Mesen2 APUI/DSP trace script | `external/mesen2-oracle/trace_apui_dsp.lua` | added | Added a Lua script that uses Mesen2 memory callbacks for S-CPU writes to `$2140-$2143` and SPC writes to `$f2/$f3`, plus `startFrame`, `endFrame`, and `nmi` event callbacks. The intended output is JSONL with frame numbers, APUI values, selected DSP register, and DSP data values. |
| Mesen2 oracle notes | `external/mesen2-oracle/README.md` | added | Documented the intended Mesen2 role: trace/debug oracle for APUI and DSP write timing, while snes9x/ares remain the final audio/video oracle. Also included startup comparison steps against Rust `--trace-startup-audio`. |
| verification | Mesen2 scaffold | verified-open | Upstream Mesen2 Lua docs confirm `addMemoryCallback`, `addEventCallback`, `callbackType.write`, `cpuType.snes`, `cpuType.spc`, `memType.snesMemory`, `memType.spcMemory`, `eventType.startFrame`, `eventType.endFrame`, `eventType.nmi`, and `emu.stop`. `luac -p external/mesen2-oracle/trace_apui_dsp.lua` passes after installing Homebrew Lua. The script still needs a live Mesen2 run against the ROM to validate callback behavior in the emulator. |

## 2026-05-30 snes9x Startup Audio Oracle Harness

Scope: add an independent emulator oracle for exact audio/video parity checks,
because the C-port high-level SPC player and lockstep oracle can agree with
each other while still disagreeing with snes9x/hardware startup behavior.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| libretro snes9x harness | `zelda3-bin/src/main.rs` | added | Added a minimal libretro core loader that calls `retro_run()` per frame and captures audio/video callback buffers from the official snes9x Accuracy core. This is a development oracle, not the playable runtime. |
| startup audio comparison | `zelda3-bin/src/main.rs` | added | Added `--compare-snes9x-startup-audio <snes9x_libretro.dylib> <rom> [frames]`, which prints Rust-vs-snes9x onset frames, max peaks, frame-window envelopes, and Rust SPC debug summaries. |
| SFX channel diagnostics | `crates/zelda3/src/spc_player.rs` | added | Extended `spc_player_debug_summary` with channel 6/7 SFX sound id, length, instrument, sound pointer, pitch, pan, and pitch-slide state so the startup scratch can be traced to exact SFX data. |
| startup SFX finding | audio parity | open | The first startup sound is not snes9x-equivalent yet. With no input over 180 frames, Rust starts port-3 SFX `$0a` at frame 0, crosses peak threshold at frame 2, and reaches peak 19113 at frame 21; snes9x stays silent until frame 85 and peaks at 4544. Some of the 83-frame onset delta is expected reset/bootstrap timeline mismatch between `load_play_state` and ROM power-on, but the Rust high-amplitude tail still proves the current high-level SPC path is not exact enough for final audio parity. |
| verification | oracle harness | verified | `cargo check -p zelda3-bin` passes. `cargo run -p zelda3-bin -- --compare-snes9x-startup-audio /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 180` captures 180/180 snes9x video frames at 256x224 and reports the startup audio mismatch above. |

## 2026-05-30 Native Host Rewrite

Scope: remove SDL from the playable host and dependency graph while keeping
snes9x/libretro as the exact external oracle for audio/video parity work.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| playable host | `crates/platform/src/lib.rs` | fixed | Replaced the SDL window/input/audio implementation with `winit` event polling, `pixels` frame presentation, and `cpal` output buffering. The host still exposes the same `Frontend` operations used by `zelda3-bin`, so game and PPU/APU logic remain isolated from the windowing stack. |
| SDL dependency removal | `crates/platform`, `crates/zelda3` | fixed | Removed `sdl2` from `platform`, deleted the Homebrew SDL build shim, removed the unused optional OpenGL/SDL renderer feature from `zelda3`, and deleted the legacy `opengl.rs`/`glsl_shader.rs` renderer files. |
| snes9x oracle | `zelda3-bin/src/main.rs` | preserved | The libretro snes9x diagnostic path is unchanged and remains separate from the playable host. It is used to compare Rust output against real ROM execution, not to render playable output. |
| verification | native host | verified | `cargo check -p platform -p zelda3 -p zelda3-bin --tests`, `cargo run -p zelda3-bin -- --smoke-render /path/to/zelda3.sfc 120`, and `cargo run -p zelda3-bin -- --compare-snes9x-startup-audio /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 120` pass. `cargo tree -i sdl2` reports no matching package. A short escalated playable launch initialized successfully outside the sandbox; the sandboxed launch failed before that with no `wgpu::Adapter`, which is expected for this environment rather than the normal desktop path. |

## 2026-05-30 Startup SPC SFX and Full-APU Bringup Probe

Scope: continue the first-startup-scratch investigation by manually comparing
the implicated C high-level SPC routines and checking whether the full Rust APU
can already replace the high-level player for playable audio.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Sfx_ChannelTick` | `crates/zelda3/src/spc_player.rs` | verified | Compared against `../zelda3/src/spc_player.c`. The command parser, note-length handling, volume/pan branches, instrument command `$e0`, pitch-slide commands `$f9/$f1`, loop command `$ff`, key-on/key-off behavior, and final `sfx_sound_ptr` update match the C flow. |
| `Port3_StartNewSound` / `Port3_HandleCmd` | `crates/zelda3/src/spc_player.rs` | verified | Compared against C for the port-3 startup SFX path. The channel countdown, `$191c` sound-pointer lookup, `$199a` command chaining, channel allocation order, pan/sound masking, and KOF write behavior match C. This means the startup SFX `$0a` scratch is not explained by an obvious Rust-vs-C drift in the port-3 high-level SFX logic. |
| full APU seed diagnostic | `zelda3-bin/src/main.rs`, `crates/zelda3/src/audio.rs` | added | Added `--compare-startup-apu-impls <rom> [frames]`, which seeds `snes::apu::ApuState` from the high-level SPC RAM after the first frame and runs it beside the current high-level player. This tests whether the cycle APU path can be used as a native audio replacement before doing a larger routing change. |
| full APU finding | audio parity | open | The seeded full APU is currently silent and falls into the IPL boot handshake (`out=[aa,bb,00,00]`, `pc=ffcf/ffd2`, `dsp_writes=0`) while the high-level player produces SFX `$0a`. So the high-level SPC RAM is enough for the C-style interpreter tables, but not a runnable SPC program image for the cycle APU. Exact snes9x-like audio likely needs boot/uploading the ROM's real SPC program into `ApuState`, not just copying the high-level player's table RAM. |
| verification | startup audio slice | verified | `cargo check -p zelda3-bin` and `cargo run -p zelda3-bin -- --compare-startup-apu-impls /path/to/zelda3.sfc 20` pass. The 20-frame probe shows high-level onset at frame 2 and full-APU silence, with full-APU state proving it is sitting in IPL handshake rather than writing DSP registers. |

## 2026-05-30 Song Bank Upload Layout Probe

Scope: prove whether the extracted song-bank assets can boot the full cycle APU,
or whether they are only data tables for the C high-level SPC interpreter.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `LoadSongBank` / `load_song_bank` | `crates/zelda3/src/audio.rs`, `crates/zelda3/src/misc.rs` | verified | C `LoadSongBank` only locks audio and calls `SpcPlayer_Upload(g_zenv.player, p)`. Rust `load_song_bank_asset` selects assets 0/1/2 for intro/indoor/ending exactly like C and calls `load_song_bank`, which calls `spc_player_upload`. |
| song-bank layout diagnostic | `zelda3-bin/src/main.rs`, `crates/zelda3/src/misc.rs` | added | Added `--trace-song-bank <rom> [asset-index]`, which parses the same upload-block format as `SpcPlayer_Upload`, reports block count/target range, and prints key runnable-code locations after applying the upload to a blank SPC RAM image. |
| song-bank runnable-code finding | audio parity | open | The extracted banks are not full SPC program images. Asset 0: size 50066, 8 blocks, first target `$17c0`, range `$17c0-$fdad`, reset `$0000`, `$0800/$0878` all zero. Asset 1: first target `$2b00`, reset `$0000`, `$0800/$0878` zero. Asset 2: first target `$2900`, reset `$0000`, `$0800/$0878` zero. This confirms the full APU cannot be made exact by simply copying the C high-level song bank assets into RAM. |
| verification | song-bank probe | verified | `cargo check -p zelda3-bin`, `cargo run -p zelda3-bin -- --trace-song-bank /path/to/zelda3.sfc 0`, `... 1`, and `... 2` pass and produce the layout facts above. |

## 2026-05-30 Raw ROM APU Upload Trace

Scope: find a usable exact-audio bringup path by running the unpatched ROM
startup through the Rust SNES/APU core instead of the lockstep oracle, because
the lockstep ROM patch intentionally skips `LoadSongBank`.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| raw ROM APU diagnostic | `zelda3-bin/src/main.rs` | added | Added `--trace-rom-apu-upload <rom> [opcode-budget] [apu-cycles-per-cpu-cycle]`, which loads the ROM without the lockstep patches, seeds the reset vector, advances CPU/DMA plus the full APU core, and reports APU upload address, payload coverage, IPL ROM visibility, SPC PC, and DSP write milestones. |
| upload finding | audio parity | open | The unpatched ROM does produce the runnable SPC image that the extracted high-level song banks do not. With the default `0.286` APU-cycle ratio, it starts the IPL upload near opcode 2.1k, fills `$4000+`, then uploads `$0800`/`$0878`, `$17c0`, `$d000`, and `$2b00` regions. Around opcode 1.436M, SPC leaves IPL (`spc=$0e49` then `$0862`), IPL ROM becomes disabled, and DSP writes begin. |
| implication | audio parity | open | Exact audio should be based on this raw ROM APU image/bootstrap path or a captured equivalent state, not on `SpcPlayer_Upload` song-bank data alone. This gives us a concrete native Rust route to replace the current high-level SPC player while keeping snes9x as the external oracle. |
| verification | raw ROM APU trace | verified | `cargo check -p zelda3-bin` passes. `cargo run -p zelda3-bin -- --trace-rom-apu-upload /path/to/zelda3.sfc 1500000` reaches `rom=false`, `spc=$0862`, `payload_nz=51454`, `nz0800=238`, `nz0878=72`, and DSP write milestones through 128 writes by 1.5M opcodes. |

## 2026-05-30 Raw ROM APU Bootstrap Checkpoint

Scope: turn the raw ROM APU startup discovery into a reusable artifact and
test whether the captured full-cycle APU behaves like the current high-level
SPC player when fed the same playable-state port writes.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| bootstrap checkpoint | `zelda3-bin/src/main.rs` | added | Added `--capture-rom-apu-bootstrap <rom> <out.z3apu> [opcode-budget] [apu-cycles-per-cpu-cycle]`. It runs the unpatched ROM until IPL is disabled, SPC is executing in `$0800-$0fff`, and at least 16 DSP writes have occurred, then serializes `snes::apu::ApuState` plus capture metadata. |
| bootstrap comparison | `zelda3-bin/src/main.rs` | added | Added `--compare-bootstrap-apu-startup <rom> <bootstrap.z3apu> [frames]`, which loads `ZeldaState`, feeds its per-frame APU port writes into the captured full APU, and prints the high-level-vs-full-cycle audio envelopes. |
| startup SFX finding | audio parity | open | Capturing `/private/tmp/zelda3-startup.z3apu` stops at opcode 1,438,619 with `cpu=$00:8036`, `spc=$0862`, `rom=false`, `payload_nz=51454`, and `dsp_writes=16`. Comparing 120 startup frames gives `high_onset=Some(2)`, `full_onset=Some(2)`, `high_max=Some((21, 19113))`, and `full_max=Some((22, 18776))`. This means the real SPC program and current high-level player agree closely once they receive the same early port write sequence; the remaining snes9x startup mismatch is more likely in playable-state timing/control entry than in the SFX interpreter itself. |
| verification | bootstrap checkpoint | verified | `cargo check -p zelda3-bin`, `cargo run -p zelda3-bin -- --capture-rom-apu-bootstrap /path/to/zelda3.sfc /private/tmp/zelda3-startup.z3apu 1500000`, and `cargo run -p zelda3-bin -- --compare-bootstrap-apu-startup /path/to/zelda3.sfc /private/tmp/zelda3-startup.z3apu 120` pass. |

## 2026-05-30 APUI00 Music-Playing Mirror Parity

Scope: continue the manual C-vs-Rust audio audit without using the progress
script, focusing on the startup/control path that feeds the first port-3 SFX.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `ZeldaIsMusicPlaying` | `crates/zelda3/src/audio.rs` | fixed | C `../zelda3/src/audio.c` returns `g_zenv.player->port_to_snes[0] != 0` when MSU is idle. Rust had drifted by OR-ing in the previous `RAM_APUI00` mirror value, which could make the mirror sticky. Removed that fallback so the Rust path matches C. |
| snes9x video offset check | `zelda3-bin/src/main.rs` | verified | `cargo run -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 10 --skip-snes9x-frames 85 --ignore-audio` still passes with no enabled video diff. |
| startup audio timing | audio parity | open | The APUI00 mirror fix does not resolve the snes9x startup audio mismatch. With `--skip-snes9x-frames 85 --ignore-video`, compared frame 0 still diverges at sample 642: Rust is silent while snes9x has sample value 2 and peak 4544. Rust has queued ports `[00, 00, 00, 0a]`, so the next work is still exact command timing/interleaving rather than the C high-level SFX parser. |
| full/bootstrap APU checks | audio parity | verified-open | `cargo run -p zelda3-bin -- --compare-startup-apu-impls /path/to/zelda3.sfc 20` passes and still shows the seeded blank full APU stuck in IPL while high-level audio starts at frame 2. `cargo run -p zelda3-bin -- --compare-bootstrap-apu-startup /path/to/zelda3.sfc /private/tmp/zelda3-startup.z3apu 20` passes and shows the bootstrapped full APU and high-level player both start at frame 2 when fed the same playable port writes. |
| verification | APUI00 mirror slice | verified | `cargo check -p zelda3-bin` passes. The focused snes9x and APU diagnostics above pass or fail in the expected open-audio-divergence way after the C-parity fix. |

## 2026-05-30 Startup Audio/Video Offset Probe

Scope: test whether the startup snes9x audio mismatch is just the video skip
offset being wrong, or whether audio command timing diverges inside the
otherwise video-aligned frame window.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| C frame wrapper | `crates/zelda3/src/zelda_rtl.rs` | verified | Re-read C `ZeldaRunFrame` lines 690-751 against Rust `zelda_run_frame`. The input opposite-direction masking, frame counter, replay/state-recorder path, APUI00 mirror patch, bug/feature patching, `run_what` selection, emu/internal dispatch, and final `ZeldaPushApuState` ordering match after the APUI00 helper fix. |
| C audio queue helpers | `crates/zelda3/src/audio.rs` | verified | Re-read C `ZeldaPushApuState`, `ZeldaPopApuState`, `ZeldaDiscardUnusedAudioFrames`, and `ZeldaRenderAudio` against Rust. The 16-entry queued port ring, pop-before-generate ordering, discard rule, and generate-then-sample flow match the high-level C model. |
| host audio scheduling | `zelda3-bin/src/main.rs`, `crates/platform/src/lib.rs` | open | C renders audio from SDL's callback after the device is unpaused, while the native host renders queued blocks after each game frame and then feeds cpal. This is host-scheduling compatible enough to play, but not cycle- or callback-identical. It can affect which queued APUI write lands inside a given video-frame audio block. |
| snes9x offset probe | `zelda3-bin/src/main.rs` | open | Audio-only probes with skips 82-88 show no single startup skip that proves combined audio/video parity. `--skip-snes9x-frames 82` passes only the first 3 audio frames, then diverges at frame 3 with both sides active but Rust's first nonzero sample later in the block. The video-aligned `--skip-snes9x-frames 85` still diverges immediately on audio. |
| implication | startup parity | open | The current evidence points to APUI command timing relative to the frame/audio boundary, not a simple fixed snes9x startup offset and not an obvious C-vs-Rust drift in the high-level SFX parser. The next useful check is a frame-level raw-ROM APU/input-port trace around the first `$0a` write, so we can align Rust playable command injection with the real ROM timing instead of guessing a delay. |
| verification | offset probe | verified-open | `cargo run -q -p zelda3-bin -- --compare-snes9x-oracle /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 10 --skip-snes9x-frames 82 --ignore-video` diverges at frame 3; `--skip-snes9x-frames 85 --ignore-video` diverges at frame 0; `--skip-snes9x-frames 85 --ignore-audio` remains the known passing video-offset check from the previous slice. |

## 2026-05-30 Raw-ROM Frame Timing Finding

Scope: follow the startup audio timing lead by extending the raw unpatched ROM
APU trace after bootstrap, looking for the first post-bootstrap APUI command
that should correspond to the startup port-3 SFX.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| raw unpatched ROM post-bootstrap trace | `zelda3-bin/src/main.rs` | open | Extending `--trace-rom-apu-upload` to 2,500,000 opcodes still does not see a post-bootstrap input-port command. After IPL disables and DSP writes reach 256, CPU remains around `$00:8034/$8036`, APU input ports stay `[00, 00, 00, 00]`, and SPC keeps running around `$0846-$1453`. |
| lockstep-vs-raw distinction | `crates/zelda3/src/zelda_cpu_infra.rs` | verified | The lockstep oracle does not advance the unpatched ROM with real scanline/NMI timing. It seeds PC/SP/DB/DP into known game-loop entry points (`$8034`, `$80d9`, poly path) and increments `snes.frames` after `run_emulated_snes_frame_checked`. That is correct for C-function parity, but it cannot answer when the real reset ROM writes `$2143=$0a` relative to a video/audio frame. |
| raw tracer limitation | `zelda3-bin/src/main.rs`, `crates/snes/src/snes.rs` | open | The raw tracer advances CPU opcodes, DMA, and APU cycles, but does not advance `Snes.h_pos`, `v_pos`, `frames`, vblank, NMI, or auto-joy timing. Once the real ROM reaches the main loop wait state, this leaves it waiting instead of producing frame-level APUI commands. |
| implication | oracle tooling | open | To use the Rust SNES core as a raw-ROM command-timing oracle, add a real timing stepper that advances CPU cycles into h/v position, vblank/NMI state, auto-joy, and `frames`, or reuse a snes9x-side trace that can expose APUI writes. Extending opcode budget alone is not sufficient. |
| verification | raw trace | verified-open | `cargo run -q -p zelda3-bin -- --trace-rom-apu-upload /path/to/zelda3.sfc 2500000` ends with `cpu=$00:8034`, `spc=$0876`, `rom=false`, `in=[00, 00, 00, 00]`, and `dsp_writes=256`; no post-bootstrap port-3 `$0a` command appears. |

## 2026-05-30 Bootstrapped APU Direct-Frame Trace

Scope: narrow startup audio parity by combining the real raw-ROM APU bootstrap
with the C-style direct-frame SNES entrypoints that the lockstep oracle uses.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| direct-frame APU diagnostic | `zelda3-bin/src/main.rs` | added | Added `--trace-bootstrap-apu-direct-frame <rom> [frames] [bootstrap-opcode-budget] [apu-cycles-per-cpu-cycle]`. It captures the unpatched ROM's full APU bootstrap, initializes the patched lockstep-oracle ROM, installs the captured `ApuState`, then runs the C-equivalent reset/main/NMI direct-frame loops while advancing the full APU after each CPU opcode. |
| first command finding | audio parity | verified-open | On frame 0, the high-level playable path and the bootstrapped direct-frame path both write the same startup APUI command: `high_ports=[00, 00, 00, 0a]`, `direct_in=[00, 00, 00, 0a]`. This confirms the remaining mismatch is not a wrong first command value. |
| timing finding | audio parity | open | The direct full APU remains silent through frame 2, starts on frame 3 at sample index 750, and by frame 9 goes silent again while the high-level player is still active. The trace also exposes direct-frame opcode discontinuities: frames 1-7 are about 10,686 opcodes each, but frame 8 jumps to 641,513 opcodes and frame 9 to 180,431. That makes the direct-frame/full-APU hybrid useful as a diagnostic, but not yet a stable replacement for snes9x timing. |
| implication | startup parity | open | The first command value matches, and the bootstrapped full APU can render the expected startup sound, but direct-frame CPU timing around the early intro is not frame-stable when paired with continuous APU stepping. The next stronger oracle should trace APUI writes from snes9x/libretro itself or implement raw SNES h/v/NMI timing before treating the full APU as exact per-video-frame audio. |
| verification | direct-frame trace | verified-open | `cargo check -p zelda3-bin` passes. `cargo run -q -p zelda3-bin -- --trace-bootstrap-apu-direct-frame /path/to/zelda3.sfc 12` captures bootstrap at opcode 1,438,619 (`cpu=$00:8036`, `spc=$0862`, `payload_nz=51454`, `dsp_writes=16`) and prints the command/timing facts above. |

## 2026-05-30 High-Level SPC Startup/SFX Audit

Scope: manually verify the C high-level SPC player slice involved in the first
startup port-3 sound, without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Port3_StartNewSound` / `port3_start_new_sound` | `crates/zelda3/src/spc_player.rs` | verified | C iterates channels 7 down to 0 by shifting `port3_current_bit`, sets `sfx_channel_index`, `dsp_register_index`, and `sfx_start_arg_pan`, then either continues an active SFX or arms the delayed sound pointer from `$191c + (sound - 1) * 2`. Rust matches the same order, bit shifts, countdown decrement, pointer table, and `Sfx_ChannelTick(..., is_continue)` split. |
| `Port3_AllocateChan` / shared `sfx_allocate_chan` | `crates/zelda3/src/spc_player.rs` | verified | C looks up echo flag at `$19d8 + (new_value_from_snes[3] & 0x3f)`, first reuses a matching active channel where `sfx_which_sound + sfx_pan == command`, then falls back to the first off channel scanning 7 down to 0. Rust factors this with port2 into `sfx_allocate_chan` and preserves the active mask, channel order, `current_bit`, `is_chan_on`, echo-mask, and `Sfx_MaybeDisableEcho` behavior. |
| `Port3_HandleCmd` / chained SFX command table | `crates/zelda3/src/spc_player.rs` | verified | C loops while port 3 has a nonzero command and not all channels are on, sets pan/sound/countdown, writes KOF, activates the channel, then chains `new_value_from_snes[3]` through `$199a + sound - 1`. Rust matches the loop condition, field writes, KOF ordering, and chained table lookup. |
| SPC loop timing wrapper | `crates/zelda3/src/spc_player.rs` | verified | C `Spc_Loop_Part2` gates port1/2/3 handling behind `sfx_timer_accum + (uint8)(ticks * 0x38) >= 256`, processes start/handle/read in port order 1, 2, 3, then advances echo timing. Rust matches the wrapping multiply, accumulator truncation, branch condition, port order, read-port placement, and echo increment rule. |
| init/upload/copy map | `crates/zelda3/src/spc_player.rs` | verified | C `Interrupt_Reset`, `SpcPlayer_Initialize`, `SpcPlayer_GenerateSamples`, `SpcPlayer_Upload`, and the `kSpcPlayer_Maps` copy table were re-read against Rust. The reset writes, initial loop part 1, sample-loop 64-cycle/timer behavior, upload block format, active-port clearing, and explicit copy offsets match the C code. Rust additionally guards null DSP pointers for safe cloning/destruction, but the live player path keeps a DSP just like C. |
| implication | audio parity | verified-open | This audit did not find a high-level SPC port-3/startup drift that would explain the scratch sound. Combined with the bootstrapped APU trace, the remaining suspect is still frame/audio timing against snes9x or raw-ROM h/v/NMI scheduling, not the C high-level port-3 interpreter itself. |

## 2026-05-30 snes9x Libretro Memory Probe

Scope: determine whether the existing snes9x libretro oracle can expose WRAM or
APUI-adjacent state through standard libretro memory APIs, before building a
larger timing tracer around it.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| libretro memory API bridge | `zelda3-bin/src/main.rs` | added | Added `--trace-snes9x-memory <snes9x-libretro.dylib> <rom> [frames]`, loading `retro_get_memory_data` and `retro_get_memory_size` from the snes9x core and reporting standard memory ids 0-7 plus startup frame audio/video sizes. |
| snes9x memory exposure | snes9x oracle | blocked-open | `/private/tmp/snes9x_libretro/snes9x_libretro.dylib` exports `retro_get_memory_data` and `retro_get_memory_size`, but returns size 0 / no pointer for SAVE_RAM, RTC, SYSTEM_RAM, VIDEO_RAM, and ids 4-7 after loading the Zelda ROM. It therefore cannot expose `kRam_APUI00`, WRAM digests, or APUI register timing through the standard memory API. |
| implication | startup parity | open | The existing libretro callback surface is sufficient for frame audio/video comparison, but not for APUI write tracing. Exact startup command timing against snes9x will require an instrumented snes9x/ares/mesen core with APUI write hooks, or adding real h/v/vblank/NMI/autojoy timing to the Rust raw-ROM SNES stepper. |
| verification | snes9x memory probe | verified-open | `cargo check -p zelda3-bin` passes. `cargo run -q -p zelda3-bin -- --trace-snes9x-memory /private/tmp/snes9x_libretro/snes9x_libretro.dylib /path/to/zelda3.sfc 20` reports geometry `256x224`, sample rate `48000`, all standard memory ids size 0, and no `apui00` value. |

## 2026-05-30 NMI Audio and VRAM Update Audit

Scope: manually verify the NMI surface that writes APUI01/APUI02/APUI03 and
feeds most renderer-visible VRAM/CGRAM/OAM updates, without using the progress
script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Interrupt_NMI` / `interrupt_nmi` ordering | `crates/zelda3/src/nmi.rs` | verified | C runs locked audio parts first, then if `nmi_boolean` is clear sets it, runs `NMI_DoUpdates`, and reads joypads. It then handles the NMI thread IRQ graphics/stack toggle and finally calls `WritePpuRegisters`. Rust preserves that order and the `thread_other_stack != 0x1f31 ? 0x1f31 : 0x01f2` toggle. |
| APUI sound-effect writes | `crates/zelda3/src/nmi.rs` | verified | C leaves APUI00 clearing commented out, applies music control only when nonzero and not already playing, clears `music_control`, handles ambient port 1 with the last-value zeroing rule, then writes APUI02/APUI03 unconditionally before clearing `sound_effect_1/2`. Rust matches the same order and APUI addresses `0x2141/0x2142/0x2143`. |
| core VRAM/CGRAM/OAM updates | `crates/zelda3/src/nmi.rs` | verified | C copies Link graphics from `kLinkGraphics`, runtime sprite/tile buffers from WRAM, optional bird tiles, animated tile data, HUD tile indices, CGRAM palette, and OAM. Rust maps the same asset index for Link graphics, the same WRAM source variables, the same bird-gated copies, the same animated tile copy, and converts C byte copies into equivalent u16 VRAM/OAM/CGRAM writes. |
| BG/stripe/tilemap update dispatch | `crates/zelda3/src/nmi.rs` | verified | C handles `nmi_load_bg_from_vram` cases 1-9, one-shot tilemap copy, `uvram` packet copies, then dispatches 25 NMI subroutines. Rust preserves the case mapping to RAM slices/assets 99-104, resets `vram_upload_offset` for case 1, clears `nmi_load_bg_from_vram`, implements packet modes `0x80/0x81`, clears flags, and dispatches the same 0-24 table. |
| NMI subroutines and VRAM byte semantics | `crates/zelda3/src/nmi.rs` | verified | C `memcpy` into `uint16` VRAM writes byte streams; Rust uses byte-to-word helpers for horizontal copies, vertical copies stepping by 32 words, low-byte-only copies, and stripe `memset`/copy paths. The destinations for BG3 text, OW scroll, subscreen overlays, BG char/object char DMA, dark world map, game-over text, peg tiles, star tiles, and polyhedral IRQ graphics match the C source. |
| PPU register writes | `crates/zelda3/src/nmi.rs` | verified | C `WritePpuRegisters` writes window/color/main-sub screen masks, BG1/2/3 scroll copies, INIDISP, MOSAIC, BGMODE, mode-7 fallback registers when mode is 7, and fixed BG tile base registers. Rust writes the same register sequence and values through `zelda_ppu_write`. |
| verification | NMI slice | verified | `cargo check -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 300`, and `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 30` pass. The render check reports `mismatched_pixels=0`, covering this slice's VRAM/CGRAM/OAM-visible startup behavior. |

## 2026-05-31 Ancilla Spin Spark Draw Assert Pass

Scope: compare the dirty `SpinSpark_Draw` assert change in
`crates/zelda3/src/ancilla.rs` directly against
`../zelda3/src/ancilla.c`.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `SpinSpark_Draw` table index guard | `crates/zelda3/src/ancilla.rs` | fixed | C computes `t = (ancilla_item_to_link[k] + offs) * 4`, asserts `t < 32`, then iterates four entries through `kInitialSpinSpark_Char/Flags/X/Y`. Rust now uses an unconditional `assert!(t < 32)` at the same point instead of a debug-only assert, preserving the C invalid-state stop in playable/release-style builds. |

## 2026-05-31 SNES CPU State Reset Pass

Scope: manually compare `crates/snes/src/cpu.rs` against
`../zelda3/snes/cpu.c` and `cpu.h`, without using the
progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Cpu` field layout owned by `CpuState` | `crates/snes/src/cpu.rs` | verified | Rust carries the same register, flag, interrupt, power-state, `cyclesUsed`, `spBreakpoint`, and `in_emu` fields as C `struct Cpu`, omitting only the C memory-handler pointer/type because Rust routes bus access through `Snes`. |
| `cpu_reset` state defaults | `crates/snes/src/cpu.rs` | fixed | C resets A/X/Y/DP/K/DB to zero, `SP` to `0x100`, `PC` from the reset vector, flags to `I/M/X/E` set, IRQ/NMI/wait/stop clear, `cyclesUsed = 0`, `spBreakpoint = 0`, and `in_emu = 0`. Rust now matches every state field owned by `CpuState`; `PC` is seeded by `Snes::cpu_seed_reset_vector()` after the cart is loaded. |
| flag pack/unpack | `crates/snes/src/cpu.rs`, `crates/snes/src/cpu_step.rs` | verified | Rust `pack_flags` matches C `cpu_getFlags` bit order `N V M X D I Z C`. Rust `cpu_set_flags` applies C `cpu_setFlags` post-processing: emulation mode forces M/X and page-1 stack, X flag truncates X/Y. |
| `cpu_saveload` byte range | `crates/snes/src/cpu.rs` | verified | C serializes from `Cpu.a` through `stopped`, stopping before `cyclesUsed`, then clears `spBreakpoint`. Rust serializes the same 27-byte native field range, leaves `cycles_used` and `in_emu` outside that block, and clears `sp_breakpoint` after save/load. |
| verification | SNES CPU state slice | verified | `cargo check -q -p snes -p zelda3 -p zelda3-bin` and `cargo test -q -p snes cpu` pass after the reset-default fix. |

## 2026-05-31 SNES Top-Level Bus/Saveload Pass

Scope: manually compare `crates/snes/src/snes.rs` against
`../zelda3/snes/snes.c`, `snes.h`, and the relevant
top-level API split from `snes_other.c`, without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| `Snes` owned state and reset | `crates/snes/src/snes.rs` | verified | Rust carries the same CPU/APU/PPU/DMA/cart/input state plus debug flags, frame timing, CPU-cycle accounting, IRQ/NMI state, auto-joypad state, multiply/divide registers, fast-mem flag, open-bus byte, WRAM, and WRAM data-port address. `Snes::reset` preserves C reset order (`cart`, `cpu`, `apu`, `dma`, `ppu`, inputs), hard-reset WRAM clearing, and every top-level default value. |
| C saveload top-level block | `crates/snes/src/snes.rs` | verified | C serializes components, then the native `hPos..openBus` range, WRAM, and `ramAdr`, and clears `disableHpos`. Rust serializes the same order and verified the ABI range with clang: `offsetof(hPos)=64`, `offsetof(openBus)=121`, so the top-level native block is 58 bytes as encoded by `SNES_CORE_SAVELOAD_SIZE`. |
| auto-joypad and B-bus | `crates/snes/src/snes.rs` | verified | `do_auto_joypad` matches C latch/cycle/unlatch and 16 serial reads into `portAutoRead[0..3]`. `read_b_bus`/`write_b_bus` preserve the C PPU range, APU port range plus APU catchup before writes, `$2180` WRAM data-port auto-increment, `$2181..$2183` address-byte masking, and open-bus fallback. |
| internal registers and address bus | `crates/snes/src/snes.rs` | verified | `$4210..$421f` reads, `$4200..$420d` writes, multiply/divide behavior including divide-by-zero, IRQ/NMI clear side effects, PPU latch edge, DMA/HDMA starts, WRAM mirrors, `$4016/$4017` serial reads, direct `$7e/$7f` WRAM, cart fallback, open-bus updates, and flat CPU access time of 6 match C. |
| debug and ROM-load split | `crates/snes/src/tracing.rs`, `crates/snes/src/loader.rs` | verified | C `snes_printCpuLine` is implemented as the Rust `Snes::print_cpu_line` extension in `tracing.rs`. C `snes_loadRom`/header scoring is owned by `loader.rs` and was covered by the existing SNES Cart/Loader pass, so this pass did not duplicate that surface in `snes.rs`. |
| verification | SNES top-level bus slice | verified | `cargo check -q -p snes -p zelda3 -p zelda3-bin` and `cargo test -q -p snes snes` pass after this source-comparison pass. |

## 2026-05-31 Poly Renderer Full-File Pass

Scope: manually compare `crates/zelda3/src/poly.rs` against
`../zelda3/src/poly.c` and `poly.h`, without using the
progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| static tables and model configs | `crates/zelda3/src/poly.rs` | verified | `kPolySinCos`, the two vertex arrays, polygon index/color streams, model config values, raster colors, and left/right masks match C. Rust stores typed `Vertex3`/`PolyConfig` values instead of pointer tables, but the indexed data and model selection are the same. |
| frame pipeline and shape setup | `crates/zelda3/src/poly.rs` | verified | `poly_run_frame` preserves C's empty-buffer, shape-pointer, rotation-matrix, operate-rotation, draw-polyhedron order. Shape setup writes the same `poly_var1`, `poly_tmp0`, vertex count, polygon count, and source pointer marker values. |
| rotation, projection, and divide | `crates/zelda3/src/poly.rs` | verified | Rust matches the C sin/cos lookup offsets, matrix coefficient formulas, reverse vertex walk, X/Y/Z source swizzle, projected array writes, and `Poly_Divide` sign/shift/divide behavior. |
| polygon draw and foreground color | `crates/zelda3/src/poly.rs` | fixed | Rust matches the C polygon stream parser, cross-product gate, color-mask write, and face draw call. This pass fixed `Polyhedral_SetForegroundColor` to promote `poly_tmp0` before shifting, matching C integer promotion instead of truncating as `u16` before the final `>> 8`. |
| face rasterization | `crates/zelda3/src/poly.rs` | verified | Rust matches C's minimum-Y scan, `poly_raster_dst_ptr` formula, edge setup calls, scanline pointer advance/wrap, blend masks, full-word fill loop, and left/right edge walkers including signed step termination and `uint8` Y-delta division. |
| header/call surface | `crates/zelda3/src/poly.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | All `poly.h` functions have Rust counterparts under `ZeldaState`, and `zelda_run_poly_loop` matches C's `intro_did_run_step && !nmi_flag_update_polyhedral` guard, frame call, clear, and update flag write. |
| verification | poly renderer slice | verified-open | `cargo fmt -p zelda3` and `cargo check -q -p zelda3 -p zelda3-bin` pass. `cargo test -q -p zelda3 poly` reports no matching tests, so runtime coverage for this slice remains direct source comparison plus compile only. |

## 2026-05-31 Types Header Compatibility Pass

Scope: manually compare `crates/zelda3/src/types.rs` against
`../zelda3/src/types.h`, without using the progress
script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| integer aliases and screen constants | `crates/zelda3/src/types.rs` | fixed | Rust aliases now cover C `uint8/int8/uint16/int16/uint32/int32/uint64/int64/uint`, and `ENABLE_LARGE_SCREEN = true` / `PPU_EXTRA_LEFT_RIGHT = 96` matches `kEnableLargeScreen` / `kPpuExtraLeftRight`. |
| C helper macros/functions | `crates/zelda3/src/types.rs` | fixed | `sign8`, `sign16`, `abs8`, `abs16`, `load24`, and `xy` already matched the C macro/helper semantics for Rust slice/indexed call sites. This pass added `int_min`, `int_max`, `uint_min`, `uint_max`, and `swap16` equivalents so the header compatibility surface is complete. |
| data structs | `crates/zelda3/src/types.rs` | verified | `Point16U`, `PointU8`, `Pair16U`, `PairU8`, `ProjectSpeedRet`, and `OamEnt` field order and widths match C. Rust-only helper structs such as `SpriteHitBox` and `AncillaRadialProjection` model local C scratch-return groupings and do not replace `types.h` declarations. |
| `MemBlk` representation | `crates/zelda3/src/types.rs`, `crates/zelda3/src/util.rs` | verified | C stores `{ const uint8 *ptr, size_t size }`. Rust stores `ptr: &'a [u8]`, so the pointer and size travel together as a slice; `FindIndexInMemblk` uses `bytes.len()` where C uses `data.size` and returns empty slices for C `{0, 0}` cases. |
| verification | types header slice | verified | `cargo fmt -p zelda3`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 types` pass. |

## 2026-05-31 Utility Full-File Pass

Scope: manually compare `crates/zelda3/src/util.rs` against
`../zelda3/src/util.c` and `util.h`, without using the
progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| text/token helpers | `crates/zelda3/src/util.rs` | verified | `NextDelim`, `StringEqualsNoCase`, `StringStartsWithNoCase`, `NextLineStripComments`, `NextPossiblyQuotedString`, `ReplaceFilenameWithNewPath`, `SplitKeyValue`, `SkipPrefix`, `StrSet`, and `StrFmt` preserve the C trimming, delimiter, case-folding, include-path, and formatting behavior using safe Rust string/slice ownership. |
| file loading | `crates/zelda3/src/util.rs`, `crates/zelda3/src/main.rs` | verified | `ReadWholeFile` returns file bytes plus a trailing zero byte and reports the original length, matching C's malloc-plus-NUL convention. Hard-failure behavior for required assets is handled at the call sites audited in the existing `LoadAssets` pass. |
| byte array helpers | `crates/zelda3/src/util.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | Rust `ByteArray` uses `Vec<u8>` while preserving C-visible size/capacity behavior, the `capacity + capacity/2 + 8` growth rule, resize, append-data, append-byte, and destroy semantics used by state recording. |
| memblock indexing | `crates/zelda3/src/util.rs` | verified | `FindIndexInMemblk` matches C's short-data return, 16-bit table mode for `mx < 8192`, 32-bit table mode for `mx >= 8192`, `i > mx` and table-size guards, left/right offset calculations, invalid-range return, and successful subslice return. |
| BPS patching and CRC | `crates/zelda3/src/util.rs` | verified | `BpsDecodeInt`, CRC32 polynomial/initial/final xor, `BPS1` header check, source/patch/output CRC checks, source/destination/meta size decoding, four command modes, relative offset updates, final output-size check, and failure return shape match C. Rust adds bounds checks so malformed patches return `None` instead of indexing past slices. |
| stale C-signature wrappers | `crates/zelda3/src/util.rs` | fixed | Removed unused lower-case pointer-shaped no-op wrappers for tokenizers, file/path helpers, and BPS/byte-array/string helpers. The real ports above are the typed Rust functions used by config, asset loading, and state recording; keeping no-op shims made the placeholder scan look less complete than the runtime actually is. |
| header split | `crates/zelda3/src/config.rs` | verified | `util.h` declares `ParseBool`, but C implements it in `config.c`; Rust likewise keeps boolean parsing with config parsing rather than in `util.rs`. |
| verification | utility slice | verified | `cargo fmt -p zelda3`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 util` pass. |

## 2026-05-31 SNES Shared Constants Pass

Scope: manually compare `crates/snes/src/consts.rs` against the cross-crate C
constants in `../zelda3/src/types.h` and
`../zelda3/snes/ppu.h`, without using the progress
script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| large-screen build flag | `crates/snes/src/consts.rs` | verified | C `kEnableLargeScreen = 1`; Rust `ENABLE_LARGE_SCREEN = true`, matching the enabled build-time configuration shared with `crates/zelda3/src/types.rs`. |
| horizontal overscan constant | `crates/snes/src/consts.rs` | verified | C `kPpuExtraLeftRight = kEnableLargeScreen ? 96 : 0`; Rust computes `PPU_EXTRA_LEFT_RIGHT` from the same boolean and yields 96 in this build. |
| PPU buffer width | `crates/snes/src/consts.rs` | verified | C `kPpuXPixels = 256 + kPpuExtraLeftRight * 2`; Rust `PPU_X_PIXELS = 256 + PPU_EXTRA_LEFT_RIGHT * 2`, so PPU priority buffers and mosaic arrays keep the same 448-pixel width. |
| verification | SNES shared constants slice | verified | `cargo check -q -p snes -p zelda3 -p zelda3-bin` passes after this source-comparison pass. |

## 2026-05-31 Rust Crate Boundary Classification Pass

Scope: manually inspect `crates/snes/src/lib.rs` and `crates/zelda3/src/lib.rs`
against the C source layout, without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| SNES crate module/export boundary | `crates/snes/src/lib.rs` | verified-runtime-neutral | There is no C `lib` counterpart; this file only declares Rust modules for the C `snes/` ports and re-exports the public emulator/oracle types (`Snes`, CPU/PPU/DMA/input/cart state, loader, and `cpu_run_opcode`). The behavioral surfaces remain in the audited module files. |
| Zelda crate module/export boundary | `crates/zelda3/src/lib.rs` | verified-runtime-neutral | There is no C `lib` counterpart; this file exposes config/oracle/SPC/types/util/CPU infra, maps C `src/main.c` to `zelda_main`, and exposes `ZeldaState` plus oracle report types. The C gameplay module files are included privately through `zelda_rtl.rs`, matching Rust ownership rather than changing runtime behavior. |
| lockstep oracle compatibility boundary | `crates/zelda3/src/oracle.rs` | verified-runtime-neutral | There is no C `oracle.c` counterpart; this file only re-exports the Rust lockstep/oracle API from `zelda_cpu_infra` so callers have a stable module path. The actual C-aligned lockstep behavior remains in the audited `zelda_cpu_infra`/`zelda_rtl` surfaces. |
| verification | crate boundary slice | verified | `cargo check -q -p snes -p zelda3 -p zelda3-bin` passes after this classification pass. |

## 2026-05-31 Attract Full-File Pass

Scope: manually compare `crates/zelda3/src/attract.rs` against
`../zelda3/src/attract.c` and `attract.h`, without
using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| module dispatcher and scene setup | `crates/zelda3/src/attract.rs` | verified | `Module14_Attract`, fade/fade-in/fade-out sequencing, graphics init, scene loader, polka-dot/world-map/throne/prison/maiden/end setup, skip-to-file-select, and conclusion HDMA keep the same state tests, byte/word writes, palette calls, PPU register writes, message indexes, music/SFX writes, and scene/state increments as C. |
| dungeon/palette helper split | `crates/zelda3/src/attract.rs`, `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/load_gfx.rs` | verified | C's `Dungeon_LoadAndDrawEntranceRoom` and `Dungeon_SaveAndLoadLoadAllPalettes` are preserved as Rust wrappers over the existing dungeon/palette methods. The reviewed call sites save and restore `attract_var12` and `WORD(attract_state)` exactly like C before applying room-specific palette/message/OAM setup. |
| story dramatization | `crates/zelda3/src/attract.rs` | verified | `AttractDramatize_PolkaDots`, world-map zoom, throne-room scroll/OAM, prison dramatization, Agahnim altar, maiden warp cases 0-4, timed text, and fade-in step preserve the C timers, branch thresholds, `INIDISP` changes, OAM priority countdown, sound effects, `attract_var*` updates, and sequence/state transitions. |
| OAM helpers and soldier simulation | `crates/zelda3/src/attract.rs` | verified | `Attract_DrawSpriteSet2`, `Attract_DrawPreloadedSprite`, `Attract_DrawZelda`, `Attract_ZeldaPrison_DrawA`, prison case OAM, maiden warp OAM, and `Sprite_SimulateSoldier` match the C reverse-order OAM emission, base-coordinate wrapping, extended OAM size byte, sprite coordinate setup, graphics index calculation, flags/type selection, OAM pointer setup, and guard animation call. |
| legend graphics and background DMA | `crates/zelda3/src/attract.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | The four `kAttract_Legendgraphics_*` tables are present with the same byte counts (`158/238/200/266`), `Attract_BuildNextImageTileMap` copies them to `$1002` and sets `nmi_load_bg_from_vram = 1`, `Attract_ControlMapZoom` writes 240 scaled `kMapMode_Zooms1` entries, and `Attract_BuildBackgrounds`/`Attract_TriggerBGDMA` reproduce both C tile fill loops and eight 0x100-byte VRAM row copies. |
| header surface | `crates/zelda3/src/attract.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | Every function declared in `attract.h` has a Rust counterpart under `ZeldaState` or an existing helper module. `kMapMode_Zooms2` is not consumed by `attract.rs` itself; Rust keeps it in `zelda_rtl.rs` for the same `zelda_rtl.c` address-table and Mode 7 perspective uses. |
| verification | attract slice | verified-open | `cargo check -q -p zelda3 -p zelda3-bin` passes. `cargo test -q -p zelda3 attract` reports no matching tests, so this full-file pass is backed by direct source comparison plus compile only. |

## 2026-05-31 Audio Full-File Pass

Scope: manually compare `crates/zelda3/src/audio.rs` against
`../zelda3/src/audio.c` and `audio.h`, without using
the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| track queries and MSU remapping | `crates/zelda3/src/audio.rs` | verified | `RemapMsuDeluxeTrack`, `ZeldaIsPlayingMusicTrack`, `ZeldaIsPlayingMusicTrackWithBug`, and `ZeldaGetEntranceMusicTrack` preserve the C MSU-deluxe enable gate, OW-area low-byte lookup, entrance-song `242` fallback, and `music_unk1` vs `last_music_control` bug-fix branch. |
| APUI write queue and render order | `crates/zelda3/src/audio.rs` | verified | `zelda_apu_write`, `ZeldaPushApuState`, `ZeldaPopApuState`, `ZeldaDiscardUnusedAudioFrames`, `ZeldaResetApuQueue`, `zelda_read_apui00`, `zelda_apu_read`, and `ZeldaRenderAudio` preserve C's 16-entry ring, pop-before-generate order, stale-frame discard rule, APUI00 RAM mirror, `port_to_snes` reads, and SPC-generate-before-sample flow. |
| save/load music state | `crates/zelda3/src/audio.rs` | verified | `ZeldaRestoreMusicAfterLoad_Locked` and `ZeldaSaveMusicStateToRam_Locked` preserve the C variable-copy calls, timer reset, input-port restore/save through SPC RAM `$410`, MSU volume/resume-info storage, reset initialization, transition-volume restoration, and APU queue reset. |
| audio config propagation | `crates/zelda3/src/audio.rs`, `crates/zelda3/src/main.rs` | fixed | C `configure_runtime_from_config`/`ZeldaEnableMsu` reads `g_config.audio_freq`, `g_config.msuvolume`, `g_config.resume_msu`, and `g_config.msu_path`. Rust now copies those parsed config values into `AudioState` before enabling MSU, so volume-step/target scaling, resume gating, and MSU filename construction use the same config inputs. |
| MSU resume bookkeeping | `crates/zelda3/src/audio.rs` | fixed-open | Rust now preserves the observable C `MsuPlayer_Open` pre-open resume side effect: an already-resuming/playing MSU state writes the current `resume_info` back to `msu_resume_info_alt` before closing the current file/state. The C branch that selects `msu_resume_info_alt` for module-9 resume is mirrored. PCM resume offset/sample bookkeeping is now wired; OPUZ resume remains open with the missing OPUZ decoder. |
| MSU PCM file open and streaming | `crates/zelda3/src/audio.rs` | fixed | Ported C's `MsuPlayer_Open` PCM branch: config-path filename construction, MSU1 header/read-error fallback, repeat-position parse, playing-vs-resuming state choice, resume-info initialization, total-sample calculation, current offset, and 960-frame buffer fills. `MsuPlayer_Mix` now advances the destination audio pointer per consumed block and handles non-repeating track finish/read-error fallback like C. |
| MSU no-file fallback | `crates/zelda3/src/audio.rs` | fixed | C `MsuPlayer_Open` only leaves state non-idle after opening and validating a file; on read/open error it closes the MSU player and `ZeldaPlayMsuAudioTrack` sends the original music command to SPC. Rust now has the same failed-open fallback while allowing valid PCM files to take over playback. |
| MSU enable and mixing helpers | `crates/zelda3/src/audio.rs` | fixed-open | The volume transition constants, step/target scaling, mix-with-volume, mix-ramp, and mix dispatch match C's math for already-buffered samples. This pass also removed Rust's mutation of `config_audio_freq` on OPUS enable; C only prints a warning and does not change config. PCM file IO is now implemented; OPUZ file IO/Opus decoding remains open. |
| song-bank upload | `crates/zelda3/src/audio.rs` | verified | `LoadSongBank`/`load_song_bank` preserves C's `SpcPlayer_Upload` behavior for the high-level audio player, with Rust additionally mirroring the uploaded bytes into `audio.spc_ram` for save/debug helpers. |
| verification | audio slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 audio` pass after the MSU config/fallback/resume-bookkeeping and PCM open/streaming fixes. Runtime-perfect MSU playback remains open because the C OPUZ decoder path is still absent and broader runtime/oracle coverage has not been re-run for MSU packs. |

## 2026-05-31 Overlord Full-File Pass

Scope: manually compare `crates/zelda3/src/overlord.rs` against
`../zelda3/src/overlord.c` and `overlord.h`, without
using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| dispatcher and active checks | `crates/zelda3/src/overlord.rs` | fixed | `Overlord_Main`, `Overlord_ExecuteAll`, and the 26-entry overlord dispatch table match C. This pass changed invalid nonzero overlord types from a silent no-op to a panic, matching C's direct `kOverlordFuncs[j - 1]` failure behavior instead of hiding corrupt state. |
| coordinate/math helpers | `crates/zelda3/src/overlord.rs` | verified | `Overlord_GetX/Y`, `Overlord_SetX/Y`, `ArmosMult`, and `ArmosSin` preserve the C byte layout, rounding rule, sinus sign handling, and assert-only `Overlord_StalfosFactory` slot. |
| spawn helpers and side effects | `crates/zelda3/src/overlord.rs`, `crates/zelda3/src/sprite.rs` | fixed | Overlord sprite spawns now delegate to the canonical `Sprite_SpawnDynamically(Ex)` port, preserving C's inclusive `j` search bound, `SpritePrep_LoadProperties`, `sprite_N`, floor/D/die-action/subtype initialization, and full `SpriteSpawnInfo` population. The previous local helper skipped those side effects and treated `j=12` as slots `11..0` instead of `12..0`. |
| trap/spawner bodies | `crates/zelda3/src/overlord.rs` | fixed | Boulder, invisible stalfos, pot trap, Zoro, Wizzrobe, tile room, Pirogusu, falling square, wallmaster, blob, moving floor, falling stalfos, bad switch snake/bomb trap, cannon factories, cannon balls, and position target now match the C control flow, timers, random masks, slot loops, sprite fields, sound effects, and temporary variables. This pass fixed Zoro spawns to write `sprite_ignore_projectile[j] = 1` instead of `sprite_subtype[j] = 1`. |
| tile/garnish helpers | `crates/zelda3/src/overlord.rs`, `crates/zelda3/src/sprite.rs`, `crates/zelda3/src/sprite_main_prep.rs` | verified | The overlord tile helper matches C `GetTileAttribute` for the single overlord call site, including indoor BG2 table indexing, outdoor `x >>= 3` semantics where the mutated X is unused, and `sprite_tiletype` storage. Falling-tile garnish allocation follows C `GarnishAlloc`'s high-to-low search and writes the same garnish fields/countdown/active byte. |
| Armos coordinator | `crates/zelda3/src/overlord.rs` | verified | Bounce state machine, back-wall assignments, coercion enable/disable, radial contraction/dilation, rotation angle update via `WORD(overlord_x_lo[0])`, projected X/Y writes, knight readiness check, and `tmp_counter = 6` match C. |
| header surface | `crates/zelda3/src/overlord.rs` | verified | Every function declared in `overlord.h` has a Rust counterpart under `ZeldaState`; C-static helpers remain private Rust helpers. |
| verification | overlord slice | verified-open | `cargo fmt -p zelda3`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 overlord` pass after the spawn-bound/property and Zoro field fixes. There are no focused overlord runtime tests yet, so gameplay coverage still depends on later oracle routes that exercise these overlord states. |

## 2026-05-31 Config Full-File Pass

Scope: manually compare `crates/zelda3/src/config.rs` against
`../zelda3/src/config.c`, `config.h`, and
`features.h`, without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| public constants and config layout | `crates/zelda3/src/config.rs` | fixed | Rust key command IDs, output-method IDs, MSU flags, gamepad button IDs, and `features0` masks match `config.h`/`features.h`. This pass changed `Config::default().msuvolume` back to C's zero-initialized global state; `parse_config_file` still sets the C runtime default `msuvolume = 100` before loading files. |
| default key/gamepad maps | `crates/zelda3/src/config.rs` | verified | `kDefaultKbdControls`, `kKeyNameId`, and `kDefaultGamepadCmds` match C's command ranges, default controls, intentionally empty load/save/ref slots, volume entries with no default keys, and default gamepad Controls mapping. |
| keyboard map hash | `crates/zelda3/src/config.rs` | verified | `KeyMapHash_Add`, lookup, modifier encoding, modified key lookup, duplicate behavior, and 256-entry allocation-boundary limit checks preserve C behavior, including incrementing the backing array before duplicate detection. |
| gamepad map hash | `crates/zelda3/src/config.rs` | fixed | `ParseGamepadButtonName`, modifier parsing, default registration, modifier-count insertion priority, and lookup match C. This pass removed Rust's invalid-button early return from `find_cmd_for_gamepad_button`, so corrupt callers fail through direct array indexing like C's `joymap_first[button]`. |
| ini section dispatch and value parsing | `crates/zelda3/src/config.rs` | verified | KeyMap, GamepadMap, Graphics, Sound, General, and Features dispatch match C's section IDs and key names. Window size, output method, booleans, MSU modes, extended aspect ratio flags, language/shader/path strings, and feature bits preserve the C parsing and assignment behavior. |
| file parser/include flow | `crates/zelda3/src/config.rs`, `crates/zelda3/src/util.rs` | verified | `parse_one_config_file` uses the same line/comment/key-value/include helpers as the C parser, keeps the same user-config then fallback-config order, and applies default keys after config loading. Rust owns strings instead of leaking `g_config.memory_buffer`, but the resulting config values and key maps are equivalent. |
| stale static wrappers | `crates/zelda3/src/config.rs` | fixed | Removed unused pointer-shaped private wrappers for `ParseGamepadButtonName`, `GetIniSection`, and `ParseBoolBit`; the typed Rust helpers above are the actual ports. Keeping the stale wrappers made the file look like it had alternate C surfaces, and one could not update the caller pointer like the C static helper. |
| SDL key-name resolver | `crates/zelda3/src/config.rs` | fixed | C delegates arbitrary key names to SDL's `SDL_GetKeyFromName`. Rust now uses a static SDL2 key-name table for the non-printable/scancode-backed names accepted by SDL, keeps SDL's one-character lowercase behavior, supports `F1..F24`, keypad, modifier, media, AC, and system keys, and removes the previous non-C `Enter` alias (`SDL_GetKeyFromName("Enter") == SDLK_UNKNOWN`; C accepts `Return` or `Keypad Enter`). |
| verification | config slice | verified | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 config` pass after the default-value/direct-index cleanup and SDL key-name table port. A one-off ctypes comparison against the installed SDL2 2.32.10 library also confirms all 178 Rust table entries match `SDL_GetKeyFromName`, including alias checks for `Enter`, `Return`, `F24`, `Keypad Enter`, `Print Screen`, and `PrintScreen`. |

## 2026-05-31 Player OAM LinkOam Control-Flow Pass

Scope: manually compare the pose-selection control flow in
`crates/zelda3/src/player_oam.rs` against
`../zelda3/src/player_oam.c` `LinkOam_Main`, without
using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| auxiliary-state fallthrough | `crates/zelda3/src/player_oam.rs` | fixed | C's `link_auxiliary_state == 1 && kPlayerState_TurtleRock` path sets `oam_priority_value = 0x3000` when `byte_7E034E == 0`, then jumps to `link_state_is_empty`, skipping pit and `link_state_bits` pose handling while still allowing master-sword, held-item, position-mode, medallion/spin, and sword-button pose selection. Rust previously swallowed the entire auxiliary-state branch and kept the earlier base pose. The Rust control flow now preserves C's `goto link_state_is_empty` behavior. |
| auxiliary-state non-handled cases | `crates/zelda3/src/player_oam.rs` | fixed | C does not make the `link_auxiliary_state != 0` block mutually exclusive with later pit/link-state checks unless it explicitly `goto continue_after_set`. Rust now lets unhandled auxiliary states fall through to the same later checks instead of terminating the selection chain early. |
| shield OAM slot advance | `crates/zelda3/src/player_oam.rs` | fixed | C's shield loop advances `j` and adjusts `oam_x/oam_y`, but does not increment `oam_pos` inside the three-entry `kShieldStuff_OamData` loop. Rust was advancing `oam_pos` each iteration; it now preserves C's same-slot write behavior. |
| DMA helper byte widths | `crates/zelda3/src/player_oam.rs` | fixed | C declares `link_dma_var3`, `link_dma_var4`, and `link_dma_var5` as adjacent `uint8` fields. Rust was using `write_le_u16` for these helper writes, clobbering the following byte; it now writes only the byte field C writes. |
| X-offset bit word write | `crates/zelda3/src/player_oam.rs` | fixed | C declares `bit9_of_xcoord` as `uint16` and assigns `0` or `1`, clearing the high byte. Rust only wrote the low byte; it now writes the full word for RAM parity. |
| dungeon fall shadow word reads | `crates/zelda3/src/player_oam.rs` | fixed | C's `LinkOam_DrawDungeonFallShadow` reads `tiledetect_which_y_pos[0]`, `link_y_coord`, and `BG2VOFS_copy2` as words, then truncates the resulting `uint8` temporaries. Rust was reading only the low byte of each source word; it now performs the C word math before truncation. |
| X-offset integer promotion | `crates/zelda3/src/player_oam.rs` | fixed | C computes `link_x_coord + (int8)x - BG2HOFS_copy2` with promoted integer operands before shifting. Rust was casting both 16-bit coordinates to `i16`; it now uses widened signed math before storing the word result. |
| foot-object animation byte truncation | `crates/zelda3/src/player_oam.rs` | fixed | C stores the grass/water foot-object `8 + yv` value back into a `uint8`. Rust now uses byte wrapping for that addition instead of widening it to `usize` before the OOB/chardata branch. |
| sparkle/thrown/shadow helper tables | `crates/zelda3/src/player_oam.rs` | verified | `kSwordStuff_oam_index_ptrs_*`, throwing-state/X/Y tables, shield shadow offsets, shadow OAM index tables, shadow char data, and `kPlayerOam_DrawOam_2X` match the C table values in the helper cluster. |
| body/sword/shield OAM tables | `crates/zelda3/src/player_oam.rs` | verified | `kPlayerOam_Prio`, `kDrawSword_y`, `kDrawSword_x`, `kSwordTiledata`, `kShieldStuff_x`, `kShieldStuff_y`, `kShieldStuff_oam_index_ptrs_*`, `kShieldStuff_OamData`, `kLinkBody_oam_index_*`, `kLinkDmaGraphicsIndices`, and all 303 `kLinkSpriteBodys` entries match C table values exactly. |
| front-half pose/animation tables | `crates/zelda3/src/player_oam.rs` | verified | `kPlayerOam_StairsOffsY`, floor priority/sort/tab tables, sword-tip and sword OAM offset tables, `kPlayerOamOtherOffs`, main sword/shield selector tables, sprite-location tables, sprite bank/X/Y tables, and the tab19/tab20 helper tables match C table values exactly. |
| public header surface | `crates/zelda3/src/player_oam.rs` | verified | Every function declared in `player_oam.h` has a Rust `ZeldaState` counterpart or exported helper: `PlayerOam_WantInvokeSword`, `CalculateSwordHitBox`, `LinkOam_Main`, `FindMostSignificantBit`, weapon/equipment VRAM offset setters, sword sparkle positioning, unused weapon settings, dungeon fall shadow, foot object, and X-offset relative-to-Link. |
| body/sword/shield OAM writes | `crates/zelda3/src/player_oam.rs` | verified-open | The reviewed sword, shield, shadow, and body OAM write paths now match C's table selection, z-coordinate helper use, priority/palette masking, OAM char/XY writes, extended OAM writes, body DMA index update, doorway/blink/cape hide behavior, and staircase Y restore. This remains runtime-open until an oracle route exercises enough Link OAM pose combinations. |
| remaining file scope | `crates/zelda3/src/player_oam.rs` | verified-open | Direct C/Rust source comparison now covers the header functions, `LinkOam_Main` control flow, static OAM tables, sword/shield/body/shadow/foot helper paths, and exact table values. Full runtime parity remains open until oracle routes exercise enough Link pose/OAM combinations. |
| verification | player OAM slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3 -p zelda3-bin`, and `cargo test -q -p zelda3 player_oam` pass after the player-OAM fixes. The focused test command currently has no matching tests (`324 filtered out`), so focused runtime/oracle coverage for Turtle Rock auxiliary-state OAM remains open. |

## 2026-05-30 Select File Defensive Index Parity Pass

Scope: continue the manual `select_file.c` audit by comparing state-table
lookups in `FileSelect_Main`, confirmation menus, and name-entry cursor OAM.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| file-select fairy Y lookup | `crates/zelda3/src/select_file.rs` | fixed | C indexes `kSelectFile_Faerie_Y[selectfile_R16]` directly. Rust no longer wraps `R16` modulo five, so corrupt select-file state fails instead of being coerced to a valid menu row. |
| copy/delete confirmation fairy Y lookups | `crates/zelda3/src/select_file.rs` | fixed | C `SelectFile_Func16` and `CopyFile_HandleConfirmation` index their two-entry confirmation Y tables directly. Rust no longer masks `R16 & 1` before indexing those tables. |
| name-entry cursor X lookup | `crates/zelda3/src/select_file.rs` | fixed | C indexes `kNamePlayer_X[selectfile_var4]` directly for the active name-character cursor. Rust no longer wraps `selectfile_var4` by the table length. |
| name-entry ROM table fallbacks | `crates/zelda3/src/select_file.rs` | verified-limited | The remaining modulo fallbacks only apply if the original ROM table reads are unavailable. With the real ROM loaded, Rust reads the same ROM bytes C reaches when hacked/opposite-direction inputs drive past the short source-level arrays. |

## 2026-05-30 Name Entry BG3 Scroll Renderer Pass

Scope: debug the register-your-name screen after scripted name entry and
left/right movement, using direct frame dumps plus the snes9x visual oracle rather
than the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| snes9x frame dump helper | `zelda3-bin/src/main.rs` | added | Added `--dump-snes9x-frame <core> <rom> <frames> <out.png> [--input-script <path>] [--skip-snes9x-frames <n>]` so the hardware-oracle frame can be inspected at the same scripted menu point as Rust dumps. |
| scanline-128 IRQ scroll split | `crates/zelda3/src/zelda_rtl.rs` | fixed | C `ZeldaDrawPpuFrame` writes `selectfile_var8` to `BG3HOFS` and zeroes `BG3VOFS` at scanline 128 when `irq_flag` is set. Rust was writing those split-scroll values to BG1 (`$210d/$210e`); it now writes BG3 (`$2111/$2112`) like C, which keeps the lower name-entry alphabet scroll on the intended layer. |
| Mode 1 BG3 priority workaround | `crates/snes/src/ppu.rs` | removed | Dropped the temporary name-entry-specific BG3 priority gate and restored the C renderer constants (`0xf200` / `0x1200`) for Mode 1 BG3. The visual issue was the wrong IRQ scroll target, not a PPU priority rule. |
| verification | name-entry route | verified | `cargo check -q -p snes -p zelda3 -p zelda3-bin`, `cargo test -q -p snes ppu`, and `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2620 --input-script <name-right-script>` pass. Clean-SRAM Rust dumps for right/left movement were visually compared with snes9x dumps at `/tmp/name-rust-right-fixed.png`, `/tmp/name-rust-left-fixed.png`, `/tmp/name-snes9x-right-fixed.png`, and `/tmp/name-snes9x-left-fixed.png`. |

## 2026-05-31 Lockstep Audio Sample Oracle Harness Pass

Scope: debug the reported frame-2 `--play-lockstep` audio sample divergence
without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| C lockstep audio boundary | `zelda3-bin/src/main.rs`, `crates/zelda3/src/zelda_cpu_infra.rs` | fixed | The frame-2 failure had identical Rust/oracle traces and matching APUI command ports, but `oracle.snes.apu.dsp` produced silence. Direct source inspection shows `LockstepOracle::run_frame_with_compare` compares game RAM/SRAM/PPU state and APUI ports after running the original SNES CPU path; it does not boot and advance a sample-producing SPC/DSP reference. The sample comparator was therefore comparing Rust high-level audio against an uninitialized/silent harness buffer, not against C audio. |
| lockstep comparator behavior | `zelda3-bin/src/main.rs` | fixed | Removed sample-level and APUI command-port comparison from C lockstep modes. Lockstep still exits on state and render divergence. Exact audio command/sample comparison remains in the snes9x oracle path, which has a real sample-producing reference. |
| failure artifacts | `zelda3-bin/src/main.rs` | fixed | Lockstep artifact reports now note that `oracle_audio.wav` is not a sample oracle because the lockstep C oracle does not boot/advance a full SPC/DSP audio program. |
| verification | lockstep harness | verified | `cargo fmt -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- /path/to/zelda3.sfc --play-lockstep 5`, and `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 5` pass. The prior frame-2 sample false-positive is gone; broader exact audio parity remains assigned to the snes9x oracle path. |

## 2026-05-31 Playable snes9x Visual Oracle Pass

Scope: make the user-visible name-entry/select-file rendering issue fail against
a real hardware-class pixel reference instead of the C-lockstep render mirror.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| C-lockstep render blind spot | `zelda3-bin/src/main.rs` | verified-open | `--compare-lockstep-render` renders both Rust and C-oracle states through the Rust PPU renderer. That is still useful for state-to-render regressions, but it cannot catch a PPU renderer bug that affects both frames the same way. The name-entry route reproduced this: C-lockstep render reported zero mismatched pixels while the screen was visibly wrong. |
| playable snes9x oracle | `zelda3-bin/src/main.rs` | added | ROM-first `--play-lockstep` now accepts `--compare-snes9x-oracle <core>`, optional `--input-script`, `--ignore-video`, `--ignore-audio`, and `--compare-from-frame <n>`. It keeps C lockstep for logic/RAM parity, runs snes9x in parallel from the same input word, compares Rust pixels/audio to the snes9x callback output, and exits with the existing `target/parity-failures/<timestamp>/` artifact bundle on first enabled diff. |
| scripted compare window | `zelda3-bin/src/main.rs` | added | `--compare-snes9x-oracle` also accepts `--compare-from-frame <n>`, so known earlier startup/timing mismatches can be skipped when targeting a later visual issue such as name-entry scroll or select-file indentation. |
| name-entry visual failure artifact | `target/parity-failures/1780232353-22906/` | captured | Scripted playable lockstep with snes9x video comparison now fails at frame 1890. `rust_frame.png` shows the lower name-entry alphabet/page shifted relative to the snes9x reference, while `snes9x_frame.png` shows the expected page. The trace records `select=(r16=0 grid_col=2 name_col=1 row=3 scroll=$0010 y=179 ...)`, giving a concrete state anchor for the next logic/renderer fix. |
| verification | snes9x/playable harness | verified-open | `cargo fmt -p zelda3-bin`, `cargo check -q -p zelda3-bin`, `--compare-lockstep-render ... 1900 --input-script target/parity-failures/1780231020-16427/input.txt` still reports zero pixels, and `cargo run -q -p zelda3-bin -- /path/to/zelda3.sfc --play-lockstep 1900 --input-script target/parity-failures/1780231020-16427/input.txt --compare-snes9x-oracle external/snes9x-libretro/local/snes9x_libretro.dylib --ignore-audio --compare-from-frame 1890` exits with the snes9x artifact above. |

## 2026-05-31 Copy Player Stripe Upload Offset Pass

Scope: fix the select-file/copy-player visual corruption found while driving the
saved-name route through C/Rust lockstep, without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| copy-player prompt stripe copy | `crates/zelda3/src/select_file.rs` | fixed | C's `memcpy(vram_upload_data + 26, ...)` uses a `uint16 *`, so the destination is 52 bytes after `vram_upload_data`. Rust was using byte offset 26 for both `kCopyFile_SelectionAndBlinker_Tab1` and `kCopyFile_TargetSelectionAndBlink_Tab2`, corrupting the stripe buffer and producing shifted/garbled copy-player rendering. Both copies now use byte offset 52. |
| post-name file-select scroll pre-clear | `zelda3-bin/src/main.rs` | reverted | Returning from name entry leaves `irq_flag = 0xff`, but C does not clear it before drawing. C `ZeldaDrawPpuFrame` applies the scanline-128 BG3 split once and clears bit-7 IRQ state inside the draw loop. Rust no longer pre-clears that state in the host draw wrapper, so the post-save file-select frame follows the C draw order. |
| verification | select-file copy route | verified | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- /path/to/zelda3.sfc --play-lockstep 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt` pass. Before the fix, the same route exited at frame 2722 with WRAM/VRAM stripe-buffer divergence and artifacts in `target/parity-failures/1780233549-38678/`. |

## 2026-05-31 Select File Copy/Kill Persistence Pass

Scope: manually compare the copy-file and kill-file state machines in
`crates/zelda3/src/select_file.rs` against
`../zelda3/src/select_file.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| immediate SRAM persistence | `crates/zelda3/src/select_file.rs` | fixed | C calls `ZeldaWriteSram()` immediately after deleting a file in `SelectFile_Func16`, confirming a copy in `CopyFile_HandleConfirmation`, and finalizing a new name in `NameFile_DoTheNaming`. Rust now calls `zelda_write_sram()` at the same three commit points instead of only relying on the host exit write. |
| copy-file state machine | `crates/zelda3/src/select_file.rs` | verified | `Module02_CopyFile`, `Module_CopyFile_2`, selection/target/confirmation dispatch, header stripe deletion, source/target slot navigation, name/heart stripe writes, `selectfile_R20/R18` slot storage, prompt stripe overlays, fairy positions, SFX writes, return-to-file-select paths, and NMI upload flags match C. |
| kill-file state machine | `crates/zelda3/src/select_file.rs` | verified | `Module03_KILLFile`, `KILLFile_SetUp`, selection/confirmation dispatch, generated 253-byte target stripe, confirmation stripe, `SelectFile_Func17` name redraws, navigation wrap, `subsubmodule_index` target handoff, delete confirmation through `SelectFile_Func16`, and NMI upload flags match C. |
| verification | select-file copy/kill slice | verified | `cargo fmt -p zelda3 --check` and `cargo check -q -p zelda3-bin` pass after restoring the three immediate SRAM writes. |

## 2026-05-31 Select File Loader/Saved-Slot Display Pass

Scope: manually compare the file-select loader, SRAM validation, and saved-slot
display helpers in `crates/zelda3/src/select_file.rs` against
`../zelda3/src/select_file.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| file-select loader and SRAM validation | `crates/zelda3/src/select_file.rs` | verified | `LoadFileSelectGraphics`, `Intro_CheckCksum`, `Intro_ValidateSram`, `Module01_FileSelect`, `Module_SelectFile_0`, `FileSelect_ReInitSaveFlagsAndEraseTriforce`, and `FileSelect_EraseTriforce` match C's graphics asset IDs, VRAM destinations, font transfer, SRAM primary/backup checksum repair, sprite RAM clear, BG3 scroll clear, palette/tile/theme setup, force blank, CGRAM update flag, submodule increments, and NMI disable writes. |
| shared background stripe builders | `crates/zelda3/src/select_file.rs` | verified | `SelectFile_Func1`, `Module_EraseFile_1`, and `FileSelect_TriggerStripesAndAdvance` match C's `vram_upload_data` base, initial `$0010/$ff07` packet header, 1024-word alternating background fill, 224-byte frame packet, 18 appended vertical stripe triplets starting at `$1103`, byte terminator, `selectfile_R16 = selectfile_var2`, submodule increments, and `nmi_load_bg_from_vram` writes. |
| saved-slot OAM and stripe draw helpers | `crates/zelda3/src/select_file.rs` | verified | `FileSelect_DrawFairy`, `SelectFile_Func5_DrawOams`, `SelectFile_Func6_DrawOams2`, and `SelectFile_Func17` match C's per-slot Y/OAM-index tables, Link DMA graphics index, sword/shield char tables and hide-on-empty behavior, body OAM entries, death-counter digit clamp/placement, name stripe offsets, and health-heart stripe layout for valid saved slots. |
| main file-select state | `crates/zelda3/src/select_file.rs` | verified | `FileSelect_Main` matches C's saved-slot detection, `selectfile_arr1` writes, draw helper call order, fairy Y table, NMI upload flag, directional wrap between three slots/copy/delete rows, new-name handoff, saved-game load path, copy/delete handoff, SFX writes, and `selectfile_var2/R16/R17` side effects. |
| verification | select-file loader/display slice | verified | `cargo fmt -p zelda3 --check` and `cargo check -q -p zelda3-bin` pass after this source comparison. |

## 2026-05-31 Select File Name-Entry Completion Pass

Scope: manually compare the remaining name-entry setup, checksum, cursor-scroll,
selected-character draw, and finalize helpers in
`crates/zelda3/src/select_file.rs` against
`../zelda3/src/select_file.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| name-entry setup stripes | `crates/zelda3/src/select_file.rs` | verified | `FileSelect_TriggerNameStripesAndAdvance`, `Module04_NameFile`, `NameFile_EraseSave`, `Module_NamePlayer_1`, and `Module_NamePlayer_2` match C's 253-byte name stripe packet, `INIDISP_copy`, `nmi_disable_core_updates`, `nmi_load_bg_from_vram`, submodule dispatch/increments, IRQ flag, cursor/scroll variables, SRAM slot erase, blank-name initialization, and `attract_legend_ctr` setup. |
| checksum and selected-character stripes | `crates/zelda3/src/select_file.rs` | verified | `Intro_FixCksum`, `Intro_FixCksumSlot`, and `NameFile_DrawSelectedCharacter` match C's 0x27f-word checksum loop, checksum word at slot word `$27f`, selected-name VRAM packet, swapped tile addresses, top/bottom character words, byte terminator, and NMI upload flag. |
| name-entry cursor and finalize state | `crates/zelda3/src/select_file.rs` | fixed | `NameFile_DoTheNaming`, `NameFile_CheckForScrollInputX`, and `NameFile_CheckForScrollInputY` match C's scroll animation counters, ROM-backed letter grid tables, direct cursor X table, wrap rules, row-boundary suppression, OAM cursor rows, back/forward/END tile behavior, SFX writes, SRAM initialization, checksum, immediate SRAM write, `ReturnToFileSelect`, and `irq_flag = 0xff`. This pass fixed the blank-name finalization scan to read through C's `attract_legend_ctr` base instead of recomputing the slot from `selectfile_R16`. |
| select-file source coverage | `crates/zelda3/src/select_file.rs` | source-covered/runtime-open | Every function in `../zelda3/src/select_file.c` now has a manual C/Rust comparison entry in this ledger. Runtime-open because broader snes9x visual route coverage should still exercise more copy/delete/name-entry edge cases and bad-state assertions. |
| verification | select-file name-entry completion slice | verified | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 2700 --input-script scripts/inputs/file-select-new-game.txt --load-sram target/debug-frames/empty-sram.dat` pass; the lockstep route completes with `WRAM fnv1a64 = 75b1acbfdecd328c`. |

## 2026-05-31 Tagalong Full Source Pass

Scope: manually compare `crates/zelda3/src/tagalong.rs` against
`../zelda3/src/tagalong.c` and the C helpers it calls,
without using the progress script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| follower dispatch and movement | `crates/zelda3/src/tagalong.rs` | verified | `Tagalong_IsFollowing`, `Follower_ValidateMessageFreedom`, `Follower_MoveTowardsLink`, `Follower_Initialize`, `Sprite_BecomeFollower`, `Follower_Main`, timed follower messages, Kiki/old-man dispatch, hookshot tail advance, dropped-follower reacquire, and trigger-message dispatch match C control flow and byte/word state updates. |
| follower drop/super-bomb helper | `crates/zelda3/src/tagalong.rs` | fixed | C `AncillaAdd_SuperBombExplosion` clears `ancilla_R`, `ancilla_step`, `ancilla_arr25`, `ancilla_L`, `ancilla_arr3`, and `ancilla_item_to_link`, then reads `WORD(tagalong_var2)` for the follower position. Rust now clears the missing `ancilla_R`/`ancilla_arr25` bytes and uses the same word-wide `tagalong_var2` index instead of only the low byte. |
| tagalong draw slot clamp | `crates/zelda3/src/tagalong.rs` | fixed | C `Tagalong_Draw` computes priority from the raw `tagalong_var2`, then clamps a negative signed `tagalong_var2` byte to slot 0 before reading follower X/Y and layerbits for animation. Rust now applies the same clamp for the draw slot. |
| draw tables and OAM/DMA | `crates/zelda3/src/tagalong.rs` | verified | `kTagalongFlags`, message info/room/offset tables, `kTagalongDraw_SprXY`, DMA/flag tables, palette/offset tables, first-sprite animation table, sprite OAM offset tables, and `Follower_AnimateMovement_preserved` match C table values, animation gating, priority/palette rules, bytewise extended OAM indexing, DMA byte writes, and scroll-relative OAM placement. |
| follower-to-sprite helpers | `crates/zelda3/src/tagalong.rs` | verified-open | `Blind_SpawnFromMaiden`, `Kiki_RevertToSprite`, `Kiki_SpawnHandlerMonke/A/B`, and `OldMan_RevertToSprite` match the C spawn/property writes on successful spawn. Runtime-open because the Rust helpers safely model failed dynamic spawn as `Option` in places where C would return `-1` and then index through that result; no focused route has exercised the failure case. |
| verification | tagalong slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo test -q -p zelda3 tagalong` pass. The focused test command currently runs one tagalong-related test; broader gameplay/oracle routes for Zelda/Kiki/old-man/maiden/super-bomb follower states remain open. |

## 2026-05-31 Oracle Wrapper Classification

Scope: classify `crates/zelda3/src/oracle.rs` for the manual parity ledger.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| oracle module wrapper | `crates/zelda3/src/oracle.rs` | classified | The file contains only `pub use crate::zelda_cpu_infra::*;`, so it is Rust compatibility/API surface, not a C game-logic implementation to compare against `../zelda3/src`. Parity work for the oracle itself belongs to `crates/zelda3/src/zelda_cpu_infra.rs` and `zelda3-bin/src/main.rs` harness entries. |

## 2026-05-31 Ending Credits Tail Side-Effect Pass

Scope: manually compare the credits tail around `EndSequence_32`,
`Credits_FadeColorAndBeginAnimating`, `Credits_StopCreditsScroll`,
`Credits_FadeAndDisperseTriangles`, and
`CrystalCutscene_InitializePolyhedral` against
`../zelda3/src/ending.c`, without using the progress
script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| end-sequence credits setup | `crates/zelda3/src/ending.rs` | fixed | C writes `BG2SC` `$2108 = 0x13` immediately after `CGADSUB_copy = 162`. Rust now performs the same PPU register write before initializing fixed colors and HDMA. |
| credits scroll/fade side effects | `crates/zelda3/src/ending.rs` | fixed | `Credits_FadeOutFixedCol`, `Credits_FadeColorAndBeginAnimating`, and `Credits_AddNextAttribution` match C's fixed-color countdown, BG2 scroll increment, `BG2SC` write at `$0c00`, four `room_bounds_y` derivations, BG3 scroll advance, attribution enqueue every 8 pixels, death-counter digit packet, `R16` page wrap, VRAM upload offset, byte terminator, NMI flag, and BG1/BG2 copy-register mirrors. Rust now performs those side effects and direct table indexes. |
| late credits final fade/hang helpers | `crates/zelda3/src/ending.rs` | verified | `Credits_BrightenTriangles`, `Credits_FadeInTheEnd`, and `Credits_HangForever` match C's frame-gated `INIDISP_copy` increment, palette filter call and completion gate, triangle animation call, and four fixed OAM entries for "THE END" using C's negative coordinate wrap values. |
| credits stop/fade tail | `crates/zelda3/src/ending.rs` | fixed | C resets full-word `R16 = 0x00c0` and `R18 = 0` when stopping the credits scroll, and calls `PaletteFilter_WishPonds_Inner()` after triangle dispersal completes. Rust now matches those word writes and the final palette filter call. |
| credits text/attribution tables | `crates/zelda3/src/ending.rs` | fixed | C directly indexes `kEnding_Credits_Offs`, `kEnding_Credits_Text`, `kEnding_MapData`, `kEnding_Digits_ScrollY`, `kEnding_Func9_Tab2`, `kEnding0_Offs`, and `kEnding0_Data`. Rust now uses direct asset/table indexing for these helpers instead of `unwrap_or(0)`/bounds fallbacks that could hide bad ending state. |
| ending draw table fallbacks | `crates/zelda3/src/ending.rs` | fixed | C directly indexes `kGeneratedEndSequence15[frame_counter]` and `kEndSequence_Dmds[j >> 1] + a * sprite_graphics[k]` before drawing exactly `a` entries. Rust now direct-indexes asset 73 and the draw-multiple table/slice instead of returning early or truncating the draw when the index is invalid. |
| polyhedral initialization | `crates/zelda3/src/ending.rs` | verified | Rust already matches C for `poly_config1`, `poly_config_color_mode`, NMI thread state, base position, `poly_var1`, model, `poly_a = 16`, `TS_copy = 0`, and `TM_copy = 0x16`. |
| ending source coverage | `crates/zelda3/src/ending.rs` | source-covered/runtime-open | Every `void` function in `../zelda3/src/ending.c` now has direct manual C/Rust comparison evidence in this ledger. Runtime-open because no late-ending/credits route currently exercises the tail text, attribution, final fade, and full credits sprite scenes against an external visual oracle. |
| verification | ending tail slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, and `cargo test -q -p zelda3 ending` pass. The focused test command currently has zero ending-specific tests; this remains a source-side parity pass until a late-credits route exists. |

## 2026-05-31 Dungeon Swamp/Flood Brightness Pass

Scope: manually compare the dungeon brightness, swamp pool, watergate, and
flood-dam submodule cluster in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| grayscale/fixed-color helpers | `crates/zelda3/src/dungeon.rs` | verified | `ApplyGrayscaleFixed_Incremental`, `Dungeon_ApproachFixedColor_variable`, `Dungeon_DoubleApplyAndIncrementGrayscale`, and `Module07_0A_ChangeBrightness` match C's low-5-bit target approach, COLDATA channel masks, double palette-filter call, lamp-cone call, and submodule reset on target color. |
| swamp drain state machine | `crates/zelda3/src/dungeon.rs` | verified | `Module07_0B_DrainSwampPool` matches C's counter-gated signed water-window deltas, activated-water-off early return, `AdjustWaterHDMAWindow` call, full BG1 tile fill from `SrcPtr(0x1e0)[0]`, quadrant reset, and tilemap prep states 2..5. |
| swamp flood state machine | `crates/zelda3/src/dungeon.rs` | verified | `Module07_0C_FloodSwampWater` matches C's quadrant prep states, pre-decremented water counter, depth-based `Dungeon_AdjustWaterVomit` calls, window/color-math register setup, intentional state-9 fallthrough into the raise-window step, spotlight/window updates, activated-water early return, and final vomit tile updates for `a == 0` or `a == 8`. |
| watergate/flood-dam helpers | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_FloodSwampWater_PrepTileMap`, `Dungeon_AdjustWaterVomit`, `FloodDam_PrepTiles_init`, `Watergate_Main_State1`, `FloodDam_Expand`, and `FloodDam_Fill` match C's quadrant/upload/tilemap behavior, byte writes inside the watergate HDMA variables, 10x4 watergate tile draw, three DMA packets, fill completion reset, and spotlight reset. Rust no longer masks the watergate source-table index and no longer silently no-ops invalid `Module07_0D_FloodDam` submodule states; those now fail loudly like C's direct table/index paths. |
| verification | dungeon swamp/flood slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; a dedicated swamp-palace/watergate gameplay route is still needed for external visual/runtime proof. |

## 2026-05-31 Dungeon Spiral-Stair State Pass

Scope: manually compare the spiral-stair submodule cluster in
`crates/zelda3/src/dungeon.rs` against `../zelda3/src/dungeon.c`,
without using the progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| spiral-stair dispatch and filter setup | `crates/zelda3/src/dungeon.rs` | fixed | `Module07_0E_SpiralStairs`, `Module07_0E_01_HandleMusicAndResetProps`, and `Module07_0E_02_ApplyFilterIf` match C's VRAM/attribute threshold, per-frame `HandleLinkOnSpiralStairs` call, 20-entry dispatch table, music reset branch, staircase timer values, transition reset call, palette-filter countdown behavior, and Link/tagalong invisibility handoff. Rust now panics on invalid submodule states instead of silently no-oping where C directly indexes `kDungeon_SpiralStaircase[subsubmodule_index]`. |
| spiral-stair background/layer handoff | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_SyncBackgroundsFromSpiralStairs`, `Dungeon_SpiralStaircase17`, `Dungeon_SpiralStaircase18`, `Module07_0E_00_InitPriorityAndScreens`, `Module07_0E_13_SetRoomAndLayerAndCache`, and `RepositionLinkAfterSpiralStairs` match C's follower reset edge case, temporary Y/layer adjustment around high-priority exiting, BG scroll mirrors, room-layout adjustment, TM/TS rules, floor delta, blip/quadrant cache, landing countdowns, lower-layer plane tables, final room cache reset, and Link reposition offsets. |
| spiral-stair wall priority and movement helpers | `crates/zelda3/src/dungeon.rs` | fixed | `SpiralStairs_MakeNearbyWallsHighPriority_Entering`, `SpiralStairs_MakeNearbyWallsLowPriority`, `SpiralStairs_MakeNearbyWallsHighPriority_Exiting`, `HandleLinkOnSpiralStairs`, and `SpiralStairs_FindLandingSpot` match C's wall-position lookup, BG2 priority bit writes, movement velocities/timers/facing changes, low-byte X target comparison, reposition/follower reinit, staircase sound effects, and landing movement. Rust now prepares the two overlay DMA packets in the entering and low-priority wall helpers before setting `nmi_copy_packets_flag`, matching C's two `Dungeon_PrepOverlayDma_nextPrep` calls. |
| verification | dungeon spiral-stair slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; a dedicated spiral-stair gameplay/oracle route is still needed for visual/runtime proof. |

## 2026-05-31 Dungeon Straight-Stair/Landing-Wipe Pass

Scope: manually compare the straight inter-room stair, landing-wipe, and
fall-recovery submodule cluster in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| landing-wipe spotlight submodule | `crates/zelda3/src/dungeon.rs` | fixed | `Module07_0F_LandingWipe`, `Module07_0F_00_InitSpotlight`, and `Module07_0F_01_OperateSpotlight` match C's two-entry dispatch, spotlight open, sprite/iris table updates, window/math register clears, subsubmodule reset, queued music restore, and Link animation/OAM calls. Rust now panics on invalid landing-wipe substates instead of silently no-oping where C directly indexes `kDungeon_Submodule_F[subsubmodule_index]`. |
| straight inter-room stair state machine | `crates/zelda3/src/dungeon.rs` | fixed | `Module07_11_StraightInterroomStairs` and its state handlers match C's attribute/VRAM thresholds, staircase countdown and speed modifier timing, Link direction/velocity, long-entry animation, running-state reset, sound/music writes, fade and room-load sequence, BG char triggers, sprite reset, scroll-camera step, destination layer/floor adjustments, music/filter finish, and final room cache reset. Rust now panics on invalid straight-stair substates instead of silently no-oping where C directly indexes `kDungeon_StraightStairs[subsubmodule_index]`. |
| fall recovery scroll | `crates/zelda3/src/dungeon.rs` | verified | `Module07_14_RecoverFromFall` and `Module07_14_00_ScrollCamera` match C's two-case switch, two-pixel-per-frame approach toward cached BG2 scroll coordinates, substate increment when both axes match, dark-room guard around BG mirror, and drowning recovery call. Rust keeps the invalid-state no-op here because C's `switch` has no default and no direct table index. |
| verification | dungeon straight-stair slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; dedicated straight-stair, landing-wipe, and fall-recovery gameplay/oracle routes are still needed for visual/runtime proof. |

## 2026-05-31 Dungeon Warp/Peg/Pressure Submodule Pass

Scope: manually compare the warp-pad, crystal-peg update, pressure-plate, and
rescued-maiden submodule cluster in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| warp-pad dispatch and mosaic/filter helpers | `crates/zelda3/src/dungeon.rs` | fixed | `Module07_15_WarpPad`, `Module07_15_01_ApplyMosaicAndFilter`, `Module07_15_04_SyncRoomPropsAndBuildOverlay`, `Module07_15_0E_FadeInFromWarp`, and `Module07_15_0F_FinalizeAndCacheEntry` match C's VRAM/attribute threshold, 15-entry teleport dispatch table, mosaic control, `MOSAIC_copy = mosaic_level | 3`, palette filter calls, room `$17` floor override, BG mirror/layout adjustment, TM/TS rules, quadrant upload, fade-in mosaic decrement, BGMODE write, visited-quadrant save, submodule reset, and room-entry cache reset. Rust removed a non-C `$0104` spotlight special case from `Module07_15_01_ApplyMosaicAndFilter` and now panics on invalid warp-pad substates instead of silently no-oping where C directly indexes `kDungeon_Teleport[subsubmodule_index]`. |
| crystal peg and pressure plate submodules | `crates/zelda3/src/dungeon.rs` | verified | `Module07_16_UpdatePegs` matches C's preincrement, three-frame gate, step dispatch, final crystal-peg attribute flip, and submodule/subsubmodule reset. `Module07_17_PressurePlate` matches C's predecrement return gate, Link Y adjustment, common-tile update coordinate derivation from `word_7E04B6`, tile `$0e`, and saved-submodule restore. Rust keeps the invalid-state no-op in the peg switch because C uses a `switch` with no default after the frame gate. |
| rescued maiden state machine | `crates/zelda3/src/dungeon.rs` | verified | `Module07_18_RescuedMaiden` matches C's fade/palette gate, BG1/BG2 crystal-tile clearing, BG offset/floor offset resets, quadrant reset, crystal palette pass, Link immobilization, boss-room crystal tile pattern, alternating quadrant-upload state dispatch, NMI thread increment, polyhedral/cutscene initialization, and submodule/subsubmodule reset. This confirms the previously fixed reverse boss-room lookup remains aligned in this full local state-machine comparison. |
| verification | dungeon warp/peg/pressure slice | verified-open | `cargo fmt -p zelda3 --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; dedicated warp-pad, peg-toggle, pressure-plate, and maiden-crystal gameplay/oracle routes are still needed for visual/runtime proof. |

## 2026-05-31 Dungeon Mirror/Triforce/Falling-Entrance Pass

Scope: manually compare mirror fade, Triforce-door room draw, and dungeon
falling-entrance loading in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| mirror fade | `crates/zelda3/src/dungeon.rs` | verified | `Module07_19_MirrorFade` matches C's mosaic reset call, predecremented `INIDISP_copy`, main/submodule handoff to overworld, `nmi_load_bg_from_vram` clear, `last_music_control = music_unk1`, and conditional translucency palette revert. |
| Triforce-door room draw | `crates/zelda3/src/dungeon.rs` | fixed | `Module07_1A_RoomDraw_OpenTriforceDoor_bounce` matches C's immobilization, bytewise `R16` countdown semantics, ambient sound, sword/direction release, four-frame gate, door tile source table, 8x4 BG2 tile writes, watergate DMA packet, final attr writes, room-bound change, submodule reset, and NMI copy flag. Rust now direct-indexes the four-entry Ganon door source table instead of falling back to source `0` on invalid state. |
| dungeon falling entrance | `crates/zelda3/src/dungeon.rs` | fixed | `Module11_DungeonFallingEntrance` and `Module11_02_LoadEntrance` match C's switch/no-default dispatch, entrance-music gate, frame-gated palette filter, load-sprite state, state-4 fallthrough into landing handling, pit landing finalization, force blank, CGWSEL setup, entrance load, key restore, HUD rebuild, Link visibility/speed/OAM setup, byte-local Y delta, full-word `dungeon_room_index_prev` and `tiledetect_which_y_pos[0]` writes, room reload/custom attr/animated tile/attribute sequence, palette/tileset reset, bunny palette branch, HDMA, HUD refill, ambient SFX, submodule set, and song-bank load. Rust now uses full-word writes for the two C `uint16` assignments and direct-indexes both the entrance music asset and `kDungAnimatedTiles[main_tile_theme_index]`. |
| verification | dungeon mirror/falling slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; dedicated mirror, Triforce-door, and falling-entrance gameplay/oracle routes are still needed for visual/runtime proof. |

## 2026-05-31 Dungeon Quadrant/Camera Helper Pass

Scope: manually compare post-transition quadrant persistence and dungeon camera
scroll helpers in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| quadrant visited/save helpers | `crates/zelda3/src/dungeon.rs` | fixed | `SetAndSaveVisitedQuadrantFlags`, `SaveQuadrantsToSram`, `Dungeon_FlagRoomData_Quadrants`, and `Dung_SaveDataForCurrentRoom` now match C's direct `kQuadrantVisitingFlags[...]` indexing and direct `save_dung_info[dungeon_room_index]` writes. Rust no longer masks the quadrant-flag index with `& 0x0f` or silently skips the room save behind a RAM-length guard. |
| quadrant/camera adjust helpers | `crates/zelda3/src/dungeon.rs` | verified | `AdjustQuadrantAndCamera_{right,left,down,up}` match C's quadrant bit toggles, `Dungeon_AdjustQuadrant` call, `RoomBounds_{Add,Sub}A` call on the correct axis, and final quadrant-save call. `HandleEdgeTransition_AdjustCameraBoundaries` matches C's transition-direction store, direction-bit branch, camera bounds tables, quadrant-dependent table index, and `low + 2` high-bound write. |
| dungeon camera scroll loop | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_HandleCamera` matches C's Y then X handling, Z-adjusted Y probe, signed velocity absolute value, per-pixel loop gates, room-bound checks, BG2 scroll update before the `$ffff` room guard, BG1 subpixel half-scroll, camera low/high update, and final BG1/BG2 mirroring for BG2 property values `0,2,3,4,>=6`. |
| verification | dungeon quadrant/camera slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for the reviewed cluster; broader room-transition route coverage is still needed for final runtime proof. |

## 2026-05-31 Dungeon Transition Scroll/Subtile Pass

Scope: manually compare inter-room transition scroll and subtile landing helpers
in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| inter-room scroll step | `crates/zelda3/src/dungeon.rs` | verified | `DungeonTransition_ScrollRoom` matches C's preincremented `transition_counter`, direction index from `overworld_screen_transition`, BG1 offset reset, signed `kStaircaseTab3` delta, horizontal/vertical BG1/BG2 scroll update masked with `~1`, delayed Link coordinate movement after `kStaircaseTab4`, target comparison through the contiguous scroll-target words, quadrant-save call, transition-counter reset, and flood-quadrant upload when `submodule_index == 2`. |
| straight-stair camera scroll | `crates/zelda3/src/dungeon.rs` | verified | `Module07_11_0A_ScrollCamera` matches C's Link/tagalong invisibility, vertical BG1/BG2 scroll update masked with `~3`, target comparison, lower-level submodule offset into `kStaircaseTab5`, Link Y correction, visibility restore, and substate advance. |
| subtile transition landing | `crates/zelda3/src/dungeon.rs` | verified | `DungeonTransition_FindSubtileLanding`, `SubtileTransitionCalculateLanding`, `Dungeon_IntraRoomTrans_State5`, `DungeonTransition_MoveLinkOutDoor`, and `CalculateTransitionLanding` match C's torch/player reset, landing-class remap, signed subtile offset adjustment, low-byte Link coordinate write on the transition axis, room-save OR, moving-animation gate, doorway flag clear, forced movement reset, transition reset, signed two-pixel Link exit movement, low-byte target compare, BG2 attribute probe, and landing class writeback to `byte_7E004E`. |
| verification | dungeon transition scroll/subtile slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This slice had no code drift; runtime proof still needs route coverage that exercises each transition class. |

## 2026-05-31 Dungeon Entrance Load State Pass

Scope: manually compare dungeon entrance loading and full-room upload helpers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| full room upload wrapper | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LoadAndDrawRoom` matches C's HDMA save/disable, room load, transition/map-state reset, repeated BG1/BG2 quadrant upload sequence, HDMA restore, NMI subroutine clear, map-state clear, and subsubmodule reset. Rust factors the quadrant loop into `Dungeon_UploadRoomQuadrants`, whose body matches the C `TileMapPrep_NotWaterOnTag`, `NMI_UploadTilemap`, `Dungeon_PrepareNextRoomQuadrantUpload`, `NMI_UploadTilemap` loop. |
| entrance exit-state cache | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_LoadEntrance` now matches C's `death_var5` branch: the overworld exit-state cache and the `overworld_screen_index`/`overlay_index` clears only run when `death_var5` is clear. Rust previously cleared `overworld_screen_index` and `overlay_index` unconditionally after the branch, which could clobber death-resume state. |
| starting-point/entrance field load | `crates/zelda3/src/dungeon.rs` | fixed | The shared Rust helper matches both C data branches for room, scroll, optional player coordinates, camera bounds, tilemap mask, door settings, scroll targets, room bounds, tile/floor/palace, BG layer, quadrant size, and quadrant index fields. Rust now writes `which_entrance` as a word in the starting-point path, tests the C `WORD(sram_progress_indicator)` expression before loading Link coordinates, and preserves the high byte of `link_z_coord` by writing only its low byte like C's `BYTE(link_z_coord) = 0xff`. |
| entrance room scratch data | `crates/zelda3/src/dungeon.rs` | fixed | The movable-block and torch initialization copies now fail on missing assets and copy the exact asset ranges C copies: `kMovableBlockDataInit`, the 116-byte torch-data junk overlay into `movable_block_datas[99]`, `kTorchDataInit`, and `kTorchDataJunk` at `dung_torch_data[144]`. The memorized-tile and revealed-pot clears, orange/blue barrier reset, and `byte_7E04BC` reset match the C final state. |
| verification | dungeon entrance load slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for entrance-load state; dedicated death-resume and entrance-return runtime routes are still needed for final proof. |

## 2026-05-31 Dungeon Push-Block Interaction Pass

Scope: manually compare push-block movement/falling/collision helpers and the
straight-stair setup helper in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| push-block slide/fall state | `crates/zelda3/src/dungeon.rs` | fixed | `PushBlock_Slide` and `PushBlock_HandleFalling` now match C's signed comparison against `index_of_changable_dungeon_objs[1] - 1`, timeout predecrement/sign test, timeout reset, four-step state advance, low-byte-only `BYTE(dung_replacement_tile_state[y]) = 0`, selected changeable-object slot clear, and collision call after velocity application. Rust previously used wrapping `u8` comparisons and cleared the full replacement-tile word. |
| push-block velocity and sprite recoil | `crates/zelda3/src/dungeon.rs` | fixed | `PushBlock_ApplyVelocity` matches C's direct facing-table index, Link velocity zeroing, signed 12-pixel block movement on the selected axis, subpixel/lo/hi writeback, target-nibble test, replacement-tile increment, drag/direction flag clear, and reverse sprite collision loop/recoil write. Rust no longer masks the facing index with `& 3`; invalid pushed-block facing now fails like a bad C table index would. |
| push-block collision and straight-stair setup | `crates/zelda3/src/dungeon.rs` | verified | `PushBlock_HandleCollision` matches C's safe-return high-byte cache, direction-bit scan from 3 down, axis selection, overlap math, drag-state update, Link coordinate/velocity correction, and final indoor camera/door handler. `UsedForStraightInterRoomStaircase` matches C's ancilla-13 clearing loop, animation/subpixel/timer resets, sprite-damage disable, near SFX selection, and `tiledetect_which_y_pos` word writes. |
| verification | dungeon push-block slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. This is source-side coverage for push-block interactions; a dedicated pushed-block gameplay route is still needed for final runtime proof. |

## 2026-05-31 Dungeon Spiral/Layer Effect Pass

Scope: manually compare spiral-stair motion and dungeon layer-effect handlers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| spiral-stair motion | `crates/zelda3/src/dungeon.rs` | verified | `HandleLinkOnSpiralStairs` and `SpiralStairs_FindLandingSpot` match C's previous-coordinate cache, animation-step early return, damage/incapacitation/auxiliary reset, branch-specific tired timer decrement and signed velocity writes, moving-animation call, countdown/facing update, low-byte signed X-distance test against `tiledetect_which_y_pos[1]`, follower reinitialization, landing-target word update, SFX selection, sprite-damage clear, landing velocity selection, and low-byte landing compare. |
| layer-effect dispatch and palette effects | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_HandleLayerEffect`, `LayerEffect_Nothing`, `LayerEffect_Agahnim2`, `LayerEffect_InvisibleFloor`, and `LayerEffect_Ganon` match C's handler table for states 0-7, no-op states 0/1, Agahnim palette flash/restore frames, CGRAM update increments, invisible-floor tile-count palette toggle, and Ganon floor-count TS/CGADSUB rules. |
| moving/rapid layer scroll | `crates/zelda3/src/dungeon.rs` | fixed | `LayerEffect_Scroll`, `LayerEffect_Trinexx`, and `LayerEffect_WaterRapids` now use C's byte subpixel source `dung_some_subpixel[1]` at `$041d`, store the wrapped byte result, derive velocity from the unmasked byte+delta sum, and write/read full 16-bit `dung_floor_{x,y}_vel` words. Rust previously read a word from `$041e`, overlapping unrelated moving-wall state, and only updated the low velocity byte. |
| verification | dungeon spiral/layer-effect slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Dedicated moving-floor/rapid-water gameplay routes are still needed for final runtime proof. |

## 2026-05-31 Dungeon Custom Attr/Bunny Helper Pass

Scope: manually compare the dungeon custom tile attribute loader and dungeon
`Link_CheckBunnyStatus` helper in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| custom tile attributes | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_LoadCustomTileAttr` now matches C's exact `memcpy(&attributes_for_tile[0x140], &kDungAttrsForTile[kDungAttrsForTile_Offs[aux_tile_theme_index]], 0x80)`. Rust previously tolerated missing or short assets and zero-filled bytes instead of failing like a bad C table/copy would. |
| dungeon bunny recoil helper | `crates/zelda3/src/dungeon.rs` | fixed | `link_check_bunny_status` now matches C `Link_CheckBunnyStatus`: only when `link_player_handler_state == kPlayerState_RecoilWall` does it restore the state to ground, temporary bunny, or permanent bunny based on `link_is_bunny_mirror` and `link_item_moon_pearl`. Rust previously cleared bunny flags based on moon-pearl/light-world state, which was not this dungeon helper. |
| verification | dungeon custom attr/bunny helper slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Runtime-open for focused dark-world bunny/recoil and custom-tile room routes. |

## 2026-05-31 Dungeon Main Loop/Torch Door Pass

Scope: manually compare the main dungeon frame loop and torch/door interaction
front half in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| main dungeon loop gate | `crates/zelda3/src/dungeon.rs` | fixed | `module07_dungeon` now matches C's `enhanced_features0 & kFeatures0_MiscBugFixes && main_module_index != 7` skip point immediately after the submodule dispatch. Rust previously applied a later partial gate inside room tags, allowing push-block, CHR, camera, and room-tag setup to run after a submodule had already switched the main module away from dungeon. |
| torch timers and openable door front half | `crates/zelda3/src/dungeon.rs` | verified | `dungeon_process_torches_and_doors` matches C's four-frame torch timer decrement/extinguish path, Link-facing attribute probe offsets, lower-level attr bit, openable-door key index, door-direction rejection, breakable-wall dash debris/SFX/submodule path, big-key door message gate, small-key decrement, panned door-open SFX, invisible eye-watch door open/close bit update, overlay DMA prep, attr reload, NMI copy flag, and SFX. |
| slashable curtain/door path | `crates/zelda3/src/dungeon.rs` | verified | The B-button frame-4 interaction path matches C's OAM-offset tile probe order, `0x6c` curtain normalization, attr writes, four-tile draw source selection, slashable-door type check, opened-door bit updates, door counters/key index, door source tile draw, toggle-door attr reload, overlay DMA prep, arbitrary pan, and NMI copy flag. |
| verification | dungeon main loop/torch door slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Runtime-open for focused door/curtain/invisible-eye-door gameplay routes. |

## 2026-05-31 Dungeon Exploding Wall Cleanup Pass

Scope: manually compare exploding-wall cleanup in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| exploding-wall cleanup | `crates/zelda3/src/dungeon.rs` | fixed | `dungeon_clear_away_exploding_wall` matches C's immobilization gate, message-buffer wait, `word_7E045E` clear, door-animation reset, current-door index copy, tilemap address predecrement, blast-wall draw/stripe calls, NMI core-update disable, two-step blast counter, final opened-door bits, axis-specific blast flag/quadrant size update, quadrant cache word copy, blast-wall attr load, counter reset, quadrant save, immobilization clear, and NMI copy flag. Rust now clears only byte `g_ram[12]` like C instead of clearing the full `R12` word and clobbering `R13`. |
| blast-wall draw/stripe helpers | `crates/zelda3/src/dungeon.rs` | verified | `Door_BlastWallExploding_Draw` and `ClearAndStripeExplodingWall` match C's source tile ranges, variable-width fill, edge-pair clears, VRAM packet address mapping, vertical/horizontal stride choice from door direction, packet sizing, copied BG2 tile rows, and terminator write. |
| verification | dungeon exploding-wall cleanup slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Runtime-open for focused bombable-wall gameplay routes. |

## 2026-05-31 Dungeon Liftable/Bomb Destructible Pass

Scope: manually compare liftable-tile queries, pot/lift replacement, and bomb
destructible handling in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| liftable tile query | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_CheckForAndIDLiftableTile` matches C's facing-indexed signed probe offsets, 8-pixel alignment masks, lower-level BG2 attr offset, `$70` attr class gate, replacement tile state lookup, zero-state carry-clear return, `$2020` lightened-hole return `$55`, and replacement low-nibble return table. |
| lift and replace liftable | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LiftAndReplaceLiftable`, `ThievesAttic_DrawLightenedHole`, `HandleItemTileAction_Dungeon`, `ManipBlock_Something`, and `RoomDraw_16x16Single` match C's unmasked point return coordinates, `R16/R18` writes, replacement state classes `$1010/$2020/$4040`, pot-item reveal before redraw, misc object index writes, four-piece attic-hole redraw, manipulated point derivation from object tilemap and Link high coordinate bits, sword/bush-smash feature gate, smashed-terrain spawn, bush poof, and `uint8` return behavior. |
| bomb destructibles | `crates/zelda3/src/dungeon.rs` | fixed | `bomb_check_for_destructibles` matches C's overworld fallback, 3x3-ish dungeon attr probe pattern from `k - $82`, `$62` destructible handling, room `$65` save bit, breakable-wall door type gate, debris coordinate/direction setup, SFX, and submodule handoff. Rust now calls `ThievesAttic_DrawLightenedHole(0, 0, &pt)` for attr `$62` like C; it previously skipped that redraw/replacement side effect and only set SFX. |
| verification | dungeon liftable/bomb destructible slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Runtime-open for focused pot, attic-hole, and bombable-object gameplay routes. |

## 2026-05-31 Dungeon Door Animation State Pass

Scope: manually compare door opening and shutter-door animation helpers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| door drawing dispatch | `crates/zelda3/src/dungeon.rs` | verified | `DrawDoorOpening_Step1`, `DrawShutterDoorSteps`, `DrawEyeWatchDoor`, the four `DoorDoorStep1_*` helpers, the directional `GetDoorDrawDataIndex_*` helpers, and directional draw-to-tilemap/object writers match C's current-door/key index writes, opposite-room boundary checks, stair-mask offset adjustments, adjacent-door redraw and DMA prep, single-door attr reload, open/closed/remapped graphics index rules, shutter animation source offsets, and orientation-specific tile writes. |
| shutter-door animation | `crates/zelda3/src/dungeon.rs` | fixed | `OperateShutterDoors` now matches C's full-word `++door_animation_step_indicator` comparisons for steps 4 and 8, the explicit low-byte `BYTE(door_animation_step_indicator) != 0x10` getout check, trapdoor-dependent open/close counter choice, shutter/two-way-shutter filter, opened-bit toggles, SFX, draw and DMA prep, attr reload at step 8, final current-door position, NMI disable/copy flags, and submodule reset. Rust previously incremented and compared only the low byte for the step-4/8 gates. |
| locked/cracked door animation | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_OpeningLockedDoor_Combined`, `OpenCrackedDoor`, and `DrawCompletelyOpenDoor` now match C's full-word step indicator semantics for skip, step 4, step 12, and step 16, opened-door bit set at step 12/skip, door-open counter values, draw/DMA/SFX/NMI updates, final attr reload, stair-mask complete-open redraw, full up-north straight/up-south straight count skips, `$3434` lower-half attr seed remap, south-down skip, down-north spiral attr writes, and submodule reset. Rust previously used byte-only step increments/comparisons and stopped `DrawCompletelyOpenDoor` after the up-north spiral groups. |
| toggle-door tile attrs | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LoadToggleDoorAttr_OtherEntry` and `Dungeon_LoadSingleDoorTileAttribute` match C's single-door attr reload followed by floor/palace toggle tile attr refresh, BG2-vs-BG1 attr table selection by `$80` class, two-byte attr reads at each toggle position, and OR masks `$1010`/`$2020` over both vertical attr entries. |
| verification | dungeon door animation state slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass after the word-step and complete-open attr fixes. Runtime-open for focused shutter/locked/cracked-door gameplay routes. |

## 2026-05-31 Dungeon Attribute Table/Object Attribute Pass

Scope: manually compare dungeon attribute-table rebuild and object/stair
attribute overlays in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature script.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| full/selectable attr table loader | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_LoadAttributeTable`, `Dungeon_LoadAttribute_Selectable`, `Dungeon_LoadBasicAttribute_full`, and `attribute_for_bg_tile` now match C's indicator reset, 0x1000/0x40 chunking, BG2 tile attr lookup, flip-bit merge for attr range `$10..$1b`, map-state increment at `$2000`, object/door attr pass ordering, crystal-peg flip gate, and assert-on-invalid selectable state. Rust no longer performs a non-C custom-attribute asset copy inside `Dungeon_LoadAttributeTable`; the C-equivalent copy remains in `Dungeon_LoadCustomTileAttr`. |
| object and stair attrs | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_LoadObjectAttribute` now includes C's star-switch attrs, full inter-room staircase attr families, in-room staircase kind/attr selection, wet/activated water ladder attrs, misc object attrs, torch attrs and `dung_index_of_torches` reset, chest and big-key lock attrs, lower-plane staircase attrs, and up-south water-stair attrs. Rust previously started at misc objects/chests and skipped the staircase/water/torch front and tail passes. |
| tag-suppressed big-key attrs | `crates/zelda3/src/dungeon.rs` | fixed | The chest/big-key attr section now matches C's `no_big_key_locks` branch: when chest count is nonzero and either room tag suppresses big-key locks, both normal chest attr writes and big-key lock attr writes are skipped. Rust previously skipped chest attrs but still wrote big-key lock attrs. |
| verification | dungeon attribute table/object attr slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --compare-lockstep-render /path/to/zelda3.sfc 2901 --input-script target/debug-frames/snes9x-save-probe-start.txt`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` pass. Runtime-open for focused stair, water-ladder, torch, and big-key-lock route coverage. |

## 2026-05-31 Dungeon Room Tag/Staircase Plane Pass

Scope: manually compare `Dungeon_HandleRoomTags`, `Dungeon_DetectStaircase`,
and the immediate quadrant-trigger helpers in `crates/zelda3/src/dungeon.rs`
against `../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| room-tag dispatch gate | `crates/zelda3/src/dungeon.rs` | fixed | `dungeon_handle_room_tags` now matches C's optional `enhanced_features0 & kFeatures0_MiscBugFixes && submodule_index != 0` return immediately after `Dungeon_DetectStaircase`. Rust previously continued into both tag routines after a staircase changed the submodule under that feature flag. |
| staircase destination plane | `crates/zelda3/src/dungeon.rs` | fixed | `Dungeon_DetectStaircase` now reads `cur_staircase_plane` from C's `dung_hdr_staircase_plane[j]` at `$063d + j`. Rust previously read `$063c + j + 1` through the hole/teleporter-plane symbol, which aliases the same bytes for `j=0..3` but obscured the C table split and risked future drift. |
| staircase probe and setup | `crates/zelda3/src/dungeon.rs` | verified | The staircase probe matches C's `link_direction & 12` gate, buggy signed Y lookup, BG2 attr probe including the left-facing `+0x80`, attr2 `$30..$37` gate, carrying-state Y restore, previous-room/quadrant save, inter-room transition-start calls, destination-room write, submodule/timer/input clears, sound effects, and straight-stair handoff. |
| quadrant tag helpers | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_NorthWestTrigger`, `Dung_TagRoutine_0x2A..0x30`, `RoomTag_QuadrantTrigger`, `Dung_TagRoutine_TrapdoorsUp`, `RoomTag_RoomTrigger`, `RoomTag_RoomTrigger_BlockDoor`, and `RoomTag_PrizeTriggerDoorDoor` match C's quadrant bit tests, room/screen clear checks, trapdoor state toggles, sound/submodule writes, and prize/tag clears. |

## 2026-05-31 Dungeon Switch/Pressure-Plate Tag Pass

Scope: manually compare switch-triggered room-tag doors, pressure-plate
updates, torch puzzle doors, and the immediate blast-wall tag setup in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| switch-triggered trapdoor word state | `crates/zelda3/src/dungeon.rs` | fixed | `RoomTag_SwitchTrigger_HoldDoor`, `RoomTag_SwitchTrigger_ToggleDoor`, `RoomTag_RoomTrigger_BlockDoor`, `RoomTag_PrizeTriggerDoorDoor`, `Dung_TagRoutine_TrapdoorsUp`, and `RoomTag_TorchPuzzleDoor` now preserve C's 16-bit `dung_flag_trapdoors_down` semantics in the paths where C reads or writes the word. Rust previously collapsed several of these to low-byte reads/writes. |
| pushed-block pressure-plate latch | `crates/zelda3/src/dungeon.rs` | fixed | The pushed-block replacement path now writes `related_to_trapdoors_somehow = dung_flag_trapdoors_down ^ 1` as a word like C. Rust previously wrote only the low byte, leaving a stale high byte possible for `RoomTag_SwitchTrigger_HoldDoor`'s word read. |
| pressure plate and blast-wall setup | `crates/zelda3/src/dungeon.rs` | verified | `PushPressurePlate`, `RoomTag_Switch_ExplodingWall`, `RoomTag_PullSwitchExplodingWall`, and `Dung_TagRoutine_BlastWallStuff` match C's attr gate, saved-module/menu handoff, Link Y nudge, tilemap update coordinates, shutter-check gate, blast-wall door scan, message-buffer coordinate writes, SFX, and blast-wall ancilla spawn. |

## 2026-05-31 Dungeon Water/Chest/Moving-Wall Tag Pass

Scope: manually compare water-state room tags, chest-reveal room tags, and
moving-wall room tags in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| water room tags | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_WaterOff`, `RoomTag_WaterOn`, `RoomTag_WaterGate`, and `RoomTag_OperateWaterFlooring` match C's statechange/save-bit gates, window/color math registers, submodule/subsubmodule writes, water HDMA variables, event/save updates, water tile redraw, overlay DMA prep, sound effects, and BG1 water-floor layout loop. |
| chest reveal tags | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_PushBlockForChest`, `RoomTag_TriggerChest`, `RoomTag_TorchPuzzleChest`, `RoomTag_OperateChestReveal`, and `RoomTag_BuildChestStripes` match C's trigger gates, chest-location attr writes, source tile order, VRAM upload packet layout, upload offset stride, map-state loop, SFX, and `nmi_load_bg_from_vram` set. |
| moving-wall tags | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_MovingWall_East`, `RoomTag_MovingWall_West`, `RoomTag_MovingWallShakeItUp`, `RoomTag_MovingWallTorchesCheck`, `MovingWall_MoveALittle`, and `RoomTag_AdvanceGiganticWall` match C's torch/statechange start gate, movement flag increment, immobilization/ambient state, signed wall targets, scroll offsets, NMI target calculations, finish state, shake offsets, and subpixel velocity behavior. |

## 2026-05-31 Dungeon Room-Tag Probe/Boss Tag Pass

Scope: manually compare the shared room-tag tile probes and adjacent boss/prize
room tags in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| room-tag tile probes | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_GetTilemapCoords`, `RoomTag_MaybeCheckShutters`, and `RoomTag_CheckForPressedSwitch` match C's Link coordinate formula, lower-level bit, `word_7E04B6` clear/write, immobilized/auxiliary early return, four-position probe order, two-row validation, accepted attr classes, returned low-byte attr, and pressed-switch `y_out` behavior. |
| boss/prize room tags | `crates/zelda3/src/dungeon.rs` | verified | `RoomTag_GetHeartForPrize`, `RoomTag_Agahnim`, `RoomTag_GanonDoor`, and `RoomTag_KillRoomBlock` match C's save-bit and pendant/crystal gates, falling-prize spawn failure return, event-bit check, translucency restore, sprite-state scan, falling-into-hole guard, immobilization/submodule/sword/button/R16 writes, quadrant check, screen-clear check, and tag clears. |
| activated-water attr helpers | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_SetAttrForActivatedWaterOff` and `Dungeon_SetAttrForActivatedWater` match C's color/window register writes, collision/TMW clears, water-stair attr-table loops, CGRAM flag increment, submodule/subsubmodule/NMI reset writes, and BG1/BG2 attr destinations. |
| Ganon torch extinguish helpers | `crates/zelda3/src/dungeon.rs` | verified | `Ganon_ExtinguishTorch_adjust_translucency`, `Ganon_ExtinguishTorch`, and `Dungeon_ExtinguishTorch` match C's translucency assert call, `byte_7E0333` setup/clear, torch tilemap high-bit clear, torch-data table write, lighting redraw, NMI copy flag, lights-out lit-torch decrement, `TS_copy`/fixed-color/submodule changes, and timer clear. |

## 2026-05-31 Dungeon Save/Transition Helper Pass

Scope: manually compare dungeon save/key helpers, room-layout quadrant sizing,
and the immediate inter-room transition/camera helpers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| dungeon save/key helpers | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LoadSongBankIfNeeded`, `Mirror_SaveRoomData`, `SaveDungeonKeys`, `Dung_HandleExitToOverworld`, `SaveQuadrantsToSram`, `Dungeon_FlagRoomData_Quadrants`, and `Dung_SaveDataForCurrentRoom` match C's queued-music exceptions, overworld-vs-dungeon song bank gate, mirror failure/success SFX and submodule writes, palace-index remap for key storage, save-dungeon-info word composition, and visited-quadrant OR behavior. |
| room layout and offset adjustment | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_AdjustForRoomLayout`, `Dungeon_AdjustAfterSpiralStairs`, `Dungeon_AdjustForTeleportDoors`, `add_dungeon_room_delta_x`, and `add_dungeon_room_delta_y` match C's layout/quadrant composition, full-size X/Y masks, `dung_unk2` low/high overrides, spiral room delta math, teleport-door room-index writes, BG2/room-bound/link coordinate word deltas, and tagalong Y high-byte refresh. |
| inter-room transition camera helpers | `crates/zelda3/src/dungeon.rs` | verified | `RoomBounds_AddA`, `RoomBounds_AddB`, `RoomBounds_SubA`, `RoomBounds_SubB`, `DungeonTransition_AdjustCamera_X`, `DungeonTransition_AdjustCamera_Y`, `HandleEdgeTransition_AdjustCameraBoundaries`, and the four `Dungeon_StartInterRoomTrans_*` helpers match C's quadrant flips, bounds adjustment order, current-room save, scroll target tables, camera low/high boundary tables, teleport-door branches, spiral-stair branch conditions, lower-level/palace toggles, and quadrant-fullsize recalculation. |
| file-select render sanity route | `crates/zelda3/src/select_file.rs`, renderer support | verified | Rechecked the clean new-file/name-save route after the reported select-screen indentation symptom. Current Rust frame output after saving a one-character name matches the C lockstep renderer through 5000 frames with `mismatched_pixels=0`; standalone dumped frame `target/debug-frames/rust-current-newgame-5000.png` shows the expected player-select layout, so no source drift was confirmed in this headless C/Rust path. |

## 2026-05-31 Dungeon Transition Scroll/Push-Block Pass

Scope: manually compare transition scroll, entrance room load, pushed-block
movement/collision, and straight/spiral staircase movement helpers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| transition scroll and subtile landing | `crates/zelda3/src/dungeon.rs` | verified | `DungeonTransition_ScrollRoom`, `Module07_11_0A_ScrollCamera`, `DungeonTransition_FindSubtileLanding`, `SubtileTransitionCalculateLanding`, `Dungeon_IntraRoomTrans_State5`, `DungeonTransition_MoveLinkOutDoor`, and `CalculateTransitionLanding` match C's transition-counter increments, BG1/BG2 offset masks, link coordinate nudges, scroll target checks, quadrant-save trigger, water quadrant upload gate, link visibility/tagalong state, subtile landing remap, and low-byte doorway target comparisons. |
| entrance room load/draw boundary | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LoadAndDrawRoom`, `Dungeon_LoadEntrance`, and `dungeon_load_entrance_fields` match C's HDMA disable/restore, quadrant upload loop abstraction, overworld exit-state caching, starting-point vs entrance data selection, scroll/camera/bounds setup, music track overrides, player/floor/palace/lower-level/quadrant fields, and movable-block/torch/pot reset buffers. |
| pushed-block motion and collision | `crates/zelda3/src/dungeon.rs` | verified | `PushBlock_Slide`, `PushBlock_HandleFalling`, `PushBlock_ApplyVelocity`, and `PushBlock_HandleCollision` match C's two-block index selection, timeout/falling cadence, byte write to replacement-tile state, velocity/subpixel accumulation, target nibble gate, drag-state update, sprite recoil scan, safe-return high-byte writes, overlap tests, link coordinate/velocity correction, and indoor camera/door call. |
| straight and spiral staircase movement | `crates/zelda3/src/dungeon.rs` | verified | `UsedForStraightInterRoomStaircase`, `HandleLinkOnSpiralStairs`, and `SpiralStairs_FindLandingSpot` match C's ancilla clear loop, animation/subpixel/timer setup, SFX selection, tile-detect target coordinates, previous-coordinate copies, damage/incapacitated/auxiliary clears, staircase direction velocities, tired/countdown timer handling, facing changes, follower reinit, and low-byte landing comparison. |

## 2026-05-31 Dungeon Layer Effect Pass

Scope: manually compare dungeon layer-effect dispatch and the individual
scroll/palette/color-math handlers in `crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| layer-effect dispatch | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_HandleLayerEffect` and `LayerEffect_Nothing` match C's `dung_hdr_collision_2` dispatch for effects 2 through 7, with the default no-op path preserved. |
| moving-floor scroll effects | `crates/zelda3/src/dungeon.rs` | verified | `LayerEffect_Scroll`, `LayerEffect_Trinexx`, and `LayerEffect_WaterRapids` match C's save-bit shutdown, movement flag gates, byte subpixel accumulation, signed velocity derivation, X/Y offset updates, BG1-vs-BG2 offset writes, Trinexx velocity carry/clear, and rapids negative X velocity. |
| palette and color-math layer effects | `crates/zelda3/src/dungeon.rs` | verified | `LayerEffect_Agahnim2`, `LayerEffect_InvisibleFloor`, and `LayerEffect_Ganon` match C's frame-counter flash windows, main/aux palette writes, CGRAM update flag increments, object-tilemap high-bit counts, invisible-floor palette toggles, Ganon `byte_7E04C5`, `TS_copy`, and `CGADSUB_copy` cases. |

## 2026-05-31 Dungeon Tail Crystal Helper Pass

Scope: manually compare the final `dungeon.c` tail helpers in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using the
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| custom attr and bunny tail helpers | `crates/zelda3/src/dungeon.rs` | verified | `Dungeon_LoadCustomTileAttr` and `Link_CheckBunnyStatus` are already covered by earlier focused passes; this tail check reconfirmed they match C's custom-attribute copy source/destination and recoil-wall-only bunny state restoration. |
| crystal cutscene dungeon setup | `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/ending.rs` | verified-open | `CrystalCutscene_Initialize` and `CrystalCutscene_SpawnMaiden` match C's color math, palette filter/countdown clears, crystal maiden palette writes, CGRAM flag, maiden dynamic spawn fields, follower graphics load through temporary follower indicators, floor offsets from BG2/link and low-byte BG1 Y, and collision mirror write. Runtime-open because the Rust dynamic-spawn helper safely returns on failed spawn where C would index through the returned slot; no focused route has exercised a failed crystal maiden spawn. `CrystalCutscene_InitializePolyhedral` remains covered in the ending tail pass. |

## 2026-05-31 snes9x Name-Entry Visual Oracle / Startup Audio Probe

Scope: verify the reported name-entry/select-screen indentation symptom against
the C lockstep renderer and snes9x libretro oracle, then isolate the next audio
parity failure without using progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| snes9x SRAM isolation | `zelda3-bin/src/main.rs` | fixed/verified | `--compare-snes9x-oracle`, `--dump-snes9x-frame`, `--dump-frame`, and playable lockstep can now seed the same SRAM on both sides. The snes9x wrapper no longer depends on persistent `/private/tmp/zelda3.srm`; it creates isolated `target/snes9x-oracle-save/<pid>/` save dirs and writes the requested `.srm` before `retro_load_game`, which removes false select-screen/layout diffs from stale external SRAM. |
| snes9x video color comparison | `zelda3-bin/src/main.rs`, `crates/snes/src/ppu.rs` | fixed/verified | Forced neutral snes9x video core options (`None` filter, disabled blur, neutral luminance/saturation/gamma, disabled fast PPU/DSP), fixed Rust-frame BGRA-to-RGBA comparison, and matched the C renderer brightness formula `((i << 3) | (i >> 2)) * brightness / 15`. With empty SRAM and the name-left/right script, `--compare-snes9x-oracle ... --ignore-audio --compare-from-frame 2450 --skip-snes9x-frames 83` completed 2451 frames with no video diff; `--compare-lockstep-render ... 2600` completed with `mismatched_pixels=0`. |
| snes9x video route alignment | `zelda3-bin/src/main.rs` | fixed/verified | Added video-only `--auto-align-video` for the snes9x oracle so route-specific startup/animation phase offsets do not stop the run before a real RGB layout difference. The mode requires `--ignore-audio`, advances snes9x only when it finds a full-frame RGB match, and otherwise still emits parity artifacts. Verified the live saved-slot select screen with `--load-sram saves/sram.dat --ignore-audio --compare-from-frame 2400 --auto-align-video` through 3000 frames, and the empty-SRAM new-name/save/select route with `--load-sram target/debug-frames/empty.sram --ignore-audio --compare-from-frame 2600 --auto-align-video` through 5000 frames. |
| C lockstep render route checks | `zelda3-bin/src/main.rs`, `crates/snes/src/ppu.rs` | verified | Re-ran the C lockstep renderer after the PPU source pass: empty-SRAM name/new-game route `--compare-lockstep-render ... 5000 --input-script scripts/inputs/file-select-new-game.txt --load-sram target/debug-frames/empty.sram` completed with `mismatched_pixels=0`; saved-slot select route `--compare-lockstep-render ... 3000 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat` completed with `mismatched_pixels=0`; house-exit route `--lockstep ... 9000 --input-script scripts/inputs/file-select-enter-game-exit-house.txt --load-sram saves/sram.dat` completed with `WRAM fnv1a64 = 866fd19dc6969067`. These checks point away from C-vs-Rust PPU source drift for the reported select-screen indentation and house-exit symptoms. |
| startup audio exactness | `crates/zelda3/src/audio.rs`, `crates/zelda3/src/spc_player.rs`, `crates/snes/src/apu.rs`, `zelda3-bin/src/main.rs` | open | The earliest snes9x audio failure is still frame 2 without snes9x pre-roll: Rust high-level SPC produces the first `$0a` startup SFX while snes9x remains silent. `--compare-snes9x-startup-audio ... 300` shows snes9x onset at frame 85 and Rust onset at frame 2, a delta of 83 frames. Applying the same 83-frame snes9x pre-roll aligns onset frame-level timing but not samples: `--compare-snes9x-oracle ... 3 --ignore-video --skip-snes9x-frames 83 --compare-from-frame 2` fails with first mismatch at interleaved sample 642 and Rust first nonzero at 1044. This is not a select-screen/rendering issue; it is the open split between the C high-level `SpcPlayer` model and real ROM SPC boot/upload timing. |
| full-APU debug bridge | `crates/zelda3/src/audio.rs`, `zelda3-bin/src/main.rs` | fixed/limited | `zelda_debug_full_apu_from_spc` now hides the SPC IPL ROM after copying the high-level SPC RAM and setting `PC=$0800`; before this, the debug APU kept fetching boot-ROM bytes at `$ffc0+`, so `--compare-startup-apu-impls` could not exercise the cloned RAM path. The asset song bank itself is data-only (`ram0800` remains zero), so this bridge is diagnostic only. A real raw-ROM bootstrap checkpoint from `--capture-rom-apu-bootstrap ... 2000000 21.477` reaches `spc=$085a rom=false payload_nz=51454 dsp_writes=19`, and `--compare-bootstrap-apu-startup ... 120` shows high-level and bootstrapped full-APU onset both at frame 2, while sample values still differ. |

## 2026-05-31 APU Shell Source Pass

Scope: manually compare the host-visible APU wrapper and SPC front matter in
`crates/snes/src/apu.rs` against `../zelda3/snes/apu.c`,
`apu.h`, and the opening of `spc.c`, without using progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| APU reset/cycle/register shell | `crates/snes/src/apu.rs` | verified | `ApuState::reset`, `cycle`, `cpu_read`, and `cpu_write` match C `apu_reset`, `apu_cycle`, `apu_cpuRead`, and `apu_cpuWrite`: reset enables IPL before SPC reset, clears RAM/ports/timers/history, sets `cpuCyclesLeft=7`, runs one SPC opcode when the cycle budget reaches zero, cycles DSP every 32 APU cycles, advances timers with 128/16 dividers, clears counters on `$fd-$ff` reads, handles `$f1` timer/port/ROM bits, records up to 256 DSP writes, writes DSP only for registers below `$80`, mirrors `$f8-$f9` into the extra APUI RAM bytes, and writes every CPU write back to APU RAM. |
| APU saveload prefix layout | `crates/snes/src/apu.rs` | verified | Rust `save_c_saveload_prefix` / `load_c_saveload_prefix` matches C `apu_saveload`'s prefix range from `Apu.ram` through `cpuCyclesLeft`, including native padding around `cycles`, six input-port bytes, four output-port bytes, three 5-byte timers, and the byte at offset `$10021`; nested DSP/SPC saveload remains handled by their own C-layout helpers. |
| SPC front matter | `crates/snes/src/apu.rs` | verified | Rust SPC reset/run helpers match C `spc_reset` / `spc_runOpcode` front matter: registers/flags/stopped/cycle budget reset, reset vector read from `$fffe/$ffff` through the APU read path so IPL visibility is honored, stopped opcodes return one cycle, opcode fetch increments PC with wrapping semantics, cycle table selection matches the C table, branch helper adds two cycles only for taken branches, and stack byte/word push/pull order matches C. |

## 2026-05-31 DSP Envelope Arithmetic Fix

Scope: manually compare the DSP reset/cycle/write/envelope core in
`crates/snes/src/apu.rs` against `../zelda3/snes/dsp.c`,
without using progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| DSP reset/cycle/write overview | `crates/snes/src/apu.rs` | verified | `DspState::reset`, `write`, `cycle`, `handle_echo`, `cycle_channel`, BRR decode, Gaussian interpolation, noise, and sample-buffer extraction match the C DSP structure and register behavior at source level, including the C `MY_CHANGES` immediate KON/KOF handling, ENDX clear-on-write, echo/FIR clipping points, sample buffer limit, and `sampleOffset` reset after `getSamples`. |
| exponential envelope decrease | `crates/snes/src/apu.rs` | fixed/verified | Rust previously used `wrapping_sub(((gain.wrapping_sub(1) >> 8) + 1))` plus extra clamps for decay/sustain/gain-mode 1. C's `uint16_t gain` is promoted to signed `int` for `(gain - 1) >> 8`, so gain 0 stays 0, and C does not clamp the decay/sustain/exponential-decrease states after subtraction. Added `dsp_exp_decrease_gain` and removed those extra clamps so ADSR decay, sustain, and GAIN exponential decrease now match C. |
| startup audio impact | `zelda3-bin/src/main.rs` | open | The DSP arithmetic fix does not remove the earliest snes9x startup mismatch: without pre-roll, Rust still emits the first startup SFX on frame 2 while snes9x is silent; with the known 83-frame snes9x pre-roll, onset is frame-aligned but sample values still differ starting at interleaved sample 642. That leaves external reset/upload/frame timing as the open issue, not this C DSP envelope arithmetic slice. |

## 2026-05-31 PPU Register/Scanline Shell Pass

Scope: manually compare the bounded PPU shell in `crates/snes/src/ppu.rs`
against `../zelda3/snes/ppu.c` and
`../zelda3/snes/ppu.h`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| reset and C saveload layout | `crates/snes/src/ppu.rs` | verified | `PpuState::reset`, `save_c_saveload`, and `load_c_saveload` match C `ppu_reset`/`ppu_saveload` for VRAM/CGRAM/OAM clears, object tile bases, BG layer reset fields, mode 7 flags, window/color-math state, forced blank/brightness defaults, and the exact C snapshot byte order: `$8000` VRAM words, 10 padding bytes, `$100` CGRAM words, 556 + 520 padding bytes, four 12-byte BG tilemap snapshot blocks, then 123 padding bytes. |
| register reads and writes | `crates/snes/src/ppu.rs` | verified | `read` and `write` match the C switch for `$2134-$2136` mode-7 multiply reads, default `$ff` reads, INIDISP, OAM pair writes and `$2103` high bit assertion, BGMODE/MOSAIC/BGSC/NBA, BG scroll latch behavior, VMAIN/VMADD/VMDATA increment timing, M7 matrix latch/sign/mask behavior, CGRAM pair writes, window registers, TM/TS/TMW/TSW, CGWSEL/CGADSUB/COLDATA, and SETINI. Rust has debug assertions for C assertions; release behavior keeps the same state writes. |
| window edge calculation | `crates/snes/src/ppu.rs` | verified | `ppu_windows_clear`, `insert_window_edge`, `ppu_windows_calc`, and `windows_for_layer` match C `PpuWindows_Clear`/`PpuWindows_Calc`: BG3 excludes large-screen side extension, other layers use `extraLeftCur`/`extraRightCur`, enabled windows are ignored when left exceeds right, edge insertion keeps the Snes9x-derived ordered span list, inverse bits are complemented in the same enabled+inversed cases, and unwindowed layers return the full clear span. |
| scanline shell and select-screen IRQ split | `crates/snes/src/ppu.rs`, `crates/zelda3/src/zelda_rtl.rs`, `crates/zelda3/src/select_file.rs` | verified | `run_line` follows C `ppu_runLine`/`PpuDrawWholeLine` shell behavior for line 0 no-op, mosaic modulo refresh, backdrop clear, forced-blank sprite suppression, visible-height clearing, and forced-blank line clearing. The select/name-entry scanline-128 BG3 split in `zelda_draw_ppu_frame` matches C `ZeldaDrawPpuFrame`, including the `$ff` IRQ handoff after `ReturnToFileSelect`; C's adjacent `zelda_snes_dummy_write(NMITIMEN, 0x81)` is an inline no-op in this native runtime, so Rust correctly has no extra side effect there. |

## 2026-05-31 PPU BG/Sprite Loop Pass

Scope: manually compare the PPU background and sprite loop bodies in
`crates/snes/src/ppu.rs` against `../zelda3/snes/ppu.c`,
without using progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| 4bpp and 2bpp BG loops | `crates/snes/src/ppu.rs` | verified | `draw_background_4bpp`, `draw_background_2bpp`, and `bg_tile_entry` match C `PpuDrawBackground_4bpp`/`PpuDrawBackground_2bpp`: screen/window gates, scroll-added Y, tilemap high/wide page selection, H/V flip row selection, tile-address stride, planar pixel bit extraction, transparent pixel skip, palette-priority shifts (`>> 6` for 4bpp, `>> 8` for 2bpp), and z-buffer compare/update behavior are preserved. Rust uses a per-pixel loop instead of C's clipped/full-tile macro blocks, but the tile pointer paging and `kPpuExtraLeftRight` destination offset resolve to the same VRAM and z-buffer positions. |
| mosaic BG loops | `crates/snes/src/ppu.rs` | verified | `draw_background_4bpp_mosaic` and `draw_background_2bpp_mosaic` match C's mosaic path for vertical mosaic source line, initial horizontal mosaic run length, tile/pixel sampling at the run start, repeated write across the run, tile-pointer advancement every 8 source pixels, and reset to `mosaicSize` for following runs. The Rust bounds checks are defensive only; in in-range C windows they produce the same writes. |
| mode 1 background dispatch | `crates/snes/src/ppu.rs` | verified | `draw_backgrounds` matches C `PpuDrawBackgrounds` for mode 1 ordering: sprites are copied onto an empty main/sub backdrop first when present, BG1 4bpp uses `0xc000/0x8000`, BG2 4bpp uses `0xb100/0x7100`, BG3 2bpp uses `0xf200/0x1200`, and each layer selects mosaic vs normal based on the corresponding `mosaicEnabled` bit. |
| sprite evaluation and merge | `crates/snes/src/ppu.rs` | verified | `evaluate_sprites` and `draw_sprites` match C `ppu_evaluateSprites`/`PpuDrawSprites`: OAM iteration wraps through 128 entries, `$f0` Y skips, high-OAM X/size bits are decoded the same way, X wrapping uses `extraLeftRight`, sprite and tile limits keep the C off-by-one counters, Y/H flips select row/column, object address and tile page math match, transparent pixels only fill empty OBJ buffer cells, and sprite merge either copies over the backdrop or priority-compares against BG data. |

## 2026-05-31 PPU Composition/Mode7 Pass

Scope: manually compare final whole-line composition, mode 7, and mode 7
upsampling in `crates/snes/src/ppu.rs` and
`crates/zelda3/src/zelda_rtl.rs` against
`../zelda3/snes/ppu.c` and
`../zelda3/src/zelda_rtl.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| brightness and mode 7 color map | `crates/snes/src/ppu.rs` | verified | `refresh_brightness_cache`, `begin_drawing`, and `current_render_scale` preserve C's brightness formula `((i << 3) | (i >> 2)) * brightness / 15`, 31-entry overrun padding for add/clamp-free color math, half-color pair table, and mode 7 `colorMapRgb` construction. Rust keeps the post-legacy invariant that the whole-line renderer is always active, so `MODE7_4X4` alone selects 4x mode 7 rather than also requiring the removed C `NewRenderer` flag. |
| normal mode 7 renderer | `crates/snes/src/ppu.rs` | verified | `draw_background_mode7` matches C `PpuDrawBackground_mode7` for 13-bit sign extension, center-relative scroll clipping, x/y flip, large-field/char-fill outside handling, mosaic and non-mosaic stepping, VRAM tile/pixel lookup, and direct z-buffer writes with the mode 7 priority. The Rust loops spell out C's do/while pointer movement and retain the same per-region window gating. |
| upsampled mode 7 renderer | `crates/snes/src/ppu.rs`, `crates/zelda3/src/zelda_rtl.rs` | verified | `set_mode7_perspective_correction` and `draw_mode7_upsampled` match C's HDMA-table perspective detection, low-zero fast path, four fractional interpolation offsets, center/scroll math, extra-left compensation, 4x horizontal and vertical expansion, optional half-color map, sprite overlay expansion, and side clearing. C stores `1.0f / high` even when `high == 0`, but that value is unused on the low-zero fast path; Rust stores `0.0` for the same unused case. |
| final color composition | `crates/snes/src/ppu.rs` | verified | `draw_whole_line` matches C `PpuDrawWholeLine`: forced blank clears the line, mode 7 4x bypasses the normal compositor, BG/subscreen rendering order is preserved, color-window bits produce the same clip/math masks, math-disabled and math-enabled loops use the same fixed-color/subscreen/half-color rules, RGB channels index the padded brightness tables just like C, and large-screen side clear widths match `extraLeftRight - extraLeftCur/RightCur`. |

## 2026-05-31 Dungeon Falling/Stair Dispatch Pass

Scope: manually compare the falling-transition, fat inter-room staircase, and
intra-room straight staircase dispatchers in `crates/zelda3/src/dungeon.rs`
against `../zelda3/src/dungeon.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| falling transition dispatch | `crates/zelda3/src/dungeon.rs` | fixed/verified | `Module07_07_FallingTransition` and states `00`, `06`, `0F`, `10`, and `11` match C's VRAM/attribute/grayscale threshold, room-load sequence, BG sync, falling fade-in Y high-byte carry, boss-music handoff, floor decrement and room exceptions, pit landing retry, CHR half-slot load, final cache/music reload gate, and exact state table order. Rust now panics for invalid `subsubmodule_index` values instead of silently no-oping, matching C's direct `kDungeon_Submodule_7_DownFloorTrans[subsubmodule_index]()` table dispatch. |
| fat inter-room stairs | `crates/zelda3/src/dungeon.rs` | verified | `Module07_06_FatInterRoomStairs`, `DungeonTransition_AdjustForFatStairScroll`, `ResetTransitionPropsAndAdvance_ResetInterface`, `ResetTransitionPropsAndAdvanceSubmodule`, and the BGC trigger helpers match C's attribute/VRAM thresholds, palette-bounce double-call, staircase countdown post-decrement timing, speed modifier switch at `$10`, direction selection from `which_staircase_index`, floor increment/decrement, sound effects, TM/TS layer handoff, torch/dark-room reset, palette cache copy, and substate advancement. |
| intra-room straight stairs | `crates/zelda3/src/dungeon.rs` | fixed/verified | `Module07_08_NorthIntraRoomStairs` and `Module07_10_SouthIntraRoomStairs` match C's per-frame staircase countdown, speed change at `20`, velocity/camera/animation order, lower-layer mirror/real-layer writes or toggles, common completion reset, and quadrant persistence. Rust now panics for invalid substates instead of silently no-oping, matching C's direct two-entry staircase dispatch tables. |
| transition reset wrappers | `crates/zelda3/src/dungeon.rs` | fixed/verified | The lower-case reset wrappers used by warp/ending paths now delegate to the same C-shaped `ResetTransitionPropsAndAdvance_*` helpers as the rest of dungeon code. This removes the remaining Rust-only room `$0104` mosaic/palette preservation and generic-reset `spotlight_open` side effect; C always clears the reset fields here, and dungeon spotlight opening belongs only to `Module07_0F_00_InitSpotlight`. |

## 2026-05-31 Dungeon Chest Open Helper Pass

Scope: manually compare chest-opening helpers in `crates/zelda3/src/dungeon.rs`
and the shared tile helper in `crates/zelda3/src/zelda_rtl.rs` against
`../zelda3/src/dungeon.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| normal and locked chests | `crates/zelda3/src/dungeon.rs`, `crates/zelda3/src/zelda_rtl.rs` | fixed/verified | `OpenChestForItem`, `OpenChestForItemResult`, and `apply_opened_chest_tiles` match C's tile dispatch, big-key door message, room-chest scan, opened-bit writes, 2x2 BG2 tile replacement, attr writes, VRAM upload packet layout, quadrant save, conditional chest SFX, and returned chest position. Rust now delays the opened-bit write for big-chest room entries until after the big-key check succeeds, matching C instead of marking a denied big chest as opened. |
| mini-game and big chests | `crates/zelda3/src/dungeon.rs` | fixed/verified | `OpenMiniGameChest` / `OpenMiniGameChestResult` match C's credit gates, chest-position probe, attr clears, intentional `pos + XY(0, 2)` tile-write bug, upload packet layout, prize RNG reuse avoidance, unique prize save bit, NMI flag, SFX, and returned item/position. `OpenBigChest` / `OpenBigChestResult` now match C's 4x3 tile writes, overlay DMA prep at `loc`, returned `loc + 2`, six word-sized attr writes at columns `0/2` across three rows, quadrant save, SFX, NMI copy flag, and `byte_7E0B9E` set. Rust previously omitted the overlay DMA prep and wrote the big-chest attr word pairs at columns `0/1`. |
| verification | dungeon chest-open helper slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused chest-opening visual/audio oracle routes. |

## 2026-05-31 Player Receive/Sword Control Pass

Scope: manually compare item receipt, sleep-state wakeup, and sword cooldown /
startup swing helpers in `crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| item receipt and opening bed state | `crates/zelda3/src/player.rs` | verified | `link_receive_item`, `link_tuck_into_bed`, and `link_state_sleeping` match C's auxiliary-state clear, receipt index/SFX/timer writes, hold-up-item state setup for receipt methods `0/3`, item-receipt ancilla spawn, HUD refresh exclusions, dash cancel, bed coordinates/state/timer/blanket spawn, snore cadence, input-gated wakeup, and recoil launch fields. |
| sword cooldown and startup swing | `crates/zelda3/src/player.rs` | fixed/verified | `link_handle_sword_cooldown`, `handle_sword_sfx_and_beam`, `link_check_for_sword_swing`, `handle_sword_controls`, and `link_reset_sword_and_item_usage` match C's signed cooldown predecrement, in-hand/position-mode guard, running gate, sword-beam health/type/ancilla scan, doorway sword-swing veto, button-mask transitions, sparkle/tile-detect timing, spin-charge handoff, and reset masks. Rust now directly indexes `FIRE_BEAM_SOUNDS[sword]` after C's only `0xfe/0xff` exclusions instead of clamping out-of-range sword indexes to the last table entry. |
| verification | player receive/sword control slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused sword/item visual/audio oracle routes. |

## 2026-05-31 Player Cape/Magic Helper Pass

Scope: manually compare cape, Y-button, and magic-cost helpers in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| cape equip/drain/unequip helpers | `crates/zelda3/src/player.rs` | fixed/verified | `link_item_cape`, `link_force_unequip_cape`, `link_force_unequip_cape_quietly`, `halt_link_when_using_items`, `link_handle_cape_passive_lift_check`, and `player_check_handle_cape_stuff` match C's signed transform timer gate, Y-button check, no-magic prompt, cape mode/timer setup, poof/SFX, damage disable, magic-drain cadence, misc-bug-fix anti-underflow branch, filtered-Y unequip, platform/collision halt behavior, and passive drain while lifting/grabbing. Rust now directly indexes `CAPE_DEPLETION_TIMERS[link_magic_consumption]` like C instead of clamping the consumption index. |
| Y-button and magic-cost helpers | `crates/zelda3/src/player.rs` | fixed/verified | `check_y_button_press`, `link_check_magic_cost`, `refund_magic`, and `link_item_reset_from_overworld_things` match C's button-mask/incapacitated/input gate, magic-cost subtraction/underflow test, no-magic prompt skip for item `3`, misc-bug-fix refund cap, item state clears, and direction-lock clear. Rust now directly indexes `LINK_ITEM_MAGIC_COSTS[item * 3 + link_magic_consumption]` like C instead of clamping the item/consumption index to the table tail. |
| verification | player cape/magic helper slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused cape/magic-drain oracle routes. |

## 2026-05-31 Player Somaria/Byrna/Net Table Pass

Scope: re-check the Somaria, Byrna, and net item handlers in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| Somaria startup path | `crates/zelda3/src/player.rs` | verified | `link_item_cane_of_somaria` still matches C's platform/door/Y-button gate, existing-block magic bypass, no-magic bugfix mask clear, Somaria-block spawn, refund semantics, halt/direction clear, animation timers, debug flag, position-mode cleanup, and final Y-button mask clear. Rust keeps the valid cleanup frame from indexing past the 3-entry delay table; C writes `kRodAnimDelays[player_handler_timer]` before checking `player_handler_timer != 3`, but that out-of-bounds read is not a semantic state dependency. |
| Byrna and net animation tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_item_cane_of_byrna`, `search_for_byrna_spark`, and `link_item_net` match C's existing-spark early return, Y-button/magic gates, Byrna init-spark type `$30`, SFX timing, mode/direction locks, net swing SFX, frame-table timing, cleanup fields, and OAM offset reset. Rust now directly indexes `BYRNA_DELAYS[player_handler_timer]` and `BUG_NET_TIMERS[(link_direction_facing >> 1) * 10 + link_var30d]` instead of clamping invalid table indices. |
| verification | player Somaria/Byrna/net table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused Somaria/Byrna/net oracle routes. |

## 2026-05-31 Player Medallion/Mirror Item Pass

Scope: re-check Ether, Bombos, Quake, and Magic Mirror item handlers in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| medallion startup and animation states | `crates/zelda3/src/player.rs` | fixed/verified | `link_item_ether`, `link_item_bombos`, `link_item_quake`, `link_state_using_ether`, `link_state_using_bombos`, and `link_state_using_quake` match C's Y-button, doorway/menu/save/sword/follower, ancilla, magic-cost, handler-state, delay/state, SFX, Quake Z-motion, and spell-spawn behavior. Rust now directly indexes the Ether, Bombos, and Quake delay/state tables after the same terminal step-counter rewrites as C instead of clamping table indexes. |
| mirror item and world-crossing state | `crates/zelda3/src/player.rs` | fixed/verified | `link_item_mirror`, `do_sword_interaction_with_tiles_mirror`, `link_state_crossing_worlds`, and `handle_followers_after_mirroring` match C's Y-button/follower gates, wrong-world rejection, dungeon room-save side effects, overworld mirror setup, bonk/deep-water retry behavior, state reset fields, moon-pearl/bunny handoff, and follower conversions. Rust now includes C's `!cheatWalkThroughWalls` guard before rejecting light-world mirror use when `MirrorToDarkworld` is disabled. |
| verification | player medallion/mirror item slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused medallion and mirror visual/audio oracle routes. |

## 2026-05-31 Player Hookshot/Powder/Shovel Table Pass

Scope: re-check hookshot setup plus powder and shovel item animation tables in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| hookshot setup direction table | `crates/zelda3/src/player.rs` | fixed/verified | `ancilla_add_hookshot_inner` matches C `AncillaAdd_Hookshot` allocation, timer/state/global reset, hookshot effect index, direction-derived velocity tables, and Link-relative launch coordinates. Rust now directly uses `link_direction_facing >> 1` for the direction table indexes like C instead of clamping to the final direction entry. |
| powder and shovel animation tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_item_powder`, `link_item_shovel_and_flute`, and `link_item_shovel` match C's Y-button/door gates, magic-cost and no-powder exit behavior, movement halt, animation step timers, powder spawn, shovel tile-detect/flute/prize/dirt side effects, and cleanup fields. Rust now directly indexes the valid powder and shovel timer/state tables like C instead of clamping the valid step indexes. |
| rod and hammer cleanup-table reads | `crates/zelda3/src/player.rs` | verified | `link_item_rod` and `link_item_hammer` retain their existing cleanup guards instead of reproducing C's out-of-bounds delay-table reads on the final cleanup frame. C overwrites the delay immediately afterward, so the read is not a semantic state dependency; direct Rust indexing would panic during normal item completion. |
| verification | player hookshot/powder/shovel table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused hookshot, powder, and shovel visual/audio oracle routes. |

## 2026-05-31 Player Spin/Dash Table Pass

Scope: re-check spin-attack animation and dash-state table paths in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| spin attack animation tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_state_spin_attack` matches C's auxiliary-state electrocution/recoil handoff, velocity/collision/camera order, spin SFX timing, cleanup fields, tile-detect call, and animation delay/state selection. Rust now directly indexes `LINK_SPIN_GRAPHICS_BY_DIR[step_counter_for_spin_attack + link_spin_offsets]` and `LINK_SPIN_DELAYS[step_counter_for_spin_attack]` like C instead of clamping. |
| dash state timing, direction, and follower tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_perform_dash` and `link_state_dashing` match C's dash startup fields, follower reacquire/write-warning condition, charging SFX cadence, charging dust/movement branch, countdown expiry follower conversion, stop-dash decision, turn-while-dashing feature path, and forced/default direction selection. Rust now directly indexes `TAGALONG_ARR1/2`, `DASH_TAB1`, and `DASH_TAB2` like C instead of silently skipping or clamping invalid indexes. |
| verification | player spin/dash table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused dash/spin visual/audio oracle routes. |

## 2026-05-31 Player Swim/Rebound Table Pass

Scope: re-check swimming momentum and dash rebound table paths in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| swimming momentum tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_set_momentum`, `link_flag_max_accels`, `link_set_ice_max_accel`, and `link_handle_swim_accels` match C's two-axis loop order, acceleration target selection, max-accel writes, and startup acceleration fields. Rust now directly indexes `SWIMMING_TAB2[link_flag_moving - 1]` like C instead of clamping `link_flag_moving` to the final entry. |
| dash rebound tables | `crates/zelda3/src/player.rs` | fixed/verified | `repel_dash`, `sprite_repel_dash`, and `link_apply_tile_rebound` match C's dash-tremor/SFX/rebound ordering, direction-derived velocities, swim collision direction writes, auxiliary/noise flags, scratch clear, and velocity-axis clear. Rust now directly indexes the rebound direction and swim-collision tables with `link_last_direction_moved_towards` and `(link_flag_moving - 1) * 4 + direction` like C instead of masking or clamping. |
| verification | player swim/rebound table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused swimming and dash-rebound visual/audio oracle routes. |

## 2026-05-31 Player Slope Judder Table Pass

Scope: re-check diagonal slope adjustment helpers in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| diagonal slope adjustment tables | `crates/zelda3/src/player.rs` | fixed/verified | `flag_moving_into_slopes_y` and `flag_moving_into_slopes_x` match C's coordinate low-bit selection, `R12`/diag-state offsets, misc-bug-fix Y adjustment branch, X adjustment branch, velocity sign handling, coordinate nudges, and `link_moving_against_diag_tile` flags. Rust now directly indexes the 32-entry avoid-judder table using C's computed offset instead of clamping to entry 31. |
| verification | player slope judder table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused diagonal-slope movement oracle routes. |

## 2026-05-31 Player Landing/Kickback Table Pass

Scope: re-check ledge landing and diagonal kickback table paths in
`crates/zelda3/src/player.rs` against
`../zelda3/src/player.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| ledge landing recoil tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_move_y_recoil_other` and `link_find_valid_landing_tile_diagonal_north` match C's landing probe loops, deep-water transition, distance-derived velocity/timer selection, diagonal landing setup, Z reset, auxiliary/electrocution flags, and handler-state writes. Rust now directly indexes the 32-entry landing velocity/timer tables from `diff >> 3` like C instead of clamping to entry 31. |
| diagonal kickback tables | `crates/zelda3/src/player.rs` | fixed/verified | `link_handle_diagonal_kickback` matches C's X-first/Y-second slope probes, coordinate restore, deadlock flag, velocity deltas, and diagonal coordinate nudges. Rust now directly indexes the X/Y kickback tables with signed velocity magnitude like C, including the known 16-entry negative-Y table. |
| verification | player landing/kickback table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused ledge and diagonal-kickback movement oracle routes. |

## 2026-05-31 Dungeon Torch Color Table Pass

Scope: re-check dungeon dark-room torch color table paths in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| torch fixed-color table | `crates/zelda3/src/dungeon.rs` | fixed/verified | `DungeonTransition_RunFiltering` and `Dungeon_LoadRoom_VerifyFloor` match C's dark-room fixed-color selection, lights-out fallback to torch slot `3`, CGWSEL/CGADSUB setup, palette countdown, mosaic reset, darkening flag, and room-entry reset flow. Rust now directly indexes `LIT_TORCHES_COLOR_PLUS[torch]` in these paths like C; the nearby lit-torch decrement path already used direct indexing. |
| verification | dungeon torch color table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused dark-room visual oracle routes. |

## 2026-05-31 Ancilla OAM Priority Table Pass

Scope: re-check ancilla OAM priority layer helpers in
`crates/zelda3/src/ancilla.rs` against
`../zelda3/src/ancilla.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| ancilla floor priority table | `crates/zelda3/src/ancilla.rs` | fixed/verified | `ancilla_prep_oam_coord` and `ancilla_prep_adjusted_oam_coord` match C's coordinate subtraction and OAM priority selection from `kTagalongLayerBits[ancilla_floor[k]]`. Rust now directly indexes `TAGALONG_LAYER_BITS[floor]` like C instead of clamping floor values to the final table entry. |
| verification | ancilla OAM priority table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused ancilla layer-priority visual oracle routes. |

## 2026-05-31 Dungeon Rescued-Maiden Crystal Lookup Pass

Scope: re-check the rescued-maiden crystal tile lookup in
`crates/zelda3/src/dungeon.rs` against
`../zelda3/src/dungeon.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| rescued-maiden crystal slot lookup | `crates/zelda3/src/dungeon.rs` | fixed/verified | `Module07_18_RescuedMaiden` still matches C's reverse boss-room lookup and crystal BG1 tile pattern. Rust now removes the fallback/clamp around the boss-room index and directly derives `FindInWordArray(kBossRooms, room) - 4` semantics with checked panics for invalid state, then indexes `CRYSTAL_TAB0[j]` like C. |
| verification | dungeon rescued-maiden crystal lookup slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused maiden-crystal visual oracle routes. |

## 2026-05-31 Sprite OAM Priority Table Pass

Scope: re-check the sprite floor priority table path in
`crates/zelda3/src/sprite.rs` against
`../zelda3/src/sprite.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| sprite floor priority table | `crates/zelda3/src/sprite.rs` | fixed/verified | `sprite_timers_and_oam` matches C's timer decrement, hit-timer priority, floor selection, and final object-priority write. Rust now directly indexes `SPRITE_PRIOS[floor]` like C's `kSpritePrios[floor]` instead of clamping floor values to the final table entry. |
| verification | sprite OAM priority table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `git diff --check`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused sprite-priority visual oracle routes. |

## 2026-05-31 Sprite Direct Draw/Type Table Pass

Scope: re-check small sprite draw/type table paths in
`crates/zelda3/src/sprite_main_npcs.rs` and
`crates/zelda3/src/sprite_main_small_bosses.rs` against
`../zelda3/src/sprite_main.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| bottle vendor draw data index | `crates/zelda3/src/sprite_main_npcs.rs` | fixed/verified | `BottleVendor_Draw` in C passes `&kBottleVendor_Dmd[sprite_graphics[k] * 2]` directly. Rust now uses the same direct `SPRITE_GRAPHICS * 2` base instead of clamping to the final valid two-entry draw pair. |
| Trinexx side-head X offset index | `crates/zelda3/src/sprite_main_small_bosses.rs` | fixed/verified | `Sprite_Sidenexx` in C indexes `kTrinexxHead_Xoffs[sprite_type[k] - 0xcc]` directly. Rust now indexes `K_TRINEXX_HEAD_XOFFS[idx]` directly instead of clamping side-head type values to the right-head entry. |
| verification | sprite direct draw/type table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `git diff --check`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused NPC and Trinexx draw visual oracle routes. |

## 2026-05-31 Sprite Throwable/Absorbable Table Pass

Scope: re-check throwable scenery and absorbable table paths in
`crates/zelda3/src/sprite.rs` against
`../zelda3/src/sprite.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| throwable scenery flags | `crates/zelda3/src/sprite.rs` | fixed/verified | `ThrowableScenery_Spawn` in C reads `kThrowableScenery_Flags[what]` directly before the indoor-pot override. Rust now directly indexes `K_THROWABLE_SCENERY_FLAGS[what]` instead of using a zero fallback for out-of-range `what`. |
| absorbable draw type tables | `crates/zelda3/src/sprite.rs` | fixed/verified | `SpriteDraw_AbsorbableTransient` in C directly indexes `kAbsorbable_Tab2[j - 0xd8]`, then `kAbsorbable_Tab1[j - 0xd8]` only when the numbered-absorbable table returns zero. Rust now follows the same direct table indexing instead of treating invalid absorbable types as single-small draws. |
| throwable debris SFX table | `crates/zelda3/src/sprite.rs` | fixed/verified | `ThrowableScenery_TransmuteToDebris` in C queues `kSprite_Func21_Sfx[a]` directly. Rust now indexes `K_SPRITE_FUNC21_SFX[a]` directly instead of clamping invalid debris categories to the final sound entry. |
| verification | sprite throwable/absorbable table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `git diff --check`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused pickup/throwable and absorbable visual/audio oracle routes. |

## 2026-05-31 Sprite Combat/Tile Table Pass

Scope: re-check sprite combat damage and tile-probe table paths in
`crates/zelda3/src/sprite.rs` against
`../zelda3/src/sprite.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| sword damage type table | `crates/zelda3/src/sprite.rs` | fixed/verified | `Sprite_CalculateSwordDamage` in C directly indexes `kSprite_Func14_Damage[a]`. Rust now directly indexes `K_SPRITE_FUNC14_DAMAGE[a]` instead of clamping invalid sword/button-derived damage classes. |
| player bump damage table | `crates/zelda3/src/sprite.rs` | fixed/verified | `Sprite_AttemptDamageToLinkPlusRecoil` in C directly indexes `kPlayerDamages[3 * (sprite_bump_damage[k] & 0xf) + link_armor]`. Rust now directly indexes `PLAYER_DAMAGES[idx]` instead of clamping invalid armor/damage combinations. |
| sprite tile-property probe offsets | `crates/zelda3/src/sprite.rs` | fixed/verified | `Sprite_CheckTileProperty` in C shifts `j` and directly indexes `kSprite_Func5_X[j]` and `kSprite_Func5_Y[j]`. Rust now uses the shifted index directly instead of forcing negative/too-large probe indexes into the last valid offset pair. |
| verification | sprite combat/tile table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `git diff --check`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused combat and sprite tile-collision oracle routes. |

## 2026-05-31 Select-File Name Table Pass

Scope: re-check name-entry lookup tables in `crates/zelda3/src/select_file.rs`
against `../zelda3/src/select_file.c`, without using
progress/signature scripts.

| Surface | Rust location | Verdict | Notes |
|---|---:|---|---|
| name-entry horizontal cursor table | `crates/zelda3/src/select_file.rs` | fixed/verified | `NameFile_DoTheNaming` in C reads the table bytes at the ROM addresses for `kNamePlayer_Tab0`, `kNamePlayer_Tab1`, and `kNamePlayer_Tab2`, including the intentional overread beyond the nominal 32-entry horizontal table. Rust now uses the same raw ROM addresses instead of indexing the in-Rust arrays directly. |
| name-entry character table | `crates/zelda3/src/select_file.rs` | fixed/verified | `NameFile_DoTheNaming` in C reads `kNamePlayer_Tab3` from the ROM address directly. Rust now does the same instead of using a table-local fallback path. |
| verification | select-file name table slice | verified-open | `cargo fmt -p zelda3 -p zelda3-bin --check`, `cargo check -q -p zelda3-bin`, `git diff --check`, `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 7600 --input-script scripts/inputs/file-select-enter-game.txt --load-sram saves/sram.dat`, and `cargo run -q -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 60000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram` pass. The 60k TAS route completed with `WRAM fnv1a64 = 3b99d54bddde282e`. Runtime-open remains for focused name-entry visual oracle routes. |
