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
pub enum DeveloperMapPanel {
    Overview,
    CuratedPresets,
    RouteBookmarks,
    LockedBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostMenuInput {
    Up,
    Down,
    Left,
    Right,
    PreviousTab,
    NextTab,
    CyclePresentation,
    CycleLighting,
    CycleShadows,
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
pub enum ViewportChoice {
    Integer,
    Fit,
    Stretch,
}

impl ViewportChoice {
    fn next(self) -> Self {
        match self {
            Self::Integer => Self::Fit,
            Self::Fit => Self::Stretch,
            Self::Stretch => Self::Integer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub presentation: PresentationChoice,
    pub lighting: LightingChoice,
    pub shadows: ShadowChoice,
    pub viewport: ViewportChoice,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            presentation: PresentationChoice::Off,
            lighting: LightingChoice::Off,
            shadows: ShadowChoice::Off,
            viewport: ViewportChoice::Integer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlsPanel {
    Keyboard,
    Gamepad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperDestinationStatus {
    Verified,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperDestinationKind {
    CuratedPreset,
    RouteBookmark,
    LockedBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeveloperThumbnail {
    RouteStart,
    FileSelect,
    Sanctuary,
    LateDungeon,
    DevRoom,
    LockedOverworld,
    LockedDungeon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperDestination {
    pub id: &'static str,
    pub label: &'static str,
    pub menu_label: &'static str,
    pub location: &'static str,
    pub provenance: &'static str,
    pub detail: &'static str,
    pub status: DeveloperDestinationStatus,
    pub kind: DeveloperDestinationKind,
    pub thumbnail: DeveloperThumbnail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperCurrentLocation {
    pub label: String,
    pub location: String,
    pub detail: String,
    pub thumbnail: DeveloperThumbnail,
}

impl DeveloperDestination {
    pub fn verified(id: &'static str, label: &'static str, provenance: &'static str) -> Self {
        Self::route_bookmark(
            id,
            label,
            label,
            "ROUTE",
            provenance,
            provenance,
            DeveloperThumbnail::RouteStart,
        )
    }

    pub fn curated_preset(
        id: &'static str,
        label: &'static str,
        menu_label: &'static str,
        location: &'static str,
        provenance: &'static str,
        detail: &'static str,
        thumbnail: DeveloperThumbnail,
    ) -> Self {
        Self {
            id,
            label,
            menu_label,
            location,
            provenance,
            detail,
            status: DeveloperDestinationStatus::Verified,
            kind: DeveloperDestinationKind::CuratedPreset,
            thumbnail,
        }
    }

    pub fn route_bookmark(
        id: &'static str,
        label: &'static str,
        menu_label: &'static str,
        location: &'static str,
        provenance: &'static str,
        detail: &'static str,
        thumbnail: DeveloperThumbnail,
    ) -> Self {
        Self {
            id,
            label,
            menu_label,
            location,
            provenance,
            detail,
            status: DeveloperDestinationStatus::Verified,
            kind: DeveloperDestinationKind::RouteBookmark,
            thumbnail,
        }
    }

    pub fn locked(id: &'static str, label: &'static str, provenance: &'static str) -> Self {
        let thumbnail = if id.contains("overworld") {
            DeveloperThumbnail::LockedOverworld
        } else {
            DeveloperThumbnail::LockedDungeon
        };
        Self::locked_browser(
            id, label, label, "LOCKED", provenance, provenance, thumbnail,
        )
    }

    pub fn locked_browser(
        id: &'static str,
        label: &'static str,
        menu_label: &'static str,
        location: &'static str,
        provenance: &'static str,
        detail: &'static str,
        thumbnail: DeveloperThumbnail,
    ) -> Self {
        Self {
            id,
            label,
            menu_label,
            location,
            provenance,
            detail,
            status: DeveloperDestinationStatus::Locked,
            kind: DeveloperDestinationKind::LockedBrowser,
            thumbnail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostMenuAction {
    StartQuest,
    Resume,
    Quit,
    SaveAndQuit,
    SetPresentation(PresentationChoice),
    SetLighting(LightingChoice),
    SetShadows(ShadowChoice),
    SetViewport(ViewportChoice),
    ShowControls(ControlsPanel),
    ResetRuntimeSettings(RuntimeSettings),
    WarpToVerifiedDestination(&'static str),
}

#[derive(Debug, Clone)]
pub struct HostMenuState {
    mode: HostMenuMode,
    open: bool,
    active_tab: HostMenuTab,
    selected_index: usize,
    runtime_settings: RuntimeSettings,
    controls_panel: Option<ControlsPanel>,
    developer_map_panel: DeveloperMapPanel,
    developer_destinations: Vec<DeveloperDestination>,
    current_developer_location: Option<DeveloperCurrentLocation>,
}

impl HostMenuState {
    pub fn new(mode: HostMenuMode, developer_destinations: Vec<DeveloperDestination>) -> Self {
        Self {
            mode,
            open: true,
            active_tab: HostMenuTab::Play,
            selected_index: 0,
            runtime_settings: RuntimeSettings::default(),
            controls_panel: None,
            developer_map_panel: DeveloperMapPanel::Overview,
            developer_destinations,
            current_developer_location: None,
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
        if tab == HostMenuTab::DeveloperMap {
            self.developer_map_panel = DeveloperMapPanel::Overview;
        }
    }

    pub fn runtime_settings(&self) -> RuntimeSettings {
        self.runtime_settings
    }

    pub fn controls_panel(&self) -> Option<ControlsPanel> {
        self.controls_panel
    }

    pub fn developer_map_panel(&self) -> DeveloperMapPanel {
        self.developer_map_panel
    }

    pub fn set_current_developer_location(&mut self, location: DeveloperCurrentLocation) {
        self.current_developer_location = Some(location);
    }

    pub fn developer_map_items(&self) -> Vec<&'static str> {
        match self.developer_map_panel {
            DeveloperMapPanel::Overview => {
                vec!["Curated Presets", "Route Bookmarks", "Locked Browser"]
            }
            DeveloperMapPanel::CuratedPresets => self
                .developer_destinations
                .iter()
                .filter(|destination| destination.kind == DeveloperDestinationKind::CuratedPreset)
                .map(|destination| destination.label)
                .collect(),
            DeveloperMapPanel::RouteBookmarks => self
                .developer_destinations
                .iter()
                .filter(|destination| destination.kind == DeveloperDestinationKind::RouteBookmark)
                .map(|destination| destination.label)
                .collect(),
            DeveloperMapPanel::LockedBrowser => self
                .developer_destinations
                .iter()
                .filter(|destination| destination.kind == DeveloperDestinationKind::LockedBrowser)
                .map(|destination| destination.label)
                .collect(),
        }
    }

    pub fn selected_developer_destination(&self) -> Option<&DeveloperDestination> {
        let kind = match self.developer_map_panel {
            DeveloperMapPanel::Overview => return None,
            DeveloperMapPanel::CuratedPresets => DeveloperDestinationKind::CuratedPreset,
            DeveloperMapPanel::RouteBookmarks => DeveloperDestinationKind::RouteBookmark,
            DeveloperMapPanel::LockedBrowser => DeveloperDestinationKind::LockedBrowser,
        };
        self.developer_destinations
            .iter()
            .filter(|destination| destination.kind == kind)
            .nth(self.selected_index)
    }

    pub fn developer_thumbnail(&self) -> Option<DeveloperThumbnail> {
        self.selected_developer_destination()
            .map(|destination| destination.thumbnail)
            .or_else(|| {
                (self.active_tab == HostMenuTab::DeveloperMap
                    && self.developer_map_panel == DeveloperMapPanel::Overview)
                    .then(|| {
                        self.current_developer_location
                            .as_ref()
                            .map(|location| location.thumbnail)
                    })
                    .flatten()
            })
    }

    pub fn developer_detail_lines(&self) -> Vec<String> {
        let Some(destination) = self.selected_developer_destination() else {
            return match self.developer_map_panel {
                DeveloperMapPanel::Overview => self
                    .current_developer_location
                    .as_ref()
                    .map(|location| {
                        vec![
                            location.label.clone(),
                            location.location.clone(),
                            "CURRENT".to_string(),
                            location.detail.clone(),
                        ]
                    })
                    .unwrap_or_else(|| {
                        vec![
                            "SELECT A PANEL".to_string(),
                            "VERIFIED ONLY".to_string(),
                            "LOCKED SHOW WHY".to_string(),
                        ]
                    }),
                _ => vec!["NO DESTINATION".to_string()],
            };
        };
        vec![
            destination.menu_label.to_string(),
            destination.location.to_string(),
            match destination.status {
                DeveloperDestinationStatus::Verified => "VERIFIED".to_string(),
                DeveloperDestinationStatus::Locked => "LOCKED".to_string(),
            },
            destination.detail.to_string(),
            destination.provenance.to_string(),
        ]
    }

    pub fn cycle_presentation(&mut self) -> HostMenuAction {
        self.runtime_settings.presentation = self.runtime_settings.presentation.next();
        HostMenuAction::SetPresentation(self.runtime_settings.presentation)
    }

    pub fn cycle_lighting(&mut self) -> HostMenuAction {
        self.runtime_settings.lighting = self.runtime_settings.lighting.next();
        HostMenuAction::SetLighting(self.runtime_settings.lighting)
    }

    pub fn cycle_shadows(&mut self) -> HostMenuAction {
        self.runtime_settings.shadows = self.runtime_settings.shadows.next();
        HostMenuAction::SetShadows(self.runtime_settings.shadows)
    }

    pub fn cycle_viewport(&mut self) -> HostMenuAction {
        self.runtime_settings.viewport = self.runtime_settings.viewport.next();
        HostMenuAction::SetViewport(self.runtime_settings.viewport)
    }

    pub fn reset_runtime_settings(&mut self) -> HostMenuAction {
        self.runtime_settings = RuntimeSettings::default();
        HostMenuAction::ResetRuntimeSettings(self.runtime_settings)
    }

    pub fn selected_label(&self) -> &'static str {
        match self.active_tab {
            HostMenuTab::Play => self.play_items()[self.selected_index],
            HostMenuTab::Video => self.video_items()[self.selected_index],
            HostMenuTab::Controls => self.controls_items()[self.selected_index],
            HostMenuTab::DeveloperMap => self
                .developer_map_items()
                .get(self.selected_index)
                .copied()
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
            HostMenuInput::CyclePresentation => Some(self.cycle_presentation()),
            HostMenuInput::CycleLighting => Some(self.cycle_lighting()),
            HostMenuInput::CycleShadows => Some(self.cycle_shadows()),
            HostMenuInput::Cancel => {
                if self.active_tab == HostMenuTab::DeveloperMap
                    && self.developer_map_panel != DeveloperMapPanel::Overview
                {
                    let overview_index = match self.developer_map_panel {
                        DeveloperMapPanel::Overview => 0,
                        DeveloperMapPanel::CuratedPresets => 0,
                        DeveloperMapPanel::RouteBookmarks => 1,
                        DeveloperMapPanel::LockedBrowser => 2,
                    };
                    self.developer_map_panel = DeveloperMapPanel::Overview;
                    self.selected_index = overview_index;
                    return None;
                }
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
                (HostMenuMode::PreGame, 4) => Some(HostMenuAction::Quit),
                (HostMenuMode::InGame, 4) => Some(HostMenuAction::SaveAndQuit),
                _ => None,
            },
            HostMenuTab::Video => match self.selected_index {
                0 => Some(self.cycle_presentation()),
                1 => Some(self.cycle_lighting()),
                2 => Some(self.cycle_shadows()),
                3 => Some(self.cycle_viewport()),
                _ => None,
            },
            HostMenuTab::Controls => match self.selected_index {
                0 => {
                    self.controls_panel = Some(ControlsPanel::Keyboard);
                    Some(HostMenuAction::ShowControls(ControlsPanel::Keyboard))
                }
                1 => {
                    self.controls_panel = Some(ControlsPanel::Gamepad);
                    Some(HostMenuAction::ShowControls(ControlsPanel::Gamepad))
                }
                2 => Some(self.reset_runtime_settings()),
                _ => None,
            },
            HostMenuTab::DeveloperMap => self.confirm_developer_map_selected(),
        }
    }

    fn confirm_developer_map_selected(&mut self) -> Option<HostMenuAction> {
        match self.developer_map_panel {
            DeveloperMapPanel::Overview => {
                match self.selected_index {
                    0 => self.developer_map_panel = DeveloperMapPanel::CuratedPresets,
                    1 => self.developer_map_panel = DeveloperMapPanel::RouteBookmarks,
                    2 => self.developer_map_panel = DeveloperMapPanel::LockedBrowser,
                    _ => {}
                }
                self.selected_index = 0;
                None
            }
            DeveloperMapPanel::CuratedPresets => self
                .developer_destinations
                .iter()
                .filter(|destination| destination.kind == DeveloperDestinationKind::CuratedPreset)
                .nth(self.selected_index)
                .filter(|destination| destination.status == DeveloperDestinationStatus::Verified)
                .map(|destination| HostMenuAction::WarpToVerifiedDestination(destination.id)),
            DeveloperMapPanel::RouteBookmarks => self
                .developer_destinations
                .iter()
                .filter(|destination| destination.kind == DeveloperDestinationKind::RouteBookmark)
                .nth(self.selected_index)
                .filter(|destination| destination.status == DeveloperDestinationStatus::Verified)
                .map(|destination| HostMenuAction::WarpToVerifiedDestination(destination.id)),
            DeveloperMapPanel::LockedBrowser => None,
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
        let next = match self.active_tab {
            HostMenuTab::Play => HostMenuTab::Video,
            HostMenuTab::Video => HostMenuTab::Controls,
            HostMenuTab::Controls => HostMenuTab::DeveloperMap,
            HostMenuTab::DeveloperMap => HostMenuTab::Play,
        };
        self.set_active_tab(next);
    }

    fn previous_tab(&mut self) {
        let previous = match self.active_tab {
            HostMenuTab::Play => HostMenuTab::DeveloperMap,
            HostMenuTab::Video => HostMenuTab::Play,
            HostMenuTab::Controls => HostMenuTab::Video,
            HostMenuTab::DeveloperMap => HostMenuTab::Controls,
        };
        self.set_active_tab(previous);
    }

    fn current_len(&self) -> usize {
        match self.active_tab {
            HostMenuTab::Play => self.play_items().len(),
            HostMenuTab::Video => self.video_items().len(),
            HostMenuTab::Controls => self.controls_items().len(),
            HostMenuTab::DeveloperMap => self.developer_map_items().len().max(1),
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
                DeveloperDestination::curated_preset(
                    "sanctuary",
                    "Sanctuary",
                    "SANCTUARY",
                    "ROOM 0050",
                    "12000",
                    "ROUTE FRAME",
                    DeveloperThumbnail::Sanctuary,
                ),
                DeveloperDestination::route_bookmark(
                    "sanctuary",
                    "Sanctuary",
                    "SANCTUARY",
                    "ROOM 0050",
                    "12000",
                    "ROUTE FRAME",
                    DeveloperThumbnail::Sanctuary,
                ),
                DeveloperDestination::locked_browser(
                    "room-003f",
                    "Room 003F",
                    "ROOM 003F",
                    "ROOM 003F",
                    "unverified room init",
                    "UNVERIFIED INIT",
                    DeveloperThumbnail::LockedDungeon,
                ),
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
    fn direct_runtime_setting_api_cycles_each_setting() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());

        assert_eq!(
            state.cycle_presentation(),
            HostMenuAction::SetPresentation(PresentationChoice::Sharp)
        );
        assert_eq!(
            state.cycle_lighting(),
            HostMenuAction::SetLighting(LightingChoice::Ambient)
        );
        assert_eq!(
            state.cycle_shadows(),
            HostMenuAction::SetShadows(ShadowChoice::Raycast)
        );
        assert_eq!(
            state.runtime_settings(),
            RuntimeSettings {
                presentation: PresentationChoice::Sharp,
                lighting: LightingChoice::Ambient,
                shadows: ShadowChoice::Raycast,
                viewport: ViewportChoice::Integer,
            }
        );
    }

    #[test]
    fn runtime_shortcut_inputs_cycle_shared_settings() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());

        assert_eq!(
            state.handle_input(HostMenuInput::CyclePresentation),
            Some(HostMenuAction::SetPresentation(PresentationChoice::Sharp))
        );
        assert_eq!(
            state.handle_input(HostMenuInput::CycleLighting),
            Some(HostMenuAction::SetLighting(LightingChoice::Ambient))
        );
        assert_eq!(
            state.handle_input(HostMenuInput::CycleShadows),
            Some(HostMenuAction::SetShadows(ShadowChoice::Raycast))
        );
        assert_eq!(
            state.runtime_settings(),
            RuntimeSettings {
                presentation: PresentationChoice::Sharp,
                lighting: LightingChoice::Ambient,
                shadows: ShadowChoice::Raycast,
                viewport: ViewportChoice::Integer,
            }
        );
    }

    #[test]
    fn video_menu_confirmation_matches_direct_runtime_setting_api() {
        let mut menu_state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        let mut direct_state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        menu_state.set_active_tab(HostMenuTab::Video);

        assert_eq!(
            menu_state.handle_input(HostMenuInput::Confirm),
            Some(direct_state.cycle_presentation())
        );
        assert_eq!(
            menu_state.runtime_settings(),
            direct_state.runtime_settings()
        );

        menu_state.handle_input(HostMenuInput::Down);
        assert_eq!(
            menu_state.handle_input(HostMenuInput::Confirm),
            Some(direct_state.cycle_lighting())
        );
        assert_eq!(
            menu_state.runtime_settings(),
            direct_state.runtime_settings()
        );

        menu_state.handle_input(HostMenuInput::Down);
        assert_eq!(
            menu_state.handle_input(HostMenuInput::Confirm),
            Some(direct_state.cycle_shadows())
        );
        assert_eq!(
            menu_state.runtime_settings(),
            direct_state.runtime_settings()
        );

        menu_state.handle_input(HostMenuInput::Down);
        assert_eq!(
            menu_state.handle_input(HostMenuInput::Confirm),
            Some(direct_state.cycle_viewport())
        );
        assert_eq!(
            menu_state.runtime_settings(),
            direct_state.runtime_settings()
        );
    }

    #[test]
    fn controls_menu_exposes_help_and_reset_actions() {
        let mut state = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        state.set_active_tab(HostMenuTab::Controls);

        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::ShowControls(ControlsPanel::Keyboard))
        );
        assert_eq!(state.controls_panel(), Some(ControlsPanel::Keyboard));

        state.handle_input(HostMenuInput::Down);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::ShowControls(ControlsPanel::Gamepad))
        );
        assert_eq!(state.controls_panel(), Some(ControlsPanel::Gamepad));

        state.cycle_presentation();
        state.cycle_lighting();
        state.cycle_shadows();
        state.cycle_viewport();
        assert_ne!(state.runtime_settings(), RuntimeSettings::default());

        state.handle_input(HostMenuInput::Down);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::ResetRuntimeSettings(
                RuntimeSettings::default()
            ))
        );
        assert_eq!(state.runtime_settings(), RuntimeSettings::default());
    }

    #[test]
    fn pregame_quit_uses_distinct_action_from_ingame_save_and_quit() {
        let mut pregame = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        let mut ingame = HostMenuState::new(HostMenuMode::InGame, Vec::new());

        for _ in 0..4 {
            pregame.handle_input(HostMenuInput::Down);
            ingame.handle_input(HostMenuInput::Down);
        }

        assert_eq!(
            pregame.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::Quit)
        );
        assert_eq!(
            ingame.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::SaveAndQuit)
        );
    }

    #[test]
    fn developer_map_only_activates_verified_destinations() {
        let mut state = state_with_destinations();
        state.set_active_tab(HostMenuTab::DeveloperMap);
        assert_eq!(state.selected_label(), "Curated Presets");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
        assert_eq!(
            state.developer_map_panel(),
            DeveloperMapPanel::CuratedPresets
        );

        state.handle_input(HostMenuInput::Down);
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::WarpToVerifiedDestination("sanctuary"))
        );

        assert_eq!(state.handle_input(HostMenuInput::Cancel), None);
        state.handle_input(HostMenuInput::Down);
        assert_eq!(state.selected_label(), "Route Bookmarks");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
        assert_eq!(state.selected_label(), "Sanctuary");
        assert_eq!(
            state.handle_input(HostMenuInput::Confirm),
            Some(HostMenuAction::WarpToVerifiedDestination("sanctuary"))
        );

        assert_eq!(state.handle_input(HostMenuInput::Cancel), None);
        assert_eq!(state.selected_label(), "Route Bookmarks");
        state.handle_input(HostMenuInput::Down);
        assert_eq!(state.selected_label(), "Locked Browser");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
        assert_eq!(state.selected_label(), "Room 003F");
        assert_eq!(state.handle_input(HostMenuInput::Confirm), None);
    }

    #[test]
    fn developer_map_exposes_selected_destination_details_and_thumbnail() {
        let mut state = state_with_destinations();
        state.set_active_tab(HostMenuTab::DeveloperMap);
        state.handle_input(HostMenuInput::Confirm);

        let destination = state
            .selected_developer_destination()
            .expect("curated preset should expose selected destination metadata");
        assert_eq!(destination.id, "sanctuary");
        assert_eq!(destination.kind, DeveloperDestinationKind::CuratedPreset);
        assert_eq!(destination.status, DeveloperDestinationStatus::Verified);
        assert_eq!(destination.location, "ROOM 0050");
        assert_eq!(destination.thumbnail, DeveloperThumbnail::Sanctuary);

        let detail_lines = state.developer_detail_lines();
        assert!(detail_lines.contains(&"SANCTUARY".to_string()));
        assert!(detail_lines.contains(&"ROOM 0050".to_string()));
        assert!(detail_lines.contains(&"VERIFIED".to_string()));
        assert!(detail_lines.contains(&"ROUTE FRAME".to_string()));
        assert!(detail_lines.contains(&"12000".to_string()));
    }

    #[test]
    fn developer_map_overview_shows_current_location_marker_when_available() {
        let mut state = state_with_destinations();
        state.set_active_tab(HostMenuTab::DeveloperMap);
        state.set_current_developer_location(DeveloperCurrentLocation {
            label: "CURRENT LOC".to_string(),
            location: "ROOM 0050".to_string(),
            detail: "FRAME 12000".to_string(),
            thumbnail: DeveloperThumbnail::Sanctuary,
        });

        assert_eq!(state.developer_map_panel(), DeveloperMapPanel::Overview);
        assert_eq!(
            state.developer_detail_lines(),
            vec![
                "CURRENT LOC".to_string(),
                "ROOM 0050".to_string(),
                "CURRENT".to_string(),
                "FRAME 12000".to_string(),
            ]
        );
        assert_eq!(
            state.developer_thumbnail(),
            Some(DeveloperThumbnail::Sanctuary)
        );
    }
}
