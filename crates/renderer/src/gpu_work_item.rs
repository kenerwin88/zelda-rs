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
