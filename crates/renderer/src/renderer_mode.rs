use std::env;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererMode {
    ModernCompare,
    Modern,
}

impl RendererMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("modern-compare") => Self::ModernCompare,
            _ => Self::Modern,
        }
    }

    pub fn from_effective_mode(mode: EffectiveRendererMode<'_>) -> Self {
        if mode.uses_source_atlas() {
            Self::Modern
        } else {
            Self::parse(Some(mode.name()))
        }
    }

    pub fn from_effective_env() -> Self {
        Self::from_effective_mode(EffectiveRendererMode::from_env())
    }

    pub fn live_gpu_asset_from_env() -> Result<Self, String> {
        EffectiveRendererMode::live_gpu_asset_from_env().map(Self::from_effective_mode)
    }
}

pub const DEFAULT_RENDERER_ENV: &str = "assets-variant-gpu";

/// Effective renderer for paths that honor `ZELDA3_RENDERER`. Unset defaults to
/// `assets-variant-gpu` so stable base-art draws use the RGBA variant atlas and
/// dynamic effects route through shader/material metadata. Explicit
/// `assets-anim-gpu` keeps the older full indexed GPU path and `assets-anim`
/// keeps the CPU atlas compositor. (The classic wgpu PPU path was removed.)
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

impl EffectiveRendererMode<'static> {
    pub fn from_env() -> Self {
        let renderer_env = env::var("ZELDA3_RENDERER").ok();
        let variant_atlas_env = env::var("ZELDA3_VARIANT_ATLAS").ok();
        Self {
            name: renderer_env_or_default_static(
                renderer_env.as_deref(),
                variant_atlas_env.as_deref(),
            ),
        }
    }

    pub fn live_gpu_asset_from_env() -> Result<Self, String> {
        let renderer_env = env::var("ZELDA3_RENDERER").ok();
        live_gpu_asset_mode_from_env_value(renderer_env.as_deref())
    }
}

pub fn live_gpu_asset_mode_from_env_value(
    value: Option<&str>,
) -> Result<EffectiveRendererMode<'static>, String> {
    match value {
        None | Some("assets-variant-gpu") => Ok(EffectiveRendererMode::from_name(
            "assets-variant-gpu",
        )),
        Some(value) => Err(format!(
            "ZELDA3_RENDERER={value:?} is diagnostic-only for live play; expected assets-variant-gpu"
        )),
    }
}

pub fn default_renderer_env_for_variant_setting(_value: Option<&str>) -> &'static str {
    DEFAULT_RENDERER_ENV
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

fn renderer_env_or_default_static(
    value: Option<&str>,
    variant_atlas_setting: Option<&str>,
) -> &'static str {
    match value {
        Some("assets-variant-gpu") => "assets-variant-gpu",
        Some("assets-anim-gpu") => "assets-anim-gpu",
        Some("assets-anim") => "assets-anim",
        Some("modern") => "modern",
        Some("modern-compare") => "modern-compare",
        Some(_) | None => default_renderer_env_for_variant_setting(variant_atlas_setting),
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
    fn renderer_mode_parse_defaults_to_modern() {
        assert_eq!(RendererMode::parse(None), RendererMode::Modern);
        assert_eq!(
            RendererMode::parse(Some("modern-compare")),
            RendererMode::ModernCompare
        );
        assert_eq!(RendererMode::parse(Some("modern")), RendererMode::Modern);
        assert_eq!(RendererMode::parse(Some("garbage")), RendererMode::Modern);
    }

    #[test]
    fn renderer_mode_from_effective_mode_owns_asset_mode_mapping() {
        assert_eq!(
            RendererMode::from_effective_mode(EffectiveRendererMode::from_name(
                "assets-variant-gpu"
            )),
            RendererMode::Modern
        );
        assert_eq!(
            RendererMode::from_effective_mode(EffectiveRendererMode::from_name("assets-anim-gpu")),
            RendererMode::Modern
        );
        assert_eq!(
            RendererMode::from_effective_mode(EffectiveRendererMode::from_name("assets-anim")),
            RendererMode::Modern
        );
        assert_eq!(
            RendererMode::from_effective_mode(EffectiveRendererMode::from_name("modern-compare")),
            RendererMode::ModernCompare
        );
    }

    #[test]
    fn unset_renderer_defaults_to_full_gpu_asset_path() {
        let default_mode =
            EffectiveRendererMode::from_name(default_renderer_env_for_variant_setting(None));
        assert_eq!(default_mode.name(), "assets-variant-gpu");
        assert!(default_mode.uses_gpu_assets());
        assert!(default_mode.uses_source_atlas());
        assert!(default_mode.uses_variant_atlas());

        let legacy_variant_off_mode =
            EffectiveRendererMode::from_name(default_renderer_env_for_variant_setting(Some("off")));
        assert_eq!(legacy_variant_off_mode.name(), "assets-variant-gpu");
        assert!(legacy_variant_off_mode.uses_gpu_assets());
        assert!(legacy_variant_off_mode.uses_source_atlas());
        assert!(legacy_variant_off_mode.uses_variant_atlas());
        assert_eq!(
            EffectiveRendererMode::from_env_value(None, Some("off")).name(),
            "assets-variant-gpu",
            "legacy variant-atlas opt-out must not downgrade the default renderer"
        );
        assert_eq!(
            renderer_env_or_default_static(None, Some("off")),
            "assets-variant-gpu",
            "static env resolver must keep the default on the RGBA variant path"
        );
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
            renderer_env_or_default_static(Some("classic"), Some("off")),
            "assets-variant-gpu",
            "the removed classic mode falls back to the default variant path"
        );
        assert_eq!(
            renderer_env_or_default_static(Some("unknown-mode"), Some("off")),
            "assets-variant-gpu",
            "unknown explicit renderer modes fall back to the default variant path"
        );
    }

    #[test]
    fn live_gpu_asset_mode_defaults_to_variant_gpu() {
        let mode = live_gpu_asset_mode_from_env_value(None).expect("default live mode");

        assert_eq!(mode.name(), "assets-variant-gpu");
        assert!(mode.uses_gpu_assets());
        assert!(mode.uses_variant_atlas());
        assert_eq!(
            RendererMode::from_effective_mode(mode),
            RendererMode::Modern
        );
    }

    #[test]
    fn live_gpu_asset_mode_rejects_cpu_or_diagnostic_modes() {
        for value in [
            "assets-anim-gpu",
            "assets-anim",
            "modern",
            "modern-compare",
            "unknown-mode",
        ] {
            let error = live_gpu_asset_mode_from_env_value(Some(value)).unwrap_err();

            assert_eq!(
                error,
                format!(
                    "ZELDA3_RENDERER={value:?} is diagnostic-only for live play; expected assets-variant-gpu"
                )
            );
        }
    }
}
