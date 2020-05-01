use {super::PanelId, crate::selection_type::SelectionType};

pub enum PanelPurpose {
    None,
    ArgEdition {
        parent_panel_id: PanelId,
        arg_type: SelectionType,
    },
}
