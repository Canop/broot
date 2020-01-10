use {
    crate::{
        file_sizes::Size,
        flat_tree::{LineType, Tree, TreeLine},
        errors::ProgramError,
        patterns::Pattern,
        skin::Skin,
    },
    chrono::{offset::Local, DateTime},
    crossterm::{
        cursor,
        style::{Color, SetBackgroundColor},
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    std::{time::SystemTime},
    termimad::{
        CompoundStyle,
        ProgressBar,
    },
};

#[cfg(unix)]
use {
    crate::permissions,
    std::os::unix::fs::MetadataExt,
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
    }
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

impl<'s, 't> DisplayableTree<'s, 't> {

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

    fn write_line_size<F>(
        &self,
        f: &mut F,
        line: &TreeLine,
        total_size: Size,
        selected: bool,
    ) -> Result<(), termimad::Error> where F: std::io::Write {
        if let Some(s) = line.size {
            let pb = ProgressBar::new(s.part_of(total_size), 10);
            cond_bg!(size_style, self, selected, self.name_style(&line));
            size_style.queue(f, format!("{:>5} {:<10} ", s.to_string(), pb))
        } else {
            self.skin.tree.queue_str(f, "──────────────── ")
        }
    }

    fn write_date<F>(
        &self,
        f: &mut F,
        system_time: SystemTime,
        selected: bool,
    ) -> Result<(), termimad::Error> where F: std::io::Write {
        let date_time: DateTime<Local> = system_time.into();
        cond_bg!(date_style, self, selected, self.skin.dates);
        date_style.queue(f, date_time.format("%Y/%m/%d %R ").to_string())
    }

    fn write_line_name<F>(
        &self,
        f: &mut F,
        line: &TreeLine,
        idx: usize,
        pattern: &Pattern,
        selected: bool,
    ) -> Result<(), ProgramError> where F: std::io::Write {
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
        if idx == 0 {
            style.queue_str(f, &line.path.to_string_lossy())?;
        } else {
            pattern.style(&line.name, &style, &char_match_style).write_on(f)?;
        }
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

    pub fn write_on<F>(
        &self,
        f: &mut F,
    ) -> Result<(), ProgramError> where F: std::io::Write {
        let tree = self.tree;
        #[cfg(unix)]
        let user_group_max_lengths = user_group_max_lengths(&tree);
        let total_size = tree.total_size();
        let scrollbar = if self.in_app {
            self.area.scrollbar(tree.scroll, tree.lines.len() as i32)
        } else {
            None
        };
        for y in 0..self.area.height {
            if self.in_app {
                f.queue(cursor::MoveTo(0, y))?;
            }
            let mut line_index = y as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            let mut selected = false;
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                selected = self.in_app && line_index == tree.selection;
                for depth in 0..line.depth {
                    self.skin.tree.queue_str(
                        f,
                        if line.left_branchs[depth as usize] {
                            if self.tree.has_branch(line_index + 1, depth as usize) {
                                if depth == line.depth - 1 {
                                    "├──"
                                } else {
                                    "│  "
                                }
                            } else {
                                "└──"
                            }
                        } else {
                            "   "
                        },
                    )?;
                }
                if tree.options.show_sizes && line_index > 0 {
                    self.write_line_size(f, line, total_size, selected)?;
                }
                #[cfg(unix)]
                {
                    if tree.options.show_permissions && line_index > 0 {
                        if line.is_selectable() {
                            cond_bg!(perm_style, self, selected, self.skin.permissions);
                            perm_style.queue(f, line.mode())?;
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
                if tree.options.show_dates && line_index > 0 {
                    if let Some(date) = line.modified() {
                        self.write_date(f, date, selected)?;
                    } else {
                        self.skin.tree.queue_str(f, "─────────────────")?;
                    }
                }
                self.write_line_name(f, line, line_index, &tree.options.pattern, selected)?;
            }
            if selected {
                self.skin.selected_line.queue_bg(f)?;
            } else {
                self.skin.default.queue_bg(f)?;
            }
            if self.in_app {
                f.queue(Clear(ClearType::UntilNewLine))?;
                f.queue(SetBackgroundColor(Color::Reset))?; // to end selection background
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

