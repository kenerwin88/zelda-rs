# Host Overlay Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an LTTP-styled host overlay menu that appears before game startup, opens via `ESC` during play, pauses gameplay, keeps existing music softly audible, and exposes a verified Developer Map v1.

**Architecture:** Implement the menu outside Zelda core state in the host/platform/rendering layer. Input is intercepted before SNES input conversion when the menu is visible, the game loop stops advancing while paused, and the renderer draws a menu overlay downstream of the fixed `256x224` game frame.

**Tech Stack:** Rust, winit 0.30, wgpu 29, cpal, existing `platform`, `renderer`, `zelda3-bin`, and `parity` crates.

## Global Constraints

- Keep menu state outside Zelda game modules and parity-sensitive native state.
- Preserve the fixed `256x224` game composition boundary.
- `ESC` opens a normal resume-first pause menu, not the Developer Map directly.
- Pre-game menu appears before Zelda game state advances.
- Menu-open gameplay fully pauses; no game frames, RAM mutations, input edges, or replay recorder events occur.
- Already-playing music continues softly while the in-game menu is open.
- Developer Map v1 enables only curated presets and route bookmarks with verification metadata.
- Arbitrary overworld screens and dungeon rooms are visible only as locked destinations until verified.
- F6/F7/F8 remain shortcuts and update the same runtime settings used by the menu.
- Existing replay and all-layer parity gates must remain unchanged.

---

## File Structure

- Create `crates/platform/src/host_menu.rs`: pure host menu state machine, menu input events, actions, settings, and destination metadata.
- Modify `crates/platform/src/lib.rs`: expose host menu types, collect host input events from winit/gamepad, consume SNES input while menu is open, and add audio ducking.
- Modify `crates/renderer/src/lib.rs`: add a lightweight host menu overlay render path and tests for menu overlay text/style data.
- Modify `zelda3-bin/src/main.rs`: integrate pre-game menu state, in-game pause behavior, menu actions, and developer destination selection into the playable loop only.
- Create `zelda3-bin/src/developer_destinations.rs`: checked-in curated Developer Map v1 presets and route bookmarks.
- Modify `zelda3-bin/src/main.rs` to add `mod developer_destinations;` near the top-level imports.
- Test with existing crate tests, frontend smoke paths, and parity pre-commit gates.

---

### Task 1: Host Menu State Machine

**Files:**
- Create: `crates/platform/src/host_menu.rs`
- Modify: `crates/platform/src/lib.rs`

**Interfaces:**
- Produces: `HostMenuState`, `HostMenuMode`, `HostMenuTab`, `HostMenuInput`, `HostMenuAction`, `RuntimeSettings`, `PresentationChoice`, `LightingChoice`, `ShadowChoice`, `DeveloperDestination`, `DeveloperDestinationStatus`.
- Consumes: no new code from subsequent tasks.

- [ ] **Step 1: Add the module export shell**

Add this near the top of `crates/platform/src/lib.rs`, after the `use` block:

```rust
pub mod host_menu;
pub use host_menu::{
    DeveloperDestination, DeveloperDestinationStatus, HostMenuAction, HostMenuInput,
    HostMenuMode, HostMenuState, HostMenuTab, LightingChoice, PresentationChoice,
    RuntimeSettings, ShadowChoice,
};
```

- [ ] **Step 2: Write failing state-machine tests**

Create `crates/platform/src/host_menu.rs` with the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_destinations() -> HostMenuState {
        HostMenuState::new(
            HostMenuMode::InGame,
            vec![
                DeveloperDestination::verified("sanctuary", "Sanctuary", "route frame 12000"),
                DeveloperDestination::locked("room-003f", "Room 003F", "unverified room init"),
            ],
        )
    }

    #[test]
    fn pregamemenu_starts_on_play_tab_with_start_selected() {
        let state = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        assert!(state.is_open());
        assert_eq!(state.mode(), HostMenuMode::PreGame);
        assert_eq!(state.active_tab(), HostMenuTab::Play);
        assert_eq!(state.selected_label(), "Start Quest");
    }

    #[test]
    fn ingame_menu_opens_on_resume_first() {
        let state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        assert!(state.is_open());
        assert_eq!(state.selected_label(), "Resume Quest");
    }

    #[test]
    fn escape_resumes_from_top_level_ingame_menu() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        assert_eq!(state.handle_input(HostMenuInput::Cancel), Some(HostMenuAction::Resume));
    }

    #[test]
    fn escape_does_not_close_pregame_menu() {
        let mut state = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        assert_eq!(state.handle_input(HostMenuInput::Cancel), None);
        assert!(state.is_open());
    }

    #[test]
    fn video_tab_cycles_runtime_settings() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        state.handle_input(HostMenuInput::NextTab);
        assert_eq!(state.active_tab(), HostMenuTab::Video);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::SetPresentation(PresentationChoice::Sharp))
        );
        assert_eq!(state.runtime_settings().presentation, PresentationChoice::Sharp);
    }

    #[test]
    fn developer_map_only_activates_verified_destinations() {
        let mut state = state_with_destinations();
        state.set_active_tab(HostMenuTab::DeveloperMap);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::WarpToVerifiedDestination("sanctuary"))
        );
        state.handle_input(HostMenuInput::Down);
        assert_eq!(state.selected_label(), "Room 003F");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
    }
}
```

- [ ] **Step 3: Run the failing tests**

Run:

```bash
cargo test -p platform host_menu -- --nocapture
```

Expected: compilation fails because `host_menu` types do not exist yet.

- [ ] **Step 4: Implement the minimal host menu model**

Replace `crates/platform/src/host_menu.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostMenuMode {
    PreGame,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostMenuTab {
    Play,
    Video,
    Controls,
    DeveloperMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostMenuInput {
    Up,
    Down,
    Left,
    Right,
    PreviousTab,
    NextTab,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationChoice {
    Off,
    Sharp,
    Crt,
}

impl PresentationChoice {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Sharp,
            Self::Sharp => Self::Crt,
            Self::Crt => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingChoice {
    Off,
    Ambient,
    Dynamic,
}

impl LightingChoice {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Ambient,
            Self::Ambient => Self::Dynamic,
            Self::Dynamic => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowChoice {
    Off,
    Raycast,
}

impl ShadowChoice {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Raycast,
            Self::Raycast => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub presentation: PresentationChoice,
    pub lighting: LightingChoice,
    pub shadows: ShadowChoice,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            presentation: PresentationChoice::Off,
            lighting: LightingChoice::Off,
            shadows: ShadowChoice::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperDestinationStatus {
    Verified,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperDestination {
    pub id: &'static str,
    pub label: &'static str,
    pub provenance: &'static str,
    pub status: DeveloperDestinationStatus,
}

impl DeveloperDestination {
    pub fn verified(id: &'static str, label: &'static str, provenance: &'static str) -> Self {
        Self {
            id,
            label,
            provenance,
            status: DeveloperDestinationStatus::Verified,
        }
    }

    pub fn locked(id: &'static str, label: &'static str, provenance: &'static str) -> Self {
        Self {
            id,
            label,
            provenance,
            status: DeveloperDestinationStatus::Locked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostMenuAction {
    StartQuest,
    Resume,
    SaveAndQuit,
    SetPresentation(PresentationChoice),
    SetLighting(LightingChoice),
    SetShadows(ShadowChoice),
    WarpToVerifiedDestination(&'static str),
}

#[derive(Debug, Clone)]
pub struct HostMenuState {
    mode: HostMenuMode,
    open: bool,
    active_tab: HostMenuTab,
    selected_index: usize,
    runtime_settings: RuntimeSettings,
    developer_destinations: Vec<DeveloperDestination>,
}

impl HostMenuState {
    pub fn new(mode: HostMenuMode, developer_destinations: Vec<DeveloperDestination>) -> Self {
        Self {
            mode,
            open: true,
            active_tab: HostMenuTab::Play,
            selected_index: 0,
            runtime_settings: RuntimeSettings::default(),
            developer_destinations,
        }
    }

    pub fn open_ingame(&mut self) {
        self.mode = HostMenuMode::InGame;
        self.open = true;
        self.active_tab = HostMenuTab::Play;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn mode(&self) -> HostMenuMode {
        self.mode
    }

    pub fn active_tab(&self) -> HostMenuTab {
        self.active_tab
    }

    pub fn set_active_tab(&mut self, tab: HostMenuTab) {
        self.active_tab = tab;
        self.selected_index = 0;
    }

    pub fn runtime_settings(&self) -> RuntimeSettings {
        self.runtime_settings
    }

    pub fn selected_label(&self) -> &'static str {
        match self.active_tab {
            HostMenuTab::Play => self.play_items()[self.selected_index],
            HostMenuTab::Video => self.video_items()[self.selected_index],
            HostMenuTab::Controls => self.controls_items()[self.selected_index],
            HostMenuTab::DeveloperMap => self
                .developer_destinations
                .get(self.selected_index)
                .map(|destination| destination.label)
                .unwrap_or("No verified destinations"),
        }
    }

    pub fn handle_input(&mut self, input: HostMenuInput) -> Option<HostMenuAction> {
        match input {
            HostMenuInput::Up => {
                self.move_selection(-1);
                None
            }
            HostMenuInput::Down => {
                self.move_selection(1);
                None
            }
            HostMenuInput::Left | HostMenuInput::PreviousTab => {
                self.previous_tab();
                None
            }
            HostMenuInput::Right | HostMenuInput::NextTab => {
                self.next_tab();
                None
            }
            HostMenuInput::Cancel => {
                if self.mode == HostMenuMode::InGame {
                    Some(HostMenuAction::Resume)
                } else {
                    None
                }
            }
            HostMenuInput::Confirm => self.confirm_selected(),
        }
    }

    fn confirm_selected(&mut self) -> Option<HostMenuAction> {
        match self.active_tab {
            HostMenuTab::Play => match (self.mode, self.selected_index) {
                (HostMenuMode::PreGame, 0) => Some(HostMenuAction::StartQuest),
                (HostMenuMode::InGame, 0) => Some(HostMenuAction::Resume),
                (_, 1) => {
                    self.set_active_tab(HostMenuTab::Video);
                    None
                }
                (_, 2) => {
                    self.set_active_tab(HostMenuTab::Controls);
                    None
                }
                (_, 3) => {
                    self.set_active_tab(HostMenuTab::DeveloperMap);
                    None
                }
                (_, 4) => Some(HostMenuAction::SaveAndQuit),
                _ => None,
            },
            HostMenuTab::Video => match self.selected_index {
                0 => {
                    self.runtime_settings.presentation = self.runtime_settings.presentation.next();
                    Some(HostMenuAction::SetPresentation(
                        self.runtime_settings.presentation,
                    ))
                }
                1 => {
                    self.runtime_settings.lighting = self.runtime_settings.lighting.next();
                    Some(HostMenuAction::SetLighting(self.runtime_settings.lighting))
                }
                2 => {
                    self.runtime_settings.shadows = self.runtime_settings.shadows.next();
                    Some(HostMenuAction::SetShadows(self.runtime_settings.shadows))
                }
                _ => None,
            },
            HostMenuTab::Controls => None,
            HostMenuTab::DeveloperMap => self
                .developer_destinations
                .get(self.selected_index)
                .filter(|destination| destination.status == DeveloperDestinationStatus::Verified)
                .map(|destination| HostMenuAction::WarpToVerifiedDestination(destination.id)),
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.current_len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }
        self.selected_index = match delta {
            -1 if self.selected_index == 0 => len - 1,
            -1 => self.selected_index - 1,
            1 => (self.selected_index + 1) % len,
            _ => self.selected_index,
        };
    }

    fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            HostMenuTab::Play => HostMenuTab::Video,
            HostMenuTab::Video => HostMenuTab::Controls,
            HostMenuTab::Controls => HostMenuTab::DeveloperMap,
            HostMenuTab::DeveloperMap => HostMenuTab::Play,
        };
        self.selected_index = 0;
    }

    fn previous_tab(&mut self) {
        self.active_tab = match self.active_tab {
            HostMenuTab::Play => HostMenuTab::DeveloperMap,
            HostMenuTab::Video => HostMenuTab::Play,
            HostMenuTab::Controls => HostMenuTab::Video,
            HostMenuTab::DeveloperMap => HostMenuTab::Controls,
        };
        self.selected_index = 0;
    }

    fn current_len(&self) -> usize {
        match self.active_tab {
            HostMenuTab::Play => self.play_items().len(),
            HostMenuTab::Video => self.video_items().len(),
            HostMenuTab::Controls => self.controls_items().len(),
            HostMenuTab::DeveloperMap => self.developer_destinations.len().max(1),
        }
    }

    fn play_items(&self) -> &'static [&'static str] {
        match self.mode {
            HostMenuMode::PreGame => &["Start Quest", "Video & Effects", "Controls", "Developer Map", "Quit"],
            HostMenuMode::InGame => &["Resume Quest", "Video & Effects", "Controls", "Developer Map", "Save & Quit"],
        }
    }

    fn video_items(&self) -> &'static [&'static str] {
        &["Presentation", "Lighting", "Shadows", "Viewport"]
    }

    fn controls_items(&self) -> &'static [&'static str] {
        &["Keyboard", "Gamepad", "Reset Defaults"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_destinations() -> HostMenuState {
        HostMenuState::new(
            HostMenuMode::InGame,
            vec![
                DeveloperDestination::verified("sanctuary", "Sanctuary", "route frame 12000"),
                DeveloperDestination::locked("room-003f", "Room 003F", "unverified room init"),
            ],
        )
    }

    #[test]
    fn pregamemenu_starts_on_play_tab_with_start_selected() {
        let state = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        assert!(state.is_open());
        assert_eq!(state.mode(), HostMenuMode::PreGame);
        assert_eq!(state.active_tab(), HostMenuTab::Play);
        assert_eq!(state.selected_label(), "Start Quest");
    }

    #[test]
    fn ingame_menu_opens_on_resume_first() {
        let state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        assert!(state.is_open());
        assert_eq!(state.selected_label(), "Resume Quest");
    }

    #[test]
    fn escape_resumes_from_top_level_ingame_menu() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        assert_eq!(state.handle_input(HostMenuInput::Cancel), Some(HostMenuAction::Resume));
    }

    #[test]
    fn escape_does_not_close_pregame_menu() {
        let mut state = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        assert_eq!(state.handle_input(HostMenuInput::Cancel), None);
        assert!(state.is_open());
    }

    #[test]
    fn video_tab_cycles_runtime_settings() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        state.handle_input(HostMenuInput::NextTab);
        assert_eq!(state.active_tab(), HostMenuTab::Video);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::SetPresentation(PresentationChoice::Sharp))
        );
        assert_eq!(state.runtime_settings().presentation, PresentationChoice::Sharp);
    }

    #[test]
    fn developer_map_only_activates_verified_destinations() {
        let mut state = state_with_destinations();
        state.set_active_tab(HostMenuTab::DeveloperMap);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::WarpToVerifiedDestination("sanctuary"))
        );
        state.handle_input(HostMenuInput::Down);
        assert_eq!(state.selected_label(), "Room 003F");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
    }
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p platform host_menu -- --nocapture
```

Expected: all `host_menu` tests pass.

- [ ] **Step 6: Commit**

Run:

```bash
git add crates/platform/src/lib.rs crates/platform/src/host_menu.rs
git commit -m "feat: add host menu state model"
```

---

### Task 2: Host Input Event Routing

**Files:**
- Modify: `crates/platform/src/lib.rs`

**Interfaces:**
- Consumes: `HostMenuInput` from Task 1.
- Produces: `NativeFrontend::drain_host_menu_inputs(&mut self) -> Vec<HostMenuInput>` and `NativeFrontend::poll_input_with_menu(&mut self, menu_open: bool) -> u16`.

- [ ] **Step 1: Write failing input-routing tests**

Add these tests in `crates/platform/src/lib.rs` inside the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn escape_maps_to_host_menu_input() {
    assert_eq!(
        key_to_host_menu_input(KeyCode::Escape, ElementState::Pressed),
        Some(HostMenuInput::Cancel)
    );
    assert_eq!(
        key_to_host_menu_input(KeyCode::Escape, ElementState::Released),
        None
    );
}

#[test]
fn menu_open_consumes_snes_direction_keys() {
    let mut input_state = 0;
    handle_key_input_state_with_menu(&mut input_state, KeyCode::ArrowDown, ElementState::Pressed, true);
    assert_eq!(input_state, 0);
    handle_key_input_state_with_menu(&mut input_state, KeyCode::ArrowDown, ElementState::Pressed, false);
    assert_eq!(input_state, 1 << 5);
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test -p platform escape_maps_to_host_menu_input -- --nocapture
cargo test -p platform menu_open_consumes_snes_direction_keys -- --nocapture
```

Expected: compilation fails because helper functions do not exist.

- [ ] **Step 3: Add host input event storage to `NativeHandler`**

In `NativeHandler`, add:

```rust
host_menu_inputs: VecDeque<HostMenuInput>,
menu_open: bool,
```

In the `NativeHandler` initializer in `NativeFrontend::new_with_options`, add:

```rust
host_menu_inputs: VecDeque::new(),
menu_open: false,
```

- [ ] **Step 4: Add frontend methods**

In `impl NativeFrontend`, add:

```rust
pub fn set_menu_open(&mut self, open: bool) {
    self.handler.menu_open = open;
    if open {
        self.handler.input_state = 0;
    }
}

pub fn drain_host_menu_inputs(&mut self) -> Vec<HostMenuInput> {
    self.handler.host_menu_inputs.drain(..).collect()
}

pub fn poll_input_with_menu(&mut self, menu_open: bool) -> u16 {
    self.set_menu_open(menu_open);
    if menu_open {
        let _ = self.poll_input();
        0
    } else {
        self.poll_input()
    }
}
```

- [ ] **Step 5: Add host input mapping helpers**

Near `handle_key_input_state`, add:

```rust
fn key_to_host_menu_input(key: KeyCode, state: ElementState) -> Option<HostMenuInput> {
    if state != ElementState::Pressed {
        return None;
    }
    match key {
        KeyCode::Escape => Some(HostMenuInput::Cancel),
        KeyCode::ArrowUp => Some(HostMenuInput::Up),
        KeyCode::ArrowDown => Some(HostMenuInput::Down),
        KeyCode::ArrowLeft => Some(HostMenuInput::Left),
        KeyCode::ArrowRight => Some(HostMenuInput::Right),
        KeyCode::Enter | KeyCode::KeyZ | KeyCode::KeyX => Some(HostMenuInput::Confirm),
        KeyCode::Tab | KeyCode::KeyE | KeyCode::KeyV | KeyCode::KeyW => {
            Some(HostMenuInput::NextTab)
        }
        KeyCode::KeyQ | KeyCode::KeyC => Some(HostMenuInput::PreviousTab),
        _ => None,
    }
}

fn handle_key_input_state_with_menu(
    input_state: &mut u16,
    key: KeyCode,
    state: ElementState,
    menu_open: bool,
) {
    if menu_open {
        return;
    }
    handle_key_input_state(input_state, key, state);
}
```

- [ ] **Step 6: Route events in `window_event`**

Replace the `KeyboardInput` branch body with:

```rust
if let PhysicalKey::Code(key) = event.physical_key {
    if let Some(input) = key_to_host_menu_input(key, event.state) {
        self.host_menu_inputs.push_back(input);
    }
    if let Some(action) = presentation_hotkey_action(key, event.state) {
        if let Some(renderer) = &mut self.renderer {
            apply_presentation_hotkey(renderer, action);
        }
    }
    handle_key_input_state_with_menu(&mut self.input_state, key, event.state, self.menu_open);
}
```

`key_to_host_menu_input` is the only source of `Escape` menu events in this branch.

- [ ] **Step 7: Run tests**

Run:

```bash
cargo test -p platform escape_maps_to_host_menu_input -- --nocapture
cargo test -p platform menu_open_consumes_snes_direction_keys -- --nocapture
cargo test -p platform --lib
```

Expected: both focused tests pass, then all platform lib tests pass.

- [ ] **Step 8: Commit**

Run:

```bash
git add crates/platform/src/lib.rs
git commit -m "feat: route host menu input"
```

---

### Task 3: Runtime Settings Bridge

**Files:**
- Modify: `crates/renderer/src/lib.rs`
- Modify: `crates/platform/src/lib.rs`

**Interfaces:**
- Consumes: `RuntimeSettings`, `PresentationChoice`, `LightingChoice`, `ShadowChoice`.
- Produces: renderer-neutral `RendererRuntimeSettings`, `RendererPresentationChoice`, `RendererLightingChoice`, `RendererShadowChoice`, `FrameRenderer::apply_runtime_settings(&mut self, settings: RendererRuntimeSettings)`, and `NativeFrontend::apply_runtime_settings(&mut self, settings: RuntimeSettings)`.

- [ ] **Step 1: Write failing renderer settings test**

Add to `crates/renderer/src/lib.rs` tests:

```rust
#[test]
fn runtime_settings_map_to_renderer_presentation_params() {
    let settings = RendererRuntimeSettings {
        presentation: RendererPresentationChoice::Crt,
        lighting: RendererLightingChoice::Dynamic,
        shadows: RendererShadowChoice::Raycast,
    };
    let params = PresentationParams::from_runtime_settings(settings);
    assert_eq!(params.presentation, PresentationMode::Crt);
    assert_eq!(params.lighting, LightingMode::Dynamic);
    assert_eq!(params.shadows, ShadowMode::Raycast);
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p renderer runtime_settings_map_to_renderer_presentation_params -- --nocapture
```

Expected: compilation fails because the renderer-neutral settings types and conversion do not exist.

- [ ] **Step 3: Add renderer-neutral settings types**

Add to `crates/renderer/src/lib.rs` near `PresentationMode`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererPresentationChoice {
    Off,
    Sharp,
    Crt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererLightingChoice {
    Off,
    Ambient,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererShadowChoice {
    Off,
    Raycast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererRuntimeSettings {
    pub presentation: RendererPresentationChoice,
    pub lighting: RendererLightingChoice,
    pub shadows: RendererShadowChoice,
}
```

- [ ] **Step 4: Implement direct renderer settings API**

Add to `impl PresentationParams`:

```rust
fn from_runtime_settings(settings: RendererRuntimeSettings) -> Self {
    Self::new(
        match settings.presentation {
            RendererPresentationChoice::Off => PresentationMode::Off,
            RendererPresentationChoice::Sharp => PresentationMode::Sharp,
            RendererPresentationChoice::Crt => PresentationMode::Crt,
        },
        match settings.lighting {
            RendererLightingChoice::Off => LightingMode::Off,
            RendererLightingChoice::Ambient => LightingMode::Ambient,
            RendererLightingChoice::Dynamic => LightingMode::Dynamic,
        },
        match settings.shadows {
            RendererShadowChoice::Off => ShadowMode::Off,
            RendererShadowChoice::Raycast => ShadowMode::Raycast,
        },
    )
}
```

Add to `impl FrameRenderer`:

```rust
pub fn apply_runtime_settings(&mut self, settings: RendererRuntimeSettings) {
    let next = PresentationParams::from_runtime_settings(settings);
    let presentation_changed = self.presentation_params.presentation != next.presentation;
    self.presentation_params = next;
    if presentation_changed {
        self.rebuild_presentation_bind_groups();
    }
    self.write_cpu_presentation_params();
}
```

- [ ] **Step 5: Add frontend forwarding method**

In `impl NativeFrontend` in `crates/platform/src/lib.rs`, add:

```rust
pub fn apply_runtime_settings(&mut self, settings: RuntimeSettings) {
    let renderer_settings = renderer::RendererRuntimeSettings {
        presentation: match settings.presentation {
            PresentationChoice::Off => renderer::RendererPresentationChoice::Off,
            PresentationChoice::Sharp => renderer::RendererPresentationChoice::Sharp,
            PresentationChoice::Crt => renderer::RendererPresentationChoice::Crt,
        },
        lighting: match settings.lighting {
            LightingChoice::Off => renderer::RendererLightingChoice::Off,
            LightingChoice::Ambient => renderer::RendererLightingChoice::Ambient,
            LightingChoice::Dynamic => renderer::RendererLightingChoice::Dynamic,
        },
        shadows: match settings.shadows {
            ShadowChoice::Off => renderer::RendererShadowChoice::Off,
            ShadowChoice::Raycast => renderer::RendererShadowChoice::Raycast,
        },
    };
    if let Some(renderer) = &mut self.handler.renderer {
        renderer.apply_runtime_settings(renderer_settings);
    }
}
```

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test -p renderer runtime_settings_map_to_renderer_presentation_params -- --nocapture
cargo test -p renderer --lib
cargo test -p platform --lib
```

Expected: all pass.

- [ ] **Step 7: Commit**

Run:

```bash
git add crates/renderer/src/lib.rs crates/platform/src/lib.rs
git commit -m "feat: share menu runtime settings with renderer"
```

---

### Task 4: LTTP Overlay Render Path

**Files:**
- Modify: `crates/renderer/src/lib.rs`
- Modify: `crates/platform/src/lib.rs`

**Interfaces:**
- Consumes: `HostMenuState`.
- Produces: renderer-neutral `MenuOverlayModel`, `FrameRenderer::render_menu_overlay(&mut self, menu: &MenuOverlayModel) -> Result<(), RenderError>`, and `NativeFrontend::present_menu_overlay(&mut self, menu: &HostMenuState)`.

- [ ] **Step 1: Write failing menu text test**

Add to renderer tests:

```rust
#[test]
fn menu_overlay_lines_use_resume_first_play_tab() {
    let menu = MenuOverlayModel::resume_first_play_tab();
    let lines = menu_overlay_lines(&menu);
    assert_eq!(lines[0], "PLAY  VIDEO  CONTROLS  DEV MAP");
    assert_eq!(lines[1], "> RESUME QUEST");
    assert!(lines.iter().any(|line| *line == "DEVELOPER MAP"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p renderer menu_overlay_lines_use_resume_first_play_tab -- --nocapture
```

Expected: fails because `menu_overlay_lines` does not exist.

- [ ] **Step 3: Add text model helper**

In `crates/renderer/src/lib.rs`, add near presentation notice helpers:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOverlayTab {
    Play,
    Video,
    Controls,
    DeveloperMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuOverlayModel {
    pub tab: MenuOverlayTab,
    pub selected_index: usize,
    pub lines: Vec<&'static str>,
}

impl MenuOverlayModel {
    fn resume_first_play_tab() -> Self {
        Self {
            tab: MenuOverlayTab::Play,
            selected_index: 0,
            lines: vec![
                "PLAY  VIDEO  CONTROLS  DEV MAP",
                "> RESUME QUEST",
                "  VIDEO & EFFECTS",
                "  CONTROLS",
                "  DEVELOPER MAP",
                "  SAVE & QUIT",
            ],
        }
    }
}

fn menu_overlay_lines(menu: &MenuOverlayModel) -> Vec<&'static str> {
    match menu.tab {
        MenuOverlayTab::Play => menu.lines.clone(),
        MenuOverlayTab::Video => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> PRESENTATION",
            "  LIGHTING",
            "  SHADOWS",
            "  VIEWPORT",
        ],
        MenuOverlayTab::Controls => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> KEYBOARD",
            "  GAMEPAD",
            "  RESET DEFAULTS",
        ],
        MenuOverlayTab::DeveloperMap => vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> CURATED PRESETS",
            "  ROUTE BOOKMARKS",
            "  LOCKED BROWSER",
        ],
    }
}
```

- [ ] **Step 4: Add simple overlay render methods**

Add to `impl FrameRenderer`:

```rust
pub fn render_menu_overlay(&mut self, menu: &MenuOverlayModel) -> Result<(), RenderError> {
    self.maybe_log_viewport();
    let surface_texture = match self.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(t) => t,
        wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
        wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
            return Err(RenderError::SurfaceReconfigureNeeded);
        }
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
            return Ok(());
        }
        wgpu::CurrentSurfaceTexture::Validation => {
            return Err(RenderError::Fatal(
                "wgpu validation error in get_current_texture".to_string(),
            ));
        }
    };
    let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("host_menu_overlay"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("host_menu_overlay_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.06,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        drop(pass);
    }
    let _ = menu_overlay_lines(menu);
    self.queue.submit([encoder.finish()]);
    surface_texture.present();
    Ok(())
}
```

Complete this task with visible overlay text before committing. Use a CPU-side `256x224` RGBA menu buffer, draw dark panels, gold borders, the `menu_overlay_lines` strings with a small uppercase bitmap font, upload that buffer through the existing texture upload path, and present it with the current blit pipeline.

- [ ] **Step 5: Add frontend wrapper**

In `impl NativeFrontend`, add:

```rust
pub fn present_menu_overlay(&mut self, menu: &HostMenuState) {
    let overlay = renderer::MenuOverlayModel {
        tab: match menu.active_tab() {
            HostMenuTab::Play => renderer::MenuOverlayTab::Play,
            HostMenuTab::Video => renderer::MenuOverlayTab::Video,
            HostMenuTab::Controls => renderer::MenuOverlayTab::Controls,
            HostMenuTab::DeveloperMap => renderer::MenuOverlayTab::DeveloperMap,
        },
        selected_index: 0,
        lines: vec![
            "PLAY  VIDEO  CONTROLS  DEV MAP",
            "> RESUME QUEST",
            "  VIDEO & EFFECTS",
            "  CONTROLS",
            "  DEVELOPER MAP",
            "  SAVE & QUIT",
        ],
    };
    if let Some(renderer) = &mut self.handler.renderer {
        match renderer.render_menu_overlay(&overlay) {
            Ok(()) => {}
            Err(RenderError::SurfaceReconfigureNeeded) => {
                if let Some(window) = &self.handler.window {
                    renderer.resize(window.inner_size());
                }
            }
            Err(RenderError::SurfaceSkipped) => {}
            Err(RenderError::Fatal(e)) => eprintln!("render error: {e}"),
        }
    }
    self.sleep_after_present();
}
```

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test -p renderer menu_overlay_lines_use_resume_first_play_tab -- --nocapture
cargo test -p renderer --lib
cargo test -p platform --lib
```

Expected: all pass.

- [ ] **Step 7: Commit**

Run:

```bash
git add crates/renderer/src/lib.rs crates/platform/src/lib.rs
git commit -m "feat: render host menu overlay"
```

---

### Task 5: Pre-Game and ESC Pause Loop

**Files:**
- Modify: `zelda3-bin/src/main.rs`
- Test: existing `run_frontend_smoke` path or a new smoke mode in `zelda3-bin/src/main.rs`

**Interfaces:**
- Consumes: `HostMenuState`, `HostMenuMode`, `HostMenuAction`, `NativeFrontend::poll_input_with_menu`, `NativeFrontend::drain_host_menu_inputs`, `NativeFrontend::present_menu_overlay`, `NativeFrontend::apply_runtime_settings`.
- Produces: playable loop with pre-game menu and in-game pause behavior.

- [ ] **Step 1: Add a pure helper test for action application**

Add near play-loop helpers in `zelda3-bin/src/main.rs`:

```rust
#[cfg(test)]
mod host_menu_play_tests {
    use super::*;

    #[test]
    fn menu_resume_action_closes_ingame_menu() {
        let mut menu = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::Resume,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(!should_start);
        assert!(!should_quit);
    }

    #[test]
    fn menu_start_action_closes_pregame_menu_and_starts_game() {
        let mut menu = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::StartQuest,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(should_start);
        assert!(!should_quit);
    }
}
```

- [ ] **Step 2: Add test-only helper**

Add above the test module:

```rust
#[cfg(test)]
fn apply_host_menu_action_for_test(
    menu: &mut HostMenuState,
    action: HostMenuAction,
    should_start: &mut bool,
    should_quit: &mut bool,
) {
    match action {
        HostMenuAction::Resume => menu.close(),
        HostMenuAction::StartQuest => {
            *should_start = true;
            menu.close();
        }
        HostMenuAction::SaveAndQuit => *should_quit = true,
        HostMenuAction::SetPresentation(_)
        | HostMenuAction::SetLighting(_)
        | HostMenuAction::SetShadows(_)
        | HostMenuAction::WarpToVerifiedDestination(_) => {}
    }
}
```

- [ ] **Step 3: Run focused tests**

Run:

```bash
cargo test -p zelda3-bin host_menu_play_tests -- --nocapture
```

Expected: compile failures until imports are added, then pass after the next step.

- [ ] **Step 4: Import platform menu types**

Change the `use platform` import in `zelda3-bin/src/main.rs` to:

```rust
use platform::{
    HostMenuAction, HostMenuMode, HostMenuState, NativeFrontend, NativeFrontendOptions,
};
```

Keep `Frontend` in the import list if non-playable paths still need the trait:

```rust
use platform::{Frontend, HostMenuAction, HostMenuMode, HostMenuState, NativeFrontend, NativeFrontendOptions};
```

- [ ] **Step 5: Integrate pre-game menu in `run_play_with_state`**

After frontend creation and before the loop, add:

```rust
let mut game_started = false;
let mut host_menu = HostMenuState::new(
    HostMenuMode::PreGame,
    developer_destinations::developer_destinations(),
);
```

At the top of the loop, before `let live_input = ...`, add:

```rust
frontend.set_menu_open(host_menu.is_open());
for input in frontend.drain_host_menu_inputs() {
    if let Some(action) = host_menu.handle_input(input) {
        match action {
            HostMenuAction::Resume => host_menu.close(),
            HostMenuAction::StartQuest => {
                game_started = true;
                host_menu.close();
            }
            HostMenuAction::SaveAndQuit => break,
            HostMenuAction::SetPresentation(_)
            | HostMenuAction::SetLighting(_)
            | HostMenuAction::SetShadows(_) => {
                frontend.apply_runtime_settings(host_menu.runtime_settings());
            }
            HostMenuAction::WarpToVerifiedDestination(id) => {
                eprintln!("developer destination selected: {id}");
            }
        }
    }
}
if host_menu.is_open() {
    frontend.present_menu_overlay(&host_menu);
    continue;
}
if !game_started {
    game_started = true;
}
```

Replace:

```rust
let live_input = frontend.poll_input();
```

with:

```rust
let live_input = frontend.poll_input_with_menu(host_menu.is_open());
```

- [ ] **Step 6: Open menu on ESC during gameplay**

After polling input events and before running `zelda_run_frame`, process drained inputs when the menu is closed:

```rust
for input in frontend.drain_host_menu_inputs() {
    if matches!(input, platform::HostMenuInput::Cancel) {
        host_menu.open_ingame();
    }
}
if host_menu.is_open() {
    frontend.present_menu_overlay(&host_menu);
    continue;
}
```

Ensure this block runs before `game.zelda_run_frame(...)`.

- [ ] **Step 7: Run build and smoke**

Run:

```bash
cargo test -p zelda3-bin host_menu_play_tests -- --nocapture
cargo check -p zelda3-bin
cargo run -p zelda3-bin -- --frontend-smoke 2
```

Expected: tests pass, check passes, frontend smoke exits after 2 frames when no menu is forced. If pre-game menu blocks smoke, add `ZELDA3_SKIP_HOST_MENU=1` for smoke/test paths and document it in the code.

- [ ] **Step 8: Commit**

Run:

```bash
git add zelda3-bin/src/main.rs
git commit -m "feat: add pregame and escape host menu flow"
```

---

### Task 6: Audio Ducking While Menu Is Open

**Files:**
- Modify: `crates/platform/src/lib.rs`

**Interfaces:**
- Consumes: `NativeFrontend::set_menu_open`.
- Produces: mixer-level menu volume control through `AudioOutput::set_volume_scale(&mut self, scale: f32)`.

- [ ] **Step 1: Write sample scaling test**

Add to platform tests:

```rust
#[test]
fn audio_ducking_scales_i16_samples() {
    assert_eq!(scale_i16_sample(10_000, 0.25), 2_500);
    assert_eq!(scale_i16_sample(-10_000, 0.25), -2_500);
    assert_eq!(scale_i16_sample(i16::MAX, 2.0), i16::MAX);
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p platform audio_ducking_scales_i16_samples -- --nocapture
```

Expected: fails because `scale_i16_sample` does not exist.

- [ ] **Step 3: Add shared volume scale**

Change `AudioOutput`:

```rust
volume_scale: Arc<Mutex<f32>>,
```

In `AudioOutput::new`, create:

```rust
let volume_scale = Arc::new(Mutex::new(1.0));
```

Pass `Arc::clone(&volume_scale)` into `build_output_stream`.

- [ ] **Step 4: Add scaling helper and setter**

Add:

```rust
fn scale_i16_sample(sample: i16, scale: f32) -> i16 {
    let scaled = (sample as f32 * scale).round();
    scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i16
}
```

Add in `impl AudioOutput`:

```rust
fn set_volume_scale(&mut self, scale: f32) {
    if let Ok(mut value) = self.volume_scale.lock() {
        *value = scale.clamp(0.0, 1.0);
    }
}
```

- [ ] **Step 5: Apply volume in the audio callback**

Change `build_output_stream` signature:

```rust
fn build_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    queue: Arc<Mutex<VecDeque<i16>>>,
    volume_scale: Arc<Mutex<f32>>,
) -> Result<cpal::Stream, String>
```

Inside the callback, read:

```rust
let scale = volume_scale.lock().map(|value| *value).unwrap_or(1.0);
```

Replace:

```rust
let value = queue.pop_front().unwrap_or(0);
*sample = T::from_i16(value);
```

with:

```rust
let value = scale_i16_sample(queue.pop_front().unwrap_or(0), scale);
*sample = T::from_i16(value);
```

- [ ] **Step 6: Wire menu state to audio volume**

In `NativeFrontend::set_menu_open`, add:

```rust
if let Some(audio) = &mut self.audio {
    audio.set_volume_scale(if open { 0.35 } else { 1.0 });
}
```

- [ ] **Step 7: Run tests**

Run:

```bash
cargo test -p platform audio_ducking_scales_i16_samples -- --nocapture
cargo test -p platform --lib
```

Expected: all pass.

- [ ] **Step 8: Commit**

Run:

```bash
git add crates/platform/src/lib.rs
git commit -m "feat: duck audio under host menu"
```

---

### Task 7: Developer Destination Manifest

**Files:**
- Create: `zelda3-bin/src/developer_destinations.rs`
- Modify: `zelda3-bin/src/main.rs`

**Interfaces:**
- Consumes: `DeveloperDestination`.
- Produces: `developer_destinations::developer_destinations() -> Vec<DeveloperDestination>`.

- [ ] **Step 1: Write destination tests**

Create `zelda3-bin/src/developer_destinations.rs` with tests first:

```rust
use platform::{DeveloperDestination, DeveloperDestinationStatus};

pub fn developer_destinations() -> Vec<DeveloperDestination> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_manifest_has_verified_route_bookmarks() {
        let destinations = developer_destinations();
        assert!(destinations.iter().any(|destination| {
            destination.id == "route-start"
                && destination.status == DeveloperDestinationStatus::Verified
        }));
        assert!(destinations.iter().any(|destination| {
            destination.id == "unverified-room-browser"
                && destination.status == DeveloperDestinationStatus::Locked
        }));
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test -p zelda3-bin destination_manifest_has_verified_route_bookmarks -- --nocapture
```

Expected: fails because the manifest is empty or module is not imported.

- [ ] **Step 3: Register module**

In `zelda3-bin/src/main.rs`, add near other module declarations if any exist, or near the top-level imports:

```rust
mod developer_destinations;
```

- [ ] **Step 4: Fill the v1 manifest**

Replace `developer_destinations()` with:

```rust
pub fn developer_destinations() -> Vec<DeveloperDestination> {
    vec![
        DeveloperDestination::verified(
            "route-start",
            "Route Start",
            "saves/zelda3-combined-route.sav frame 0",
        ),
        DeveloperDestination::verified(
            "route-file-select",
            "File Select",
            "standard route first menu segment",
        ),
        DeveloperDestination::verified(
            "route-late-checkpoint",
            "Late Route Checkpoint",
            "standard route checkpoint frame 1045813",
        ),
        DeveloperDestination::locked(
            "unverified-overworld-browser",
            "Overworld Browser",
            "requires verified overworld initializer",
        ),
        DeveloperDestination::locked(
            "unverified-room-browser",
            "Dungeon Room Browser",
            "requires verified dungeon room initializer",
        ),
    ]
}
```

- [ ] **Step 5: Run tests**

Run:

```bash
cargo test -p zelda3-bin destination_manifest_has_verified_route_bookmarks -- --nocapture
cargo check -p zelda3-bin
```

Expected: pass.

- [ ] **Step 6: Commit**

Run:

```bash
git add zelda3-bin/src/main.rs zelda3-bin/src/developer_destinations.rs
git commit -m "feat: add verified developer destinations"
```

---

### Task 8: Final Verification And Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-06-25-host-overlay-menu-design.md` only if implementation discovered a corrected constraint.

**Interfaces:**
- Consumes: completed menu implementation.
- Produces: user-facing run notes and final verified state.

- [ ] **Step 1: Add README usage text**

Add a short section to `README.md` near current runtime controls:

```markdown
### Host Menu

The native frontend opens an LTTP-styled host menu before the game starts.
During play, press `ESC` to pause the game and open the same menu. Gameplay
state stops advancing while the menu is open; currently playing music continues
softly. Presentation controls are available from the Video tab, and F6/F7/F8
remain shortcuts for presentation, lighting, and shadows.

The Developer Map starts with verified route bookmarks and curated presets.
Unverified overworld screens and dungeon rooms are shown as locked until their
initialization paths are tested.
```

- [ ] **Step 2: Run full local verification**

Run:

```bash
cargo test -p platform --lib
cargo test -p renderer --lib
cargo test -p zelda3-bin host_menu -- --nocapture
cargo check -p zelda3-bin
cargo run -p zelda3-bin -- --frontend-smoke 2
git diff --check
```

Expected: all commands pass.

- [ ] **Step 3: Run parity gate**

Run:

```bash
python3 scripts/full_parity.py --with-snes9x
```

Expected: all standard route checks pass.

- [ ] **Step 4: Commit**

Run:

```bash
git add README.md docs/superpowers/specs/2026-06-25-host-overlay-menu-design.md
git commit -m "docs: document host menu controls"
```

- [ ] **Step 5: Report final outcome**

Include:

```text
Implemented host overlay menu plan.
Verified:
- cargo test -p platform --lib
- cargo test -p renderer --lib
- cargo test -p zelda3-bin host_menu -- --nocapture
- cargo check -p zelda3-bin
- cargo run -p zelda3-bin -- --frontend-smoke 2
- git diff --check
- python3 scripts/full_parity.py --with-snes9x
```

If any verification cannot be run, state the exact command and blocker.
