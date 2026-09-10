# ROM-less play restored

The playable build can run without a ROM (`zelda3` with no arguments, and
the packaged app): assets come from the embedded pack. Since the
cycle-exact ROM timing plans landed (20 August) that build panicked at
the first dialogue: the ROM-less loaders requested ROM startup timing,
the dialogue module's entry then measures `Text_Initialize` by executing
the ROM, and the plan's constructor requires the ROM bytes
(`rom too small (0 bytes)`). Every other cycle-exact plan is gated the same
way, on the startup-timing flag alone.

ROM startup timing now means what its name says: `zelda3 <rom>` and every
replay and comparison path keep it on exactly as before, while ROM-less
play (`run_standalone_play` and the standalone smoke) uses a loader that
leaves it off, so the game runs on the unmeasured schedules the code
already carries for that mode. The flag's readers, the receipt-driven
timing tests that run without a ROM, and the comparison harness's
cold-start loader are untouched.

One more ROM-less gap sat behind the first. The message-line scroll's
two-frame lag machine (begun by the translated scroll for speed 4) is
completed by the ROM timing presentation pipeline: its return-only slice
and the retirement of its staged completion run only under ROM timing,
and without them the second scroll of a message began from a stale phase.
Without ROM timing the scroll now drains the line in one frame, as the
other speeds and the endpoint catch-up already do; ROM-less play therefore
shows a speed-4 message line one frame sooner than the original. The one
unit test that exercised the lag slices with timing off now enables ROM
timing, the mode in which those slices run, and the phase-machine panic
names the offending phase.

The standalone smoke (`--standalone-smoke <frames>`) gained switches so
the ROM-less build can be walked headlessly into the game:
`ZELDA3_SMOKE_INPUT_PULSE=<mask>` presses a joypad mask for eight of every
sixteen frames, `ZELDA3_SMOKE_INPUT_LEDGER=<oracle-av-hashes.jsonl>` replays
a cached route's inputs, `ZELDA3_SMOKE_SRAM=<initial.srm>` seeds the save
the route started from, `ZELDA3_SMOKE_KEEP_SRAM=1` keeps the local save,
and `ZELDA3_SMOKE_TRACE=1` prints the module every hundred frames. With the
promoted route's inputs and save the ROM-less build runs 12,000 frames
through the intro, name entry, file select, two dialogues and Link's house
without a panic; before the fix it panicked entering the first dialogue.

## Verification

All 1,721 library tests pass under the dev profile (the receipt-driven
dialogue tests included); the ownership scanner, readability and
projection discovery are unchanged. The fifteen binary tests that fail
need replay-bisect checkpoints that are not present on this machine and
fail identically without this change.

The main tree validated this fix on parity binary
`57677d4dd34db86ec1b22c315cd1e5d8a1921bc7b4de9ebeb219192fccb52971`: the
200,000-frame cached comparison matched every video and audio hash in
187.62 seconds (the first 200k on the threaded comparison in the main
tree), the frame 60000 and 150470 WRAM goldens match, the 200,000-frame
WRAM endpoint is the recorded `dd45975c…` image, and all 1,721 library
tests pass under the parity profile.
