#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuWorkItemKind {
    ClearBackdrop,
    MainBgLayer,
    MainSpritePriority,
    Mode7MainBg,
    ClearSubBackdrop,
    Mode7SubBg,
    SubBgLayer,
    SubSpritePriority,
    BgEffect,
    SpriteEffects,
    PostProcess,
}

pub(crate) trait GpuWorkItem {
    fn kind(&self) -> GpuWorkItemKind;
}

#[cfg(test)]
pub(crate) fn work_item_kinds<T: GpuWorkItem>(items: &[T]) -> Vec<GpuWorkItemKind> {
    items.iter().map(GpuWorkItem::kind).collect()
}
