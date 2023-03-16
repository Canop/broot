use {
    super::CropWriter,
    crate::{
        display::cond_bg,
        errors::ProgramError,
        git::TreeGitStatus,
        skin::StyleMap,
    },
};

pub struct GitStatusDisplay<'a, 's> {
    status: &'a TreeGitStatus,
    skin: &'s StyleMap,
    show_branch: bool,
    show_wide: bool,
    show_stats: bool,
    pub width: usize,
}

impl<'a, 's> GitStatusDisplay<'a, 's> {
    pub fn from(status: &'a TreeGitStatus, skin: &'s StyleMap, available_width: usize) -> Self {
        let mut show_branch = false;
        let mut width = 0;
        if let Some(branch) = &status.current_branch_name {
            let branch_width = branch.chars().count();
            if branch_width < available_width {
                width += branch_width;
                show_branch = true;
            }
        }
        let mut show_stats = false;
        let unstyled_stats = format!("+{}-{}", status.insertions, status.deletions);
        let stats_width = unstyled_stats.len();
        if width + stats_width < available_width {
            width += stats_width;
            show_stats = true;
        }
        let show_wide = width + 3 < available_width;
        if show_wide {
            width += 3; // difference between compact and wide format widths
        }
        Self {
            status,
            skin,
            show_branch,
            show_wide,
            show_stats,
            width,
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
        if self.show_branch {
            cond_bg!(branch_style, self, selected, self.skin.git_branch);
            if let Some(name) = &self.status.current_branch_name {
                if self.show_wide {
                    cw.queue_str(branch_style, " ᚜ ")?;
                } else {
                    cw.queue_char(branch_style, ' ')?;
                }
                cw.queue_str(branch_style, name)?;
                cw.queue_char(branch_style, ' ')?;
            }
        }
        if self.show_stats {
            cond_bg!(insertions_style, self, selected, self.skin.git_insertions);
            cw.queue_g_string(insertions_style, format!("+{}", self.status.insertions))?;
            cond_bg!(deletions_style, self, selected, self.skin.git_deletions);
            cw.queue_g_string(deletions_style, format!("-{}", self.status.deletions))?;
        }
        Ok(())
    }
}
