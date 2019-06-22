use std::fmt;
use std::time::SystemTime;
use crossterm::{ClearType, Terminal};
use chrono::offset::Local;
use chrono::DateTime;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(unix)]
use crate::permissions;

use crate::file_sizes::Size;
use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::patterns::Pattern;
use crate::skin::{Skin, SkinEntry};

use crossterm::{Color, Colored, TerminalCursor};

/// A tree wrapper implementing Display
/// which can be used either
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

    fn write_line_size(
        &self,
        f: &mut fmt::Formatter<'_>,
        line: &TreeLine,
        total_size: Size,
    ) -> fmt::Result {
        if let Some(s) = line.size {
            let dr: usize = s.discrete_ratio(total_size, 8) as usize;
            let s: Vec<char> = s.to_string().chars().collect();
            let mut bar = String::new();
            for i in 0..dr {
                bar.push(if i < s.len() { s[i] } else { ' ' });
            }
            self.skin.size_bar.write(f, &bar)?;
            let mut no_bar = String::new();
            for i in dr..8 {
                no_bar.push(if i < s.len() { s[i] } else { ' ' });
            }
            self.skin.size_no_bar.write(f, &no_bar)?;
            write!(f, " ")
        } else {
            self.skin.tree.write(f, "──────── ")
        }
    }

    fn write_date(&self, f: &mut fmt::Formatter<'_>, system_time: SystemTime) -> fmt::Result {
        let date_time: DateTime<Local> = system_time.into();
        write!(
            f,
            "{}",
            self.skin.dates.apply_to(
                date_time.format("%Y/%m/%d %R ")
            ),
        )
    }

    #[cfg(unix)]
    fn write_mode(&self, f: &mut fmt::Formatter<'_>, mode: u32) -> fmt::Result {
        write!(
            f,
            "{}",
            self.skin.permissions.apply_to(format!(
                "{}{}{}{}{}{}{}{}{}",
                if (mode & (1 << 8)) != 0 { 'r' } else { '-' },
                if (mode & (1 << 7)) != 0 { 'w' } else { '-' },
                if (mode & (1 << 6)) != 0 { 'x' } else { '-' },
                if (mode & (1 << 5)) != 0 { 'r' } else { '-' },
                if (mode & (1 << 4)) != 0 { 'w' } else { '-' },
                if (mode & (1 << 3)) != 0 { 'x' } else { '-' },
                if (mode & (1 << 2)) != 0 { 'r' } else { '-' },
                if (mode & (1 << 1)) != 0 { 'w' } else { '-' },
                if (mode & 1) != 0 { 'x' } else { '-' },
            ))
        )
    }

    fn write_line_name(
        &self,
        f: &mut fmt::Formatter<'_>,
        line: &TreeLine,
        idx: usize,
        pattern: &Pattern,
        selected: bool,
    ) -> fmt::Result {
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
        let mut style = style.clone();
        if selected {
            if let Some(c) = self.skin.selected_line.bg_color {
                style = style.bg(c);
            }
        }
        if idx == 0 {
            style.write(f, &line.path.to_string_lossy())?;
        } else {
            write!(
                f,
                "{}",
                pattern.style(&line.name, &style, &self.skin.char_match,),
            )?;
        }
        match &line.line_type {
            LineType::Dir => {
                if line.unlisted > 0 {
                    style.write(f, " …")?;
                }
            }
            LineType::SymLinkToFile(target) | LineType::SymLinkToDir(target) => {
                style.write(f, " -> ")?;
                if line.has_error {
                    self.skin.file_error.write(f, &target)?;
                } else {
                    let target_style = if line.is_dir() {
                        &self.skin.directory
                    } else {
                        &self.skin.file
                    };
                    let mut target_style = target_style.clone();
                    if selected {
                        if let Some(c) = self.skin.selected_line.bg_color {
                            target_style = target_style.bg(c);
                        }
                    }
                    target_style.write(f, &target)?;
                }
            }
            _ => {}
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

impl fmt::Display for DisplayableTree<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let terminal = Terminal::new();
        let cursor = TerminalCursor::new();
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
                cursor.goto(0, y).unwrap();
            }
            let mut line_index = y as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                for depth in 0..line.depth {
                    self.skin.tree.write(
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
                    self.write_line_size(f, line, total_size)?;
                }
                #[cfg(unix)]
                {
                    if tree.options.show_permissions && line_index > 0 {
                        if line.is_selectable() {
                            self.write_mode(f, line.metadata.mode())?;
                            let user = permissions::user_name(line.metadata.uid());
                            write!(f, " {:w$}", &user, w = user_group_max_lengths.0,)?;
                            let group = permissions::group_name(line.metadata.gid());
                            write!(f, " {:w$} ", &group, w = user_group_max_lengths.1,)?;
                        } else {
                            self.skin.tree.write(f, "──────────────")?;
                        }
                    }
                }
                if tree.options.show_dates && line_index > 0 {
                    if let Ok(date) = line.metadata.modified() {
                        self.write_date(f, date)?;
                    } else {
                        self.skin.tree.write(f, "──────────────── ")?;
                    }
                }
                let selected = self.in_app && line_index == tree.selection;
                self.write_line_name(f, line, line_index, &tree.options.pattern, selected)?;
                if selected {
                    self.skin.selected_line.print_bg();
                }
            }
            if self.in_app {
                terminal.clear(ClearType::UntilNewLine).unwrap();
                write!(f, "{}", Colored::Bg(Color::Reset))?; // to end selection background
                if let Some((sctop, scbottom)) = scrollbar {
                    cursor.goto(self.area.width, y).unwrap();
                    let style = if sctop <= y && y <= scbottom {
                        &self.skin.scrollbar_thumb
                    } else {
                        &self.skin.scrollbar_track
                    };
                    style.write(f, "▐")?;
                }
            }
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}
