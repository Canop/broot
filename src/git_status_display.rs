use {
    crate::{
        errors::ProgramError,
        git_status::{
            TreeGitStatus,
        },
        skin::Skin,
    },
    std::io::Write,
};

pub struct GitStatusDisplay<'a> {
    status: &'a TreeGitStatus,
    show_branch: bool,
    show_wide: bool,
    show_stats: bool,
    pub width: usize,
}

impl<'a> GitStatusDisplay<'a> {
    pub fn from(
        status: &'a TreeGitStatus,
        available_width: usize,
    ) -> Self {
        let mut show_branch = false;
        let mut width = 0;
        if let Some(branch) = &status.current_branch_name {
            let branch_width = branch.chars().count();
            if branch_width <= available_width {
                width += branch_width;
                show_branch = true;
            }
        }
        let mut show_stats = false;
        let unstyled_stats = format!("+{}-{}", status.insertions, status.deletions);
        let stats_width = unstyled_stats.len();
        if width + stats_width <= available_width {
            width += stats_width;
            show_stats = true;
        }
        let mut show_wide = false;
        if width + 4 < available_width {
            width += 4;
            show_wide = true;
        }
        Self {
            status,
            show_branch,
            show_stats,
            show_wide,
            width,
        }
    }
    pub fn write(
        &self,
        f: &mut impl Write,
        skin: &Skin,
    ) -> Result<(), ProgramError> {
        if self.show_branch {
            if let Some(name) = &self.status.current_branch_name {
                if self.show_wide {
                    skin.git_branch.queue(f, format!(" ᚜ {} ", name))?;
                } else {
                    skin.git_branch.queue_str(f, name)?;
                }
            }
        }
        if self.show_stats {
            skin.git_insertions.queue(f, format!("+{}", self.status.insertions))?;
            skin.git_deletions.queue(f, format!("-{}", self.status.deletions))?;
        }
        Ok(())
    }
}

