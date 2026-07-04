use crate::gpu_work_item::{GpuRenderPlan, GpuWorkItem, GpuWorkItemKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameMainWorkCommand {
    ClearBackdrop,
    SpritePriority {
        priority: u32,
        math_bit_pos: u32,
        screen_idx: usize,
        window_layer_bit: u8,
        window_flags_shift: u32,
    },
    BgLayer {
        layer_idx: usize,
        hi_priority: bool,
        layer_bit: u32,
        math_bit_pos: u32,
    },
    Mode7Bg {
        math_bit_pos: u32,
        layer_bit: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameSubWorkCommand {
    ClearBackdrop,
    Mode7Bg {
        math_bit_pos: u32,
        layer_bit: u32,
    },
    BgLayer {
        layer_idx: usize,
        hi_priority: bool,
        screen_layer_bit: u8,
        render_layer_bit: u32,
        math_bit_pos: u32,
    },
    SpritePriority {
        priority: u32,
        math_bit_pos: u32,
        screen_idx: usize,
        window_layer_bit: u8,
        window_flags_shift: u32,
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
