# Annotating a translated routine with its cycle cost

The engine's cycle ledger (`crates/zelda3/src/cycle_ledger.rs`) accumulates
the master cycles the original 65816 would spend in the code the translated
engine runs. A routine is "annotated" when its translation charges the cost
of the assembly it corresponds to, block by block, as it executes. This is
the mechanical step of the native-timing program in
`docs/parity/romless-exact-play.md`: no separate model, the translation
prices itself.

## Tools

- `python3 scripts/rom_function_map.py <CName>` prints the ROM address of a
  C-port routine and where its translation lives in `crates/zelda3/src`.
- `python3 scripts/rom_block_costs.py <CName|address> [--m16] [--x16]` lists
  the routine's instructions with master cycles, grouped into basic blocks
  with block totals. Branches are priced not taken; `+6` is the taken
  increment. The entry register widths matter: pick `--m16`/`--x16` to match
  the code (a wrong width shows up as nonsense such as `BRK` in the
  listing). When in doubt, the per-instruction costs in a shadow profile
  (`ZELDA3_DEBUG_ROM_CPU_PROFILE`, `instructions_by_pc`) settle it.
- The C source at `~/Documents/zelda3/src/*.c` is the bridge between the
  assembly and the Rust: same control flow, with `// 80xxxx` addresses.

## Rules

1. At the routine's entry add `let _scope = crate::cycle_ledger::routine(0x00_841e);`
   (its ROM address). The scope records the routine's charge when it drops,
   so early returns need nothing.
2. Charge each basic block where the translation executes it:
   `crate::cycle_ledger::charge(N)`, with N the block total from the listing.
   A taken branch costs 6 more than a not-taken one: charge the branch's
   taken cost on the path that takes it.
3. Loops charge per iteration inside the loop body; data-dependent branches
   charge inside the matching `if`/`else`. The translation usually has the
   same branches; when it computes the same result a different way, charge
   what the ROM would have done for the same data (say so in a comment).
4. Calls to other routines are priced as the `JSR`/`JSL` instruction only (46
   or 62); the callee charges its own body when it is annotated. Do not
   charge a callee's cost in the caller.
5. A routine whose cost does not depend on its inputs can use one line:
   `crate::cycle_ledger::charge_routine(address, N)`, with the derivation in
   a comment (see `clear_oam_buffer` in `zelda_rtl/rtl_oam_obj.rs`).
6. `abs` operands in `$2000-$5FFF` cost 2 less per access when the data bank
   is a system bank (register writes in the NMI handler); the listing notes
   it. `abs,X`/`abs,Y` reads with 8-bit index registers cost 6 more when the
   index crosses a page; charge it when the data makes it so.
7. Never change behavior. Annotations only add `charge` calls and comments.
   Do not reorder, merge or skip any existing statement.
8. Comment every charge with the block's address range so a reviewer can
   check it against the listing.

## Worked example

`ClearOamBuffer` (`$00:841E`): `LDX #$60`, then four passes (X = $60, $40,
$20, $00) of `LDA #$F0` and 32 `STA $0801,X`-style stores over the OAM Y
bytes, closed by `TXA : SEC : SBC #$20 : TAX : BPL`. The listing gives the
loop block 1,306 not taken; the branch is taken three of four times:
16 + 4 x 1,306 + 3 x 6 + 42 = 5,300, a constant, so the translation charges
`charge_routine(0x00_841e, 5_300)`.

## What a scope records

A routine scope records the routine's self cost: the charges made while
it was open minus the charges of annotated scopes nested inside it. The
profiler reports the same quantity for the shadow CPU (a subroutine's
inclusive cycles minus the inclusive cycles of the frames it called,
interrupt handlers included), so an annotated caller and an annotated
callee are checked independently, and an unannotated callee costs nothing
on either side until it is annotated.

## Checking

The comparison against the shadow CPU is central (it needs the GPU
comparison harness, one run at a time): a cached comparison with both
`ZELDA3_DEBUG_CYCLE_LEDGER=<dir>` and `ZELDA3_DEBUG_ROM_CPU_PROFILE=<dir>`
set, then `python3 scripts/cycle_ledger_check.py <ledger-dir> <profile-dir>`
reports, per annotated routine, the hosts where the ledger and the profile
agree and the mismatches with both totals. A mismatch names the routine and
the host; the profile's per-instruction counts for that host show which
block was charged wrongly.
