use crate::gpu_work_item::{GpuRenderPlan, SourcedGpuWorkCommand};
use crate::modern_gpu::{mode1_effect_rank_dispatches, Mode1EffectRankDispatch};
#[cfg(test)]
use crate::modern_gpu_work_command::ModernGpuWorkCommandKind;
use crate::modern_gpu_work_command::{
    ModernGpuCommandLoad, ModernGpuWorkCommand, ModernGpuWorkItem,
};

pub(crate) struct PreparedMode1EffectDrawWork<'frame> {
    rank_dispatches: Vec<Mode1EffectRankDispatch<'frame>>,
}

impl<'frame> PreparedMode1EffectDrawWork<'frame> {
    pub(crate) fn from_plan(plan: &crate::modern_variant_draw::VariantDrawPlan<'frame>) -> Self {
        Self {
            rank_dispatches: mode1_effect_rank_dispatches(plan),
        }
    }

    #[cfg(test)]
    pub(crate) fn rank_dispatches(&self) -> &[Mode1EffectRankDispatch<'frame>] {
        &self.rank_dispatches
    }

    pub(crate) fn render_plan<'work>(
        &'work self,
        frame: &'work crate::modern_frame::ModernFrame,
        atlas: &'work crate::modern_variant_atlas::ModernVariantAtlas,
    ) -> PreparedMode1EffectRenderPlan<'work, 'frame> {
        let mut rendered_any = false;
        let mut rank_plans = Vec::with_capacity(self.rank_dispatches.len());
        for (rank_index, rank_dispatch) in self.rank_dispatches.iter().enumerate() {
            let rendered_before = rendered_any;
            let render_plan = rank_dispatch.render_plan(frame, atlas, rendered_before);
            if !render_plan.is_empty() {
                rendered_any = true;
            }
            rank_plans.push(PreparedMode1EffectRankRenderPlan {
                rank_index,
                #[cfg(test)]
                rendered_before,
                render_plan,
            });
        }
        PreparedMode1EffectRenderPlan {
            rank_plans,
            needs_empty_frame_fallback: !rendered_any,
        }
    }
}

pub(crate) struct PreparedMode1EffectRenderPlan<'rank, 'frame> {
    pub(crate) rank_plans: Vec<PreparedMode1EffectRankRenderPlan<'rank, 'frame>>,
    pub(crate) needs_empty_frame_fallback: bool,
}

impl<'rank, 'frame> PreparedMode1EffectRenderPlan<'rank, 'frame> {
    pub(crate) fn into_command_plan(self) -> Mode1EffectCommandPlan<'rank, 'frame> {
        let needs_empty_frame_fallback = self.needs_empty_frame_fallback;
        self.rank_plans
            .into_iter()
            .flat_map(|rank_plan| {
                let PreparedMode1EffectRankRenderPlan {
                    rank_index,
                    render_plan,
                    ..
                } = rank_plan;
                render_plan
                    .into_iter()
                    .map(move |command| SourcedGpuWorkCommand {
                        source: Mode1EffectCommandSource::Rank(rank_index),
                        command,
                    })
            })
            .chain(needs_empty_frame_fallback.then_some(SourcedGpuWorkCommand {
                source: Mode1EffectCommandSource::EmptyFrameFallback,
                command: ModernGpuWorkCommand {
                    target_load: ModernGpuCommandLoad::ClearFrame,
                    work_item: ModernGpuWorkItem::ClearBackdrop,
                },
            }))
            .collect()
    }

    #[cfg(test)]
    fn into_steps(self) -> impl Iterator<Item = PreparedMode1EffectRenderStep<'rank, 'frame>> {
        let needs_empty_frame_fallback = self.needs_empty_frame_fallback;
        self.rank_plans
            .into_iter()
            .map(PreparedMode1EffectRenderStep::Rank)
            .chain(
                needs_empty_frame_fallback
                    .then_some(PreparedMode1EffectRenderStep::EmptyFrameFallback),
            )
    }

    #[cfg(test)]
    pub(crate) fn steps(&self) -> Vec<PreparedMode1EffectRenderStepKind> {
        let mut steps = self
            .rank_plans
            .iter()
            .map(|rank_plan| PreparedMode1EffectRenderStepKind::Rank {
                rank_index: rank_plan.rank_index(),
                is_empty: rank_plan.is_empty(),
                rendered_before: rank_plan.rendered_before(),
            })
            .collect::<Vec<_>>();
        if self.needs_empty_frame_fallback {
            steps.push(PreparedMode1EffectRenderStepKind::EmptyFrameFallback);
        }
        steps
    }

    #[cfg(test)]
    pub(crate) fn into_step_kinds(self) -> Vec<PreparedMode1EffectRenderStepKind> {
        let mut steps = Vec::new();
        for step in self.into_steps() {
            steps.push(step.kind());
        }
        steps
    }

    #[cfg(test)]
    pub(crate) fn into_command_kinds(self) -> Vec<PreparedMode1EffectRenderCommandKind> {
        let mut commands = Vec::new();
        self.into_command_plan()
            .execute_with(|command| commands.push(command.mode1_kind()));
        commands
    }

    #[cfg(test)]
    pub(crate) fn needs_empty_frame_fallback(&self) -> bool {
        self.needs_empty_frame_fallback
    }

    #[cfg(test)]
    pub(crate) fn rank_plans(&self) -> &[PreparedMode1EffectRankRenderPlan<'rank, 'frame>] {
        &self.rank_plans
    }
}

#[cfg(test)]
enum PreparedMode1EffectRenderStep<'rank, 'frame> {
    Rank(PreparedMode1EffectRankRenderPlan<'rank, 'frame>),
    EmptyFrameFallback,
}

pub(crate) type PreparedMode1EffectRenderCommand<'rank, 'frame> =
    SourcedGpuWorkCommand<Mode1EffectCommandSource, ModernGpuWorkCommand<'rank, 'frame>>;
type Mode1EffectCommandPlan<'rank, 'frame> =
    GpuRenderPlan<PreparedMode1EffectRenderCommand<'rank, 'frame>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode1EffectCommandSource {
    Rank(usize),
    EmptyFrameFallback,
}

#[cfg(test)]
impl SourcedGpuWorkCommand<Mode1EffectCommandSource, ModernGpuWorkCommand<'_, '_>> {
    fn mode1_kind(&self) -> PreparedMode1EffectRenderCommandKind {
        PreparedMode1EffectRenderCommandKind::Work {
            source: self.source,
            command: self.command.kind(),
        }
    }
}

#[cfg(test)]
impl PreparedMode1EffectRenderStep<'_, '_> {
    fn kind(&self) -> PreparedMode1EffectRenderStepKind {
        match self {
            Self::Rank(rank_plan) => PreparedMode1EffectRenderStepKind::Rank {
                rank_index: rank_plan.rank_index(),
                is_empty: rank_plan.is_empty(),
                rendered_before: rank_plan.rendered_before(),
            },
            Self::EmptyFrameFallback => PreparedMode1EffectRenderStepKind::EmptyFrameFallback,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedMode1EffectRenderStepKind {
    Rank {
        rank_index: usize,
        is_empty: bool,
        rendered_before: bool,
    },
    EmptyFrameFallback,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedMode1EffectRenderCommandKind {
    Work {
        source: Mode1EffectCommandSource,
        command: ModernGpuWorkCommandKind,
    },
}

pub(crate) struct PreparedMode1EffectRankRenderPlan<'rank, 'frame> {
    pub(crate) rank_index: usize,
    #[cfg(test)]
    pub(crate) rendered_before: bool,
    pub(crate) render_plan: crate::modern_gpu::Mode1EffectRankRenderPlan<'rank, 'frame>,
}

impl<'rank, 'frame> PreparedMode1EffectRankRenderPlan<'rank, 'frame> {
    #[cfg(test)]
    pub(crate) fn rank_index(&self) -> usize {
        self.rank_index
    }

    #[cfg(test)]
    pub(crate) fn rendered_before(&self) -> bool {
        self.rendered_before
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.render_plan.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn kinds(&self) -> Vec<crate::gpu_work_item::GpuWorkItemKind> {
        self.render_plan.kinds()
    }
}
