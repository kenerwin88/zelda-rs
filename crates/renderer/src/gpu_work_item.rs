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

pub(crate) struct GpuRenderPlan<T> {
    work_items: Vec<T>,
}

pub(crate) struct SourcedGpuWorkCommand<Source, Command> {
    pub(crate) source: Source,
    pub(crate) command: Command,
}

pub(crate) struct LoadedGpuWorkCommand<Load, WorkItem> {
    pub(crate) target_load: Load,
    pub(crate) work_item: WorkItem,
}

impl<Source, Command> GpuWorkItem for SourcedGpuWorkCommand<Source, Command>
where
    Command: GpuWorkItem,
{
    fn kind(&self) -> GpuWorkItemKind {
        GpuWorkItem::kind(&self.command)
    }
}

impl<Load, WorkItem> GpuWorkItem for LoadedGpuWorkCommand<Load, WorkItem>
where
    WorkItem: GpuWorkItem,
{
    fn kind(&self) -> GpuWorkItemKind {
        GpuWorkItem::kind(&self.work_item)
    }
}

impl<T> GpuRenderPlan<T> {
    pub(crate) fn new(work_items: Vec<T>) -> Self {
        Self { work_items }
    }

    #[cfg(test)]
    pub(crate) fn work_items(&self) -> &[T] {
        &self.work_items
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.work_items.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.work_items.len()
    }
}

impl<T> FromIterator<T> for GpuRenderPlan<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<T: GpuWorkItem> GpuRenderPlan<T> {
    pub(crate) fn execute_with<F>(self, mut execute: F)
    where
        F: FnMut(T),
    {
        for work_item in self.work_items {
            let _ = work_item.kind();
            execute(work_item);
        }
    }

    #[cfg(test)]
    pub(crate) fn kinds(&self) -> Vec<GpuWorkItemKind> {
        work_item_kinds(&self.work_items)
    }
}

impl<T> IntoIterator for GpuRenderPlan<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.work_items.into_iter()
    }
}

#[cfg(test)]
pub(crate) fn work_item_kinds<T: GpuWorkItem>(items: &[T]) -> Vec<GpuWorkItemKind> {
    items.iter().map(GpuWorkItem::kind).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        GpuRenderPlan, GpuWorkItem, GpuWorkItemKind, LoadedGpuWorkCommand, SourcedGpuWorkCommand,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestWorkItem {
        Clear,
        Draw,
    }

    impl GpuWorkItem for TestWorkItem {
        fn kind(&self) -> GpuWorkItemKind {
            match self {
                Self::Clear => GpuWorkItemKind::ClearBackdrop,
                Self::Draw => GpuWorkItemKind::BgEffect,
            }
        }
    }

    #[test]
    fn render_plan_execute_with_preserves_work_item_order() {
        let plan = GpuRenderPlan::new(vec![TestWorkItem::Clear, TestWorkItem::Draw]);
        let mut executed = Vec::new();

        plan.execute_with(|work_item| executed.push(work_item));

        assert_eq!(executed, vec![TestWorkItem::Clear, TestWorkItem::Draw]);
    }

    #[test]
    fn render_plan_collect_preserves_work_item_order() {
        let plan = [TestWorkItem::Clear, TestWorkItem::Draw]
            .into_iter()
            .collect::<GpuRenderPlan<_>>();

        assert_eq!(
            plan.work_items(),
            &[TestWorkItem::Clear, TestWorkItem::Draw]
        );
        assert_eq!(
            plan.kinds(),
            vec![GpuWorkItemKind::ClearBackdrop, GpuWorkItemKind::BgEffect]
        );
    }

    #[test]
    fn command_wrappers_forward_inner_work_item_kind() {
        let sourced = SourcedGpuWorkCommand {
            source: "rank-0",
            command: TestWorkItem::Draw,
        };
        let loaded = LoadedGpuWorkCommand {
            target_load: "clear",
            work_item: TestWorkItem::Clear,
        };

        assert_eq!(sourced.kind(), GpuWorkItemKind::BgEffect);
        assert_eq!(loaded.kind(), GpuWorkItemKind::ClearBackdrop);
    }
}
