#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererMode {
    Classic,
    ModernCompare,
    Modern,
}

impl RendererMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("modern-compare") => Self::ModernCompare,
            Some("modern") => Self::Modern,
            _ => Self::Classic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_mode_parse_defaults_to_classic() {
        assert_eq!(RendererMode::parse(None), RendererMode::Classic);
        assert_eq!(
            RendererMode::parse(Some("modern-compare")),
            RendererMode::ModernCompare
        );
        assert_eq!(RendererMode::parse(Some("modern")), RendererMode::Modern);
        assert_eq!(RendererMode::parse(Some("garbage")), RendererMode::Classic);
    }
}
