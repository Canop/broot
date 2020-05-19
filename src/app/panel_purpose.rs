use {
    crate::selection_type::SelectionType,
};

/// the possible special reason the panel was open
#[derive(Debug, Clone, Copy)]
pub enum PanelPurpose {
    None,
    ArgEdition {
        arg_type: SelectionType,
    },
}

impl PanelPurpose {
    pub fn is_arg_edition(self) -> bool {
        match self {
            PanelPurpose::ArgEdition { .. } => true,
            _ => false,
        }
    }
}
