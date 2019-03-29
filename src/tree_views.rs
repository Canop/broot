use std::borrow::Cow;
use std::io::{self, Write};

use crate::file_sizes::Size;
use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::patterns::Pattern;
use crate::permissions;
use crate::screens::{Screen, ScreenArea};
use crate::skin::Skin;

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

    pub fn from_screen(screen: &mut Screen) -> TreeView<'_> {
            TreeView {
                w: screen.w,
                h: screen.h-2,
                out: &mut screen.alternate_screen.screen,
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
        let scrollbar = area.scrollbar();
        if self.in_app {
            write!(self.out, "{}", termion::cursor::Goto(1, 1))?;
        }
        for y in 1..self.h + 1 {
            let mut line_index = (y - 1) as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                write!(self.out, "{}", self.skin.tree.fgbg())?;
                for depth in 0..line.depth {
                    write!(
                        self.out,
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
                    )?;
                }
                if tree.options.show_sizes && line_index > 0 {
                    self.write_line_size(line, total_size)?;
                }
                if tree.options.show_permissions && line_index > 0 {
                    if line.is_selectable() {
                        self.write_mode(line.mode)?;
                        let user = permissions::user_name(line.uid);
                        write!(
                            self.out,
                            " {:w$}",
                            &user,
                            w = max_user_len,
                        )?;
                        let group = permissions::group_name(line.gid);
                        write!(
                            self.out,
                            " {:w$} ",
                            &group,
                            w = max_group_len,
                        )?;
                    } else {
                        write!(
                            self.out,
                            "{}──────────────{}",
                            self.skin.tree.fg, self.skin.reset.fg,
                        )?;
                    }
                }
                if self.in_app && line_index == tree.selection {
                    write!(self.out, "{}", self.skin.selected_line.bg)?;
                }
                self.write_line_name(line, line_index, &tree.options.pattern)?;
            } else if !self.in_app {
                write!(self.out, "\r\n",)?;
                break; // no need to add empty lines
            }
            if self.in_app {
                write!(self.out, "{}", termion::clear::UntilNewline)?;
                if let Some((sctop, scbottom)) = scrollbar {
                    if sctop <= y && y <= scbottom {
                        write!(self.out, "{}▐", termion::cursor::Goto(self.w, y),)?;
                    }
                }
            }
            write!(self.out, "{}", self.skin.style_reset)?;
            write!(self.out, "\r\n",)?;
        }
        self.out.flush()?;
        Ok(())
    }

    fn write_mode(&mut self, mode: u32) -> io::Result<()> {
        write!(
            self.out,
            "{}{}{}{}{}{}{}{}{}{}",
            self.skin.permissions.fg,
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

    fn write_line_size(&mut self, line: &TreeLine, total_size: Size) -> io::Result<()> {
        if let Some(s) = line.size {
            let dr: usize = s.discrete_ratio(total_size, 8) as usize;
            let s: Vec<char> = s.to_string().chars().collect();
            write!(
                self.out,
                "{}{}",
                self.skin.size_text.fg, self.skin.size_bar_full.bg,
            )?;
            for i in 0..dr {
                write!(self.out, "{}", if i < s.len() { s[i] } else { ' ' })?;
            }
            write!(self.out, "{}", self.skin.size_bar_void.bg)?;
            for i in dr..8 {
                write!(self.out, "{}", if i < s.len() { s[i] } else { ' ' })?;
            }
            write!(self.out, "{}{} ", self.skin.reset.fg, self.skin.reset.bg,)
        } else {
            write!(
                self.out,
                "{}────────{} ",
                self.skin.tree.fg, self.skin.reset.fg,
            )
        }
    }

    fn write_line_name(
        &mut self,
        line: &TreeLine,
        idx: usize,
        pattern: &Pattern,
    ) -> io::Result<()> {
        match &line.line_type {
            LineType::Dir => {
                if idx == 0 {
                    write!(
                        self.out,
                        "{}{}{}",
                        self.skin.style_folder,
                        &self.skin.directory.fg,
                        &line.path.to_string_lossy(),
                    )?;
                } else {
                    write!(
                        self.out,
                        "{}{}{}",
                        self.skin.style_folder,
                        &self.skin.directory.fg,
                        decorated_name(
                            &line.name,
                            pattern,
                            &self.skin.char_match.fg,
                            &self.skin.directory.fg
                        ),
                    )?;
                    if line.unlisted > 0 {
                        write!(self.out, " …",)?;
                    }
                }
            }
            LineType::File => {
                if line.is_exe() {
                    write!(
                        self.out,
                        "{}{}",
                        &self.skin.exe.fg,
                        decorated_name(
                            &line.name,
                            pattern,
                            &self.skin.char_match.fg,
                            &self.skin.exe.fg
                        ),
                    )?;
                } else {
                    write!(
                        self.out,
                        "{}{}",
                        &self.skin.file.fg,
                        decorated_name(
                            &line.name,
                            pattern,
                            &self.skin.char_match.fg,
                            &self.skin.file.fg
                        ),
                    )?;
                }
            }
            LineType::SymLinkToFile(target) => {
                write!(
                    self.out,
                    "{}{} {}->{} {}",
                    &self.skin.link.fg,
                    decorated_name(
                        &line.name,
                        pattern,
                        &self.skin.char_match.fg,
                        &self.skin.link.fg
                    ),
                    if line.has_error {
                        &self.skin.file_error.fg
                    } else {
                        &self.skin.link.fg
                    },
                    &self.skin.file.fg,
                    &target,
                )?;
            }
            LineType::SymLinkToDir(target) => {
                write!(
                    self.out,
                    "{}{} {}->{}{} {}",
                    &self.skin.link.fg,
                    decorated_name(
                        &line.name,
                        pattern,
                        &self.skin.char_match.fg,
                        &self.skin.link.fg
                    ),
                    if line.has_error {
                        &self.skin.file_error.fg
                    } else {
                        &self.skin.link.fg
                    },
                    self.skin.style_folder,
                    &self.skin.directory.fg,
                    &target,
                )?;
            }
            LineType::Pruning => {
                write!(
                    self.out,
                    //"{}{}… {} unlisted", still not sure whether I want this '…'
                    "{}{}{} unlisted",
                    self.skin.unlisted.fg,
                    self.skin.style_pruning,
                    &line.unlisted,
                )?;
            }
        }
        Ok(())
    }
}

fn decorated_name<'a>(
    name: &'a str,
    pattern: &Pattern,
    prefix: &str,
    postfix: &str,
) -> Cow<'a, str> {
    if pattern.is_some() {
        if let Some(m) = pattern.find(name) {
            return Cow::Owned(m.wrap_matching_chars(name, prefix, postfix));
        }
    }
    Cow::Borrowed(name)
}
