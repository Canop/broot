use {
    super::{CropWriter, GitStatusDisplay},
    crate::{
        content_search::ContentMatch,
        errors::ProgramError,
        file_sizes::FileSize,
        pattern::Pattern,
        skin::StyleMap,
        task_sync::ComputationResult,
        tree::{Tree, TreeLine, TreeLineType},
    },
    chrono::{offset::Local, DateTime},
    crossterm::{
        cursor,
        style::{Color, SetBackgroundColor},
        terminal::ClearType,
        QueueableCommand,
    },
    git2::Status,
    std::{io::Write, time::SystemTime},
    termimad::{CompoundStyle, ProgressBar},
};

#[cfg(unix)]
use {crate::permissions, std::os::unix::fs::MetadataExt, umask::*};

/// A tree wrapper which can be used either
/// - to write on the screen in the application,
/// - or to write in a file or an exported string.
/// Using it in the application (with in_app true) means that
///  - the selection is drawn
///  - a scrollbar may be drawn
///  - the empty lines will be erased
pub struct DisplayableTree<'s, 't> {
    pub tree: &'t Tree,
    pub skin: &'s StyleMap,
    pub area: termimad::Area,
    pub in_app: bool, // if true we show the selection and scrollbar
}

impl<'s, 't> DisplayableTree<'s, 't> {

    pub fn out_of_app(tree: &'t Tree, skin: &'s StyleMap, width: u16) -> DisplayableTree<'s, 't> {
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
            TreeLineType::Dir => &self.skin.directory,
            TreeLineType::File => {
                if line.is_exe() {
                    &self.skin.exe
                } else {
                    &self.skin.file
                }
            }
            TreeLineType::SymLinkToFile(_) | TreeLineType::SymLinkToDir(_) => &self.skin.link,
            TreeLineType::Pruning => &self.skin.pruning,
        }
    }

    fn write_line_size<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        total_size: FileSize,
        selected: bool,
    ) -> Result<(), termimad::Error>
    where
        W: Write,
    {
        if let Some(s) = line.size {
            let pb = ProgressBar::new(s.part_of(total_size), 10);
            cond_bg!(size_style, self, selected, self.name_style(&line));
            cond_bg!(sparse_style, self, selected, self.skin.sparse);
            cw.queue_string(&size_style, format!("{:>5}", s.to_string()))?;
            cw.queue_char(&sparse_style, if s.sparse { 's' } else { ' ' })?;
            cw.queue_string(&size_style, format!("{:<10} ", pb))
        } else {
            cw.queue_str(&self.skin.tree, "──────────────── ")
        }
    }

    fn write_line_git_status<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
    ) -> Result<(), termimad::Error>
    where
        W: Write,
    {
        if !line.is_selectable() {
            cw.queue_char(&self.skin.tree, ' ')
        } else {
            match line.git_status.map(|s| s.status) {
                Some(Status::CURRENT) => cw.queue_char(&self.skin.git_status_current, ' '),
                Some(Status::WT_NEW) => cw.queue_char(&self.skin.git_status_new, 'N'),
                Some(Status::CONFLICTED) => cw.queue_char(&self.skin.git_status_conflicted, 'C'),
                Some(Status::WT_MODIFIED) => cw.queue_char(&self.skin.git_status_modified, 'M'),
                Some(Status::IGNORED) => cw.queue_char(&self.skin.git_status_ignored, 'I'),
                None => cw.queue_char(&self.skin.tree, ' '),
                _ => cw.queue_char(&self.skin.git_status_other, '?'),
            }
        }
    }

    fn write_date<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        system_time: SystemTime,
        selected: bool,
    ) -> Result<(), termimad::Error>
    where
        W: Write,
    {
        let date_time: DateTime<Local> = system_time.into();
        cond_bg!(date_style, self, selected, self.skin.dates);
        cw.queue_string(date_style, date_time.format(self.tree.options.date_time_format).to_string())
    }

    #[cfg(unix)]
    fn write_mode<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        mode: Mode,
        selected: bool,
    ) -> Result<(), termimad::Error>
    where
        W: Write,
    {
        cond_bg!(n_style, self, selected, self.skin.perm__);
        cond_bg!(r_style, self, selected, self.skin.perm_r);
        cond_bg!(w_style, self, selected, self.skin.perm_w);
        cond_bg!(x_style, self, selected, self.skin.perm_x);

        if mode.has(USER_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(USER_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(USER_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        if mode.has(GROUP_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(GROUP_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(GROUP_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        if mode.has(OTHERS_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(OTHERS_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(OTHERS_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        Ok(())
    }

    fn write_line_name<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        pattern: &Pattern,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: Write,
    {
        let style = match &line.line_type {
            TreeLineType::Dir => &self.skin.directory,
            TreeLineType::File => {
                if line.is_exe() {
                    &self.skin.exe
                } else {
                    &self.skin.file
                }
            }
            TreeLineType::SymLinkToFile(_) | TreeLineType::SymLinkToDir(_) => &self.skin.link,
            TreeLineType::Pruning => &self.skin.pruning,
        };
        cond_bg!(style, self, selected, style);
        cond_bg!(char_match_style, self, selected, self.skin.char_match);
        pattern
            .style(&line.name, &style, &char_match_style)
            .write_on(cw)?;
        match &line.line_type {
            TreeLineType::Dir => {
                if line.unlisted > 0 {
                    cw.queue_str(style, " …")?;
                }
            }
            TreeLineType::SymLinkToFile(target) | TreeLineType::SymLinkToDir(target) => {
                cw.queue_str(style, " -> ")?;
                if line.has_error {
                    cw.queue_str(&self.skin.file_error, &target)?;
                } else {
                    let target_style = if line.is_dir() {
                        &self.skin.directory
                    } else {
                        &self.skin.file
                    };
                    cond_bg!(target_style, self, selected, target_style);
                    cw.queue_str(target_style, &target)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn write_content_extract<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        extract: ContentMatch,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: Write,
    {
        cond_bg!(extract_style, self, selected, self.skin.content_extract);
        cond_bg!(match_style, self, selected, self.skin.content_match);
        cw.queue_char(&extract_style, ' ')?;
        if extract.needle_start > 0 {
            cw.queue_str(&extract_style, &extract.extract[0..extract.needle_start])?;
        }
        cw.queue_str(&match_style, &extract.extract[extract.needle_start..extract.needle_end])?;
        if extract.needle_end < extract.extract.len() {
            cw.queue_str(&extract_style, &extract.extract[extract.needle_end..])?;
        }
        Ok(())
    }

    pub fn write_root_line<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: Write,
    {
        cond_bg!(style, self, selected, self.skin.directory);
        let title = self.tree.lines[0].path.to_string_lossy();
        cw.queue_str(&style, &title)?;
        if self.in_app {
            self.extend_line_bg(cw, selected)?;
            let title_len = title.chars().count();
            if title_len < self.area.width as usize {
                if let ComputationResult::Done(git_status) = &self.tree.git_status {
                    let git_status_display = GitStatusDisplay::from(
                        git_status,
                        &self.skin,
                        self.area.width as usize - title_len,
                    );
                    git_status_display.write(cw, selected)?;
                }
            }
        }
        Ok(())
    }

    /// if in app, extend the background till the end of screen row
    pub fn extend_line_bg<'w, W>(
        &self,
        cw: &mut CropWriter<'w, W>,
        selected: bool,
    ) -> Result<(), ProgramError>
    where
        W: Write,
    {
        if self.in_app {
            if selected {
                cw.queue_bg(&self.skin.selected_line)?;
            } else {
                cw.queue_bg(&self.skin.default)?;
            }
            cw.clear(ClearType::UntilNewLine)?;
        }
        Ok(())
    }

    /// write the whole tree on the given `W`
    pub fn write_on<W>(&self, f: &mut W) -> Result<(), ProgramError>
    where
        W: Write,
    {
        let tree = self.tree;
        #[cfg(unix)]
        let user_group_max_lengths = user_group_max_lengths(&tree);
        let total_size = tree.total_size();
        let scrollbar = if self.in_app {
            self.area
                .scrollbar(tree.scroll, tree.lines.len() as i32 - 1)
        } else {
            None
        };
        if self.in_app {
            f.queue(cursor::MoveTo(self.area.left, self.area.top))?;
        }
        let mut cw = CropWriter::new(f, self.area.width as usize);
        self.write_root_line(&mut cw, tree.selection == 0)?;
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
            let mut cw = CropWriter::new(f, self.area.width as usize);
            let cw = &mut cw;
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                selected = self.in_app && line_index == tree.selection;
                if !tree.git_status.is_none() {
                    self.write_line_git_status(cw, line)?;
                }
                for depth in 0..line.depth {
                    cw.queue_str(
                        &self.skin.tree,
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
                    self.write_line_size(cw, line, total_size, selected)?;
                }
                #[cfg(unix)]
                {
                    if tree.options.show_permissions {
                        if line.is_selectable() {
                            self.write_mode(cw, line.mode(), selected)?;
                            let owner = permissions::user_name(line.metadata.uid());
                            cond_bg!(owner_style, self, selected, self.skin.owner);
                            cw.queue_string(
                                &owner_style,
                                format!(" {:w$}", &owner, w = user_group_max_lengths.0,),
                            )?;
                            let group = permissions::group_name(line.metadata.gid());
                            cond_bg!(group_style, self, selected, self.skin.group);
                            cw.queue_string(
                                &group_style,
                                format!(" {:w$} ", &group, w = user_group_max_lengths.1,),
                            )?;
                        } else {
                            let length =
                                9 + 1 + user_group_max_lengths.0 + 1 + user_group_max_lengths.1 + 1;
                            for _ in 0..length {
                                cw.queue_char(&self.skin.tree, ' ')?;
                            }
                        }
                    }
                }
                if tree.options.show_dates {
                    if let Some(date) = line.modified() {
                        self.write_date(cw, date, selected)?;
                    } else {
                        cw.queue_str(&self.skin.tree, "─────────────────")?;
                    }
                }
                self.write_line_name(cw, line, &tree.options.pattern, selected)?;
                if cw.allowed > 8 {
                    let extract = tree.options.pattern.get_content_match(&line.path, cw.allowed - 2);
                    if let Some(extract) = extract {
                        debug!("extract: {:?}", extract);
                        self.write_content_extract(cw, extract, selected)?;
                    }
                }
            }
            self.extend_line_bg(cw, selected)?;
            f.queue(SetBackgroundColor(Color::Reset))?;
            if self.in_app && y > 0 {
                if let Some((sctop, scbottom)) = scrollbar {
                    f.queue(cursor::MoveTo(self.area.left + self.area.width - 1, y))?;
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
