
use {
    super::W,
    crate::{
        errors::ProgramError,
        flag::Flag,
        skin::PanelSkin,
    },
};

/// compute the needed length for displaying the flags
pub fn visible_width(flags: &[Flag]) -> u16 {
    let mut width = flags.len() * 2 + 1;
    for flag in flags {
        width += flag.name.len(); // we assume only ascii chars
        width += flag.value.len();
    }
    width as u16
}

/// draw the flags
pub fn write(
    w: &mut W,
    flags: &[Flag],
    panel_skin: &PanelSkin,
) -> Result<(), ProgramError> {
    for flag in flags {
        panel_skin.styles.flag_label.queue_str(w, &format!( " {}:", flag.name))?;
        panel_skin.styles.flag_value.queue(w, flag.value)?;
        panel_skin.styles.flag_label.queue(w, ' ')?;
    }
    Ok(())
}
