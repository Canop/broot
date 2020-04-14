use {
    crate::{
        errors::ProgramError,
        file_sizes::FileSize,
        flat_tree::{LineType, Tree, TreeLine},
        task_sync::ComputationResult,
        git::GitStatusDisplay,
        pattern::Pattern,
        skin::Skin,
    },
    chrono::{offset::Local, DateTime},
    crossterm::{
        cursor,
        style::{Color, SetBackgroundColor},
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    git2::{
        Status,
    },
    std::{io::Write, time::SystemTime},
    termimad::{CompoundStyle, ProgressBar},
};

#[cfg(unix)]
use {
    crate::permissions,
    std::os::unix::fs::MetadataExt,
    umask::*,
};

/// declare a style named `$dst` which is usually a reference to the `$src`
/// skin but, in case `selected` is true, is a clone with background changed
/// to the one of selected lines.
macro_rules! cond_bg {
    ($dst:ident, $self:ident, $selected:expr, $src:expr) => {
        let mut cloned_style;
        let $dst = if $selected {
            cloned_style = $src.clone();
            if let Some(c) = $self.skin.selected_line.get_bg() {
                cloned_style.set_bg(c);
            }
            &cloned_style
        } else {
            &$src
        };
    };
}


/// A tree wrapper which can be used either
/// - to write on the screen in the application,
/// - or to write in a file or an exported string.
/// Using it in the application (with in_app true) means that
///  - the selection is drawn
///  - a scrollbar may be drawn
///  - the empty lines will be erased
/// (cleaning while printing isn't clean but cleaning
///  before would involve a visible flash on redraw)
pub struct DisplayableTree<'s, 't> {
    pub tree: &'t Tree,
    pub skin: &'s Skin,
    pub area: termimad::Area,
    pub in_app: bool, // if true we show the selection and scrollbar
}

impl<'s, 't> DisplayableTree<'s, 't>
{

    pub fn out_of_app(tree: &'t Tree, skin: &'s Skin, width: u16) -> DisplayableTree<'s, 't> {
        DisplayableTree {
            tree,
            skin,
            area: termimad::Area {
                left: 0,
                top: 0,
                width,
                height: tree.lines.len() as u16,
            },
            in_app: false,
        }
    }

    fn name_style(&self, line: &TreeLine) -> &CompoundStyle {
        match &line.line_type {
            LineType::Dir => &self.skin.directory,
            LineType::File => {
                if line.is_exe() {
                    &self.skin.exe
                } else {
                    &self.skin.file
                }
            }
            LineType::SymLinkToFile(_) | LineType::SymLinkToDir(_) => &self.skin.link,
            LineType::Pruning => &self.skin.pruning,
        }
    }

    fn write_line_size<W>(
        &self,
        f: &mut W,
        line: &TreeLine,
        total_size: FileSize,
        selected: bool,
    ) -> Result<(), termimad::Error>
            where W: Write
    {
        if let Some(s) = line.size {
            let pb = ProgressBar::new(s.part_of(total_size), 10);
            cond_bg!(size_style, self, selected, self.name_style(&line));
            cond_bg!(sparse_style, self, selected, self.skin.sparse);
            size_style.queue(f, format!("{:>5}", s.to_string()))?;
            sparse_style.queue(f, if s.sparse { 's' } else { ' ' })?;
            size_style.queue(f, format!("{:<10} ", pb))
        } else {
            self.skin.tree.queue_str(f, "──────────────── ")
        }
    }

    fn write_line_git_status<W>(
        &self,
        f: &mut W,
        line: &TreeLine,
    ) -> Result<(), termimad::Error>
        where W: Write
    {
        if !line.is_selectable() {
            self.skin.tree.queue(f, ' ')
        } else {
            match line.git_status.map(|s| s.status) {
                Some(Status::CURRENT) => self.skin.git_status_current.queue(f, ' '),
                Some(Status::WT_NEW) => self.skin.git_status_new.queue(f, 'N'),
                Some(Status::CONFLICTED) => self.skin.git_status_conflicted.queue(f, 'C'),
                Some(Status::WT_MODIFIED) => self.skin.git_status_modified.queue(f, 'M'),
                Some(Status::IGNORED) => self.skin.git_status_ignored.queue(f, 'I'),
                None => self.skin.tree.queue(f, ' '),
                _ => self.skin.git_status_other.queue_str(f, "?"),
            }
        }
    }

    fn write_date<W>(
        &self,
        f: &mut W,
        system_time: SystemTime,
        selected: bool,
    ) -> Result<(), termimad::Error>
        where W: Write
    {
        let date_time: DateTime<Local> = system_time.into();
        cond_bg!(date_style, self, selected, self.skin.dates);
        date_style.queue(f, date_time.format("%Y/%m/%d %R ").to_string())
    }

    #[cfg(unix)]
    fn write_mode<W>(
        &self,
        f: &mut W,
        mode: Mode,
        selected: bool,
    ) -> Result<(), termimad::Error>
        where W: Write
    {
        cond_bg!(n_style, self, selected, self.skin.perm__);
        cond_bg!(r_style, self, selected, self.skin.perm_r);
        cond_bg!(w_style, self, selected, self.skin.perm_w);
        cond_bg!(x_style, self, selected, self.skin.perm_x);

        if mode.has(USER_READ) {
            r_style.queue(f, 'r')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(USER_WRITE) {
            w_style.queue(f, 'w')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(USER_EXEC) {
            x_style.queue(f, 'x')?;
        } else {
            n_style.queue(f, '_')?;
        }

        if mode.has(GROUP_READ) {
            r_style.queue(f, 'r')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(GROUP_WRITE) {
            w_style.queue(f, 'w')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(GROUP_EXEC) {
            x_style.queue(f, 'x')?;
        } else {
            n_style.queue(f, '_')?;
        }

        if mode.has(OTHERS_READ) {
            r_style.queue(f, 'r')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(OTHERS_WRITE) {
            w_style.queue(f, 'w')?;
        } else {
            n_style.queue(f, '_')?;
        }
        if mode.has(OTHERS_EXEC) {
            x_style.queue(f, 'x')?;
        } else {
            n_style.queue(f, '_')?;
        }

        Ok(())
    }

    fn write_line_name<W>(
        &self,
        f: &mut W,
        line: &TreeLine,
        pattern: &Pattern,
        selected: bool,
    ) -> Result<(), ProgramError>
        where W: Write
    {
        let style = match &line.line_type {
            LineType::Dir => &self.skin.directory,
            LineType::File => {
                if line.is_exe() {
                    &self.skin.exe
                } else {
                    &self.skin.file
                }
            }
            LineType::SymLinkToFile(_) | LineType::SymLinkToDir(_) => &self.skin.link,
            LineType::Pruning => &self.skin.pruning,
        };
        cond_bg!(style, self, selected, style);
        cond_bg!(char_match_style, self, selected, self.skin.char_match);
        pattern.style(&line.name, &style, &char_match_style).write_on(f)?;
        match &line.line_type {
            LineType::Dir => {
                if line.unlisted > 0 {
                    style.queue_str(f, " …")?;
                }
            }
            LineType::SymLinkToFile(target) | LineType::SymLinkToDir(target) => {
                style.queue_str(f, " -> ")?;
                if line.has_error {
                    self.skin.file_error.queue_str(f, &target)?;
                } else {
                    let target_style = if line.is_dir() {
                        &self.skin.directory
                    } else {
                        &self.skin.file
                    };
                    cond_bg!(target_style, self, selected, target_style);
                    target_style.queue(f, &target)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn write_root_line<W>(
        &self,
        f: &mut W,
        selected: bool,
    ) -> Result<(), ProgramError>
        where W: Write
    {
        cond_bg!(style, self, selected, self.skin.directory);
        let title = self.tree.lines[0].path.to_string_lossy();
        style.queue_str(f, &title)?;
        if self.in_app {
            self.extend_line(f, selected)?;
            let title_len = title.chars().count();
            if title_len < self.area.width as usize {
                if let ComputationResult::Done(git_status) = &self.tree.git_status {
                    let git_status_display = GitStatusDisplay::from(
                        git_status,
                        &self.skin,
                        self.area.width as usize - title_len,
                    );
                    git_status_display.write(f, selected)?;
                }
            }
        }
        Ok(())
    }

    /// if in app, extend the background till the end of screen row
    pub fn extend_line<W>(
        &self,
        f: &mut W,
        selected: bool,
    ) -> Result<(), ProgramError>
        where W: Write
    {
        if self.in_app {
            if selected {
                self.skin.selected_line.queue_bg(f)?;
            } else {
                self.skin.default.queue_bg(f)?;
            }
            f.queue(Clear(ClearType::UntilNewLine))?;
        }
        Ok(())
    }

    /// write the whole tree on the given `W`
    pub fn write_on<W>(
        &self,
        f: &mut W,
    ) -> Result<(), ProgramError>
        where W: Write
    {
        let tree = self.tree;
        #[cfg(unix)]
        let user_group_max_lengths = user_group_max_lengths(&tree);
        let total_size = tree.total_size();
        let scrollbar = if self.in_app {
            self.area.scrollbar(tree.scroll, tree.lines.len() as i32 - 1)
        } else {
            None
        };
        if self.in_app {
            f.queue(cursor::MoveTo(self.area.left, self.area.top))?;
        }
        self.write_root_line(f, tree.selection==0)?;
        f.queue(SetBackgroundColor(Color::Reset))?;
        for y in 1..self.area.height {
            if self.in_app {
                f.queue(cursor::MoveTo(self.area.left, y + self.area.top))?;
            } else {
                write!(f, "\r\n")?;
            }
            let mut line_index = y as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            let mut selected = false;
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                selected = self.in_app && line_index == tree.selection;
                if !tree.git_status.is_none() {
                    self.write_line_git_status(f, line)?;
                }
                for depth in 0..line.depth {
                    self.skin.tree.queue_str(
                        f,
                        if line.left_branchs[depth as usize] {
                            if self.tree.has_branch(line_index + 1, depth as usize) {
                                if depth == line.depth - 1 {
                                    "├──"
                                } else {
                                    "│  "
                                }
                            } else {
                                "└──"
                            }
                        } else {
                            "   "
                        },
                    )?;
                }
                if tree.options.show_sizes {
                    self.write_line_size(f, line, total_size, selected)?;
                }
                #[cfg(unix)]
                {
                    if tree.options.show_permissions {
                        if line.is_selectable() {
                            self.write_mode(f, line.mode(), selected)?;
                            let owner = permissions::user_name(line.metadata.uid());
                            cond_bg!(owner_style, self, selected, self.skin.owner);
                            owner_style.queue(f, format!(" {:w$}", &owner, w = user_group_max_lengths.0,))?;
                            let group = permissions::group_name(line.metadata.gid());
                            cond_bg!(group_style, self, selected, self.skin.group);
                            group_style.queue(f, format!(" {:w$} ", &group, w = user_group_max_lengths.1,))?;
                        } else {
                            let length = 9 + 1 +user_group_max_lengths.0 + 1 + user_group_max_lengths.1 + 1;
                            for _ in 0..length {
                                self.skin.tree.queue_str(f, "─")?;
                            }
                        }
                    }
                }
                if tree.options.show_dates {
                    if let Some(date) = line.modified() {
                        self.write_date(f, date, selected)?;
                    } else {
                        self.skin.tree.queue_str(f, "─────────────────")?;
                    }
                }
                self.write_line_name(f, line, &tree.options.pattern, selected)?;
            }
            self.extend_line(f, selected)?;
            f.queue(SetBackgroundColor(Color::Reset))?;
            if self.in_app && y > 0 {
                if let Some((sctop, scbottom)) = scrollbar {
                    f.queue(cursor::MoveTo(self.area.width, y))?;
                    let style = if sctop <= y && y <= scbottom {
                        &self.skin.scrollbar_thumb
                    } else {
                        &self.skin.scrollbar_track
                    };
                    style.queue_str(f, "▐")?;
                }
            }
        }
        if !self.in_app {
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}

#[cfg(unix)]
fn user_group_max_lengths(tree: &Tree) -> (usize, usize) {
    let mut max_user_len = 0;
    let mut max_group_len = 0;
    if tree.options.show_permissions {
        for i in 1..tree.lines.len() {
            let line = &tree.lines[i];
            let user = permissions::user_name(line.metadata.uid());
            max_user_len = max_user_len.max(user.len());
            let group = permissions::group_name(line.metadata.gid());
            max_group_len = max_group_len.max(group.len());
        }
    }
    (max_user_len, max_group_len)
}

