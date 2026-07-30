#!/usr/bin/env python3
"""Terminal browser for Snes9x route-recorder projects."""

from __future__ import annotations

import curses
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

import snes9x_route_recorder as recorder


RECORDING_KIND = "zelda3_snes9x_route_recording_v1"
COLOR_BODY = 1
COLOR_HEADING = 2
COLOR_FOCUSED = 3
COLOR_SELECTED = 4
COLOR_MESSAGE = 5


@dataclass
class ProjectView:
    path: Path
    manifest: dict
    labels: dict
    archived_boundaries: set[int]
    pairings: dict
    sram_origin: dict
    archived: bool = False

    @property
    def name(self) -> str:
        return self.path.name

    @property
    def boundaries(self) -> list[dict]:
        return self.manifest.get("boundaries", [])

    @property
    def takes(self) -> list[dict]:
        return self.manifest.get("takes", [])


@dataclass
class BrowserState:
    projects: list[ProjectView]
    project_index: int = 0
    item_index: int = 0
    focus: str = "projects"
    item_mode: str = "boundaries"
    show_hidden: bool = False
    message: str = ""

    def __post_init__(self) -> None:
        self.select_latest_item()

    def select_latest_item(self) -> None:
        self.item_index = max(0, len(self.items) - 1)

    @property
    def project(self) -> ProjectView | None:
        if not self.projects:
            return None
        self.project_index = min(self.project_index, len(self.projects) - 1)
        return self.projects[self.project_index]

    @property
    def items(self) -> list[dict]:
        project = self.project
        if project is None:
            return []
        if self.item_mode == "boundaries":
            if self.show_hidden:
                return project.boundaries
            return [
                boundary
                for boundary in project.boundaries
                if int(boundary["id"]) not in project.archived_boundaries
            ]
        if self.show_hidden:
            return project.takes
        return [take for take in project.takes if recorder.take_is_active(take)]

    @property
    def item(self) -> dict | None:
        items = self.items
        if not items:
            return None
        self.item_index = min(self.item_index, len(items) - 1)
        return items[self.item_index]


@dataclass(frozen=True)
class TuiConfig:
    project_root: Path
    included_project: Path
    binary: Path
    core: Path
    rom: Path
    no_build: bool = False
    recorder_script: Path = field(
        default_factory=lambda: Path(recorder.__file__).resolve()
    )


def discover_projects(
    project_root: Path, included_project: Path, *, show_hidden: bool = False
) -> list[ProjectView]:
    manifests: set[Path] = set()
    for candidate in (project_root, included_project):
        candidate = candidate.resolve()
        direct = candidate / "manifest.json"
        if direct.is_file():
            manifests.add(direct)
        if candidate.is_dir() and candidate == project_root.resolve():
            manifests.update(candidate.rglob("manifest.json"))

    projects = []
    for manifest_path in manifests:
        try:
            manifest = json.loads(manifest_path.read_text())
        except (OSError, json.JSONDecodeError):
            continue
        if manifest.get("kind") != RECORDING_KIND:
            continue
        path = manifest_path.parent
        try:
            label_data = recorder.load_labels(path)
            labels = label_data["boundaries"]
            archived = {
                int(value) for value in label_data.get("archived_boundaries", [])
            }
            pairings = recorder.load_pairings(path)["boundaries"]
            project_archived = bool(label_data.get("archived_project", False))
        except (OSError, ValueError, SystemExit):
            labels = {}
            archived = set()
            pairings = {}
            project_archived = False
        origin_path = path / "sram-origin.json"
        try:
            origin = json.loads(origin_path.read_text())
        except (OSError, json.JSONDecodeError):
            origin = {"source": "captured boundary SRAM (legacy project)"}
        if project_archived and not show_hidden:
            continue
        projects.append(
            ProjectView(
                path,
                manifest,
                labels,
                archived,
                pairings,
                origin,
                project_archived,
            )
        )
    return sorted(
        projects,
        key=lambda project: project.path.joinpath("manifest.json").stat().st_mtime,
        reverse=True,
    )


def build_resume_command(config: TuiConfig, project: Path, boundary: int) -> list[str]:
    command = [
        sys.executable,
        str(config.recorder_script),
        "record",
        "--project",
        str(project),
        "--start",
        str(boundary),
        "--binary",
        str(config.binary),
        "--core",
        str(config.core),
        "--rom",
        str(config.rom),
        "--allow-core-rollover",
    ]
    if config.no_build:
        command.append("--no-build")
    return command


def build_new_project_command(config: TuiConfig, project: Path, sram: str) -> list[str]:
    command = [
        sys.executable,
        str(config.recorder_script),
        "record",
        "--project",
        str(project),
        "--binary",
        str(config.binary),
        "--core",
        str(config.core),
        "--rom",
        str(config.rom),
    ]
    if sram.casefold() == "blank":
        command.append("--blank-sram")
    else:
        command.extend(["--sram", sram])
    if config.no_build:
        command.append("--no-build")
    return command


def route_folder_name(value: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "-", value.strip()).strip("-.")


def rename_project(project_root: Path, project: Path, requested_name: str) -> Path:
    root = project_root.resolve()
    source = project.resolve()
    if source.parent != root:
        raise SystemExit(f"project is outside the route root: {source}")
    slug = route_folder_name(requested_name)
    if not slug:
        raise SystemExit("route name must contain letters or numbers")
    destination = root / slug
    if destination == source:
        return source
    if destination.exists():
        raise SystemExit(f"project already exists: {destination}")
    source.rename(destination)
    return destination


def boundary_line(project: ProjectView, boundary: dict) -> str:
    boundary_id = int(boundary["id"])
    telemetry = boundary.get("telemetry", {})
    label = project.labels.get(str(boundary_id), "(unnamed)")
    if boundary.get("reset_start", False):
        parity = "reset-ready"
    elif str(boundary_id) in project.pairings:
        parity = "rust-paired"
    else:
        parity = "rust-needed"
    archived = " archived" if boundary_id in project.archived_boundaries else ""
    return (
        f"#{boundary_id:<3} {label:<24.24} parity={parity:<11} "
        f"module={telemetry.get('main', '?')}/{telemetry.get('sub', '?')} "
        f"hp={telemetry.get('health', '?')}{archived}"
    )


def take_line(take: dict) -> str:
    return (
        f"#{int(take['id']):<3} {recorder.take_name(take):<28.28} "
        f"boundary={take.get('start_boundary', '?'):<3} "
        f"frames={take.get('frames', 0):<8} status={take.get('status', '?')}"
    )


def project_line(project: ProjectView) -> str:
    archived = " archived" if project.archived else ""
    return (
        f"{project.name:<18.18} {len(project.boundaries)} saves / "
        f"{len(project.takes)} takes{archived}"
    )


def storage_description(project: ProjectView) -> str:
    origin = project.sram_origin
    if origin.get("source") == "file":
        source = origin.get("path") or origin.get("filename", "external file")
        return f"SRAM: {source} ({origin.get('sha256', '?')[:12]}…)"
    if origin.get("source") == "blank":
        return "SRAM: blank at project creation"
    return f"SRAM: {origin.get('source', 'captured in each boundary')}"


def viewport_start(total: int, selected: int, rows: int) -> int:
    if rows <= 0 or total <= rows:
        return 0
    return max(0, min(selected - rows + 1, total - rows))


def _configure_theme(stdscr) -> None:
    if not curses.has_colors():
        return
    try:
        curses.start_color()
        curses.init_pair(COLOR_BODY, curses.COLOR_WHITE, curses.COLOR_BLACK)
        curses.init_pair(COLOR_HEADING, curses.COLOR_CYAN, curses.COLOR_BLACK)
        curses.init_pair(COLOR_FOCUSED, curses.COLOR_BLACK, curses.COLOR_CYAN)
        curses.init_pair(COLOR_SELECTED, curses.COLOR_BLACK, curses.COLOR_WHITE)
        curses.init_pair(COLOR_MESSAGE, curses.COLOR_YELLOW, curses.COLOR_BLACK)
        stdscr.bkgd(" ", curses.color_pair(COLOR_BODY))
    except curses.error:
        # Some minimal terminals advertise colors but reject palette changes.
        pass


def _theme_attr(pair: int, fallback: int = 0) -> int:
    return curses.color_pair(pair) if curses.has_colors() else fallback


def _addstr(window, y: int, x: int, value: str, attr: int = 0) -> None:
    height, width = window.getmaxyx()
    if y < 0 or y >= height or x >= width:
        return
    try:
        window.addnstr(y, x, value, max(0, width - x - 1), attr)
    except curses.error:
        pass


def _prompt(stdscr, label: str, initial: str = "") -> str | None:
    height, width = stdscr.getmaxyx()
    default = f" [{initial}]" if initial else ""
    prompt = f"{label}{default}: "
    stdscr.move(height - 1, 0)
    stdscr.clrtoeol()
    _addstr(stdscr, height - 1, 0, prompt, curses.A_BOLD)
    stdscr.refresh()
    curses.echo()
    curses.curs_set(1)
    try:
        raw = stdscr.getstr(
            height - 1,
            min(len(prompt), width - 1),
            max(1, width - len(prompt) - 1),
        )
    except curses.error:
        return None
    finally:
        curses.noecho()
        curses.curs_set(0)
    value = raw.decode(errors="replace").strip()
    return value or initial or None


def _run_external(stdscr, command: list[str]) -> int:
    curses.def_prog_mode()
    curses.endwin()
    try:
        return subprocess.run(command, cwd=recorder.ROOT).returncode
    finally:
        curses.reset_prog_mode()
        stdscr.clear()
        stdscr.refresh()


def _refresh(state: BrowserState, config: TuiConfig) -> None:
    current_path = state.project.path if state.project else None
    state.projects = discover_projects(
        config.project_root,
        config.included_project,
        show_hidden=state.show_hidden,
    )
    if current_path is not None:
        state.project_index = next(
            (
                index
                for index, project in enumerate(state.projects)
                if project.path == current_path
            ),
            0,
        )
    state.item_index = min(state.item_index, max(0, len(state.items) - 1))


def _draw(stdscr, state: BrowserState, config: TuiConfig) -> None:
    stdscr.erase()
    height, width = stdscr.getmaxyx()
    _addstr(
        stdscr,
        0,
        0,
        "Snes9x Route Recorder",
        _theme_attr(COLOR_HEADING, curses.A_BOLD) | curses.A_BOLD,
    )
    _addstr(stdscr, 1, 0, f"Storage root: {config.project_root.resolve()}")
    if height < 12 or width < 80:
        _addstr(stdscr, 3, 0, "Resize terminal to at least 80x12.", curses.A_BOLD)
        stdscr.refresh()
        return

    left_width = min(34, max(24, width // 3))
    project_attr = (
        _theme_attr(COLOR_FOCUSED, curses.A_REVERSE)
        if state.focus == "projects"
        else _theme_attr(COLOR_HEADING)
    ) | curses.A_BOLD
    item_attr = (
        _theme_attr(COLOR_FOCUSED, curses.A_REVERSE)
        if state.focus == "items"
        else _theme_attr(COLOR_HEADING)
    ) | curses.A_BOLD
    _addstr(stdscr, 3, 0, " Projects ", project_attr)
    hidden = " + hidden" if state.show_hidden else ""
    _addstr(
        stdscr,
        3,
        left_width + 1,
        f" {state.item_mode.title()}{hidden} ",
        item_attr,
    )
    rows = max(1, height - 10)
    project_start = viewport_start(len(state.projects), state.project_index, rows)
    for visible_index, project in enumerate(
        state.projects[project_start : project_start + rows]
    ):
        row = visible_index + 4
        project_index = project_start + visible_index
        attr = (
            _theme_attr(COLOR_SELECTED, curses.A_REVERSE)
            if project_index == state.project_index
            else 0
        )
        _addstr(
            stdscr,
            row,
            1,
            project_line(project),
            attr,
        )
    for y in range(3, height - 5):
        _addstr(stdscr, y, left_width, "│")

    project = state.project
    if project is None:
        _addstr(stdscr, 5, left_width + 2, "No recorder projects found.")
    else:
        item_start = viewport_start(len(state.items), state.item_index, rows)
        for visible_index, item in enumerate(
            state.items[item_start : item_start + rows]
        ):
            row = visible_index + 4
            item_index = item_start + visible_index
            attr = (
                _theme_attr(COLOR_SELECTED, curses.A_REVERSE)
                if item_index == state.item_index
                else 0
            )
            text = (
                boundary_line(project, item)
                if state.item_mode == "boundaries"
                else take_line(item)
            )
            _addstr(stdscr, row, left_width + 2, text, attr)
        detail_y = height - 5
        _addstr(stdscr, detail_y, 0, f"Project: {project.path.resolve()}")
        _addstr(stdscr, detail_y + 1, 0, storage_description(project))
        item = state.item
        if item and state.item_mode == "boundaries":
            boundary_dir = project.path / Path(item["state_path"]).parent
            _addstr(
                stdscr,
                detail_y + 2,
                0,
                f"Selected save files: {boundary_dir.resolve()}",
            )
        elif item:
            take_id = int(item["id"])
            _addstr(
                stdscr,
                detail_y + 2,
                0,
                f"Selected take files: {(project.path / f'takes/{take_id:04}').resolve()}",
            )

    help_text = (
        "Tab switch  ↑↓ move  Enter resume  n rename selected  o screenshot  "
        "t saves/takes  m merge across save  x archive/restore  v show hidden  "
        "a new route  r refresh  q quit"
    )
    _addstr(
        stdscr,
        height - 2,
        0,
        help_text,
        _theme_attr(COLOR_HEADING, curses.A_BOLD) | curses.A_BOLD,
    )
    _addstr(stdscr, height - 1, 0, state.message, _theme_attr(COLOR_MESSAGE))
    stdscr.refresh()


def _selected_move(state: BrowserState, delta: int) -> None:
    if state.focus == "projects":
        if state.projects:
            state.project_index = max(
                0, min(len(state.projects) - 1, state.project_index + delta)
            )
            state.select_latest_item()
    elif state.items:
        state.item_index = max(0, min(len(state.items) - 1, state.item_index + delta))


def toggle_selected_archive(state: BrowserState) -> str:
    project = state.project
    if project is None:
        return "No project selected."
    if state.focus == "projects":
        archived = not project.archived
        recorder.set_project_archived(project.path, archived)
        return (
            f"Project {project.name} {'archived' if archived else 'restored'}; "
            "files preserved."
        )
    item = state.item
    if item is None:
        return "No save or take selected."
    item_id = int(item["id"])
    if state.item_mode == "boundaries":
        archived = item_id not in project.archived_boundaries
        recorder.set_boundary_archived(project.path, item_id, archived)
        noun = "Save"
    else:
        archived = item.get("status") != "discarded"
        try:
            recorder.set_take_discarded(project.path, item_id, archived)
        except SystemExit as error:
            return str(error)
        noun = "Take"
    return (
        f"{noun} #{item_id} {'archived' if archived else 'restored'}; files preserved."
    )


def merge_selected_boundary(state: BrowserState) -> str:
    project = state.project
    item = state.item
    if (
        project is None
        or state.focus != "items"
        or state.item_mode != "boundaries"
        or item is None
    ):
        return "Select an intermediate save to merge its adjacent takes."
    boundary_id = int(item["id"])
    merged = recorder.merge_takes_across_boundary(project.path, boundary_id)
    source_ids = merged["merged_from_takes"]
    return (
        f"Merged takes #{source_ids[0]} and #{source_ids[1]} into take #{merged['id']} "
        f"across save #{boundary_id}; originals preserved as hidden provenance."
    )


def _handle_action(stdscr, state: BrowserState, config: TuiConfig, key: int) -> bool:
    project = state.project
    item = state.item
    if key in (ord("q"), 27):
        return False
    if key in (curses.KEY_UP, ord("k")):
        _selected_move(state, -1)
    elif key in (curses.KEY_DOWN, ord("j")):
        _selected_move(state, 1)
    elif key in (9, curses.KEY_LEFT, curses.KEY_RIGHT):
        state.focus = "items" if state.focus == "projects" else "projects"
    elif key == ord("t"):
        state.item_mode = "takes" if state.item_mode == "boundaries" else "boundaries"
        state.select_latest_item()
        state.focus = "items"
    elif key == ord("v"):
        state.show_hidden = not state.show_hidden
        _refresh(state, config)
        state.select_latest_item()
        state.message = (
            "Showing hidden items." if state.show_hidden else "Hidden items concealed."
        )
    elif key == ord("r"):
        _refresh(state, config)
        state.message = "Refreshed recorder projects."
    elif key in (10, 13, curses.KEY_ENTER) and project and item:
        if state.focus == "projects":
            state.focus = "items"
        elif state.item_mode == "boundaries":
            boundary = int(item["id"])
            code = _run_external(
                stdscr, build_resume_command(config, project.path, boundary)
            )
            _refresh(state, config)
            state.item_mode = "boundaries"
            state.select_latest_item()
            state.message = f"Recorder returned with status {code}."
    elif key == ord("n") and state.focus == "projects" and project:
        name = _prompt(stdscr, "New route folder name", project.name)
        if name:
            try:
                renamed = rename_project(config.project_root, project.path, name)
                _refresh(state, config)
                state.project_index = next(
                    (
                        index
                        for index, candidate in enumerate(state.projects)
                        if candidate.path == renamed
                    ),
                    state.project_index,
                )
                state.message = f"Renamed route to {renamed.name}."
            except SystemExit as error:
                state.message = str(error)
    elif (
        key == ord("n")
        and state.focus == "items"
        and project
        and item
        and state.item_mode == "boundaries"
    ):
        old = project.labels.get(str(item["id"]), "")
        label = _prompt(stdscr, "New save name", old)
        if label:
            try:
                recorder.name_boundary(project.path, int(item["id"]), label)
                _refresh(state, config)
                state.message = f"Named save #{item['id']}: {label}"
            except SystemExit as error:
                state.message = str(error)
    elif (
        key == ord("n")
        and state.focus == "items"
        and project
        and item
        and state.item_mode == "takes"
    ):
        name = _prompt(stdscr, "New take name", recorder.take_name(item))
        if name:
            try:
                recorder.name_take(project.path, int(item["id"]), name)
                _refresh(state, config)
                state.message = f"Named take #{item['id']}: {name}"
            except SystemExit as error:
                state.message = str(error)
    elif (
        key == ord("o")
        and state.focus == "items"
        and project
        and item
        and state.item_mode == "boundaries"
    ):
        screenshot = project.path / item.get("screenshot_path", "")
        if screenshot.is_file() and sys.platform == "darwin":
            subprocess.run(["open", str(screenshot)], check=False)
            state.message = f"Opened {screenshot.name}."
        else:
            state.message = f"Screenshot: {screenshot.resolve()}"
    elif key == ord("x") and project:
        message = toggle_selected_archive(state)
        _refresh(state, config)
        state.select_latest_item()
        state.message = message
    elif (
        key == ord("m")
        and state.focus == "items"
        and project
        and item
        and state.item_mode == "boundaries"
    ):
        confirmation = _prompt(
            stdscr,
            f"Type merge to combine takes across save #{item['id']}",
        )
        if confirmation and confirmation.strip().lower() == "merge":
            try:
                state.message = merge_selected_boundary(state)
                _refresh(state, config)
            except SystemExit as error:
                state.message = str(error)
        elif confirmation is not None:
            state.message = "Merge cancelled."
    elif key == ord("a"):
        name = _prompt(stdscr, "New route folder name")
        if name:
            slug = route_folder_name(name)
            if not slug:
                state.message = "Route name must contain letters or numbers."
            else:
                seed = _prompt(stdscr, "SRAM path, or type blank", "blank")
                if seed:
                    path = config.project_root / slug
                    if path.exists():
                        state.message = f"Project already exists: {path}"
                    else:
                        code = _run_external(
                            stdscr, build_new_project_command(config, path, seed)
                        )
                        _refresh(state, config)
                        state.project_index = next(
                            (
                                index
                                for index, project in enumerate(state.projects)
                                if project.path == path.resolve()
                            ),
                            state.project_index,
                        )
                        state.item_mode = "boundaries"
                        state.select_latest_item()
                        state.message = f"New route returned with status {code}."
    return True


def _main(stdscr, config: TuiConfig) -> None:
    _configure_theme(stdscr)
    curses.curs_set(0)
    stdscr.keypad(True)
    state = BrowserState(
        discover_projects(config.project_root, config.included_project)
    )
    while True:
        _draw(stdscr, state, config)
        if not _handle_action(stdscr, state, config, stdscr.getch()):
            return


def run(args) -> None:
    config = TuiConfig(
        project_root=args.project_root,
        included_project=args.project,
        binary=args.binary,
        core=args.core,
        rom=args.rom,
        no_build=args.no_build,
    )
    config.project_root.mkdir(parents=True, exist_ok=True)
    curses.wrapper(_main, config)
