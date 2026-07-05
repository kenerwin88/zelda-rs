#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererMode {
    Classic,
    ModernCompare,
    Modern,
}

impl RendererMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("modern-compare") => Self::ModernCompare,
            Some("modern") => Self::Modern,
            _ => Self::Classic,
        }
    }
}

pub const DEFAULT_RENDERER_ENV: &str = "assets-variant-gpu";
pub const DEFAULT_VARIANT_ATLAS_OFF_RENDERER_ENV: &str = "assets-anim-gpu";

/// Effective renderer for paths that honor `ZELDA3_RENDERER`. Unset defaults to
/// `assets-variant-gpu` so stable base-art draws use the RGBA variant atlas and
/// dynamic rows fall back to the indexed GPU compositor. Set
/// `ZELDA3_VARIANT_ATLAS=off` to keep the older full indexed GPU path. Explicit
/// `assets-anim` keeps the CPU atlas compositor, and `classic` opts back into
/// the wgpu PPU path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveRendererMode<'a> {
    name: &'a str,
}

impl<'a> EffectiveRendererMode<'a> {
    pub fn from_name(name: &'a str) -> Self {
        Self { name }
    }

    pub fn from_env_value(value: Option<&'a str>, variant_atlas_setting: Option<&str>) -> Self {
        Self {
            name: renderer_env_or_default(value, variant_atlas_setting),
        }
    }

    pub fn name(self) -> &'a str {
        self.name
    }

    pub fn uses_gpu_assets(self) -> bool {
        self.name == "assets-anim-gpu" || self.name == "assets-variant-gpu"
    }

    pub fn uses_source_atlas(self) -> bool {
        self.name == "assets-anim" || self.uses_gpu_assets()
    }

    pub fn uses_variant_atlas(self) -> bool {
        self.name == "assets-variant-gpu"
    }
}

pub fn default_renderer_env_for_variant_setting(value: Option<&str>) -> &'static str {
    match value {
        Some(value) if value.eq_ignore_ascii_case("off") => DEFAULT_VARIANT_ATLAS_OFF_RENDERER_ENV,
        _ => DEFAULT_RENDERER_ENV,
    }
}

pub fn renderer_env_or_default<'a>(
    value: Option<&'a str>,
    variant_atlas_setting: Option<&str>,
) -> &'a str {
    match value {
        Some(value) => value,
        None => default_renderer_env_for_variant_setting(variant_atlas_setting),
    }
}

pub fn source_atlas_renderer_mode(mode: &str) -> bool {
    EffectiveRendererMode::from_name(mode).uses_source_atlas()
}

pub fn variant_atlas_renderer_mode(mode: &str) -> bool {
    EffectiveRendererMode::from_name(mode).uses_variant_atlas()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_mode_parse_defaults_to_classic() {
        assert_eq!(RendererMode::parse(None), RendererMode::Classic);
        assert_eq!(
            RendererMode::parse(Some("modern-compare")),
            RendererMode::ModernCompare
        );
        assert_eq!(RendererMode::parse(Some("modern")), RendererMode::Modern);
        assert_eq!(RendererMode::parse(Some("garbage")), RendererMode::Classic);
    }

    #[test]
    fn unset_renderer_defaults_to_full_gpu_asset_path() {
        let default_mode =
            EffectiveRendererMode::from_name(default_renderer_env_for_variant_setting(None));
        assert_eq!(default_mode.name(), "assets-variant-gpu");
        assert!(default_mode.uses_gpu_assets());
        assert!(default_mode.uses_source_atlas());
        assert!(default_mode.uses_variant_atlas());

        let indexed_gpu_mode =
            EffectiveRendererMode::from_name(default_renderer_env_for_variant_setting(Some("off")));
        assert_eq!(indexed_gpu_mode.name(), "assets-anim-gpu");
        assert!(indexed_gpu_mode.uses_gpu_assets());
        assert!(indexed_gpu_mode.uses_source_atlas());
        assert!(!indexed_gpu_mode.uses_variant_atlas());
    }

    #[test]
    fn explicit_renderer_values_preserve_opt_out_modes() {
        let cpu_atlas_mode = EffectiveRendererMode::from_name("assets-anim");
        assert!(!cpu_atlas_mode.uses_gpu_assets());
        assert!(cpu_atlas_mode.uses_source_atlas());
        assert!(!cpu_atlas_mode.uses_variant_atlas());

        assert!(source_atlas_renderer_mode("assets-variant-gpu"));
        assert!(source_atlas_renderer_mode("assets-anim-gpu"));
        assert!(source_atlas_renderer_mode("assets-anim"));
        assert!(!source_atlas_renderer_mode("classic"));
        assert!(variant_atlas_renderer_mode("assets-variant-gpu"));
        assert!(!variant_atlas_renderer_mode("assets-anim-gpu"));
        assert!(!variant_atlas_renderer_mode("assets-anim"));
        assert_eq!(
            renderer_env_or_default(Some("assets-anim"), None),
            "assets-anim",
            "explicit CPU atlas mode remains an opt-out"
        );
        assert_eq!(
            renderer_env_or_default(Some("classic"), Some("off")),
            "classic",
            "explicit classic mode remains an opt-out"
        );
    }
}
