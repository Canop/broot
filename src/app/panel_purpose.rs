use {
    super::SelectionType,
};

/// the possible special reason the panel was open
#[derive(Debug, Clone, Copy)]
pub enum PanelPurpose {
    None,
    ArgEdition {
        arg_type: SelectionType,
    },
    Preview,
}

impl PanelPurpose {
    pub fn is_arg_edition(self) -> bool {
        matches!(self, PanelPurpose::ArgEdition { .. })
    }
    pub fn is_preview(self) -> bool {
        matches!(self, PanelPurpose::Preview)
    }
}
