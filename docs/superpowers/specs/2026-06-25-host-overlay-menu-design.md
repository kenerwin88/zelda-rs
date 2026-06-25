# Host Overlay Menu Design

Date: 2026-06-25
Status: Approved design

## Goal

Add a polished LTTP-styled host overlay menu that appears before the game starts and is also accessible during play with `ESC`. The menu should make player-facing settings easy to reach, preserve the parity-sensitive Zelda core, and provide a first version of a developer map for safe navigation.

## Product Shape

The menu uses a hybrid structure: player-facing first, developer tools clearly available. `ESC` opens a normal pause-style screen first, not the developer map directly.

Primary tabs:

- `Play`: resume/start, video and effects shortcut, controls shortcut, developer map shortcut, save and quit.
- `Video`: presentation mode, lighting mode, shadows, viewport/fullscreen, live preview notice.
- `Controls`: keyboard/gamepad bindings, hotkey reference, reset defaults.
- `Developer Map`: curated presets, route bookmarks, destination details, locked unverified browser.
- `Dev Tools`: later tab or nested panel for state inspection, captures, and parity/debug helpers.

The visual language should feel like A Link to the Past: dark framed panels, gold borders, compact pixel-style text, cursor arrow selection, restrained color, and no modern floating-card treatment inside the game viewport.

## Runtime Behavior

Before startup:

- The window and host renderer can initialize.
- The menu appears before Zelda game state advances.
- Choosing normal play starts the usual boot/file-select path.
- Choosing a verified developer destination starts from a prepared destination path only after that path is proven safe.

During play:

- `ESC` opens the menu on the `Play` tab with `Resume Quest` selected.
- Game simulation fully pauses while the menu is open.
- Existing music continues at reduced menu volume if music was already playing.
- No new game frames, RAM mutations, input edges, or replay recorder events should occur while paused by the menu.
- `ESC` from the top-level menu resumes, except inside destructive confirmations where it cancels the confirmation.
- Window close still follows the normal quit/save path.

## Architecture

Use a host overlay, not an in-game Zelda module.

The menu belongs in the platform/bin layer around `NativeFrontend`, input handling, and presentation rendering. It must stay downstream of the fixed `256x224` game frame. The Zelda core should not gain a new menu module or parity-sensitive state for this feature.

Proposed pieces:

- `HostMenuState`: owns menu visibility, active tab, selection index, confirmation state, and pre-game versus in-game mode.
- `HostMenuAction`: describes effects requested by the menu, such as resume, start game, save and quit, cycle presentation mode, open developer map, or request a verified warp.
- `FrontendEvent` or equivalent input event stream: lets platform code report host keys such as `Escape`, arrows, confirm/cancel, tab navigation, and hotkeys before they are converted into SNES input.
- `MenuRenderer`: draws LTTP-styled panels and text as an overlay after the game frame has been drawn. In pre-game mode it draws over a static menu background.
- `RuntimeSettings`: central host-side settings model for presentation, lighting, shadows, viewport, fullscreen, and audio menu ducking.

Input routing:

- When the menu is closed, input flows to the game as it does today, except host hotkeys can still be intercepted.
- When the menu is open, menu navigation consumes host inputs and no SNES input is sent to `zelda_run_frame`.
- F6/F7/F8 remain shortcuts for presentation, lighting, and shadows, but they update the same `RuntimeSettings` used by the menu and show the same brief on-screen notice.

Rendering:

- Preserve the existing fixed-resolution game composition boundary.
- Draw the last completed game frame while paused.
- Draw the overlay after presentation effects only if that gives the crispest menu text; otherwise draw it in the same surface pass but keep it outside core game rendering. The implementation plan should choose the exact pass after inspecting renderer constraints.

Audio:

- Add a host-side menu ducking control.
- When the menu opens during play, keep existing music/audio running softly.
- Do not advance Zelda audio generation while gameplay is paused unless the implementation can prove it does not advance game state. Prefer mixer-level volume control over core audio mutation.

## Developer Map V1

The first Developer Map is a warp/navigation tool, not a general state inspector.

Scope:

- Curated presets.
- Current-route bookmarks.
- Current location marker.
- Destination details panel with room/screen id, source/provenance, and verification status.
- Locked arbitrary browser for unverified overworld screens and dungeon rooms.

Destination safety:

- Only verified destinations are selectable for warp/start.
- A destination is verified when it has a known safe initialization path or prepared checkpoint.
- Arbitrary room/screen ids may be visible but disabled until tested.

Data sources:

- Reuse existing route/probe infrastructure where possible: route coverage, replay checkpoints, direct entrance probes, dungeon room probes, and overworld screen probes.
- Store curated presets in a small checked-in manifest with stable ids, display names, destination type, proof source, and any checkpoint/initializer reference.
- Route bookmarks can be generated from the standard route and filtered to meaningful places instead of every frame.

Warp execution:

- Prefer prepared checkpoints for v1 if they are the lowest-risk way to start at verified destinations.
- If a direct initializer is cleaner for a destination class, require focused parity/replay tests before enabling it.
- The UI should not expose unsafe partial warps as active controls.

## Error Handling

- If a destination manifest entry points to a missing checkpoint or unsupported initializer, show it as locked with an explanation.
- If renderer/menu initialization fails, fall back to the existing playable path where possible, or print a clear startup error.
- If audio ducking fails, gameplay/menu still works with normal audio volume.
- If a menu action would discard progress, require a confirmation panel.

## Testing

Unit tests:

- `HostMenuState` opens pre-game and in-game with the correct default tab and selection.
- `ESC` toggles resume/cancel behavior correctly.
- Menu-open input is consumed and does not become SNES input.
- Settings changes update the shared runtime settings model.
- Developer destinations are enabled only when verified.

Frontend/platform tests:

- `--frontend-smoke` or a new smoke mode proves `ESC` opens the menu instead of quitting.
- Closing the window still requests quit/save.
- F6/F7/F8 and menu controls update the same presentation settings.

Pause tests:

- While menu is open, game RAM/frame count does not advance.
- The last rendered game frame remains visible behind the overlay.
- Audio is ducked at the host layer when music was already playing.

Parity:

- Existing replay and all-layer parity gates remain unchanged because the host overlay is outside core emulation.
- Any enabled developer destination initializer must get focused tests before it becomes selectable.

## Non-Goals For V1

- Arbitrary unrestricted room/screen warping.
- Full save-state manager UI.
- Deep RAM editor.
- Replacing the original Zelda file-select UI.
- Moving presentation effects into the parity-sensitive game renderer.

## Open Implementation Notes

- The implementation plan should decide whether overlay text is drawn before or after presentation effects based on crispness and renderer simplicity.
- The plan should inspect whether current audio output can support mixer-level ducking without changing Zelda audio generation.
- The first bookmark manifest should be small and useful rather than exhaustive.
