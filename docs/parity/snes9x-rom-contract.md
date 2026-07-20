# Snes9x ROM boot contract

Parity fixes begin with the original ROM’s behavior, not with a renderer workaround.

The contract has two sources:

- The exact Snes9x 1.63 Libretro core runs the supplied ROM and emits ordered WRAM, OAM, VRAM, and DMA events.
- `docs/nes-ver2/ram_symbol_crosswalk.json` maps observed WRAM addresses to the ROM decompilation’s labels and the equivalent Rust state names.

`scripts/snes9x_boot_contract.py` converts the raw trace to deterministic JSON. A DMA event must include the ROM program counter that issued it; a frame must contain both `state-before` and `state-after`. Unknown state stays unknown—there is no seed, PPU fallback, or inferred value path.

Prepare a clean exact-revision Snes9x checkout, then capture and validate the boot window:

```sh
python3 scripts/prepare_snes9x_trace_oracle.py /path/to/clean/snes9x-1.63 --build
trace=$(mktemp /tmp/zelda3-snes9x-trace.XXXXXX)
ZELDA3_SNES9X_VRAM_TRACE="$trace" \
  target/debug/zelda3 --compare-snes9x-oracle /path/to/clean/snes9x-1.63/libretro/snes9x_libretro.dylib zelda3.sfc 100 --ignore-audio --scan-all
python3 scripts/snes9x_boot_contract.py "$trace" --frames 100 --validate \
  --output /tmp/zelda3-boot-contract.json
```

For every mismatch, work in this order:

1. Find the first differing contract event, not the first differing pixel.
2. Use the event’s WRAM symbols and DMA program counter to identify the ROM producer.
3. Implement that producer through Rust’s normal semantic state and asset/GPU render path.
4. Add the minimized trace window as a regression receipt.

The GPU renderer consumes semantic frame state only. It must never be repaired by directly setting PPU VRAM, OAM, CGRAM, or a special bootstrap texture.

To gate the Rust simulation before GPU readback, enable the live semantic contract:

```sh
ZELDA3_ASSERT_ORACLE_BOOT_CONTRACT=1 \
ZELDA3_RUST_BOOT_CONTRACT=/tmp/zelda3-rust-boot.jsonl \
  target/debug/zelda3 --compare-snes9x-oracle /path/to/snes9x_libretro.dylib zelda3.sfc 100 --ignore-audio --scan-all
```

It compares the symbolized frame-publication fields (`MAIN_MODULE_INDEX`,
`SUBMODULE_INDEX`, `NMI_BOOLEAN`, `INIDISP_COPY`, and the NMI upload flags)
immediately after each oracle frame. A failure names the first differing field
and frame before any GPU pixel comparison runs. The JSONL artifact preserves
the corresponding Rust boundaries for a durable receipt.
