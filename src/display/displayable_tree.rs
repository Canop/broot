use {
    super::{
        Col,
        Cols,
        CropWriter,
        file_size,
        GitStatusDisplay,
        MatchedString,
    },
    crate::{
        content_search::ContentMatch,
        errors::ProgramError,
        file_sum::FileSum,
        pattern::PatternObject,
        skin::{ExtColorMap, StyleMap},
        task_sync::ComputationResult,
        tree::{Tree, TreeLine, TreeLineType},
    },
    chrono::{Local, DateTime, TimeZone},
    crossterm::{
        cursor,
        style::{Color, SetBackgroundColor},
        QueueableCommand,
    },
    git2::Status,
    std::io::Write,
    termimad::{CompoundStyle, ProgressBar},
};

#[cfg(unix)]
use {crate::permissions, std::os::unix::fs::MetadataExt, umask::*};

static LONG_SPACE: &str = "                                                                                                                                                                                                                                                                                                                                           ";
static LONG_BRANCH: &str = "───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────";

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
    pub cols: &'s Cols,
    pub ext_colors: &'s ExtColorMap,
}

impl<'s, 't> DisplayableTree<'s, 't> {

    pub fn out_of_app(
        tree: &'t Tree,
        skin: &'s StyleMap,
        cols: &'s Cols,
        ext_colors: &'s ExtColorMap,
        width: u16,
    ) -> DisplayableTree<'s, 't> {
        DisplayableTree {
            tree,
            skin,
            cols,
            ext_colors,
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

    fn write_line_count<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        selected: bool,
    ) -> Result<usize, termimad::Error> {
        Ok(if let Some(s) = line.sum {
            cond_bg!(count_style, self, selected, self.skin.count);
            cw.queue_string(&count_style, format!("{:>8}", s.to_count()))?;
            1
        } else {
            9
        })
    }

    fn write_line_size<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        selected: bool,
    ) -> Result<usize, termimad::Error> {
        Ok(if let Some(s) = line.sum {
            cond_bg!(size_style, self, selected, self.name_style(&line));
            cw.queue_string(
                &size_style,
                format!("{:>4}", file_size::fit_4(s.to_size())),
            )?;
            1
        } else {
            5
        })
    }

    /// only makes sense when there's only one level
    /// (so in sort mode)
    fn write_line_size_with_bar<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        total_size: FileSum,
        selected: bool,
    ) -> Result<usize, termimad::Error> {
        Ok(if let Some(s) = line.sum {
            let pb = ProgressBar::new(s.part_of_size(total_size), 10);
            cond_bg!(size_style, self, selected, self.name_style(&line));
            cond_bg!(sparse_style, self, selected, self.skin.sparse);
            cw.queue_string(
                &size_style,
                format!("{:>4}", file_size::fit_4(s.to_size())),
            )?;
            cw.queue_char(
                &sparse_style,
                if s.is_sparse() && line.is_file() { 's' } else { ' ' },
            )?;
            cw.queue_string(&size_style, format!("{:<10}", pb))?;
            1
        } else {
            16
        })
    }

    fn write_line_git_status<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        selected: bool,
    ) -> Result<usize, termimad::Error> {
        let (style, char) =
        if !line.is_selectable() {
            (&self.skin.tree, ' ')
        } else {
            match line.git_status.map(|s| s.status) {
                Some(Status::CURRENT) => (&self.skin.git_status_current, ' '),
                Some(Status::WT_NEW) => (&self.skin.git_status_new, 'N'),
                Some(Status::CONFLICTED) => (&self.skin.git_status_conflicted, 'C'),
                Some(Status::WT_MODIFIED) => (&self.skin.git_status_modified, 'M'),
                Some(Status::IGNORED) => (&self.skin.git_status_ignored, 'I'),
                None => (&self.skin.tree, ' '),
                _ => (&self.skin.git_status_other, '?'),
            }
        };
        cond_bg!(git_style, self, selected, style);
        cw.queue_char(git_style, char)?;
        Ok(0)
    }

    fn write_date<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        seconds: i64,
        selected: bool,
    ) -> Result<usize, termimad::Error> {
        let date_time: DateTime<Local> = Local.timestamp(seconds, 0);
        cond_bg!(date_style, self, selected, self.skin.dates);
        cw.queue_string(date_style, date_time.format(self.tree.options.date_time_format).to_string())?;
        Ok(1)
    }

    #[cfg(unix)]
    fn write_mode<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        mode: Mode,
        selected: bool,
    ) -> Result<(), termimad::Error> {
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

    #[cfg(unix)]
    fn write_permissions<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        user_group_max_lengths: (usize, usize),
        selected: bool,
    ) -> Result<usize, ProgramError> {
        Ok(if line.is_selectable() {
            self.write_mode(cw, line.mode(), selected)?;
            let owner = permissions::user_name(line.metadata.uid());
            cond_bg!(owner_style, self, selected, self.skin.owner);
            cw.queue_string(
                &owner_style,
                format!(" {:w$}", &owner, w = user_group_max_lengths.0),
            )?;
            let group = permissions::group_name(line.metadata.gid());
            cond_bg!(group_style, self, selected, self.skin.group);
            cw.queue_string(
                &group_style,
                format!(" {:w$}", &group, w = user_group_max_lengths.1),
            )?;
            1
        } else {
            9 + 1 + user_group_max_lengths.0 + 1 + user_group_max_lengths.1 + 1
        })
    }

    fn write_branch<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line_index: usize,
        line: &TreeLine,
        selected: bool,
    ) -> Result<usize, ProgramError> {
        cond_bg!(branch_style, self, selected, self.skin.tree);
        let mut branch = String::new();
        for depth in 0..line.depth {
            branch.push_str(
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
            );
        }
        if !branch.is_empty() {
            cw.queue_string(&branch_style, branch)?;
        }
        Ok(0)
    }

    /// write the name or subpath, depending on the pattern_object
    fn write_line_label<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        line: &TreeLine,
        pattern_object: PatternObject,
        selected: bool,
    ) -> Result<usize, ProgramError> {
        let mut style = match &line.line_type {
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
        let mut cloned_style;
        if let Some(ext_color) = line.extension().and_then(|ext| self.ext_colors.get(ext)) {
            debug!("extension: {:?}", ext_color);
            cloned_style = style.clone();
            cloned_style.set_fg(ext_color);
            style = &cloned_style;
        }
        cond_bg!(style, self, selected, style);
        cond_bg!(char_match_style, self, selected, self.skin.char_match);
        let label = if pattern_object.subpath {
            &line.subpath
        } else {
            &line.name
        };
        let name_match = self.tree.options.pattern.pattern.search_string(label);
        let matched_string = MatchedString {
            name_match,
            string: label,
            base_style: &style,
            match_style: &char_match_style,
        };
        matched_string.queue_on(cw)?;
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
        Ok(1)
    }

    fn write_content_extract<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        extract: ContentMatch,
        selected: bool,
    ) -> Result<(), ProgramError> {
        cond_bg!(extract_style, self, selected, self.skin.content_extract);
        cond_bg!(match_style, self, selected, self.skin.content_match);
        cw.queue_str(&extract_style, "  ")?;
        if extract.needle_start > 0 {
            cw.queue_str(&extract_style, &extract.extract[0..extract.needle_start])?;
        }
        cw.queue_str(&match_style, &extract.extract[extract.needle_start..extract.needle_end])?;
        if extract.needle_end < extract.extract.len() {
            cw.queue_str(&extract_style, &extract.extract[extract.needle_end..])?;
        }
        Ok(())
    }

    pub fn write_root_line<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        selected: bool,
    ) -> Result<(), ProgramError> {
        cond_bg!(style, self, selected, self.skin.directory);
        let title = self.tree.lines[0].path.to_string_lossy();
        cw.queue_str(&style, &title)?;
        if self.in_app {
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
            self.extend_line_bg(cw, selected)?;
        }
        Ok(())
    }

    /// if in app, extend the background till the end of screen row
    pub fn extend_line_bg<'w, W: Write>(
        &self,
        cw: &mut CropWriter<'w, W>,
        selected: bool,
    ) -> Result<(), ProgramError> {
        if self.in_app && !cw.is_full() {
            let style = if selected {
                &self.skin.selected_line
            } else {
                &self.skin.default
            };
            cw.fill(style, LONG_SPACE)?;
        }
        Ok(())
    }

    /// write the whole tree on the given `W`
    pub fn write_on<W: Write>(&self, f: &mut W) -> Result<(), ProgramError> {
        let tree = self.tree;
        #[cfg(unix)]
        let user_group_max_lengths = user_group_max_lengths(&tree);
        let total_size = tree.total_sum();
        let scrollbar = if self.in_app {
            self.area.scrollbar(tree.scroll, tree.lines.len() as i32 - 1)
        } else {
            None
        };
        if self.in_app {
            f.queue(cursor::MoveTo(self.area.left, self.area.top))?;
        }
        let mut cw = CropWriter::new(f, self.area.width as usize);
        let pattern_object = tree.options.pattern.pattern.object();
        self.write_root_line(&mut cw, tree.selection == 0)?;
        f.queue(SetBackgroundColor(Color::Reset))?;

        // we compute the length of the dates, depending on the format
        let date_len = if tree.options.show_dates {
            let date_time: DateTime<Local> = Local::now();
            date_time.format(tree.options.date_time_format).to_string().len()
        } else {
            0 // we don't care
        };

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
                let mut in_branch = false;
                let space_style = if selected {
                    &self.skin.selected_line
                } else {
                    &self.skin.default
                };
                cw.queue_char(space_style, ' ')?;
                for col in self.cols {
                    let void_len = match col {

                        Col::Git if !tree.git_status.is_none() => {
                            self.write_line_git_status(cw, line, selected)?
                        }

                        Col::Branch => {
                            in_branch = true;
                            self.write_branch(cw, line_index, line, selected)?
                        }

                        #[cfg(unix)]
                        Col::Permission if tree.options.show_permissions => {
                            self.write_permissions(cw, line, user_group_max_lengths, selected)?
                        }

                        Col::Date if tree.options.show_dates => {
                            if let Some(seconds) = line.sum.and_then(|sum| sum.to_valid_seconds()) {
                                self.write_date(cw, seconds, selected)?
                            } else {
                                date_len + 1
                            }
                        }

                        Col::Size if tree.options.show_sizes => {
                            if tree.options.sort.is_some() {
                                // as soon as there's only one level displayed we can show the size bars
                                self.write_line_size_with_bar(cw, line, total_size, selected)?
                            } else {
                                self.write_line_size(cw, line, selected)?
                            }
                        }

                        Col::Count if tree.options.show_counts => {
                            self.write_line_count(cw, line, selected)?
                        }

                        Col::Name => {
                            in_branch = false;
                            self.write_line_label(cw, line, pattern_object, selected)?
                        }

                        _ => {
                            0 // we don't write the intercol
                        }
                    };
                    // void: intercol & replacing missing cells
                    let (void_base_style, void) = if in_branch && void_len > 2 {
                        (&self.skin.tree, LONG_BRANCH)
                    } else {
                        (&self.skin.default, LONG_SPACE)
                    };
                    cond_bg!(void_style, self, selected, void_base_style);
                    cw.repeat(void_style, void, void_len)?;
                }

                if cw.allowed > 8 && pattern_object.content {
                    let extract = tree.options.pattern.pattern.search_content(&line.path, cw.allowed - 2);
                    if let Some(extract) = extract {
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
