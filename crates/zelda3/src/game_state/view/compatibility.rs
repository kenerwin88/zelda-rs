pub(crate) struct CompatibilityBytesView<'a> {
    ram: &'a [u8],
}

impl<'a> CompatibilityBytesView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn range(&self, offset: usize, len: usize) -> &'a [u8] {
        &self.ram[offset..offset + len]
    }
}

pub(crate) struct CompatibilityBytesViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> CompatibilityBytesViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_byte_at(&mut self, offset: usize, value: u8) {
        self.ram[offset] = value;
    }
}
