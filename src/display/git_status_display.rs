use {
    super::CropWriter,
    crate::{errors::ProgramError, git::TreeGitStatus, skin::Skin},
};

pub struct GitStatusDisplay<'a, 's> {
    status: &'a TreeGitStatus,
    skin: &'s Skin,
    show_branch: bool,
    show_wide: bool,
    show_stats: bool,
    pub width: usize,
}

impl<'a, 's> GitStatusDisplay<'a, 's> {
    pub fn from(status: &'a TreeGitStatus, skin: &'s Skin, available_width: usize) -> Self {
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
        let mut show_wide = false;
        if width + 3 < available_width {
            width += 3; // difference between compact and wide format widths
            show_wide = true;
        }
        Self {
            status,
            skin,
            show_branch,
            show_stats,
            show_wide,
            width,
        }
    }

    pub fn write<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: std::io::Write,
    {
        if self.show_branch {
            cond_bg!(branch_style, self, selected, self.skin.git_branch);
            if let Some(name) = &self.status.current_branch_name {
                if self.show_wide {
                    cw.queue_string(&branch_style, format!(" ᚜ {} ", name))?;
                } else {
                    cw.queue_string(&branch_style, format!(" {} ", name))?;
                }
            }
        }
        if self.show_stats {
            cond_bg!(insertions_style, self, selected, self.skin.git_insertions);
            cw.queue_string(&insertions_style, format!("+{}", self.status.insertions))?;
            cond_bg!(deletions_style, self, selected, self.skin.git_deletions);
            cw.queue_string(&deletions_style, format!("-{}", self.status.deletions))?;
        }
        Ok(())
    }
}
