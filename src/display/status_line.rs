use {
    super::{Screen, W},
    crate::{
        app::Status,
        errors::ProgramError,
        skin::PanelSkin,
    },
    termimad::{
        minimad::{Alignment, Composite},
        Area, StyledChar,
    },
};

/// write the whole status line (task + status)
pub fn write(
    w: &mut W,
    task: Option<&str>,
    status: &Status,
    area: &Area,
    panel_skin: &PanelSkin,
    screen: Screen,
) -> Result<(), ProgramError> {
    let y = area.top;
    screen.goto(w, area.left, y)?;
    let mut x = area.left;
    if let Some(pending_task) = task {
        let pending_task = format!(" {pending_task}… ");
        x += pending_task.chars().count() as u16;
        panel_skin.styles.status_job.queue(w, pending_task)?;
    }
    screen.goto(w, x, y)?;
    let style = if status.error {
        &panel_skin.status_skin.error
    } else {
        &panel_skin.status_skin.normal
    };
    style.write_inline_on(w, " ")?;
    let remaining_width = (area.width - (x - area.left) - 1) as usize;
    style.write_composite_fill(
        w,
        Composite::from_inline(&status.message),
        remaining_width,
        Alignment::Unspecified,
    )?;
    Ok(())
}

/// erase the whole status line
pub fn erase(
    w: &mut W,
    area: &Area,
    panel_skin: &PanelSkin,
    screen: Screen,
) -> Result<(), ProgramError> {
    screen.goto(w, area.left, area.top)?;
    let sc = StyledChar::new(
        panel_skin.status_skin.normal.paragraph.compound_style.clone(),
        ' ',
    );
    sc.queue_repeat(w, area.width as usize)?;
    Ok(())
}
