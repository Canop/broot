use {
    crate::{
        display::cond_bg,
        errors::ProgramError,
        skin::StyleMap,
    },
    crokey::crossterm::{
        style::{ResetColor, SetBackgroundColor, SetForegroundColor},
        QueueableCommand,
    },
    lfs_core::Mount,
    termimad::*,
};

/// an abstract of the space info relative to a block device.
/// It's supposed to be shown on top of screen next to the root.
pub struct MountSpaceDisplay<'m, 's> {
    mount: &'m Mount,
    skin: &'s StyleMap,
    pub available_width: usize,
}

impl<'m, 's> MountSpaceDisplay<'m, 's> {
    pub fn from(mount: &'m Mount, skin: &'s StyleMap, available_width: usize) -> Self {
        Self {
            mount,
            skin,
            available_width,
        }
    }

    pub fn write<W>(
        &self,
        cw: &mut CropWriter<W>,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: std::io::Write,
    {
        if self.available_width < 4 {
            return Ok(());
        }
        let bg = if selected {
            self.skin.selected_line.get_bg()
        } else {
            self.skin.default.get_bg()
        };
        cond_bg!(txt_style, self, selected, self.skin.default);
        let w_fs = self.mount.info.fs.chars().count();
        if let Some(s) = &self.mount.stats() {
            //- width computation
            let mut e_fs = false;
            let dsk = self.mount.disk.as_ref().map_or("", |d| d.disk_type());
            let w_dsk = dsk.chars().count();
            let mut e_dsk = false;
            let w_fraction = 9;
            let mut e_fraction = false;
            let mut w_bar = 2; // min width
            let mut e_bar = false;
            let w_percent = 4;
            let mut rem = self.available_width - w_percent;
            let share_color = self.skin.good_to_bad_color(s.use_share());
            if rem > 1 {
                // left margin for readability
                rem -= 1;
                cw.queue_char(txt_style, ' ')?;
            }
            if rem > w_fs {
                rem -= w_fs + 1; // 1 for margin
                e_fs = true;
            }
            if rem > w_fraction {
                rem -= w_fraction + 1;
                e_fraction = true;
            }
            if rem > w_bar {
                rem -= w_bar + 1;
                e_bar = true;
            }
            if rem > w_dsk && w_dsk > 0 {
                rem -= w_dsk + 1;
                e_dsk = true;
            }
            if e_bar && rem > 0 {
                w_bar += rem.min(7);
            }
            //- display
            if e_fs {
                cw.queue_g_string(txt_style, format!(" {}", &self.mount.info.fs))?;
            }
            if e_dsk {
                cw.queue_char(txt_style, ' ')?;
                cw.queue_g_string(txt_style, dsk.to_string())?;
            }
            if e_fraction {
                if let Some(bg_color) = bg {
                    cw.w.queue(SetBackgroundColor(bg_color))?;
                } else {
                    cw.w.queue(ResetColor {})?;
                }
                cw.w.queue(SetForegroundColor(share_color))?;
                cw.queue_unstyled_char(' ')?;
                cw.queue_unstyled_g_string(file_size::fit_4(s.used()))?;
                cw.queue_g_string(txt_style, format!("/{}", file_size::fit_4(s.size())))?;
            }
            if e_bar {
                let pb = ProgressBar::new(s.use_share() as f32, w_bar);
                cw.w.queue(ResetColor {})?;
                if let Some(bg_color) = bg {
                    cw.w.queue(SetBackgroundColor(bg_color))?;
                }
                cw.queue_unstyled_char(' ')?;
                cw.w.queue(SetBackgroundColor(share_color))?;
                cw.queue_unstyled_g_string(format!("{pb:<w_bar$}"))?;
            }
            if let Some(bg_color) = bg {
                cw.w.queue(SetBackgroundColor(bg_color))?;
            } else {
                cw.w.queue(ResetColor {})?;
            }
            cw.w.queue(SetForegroundColor(share_color))?;
            cw.queue_unstyled_g_string(format!("{:>3.0}%", 100.0 * s.use_share()))?;
        } else {
            // there's not much to print if there's no size info
            cw.queue_g_string(txt_style, format!(" {}", &self.mount.info.fs))?;
        }
        cw.w.queue(ResetColor {})?;
        Ok(())
    }
}

