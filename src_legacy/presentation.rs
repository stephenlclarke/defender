//! Presentation backend selection for live play.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentationBackend {
    Kitty,
    #[default]
    Wgpu,
}

impl PresentationBackend {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "kitty" => Some(Self::Kitty),
            "wgpu" => Some(Self::Wgpu),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Kitty => "kitty",
            Self::Wgpu => "wgpu",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PresentationBackend;

    #[test]
    fn presentation_backend_parser_and_labels_are_stable() {
        assert_eq!(
            PresentationBackend::parse("kitty"),
            Some(PresentationBackend::Kitty)
        );
        assert_eq!(
            PresentationBackend::parse("wgpu"),
            Some(PresentationBackend::Wgpu)
        );
        assert_eq!(PresentationBackend::parse("unknown"), None);
        assert_eq!(PresentationBackend::Kitty.label(), "kitty");
        assert_eq!(PresentationBackend::Wgpu.label(), "wgpu");
    }
}
