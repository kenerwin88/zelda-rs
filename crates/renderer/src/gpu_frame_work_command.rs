use crate::gpu_work_item::{GpuRenderPlan, GpuWorkItem, GpuWorkItemKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFrameWindowSelector {
    pub(crate) screen_idx: usize,
    pub(crate) layer_bit: u8,
    pub(crate) flags_shift: u32,
}

impl GpuFrameWindowSelector {
    pub(crate) fn main(layer_bit: u8, flags_shift: u32) -> Self {
        Self {
            screen_idx: 0,
            layer_bit,
            flags_shift,
        }
    }

    pub(crate) fn sub(layer_bit: u8, flags_shift: u32) -> Self {
        Self {
            screen_idx: 1,
            layer_bit,
            flags_shift,
        }
    }

    pub(crate) fn flags(self, windowsel: u32) -> u32 {
        (windowsel >> self.flags_shift) & 0x0f
    }

    pub(crate) fn is_windowed(self, screen_windowed: [u8; 2]) -> bool {
        screen_windowed[self.screen_idx] & self.layer_bit != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameMainWorkCommand {
    ClearBackdrop,
    SpritePriority {
        priority: u32,
        math_bit_pos: u32,
        window: GpuFrameWindowSelector,
    },
    BgLayer {
        layer_idx: usize,
        hi_priority: bool,
        is_2bpp: bool,
        layer_bit: u32,
        math_bit_pos: u32,
        mosaic_layer_bit: u8,
        window: GpuFrameWindowSelector,
    },
    Mode7Bg {
        math_bit_pos: u32,
        layer_bit: u32,
        window: GpuFrameWindowSelector,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameSubWorkCommand {
    ClearBackdrop,
    Mode7Bg {
        math_bit_pos: u32,
        layer_bit: u32,
        window: GpuFrameWindowSelector,
    },
    BgLayer {
        layer_idx: usize,
        hi_priority: bool,
        is_2bpp: bool,
        screen_layer_bit: u8,
        render_layer_bit: u32,
        math_bit_pos: u32,
        mosaic_layer_bit: u8,
        window: GpuFrameWindowSelector,
    },
    SpritePriority {
        priority: u32,
        math_bit_pos: u32,
        window: GpuFrameWindowSelector,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameWorkCommand {
    Main(GpuFrameMainWorkCommand),
    Sub(GpuFrameSubWorkCommand),
    PostProcess,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameRenderPhase {
    Main,
    Sub,
    PostProcess,
}

pub(crate) type GpuFrameRenderPlan = GpuRenderPlan<GpuFrameWorkCommand>;

#[cfg(test)]
impl GpuFrameWorkCommand {
    pub(crate) fn phase(&self) -> GpuFrameRenderPhase {
        match self {
            Self::Main(_) => GpuFrameRenderPhase::Main,
            Self::Sub(_) => GpuFrameRenderPhase::Sub,
            Self::PostProcess => GpuFrameRenderPhase::PostProcess,
        }
    }
}

impl GpuWorkItem for GpuFrameMainWorkCommand {
    fn kind(&self) -> GpuWorkItemKind {
        match self {
            Self::ClearBackdrop => GpuWorkItemKind::ClearBackdrop,
            Self::SpritePriority { .. } => GpuWorkItemKind::MainSpritePriority,
            Self::BgLayer { .. } => GpuWorkItemKind::MainBgLayer,
            Self::Mode7Bg { .. } => GpuWorkItemKind::Mode7MainBg,
        }
    }
}

impl GpuWorkItem for GpuFrameSubWorkCommand {
    fn kind(&self) -> GpuWorkItemKind {
        match self {
            Self::ClearBackdrop => GpuWorkItemKind::ClearSubBackdrop,
            Self::Mode7Bg { .. } => GpuWorkItemKind::Mode7SubBg,
            Self::BgLayer { .. } => GpuWorkItemKind::SubBgLayer,
            Self::SpritePriority { .. } => GpuWorkItemKind::SubSpritePriority,
        }
    }
}

impl GpuWorkItem for GpuFrameWorkCommand {
    fn kind(&self) -> GpuWorkItemKind {
        match self {
            Self::Main(command) => command.kind(),
            Self::Sub(command) => command.kind(),
            Self::PostProcess => GpuWorkItemKind::PostProcess,
        }
    }
}

pub(crate) fn main_frame_work_command(command: GpuFrameMainWorkCommand) -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::Main(command)
}

pub(crate) fn sub_frame_work_command(command: GpuFrameSubWorkCommand) -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::Sub(command)
}

pub(crate) fn post_process_frame_work_command() -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::PostProcess
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_selector_reads_shifted_flags_and_screen_layer_bits() {
        let main_bg3 = GpuFrameWindowSelector::main(0x04, 8);
        assert_eq!(main_bg3.flags(0x0000_0a00), 0x0a);
        assert!(main_bg3.is_windowed([0x04, 0x00]));
        assert!(!main_bg3.is_windowed([0x00, 0x04]));

        let sub_obj = GpuFrameWindowSelector::sub(0x10, 16);
        assert_eq!(sub_obj.flags(0x000b_0000), 0x0b);
        assert!(sub_obj.is_windowed([0x00, 0x10]));
        assert!(!sub_obj.is_windowed([0x10, 0x00]));
    }
}
