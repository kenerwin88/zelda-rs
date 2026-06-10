#!/usr/bin/env python3
import argparse
import concurrent.futures
import os
import pathlib
import re
import subprocess
import sys
import tempfile
from contextlib import contextmanager
from collections import defaultdict


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = ROOT.parent / "zelda3"
FOCUSED_FIELDS = [
    "ending", "fc", "rng", "ramhash",
    "ram0", "ram1", "ram2", "ram3", "ram4", "ram5", "ram6", "ram7",
    "sramhash", "roommask", "hist", "histmask",
    "main", "sub", "subsub", "saved", "indoors", "room", "ow",
    "msg", "msgpos", "textrs",
    "x", "y", "subpix", "hp", "item", "big", "sram3",
    "joyh", "joyl", "fh", "fl", "dir", "face", "state", "inwater", "aux",
    "incap", "recoil", "drag", "grab", "vx", "vy", "speed", "lspeed", "dash", "cdf",
    "z", "vz", "vzcopy",
    "below", "tile", "action", "interact", "col", "bugs", "feat", "wanted",
    "ptimer", "bframes",
    "r14", "r12", "misc", "pit", "spike", "vledge", "stair",
    "deep", "normal",
]
DEFAULT_FIELDS = "all"
DEFAULT_DUMP_PREFIXES = [
    "ancilla",
    "sprites",
    "ram-pages",
    "ram-page-bytes",
    "ram0000",
    "ram0400",
    "rng ",
    "garnish",
    "room-masks",
    "room-history",
    "overlords",
    "doors",
    "dungmap",
    "message",
    "palette",
    "dungeon-attr-state",
    "dungeon-attrs",
    "file-select-stall",
]
DEFAULT_CHECKPOINT_DIR = ROOT / ".cache" / "replay-bisect"
RUNTIME_CWD_FILES = ["zelda3.ini", "zelda3_assets.dat", "zelda3_assets.bps", "zelda3.sfc"]
DEFAULT_REGRESSION_FRAMES = [
    1000,
    12000,
    42998,
    80000,
    112078,
    180000,
    202254,
    202255,
    350000,
    700000,
    1073092,
]
REPLAY_TIMING_HACK_ENV = [
    "ZELDA3_SMV_SELECT_FILE_TIMING_HACKS",
    "ZELDA3_SMV_LOADFILE_TIMING_HACKS",
    "ZELDA3_SMV_DUNGEON_TIMING_HACKS",
    "ZELDA3_SMV_OVERWORLD_TIMING_HACKS",
    "ZELDA3_SMV_MESSAGING_TIMING_HACKS",
    "ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS",
    "ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS",
]
DUMP_ENVS = {
    "ancilla": "ZELDA3_REPLAY_ANCILLA_DUMP",
    "doors": "ZELDA3_REPLAY_DOOR_DUMP",
    "dungeon-attrs": "ZELDA3_REPLAY_DUNGEON_ATTR_DUMP",
    "dungeon-attr-state": "ZELDA3_REPLAY_DUNGEON_ATTR_STATE_DUMP",
    "dungmap": "ZELDA3_REPLAY_DUNGMAP_DUMP",
    "file-select-stall": "ZELDA3_REPLAY_FILE_SELECT_STALL_DUMP",
    "garnish": "ZELDA3_REPLAY_GARNISH_DUMP",
    "message": "ZELDA3_REPLAY_MESSAGE_DUMP",
    "overlords": "ZELDA3_REPLAY_OVERLORD_DUMP",
    "palette": "ZELDA3_REPLAY_PALETTE_DUMP",
    "ram-pages": "ZELDA3_REPLAY_RAM_PAGE_DUMP",
    "ram0000": "ZELDA3_REPLAY_RAM0000_DUMP",
    "ram0400": "ZELDA3_REPLAY_RAM0400_DUMP",
    "rng": "ZELDA3_REPLAY_RNG_DUMP",
    "room-history": "ZELDA3_REPLAY_ROOM_HISTORY_DUMP",
    "room-masks": "ZELDA3_REPLAY_ROOM_MASK_DUMP",
    "sprites": "ZELDA3_REPLAY_SPRITE_DUMP",
}
CLASS_DUMPS = {
    "gameplay": ["rng", "sprites", "room-masks", "room-history", "doors", "dungmap", "message"],
    "palette-or-buffer": ["palette", "ram-pages", "ram0000", "dungmap", "message"],
    "low-ram-scratch-or-state": ["ram0000", "dungmap", "message", "rng"],
    "save-or-high-ram": ["room-masks", "room-history", "doors", "ram-pages"],
    "subsystem-dump": ["sprites", "room-masks", "overlords", "doors", "dungmap", "message", "palette"],
    "hash-only": ["ram-pages", "rng", "sprites", "room-masks", "doors", "dungmap", "message", "palette"],
    "process": ["ram-pages", "rng", "sprites", "room-masks", "doors", "dungmap", "message", "palette"],
}
FIELD_SYMBOLS = {
    "rng": "byte_7E0FA1",
    "roommask": "sprite_where_in_room[current_room]",
    "histmask": "sprite_where_in_room[room_history]",
    "hp": "link_health_current",
    "state": "link_player_handler_state",
    "speed": "link_speed_setting/link_speed_modifier",
    "lspeed": "link_actual_vel_x/link_actual_vel_y",
    "x": "link_x_coord",
    "y": "link_y_coord",
    "msg": "dialogue_message_index",
    "msgpos": "dialogue_msg_read_pos",
    "textrs": "text_render_state",
    "room": "dungeon_room_index",
    "ow": "overworld_screen_index",
    "main": "main_module_index",
    "sub": "submodule_index",
    "subsub": "subsubmodule_index",
}
ADDRESS_SYMBOLS = [
    (0x0010, 1, "main_module_index"),
    (0x0011, 1, "submodule_index"),
    (0x001a, 1, "frame_counter"),
    (0x0020, 2, "link_y_coord"),
    (0x0022, 2, "link_x_coord"),
    (0x0027, 1, "link_actual_vel_y"),
    (0x0028, 1, "link_actual_vel_x"),
    (0x0057, 1, "link_speed_modifier"),
    (0x005d, 1, "link_player_handler_state"),
    (0x005e, 1, "link_speed_setting"),
    (0x00a0, 2, "dungeon_room_index"),
    (0x0200, 1, "overworld_map_state"),
    (0x020d, 1, "dungmap_init_state"),
    (0x020e, 2, "dungmap_cur_floor"),
    (0x0210, 1, "dungmap_var2"),
    (0x0211, 2, "dungmap_idx"),
    (0x0213, 2, "dungmap_var4"),
    (0x0215, 2, "dungmap_var3"),
    (0x0217, 2, "dungmap_var5"),
    (0x0400, 2, "dung_door_opened"),
    (0x040c, 2, "cur_palace_index_x2"),
    (0x048e, 2, "dungeon_room_index2"),
    (0x0cf5, 2, "dungmap_var6"),
    (0x0dd0, 16, "sprite_state[]"),
    (0x0e20, 16, "sprite_type[]"),
    (0x0fa1, 1, "byte_7E0FA1"),
    (0x0fa8, 2, "dungmap_var7"),
    (0x0faa, 2, "dungmap_var8"),
    (0x0aa8, 2, "overworld_palette_aux_or_main"),
    (0x0ab1, 1, "palette_sp6r_indoors"),
    (0x0ab2, 1, "hud_palette"),
    (0x0ab6, 1, "palette_main_indoors"),
    (0x0c007, 2, "palette_filter_countdown"),
    (0x0c300, 0x200, "aux_palette_buffer"),
    (0x0c500, 0x200, "main_palette_buffer"),
    (0x11200, 0x800, "messaging_text_buffer"),
    (0x1cd9, 2, "dialogue_msg_read_pos"),
    (0x1cd4, 1, "text_render_state"),
    (0x1cd5, 1, "vwf_line_speed_cur"),
    (0x1cd6, 1, "vwf_line_speed"),
    (0x1cd8, 1, "messaging_module"),
    (0x1ce0, 2, "text_wait_countdown"),
    (0x1ce9, 1, "text_wait_countdown2"),
    (0x1cf0, 2, "dialogue_message_index"),
    (0x1df80, 0x300, "sprite_where_in_room[]"),
]
WRITER_HINTS = {
    "dungmap_var3": ("C DungeonMap_DrawRoomMarkers", "Rust DungeonMap_DrawRoomMarkers"),
    "dungmap_var5": ("C DungeonMap_DrawRoomMarkers/DungMap_4", "Rust DungeonMap_DrawRoomMarkers/DungMap_4"),
    "dungmap": ("C messaging.c dungeon map functions", "Rust messaging.rs dungeon map functions"),
    "link_health_current": ("C Link_ReceiveDamage/health writers", "Rust player/sprite damage writers"),
    "link_player_handler_state": ("C player.c LinkState handlers", "Rust player.rs LinkState handlers"),
    "link_speed_setting/link_speed_modifier": ("C player movement speed setup", "Rust player movement speed setup"),
    "byte_7E0FA1": ("C GetRandomNumber", "Rust get_random_number"),
    "sprite_where_in_room[]": ("C sprite room death mask writers", "Rust sprite_where_in_room_mask/set helpers"),
    "aux_palette_buffer": ("C load_gfx.c/messaging palette writers", "Rust load_gfx.rs/messaging palette writers"),
    "main_palette_buffer": ("C load_gfx.c/messaging palette writers", "Rust load_gfx.rs/messaging palette writers"),
    "messaging_text_buffer": ("C messaging.c text decode/render", "Rust messaging.rs text decode/render"),
    "dung_door_opened": ("C dungeon.c door state writers", "Rust dungeon.rs door state writers"),
}


def canonical_value(value):
    if "/" in value:
        return "/".join(canonical_value(part) for part in value.split("/"))
    if re.fullmatch(r"0x[0-9a-fA-F]+", value):
        return str(int(value, 16))
    if re.fullmatch(r"[0-9]+", value):
        return str(int(value, 10))
    return value


def parse_state(output):
    lines = [line for line in output.splitlines() if line.strip()]
    state_line = next(
        (
            line for line in reversed(lines)
            if line.startswith("smv-test frame=")
            or line.startswith("replay-save completed frames=")
        ),
        "",
    )
    if not state_line:
        raise RuntimeError("no replay checkpoint line found")
    state = {
        key: canonical_value(value)
        for key, value in re.findall(r"\b([a-z0-9_]+)=([^ \n]+)", state_line)
    }
    return state_line, state


def parse_dump_lines(output, prefixes):
    dumps = {}
    for line in output.splitlines():
        for prefix in prefixes:
            if line.startswith(prefix):
                dumps[prefix] = line
                break
    return dumps


def parse_dump_items(line):
    if line is None:
        return None
    items = re.findall(r"\[([^=\]]+)=([^\]]+)\]", line)
    if not items:
        return None
    return dict(items)


def parse_int(value):
    try:
        return int(value, 0)
    except ValueError:
        return None


def address_symbol(address):
    for start, size, name in ADDRESS_SYMBOLS:
        if start <= address < start + size:
            if size == 1:
                return name
            if name.endswith("[]"):
                return f"{name[:-2]}[{address - start}]"
            return name if address == start else f"{name}+0x{address - start:x}"
    return None


def symbolize_diff_field(field):
    if field in FIELD_SYMBOLS:
        return f"{field}({FIELD_SYMBOLS[field]})"
    parts = field.split(":")
    if len(parts) >= 3 and parts[0] == "dump":
        key = parts[2]
        address = None
        if re.fullmatch(r"0x[0-9a-fA-F]+", key):
            address = int(key, 16)
        elif re.fullmatch(r"[0-9a-fA-F]{3,5}", key):
            address = int(key, 16)
        if address is not None:
            symbol = address_symbol(address)
            if symbol:
                return f"{field}({symbol})"
    return field


def symbols_in_diff_field(field):
    labelled = symbolize_diff_field(field)
    match = re.search(r"\(([^()]+)\)$", labelled)
    if match:
        return [match.group(1)]
    if field.startswith("dump:dungmap"):
        return ["dungmap"]
    if field.startswith("dump:message"):
        return ["messaging_text_buffer"]
    if field.startswith("dump:palette"):
        return ["aux_palette_buffer", "main_palette_buffer"]
    if field.startswith("dump:doors"):
        return ["dung_door_opened"]
    return []


@contextmanager
def runtime_cwd(args, root):
    if not args.isolate_runtime_cwd:
        yield root
        return

    with tempfile.TemporaryDirectory(prefix="zelda3-replay-") as path:
        cwd = pathlib.Path(path)
        (cwd / "saves").mkdir()
        for name in RUNTIME_CWD_FILES:
            source = root / name
            if source.exists():
                os.symlink(source, cwd / name)
        yield cwd


def run_c(args, frame, dump_env):
    env = os.environ.copy()
    for name in REPLAY_TIMING_HACK_ENV:
        env.pop(name, None)
    env.update({
        "SDL_VIDEODRIVER": "dummy",
        "SDL_AUDIODRIVER": "dummy",
        "SDL_RENDER_DRIVER": "software",
    })
    env.update(dump_env)
    cmd = [
        str(args.c_root / "zelda3"),
        "--config", str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save", str(args.save),
    ]
    if args.checkpoint_frame is not None and frame >= args.checkpoint_frame:
        cmd.extend(["--load-state", str(args.c_checkpoint)])
    cmd.extend(["--smv-test-frames", str(frame)])
    with runtime_cwd(args, args.c_root) as cwd:
        return subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )


def run_rust(args, frame, dump_env):
    env = os.environ.copy()
    env.update({
        "ZELDA3_SMV_SELECT_FILE_TIMING_HACKS": "1",
        "ZELDA3_SMV_LOADFILE_TIMING_HACKS": "1",
        "ZELDA3_SMV_DUNGEON_TIMING_HACKS": "1",
        "ZELDA3_SMV_OVERWORLD_TIMING_HACKS": "1",
        "ZELDA3_SMV_MESSAGING_TIMING_HACKS": "1",
        "ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS": "1",
        "ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS": "1",
    })
    env.update(dump_env)
    cmd = [
        str(args.rust_bin),
        "--replay-save", str(args.rom), str(args.save), str(frame),
    ]
    if args.checkpoint_frame is not None and frame >= args.checkpoint_frame:
        cmd.extend(["--load-state", str(args.rust_checkpoint)])
    with runtime_cwd(args, ROOT) as cwd:
        return subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )


def run_c_save_checkpoint_at(args, frame, checkpoint):
    env = os.environ.copy()
    for name in REPLAY_TIMING_HACK_ENV:
        env.pop(name, None)
    env.update({
        "SDL_VIDEODRIVER": "dummy",
        "SDL_AUDIODRIVER": "dummy",
        "SDL_RENDER_DRIVER": "software",
    })
    cmd = [
        str(args.c_root / "zelda3"),
        "--config", str(args.c_root / "other" / "headless_replay.ini"),
        "--replay-save", str(args.save),
        "--smv-test-frames", str(frame),
        "--save-state", str(checkpoint),
    ]
    return subprocess.run(
        cmd,
        cwd=args.c_root,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def run_rust_save_checkpoint_at(args, frame, checkpoint):
    env = os.environ.copy()
    env.update({
        "ZELDA3_SMV_SELECT_FILE_TIMING_HACKS": "1",
        "ZELDA3_SMV_LOADFILE_TIMING_HACKS": "1",
        "ZELDA3_SMV_DUNGEON_TIMING_HACKS": "1",
        "ZELDA3_SMV_OVERWORLD_TIMING_HACKS": "1",
        "ZELDA3_SMV_MESSAGING_TIMING_HACKS": "1",
        "ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS": "1",
        "ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS": "1",
    })
    cmd = [
        str(args.rust_bin),
        "--replay-save", str(args.rom), str(args.save), str(frame),
        "--save-state", str(checkpoint),
    ]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def run_c_save_checkpoint(args):
    return run_c_save_checkpoint_at(args, args.checkpoint_frame, args.c_checkpoint)


def run_rust_save_checkpoint(args):
    return run_rust_save_checkpoint_at(args, args.checkpoint_frame, args.rust_checkpoint)


def ensure_checkpoints(args):
    if args.checkpoint_frame is None:
        return
    args.checkpoint_dir.mkdir(parents=True, exist_ok=True)
    if args.refresh_checkpoints:
        for path in [args.c_checkpoint, args.rust_checkpoint]:
            if path.exists():
                path.unlink()
    if not args.c_checkpoint.exists():
        print(f"creating C checkpoint frame={args.checkpoint_frame} path={args.c_checkpoint}", flush=True)
        result = run_c_save_checkpoint(args)
        if result.returncode != 0:
            print(result.stdout, file=sys.stderr)
            raise RuntimeError("failed to create C checkpoint")
    if not args.rust_checkpoint.exists():
        print(f"creating Rust checkpoint frame={args.checkpoint_frame} path={args.rust_checkpoint}", flush=True)
        result = run_rust_save_checkpoint(args)
        if result.returncode != 0:
            print(result.stdout, file=sys.stderr)
            raise RuntimeError("failed to create Rust checkpoint")


def save_local_checkpoints(args, frame):
    args.checkpoint_dir.mkdir(parents=True, exist_ok=True)
    c_path = args.checkpoint_dir / f"c-frame-{frame}.sav"
    rust_path = args.checkpoint_dir / f"rust-frame-{frame}.sav"
    if args.refresh_checkpoints:
        for path in [c_path, rust_path]:
            if path.exists():
                path.unlink()
    if not c_path.exists():
        print(f"creating local C checkpoint frame={frame} path={c_path}", flush=True)
        result = run_c_save_checkpoint_at(args, frame, c_path)
        if result.returncode != 0:
            print(result.stdout, file=sys.stderr)
            raise RuntimeError("failed to create local C checkpoint")
    if not rust_path.exists():
        print(f"creating local Rust checkpoint frame={frame} path={rust_path}", flush=True)
        result = run_rust_save_checkpoint_at(args, frame, rust_path)
        if result.returncode != 0:
            print(result.stdout, file=sys.stderr)
            raise RuntimeError("failed to create local Rust checkpoint")
    print(
        "local-checkpoint "
        f"frame={frame} c={c_path} rust={rust_path} "
        "use="
        f"python3 scripts/replay_bisect.py --check-frame {frame + 1} "
        f"--checkpoint-frame {frame} --c-checkpoint {c_path} --rust-checkpoint {rust_path}",
        flush=True,
    )


def fields_to_compare(c_state, r_state, fields):
    if fields == ["all"]:
        return sorted(set(c_state) & set(r_state))
    return fields


def compare_states(c_state, r_state, fields):
    diffs = []
    for field in fields_to_compare(c_state, r_state, fields):
        c_value = c_state.get(field)
        r_value = r_state.get(field)
        if c_value != r_value:
            diffs.append((field, c_value, r_value))
    return diffs


def compare_dumps(c_output, r_output, prefixes):
    c_dumps = parse_dump_lines(c_output, prefixes)
    r_dumps = parse_dump_lines(r_output, prefixes)
    diffs = []
    for prefix in prefixes:
        c_value = c_dumps.get(prefix)
        r_value = r_dumps.get(prefix)
        if c_value != r_value:
            c_items = parse_dump_items(c_value)
            r_items = parse_dump_items(r_value)
            if c_items is not None and r_items is not None:
                item_diffs = []
                for key in sorted(set(c_items) | set(r_items)):
                    c_item = c_items.get(key, "0")
                    r_item = r_items.get(key, "0")
                    if c_item != r_item:
                        item_diffs.append((key, c_item, r_item))
                for key, c_item, r_item in item_diffs[:20]:
                    diffs.append((f"dump:{prefix}:{key}", c_item, r_item))
                if len(item_diffs) > 20:
                    diffs.append((f"dump:{prefix}:...", f"{len(item_diffs) - 20} more", ""))
                continue
            diffs.append((f"dump:{prefix}", c_value, r_value))
    return diffs


def check_frame(args, frame, dump_env=None, auto_dumps=False):
    dump_env = dump_env if dump_env is not None else (make_dump_env(args) if args.compare_dumps else {})
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        c_future = executor.submit(run_c, args, frame, dump_env)
        r_future = executor.submit(run_rust, args, frame, dump_env)
        c_result = c_future.result()
        r_result = r_future.result()
    try:
        c_line, c_state = parse_state(c_result.stdout)
        r_line, r_state = parse_state(r_result.stdout)
    except RuntimeError:
        print(f"failed to parse replay checkpoint for frame={frame}", file=sys.stderr)
        print(f"C rc={c_result.returncode} tail:", file=sys.stderr)
        print("\n".join(c_result.stdout.splitlines()[-40:]), file=sys.stderr)
        print(f"Rust rc={r_result.returncode} tail:", file=sys.stderr)
        print("\n".join(r_result.stdout.splitlines()[-80:]), file=sys.stderr)
        raise
    diffs = compare_states(c_state, r_state, args.fields)
    if args.compare_dumps:
        diffs.extend(compare_dumps(c_result.stdout, r_result.stdout, args.dump_prefixes))
    check = {
        "frame": frame,
        "ok": c_result.returncode == 0 and r_result.returncode == 0 and not diffs,
        "diffs": diffs,
        "c_line": c_line,
        "r_line": r_line,
        "c_state": c_state,
        "r_state": r_state,
        "c_output": c_result.stdout,
        "r_output": r_result.stdout,
        "c_returncode": c_result.returncode,
        "r_returncode": r_result.returncode,
    }
    if auto_dumps and check["diffs"] and not args.compare_dumps:
        focused_env = make_dump_env(args, [classify_diff(check)])
        if focused_env:
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
                c_future = executor.submit(run_c, args, frame, focused_env)
                r_future = executor.submit(run_rust, args, frame, focused_env)
                c_dump_result = c_future.result()
                r_dump_result = r_future.result()
            dump_diffs = compare_dumps(c_dump_result.stdout, r_dump_result.stdout, args.dump_prefixes)
            if dump_diffs:
                check["diffs"].extend(dump_diffs)
                check["c_output"] = c_dump_result.stdout
                check["r_output"] = r_dump_result.stdout
                check["c_returncode"] = c_dump_result.returncode
                check["r_returncode"] = r_dump_result.returncode
                check["ok"] = False
    return check


def print_check(check, verbose=False):
    status = "ok" if check["ok"] else "DIFF"
    print(f"{status} frame={check['frame']} c_rc={check['c_returncode']} r_rc={check['r_returncode']}", flush=True)
    if check["diffs"]:
        for field, c_value, r_value in check["diffs"][:20]:
            print(f"  {symbolize_diff_field(field)}: C={c_value} Rust={r_value}")
        if len(check["diffs"]) > 20:
            print(f"  ... {len(check['diffs']) - 20} more")
        for hint in writer_hints(check):
            print(f"  hint: {hint}")
    if verbose or check["diffs"]:
        print(f"  C: {check['c_line']}")
        print(f"  R: {check['r_line']}")
    sys.stdout.flush()


def writer_hints(check):
    hints = []
    seen = set()
    for field, _, _ in check["diffs"]:
        for symbol in symbols_in_diff_field(field):
            lookup = symbol
            if "[" in lookup:
                lookup = lookup.split("[", 1)[0] + "[]"
            pair = WRITER_HINTS.get(lookup)
            if pair and lookup not in seen:
                seen.add(lookup)
                hints.append(f"{lookup}: {pair[0]} -> {pair[1]}")
    return hints[:6]


def diff_summary(check):
    if check["ok"]:
        return "ok"
    if check["c_returncode"] != 0 or check["r_returncode"] != 0:
        return f"rc C={check['c_returncode']} Rust={check['r_returncode']}"
    fields = [field for field, _, _ in check["diffs"]]
    page_fields = [field for field in fields if re.fullmatch(r"ram[0-7]", field)]
    dump_fields = [field for field in fields if field.startswith("dump:")]
    other_fields = [field for field in fields if field not in page_fields and field not in dump_fields]
    parts = []
    if page_fields:
        parts.append("pages=" + ",".join(page_fields))
    if other_fields:
        parts.append("fields=" + ",".join(other_fields[:8]))
        if len(other_fields) > 8:
            parts[-1] += f"+{len(other_fields) - 8}"
    if dump_fields:
        parts.append("dumps=" + ",".join(dump_fields[:4]))
        if len(dump_fields) > 4:
            parts[-1] += f"+{len(dump_fields) - 4}"
    return " ".join(parts) if parts else "diff"


def classify_diff(check):
    if check["ok"]:
        return "ok"
    fields = [field for field, _, _ in check["diffs"]]
    if check["c_returncode"] != 0 or check["r_returncode"] != 0:
        return "process"
    gameplay = {
        "ending", "main", "sub", "subsub", "saved", "indoors", "room", "ow",
        "hp", "state", "speed", "lspeed", "rng", "roommask", "x", "y",
        "joyh", "joyl", "fh", "fl", "dir", "face",
    }
    if any(field in gameplay for field in fields):
        return "gameplay"
    if any(field.startswith("dump:ram-page") or field == "ram3" for field in fields):
        return "palette-or-buffer"
    if any(field == "ram0" for field in fields):
        return "low-ram-scratch-or-state"
    if any(field == "ram7" or field == "sramhash" for field in fields):
        return "save-or-high-ram"
    if any(field.startswith("dump:") for field in fields):
        return "subsystem-dump"
    return "hash-only"


def print_sweep_row(check, verbose=False):
    c_state = check["c_state"]
    frame = check["frame"]
    status = "ok" if check["ok"] else "DIFF"
    location = (
        f"main={c_state.get('main', '?')} sub={c_state.get('sub', '?')} "
        f"room={c_state.get('room', '?')} ow={c_state.get('ow', '?')}"
    )
    print(
        f"{status} frame={frame} {location} class={classify_diff(check)} {diff_summary(check)}",
        flush=True,
    )
    if verbose and not check["ok"]:
        for field, c_value, r_value in check["diffs"][:12]:
            print(f"  {symbolize_diff_field(field)}: C={c_value} Rust={r_value}")
        for hint in writer_hints(check):
            print(f"  hint: {hint}")


def run_sweep(args):
    frames = []
    if args.sweep_frames:
        for chunk in args.sweep_frames.split(","):
            chunk = chunk.strip()
            if not chunk:
                continue
            frames.append(int(chunk, 0))
    if args.sweep_start is not None:
        for frame in range(args.sweep_start, args.sweep_end + 1, args.sweep_step):
            frames.append(frame)
    frames = sorted(dict.fromkeys(frames))
    if not frames:
        raise RuntimeError("sweep requested without frames")

    failed = False
    clusters = defaultdict(list)
    for frame in frames:
        check = check_frame(args, frame, auto_dumps=args.auto_dumps)
        print_sweep_row(check, args.verbose)
        if not check["ok"]:
            failed = True
            clusters[classify_diff(check)].append(check["frame"])

    if clusters:
        print("sweep-clusters:", flush=True)
        for key in sorted(clusters):
            values = clusters[key]
            print(
                f"  {key}: count={len(values)} first={values[0]} last={values[-1]}",
                flush=True,
            )
    return 1 if failed and args.sweep_fail_on_diff else 0


def make_dump_env(args, classes=None):
    env = {}
    if args.ram_dump_page is not None:
        env["ZELDA3_REPLAY_RAM_DUMP_PAGE"] = args.ram_dump_page
    dump_names = set()
    if classes:
        for class_name in classes:
            dump_names.update(CLASS_DUMPS.get(class_name, []))
    elif args.dumps:
        dump_names.update(DUMP_ENVS)
    for name in dump_names:
        env_name = DUMP_ENVS.get(name)
        if env_name:
            env[env_name] = "1"
    return env


def regression_frames(args):
    if args.regression_frames:
        return sorted(dict.fromkeys(int(chunk.strip(), 0) for chunk in args.regression_frames.split(",") if chunk.strip()))
    return DEFAULT_REGRESSION_FRAMES


def run_regression(args):
    failed = False
    clusters = defaultdict(list)
    print("regression-windows:", flush=True)
    frames = regression_frames(args)
    if args.regression_workers > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.regression_workers) as executor:
            checks = list(executor.map(lambda frame: check_frame(args, frame, auto_dumps=args.auto_dumps), frames))
    else:
        checks = [check_frame(args, frame, auto_dumps=args.auto_dumps) for frame in frames]

    for check in checks:
        print_sweep_row(check, args.verbose)
        if not check["ok"]:
            failed = True
            clusters[classify_diff(check)].append(check["frame"])
    if clusters:
        print("regression-clusters:", flush=True)
        for key in sorted(clusters):
            values = clusters[key]
            print(f"  {key}: count={len(values)} first={values[0]} last={values[-1]}", flush=True)
    return 1 if failed else 0


def main():
    parser = argparse.ArgumentParser(
        description="Bisect the first C/R replay-save checkpoint divergence."
    )
    parser.add_argument("--good", type=int, help="known matching frame")
    parser.add_argument("--bad", type=int, help="known divergent frame")
    parser.add_argument("--check-frame", type=int, action="append",
                        help="check one absolute frame and exit; may be repeated")
    parser.add_argument("--sweep-frames",
                        help="comma-separated absolute frames to check and summarize")
    parser.add_argument("--sweep-start", type=int,
                        help="first frame for range sweep")
    parser.add_argument("--sweep-end", type=int,
                        help="last frame for range sweep")
    parser.add_argument("--sweep-step", type=int, default=1000,
                        help="step size for --sweep-start/--sweep-end")
    parser.add_argument("--sweep-fail-on-diff", action="store_true",
                        help="return nonzero if any sweep frame diverges")
    parser.add_argument("--regression", action="store_true",
                        help="run the standard multi-window replay parity regression frames")
    parser.add_argument("--regression-frames",
                        help="comma-separated frame list for --regression; defaults to built-in route windows")
    parser.add_argument("--regression-workers", type=int, default=1,
                        help="parallel workers for --regression frame checks")
    parser.add_argument("--save", type=pathlib.Path, default=ROOT / "saves" / "zelda3-combined-route.sav")
    parser.add_argument("--rom", type=pathlib.Path, default=DEFAULT_C_ROOT / "zelda3.sfc")
    parser.add_argument("--c-root", type=pathlib.Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--rust-bin", type=pathlib.Path, default=ROOT / "target" / "release" / "zelda3")
    parser.add_argument("--checkpoint-frame", type=int,
                        help="optional matching frame to cache and resume from for later probes")
    parser.add_argument("--checkpoint-dir", type=pathlib.Path, default=DEFAULT_CHECKPOINT_DIR,
                        help="directory for generated C/R replay checkpoints")
    parser.add_argument("--c-checkpoint", type=pathlib.Path,
                        help="explicit C checkpoint path; implies --checkpoint-frame when used with --rust-checkpoint")
    parser.add_argument("--rust-checkpoint", type=pathlib.Path,
                        help="explicit Rust checkpoint path; implies --checkpoint-frame when used with --c-checkpoint")
    parser.add_argument("--refresh-checkpoints", action="store_true",
                        help="recreate generated C/R checkpoint files before probing")
    parser.add_argument("--save-local-checkpoint", action=argparse.BooleanOptionalAction, default=True,
                        help="after bisection, cache C/R checkpoints at the nearest known-good frame")
    parser.add_argument("--fields", default=DEFAULT_FIELDS,
                        help="comma-separated checkpoint keys to compare, or 'all' for every shared key")
    parser.add_argument("--list-focused-fields", action="store_true",
                        help="print the built-in focused field set and exit")
    parser.add_argument("--dumps", action="store_true",
                        help="enable normalized subsystem dumps at the final divergent frame")
    parser.add_argument("--compare-dumps", action="store_true",
                        help="include normalized dump lines in every frame comparison")
    parser.add_argument("--auto-dumps", action=argparse.BooleanOptionalAction, default=True,
                        help="re-run divergent check/sweep/regression frames with class-specific dumps")
    parser.add_argument("--isolate-runtime-cwd", action="store_true",
                        help="run each C/R process from a temp cwd with isolated saves/sram.* files")
    parser.add_argument("--dump-prefixes", default=",".join(DEFAULT_DUMP_PREFIXES),
                        help="comma-separated dump line prefixes to compare with --compare-dumps")
    parser.add_argument("--ram-dump-page",
                        help="dump nonzero bytes from one 1 KB WRAM page, e.g. 0x1000")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()
    if args.list_focused_fields:
        print(",".join(FOCUSED_FIELDS))
        return
    sweep_requested = args.sweep_frames is not None or args.sweep_start is not None or args.sweep_end is not None
    if args.regression_frames and not args.regression:
        parser.error("--regression-frames requires --regression")
    if args.regression_workers <= 0:
        parser.error("--regression-workers must be positive")
    if args.regression_workers > 1 and not args.isolate_runtime_cwd:
        parser.error("--regression-workers > 1 requires --isolate-runtime-cwd")
    if args.check_frame is None and not sweep_requested and not args.regression and (args.good is None or args.bad is None):
        parser.error("--good and --bad are required unless --list-focused-fields, --check-frame, --sweep-*, or --regression is used")
    if args.check_frame is not None and (args.good is not None or args.bad is not None or sweep_requested or args.regression):
        parser.error("--check-frame cannot be combined with --good/--bad, --sweep-*, or --regression")
    if sweep_requested and (args.good is not None or args.bad is not None or args.regression):
        parser.error("--sweep-* cannot be combined with --good/--bad or --regression")
    if args.regression and (args.good is not None or args.bad is not None):
        parser.error("--regression cannot be combined with --good/--bad")
    if (args.sweep_start is None) != (args.sweep_end is None):
        parser.error("--sweep-start and --sweep-end must be used together")
    if args.sweep_step <= 0:
        parser.error("--sweep-step must be positive")
    args.save = args.save.resolve()
    args.rom = args.rom.resolve()
    args.c_root = args.c_root.resolve()
    args.rust_bin = args.rust_bin.resolve()
    args.checkpoint_dir = args.checkpoint_dir.resolve()
    if args.checkpoint_frame is not None:
        if args.check_frame is not None:
            first_probe = min(args.check_frame)
        elif args.regression:
            first_probe = min(regression_frames(args))
        elif sweep_requested:
            sweep_candidates = []
            if args.sweep_frames:
                sweep_candidates.extend(int(chunk.strip(), 0) for chunk in args.sweep_frames.split(",") if chunk.strip())
            if args.sweep_start is not None:
                sweep_candidates.append(args.sweep_start)
            first_probe = min(sweep_candidates)
        else:
            first_probe = args.good
        if args.checkpoint_frame > first_probe:
            parser.error("--checkpoint-frame must be <= the first checked frame")
        if args.c_checkpoint is None:
            args.c_checkpoint = args.checkpoint_dir / f"c-frame-{args.checkpoint_frame}.sav"
        if args.rust_checkpoint is None:
            args.rust_checkpoint = args.checkpoint_dir / f"rust-frame-{args.checkpoint_frame}.sav"
    elif args.c_checkpoint is not None or args.rust_checkpoint is not None:
        parser.error("--c-checkpoint/--rust-checkpoint require --checkpoint-frame")
    if args.c_checkpoint is not None:
        args.c_checkpoint = args.c_checkpoint.resolve()
    if args.rust_checkpoint is not None:
        args.rust_checkpoint = args.rust_checkpoint.resolve()
    args.fields = [field for field in args.fields.split(",") if field]
    args.dump_prefixes = [prefix for prefix in args.dump_prefixes.split(",") if prefix]
    if args.ram_dump_page is not None:
        args.compare_dumps = True
    if args.compare_dumps:
        args.dumps = True
    ensure_checkpoints(args)

    if sweep_requested:
        return run_sweep(args)

    if args.regression:
        return run_regression(args)

    if args.check_frame is not None:
        failed = False
        for frame in args.check_frame:
            check = check_frame(args, frame, auto_dumps=args.auto_dumps)
            print_check(check, args.verbose)
            failed |= not check["ok"]
        return 1 if failed else 0

    good = check_frame(args, args.good)
    bad = check_frame(args, args.bad)
    print_check(good, args.verbose)
    print_check(bad, args.verbose)
    if not good["ok"]:
        print("--good must be a matching frame", file=sys.stderr)
        return 2
    if bad["ok"]:
        print("--bad must be a divergent frame", file=sys.stderr)
        return 2

    lo, hi = args.good, args.bad
    while hi - lo > 1:
        mid = (lo + hi) // 2
        check = check_frame(args, mid)
        print_check(check, args.verbose)
        if check["ok"]:
            lo = mid
        else:
            hi = mid

    print(f"first-divergent-frame={hi}", flush=True)
    if args.save_local_checkpoint:
        save_local_checkpoints(args, lo)
    final = check_frame(args, hi, make_dump_env(args, [classify_diff(check_frame(args, hi))]))
    print_check(final, verbose=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
