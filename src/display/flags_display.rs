
use {
    crate::{
        display::{Screen, W},
        errors::ProgramError,
        flag::Flag,
        skin::PanelSkin,
    },
    termimad::Area,
};

/// draw the flags at the bottom right of the given area
/// (this area is usually the input: flags are displayed over it)
pub fn write_flags(
    w: &mut W,
    flags: &[Flag],
    area: &Area,
    input_content_len: u16,
    screen: &mut Screen,
    panel_skin: &PanelSkin,
) -> Result<(), ProgramError> {
    if flags.is_empty() {
        return Ok(());
    }
    let mut width = flags.len() * 5 - 3;
    for flag in flags {
        width += flag.name.len(); // we assume only ascii chars
        width += flag.value.len();
    }
    let width = width as u16;
    if width + input_content_len + 2 >= area.width {
        debug!("not enough space to display flags");
        return Ok(());
    }
    screen.goto(w, screen.width - 1 - width, screen.height - 1)?;
    screen.clear_line(w)?;
    for (i, flag) in flags.iter().enumerate() {
        panel_skin.styles.flag_label.queue_str(
            w,
            &format!(
                "{}{}:",
                if i==0 { " " } else { "  " },
                flag.name,
            )
        )?;
        panel_skin.styles.flag_value.queue(w, flag.value)?;
    }
    Ok(())
}
