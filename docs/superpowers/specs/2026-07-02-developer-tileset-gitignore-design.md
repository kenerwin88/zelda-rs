# Developer-Tileset Artifact Gitignore + Regeneration — Design

**Date:** 2026-07-02
**Status:** Approved (brainstorming)

## Goal

Remove ~11 MB of ROM-regenerable generated artifacts from git history going
forward, while keeping them fully regenerable from the ROM and keeping the test
suite green both with and without the artifacts present (CI/fresh clones have no
ROM). Also stop the ~100 MB of untracked WIP dumps in the same directory from
being accidentally committed.

## Background / current state

`zelda3-bin/developer_tilesets/` holds generated artifacts, all produced by
`--dump-*` subcommands of `zelda3-bin` that replay the ROM:

| Artifact | Approx size | Regen command | Runtime/test reader |
|---|---|---|---|
| `assets_by_source.{png,json}` | 3.8 MB | `--dump-assets-by-source 1073092` | `modern_source_atlas::load_modern_source_atlas` + test `loads_committed_assets_by_source_atlas` |
| `dungeon_sheet.{png,json}` | 5.0 MB | `--dump-dungeon-sheet-png` | none (dev visualization output only) |
| `sprite_sheet.{png,json}` | 1.8 MB | `--dump-sprite-sheet-png` | none (dev visualization output only) |
| `dungeon_index_tiles.{bin,json}` | 0.4 MB | `--dump-dungeon-index-tiles` | `modern_dungeon_atlas` + test `loads_dungeon_atlas_and_resolves_by_theme` |
| `sprite_index_tiles.{bin,json}` | 1.3 MB | `--dump-sprite-index-tiles` | `modern_sprite_atlas` + test `loads_sprite_atlas_and_resolves_by_context_tile` |
| `overworld_index_tiles.{bin,json}` | 0.6 MB | `--dump-unique-overworld-tiles` (`run_dump_unique_overworld_tiles`, writer at `main.rs:9818`) | `modern_index_atlas` + test `loads_index_atlas_and_resolves_graphics_key` |

Note: `--dump-unique-overworld-tiles` also emits the large `overworld_unique_tiles.{png,json}` WIP files (gitignored); the regen script runs it for the `overworld_index_tiles` output and lets the WIP outputs land ignored.

**Kept tracked (out of scope):** `kakariko_town_tileset.json` — embedded at
compile time via `include_str!` (`main.rs:2366`) and a curated manifest, not a
pure ROM dump. Untracking it would break the build.

**Untracked WIP dumps to gitignore (never commit):**
`overworld_unique_cells.{png,json}` (~83 MB), `overworld_unique_tiles.{png,json}`
(~19 MB).

**Key constraints discovered:**
- Regeneration needs the gitignored ROM (`saves/zelda3.sfc`) plus, for
  `assets_by_source`, a full ~1.07M-frame replay (~90 s). CI and fresh clones
  have neither → artifact-dependent tests **must skip, not fail**, when absent.
- Footgun: `--dump-assets-by-source` defaults to **60 000** frames
  (`main.rs:10252`); the full route is **1 073 092**. Dumping the default
  silently truncates the atlas (misses cells that only appear later).
- Runtime consumers already degrade gracefully: the live `assets-anim` path
  (`main.rs:1408`) catches a load error and falls back to the VRAM-decoded
  modern path. Only tests hard-`.expect()` the files.
- Only `kakariko_town_tileset.json` is compile-time-embedded; every target
  artifact is read at runtime via `std::fs`, so untracking them cannot break the
  build.

## Architecture

Five independent pieces, each testable on its own:

1. **Untrack + gitignore.** `git rm --cached` (keep working copies) the six
   regenerable families; add `.gitignore` entries for those families and the WIP
   dumps. Keep `kakariko_town_tileset.json` tracked.

2. **Test skip-guards.** Rust has no first-class test skip, so each of the four
   loader tests early-returns with a clear message when its artifact is absent.
   A tiny shared helper keeps this DRY:
   ```rust
   /// Returns true if the artifact exists. When false, the caller should
   /// early-return (Rust has no native test-skip); print the standard message
   /// so a fresh clone knows how to regenerate.
   pub fn developer_artifact_present(path: &std::path::Path) -> bool {
       if path.exists() { return true; }
       eprintln!(
           "SKIP: {} absent — run scripts/regen_developer_tilesets.sh (needs the ROM)",
           path.display()
       );
       false
   }
   ```
   Each guarded test becomes: resolve the path, `if !developer_artifact_present(&p) { return; }`, then the existing assertions unchanged. With the file present the test runs exactly as before.

3. **Regen script** `scripts/regen_developer_tilesets.sh`:
   - Resolve ROM from `$ZELDA3_ROM` else `saves/zelda3.sfc`; if missing, print
     the expected path and exit non-zero.
   - Build `zelda3-bin` once (`cargo build --profile parity -p zelda3-bin`).
   - Run each dump with correct args; `assets_by_source` **must** pass
     `1073092`. Print each artifact written.
   - Idempotent; safe to re-run. macOS-safe (no `timeout` dependency).

4. **Footgun fix.** In `run_dump_assets_by_source`, when the effective frame
   count is below the full-route length, print a loud stderr warning
   (`WARNING: dumping N < 1073092 frames — atlas will be TRUNCATED ...`). The
   default value is unchanged; this is a warning only, so no behavior change for
   existing callers that already pass the full count.

5. **Docs.** `zelda3-bin/developer_tilesets/README.md`: states the artifacts are
   generated + gitignored, how to regenerate (`scripts/regen_developer_tilesets.sh`),
   and the file→command table above. Note that `kakariko_town_tileset.json` is
   an intentionally-committed curated manifest.

## Data flow

Fresh clone (no ROM) → artifacts absent → runtime uses classic/VRAM paths;
loader tests skip with a message → suite green.
Dev with ROM → run the regen script once → artifacts on disk (gitignored) →
loader tests run fully; `assets-anim` HD path loads the atlas.

## Error handling

- **Runtime:** unchanged; already falls back on load failure.
- **Tests:** skip (early-return + message), never fail, when an artifact is absent.
- **Script:** fail loudly with the expected ROM path when the ROM is missing;
  propagate any dump non-zero exit.

## Testing

- Unit test for `developer_artifact_present`: returns true for an existing temp
  file, false for a missing path (assert the boolean; the eprintln is a
  side-effect).
- The four guarded loader tests: pass unchanged when files are present; a
  documented manual check (rename one artifact away, run its test, confirm it
  skips cleanly and the suite stays green) — not automated, since it depends on
  local artifact state.
- `cargo build -p zelda3-bin` stays green (kakariko still embedded).
- The regen script is not run in CI (no ROM); it is exercised manually on a dev
  machine with the ROM.

## Out of scope

- `kakariko_town_tileset.json` (compile-time embedded, curated).
- Changing dump *contents* or the atlas format.
- Rewriting git history to purge already-committed blobs (this only stops future
  tracking; a history purge, if ever wanted, is a separate decision).
- Any change to the classic or modern render paths.
