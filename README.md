# zelda3-rs

Rust port of [zelda3](https://github.com/snesrev/zelda3) -- a
reverse-engineered re-implementation of *The Legend of Zelda: A Link to the
Past* in C.

The original C project is expected next to this checkout as `../zelda3` by
default. Parity scripts compare Rust against that checkout.

This repository does not include a ROM, generated game assets, or packaged
binaries. Builders must provide their own legally obtained USA ROM when running
asset generation, lockstep, replay, or oracle commands.

Clone with submodules:

```bash
git clone --recurse-submodules <repo-url>
cd zelda3-rs
```

## Status

See [PROGRESS.md](PROGRESS.md) for what's built, what's stubbed, and where to
pick up next session.

See [docs/PORTING_MAP.md](docs/PORTING_MAP.md) for the C-to-Rust module/function
ledger used to work through the remaining port systematically.

See [GOALS.md](GOALS.md) for the rewrite plan and the approach (hybrid:
faithful port first to keep the byte-for-byte verification oracle working,
then refactor toward idiomatic Rust once each piece is verified green).

## Quick start

```bash
cargo fmt --all -- --check
cargo check -p snes -p zelda3 -p platform -p renderer -p assets
cargo test -p snes -p zelda3 -p platform -p renderer -p assets
python3 scripts/create_ci_assets.py --out-dir "$PWD/target/ci-assets/zelda3_assets"
ZELDA3_ASSETS_DIR="$PWD/target/ci-assets/zelda3_assets" cargo check -p zelda3-bin
ZELDA3_ROM=/path/to/zelda3.sfc cargo build -p zelda3-bin --release
./target/release/zelda3         # standalone playable binary, no ROM needed at runtime
./target/release/zelda3 <path-to-zelda3.sfc>
./target/release/zelda3 --lockstep <path-to-zelda3.sfc> [frames] [--input-script <path>] [--load-sram <path>] [--trace-state]
```

CI runs ROM-free package checks and builds `zelda3-bin` against generated
placeholder assets. Those placeholder assets only prove the binary build and
smoke path; playable asset generation and oracle parity still need a local ROM
and the C checkout.

## Generated Assets

The playable binary embeds the split files under `generated/zelda3_assets/`.
That directory is gitignored, so every builder must provide their own USA ROM
when creating the binary.

Generate the assets explicitly:

```bash
python3 scripts/extract_assets.py --rom /path/to/zelda3.sfc
cargo build -p zelda3-bin --release
```

Or let Cargo generate them during the binary build:

```bash
ZELDA3_ROM=/path/to/zelda3.sfc cargo build -p zelda3-bin --release
```

The extractor writes:

```text
generated/zelda3_assets/
  manifest.json
  asset_signature.bin
  asset_key_signature.bin
  assets/
    000-kSoundBank_intro.bin
    001-kSoundBank_indoor.bin
    ...
    164-kSomeTileAttr.bin
  images/
    057-kLinkGraphics.png
    064-kSprGfx.png
    065-kBgGfx.png
    066-kOverworldMapGfx.png
    ...
```

Cargo repacks those split files into a temporary asset blob under `target/`
while compiling, then embeds that blob in the executable. The generated folder
does not contain `zelda3_assets.dat`. The `.bin` files are the runtime assets;
the PNG files are previews for graphics assets and are not used by the runtime.

For CI or fresh-clone build checks without a ROM, create placeholder assets:

```bash
python3 scripts/create_ci_assets.py --out-dir "$PWD/target/ci-assets/zelda3_assets"
ZELDA3_ASSETS_DIR="$PWD/target/ci-assets/zelda3_assets" cargo check -p zelda3-bin
ZELDA3_ASSETS_DIR="$PWD/target/ci-assets/zelda3_assets" cargo run -p zelda3-bin -- --standalone-smoke 2
```

These files are zero-filled fixtures for build coverage. They are not playable
assets and should stay under `target/`.

The extractor writes a manifest with the source ROM SHA-1, per-asset sizes, and
per-asset SHA-1 values, plus the generated preview image list. It delegates
extraction to the original C checkout's `assets/restool.py`; set
`ZELDA3_C_SOURCE=/path/to/zelda3` if that checkout is not at
`../zelda3`.

## Automatic Parity

Run the combined behavior/render/audio oracle driver:

```bash
python3 scripts/full_parity.py --rom /path/to/zelda3.sfc
```

The default driver runs the Rust lockstep behavior/render comparison and the
translated C engine audio oracle under `../zelda3`.
Override the C checkout with `ZELDA3_C_REPO=/path/to/zelda3`, `--c-repo`, or
`--c-bin`.

For optional external-emulator checks, pass `--with-bsnes` or `--with-mesen`.
On macOS arm64, `--with-bsnes` will download the bsnes libretro core into
`external/bsnes-libretro/local/` if no core is found. You can override with
`BSNES_LIBRETRO_CORE=/path/to/bsnes_libretro.dylib`, `--bsnes-core`, or
`--no-install-bsnes`. The Mesen2 runner expects the local app under
`external/mesen2-oracle/local/` unless `--mesen-runner` is supplied.

The C audio oracle can also run directly:

```bash
python3 scripts/compare_c_audio.py --frames 120
```

The no-argument binary path uses embedded generated assets and does not read a
ROM at runtime. Explicit ROM, replay, and oracle commands still load the ROM and
look for `zelda3_assets.dat` next to the ROM or in the current working directory
so parity work can keep comparing against original-ROM behavior.

## Local Git Hooks

Install the tracked hooks when your checkout has the local parity dependencies:

```bash
scripts/install_hooks.sh
```

The pre-commit hook runs RAM readability guardrails, builds the release replay
binary, and runs the standard C/Rust replay parity gate. It expects the C
checkout at `../zelda3` by default and the ROM at `../zelda3/zelda3.sfc`.
Override script paths with the documented `--c-root`, `--rom`, and environment
options when your local layout differs.

## Fixtures

The repository tracks two non-ROM binary fixtures used by parity checks:

- `saves/zelda3-combined-route.sav`
- `scripts/inputs/tas-us-full-completion-smv.sram`

See [docs/fixtures.md](docs/fixtures.md) for what those files are and which
local build artifacts should stay out of git.

## License

This project is licensed under the MIT license. See [LICENSE.txt](LICENSE.txt).

See [NOTICE.md](NOTICE.md) for upstream attribution and repository artifact
policy.

## macOS Distribution Signing

Build a signed macOS zip with:

```bash
scripts/package_macos.sh
```

By default this uses ad-hoc signing (`SIGN_IDENTITY=-`), which is useful for
local verification but does not identify a trusted developer to other Macs. For
real distribution outside the Mac App Store, install a Developer ID Application
certificate and pass its identity:

```bash
SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
  scripts/package_macos.sh
```

To submit the signed zip to Apple's notary service, first store notarytool
credentials in Keychain, then pass that profile name:

```bash
xcrun notarytool store-credentials zelda3-notary

SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
NOTARY_PROFILE=zelda3-notary \
  scripts/package_macos.sh
```

The script writes `dist/zelda3-macos-<arch>/zelda3` and
`dist/zelda3-macos-<arch>.zip`, verifies the code signature, and notarizes the
zip when `NOTARY_PROFILE` is set.
