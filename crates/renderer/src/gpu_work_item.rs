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

impl<T> GpuRenderPlan<T> {
    pub(crate) fn new(work_items: Vec<T>) -> Self {
        Self { work_items }
    }

    #[cfg(test)]
    pub(crate) fn work_items(&self) -> &[T] {
        &self.work_items
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.work_items.len()
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
    use super::{GpuRenderPlan, GpuWorkItem, GpuWorkItemKind};

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
}
