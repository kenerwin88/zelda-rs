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
            HostMenuMode::PreGame => &[
                "Start Quest",
                "Video & Effects",
                "Controls",
                "Developer Map",
                "Quit",
            ],
            HostMenuMode::InGame => &[
                "Resume Quest",
                "Video & Effects",
                "Controls",
                "Developer Map",
                "Save & Quit",
            ],
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
        assert_eq!(
            state.handle_input(HostMenuInput::Cancel),
            Some(HostMenuAction::Resume)
        );
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
        assert_eq!(
            state.runtime_settings().presentation,
            PresentationChoice::Sharp
        );
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
