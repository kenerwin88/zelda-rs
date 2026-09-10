# The game crate has no `unsafe`

Four `unsafe` blocks survived the port into the game crate, each a shape
the original C made natural and Rust does not need.

- **King Helmasaur's legs wrote through a shared reference.** The draw
  routine took the shared coordinate record every sibling routine takes,
  cast it to a raw mutable pointer, and refreshed the coordinates through
  it for the mouth routine that follows. Mutating through a shared
  reference is undefined behaviour in Rust even when the write happens to
  land. The routine now takes the record mutably, as the tail, mask and
  body routines already did, and assigns the refreshed coordinates
  directly; the caller passes it mutably.
- **The save-slot checksum fixer took a raw pointer** into the cartridge
  SRAM and rebuilt a slice from it. It now takes the slot base and borrows
  the slot from the SRAM directly; the separate slot wrapper is gone.
- **Two C-string helpers in the config parser** (`cstr_to_string` and
  `parse_bool` over `*const i8` and `*mut bool`) had no callers; the string
  form `parse_bool_str` is what the parser uses. Both are gone with the
  `CStr` import.

The crate-wide search for `unsafe` outside tests is now empty. Behaviour
is unchanged: the same bytes are written at the same points; only the
pointer casts and the dead C-ABI shims are gone.

## Verification

The King Helmasaur draw test coverage is unchanged and the file-select
checksum tests still pass over the same slot bytes. The library compiles
with no warnings in the parity, dev, and lib-test builds; readability,
projection discovery and the ownership scanner are unchanged. All 1,721
library tests pass under the dev profile.
