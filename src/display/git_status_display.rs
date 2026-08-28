use {
    super::CropWriter,
    crate::{
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
    show_ahead_behind: bool,
    show_stats: bool,
    pub width: usize,
}

/// Return the unstyled "↑3↓1" showing the divergence with the upstream branch
fn ahead_behind_string(status: &TreeGitStatus) -> Option<String> {
    let mut s = String::new();
    if status.ahead > 0 {
        s.push_str(&format!("↑{}", status.ahead));
    }
    if status.behind > 0 {
        s.push_str(&format!("↓{}", status.behind));
    }
    (!s.is_empty()).then_some(s)
}

impl<'a, 's> GitStatusDisplay<'a, 's> {
    pub fn from(
        status: &'a TreeGitStatus,
        skin: &'s StyleMap,
        available_width: usize,
    ) -> Self {
        let mut show_branch = false;
        let mut width = 0;
        if let Some(branch) = &status.current_branch_name {
            let branch_width = branch.chars().count();
            if branch_width < available_width {
                width += branch_width;
                show_branch = true;
            }
        }
        let mut show_ahead_behind = false;
        if let Some(ab) = ahead_behind_string(status) {
            let ab_width = ab.chars().count() + 1; // with the trailing space
            if width + ab_width < available_width {
                width += ab_width;
                show_ahead_behind = true;
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
            show_ahead_behind,
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
        if self.show_ahead_behind {
            if self.status.ahead > 0 {
                cond_bg!(ahead_style, self, selected, self.skin.git_insertions);
                cw.queue_g_string(ahead_style, format!("↑{}", self.status.ahead))?;
            }
            if self.status.behind > 0 {
                cond_bg!(behind_style, self, selected, self.skin.git_deletions);
                cw.queue_g_string(behind_style, format!("↓{}", self.status.behind))?;
            }
            cw.queue_char(&self.skin.default, ' ')?;
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
