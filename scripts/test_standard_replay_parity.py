#!/usr/bin/env python3
"""Run the standard C/R replay parity regression.

This is the commit-time gate for the canonical non-glitch replay route. It uses
the C checkout as the oracle and fails if Rust drifts in RAM/SRAM, player state,
RNG, sprite room masks, HP, speed, or other checkpoint fields covered by
replay_bisect.py. By default it also replays the full route from reset to the
final frame so the gate proves end-to-end standard-route state parity.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import copy
import re
import struct
import zlib
import os
import pathlib
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = ROOT.parent / "zelda3"
DEFAULT_ROM = DEFAULT_C_ROOT / "zelda3.sfc"
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_RUST_BIN = ROOT / "target" / "release" / "zelda3"
DEFAULT_CHECKPOINT_DIR = ROOT / ".cache" / "replay-bisect"
DISPLAY_FRAME = 1_045_814
DISPLAY_CHECKPOINT_FRAME = DISPLAY_FRAME - 1
TITLE_FADE_FRAMES = (680, 700, 800)
TITLE_WHITE_FRAME = 800
TITLE_MENU_LIVE_REPLAY_START = 900
TITLE_MENU_LIVE_FRAMES = 250
TITLE_MENU_LIVE_DUMP_FRAME = 90
# Verified free-play overworld checkpoint (main=9, sub=0). The previous
# 700,000 checkpoint was already in module 14 dialogue and could not open the
# equipment menu, so that gate compared an unrelated attract/dialogue scene.
MENU_CHECKPOINT_FRAME = 356_445
MENU_DISPLAY_FRAME = MENU_CHECKPOINT_FRAME + 90
MENU_LIVE_FRAMES = 100
MENU_LIVE_DUMP_FRAME = 90
MENU_AUDIO_LOCKSTEP_END_FRAME = MENU_CHECKPOINT_FRAME + 175
# Each render hash must come from a fresh replay process. The C diagnostic
# renderer mutates scratch RAM while producing a frame, so sampling every frame
# in one process changes the later oracle frames. These points cover the start,
# middle, and fully-open inventory animation without letting observation alter
# the route being observed.
MENU_RENDER_HASH_FRAMES = tuple(
    MENU_CHECKPOINT_FRAME + offset for offset in (1, 13, 25, 37, 49, 61, 73, 85, 90)
)
NAME_ENTRY_CHECKPOINT_FRAME = 12_000
NAME_ENTRY_RELEASE_FRAME = NAME_ENTRY_CHECKPOINT_FRAME + 180
STANDARD_REGRESSION_FRAMES = (
    1_000,
    12_000,
    42_998,
    80_000,
    112_078,
    180_000,
    202_254,
    202_255,
    350_000,
    700_000,
    1_073_092,
)
STANDARD_LATE_REGRESSION_CHECKPOINT_FRAME = 700_000
RENDER_HASH_CHUNK_FRAMES = (356_445, 700_000)
STANDALONE_EMBEDDED_FRAME = 500
STANDALONE_EMBEDDED_RAM_HASH = "3290e6db08347541"
# No local saves/sram.dat is available in the isolated smoke cwd, so SRAM stays
# at the all-zero embedded-startup baseline.
STANDALONE_EMBEDDED_SRAM_HASH = "b9d103fd6854a325"
RENDER_HASH_RE = re.compile(r"^render-hash frame=(\d+) hash=(0x[0-9a-fA-F]{8})$", re.MULTILINE)
STANDALONE_SMOKE_RE = re.compile(
    r"^standalone smoke completed frames=(\d+) "
    r"ram_fnv1a64=([0-9a-fA-F]{16}) "
    r"sram_fnv1a64=([0-9a-fA-F]{16})$",
    re.MULTILINE,
)
RUST_FRAME_DELAY_RE = re.compile(
    r"const\s+DELAYS_MS:\s*\[u64;\s*3\]\s*=\s*\[\s*17\s*,\s*17\s*,\s*16\s*\]"
)
RUST_AUDIO_SAMPLES_RE = re.compile(
    r"\(\(\s*534\s*\*\s*audio\.sample_rate\s+as\s+usize\s*\)\s*/\s*32_000\s*\)\.max\(1\)"
)
C_FRAME_DELAY_RE = re.compile(
    r"static\s+const\s+uint8\s+delays\[3\]\s*=\s*\{\s*17\s*,\s*17\s*,\s*16\s*\}"
)
C_NAME_ENTRY_TARGET_RE = re.compile(
    r"NameFileRomByte\(\s*0xda01\s*\+\s*selectfile_var5\s*\)"
)
STANDARD_FIELDS = ",".join(
    [
        "ramhash",
        "ram0",
        "ram1",
        "ram2",
        "ram3",
        "ram4",
        "ram5",
        "ram6",
        "ram7",
        "sramhash",
        "hp",
        "state",
        "speed",
        "lspeed",
        "rng",
        "roommask",
        "hist",
        "histmask",
        "main",
        "sub",
        "subsub",
        "saved",
        "indoors",
        "room",
        "ow",
    ]
)


def run(command: list[str], *, env: dict[str, str]) -> int:
    print("+ " + " ".join(command), flush=True)
    return subprocess.run(command, cwd=ROOT, env=env, check=False).returncode


def run_in(command: list[str], *, cwd: pathlib.Path, env: dict[str, str]) -> int:
    print("+ " + " ".join(command), flush=True)
    return subprocess.run(command, cwd=cwd, env=env, check=False).returncode


def run_capture(command: list[str], *, cwd: pathlib.Path, env: dict[str, str]) -> subprocess.CompletedProcess:
    print("+ " + " ".join(command), flush=True)
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.stdout:
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
    return result


def replay_checkpoint_pair(args: argparse.Namespace, frame: int) -> tuple[pathlib.Path, pathlib.Path]:
    return (
        args.checkpoint_dir / f"c-frame-{frame}.sav",
        args.checkpoint_dir / f"rust-frame-{frame}.sav",
    )


def run_pair(
    left: tuple[list[str], pathlib.Path, dict[str, str]],
    right: tuple[list[str], pathlib.Path, dict[str, str]],
) -> tuple[int, int]:
    left_command, left_cwd, left_env = left
    right_command, right_cwd, right_env = right
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        left_future = executor.submit(run_in, left_command, cwd=left_cwd, env=left_env)
        right_future = executor.submit(run_in, right_command, cwd=right_cwd, env=right_env)
        return left_future.result(), right_future.result()


def run_capture_pair(
    left: tuple[list[str], pathlib.Path, dict[str, str]],
    right: tuple[list[str], pathlib.Path, dict[str, str]],
) -> tuple[subprocess.CompletedProcess, subprocess.CompletedProcess]:
    left_command, left_cwd, left_env = left
    right_command, right_cwd, right_env = right
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        left_future = executor.submit(run_capture, left_command, cwd=left_cwd, env=left_env)
        right_future = executor.submit(run_capture, right_command, cwd=right_cwd, env=right_env)
        return left_future.result(), right_future.result()


def run_standalone_embedded_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    generated_dir = ROOT / "generated"
    hidden_dir = ROOT / ".cache" / "standalone-embedded-generated-hidden"
    if hidden_dir.exists():
        print(f"stale hidden generated directory exists: {hidden_dir}", file=sys.stderr)
        return 2

    moved_generated = False
    try:
        if generated_dir.exists():
            hidden_dir.parent.mkdir(parents=True, exist_ok=True)
            generated_dir.rename(hidden_dir)
            moved_generated = True

        with tempfile.TemporaryDirectory(prefix="zelda3-standalone-smoke-") as smoke_cwd:
            result = run_capture(
                [str(args.rust_bin), "--standalone-smoke", str(STANDALONE_EMBEDDED_FRAME)],
                cwd=pathlib.Path(smoke_cwd),
                env=env,
            )
    finally:
        if moved_generated:
            hidden_dir.rename(generated_dir)

    if result.returncode != 0:
        return result.returncode

    match = STANDALONE_SMOKE_RE.search(result.stdout)
    if not match:
        print("standalone embedded smoke did not print expected summary", file=sys.stderr)
        return 1

    frame, ram_hash, sram_hash = match.groups()
    expected = (
        str(STANDALONE_EMBEDDED_FRAME),
        STANDALONE_EMBEDDED_RAM_HASH,
        STANDALONE_EMBEDDED_SRAM_HASH,
    )
    actual = (frame, ram_hash.lower(), sram_hash.lower())
    if actual != expected:
        print(
            "standalone embedded smoke DIFF "
            f"expected frames={expected[0]} ram={expected[1]} sram={expected[2]} "
            f"actual frames={actual[0]} ram={actual[1]} sram={actual[2]}",
            file=sys.stderr,
        )
        return 1

    print(
        "standalone embedded smoke ok "
        f"frames={frame} ram={ram_hash.lower()} sram={sram_hash.lower()} "
        "generated_hidden=1"
    )
    return 0


def canonical_state_value(value: str) -> str:
    if "/" in value:
        return "/".join(canonical_state_value(part) for part in value.split("/"))
    if re.fullmatch(r"0x[0-9a-fA-F]+", value):
        return str(int(value, 16))
    if re.fullmatch(r"[0-9]+", value):
        return str(int(value, 10))
    return value


def parse_replay_state(output: str) -> dict[str, str]:
    state_line = next(
        (
            line
            for line in reversed(output.splitlines())
            if line.startswith("smv-test frame=")
            or line.startswith("replay-save completed frames=")
        ),
        "",
    )
    if not state_line:
        raise RuntimeError("no replay state line found")
    return {
        key: canonical_state_value(value)
        for key, value in re.findall(r"\b([a-z0-9_]+)=([^ \n]+)", state_line)
    }


def compare_common_state_fields(c_output: str, r_output: str, fields: list[str]) -> list[tuple[str, str, str]]:
    c_state = parse_replay_state(c_output)
    r_state = parse_replay_state(r_output)
    return [
        (field, c_state[field], r_state[field])
        for field in fields
        if field in c_state and field in r_state and c_state[field] != r_state[field]
    ]


def inventory_menu_state_failures(output: str, label: str) -> list[str]:
    state = parse_replay_state(output)
    expected = {
        "main": "14",
        "sub": "1",
        "map": "4",
        # BG3 has reached the C equipment-menu fully-open stop at -232.
        "bg3v": str(0xFF18),
    }
    return [
        f"{label} menu state {field} expected={value} actual={state.get(field, 'missing')}"
        for field, value in expected.items()
        if state.get(field) != value
    ]


def run_pacing_gate(args: argparse.Namespace) -> int:
    rust_platform = ROOT / "crates" / "platform" / "src" / "lib.rs"
    c_main = args.c_root / "src" / "main.c"
    rust = rust_platform.read_text()
    c = c_main.read_text()

    checks = [
        (
            C_FRAME_DELAY_RE.search(c),
            f"C frame delay pattern not found in {c_main}",
        ),
        (
            RUST_FRAME_DELAY_RE.search(rust),
            f"Rust frame delay pattern not found in {rust_platform}",
        ),
        (
            "lastTick += delays[frameCtr % 3]" in c,
            "C pacing no longer advances by delays[frameCtr % 3]",
        ),
        (
            "self.next_frame_tick += frame_delay(self.presented_frames)" in rust,
            "Rust pacing no longer advances by frame_delay(presented_frames)",
        ),
        (
            "g_frames_per_block = (534 * have.freq) / 32000" in c,
            "C audio samples-per-frame formula changed",
        ),
        (
            RUST_AUDIO_SAMPLES_RE.search(rust),
            "Rust audio samples-per-frame formula changed",
        ),
    ]
    failures = [message for ok, message in checks if not ok]
    if failures:
        print("pacing gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print("pacing ok frame_delay=17/17/16 audio_samples=(534*sample_rate)/32000")
    return 0


def run_c_oracle_gate(args: argparse.Namespace) -> int:
    c_select_file = args.c_root / "src" / "select_file.c"
    c_binary = args.c_root / "zelda3"
    select_file = c_select_file.read_text()

    failures = []
    if not C_NAME_ENTRY_TARGET_RE.search(select_file):
        failures.append(f"C name-entry Y target must read 0xda01 + selectfile_var5 in {c_select_file}")
    if "0xda0b + selectfile_var5" in select_file:
        failures.append(f"C name-entry Y target still contains stale 0xda0b address in {c_select_file}")
    if c_select_file.stat().st_mtime > c_binary.stat().st_mtime:
        failures.append(f"C oracle binary {c_binary} is older than {c_select_file}; rebuild with make -C {args.c_root} zelda3")

    if failures:
        print("C oracle gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print("C oracle ok name_entry_y_target=0xda01")
    return 0


def read_ppm_rgb(path: pathlib.Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    pos = 0

    def token() -> bytes:
        nonlocal pos
        while pos < len(data) and data[pos] in b" \t\r\n":
            pos += 1
        if pos < len(data) and data[pos] == ord("#"):
            while pos < len(data) and data[pos] not in b"\r\n":
                pos += 1
            return token()
        start = pos
        while pos < len(data) and data[pos] not in b" \t\r\n":
            pos += 1
        return data[start:pos]

    if token() != b"P6":
        raise ValueError(f"{path} is not a binary PPM")
    width = int(token())
    height = int(token())
    maxval = int(token())
    if maxval != 255:
        raise ValueError(f"{path} has unsupported maxval {maxval}")
    if pos < len(data) and data[pos] in b" \t\r\n":
        pos += 1
    rgb = data[pos:]
    if len(rgb) != width * height * 3:
        raise ValueError(f"{path} has unexpected pixel data length")
    return width, height, rgb


def read_png_rgba(path: pathlib.Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"{path} is not a PNG")
    pos = 8
    width = height = color_type = None
    idat = bytearray()
    while pos + 8 <= len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        payload = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            width, height, bit_depth, color_type, compression, filter_method, interlace = struct.unpack(
                ">IIBBBBB", payload
            )
            if bit_depth != 8 or color_type not in (2, 6) or compression or filter_method or interlace:
                raise ValueError(f"{path} has unsupported PNG format")
        elif kind == b"IDAT":
            idat.extend(payload)
        elif kind == b"IEND":
            break
    if width is None or height is None or color_type is None:
        raise ValueError(f"{path} is missing IHDR")

    channels = 4 if color_type == 6 else 3
    stride = width * channels
    raw = zlib.decompress(bytes(idat))
    rows = []
    prev = bytearray(stride)
    src = 0
    for _ in range(height):
        filter_type = raw[src]
        src += 1
        row = bytearray(raw[src : src + stride])
        src += stride
        for i in range(stride):
            left = row[i - channels] if i >= channels else 0
            up = prev[i]
            upper_left = prev[i - channels] if i >= channels else 0
            if filter_type == 1:
                row[i] = (row[i] + left) & 0xff
            elif filter_type == 2:
                row[i] = (row[i] + up) & 0xff
            elif filter_type == 3:
                row[i] = (row[i] + ((left + up) >> 1)) & 0xff
            elif filter_type == 4:
                p = left + up - upper_left
                pa = abs(p - left)
                pb = abs(p - up)
                pc = abs(p - upper_left)
                predictor = left if pa <= pb and pa <= pc else up if pb <= pc else upper_left
                row[i] = (row[i] + predictor) & 0xff
            elif filter_type != 0:
                raise ValueError(f"{path} has unsupported PNG filter {filter_type}")
        rows.append(bytes(row))
        prev = row

    rgba = bytearray()
    for row in rows:
        if channels == 4:
            rgba.extend(row)
        else:
            for i in range(0, len(row), 3):
                rgba.extend(row[i : i + 3])
                rgba.append(255)
    return width, height, bytes(rgba)


def compare_c_r_render(c_ppm: pathlib.Path, rust_png: pathlib.Path) -> tuple[int, tuple[int, int, int, int] | None]:
    c_width, c_height, c_rgb = read_ppm_rgb(c_ppm)
    r_width, r_height, r_rgba = read_png_rgba(rust_png)
    if (c_width, c_height) != (r_width, r_height):
        raise ValueError(f"render size mismatch: C={c_width}x{c_height} Rust={r_width}x{r_height}")
    diff_count = 0
    min_x = min_y = 1 << 30
    max_x = max_y = -1
    for y in range(c_height):
        for x in range(c_width):
            c_pos = (y * c_width + x) * 3
            r_pos = (y * r_width + x) * 4
            if c_rgb[c_pos : c_pos + 3] != r_rgba[r_pos : r_pos + 3]:
                diff_count += 1
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)
    bbox = None if diff_count == 0 else (min_x, min_y, max_x, max_y)
    return diff_count, bbox


def title_subtitle_palette_failure(
    label: str, width: int, height: int, pixels: bytes, channels: int
) -> str | None:
    if width < 210 or height < 148:
        return f"{label} title frame is too small for subtitle palette check: {width}x{height}"
    white_pixels = 0
    red_pixels = 0
    for y in range(135, 148):
        for x in range(90, 210):
            offset = (y * width + x) * channels
            red, green, blue = pixels[offset : offset + 3]
            white_pixels += int(red >= 240 and green >= 240 and blue >= 240)
            red_pixels += int(red >= 120 and red >= green * 2 and red >= blue * 2)
    if white_pixels < 200 or red_pixels > 5:
        return (
            f"{label} title subtitle palette expected white text "
            f"white_pixels>=200 red_pixels<=5 actual={white_pixels}/{red_pixels}"
        )
    return None


def menu_vertical_coverage_failure(
    label: str, width: int, height: int, pixels: bytes
) -> str | None:
    if width != 256 or height != 224:
        return f"{label} menu frame expected 256x224, actual={width}x{height}"
    lower_colored = 0
    for y in range(height // 2, height):
        for x in range(width):
            offset = (y * width + x) * 4
            if max(pixels[offset : offset + 3]) >= 24:
                lower_colored += 1
    if lower_colored < 10_000:
        return (
            f"{label} menu did not extend through the lower half "
            f"colored_pixels>=10000 actual={lower_colored}"
        )
    return None


def inventory_menu_panel_geometry_failure(
    label: str, width: int, height: int, pixels: bytes
) -> str | None:
    if width != 256 or height != 224:
        return f"{label} inventory frame expected 256x224, actual={width}x{height}"

    # Fully open, BG3-scrolled inventory geometry. These rectangles exclude the
    # colored borders and cover the black interiors of the lower ability and
    # equipment panels. A pause-colored or overworld backdrop can look busy
    # enough to fool a generic lower-half pixel count, but cannot satisfy this
    # panel-specific contract.
    panel_interiors = ((16, 152, 152, 208), (176, 152, 240, 208))
    near_black = 0
    sampled = 0
    for left, top, right, bottom in panel_interiors:
        for y in range(top, bottom):
            for x in range(left, right):
                sampled += 1
                offset = (y * width + x) * 4
                near_black += int(max(pixels[offset : offset + 3]) <= 16)
    if near_black < 8_000:
        return (
            f"{label} inventory lower panels are missing or clipped "
            f"near_black_pixels>=8000/{sampled} actual={near_black}/{sampled}"
        )
    return None


def dump_render_frame(args: argparse.Namespace, frame: int, *, env: dict[str, str], label: str) -> int:
    out_dir = ROOT / "target" / "standard-replay-render" / label
    out_dir.mkdir(parents=True, exist_ok=True)
    c_ppm = out_dir / f"c-{frame}.ppm"
    rust_png = out_dir / f"rust-{frame}.png"

    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    c_cmd = [
        str(args.c_root / "zelda3"),
        "--config",
        str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save",
        str(args.save),
        "--smv-test-frames",
        str(frame),
        "--dump-frame",
        str(c_ppm),
    ]
    rust_cmd = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(frame),
        "--dump-frame",
        str(rust_png),
    ]
    c_rc, rust_rc = run_pair((c_cmd, args.c_root, c_env), (rust_cmd, ROOT, env))
    if c_rc != 0:
        return c_rc
    if rust_rc != 0:
        return rust_rc

    diff_count, bbox = compare_c_r_render(c_ppm, rust_png)
    if diff_count:
        print(f"render DIFF frame={frame} pixels={diff_count} bbox={bbox}", file=sys.stderr)
        print(f"  C: {c_ppm}", file=sys.stderr)
        print(f"  R: {rust_png}", file=sys.stderr)
        return 1
    print(f"render ok frame={frame} pixels=0")
    return 0


def run_title_palette_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    for frame in TITLE_FADE_FRAMES:
        rc = dump_render_frame(args, frame, env=env, label="title")
        if rc != 0:
            return rc

    out_dir = ROOT / "target" / "standard-replay-render" / "title"
    c_width, c_height, c_rgb = read_ppm_rgb(out_dir / f"c-{TITLE_WHITE_FRAME}.ppm")
    r_width, r_height, r_rgba = read_png_rgba(out_dir / f"rust-{TITLE_WHITE_FRAME}.png")
    failures = [
        failure
        for failure in [
            title_subtitle_palette_failure("C", c_width, c_height, c_rgb, 3),
            title_subtitle_palette_failure("Rust", r_width, r_height, r_rgba, 4),
        ]
        if failure is not None
    ]
    if failures:
        print("title palette gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print(
        f"title palette ok frames={','.join(map(str, TITLE_FADE_FRAMES))} "
        f"white_frame={TITLE_WHITE_FRAME}"
    )
    return 0


def run_title_menu_live_present_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    out_dir = ROOT / "target" / "standard-replay-render" / "title-menu"
    out_dir.mkdir(parents=True, exist_ok=True)
    live_dump = out_dir / "live-gpu-menu.png"
    live_env = env.copy()
    live_env.update(
        {
            # Exercise the validation mode that previously rejected dynamic
            # title/file-entry art using counters from a different renderer.
            "ZELDA3_VARIANT_LIVE_STATS": "1",
            "ZELDA3_LIVE_WINDOW_PIXEL_PARITY": "1",
            "ZELDA3_LIVE_WINDOW_PIXEL_DUMP_FRAME": str(TITLE_MENU_LIVE_DUMP_FRAME),
            "ZELDA3_LIVE_WINDOW_PIXEL_DUMP": str(live_dump),
        }
    )
    command = [
        str(args.rust_bin),
        "--frontend-smoke",
        str(TITLE_MENU_LIVE_FRAMES),
        "--no-frame-pacing",
        "--rom",
        str(args.rom),
        "--replay-save",
        str(args.save),
        "--replay-start-frame",
        str(TITLE_MENU_LIVE_REPLAY_START),
    ]
    result = run_capture(command, cwd=ROOT, env=live_env)
    if result.returncode != 0:
        return result.returncode
    expected_summary = (
        f"frontend smoke completed frames={TITLE_MENU_LIVE_FRAMES} renderer=gpu_render"
    )
    if expected_summary not in result.stdout:
        print("title menu live-present gate missed its completion summary", file=sys.stderr)
        return 1
    width, height, pixels = read_png_rgba(live_dump)
    failure = menu_vertical_coverage_failure("live GPU", width, height, pixels)
    if failure is not None:
        print(failure, file=sys.stderr)
        return 1
    print(
        f"title menu live-present ok replay_start={TITLE_MENU_LIVE_REPLAY_START} "
        f"frames={TITLE_MENU_LIVE_FRAMES} dump_frame={TITLE_MENU_LIVE_DUMP_FRAME}"
    )
    return 0


def run_inventory_menu_live_present_gate(
    args: argparse.Namespace, *, env: dict[str, str]
) -> int:
    out_dir = ROOT / "target" / "standard-replay-render" / "menu"
    out_dir.mkdir(parents=True, exist_ok=True)
    input_script = out_dir / "frontend-open-menu-input.txt"
    input_script.write_text("0..1 0x0008\n")
    live_dump = out_dir / "live-gpu-inventory.png"
    live_env = env.copy()
    live_env.update(
        {
            # This is the independent built-in PPU renderer versus the modern
            # live GPU presenter. It catches shared C/R diagnostic-render bugs.
            "ZELDA3_LIVE_WINDOW_PIXEL_PARITY": "1",
            "ZELDA3_LIVE_WINDOW_PIXEL_DUMP_FRAME": str(MENU_LIVE_DUMP_FRAME),
            "ZELDA3_LIVE_WINDOW_PIXEL_DUMP": str(live_dump),
        }
    )
    command = [
        str(args.rust_bin),
        "--frontend-smoke",
        str(MENU_LIVE_FRAMES),
        "--no-frame-pacing",
        "--rom",
        str(args.rom),
        "--replay-save",
        str(args.save),
        "--replay-start-frame",
        str(MENU_CHECKPOINT_FRAME),
        "--input-script",
        str(input_script),
    ]
    result = run_capture(command, cwd=ROOT, env=live_env)
    if result.returncode != 0:
        print(result.stdout, file=sys.stderr)
        return result.returncode
    width, height, pixels = read_png_rgba(live_dump)
    failure = inventory_menu_panel_geometry_failure("live GPU", width, height, pixels)
    if failure is not None:
        print(failure, file=sys.stderr)
        return 1
    print(
        f"inventory live-present ok replay_start={MENU_CHECKPOINT_FRAME} "
        f"frames={MENU_LIVE_FRAMES} dump_frame={MENU_LIVE_DUMP_FRAME}"
    )
    return 0


def run_menu_sfx_lockstep_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    checkpoint = args.checkpoint_dir / f"rust-frame-{MENU_CHECKPOINT_FRAME}.sav"
    input_script = ROOT / "target" / "standard-replay-render" / "menu" / "open-menu-input.txt"
    command = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(MENU_AUDIO_LOCKSTEP_END_FRAME),
        "--load-state",
        str(checkpoint),
        "--stop-replay-after-load",
        "--input-script",
        str(input_script),
        "--audio-trace-log",
        "1",
    ]
    result = run_capture(command, cwd=ROOT, env=env)
    if result.returncode != 0:
        print(result.stdout, file=sys.stderr)
        return result.returncode
    trace = ROOT / "target" / "parity" / "menu-sfx-lockstep.jsonl"
    trace.parent.mkdir(parents=True, exist_ok=True)
    trace.write_text(result.stdout)
    check = run_capture(
        [
            sys.executable,
            str(ROOT / "scripts" / "check_modern_audio_route.py"),
            "--require-zero-unknown-sfx",
            "--require-sfx-lockstep",
            str(trace),
        ],
        cwd=ROOT,
        env=env,
    )
    if check.returncode != 0:
        print(check.stdout, file=sys.stderr)
        return check.returncode
    print(check.stdout.strip())
    return 0


def run_display_frame_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    c_checkpoint = args.checkpoint_dir / f"c-frame-{DISPLAY_CHECKPOINT_FRAME}.sav"
    rust_checkpoint = args.checkpoint_dir / f"rust-frame-{DISPLAY_CHECKPOINT_FRAME}.sav"
    missing = [path for path in [c_checkpoint, rust_checkpoint] if not path.exists()]
    if missing:
        print("missing required display checkpoint artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    out_dir = ROOT / "target" / "standard-replay-render"
    out_dir.mkdir(parents=True, exist_ok=True)
    c_ppm = out_dir / f"c-{DISPLAY_FRAME}.ppm"
    rust_png = out_dir / f"rust-{DISPLAY_FRAME}.png"

    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    c_cmd = [
        str(args.c_root / "zelda3"),
        "--config",
        str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save",
        str(args.save),
        "--load-state",
        str(c_checkpoint),
        "--smv-test-frames",
        str(DISPLAY_FRAME),
        "--dump-frame",
        str(c_ppm),
    ]
    rust_cmd = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(DISPLAY_FRAME),
        "--load-state",
        str(rust_checkpoint),
        "--dump-frame",
        str(rust_png),
    ]
    c_rc, rust_rc = run_pair((c_cmd, args.c_root, c_env), (rust_cmd, ROOT, env))
    if c_rc != 0:
        return c_rc
    if rust_rc != 0:
        return rust_rc

    diff_count, bbox = compare_c_r_render(c_ppm, rust_png)
    if diff_count:
        print(f"render DIFF frame={DISPLAY_FRAME} pixels={diff_count} bbox={bbox}", file=sys.stderr)
        print(f"  C: {c_ppm}", file=sys.stderr)
        print(f"  R: {rust_png}", file=sys.stderr)
        return 1
    print(f"render ok frame={DISPLAY_FRAME} pixels=0")
    return 0


def run_menu_display_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    c_checkpoint = args.checkpoint_dir / f"c-frame-{MENU_CHECKPOINT_FRAME}.sav"
    rust_checkpoint = args.checkpoint_dir / f"rust-frame-{MENU_CHECKPOINT_FRAME}.sav"
    missing = [path for path in [c_checkpoint, rust_checkpoint] if not path.exists()]
    if missing:
        print("missing required menu display checkpoint artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    out_dir = ROOT / "target" / "standard-replay-render" / "menu"
    out_dir.mkdir(parents=True, exist_ok=True)
    input_script = out_dir / "open-menu-input.txt"
    input_script.write_text(f"{MENU_CHECKPOINT_FRAME}..{MENU_CHECKPOINT_FRAME + 1} 0x0008\n")
    c_ppm = out_dir / f"c-{MENU_DISPLAY_FRAME}.ppm"
    rust_png = out_dir / f"rust-{MENU_DISPLAY_FRAME}.png"

    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    c_cmd = [
        str(args.c_root / "zelda3"),
        "--config",
        str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save",
        str(args.save),
        "--load-state",
        str(c_checkpoint),
        "--stop-replay-after-load",
        "--input-script",
        str(input_script),
        "--smv-test-frames",
        str(MENU_DISPLAY_FRAME),
        "--dump-frame",
        str(c_ppm),
    ]
    rust_cmd = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(MENU_DISPLAY_FRAME),
        "--load-state",
        str(rust_checkpoint),
        "--stop-replay-after-load",
        "--input-script",
        str(input_script),
        "--dump-frame",
        str(rust_png),
    ]
    c_result, rust_result = run_capture_pair((c_cmd, args.c_root, c_env), (rust_cmd, ROOT, env))
    if c_result.returncode != 0:
        return c_result.returncode
    if rust_result.returncode != 0:
        return rust_result.returncode

    menu_state_failures = inventory_menu_state_failures(c_result.stdout, "C")
    menu_state_failures.extend(inventory_menu_state_failures(rust_result.stdout, "Rust"))
    if menu_state_failures:
        print("menu route did not reach the fully-open equipment panel:", file=sys.stderr)
        for failure in menu_state_failures:
            print(f"  {failure}", file=sys.stderr)
        return 1

    state_diffs = compare_common_state_fields(
        c_result.stdout,
        rust_result.stdout,
        [field for field in STANDARD_FIELDS.split(",") if field],
    )
    if state_diffs:
        print("menu state DIFF:", file=sys.stderr)
        for field, c_value, rust_value in state_diffs[:20]:
            print(f"  {field}: C={c_value} Rust={rust_value}", file=sys.stderr)
        if len(state_diffs) > 20:
            print(f"  ... {len(state_diffs) - 20} more", file=sys.stderr)
        return 1

    diff_count, bbox = compare_c_r_render(c_ppm, rust_png)
    if diff_count:
        print(f"menu render DIFF frame={MENU_DISPLAY_FRAME} pixels={diff_count} bbox={bbox}", file=sys.stderr)
        print(f"  C: {c_ppm}", file=sys.stderr)
        print(f"  R: {rust_png}", file=sys.stderr)
        return 1
    print(f"menu render ok frame={MENU_DISPLAY_FRAME} pixels=0")
    return 0


def run_name_entry_release_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    c_checkpoint = args.checkpoint_dir / f"c-frame-{NAME_ENTRY_CHECKPOINT_FRAME}.sav"
    rust_checkpoint = args.checkpoint_dir / f"rust-frame-{NAME_ENTRY_CHECKPOINT_FRAME}.sav"
    missing = [path for path in [c_checkpoint, rust_checkpoint] if not path.exists()]
    if missing:
        print("missing required name-entry checkpoint artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    out_dir = ROOT / "target" / "standard-replay-render" / "name-entry"
    out_dir.mkdir(parents=True, exist_ok=True)
    input_script = out_dir / "release-direction-input.txt"
    input_script.write_text(f"{NAME_ENTRY_CHECKPOINT_FRAME} 0x0020\n")
    c_ppm = out_dir / f"c-{NAME_ENTRY_RELEASE_FRAME}.ppm"
    rust_png = out_dir / f"rust-{NAME_ENTRY_RELEASE_FRAME}.png"

    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    c_cmd = [
        str(args.c_root / "zelda3"),
        "--config",
        str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save",
        str(args.save),
        "--load-state",
        str(c_checkpoint),
        "--stop-replay-after-load",
        "--input-script",
        str(input_script),
        "--smv-test-frames",
        str(NAME_ENTRY_RELEASE_FRAME),
        "--render-hash-log",
        "1",
        "--dump-frame",
        str(c_ppm),
    ]
    rust_cmd = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(NAME_ENTRY_RELEASE_FRAME),
        "--load-state",
        str(rust_checkpoint),
        "--stop-replay-after-load",
        "--input-script",
        str(input_script),
        "--render-hash-log",
        "1",
        "--dump-frame",
        str(rust_png),
    ]
    c_result, rust_result = run_capture_pair((c_cmd, args.c_root, c_env), (rust_cmd, ROOT, env))
    if c_result.returncode != 0:
        return c_result.returncode
    if rust_result.returncode != 0:
        return rust_result.returncode

    state_diffs = compare_common_state_fields(
        c_result.stdout,
        rust_result.stdout,
        [*STANDARD_FIELDS.split(","), "joyh", "joyl", "fh", "fl", "sel3", "sel4", "sel5", "sel9", "sel11"],
    )
    if state_diffs:
        print("name-entry release state DIFF:", file=sys.stderr)
        for field, c_value, rust_value in state_diffs[:20]:
            print(f"  {field}: C={c_value} Rust={rust_value}", file=sys.stderr)
        if len(state_diffs) > 20:
            print(f"  ... {len(state_diffs) - 20} more", file=sys.stderr)
        return 1
    rust_state = parse_replay_state(rust_result.stdout)
    expected_settled = {"sel5": "1", "sel11": "0"}
    unsettled = [
        (field, expected, rust_state.get(field, "<missing>"))
        for field, expected in expected_settled.items()
        if rust_state.get(field) != expected
    ]
    if unsettled:
        print("name-entry release did not settle:", file=sys.stderr)
        for field, expected, actual in unsettled:
            print(f"  {field}: expected={expected} actual={actual}", file=sys.stderr)
        return 1

    c_hashes = list(RENDER_HASH_RE.findall(c_result.stdout))
    rust_hashes = list(RENDER_HASH_RE.findall(rust_result.stdout))
    expected_hashes = NAME_ENTRY_RELEASE_FRAME - NAME_ENTRY_CHECKPOINT_FRAME
    if len(c_hashes) != expected_hashes or len(rust_hashes) != expected_hashes:
        print(
            f"name-entry release render-hash count DIFF expected={expected_hashes} C={len(c_hashes)} Rust={len(rust_hashes)}",
            file=sys.stderr,
        )
        return 1
    for (c_frame, c_hash), (rust_frame, rust_hash) in zip(c_hashes, rust_hashes):
        if c_frame != rust_frame or c_hash.lower() != rust_hash.lower():
            print(
                f"name-entry release render-hash DIFF frame C={c_frame} Rust={rust_frame} C={c_hash} Rust={rust_hash}",
                file=sys.stderr,
            )
            return 1

    diff_count, bbox = compare_c_r_render(c_ppm, rust_png)
    if diff_count:
        print(
            f"name-entry release render DIFF frame={NAME_ENTRY_RELEASE_FRAME} pixels={diff_count} bbox={bbox}",
            file=sys.stderr,
        )
        print(f"  C: {c_ppm}", file=sys.stderr)
        print(f"  R: {rust_png}", file=sys.stderr)
        return 1
    print(
        f"name-entry release ok frames={expected_hashes} start={NAME_ENTRY_CHECKPOINT_FRAME + 1} end={NAME_ENTRY_RELEASE_FRAME} pixels=0"
    )
    return 0


def run_menu_render_hash_gate(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    c_checkpoint = args.checkpoint_dir / f"c-frame-{MENU_CHECKPOINT_FRAME}.sav"
    rust_checkpoint = args.checkpoint_dir / f"rust-frame-{MENU_CHECKPOINT_FRAME}.sav"
    missing = [path for path in [c_checkpoint, rust_checkpoint] if not path.exists()]
    if missing:
        print("missing required menu render-hash checkpoint artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    out_dir = ROOT / "target" / "standard-replay-render" / "menu"
    out_dir.mkdir(parents=True, exist_ok=True)
    input_script = out_dir / "open-menu-input.txt"
    input_script.write_text(f"{MENU_CHECKPOINT_FRAME}..{MENU_CHECKPOINT_FRAME + 1} 0x0008\n")

    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    for frame in MENU_RENDER_HASH_FRAMES:
        c_cmd = [
            str(args.c_root / "zelda3"),
            "--config",
            str(args.c_root / "other" / "headless_replay.ini"),
            "--replay-save",
            str(args.save),
            "--load-state",
            str(c_checkpoint),
            "--stop-replay-after-load",
            "--input-script",
            str(input_script),
            "--smv-test-frames",
            str(frame),
            "--render-hash-log",
            str(frame),
        ]
        rust_cmd = [
            str(args.rust_bin),
            "--replay-save",
            str(args.rom),
            str(args.save),
            str(frame),
            "--load-state",
            str(rust_checkpoint),
            "--stop-replay-after-load",
            "--input-script",
            str(input_script),
            "--render-hash-log",
            str(frame),
        ]
        c_result, rust_result = run_capture_pair(
            (c_cmd, args.c_root, c_env),
            (rust_cmd, ROOT, env),
        )
        c_hashes = RENDER_HASH_RE.findall(c_result.stdout)
        rust_hashes = RENDER_HASH_RE.findall(rust_result.stdout)
        expected_frame = str(frame)
        if c_result.returncode != 0 or rust_result.returncode != 0:
            print(
                f"menu render-hash command failed frame={frame} C={c_result.returncode} Rust={rust_result.returncode}",
                file=sys.stderr,
            )
            return 1
        if len(c_hashes) != 1 or len(rust_hashes) != 1:
            print(
                f"menu render-hash expected one isolated sample frame={frame} C={c_hashes} Rust={rust_hashes}",
                file=sys.stderr,
            )
            return 1
        c_frame, c_value = c_hashes[0]
        rust_frame, rust_value = rust_hashes[0]
        if c_frame != expected_frame or rust_frame != expected_frame or c_value != rust_value:
            print(
                f"menu render-hash DIFF frame C={c_frame} Rust={rust_frame} C={c_value} Rust={rust_value}",
                file=sys.stderr,
            )
            return 1

    print(
        "menu render-hash ok isolated_frames="
        + ",".join(str(frame) for frame in MENU_RENDER_HASH_FRAMES)
    )
    return 0


def start_render_hash_processes(
    args: argparse.Namespace,
    *,
    env: dict[str, str],
    end_frame: int | None = None,
    c_dump: pathlib.Path | None = None,
    rust_dump: pathlib.Path | None = None,
    log_hashes: bool = True,
    pipe_output: bool = True,
) -> tuple[subprocess.Popen, subprocess.Popen]:
    c_env = env.copy()
    c_env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    c_cmd = [
        str(args.c_root / "zelda3"),
        "--config",
        str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save",
        str(args.save),
    ]
    if args.render_hash_start_frame is not None:
        c_cmd.extend(["--load-state", str(args.render_hash_c_checkpoint)])
    c_cmd.extend(
        [
        "--smv-test-frames",
            str(args.render_hash_end_frame if end_frame is None else end_frame),
        ]
    )
    if log_hashes:
        c_cmd.extend(["--render-hash-log", str(args.render_hash_stride)])
    if c_dump is not None:
        c_cmd.extend(["--render-hash-dump-frame", str(end_frame), str(c_dump)])
    rust_cmd = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(args.render_hash_end_frame if end_frame is None else end_frame),
    ]
    if args.render_hash_start_frame is not None:
        rust_cmd.extend(["--load-state", str(args.render_hash_rust_checkpoint)])
    if log_hashes:
        rust_cmd.extend(["--render-hash-log", str(args.render_hash_stride)])
    if rust_dump is not None:
        rust_cmd.extend(["--render-hash-dump-frame", str(end_frame), str(rust_dump)])
    print("+ " + " ".join(c_cmd), flush=True)
    c_proc = subprocess.Popen(
        c_cmd,
        cwd=args.c_root,
        env=c_env,
        text=True,
        stdout=subprocess.PIPE if pipe_output else subprocess.DEVNULL,
        stderr=subprocess.STDOUT if pipe_output else subprocess.DEVNULL,
        bufsize=1,
    )
    print("+ " + " ".join(rust_cmd), flush=True)
    rust_proc = subprocess.Popen(
        rust_cmd,
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE if pipe_output else subprocess.DEVNULL,
        stderr=subprocess.STDOUT if pipe_output else subprocess.DEVNULL,
        bufsize=1,
    )
    return c_proc, rust_proc


def dump_continuous_render_hash_frame(args: argparse.Namespace, frame: int, *, env: dict[str, str], label: str) -> int:
    out_dir = ROOT / "target" / "standard-replay-render" / label
    out_dir.mkdir(parents=True, exist_ok=True)
    c_ppm = out_dir / f"c-{frame}.ppm"
    rust_png = out_dir / f"rust-{frame}.png"
    c_proc, rust_proc = start_render_hash_processes(
        args,
        env=env,
        end_frame=frame,
        c_dump=c_ppm,
        rust_dump=rust_png,
        log_hashes=False,
        pipe_output=False,
    )
    try:
        c_rc = c_proc.wait()
        r_rc = rust_proc.wait()
        if c_rc != 0 or r_rc != 0:
            print(f"render-hash diff dump failed c_rc={c_rc} r_rc={r_rc}", file=sys.stderr)
            return 1
        diff_count, bbox = compare_c_r_render(c_ppm, rust_png)
        print(f"render continuous DIFF frame={frame} pixels={diff_count} bbox={bbox}", file=sys.stderr)
        print(f"  C: {c_ppm}", file=sys.stderr)
        print(f"  R: {rust_png}", file=sys.stderr)
        return 0
    finally:
        stop_process(c_proc)
        stop_process(rust_proc)


def next_render_hash(proc: subprocess.Popen, name: str) -> tuple[int, str] | None:
    assert proc.stdout is not None
    while True:
        line = proc.stdout.readline()
        if line == "":
            return None
        line = line.rstrip("\n")
        match = RENDER_HASH_RE.match(line)
        if match:
            return int(match.group(1)), match.group(2).lower()
        if line and not line.startswith("***"):
            print(f"{name}: {line}", flush=True)


def stop_process(proc: subprocess.Popen) -> int:
    if proc.poll() is None:
        proc.terminate()
        try:
            return proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            return proc.wait()
    return proc.wait()


def bind_render_hash_checkpoints(args: argparse.Namespace) -> int:
    args.render_hash_c_checkpoint = None
    args.render_hash_rust_checkpoint = None
    if args.render_hash_start_frame is None:
        return 0

    args.render_hash_c_checkpoint = args.checkpoint_dir / f"c-frame-{args.render_hash_start_frame}.sav"
    args.render_hash_rust_checkpoint = args.checkpoint_dir / f"rust-frame-{args.render_hash_start_frame}.sav"
    missing_checkpoints = [
        str(path)
        for path in [args.render_hash_c_checkpoint, args.render_hash_rust_checkpoint]
        if not path.exists()
    ]
    if missing_checkpoints:
        print("missing required render-hash checkpoint artifact(s):", file=sys.stderr)
        for path in missing_checkpoints:
            print(f"  {path}", file=sys.stderr)
        print(
            "create them with scripts/replay_bisect.py --checkpoint-frame "
            f"{args.render_hash_start_frame}",
            file=sys.stderr,
        )
        return 1
    return 0


def run_standard_regression_gate(
    args: argparse.Namespace,
    *,
    c_root: pathlib.Path,
    rom: pathlib.Path,
    save: pathlib.Path,
    rust_bin: pathlib.Path,
    env: dict[str, str],
) -> int:
    early_frames = [
        frame
        for frame in STANDARD_REGRESSION_FRAMES
        if frame <= args.late_regression_checkpoint_frame
    ]
    late_frames = [
        frame
        for frame in STANDARD_REGRESSION_FRAMES
        if frame > args.late_regression_checkpoint_frame
    ]
    if not early_frames:
        print("standard replay parity needs at least one early regression frame", file=sys.stderr)
        return 2
    if not late_frames:
        print("standard replay parity needs at least one late regression frame", file=sys.stderr)
        return 2

    regression_base = [
        sys.executable,
        "scripts/replay_bisect.py",
        "--regression",
        "--fields",
        STANDARD_FIELDS,
        "--sweep-fail-on-diff",
        "--verbose",
        "--isolate-runtime-cwd",
        "--c-root",
        str(c_root),
        "--rom",
        str(rom),
        "--save",
        str(save),
        "--rust-bin",
        str(rust_bin),
    ]
    early_regression = [
        *regression_base,
        "--regression-frames",
        ",".join(str(frame) for frame in early_frames),
        "--regression-workers",
        str(args.regression_workers),
    ]
    rc = run(early_regression, env=env)
    if rc != 0:
        return rc

    c_checkpoint, rust_checkpoint = replay_checkpoint_pair(
        args, args.late_regression_checkpoint_frame
    )
    late_regression = [
        *regression_base,
        "--regression-frames",
        ",".join(str(frame) for frame in late_frames),
        "--regression-workers",
        str(min(args.regression_workers, len(late_frames))),
        "--checkpoint-frame",
        str(args.late_regression_checkpoint_frame),
        "--c-checkpoint",
        str(c_checkpoint),
        "--rust-checkpoint",
        str(rust_checkpoint),
    ]
    return run(late_regression, env=env)


def run_render_hash_all(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    c_proc, rust_proc = start_render_hash_processes(args, env=env)
    compared = 0
    try:
        while True:
            c_hash = next_render_hash(c_proc, "C")
            r_hash = next_render_hash(rust_proc, "Rust")
            if c_hash is None or r_hash is None:
                c_rc = c_proc.wait()
                r_rc = rust_proc.wait()
                if c_hash != r_hash or c_rc != 0 or r_rc != 0:
                    print(
                        f"render-hash stream ended unexpectedly compared={compared} c={c_hash} r={r_hash} c_rc={c_rc} r_rc={r_rc}",
                        file=sys.stderr,
                    )
                    return 1
                print(
                    f"render-hash-all ok frames={compared} start={args.render_hash_start_frame} end={args.render_hash_end_frame} stride={args.render_hash_stride}"
                )
                return 0
            compared += 1
            if c_hash != r_hash:
                c_frame, c_value = c_hash
                r_frame, r_value = r_hash
                print(
                    f"render-hash DIFF frame C={c_frame} Rust={r_frame} C={c_value} Rust={r_value}",
                    file=sys.stderr,
                )
                stop_process(c_proc)
                stop_process(rust_proc)
                if c_frame == r_frame:
                    dump_continuous_render_hash_frame(
                        args,
                        c_frame,
                        env=env,
                        label="render-hash-first-diff",
                    )
                return 1
            if args.render_hash_progress and compared % args.render_hash_progress == 0:
                print(f"render-hash progress compared={compared} frame={c_hash[0]}", flush=True)
    finally:
        stop_process(c_proc)
        stop_process(rust_proc)


def run_render_hash_chunked(args: argparse.Namespace, *, env: dict[str, str]) -> int:
    chunk_frames = [frame for frame in RENDER_HASH_CHUNK_FRAMES if frame < args.render_hash_end_frame]
    chunk_ends = [*chunk_frames, args.render_hash_end_frame]
    chunk_start = None
    for chunk_end in chunk_ends:
        if chunk_start is not None and chunk_start >= chunk_end:
            continue
        chunk_args = copy.copy(args)
        chunk_args.render_hash_start_frame = chunk_start
        chunk_args.render_hash_end_frame = chunk_end
        rc = bind_render_hash_checkpoints(chunk_args)
        if rc != 0:
            return rc
        print(f"render-hash chunk start={chunk_start} end={chunk_end}", flush=True)
        rc = run_render_hash_all(chunk_args, env=env)
        if rc != 0:
            return rc
        chunk_start = chunk_end
    print(f"render-hash-chunked ok start=0 end={args.render_hash_end_frame} stride={args.render_hash_stride}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--c-root", type=pathlib.Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--rom", type=pathlib.Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=pathlib.Path, default=DEFAULT_SAVE)
    parser.add_argument("--rust-bin", type=pathlib.Path, default=DEFAULT_RUST_BIN)
    parser.add_argument("--checkpoint-dir", type=pathlib.Path, default=DEFAULT_CHECKPOINT_DIR)
    parser.add_argument(
        "--render-hash-all",
        action="store_true",
        help="compare C/R display-frame RGB hashes for every standard-route frame",
    )
    parser.add_argument(
        "--render-hash-only",
        action="store_true",
        help="only run --render-hash-all; skips state regression and targeted render gates",
    )
    parser.add_argument(
        "--render-hash-chunked",
        action="store_true",
        help="split --render-hash-all at cached standard-route checkpoints to prove the full route faster",
    )
    parser.add_argument(
        "--render-hash-end-frame",
        type=int,
        default=None,
        help="last frame for --render-hash-all; defaults to --final-frame",
    )
    parser.add_argument(
        "--render-hash-start-frame",
        type=int,
        default=None,
        help="load cached C/R checkpoints at this frame before --render-hash-all",
    )
    parser.add_argument(
        "--render-hash-stride",
        type=int,
        default=1,
        help="hash every Nth frame for --render-hash-all",
    )
    parser.add_argument(
        "--render-hash-progress",
        type=int,
        default=10_000,
        help="print progress every N compared hashes; 0 disables progress",
    )
    parser.add_argument(
        "--final-frame",
        type=int,
        default=1_073_092,
        help="full reset-start final replay frame to check after regression windows",
    )
    parser.add_argument(
        "--skip-final-frame",
        action="store_true",
        help="only run the multi-window regression set",
    )
    parser.add_argument(
        "--skip-standalone-embedded",
        action="store_true",
        help="skip the no-ROM embedded-assets startup smoke gate",
    )
    parser.add_argument(
        "--standalone-embedded-only",
        action="store_true",
        help="only run the no-ROM embedded-assets startup smoke gate",
    )
    parser.add_argument(
        "--regression-workers",
        type=int,
        default=3,
        help="parallel workers for standard regression frame checks",
    )
    parser.add_argument(
        "--late-regression-checkpoint-frame",
        type=int,
        default=STANDARD_LATE_REGRESSION_CHECKPOINT_FRAME,
        help=(
            "checkpoint frame used to resume late standard regression frames; "
            "cached C/R checkpoints are created under --checkpoint-dir when missing"
        ),
    )
    args = parser.parse_args()
    if args.render_hash_stride <= 0:
        parser.error("--render-hash-stride must be positive")
    if args.render_hash_end_frame is None:
        args.render_hash_end_frame = args.final_frame
    if args.render_hash_start_frame is not None and args.render_hash_start_frame > args.render_hash_end_frame:
        parser.error("--render-hash-start-frame must be <= --render-hash-end-frame")
    if args.regression_workers <= 0:
        parser.error("--regression-workers must be positive")
    if args.late_regression_checkpoint_frame <= 0:
        parser.error("--late-regression-checkpoint-frame must be positive")
    if args.render_hash_chunked and args.render_hash_start_frame is not None:
        parser.error("--render-hash-chunked cannot be combined with --render-hash-start-frame")
    if args.render_hash_only:
        args.render_hash_all = True
    if args.render_hash_chunked:
        args.render_hash_all = True

    c_root = args.c_root.expanduser().resolve()
    args.c_root = c_root
    rom = args.rom.expanduser().resolve()
    save = args.save.expanduser().resolve()
    rust_bin = args.rust_bin.expanduser().resolve()
    args.rust_bin = rust_bin
    args.checkpoint_dir = args.checkpoint_dir.expanduser().resolve()
    env = os.environ.copy()
    current_asset_pack = ROOT / "zelda3_assets.dat"
    if current_asset_pack.is_file():
        # The C checkout may carry an older pack beside its oracle ROM. Make
        # the modern side's assets explicit so a stale neighboring file cannot
        # prevent the parity gates from exercising the current renderer.
        env.setdefault("ZELDA3_ASSET_PACK", str(current_asset_pack))
    if args.standalone_embedded_only:
        if not rust_bin.exists():
            print(f"missing required replay parity artifact(s):\n  {rust_bin}", file=sys.stderr)
            return 2
        return run_standalone_embedded_gate(args, env=env)

    rc = bind_render_hash_checkpoints(args)
    if rc != 0:
        return rc

    missing = [
        str(path)
        for path in [c_root / "zelda3", c_root / "other" / "headless_replay.ini", rom, save, rust_bin]
        if not path.exists()
    ]
    if missing:
        print("missing required replay parity artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    rc = run_c_oracle_gate(args)
    if rc != 0:
        return rc

    if args.render_hash_only:
        if args.render_hash_chunked:
            return run_render_hash_chunked(args, env=env)
        return run_render_hash_all(args, env=env)

    if not args.skip_standalone_embedded:
        rc = run_standalone_embedded_gate(args, env=env)
        if rc != 0:
            return rc

    rc = run_pacing_gate(args)
    if rc != 0:
        return rc

    rc = run_standard_regression_gate(
        args,
        c_root=c_root,
        rom=rom,
        save=save,
        rust_bin=rust_bin,
        env=env,
    )
    if rc != 0:
        return rc

    rc = run_title_palette_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_title_menu_live_present_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_display_frame_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_name_entry_release_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_menu_display_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_inventory_menu_live_present_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_menu_sfx_lockstep_gate(args, env=env)
    if rc != 0:
        return rc

    rc = run_menu_render_hash_gate(args, env=env)
    if rc != 0:
        return rc

    if args.render_hash_all:
        if args.render_hash_chunked:
            rc = run_render_hash_chunked(args, env=env)
        else:
            rc = run_render_hash_all(args, env=env)
        if rc != 0:
            return rc

    if args.skip_final_frame:
        return 0

    final = [
        sys.executable,
        "scripts/replay_bisect.py",
        "--check-frame",
        str(args.final_frame),
        "--fields",
        STANDARD_FIELDS,
        "--compare-dumps",
        "--ram-dump-page",
        "0x0400",
        "--no-save-local-checkpoint",
        "--verbose",
        "--c-root",
        str(c_root),
        "--rom",
        str(rom),
        "--save",
        str(save),
        "--rust-bin",
        str(rust_bin),
    ]
    return run(final, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
