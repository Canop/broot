use std::borrow::Cow;
use std::io::{self, Cursor, Write};
use crossterm::{Attribute::{self, Reset}, ClearType, Color::{self, *}, Colored, Color::AnsiValue, TerminalCursor};

use crate::file_sizes::Size;
use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::patterns::Pattern;
use crate::permissions;
use crate::screens::{Screen, ScreenArea};
use crate::skin::{self, Skin, SkinEntry};

/// A tree writer, which can be used either to write on the screen in the application,
/// or to write in a file or an exported string.
/// Using it in the application (with in_app true) means
///  - there will be a goto at start
///  - the selection is drawn
///  - a scrollbar may be drawn
///  - the empty lines will be erased
pub struct TreeView<'a> {
    pub w: u16,
    pub h: u16, // height of the tree part (so 2 lines less than the screen)
    pub out: &'a mut Write,
    pub skin: &'a Skin,
    pub in_app: bool,
}

impl TreeView<'_> {

    pub fn from_screen<'a>(screen: &'a Screen, out: &'a mut Write) -> TreeView<'a> {
            TreeView {
                w: screen.w,
                h: screen.h-2,
                out,
                //out: &mut crossterm::Terminal::new(),
                skin: &screen.skin,
                in_app: true,
            }
    }

    pub fn write_tree(&mut self, tree: &Tree) -> io::Result<()> {
        let mut max_user_len = 0;
        let mut max_group_len = 0;
        if tree.options.show_permissions {
            // we compute the max size of user/group names to reserve width for the columns
            for i in 1..tree.lines.len() {
                let line = &tree.lines[i];
                let user = permissions::user_name(line.uid);
                max_user_len = max_user_len.max(user.len());
                let group = permissions::group_name(line.gid);
                max_group_len = max_group_len.max(group.len());
            }
        }
        let total_size = tree.total_size();
        let area = ScreenArea {
            top: 1,
            bottom: self.h + 1,
            scroll: tree.scroll,
            content_length: tree.lines.len() as i32,
            width: self.w,
        };
        let cursor = TerminalCursor::new(); // TODO clean this out
        let terminal = crossterm::Terminal::new(); // TODO clean this out
        let scrollbar = area.scrollbar();
        for y in 1..self.h + 1 {
            let mut line_index = (y - 1) as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                self.skin.tree.print_fg();
                for depth in 0..line.depth {
                    print!(
                        "{}",
                        if line.left_branchs[depth as usize] {
                            if tree.has_branch(line_index + 1, depth as usize) {
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
                    );
                }
                if tree.options.show_sizes && line_index > 0 {
                    self.write_line_size(line, total_size);
                }
                if tree.options.show_permissions && line_index > 0 {
                    if line.is_selectable() {
                        self.write_mode(line.mode);
                        let user = permissions::user_name(line.uid);
                        print!(
                            " {:w$}",
                            &user,
                            w = max_user_len,
                        );
                        let group = permissions::group_name(line.gid);
                        print!(
                            " {:w$} ",
                            &group,
                            w = max_group_len,
                        );
                    } else {
                        print!(
                            "{}",
                            self.skin.tree.apply_to("──────────────"),
                        );
                    }
                }
                let selected = self.in_app && line_index == tree.selection;
                if selected {
                    self.skin.selected_line.print_bg();
                }
                self.write_line_name(line, line_index, &tree.options.pattern, selected)?;
                if selected {
                    // hack to extend selection background -> improve
                    self.skin.selected_line.print_bg();
                    terminal.clear(ClearType::UntilNewLine).unwrap();
                }
            } else if !self.in_app {
                print!("\r\n",);
                break; // no need to add empty lines
            }
            if self.in_app {
                terminal.clear(ClearType::UntilNewLine).unwrap();
                if let Some((sctop, scbottom)) = scrollbar {
                    if sctop <= y && y <= scbottom {
                        cursor.goto(self.w-1, y-1).unwrap();
                    }
                }
            }
            print!("{}\r\n", Attribute::Reset);
        }
        self.out.flush()?;
        Ok(())
    }

    fn write_mode(&mut self, mode: u32) {
        self.skin.permissions.print_fg();
        print!(
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
        )
    }

    fn write_line_size(&mut self, line: &TreeLine, total_size: Size) {
        if let Some(s) = line.size {
            let dr: usize = s.discrete_ratio(total_size, 8) as usize;
            let s: Vec<char> = s.to_string().chars().collect();
            self.skin.size_text.print_fg();
            self.skin.size_bar_full.print_bg();
            for i in 0..dr {
                print!("{}", if i < s.len() { s[i] } else { ' ' });
            }
            self.skin.size_bar_void.print_bg();
            for i in dr..8 {
                print!("{}", if i < s.len() { s[i] } else { ' ' });
            }
            skin::reset();
            print!(" ");
        } else {
            self.skin.tree.print_string("──────── ");
        }
    }

    fn write_line_name(
        &mut self,
        line: &TreeLine,
        idx: usize,
        pattern: &Pattern,
        selected: bool,
    ) -> io::Result<()> {
        let style = match &line.line_type {
            LineType::Dir => &self.skin.directory,
            LineType::File => {
                if line.is_exe() { &self.skin.exe } else { &self.skin.file }
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
            print!(
                "{}",
                style.apply_to(&line.path.to_string_lossy()),
            );
        } else {
            print!(
                "{}",
                pattern.style(
                    &line.name,
                    &style,
                    &self.skin.char_match,
                ),
            );
        }
        match &line.line_type {
            LineType::Dir => {
                if line.unlisted > 0 {
                    style.print_string(" …",);
                }
            }
            LineType::SymLinkToFile(target) | LineType::SymLinkToDir(target) => {
                style.print_string(" -> ");
                if line.has_error {
                    self.skin.file_error.print_string(&target);
                } else {
                    let target_style = if line.is_dir() { &self.skin.directory } else { &self.skin.file };
                    let mut target_style = target_style.clone();
                    if selected {
                        if let Some(c) = self.skin.selected_line.bg_color {
                            target_style = target_style.bg(c);
                        }
                    }
                    target_style.print_string(&target);
                }
            }
            _ => { }
        }
        Ok(())
    }
}

