use super::SelectionType;

/// the possible special reason the panel was open
#[derive(Debug, Clone, Copy)]
pub enum PanelPurpose {
    None,
    ArgEdition { arg_type: SelectionType },
    Preview,
}

impl PanelPurpose {
    #[must_use]
    pub fn is_arg_edition(self) -> bool {
        matches!(self, PanelPurpose::ArgEdition { .. })
    }
    #[must_use]
    pub fn is_preview(self) -> bool {
        matches!(self, PanelPurpose::Preview)
    }
}
