use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

const ASSERT_ONLY_SPRITE_TYPES: &[u8] = &[0x2d, 0xb8];
const NULL_SPRITE_TYPES: &[u8] = &[0x03];
const UNPLACED_OR_UNSPAWNED_SPRITE_ALIASES: &[u8] = &[
    // Pull-switch handler aliases; C assets place 0x04 and 0x06.
    0x05, 0x07,
    // Roller, antifairy, and laser-eye handler aliases with no C asset
    // placement or dynamic spawn site. Their placed siblings stay required.
    0x5e, 0x77, 0x98,
    // Somaria-platform aliases. The source-backed routeable platform is 0xed;
    // 0xee is the placed movable mantle.
    0xef, 0xf0, 0xf1,
];
const EMPTY_OR_UNUSED_ANCILLA_TYPES: &[u8] = &[
    0x03,
    // Dispatch aliases for `Ancilla33_BlastWallExplosion` that have no
    // source-backed spawn site. The live blast-wall path uses ancilla 0x33.
    0x0e, 0x0f, 0x10, 0x12, 0x14, 0x25,
];
const FRAME_SAMPLED_SPECIAL_OVERWORLD_SCREENS: &[u16] = &[
    // C `LoadOverworldFromDungeon` can persist special-overworld exit rooms
    // 0x180, 0x182, and 0x189 as frame-state screens 0x80, 0x81, and 0x88.
    // Other 0x80+ overworld assets are map/overlay payloads or special-exit
    // geometry selected by `dungeon_room_index`, not stable `overworld_screen`
    // frame values.
    0x0080, 0x0081, 0x0088,
];
const MAX_FRAME_SAMPLED_INDOOR_ROOM: u16 = 0x0127;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RouteCoverage {
    pub frames: u32,
    pub last_frame: u32,
    pub main_modules: BTreeSet<u8>,
    pub module_states: BTreeSet<ModuleState>,
    pub indoor_rooms: BTreeSet<u16>,
    pub overworld_screens: BTreeSet<u16>,
    pub sprite_types: BTreeSet<u8>,
    pub ancilla_types: BTreeSet<u8>,
    pub active_items: BTreeSet<u8>,
    #[serde(default)]
    pub first_seen: BTreeMap<String, BTreeMap<String, u32>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub provenance: BTreeMap<String, BTreeMap<String, String>>,
}

impl RouteCoverage {
    pub fn record(&mut self, frame: CoverageFrame) {
        let frame_number = frame.frame;
        self.frames = self.frames.saturating_add(1);
        self.last_frame = self.last_frame.max(frame_number);
        self.main_modules.insert(frame.main_module);
        self.record_first_seen(
            "main_modules",
            format!("0x{:02x}", frame.main_module),
            frame_number,
        );
        let module_state = ModuleState {
            main: frame.main_module,
            sub: frame.submodule,
            subsub: frame.subsubmodule,
        };
        self.module_states.insert(module_state);
        self.record_first_seen(
            "module_states",
            format_module_state(&module_state),
            frame_number,
        );
        if let Some(room) = frame.indoor_room {
            self.indoor_rooms.insert(room);
            self.record_first_seen("indoor_rooms", format!("0x{room:04x}"), frame_number);
        }
        if let Some(screen) = frame.overworld_screen {
            self.overworld_screens.insert(screen);
            self.record_first_seen("overworld_screens", format!("0x{screen:04x}"), frame_number);
        }
        for &ty in &frame.sprite_types {
            self.record_first_seen("sprite_types", format!("0x{ty:02x}"), frame_number);
        }
        for &ty in frame.ancilla_types.iter().filter(|&&ty| ty != 0) {
            self.record_first_seen("ancilla_types", format!("0x{ty:02x}"), frame_number);
        }
        if let Some(item) = frame.active_item.filter(|&item| item != 0) {
            self.record_first_seen("active_items", format!("0x{item:02x}"), frame_number);
        }
        self.sprite_types.extend(frame.sprite_types);
        self.ancilla_types
            .extend(frame.ancilla_types.into_iter().filter(|&ty| ty != 0));
        if let Some(item) = frame.active_item.filter(|&item| item != 0) {
            self.active_items.insert(item);
        }
    }

    pub fn source_seeded_from_c_assets(c_root: &Path) -> Self {
        let mut coverage = Self::default();
        if let Some(indoor_rooms) = asset_id_paths(c_root, "assets/dungeon", "dungeon") {
            for (room, path) in indoor_rooms {
                if room <= MAX_FRAME_SAMPLED_INDOOR_ROOM {
                    coverage.record_source_seeded_indoor_room(room, path);
                }
            }
        }
        if let Some(overworld_screens) = asset_id_paths(c_root, "assets/overworld", "overworld") {
            for (screen, path) in overworld_screens {
                if screen < 0x0080 || FRAME_SAMPLED_SPECIAL_OVERWORLD_SCREENS.contains(&screen) {
                    coverage.record_source_seeded_overworld_screen(screen, path);
                }
            }
        }
        coverage
    }

    pub fn merge(&mut self, other: &Self) {
        self.frames = self.frames.saturating_add(other.frames);
        self.last_frame = self.last_frame.max(other.last_frame);
        self.main_modules.extend(other.main_modules.iter().copied());
        self.module_states
            .extend(other.module_states.iter().copied());
        self.indoor_rooms.extend(other.indoor_rooms.iter().copied());
        self.overworld_screens
            .extend(other.overworld_screens.iter().copied());
        self.sprite_types.extend(other.sprite_types.iter().copied());
        self.ancilla_types
            .extend(other.ancilla_types.iter().copied());
        self.active_items.extend(other.active_items.iter().copied());
        for (category, entries) in &other.first_seen {
            for (value, frame) in entries {
                self.record_first_seen(category, value.clone(), *frame);
            }
        }
        for (category, entries) in &other.provenance {
            let category_entry = self.provenance.entry(category.clone()).or_default();
            for (value, source) in entries {
                category_entry
                    .entry(value.clone())
                    .or_insert_with(|| source.clone());
            }
        }
    }

    pub fn report(&self) -> CoverageReport {
        self.report_with_universe(&CoverageUniverse::standard())
    }

    pub fn report_with_universe(&self, universe: &CoverageUniverse) -> CoverageReport {
        CoverageReport {
            frames: self.frames,
            last_frame: self.last_frame,
            categories: vec![
                category_u8_with_first_seen(
                    "main_modules",
                    &self.main_modules,
                    &universe.main_modules,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_module_states(
                    "module_states",
                    &self.module_states,
                    &universe.module_states,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "sprite_types",
                    &self.sprite_types,
                    &universe.sprite_types,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "ancilla_types",
                    &self.ancilla_types,
                    &universe.ancilla_types,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u16_with_first_seen(
                    "indoor_rooms",
                    &self.indoor_rooms,
                    &universe.indoor_rooms,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u16_with_first_seen(
                    "overworld_screens",
                    &self.overworld_screens,
                    &universe.overworld_screens,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "active_items",
                    &self.active_items,
                    &universe.active_items,
                    &self.first_seen,
                    &self.provenance,
                ),
            ],
        }
    }

    pub fn route_evidence_report_with_universe(
        &self,
        universe: &CoverageUniverse,
    ) -> CoverageReport {
        let main_modules = self.route_evidence_u8("main_modules", &self.main_modules);
        let module_states = self.route_evidence_module_states("module_states", &self.module_states);
        let sprite_types = self.route_evidence_u8("sprite_types", &self.sprite_types);
        let ancilla_types = self.route_evidence_u8("ancilla_types", &self.ancilla_types);
        let indoor_rooms = self.route_evidence_u16("indoor_rooms", &self.indoor_rooms);
        let overworld_screens =
            self.route_evidence_u16("overworld_screens", &self.overworld_screens);
        let active_items = self.route_evidence_u8("active_items", &self.active_items);

        CoverageReport {
            frames: self.frames,
            last_frame: self.last_frame,
            categories: vec![
                category_u8_with_first_seen(
                    "main_modules",
                    &main_modules,
                    &universe.main_modules,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_module_states(
                    "module_states",
                    &module_states,
                    &universe.module_states,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "sprite_types",
                    &sprite_types,
                    &universe.sprite_types,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "ancilla_types",
                    &ancilla_types,
                    &universe.ancilla_types,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u16_with_first_seen(
                    "indoor_rooms",
                    &indoor_rooms,
                    &universe.indoor_rooms,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u16_with_first_seen(
                    "overworld_screens",
                    &overworld_screens,
                    &universe.overworld_screens,
                    &self.first_seen,
                    &self.provenance,
                ),
                category_u8_with_first_seen(
                    "active_items",
                    &active_items,
                    &universe.active_items,
                    &self.first_seen,
                    &self.provenance,
                ),
            ],
        }
    }

    pub fn route_worklist_with_universe(
        &self,
        universe: &CoverageUniverse,
        c_root: &Path,
    ) -> RouteCoverageWorklist {
        let route_report = self.route_evidence_report_with_universe(universe);
        let dungeon_sources = dungeon_stair_or_hole_sources(c_root);
        let indoor_rooms = missed_u16_values(&route_report, "indoor_rooms")
            .into_iter()
            .map(|room| {
                let mut strategies = dungeon_direct_entrance_strategies(c_root, room);
                if let Some(sources) = dungeon_sources.get(&room) {
                    strategies.extend(sources.iter().cloned().map(|mut strategy| {
                        if let Some(source_id) = strategy.source_id_u16 {
                            strategy.source_id = Some(format!("0x{source_id:04x}"));
                            strategy.route_source_covered = Some(
                                self.has_first_seen("indoor_rooms", &format!("0x{source_id:04x}")),
                            );
                        }
                        strategy.source_id_u16 = None;
                        strategy
                    }));
                }
                if strategies.is_empty() {
                    strategies.push(RouteWorklistStrategy::unclassified());
                }
                RouteWorklistEntry {
                    id: format!("0x{room:04x}"),
                    source: self.source_for("indoor_rooms", &format!("0x{room:04x}")),
                    strategies,
                }
            })
            .collect();
        let overworld_screens = missed_u16_values(&route_report, "overworld_screens")
            .into_iter()
            .map(|screen| {
                let mut strategies = overworld_entrance_strategies(c_root, screen);
                strategies.extend(
                    overworld_travel_source_strategies(c_root, screen)
                        .into_iter()
                        .map(|mut strategy| {
                            if let Some(source_id) = strategy.source_id_u16 {
                                strategy.source_id = Some(format!("0x{source_id:04x}"));
                                strategy.route_source_covered = Some(self.has_first_seen(
                                    "overworld_screens",
                                    &format!("0x{source_id:04x}"),
                                ));
                            }
                            strategy.source_id_u16 = None;
                            strategy
                        }),
                );
                if strategies.is_empty() {
                    strategies.push(RouteWorklistStrategy::unclassified());
                }
                RouteWorklistEntry {
                    id: format!("0x{screen:04x}"),
                    source: self.source_for("overworld_screens", &format!("0x{screen:04x}")),
                    strategies,
                }
            })
            .collect();
        RouteCoverageWorklist {
            indoor_rooms,
            overworld_screens,
        }
    }

    fn record_source_seeded_indoor_room(&mut self, room: u16, path: String) {
        self.indoor_rooms.insert(room);
        self.record_provenance(
            "indoor_rooms",
            format!("0x{room:04x}"),
            format!("source-seeded:{path}"),
        );
    }

    fn record_source_seeded_overworld_screen(&mut self, screen: u16, path: String) {
        self.overworld_screens.insert(screen);
        self.record_provenance(
            "overworld_screens",
            format!("0x{screen:04x}"),
            format!("source-seeded:{path}"),
        );
    }

    fn record_first_seen(&mut self, category: &str, value: String, frame: u32) {
        let entry = self
            .first_seen
            .entry(category.to_string())
            .or_default()
            .entry(value)
            .or_insert(frame);
        *entry = (*entry).min(frame);
    }

    fn record_provenance(&mut self, category: &str, value: String, source: String) {
        self.provenance
            .entry(category.to_string())
            .or_default()
            .entry(value)
            .or_insert(source);
    }

    fn route_evidence_u8(&self, category: &str, hits: &BTreeSet<u8>) -> BTreeSet<u8> {
        hits.iter()
            .copied()
            .filter(|value| self.has_route_evidence(category, &format!("0x{value:02x}")))
            .collect()
    }

    fn route_evidence_u16(&self, category: &str, hits: &BTreeSet<u16>) -> BTreeSet<u16> {
        hits.iter()
            .copied()
            .filter(|value| self.has_route_evidence(category, &format!("0x{value:04x}")))
            .collect()
    }

    fn route_evidence_module_states(
        &self,
        category: &str,
        hits: &BTreeSet<ModuleState>,
    ) -> BTreeSet<ModuleState> {
        hits.iter()
            .copied()
            .filter(|value| self.has_route_evidence(category, &format_module_state(value)))
            .collect()
    }

    fn has_route_evidence(&self, category: &str, value: &str) -> bool {
        self.first_seen
            .get(category)
            .is_some_and(|entries| entries.contains_key(value))
            || !self
                .provenance
                .get(category)
                .is_some_and(|entries| entries.contains_key(value))
    }

    fn has_first_seen(&self, category: &str, value: &str) -> bool {
        self.first_seen
            .get(category)
            .is_some_and(|entries| entries.contains_key(value))
    }

    fn source_for(&self, category: &str, value: &str) -> Option<String> {
        self.provenance
            .get(category)?
            .get(value)?
            .strip_prefix("source-seeded:")
            .or_else(|| {
                self.provenance
                    .get(category)?
                    .get(value)
                    .map(String::as_str)
            })
            .map(str::to_string)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageFrame {
    pub frame: u32,
    pub main_module: u8,
    pub submodule: u8,
    pub subsubmodule: u8,
    pub indoor_room: Option<u16>,
    pub overworld_screen: Option<u16>,
    pub sprite_types: Vec<u8>,
    pub ancilla_types: Vec<u8>,
    pub active_item: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ModuleState {
    pub main: u8,
    pub sub: u8,
    pub subsub: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageUniverse {
    pub main_modules: Vec<u8>,
    pub module_states: Vec<ModuleState>,
    pub sprite_types: Vec<u8>,
    pub ancilla_types: Vec<u8>,
    pub indoor_rooms: Vec<u16>,
    pub overworld_screens: Vec<u16>,
    pub active_items: Vec<u8>,
}

impl CoverageUniverse {
    pub fn standard() -> Self {
        Self {
            // C `kMainRouting[28]` includes 0x0a/0x0c/0x0d, but 0x0a is
            // a duplicate overworld-load slot with no source-backed assignment,
            // and 0x0c/0x0d route to assert-only unknown modules.
            main_modules: (0x00..=0x1b)
                .filter(|module| !matches!(module, 0x0a | 0x0c | 0x0d))
                .collect(),
            module_states: vec![],
            // C `kSpriteActiveRoutines[243]` dispatches types 0x00..=0xf2.
            // Exclude explicit NULL/assert-only slots; they are not routeable
            // gameplay surfaces and should not block a 100% route coverage gate.
            sprite_types: (0x00..=0xf2)
                .filter(|ty| {
                    !NULL_SPRITE_TYPES.contains(ty) && !ASSERT_ONLY_SPRITE_TYPES.contains(ty)
                })
                .filter(|ty| !UNPLACED_OR_UNSPAWNED_SPRITE_ALIASES.contains(ty))
                .collect(),
            // C `kAncilla_Funcs[67]` dispatches types 0x01..=0x43.
            // Exclude the empty/no-op slot, explicit unused assert slots, and
            // unreachable handler aliases that have no C source assignment.
            ancilla_types: (0x01..=0x43)
                .filter(|ty| !EMPTY_OR_UNUSED_ANCILLA_TYPES.contains(ty))
                .collect(),
            indoor_rooms: (0x0000..=MAX_FRAME_SAMPLED_INDOOR_ROOM).collect(),
            overworld_screens: (0x0000..=0x007f).collect(),
            // Old-style inventory is compiled in (`kNewStyleInventory = 0`),
            // so `Hud_LookupInventoryItem` can produce active items 0x01..=0x14.
            active_items: (0x01..=0x14).collect(),
        }
    }

    pub fn from_c_assets_or_standard(c_root: &Path) -> Self {
        let mut universe = Self::standard();
        if let Some(indoor_rooms) = asset_ids(&c_root.join("assets/dungeon"), "dungeon") {
            universe.indoor_rooms = indoor_rooms
                .into_iter()
                .filter(|room| *room <= MAX_FRAME_SAMPLED_INDOOR_ROOM)
                .collect();
        }
        if let Some(overworld_screens) = asset_ids(&c_root.join("assets/overworld"), "overworld") {
            universe.overworld_screens = overworld_screens
                .into_iter()
                .filter(|screen| {
                    *screen < 0x0080 || FRAME_SAMPLED_SPECIAL_OVERWORLD_SCREENS.contains(screen)
                })
                .collect();
        }
        universe
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageReport {
    pub frames: u32,
    pub last_frame: u32,
    pub categories: Vec<CoverageCategoryReport>,
}

impl CoverageReport {
    pub fn category(&self, name: &str) -> Option<&CoverageCategoryReport> {
        self.categories
            .iter()
            .find(|category| category.name == name)
    }

    pub fn to_text(&self) -> String {
        let mut out = format!("frames: {}\nlast_frame: {}\n", self.frames, self.last_frame);
        for category in &self.categories {
            out.push_str(&format!(
                "{}: {}/{} ({:.1}%)",
                category.name, category.hit, category.expected, category.percent
            ));
            if !category.missed.is_empty() {
                out.push_str(" missed: ");
                out.push_str(&summarize_values(&category.missed));
            }
            out.push('\n');
        }
        out
    }

    pub fn delta_from(&self, base: &CoverageReport) -> CoverageDeltaReport {
        let categories: Vec<CoverageDeltaCategoryReport> = self
            .categories
            .iter()
            .filter_map(|category| {
                let base_category = base.category(&category.name)?;
                let base_missed: BTreeSet<String> = base_category.missed.iter().cloned().collect();
                let newly_covered: Vec<String> = category
                    .covered
                    .iter()
                    .filter(|value| base_missed.contains(*value))
                    .cloned()
                    .collect();
                Some(CoverageDeltaCategoryReport {
                    name: category.name.clone(),
                    newly_covered,
                })
            })
            .collect();
        let total_newly_covered = categories
            .iter()
            .map(|category| category.newly_covered.len())
            .sum();
        CoverageDeltaReport {
            base_frames: base.frames,
            base_last_frame: base.last_frame,
            candidate_frames: self.frames,
            candidate_last_frame: self.last_frame,
            total_newly_covered,
            categories,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageCategoryReport {
    pub name: String,
    pub hit: usize,
    pub expected: usize,
    pub percent: f64,
    pub covered: Vec<String>,
    pub missed: Vec<String>,
    #[serde(default)]
    pub first_seen: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub provenance: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageDeltaReport {
    pub base_frames: u32,
    pub base_last_frame: u32,
    pub candidate_frames: u32,
    pub candidate_last_frame: u32,
    pub total_newly_covered: usize,
    pub categories: Vec<CoverageDeltaCategoryReport>,
}

impl CoverageDeltaReport {
    pub fn to_text(&self, base_label: &str) -> String {
        let mut out = format!(
            "coverage delta vs {base_label}\nbase_frames: {}\ncandidate_frames: {}\nnewly_covered_total: {}\n",
            self.base_frames, self.candidate_frames, self.total_newly_covered
        );
        for category in &self.categories {
            out.push_str(&format!(
                "{}: +{}",
                category.name,
                category.newly_covered.len()
            ));
            if !category.newly_covered.is_empty() {
                out.push(' ');
                out.push_str(&summarize_values(&category.newly_covered));
            }
            out.push('\n');
        }
        out
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageDeltaCategoryReport {
    pub name: String,
    pub newly_covered: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteCoverageWorklist {
    pub indoor_rooms: Vec<RouteWorklistEntry>,
    pub overworld_screens: Vec<RouteWorklistEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteWorklistEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub strategies: Vec<RouteWorklistStrategy>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteWorklistStrategy {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip)]
    source_id_u16: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrance_index: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrance_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_source_covered: Option<bool>,
}

impl RouteWorklistStrategy {
    fn unclassified() -> Self {
        Self {
            kind: "unclassified".to_string(),
            source_id: None,
            source_id_u16: None,
            source: None,
            via: None,
            entrance_index: None,
            entrance_id: None,
            route_source_covered: None,
        }
    }
}

fn missed_u16_values(report: &CoverageReport, category: &str) -> Vec<u16> {
    report
        .category(category)
        .map(|category| {
            category
                .missed
                .iter()
                .filter_map(|value| parse_hex_u16(value))
                .collect()
        })
        .unwrap_or_default()
}

fn dungeon_direct_entrance_strategies(c_root: &Path, room: u16) -> Vec<RouteWorklistStrategy> {
    let relative = format!("assets/dungeon/dungeon-{room}.yaml");
    let Ok(text) = std::fs::read_to_string(c_root.join(&relative)) else {
        return Vec::new();
    };
    let mut strategies: Vec<RouteWorklistStrategy> = text
        .lines()
        .filter_map(|line| {
            let entrance_index = line
                .trim()
                .strip_prefix("- entrance_index:")
                .and_then(parse_trimmed_u16)?;
            Some(RouteWorklistStrategy {
                kind: "direct_entrance".to_string(),
                source_id: None,
                source_id_u16: None,
                source: Some(relative.clone()),
                via: None,
                entrance_index: Some(entrance_index),
                entrance_id: None,
                route_source_covered: None,
            })
        })
        .collect();
    strategies.sort_by_key(|strategy| strategy.entrance_index.unwrap_or(u16::MAX));
    strategies
}

fn dungeon_stair_or_hole_sources(c_root: &Path) -> BTreeMap<u16, Vec<RouteWorklistStrategy>> {
    let mut sources: BTreeMap<u16, Vec<RouteWorklistStrategy>> = BTreeMap::new();
    let Some(assets) = asset_id_paths(c_root, "assets/dungeon", "dungeon") else {
        return sources;
    };
    for (source_room, relative) in assets {
        let Ok(text) = std::fs::read_to_string(c_root.join(&relative)) else {
            continue;
        };
        for line in text.lines() {
            let trimmed = line.trim();
            let Some((name, rest)) = trimmed.split_once(':') else {
                continue;
            };
            if !is_dungeon_route_link(name) {
                continue;
            }
            let Some(target_room) = parse_bracket_first_u16(rest) else {
                continue;
            };
            sources
                .entry(target_room)
                .or_default()
                .push(RouteWorklistStrategy {
                    kind: "stair_or_hole_source".to_string(),
                    source_id: None,
                    source_id_u16: Some(source_room),
                    source: Some(relative.clone()),
                    via: Some(name.to_string()),
                    entrance_index: None,
                    entrance_id: None,
                    route_source_covered: None,
                });
        }
    }
    for strategies in sources.values_mut() {
        strategies.sort_by(|left, right| {
            left.source_id_u16
                .cmp(&right.source_id_u16)
                .then_with(|| left.via.cmp(&right.via))
        });
    }
    sources
}

fn overworld_entrance_strategies(c_root: &Path, screen: u16) -> Vec<RouteWorklistStrategy> {
    let relative = format!("assets/overworld/overworld-{screen}.yaml");
    let Ok(text) = std::fs::read_to_string(c_root.join(&relative)) else {
        return Vec::new();
    };
    let mut strategies: Vec<RouteWorklistStrategy> = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("- {") || !trimmed.contains("entrance_id:") {
                return None;
            }
            Some(RouteWorklistStrategy {
                kind: "overworld_entrance".to_string(),
                source_id: None,
                source_id_u16: None,
                source: Some(relative.clone()),
                via: None,
                entrance_index: inline_u16(trimmed, "index"),
                entrance_id: inline_u16(trimmed, "entrance_id"),
                route_source_covered: None,
            })
        })
        .collect();
    strategies.sort_by_key(|strategy| strategy.entrance_index.unwrap_or(u16::MAX));
    strategies
}

fn overworld_travel_source_strategies(c_root: &Path, screen: u16) -> Vec<RouteWorklistStrategy> {
    let relative = format!("assets/overworld/overworld-{screen}.yaml");
    let Ok(text) = std::fs::read_to_string(c_root.join(&relative)) else {
        return Vec::new();
    };
    let mut strategies: Vec<RouteWorklistStrategy> = text
        .lines()
        .filter_map(|line| {
            let source_screen = line
                .trim()
                .strip_prefix("- whirlpool_src_area:")
                .and_then(parse_trimmed_u16)?;
            Some(RouteWorklistStrategy {
                kind: "travel_source".to_string(),
                source_id: None,
                source_id_u16: Some(source_screen),
                source: Some(relative.clone()),
                via: Some("whirlpool_src_area".to_string()),
                entrance_index: None,
                entrance_id: None,
                route_source_covered: None,
            })
        })
        .collect();
    strategies.sort_by_key(|strategy| strategy.source_id_u16.unwrap_or(u16::MAX));
    strategies
}

fn is_dungeon_route_link(name: &str) -> bool {
    (name.starts_with("hole") || name.starts_with("stair")) && name.ends_with("_dest")
}

fn parse_bracket_first_u16(text: &str) -> Option<u16> {
    text.split_once('[')?
        .1
        .split_once(',')?
        .0
        .trim()
        .parse()
        .ok()
}

fn inline_u16(text: &str, key: &str) -> Option<u16> {
    text.split(',').find_map(|part| {
        let (left, right) = part.split_once(':')?;
        let left = left
            .trim()
            .trim_start_matches('-')
            .trim()
            .trim_start_matches('{')
            .trim();
        (left == key)
            .then(|| right.trim().trim_end_matches('}').parse().ok())
            .flatten()
    })
}

fn parse_trimmed_u16(text: &str) -> Option<u16> {
    text.trim().parse().ok()
}

fn parse_hex_u16(text: &str) -> Option<u16> {
    u16::from_str_radix(text.strip_prefix("0x")?, 16).ok()
}

fn category_u8_with_first_seen(
    name: &str,
    hits: &BTreeSet<u8>,
    expected: &[u8],
    first_seen: &BTreeMap<String, BTreeMap<String, u32>>,
    provenance: &BTreeMap<String, BTreeMap<String, String>>,
) -> CoverageCategoryReport {
    let expected_set: BTreeSet<u8> = expected.iter().copied().collect();
    let hit = expected_set.intersection(hits).count();
    let covered: Vec<String> = expected_set
        .intersection(hits)
        .map(|value| format!("0x{value:02x}"))
        .collect();
    CoverageCategoryReport {
        name: name.to_string(),
        hit,
        expected: expected_set.len(),
        percent: percent(hit, expected_set.len()),
        first_seen: first_seen_for(name, &covered, first_seen),
        provenance: provenance_for(name, &covered, provenance),
        covered,
        missed: expected_set
            .difference(hits)
            .map(|value| format!("0x{value:02x}"))
            .collect(),
    }
}

fn category_u16_with_first_seen(
    name: &str,
    hits: &BTreeSet<u16>,
    expected: &[u16],
    first_seen: &BTreeMap<String, BTreeMap<String, u32>>,
    provenance: &BTreeMap<String, BTreeMap<String, String>>,
) -> CoverageCategoryReport {
    let expected_set: BTreeSet<u16> = expected.iter().copied().collect();
    let hit = expected_set.intersection(hits).count();
    let covered: Vec<String> = expected_set
        .intersection(hits)
        .map(|value| format!("0x{value:04x}"))
        .collect();
    CoverageCategoryReport {
        name: name.to_string(),
        hit,
        expected: expected_set.len(),
        percent: percent(hit, expected_set.len()),
        first_seen: first_seen_for(name, &covered, first_seen),
        provenance: provenance_for(name, &covered, provenance),
        covered,
        missed: expected_set
            .difference(hits)
            .map(|value| format!("0x{value:04x}"))
            .collect(),
    }
}

fn category_module_states(
    name: &str,
    hits: &BTreeSet<ModuleState>,
    expected: &[ModuleState],
    first_seen: &BTreeMap<String, BTreeMap<String, u32>>,
    provenance: &BTreeMap<String, BTreeMap<String, String>>,
) -> CoverageCategoryReport {
    let expected_set: BTreeSet<ModuleState> = if expected.is_empty() {
        hits.iter().copied().collect()
    } else {
        expected.iter().copied().collect()
    };
    let hit = expected_set.intersection(hits).count();
    let covered: Vec<String> = expected_set
        .intersection(hits)
        .map(format_module_state)
        .collect();
    CoverageCategoryReport {
        name: name.to_string(),
        hit,
        expected: expected_set.len(),
        percent: percent(hit, expected_set.len()),
        first_seen: first_seen_for(name, &covered, first_seen),
        provenance: provenance_for(name, &covered, provenance),
        covered,
        missed: expected_set
            .difference(hits)
            .map(format_module_state)
            .collect(),
    }
}

fn first_seen_for(
    name: &str,
    covered: &[String],
    first_seen: &BTreeMap<String, BTreeMap<String, u32>>,
) -> BTreeMap<String, u32> {
    let Some(category) = first_seen.get(name) else {
        return BTreeMap::new();
    };
    covered
        .iter()
        .filter_map(|value| category.get(value).map(|frame| (value.clone(), *frame)))
        .collect()
}

fn provenance_for(
    name: &str,
    covered: &[String],
    provenance: &BTreeMap<String, BTreeMap<String, String>>,
) -> BTreeMap<String, String> {
    let Some(category) = provenance.get(name) else {
        return BTreeMap::new();
    };
    covered
        .iter()
        .filter_map(|value| {
            category
                .get(value)
                .map(|source| (value.clone(), source.clone()))
        })
        .collect()
}

fn format_module_state(state: &ModuleState) -> String {
    format!(
        "0x{:02x}:0x{:02x}:0x{:02x}",
        state.main, state.sub, state.subsub
    )
}

fn summarize_values(values: &[String]) -> String {
    const MAX_INLINE_VALUES: usize = 24;
    if values.len() <= MAX_INLINE_VALUES {
        return values.join(",");
    }

    format!(
        "{} ... +{} more (see report JSON)",
        values[..MAX_INLINE_VALUES].join(","),
        values.len() - MAX_INLINE_VALUES
    )
}

fn percent(hit: usize, expected: usize) -> f64 {
    if expected == 0 {
        100.0
    } else {
        (hit as f64 / expected as f64) * 100.0
    }
}

fn asset_ids(dir: &Path, prefix: &str) -> Option<Vec<u16>> {
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(dir).ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let Some(id) = asset_file_id(file_name, prefix) else {
            continue;
        };
        ids.push(id);
    }
    ids.sort_unstable();
    ids.dedup();
    (!ids.is_empty()).then_some(ids)
}

fn asset_id_paths(c_root: &Path, relative_dir: &str, prefix: &str) -> Option<Vec<(u16, String)>> {
    let dir = c_root.join(relative_dir);
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir).ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let Some(id) = asset_file_id(file_name, prefix) else {
            continue;
        };
        entries.push((id, format!("{relative_dir}/{file_name}")));
    }
    entries.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    entries.dedup_by_key(|entry| entry.0);
    (!entries.is_empty()).then_some(entries)
}

fn asset_file_id(file_name: &str, prefix: &str) -> Option<u16> {
    file_name
        .strip_prefix(prefix)
        .and_then(|name| name.strip_prefix('-'))
        .and_then(|name| name.strip_suffix(".yaml"))
        .and_then(|name| name.parse::<u16>().ok())
}
