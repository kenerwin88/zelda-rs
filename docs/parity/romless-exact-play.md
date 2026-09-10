# Toward exact ROM-less play

## Direction

The no-argument launch (`zelda3` with no ROM) must reach the same
frame-exact behavior as the parity route without any ROM bytes at
runtime: the timing the engine still takes from the original code has
to become native Rust logic that matches it exactly. Embedding the ROM
in the binary was tried and rejected; this document records what the
receipt-less path looks like today, the fixes this batch made to it,
and the program that closes the gap.

## Where exactness comes from today

Full-route parity is proven for the engine driven by per-host timing
receipts from the pinned Snes9x trace core (`OriginalTimingHostReceipts`,
installed by the cached comparison from the cache's
`original-timing-host-receipts.jsonl.zst`; owner state `Live`). Live play
installs no receipts: the owner stays `PendingColdStart` and every host
runs the translated "atomic" schedule, one main iteration and one NMI
per frame. With a ROM loaded, eleven `RomCpuTimingRun` plans execute the
original code on the in-repo 65816 to decide a handful of boundaries
(dialogue initialization, dungeon module 7 advances, cached sprites,
spotlight tables, the poly-thread intro); without a ROM those plans are
unavailable and the flag-off path runs its unmeasured schedules.

In the first 200,000 route hosts, 20,657 (10.3%) carry timing the atomic
schedule cannot reproduce: a held NMI (13,522 hosts), a main-loop
interruption (1,937) or a continued call stack (16,732). Those are the
hosts a native timing owner has to predict.

## The program

1. **Replace the eleven ROM-CPU plans with native cycle models.** Each
   plan measures one routine's cycle cost from a known entry raster to
   the next NMI. The route cache holds the ground truth for every host
   (`ZELDA3_DEBUG_INSTALL_RECEIPTS` prints the vector), so each native
   model can be table-tested against thousands of measured instances
   before the ROM path is deleted.
2. **A native timing owner.** `OriginalTimingOwnerState::Live` was
   designed so that "a native timing owner can later publish the same
   Zelda-level receipt vocabulary without changing gameplay". The owner
   needs a per-host cycle budget model of the translated main loop (the
   cost of each module iteration as a function of its state) to decide
   where vblank lands: NmiAccepted gate, MainLoopInterrupted boundary,
   CallStackContinued. The 1.58M cached receipts are the regression
   oracle: `./parity receipt-compare` already checks sampled semantic
   receipts, and a native owner can be run against the whole cache
   without Snes9x.
3. **Validate the receipt-less path continuously.** The standalone smoke
   now walks the route's own inputs through live play
   (`ZELDA3_SMOKE_INPUT_LEDGER`, `ZELDA3_SMOKE_SRAM`, `ZELDA3_SMOKE_TRACE`,
   `ZELDA3_SMOKE_ROM=<rom>` to compare against a ROM-loaded launch). It
   presents each frame through the play loop's headless publication
   pipeline and hashes WRAM every 100 frames, so two launches can be
   compared frame for frame and a crash names its host.

## Receipt-less fixes in this batch

Walking the whole route's inputs through the receipt-less ROM-timing
path (the `zelda3 <rom>` live path) was the first long run of that path.
It found:

- **The message-line scroll lag frame.** `lane_rom_startup_run_main`
  claimed a returned main iteration while a scroll copy was still in
  flight and ran an ownerless leading NMI into the copy, which the NMI
  lifecycle assertion rejects (the crash the interim ROM-less fix
  avoided by turning timing off). The two leading-NMI lanes now require
  the scroll CPU to be idle, so the lag frame runs the ordinary
  game-loop path that copies the remaining passes around the frame's
  NMI.
- **A staged scroll completion at an ownerless leading NMI.** After the
  return-only slice stages the completed text, the next host's leading
  NMI publishes it through its BG3 DMA (as the open NMI does under
  receipts). The ownerless-NMI assertion now admits the two staged
  phases; its second check still proves the publication token was
  claimed. Under receipts these hosts never reach that lane, so the
  receipt-driven route is unchanged.
- **Dungeon-exit spotlight entry envelope.** Without a timing authority
  `Module0F_SpotlightClose` plans the iris close from a guessed raster
  entry envelope and asserted that both ends of the envelope give the
  same plan. At walk host 173,4xx they differ in whether Link's position
  integrates before the first NMI, which aborted play. The plan now keeps
  the earliest entry's plan, carries the widened next envelope, and
  reports the disagreement under `ZELDA3_DEBUG_SPOTLIGHT_ENVELOPE`. A
  timing authority never consults the envelope.
- **OAM model after the Module09 transition staging.** The transition
  staging runs Sprite_Main, LinkOam and the HUD refill provisionally,
  restores RAM and the native state, then keeps only the staged OAM
  bytes in RAM, which left the native OAM model one generation behind
  RAM (a debug coherence check fired at walk host 117,9xx). The two
  restore points now re-adopt the staged shadow and packed extended
  table into the model. The bridge writes through per byte and the whole
  model is never bulk-projected on the frame path, so no RAM byte
  changes.
- **Memorized-tile coherence check.** The smash and 32x32 map-update
  writers store an entry before raising the count, so the native model
  can hold one slot more than the count implies; the debug coherence
  check compared against a count-sized reload and fired past 32
  memorized tiles on one screen. The check now reloads the model's own
  extent (`load_from_ram_sized`). Debug builds only; no projection
  changed.

## Library suite time

One test, `tile_attributes_match_frozen_runtime_effects`, spent 104 of
the suite's 113 CPU seconds (131,072 tile executions, each digesting two
WRAM images); the other 1,705 tests under 50 ms sum to 1.1 s. Its
per-tile digests are independent, so it now splits the tiles across
threads while producing the identical frozen case text. The parity
profile suite runs in 13.7 s wall instead of 99.7 s; nothing was
removed.

## Evidence

Static checks are unchanged: the ownership scanner reports 0 same-mode
overlaps, 19 cross-mode overlaps and 60 overlapping bytes; RAM
readability (2,654 constants) and projection discovery pass; the dev
library test build has no warnings (no dead code).

Receipt-less walks of the route's inputs from its initial save: the
ROM-free launch walks 30,000 frames; the ROM-loaded live path walks
300,000 frames on the dev binary with every debug coherence check
enabled and the full 1,581,079-frame input ledger on the parity binary
is recorded below.

Parity binary
`ce3b59a2420c900868bb4968f96746143ab6dbee5f697a63a334739ecb532438`: the
200,000-frame cached comparison matched every video and audio hash, the
frame 60000 and 150470 WRAM goldens match, the 200,000-frame WRAM
endpoint is the recorded `dd45975c…` image, and all 1,721 library tests
pass under the parity profile in 14.66 seconds. The binary crate's
tests show the same fifteen environmental failures as before (missing
replay-bisect checkpoints).
