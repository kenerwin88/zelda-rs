use crate::gpu_work_item::{GpuWorkItem, GpuWorkItemKind, LoadedGpuWorkCommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectMaterial {
    StaticEffect,
    LiveCgram,
}

#[derive(Clone, Copy)]
pub(crate) struct EffectMaterialGroup<'dispatch, Packet> {
    pub(crate) material: EffectMaterial,
    pub(crate) packets: &'dispatch [Packet],
}

pub(crate) type BgEffectMaterialGroup<'dispatch, 'frame> =
    EffectMaterialGroup<'dispatch, crate::modern_variant_draw::VariantBgDrawPacket<'frame>>;
pub(crate) type SpriteEffectMaterialGroup<'dispatch, 'frame> =
    EffectMaterialGroup<'dispatch, crate::modern_variant_draw::VariantSpriteDrawPacket<'frame>>;

pub(crate) type ModernGpuWorkCommand<'rank, 'frame> =
    LoadedGpuWorkCommand<ModernGpuCommandLoad, ModernGpuWorkItem<'rank, 'frame>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModernGpuCommandLoad {
    ClearFrame,
    Load,
}

pub(crate) enum ModernGpuWorkItem<'rank, 'frame> {
    ClearBackdrop,
    BgEffect(BgEffectMaterialGroup<'rank, 'frame>),
    SpriteEffects(Vec<SpriteEffectMaterialGroup<'rank, 'frame>>),
}

impl GpuWorkItem for ModernGpuWorkItem<'_, '_> {
    fn kind(&self) -> GpuWorkItemKind {
        match self {
            Self::ClearBackdrop => GpuWorkItemKind::ClearBackdrop,
            Self::BgEffect(_) => GpuWorkItemKind::BgEffect,
            Self::SpriteEffects(_) => GpuWorkItemKind::SpriteEffects,
        }
    }
}

#[cfg(test)]
impl LoadedGpuWorkCommand<ModernGpuCommandLoad, ModernGpuWorkItem<'_, '_>> {
    pub(crate) fn kind(&self) -> ModernGpuWorkCommandKind {
        ModernGpuWorkCommandKind {
            target_load: self.target_load,
            work_item: self.work_item.kind(),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModernGpuWorkCommandKind {
    pub(crate) target_load: ModernGpuCommandLoad,
    pub(crate) work_item: GpuWorkItemKind,
}
