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
