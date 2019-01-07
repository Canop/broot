#![warn(clippy::all)]

use std::borrow::Cow;
use std::io::{self, Write};
use termion::{color, style};
use users::{Users, Groups, UsersCache};
use std::sync::Mutex;

use crate::flat_tree::{LineType, Tree, TreeLine};
use crate::patterns::Pattern;
use crate::screens::{Screen, ScreenArea};

pub trait TreeView {
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()>;
    fn write_line_name(
        &mut self,
        line: &TreeLine,
        idx: usize,
        pattern: &Option<Pattern>,
    ) -> io::Result<()>;
}

impl TreeView for Screen {
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()> {
        lazy_static! {
            static ref users_cache_mutex: Mutex<UsersCache> = Mutex::new(UsersCache::new());
        }
        let users_cache = users_cache_mutex.lock().unwrap();
        let mut max_user_name_len = 0;
        let mut max_group_name_len = 0;
        if tree.options.show_permissions {
            // we compute the max size of user/group names to reserve width for the columns
            for i in 1..tree.lines.len() {
                let line = &tree.lines[i];
                if let Some(user) = users_cache.get_user_by_uid(line.uid) {
                    max_user_name_len = max_user_name_len.max(user.name().to_string_lossy().len());
                }
                if let Some(group) = users_cache.get_group_by_gid(line.uid) {
                    max_group_name_len = max_group_name_len.max(group.name().to_string_lossy().len());
                }
            }
        }
        let total_size = tree.total_size();
        let area = ScreenArea {
            top: 1,
            bottom: self.h - 1,
            scroll: tree.scroll,
            content_length: tree.lines.len() as i32,
        };
        let scrollbar = area.scrollbar();
        for y in 1..self.h - 1 {
            write!(self.stdout, "{}", termion::cursor::Goto(1, y),)?;
            let mut line_index = (y - 1) as usize;
            if line_index > 0 {
                line_index += tree.scroll as usize;
            }
            if line_index < tree.lines.len() {
                let line = &tree.lines[line_index];
                for depth in 0..line.depth {
                    write!(
                        self.stdout,
                        "{}{}{}",
                        color::Fg(color::AnsiValue::grayscale(5)),
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
                        color::Fg(color::Reset),
                    )?;
                }
                if tree.options.show_sizes && line_index > 0 {
                    if let Some(s) = line.size {
                        let dr: usize = s.discreet_ratio(total_size, 8) as usize;
                        let s: Vec<char> = s.to_string().chars().collect();
                        write!(
                            self.stdout,
                            "{}{}",
                            color::Bg(color::Magenta),
                            color::Fg(color::AnsiValue::grayscale(15)),
                        )?;
                        for i in 0..dr {
                            write!(self.stdout, "{}", if i < s.len() { s[i] } else { ' ' })?;
                        }
                        write!(self.stdout, "{}", color::Bg(color::Reset))?;
                        write!(self.stdout, "{}", color::Bg(color::AnsiValue::grayscale(2)))?;
                        for i in dr..8 {
                            write!(self.stdout, "{}", if i < s.len() { s[i] } else { ' ' })?;
                        }
                        write!(
                            self.stdout,
                            "{}{} ",
                            color::Bg(color::Reset),
                            color::Fg(color::Reset),
                        )?;
                    } else {
                        write!(
                            self.stdout,
                            "{}────────{} ",
                            color::Fg(color::AnsiValue::grayscale(5)),
                            color::Fg(color::Reset),
                        )?;
                    }
                }
                if tree.options.show_permissions && line_index > 0 {
                    if line.is_selectable() {
                        write!(
                            self.stdout,
                            "{} {}{}{}{}{}{}{}{}{}",
                            color::Fg(color::AnsiValue::grayscale(15)),
                            if (line.mode & (1<<8))!=0 { 'r' } else { '-' },
                            if (line.mode & (1<<7))!=0 { 'w' } else { '-' },
                            if (line.mode & (1<<6))!=0 { 'x' } else { '-' },
                            if (line.mode & (1<<5))!=0 { 'r' } else { '-' },
                            if (line.mode & (1<<4))!=0 { 'w' } else { '-' },
                            if (line.mode & (1<<3))!=0 { 'x' } else { '-' },
                            if (line.mode & (1<<2))!=0 { 'r' } else { '-' },
                            if (line.mode & (1<<1))!=0 { 'w' } else { '-' },
                            if (line.mode & 1)     !=0 { 'x' } else { '-' },
                        )?;
                        if let Some(user) = users_cache.get_user_by_uid(line.uid) {
                            write!(
                                self.stdout,
                                " {:w$}",
                                user.name().to_string_lossy(),
                                w=max_user_name_len,
                            )?;
                        }
                        if let Some(group) = users_cache.get_group_by_gid(line.uid) {
                            write!(
                                self.stdout,
                                " {:w$} ",
                                group.name().to_string_lossy(),
                                w=max_group_name_len,
                            )?;
                        }
                    } else {
                        write!(
                            self.stdout,
                            "{}──────────────{}",
                            color::Fg(color::AnsiValue::grayscale(5)),
                            color::Fg(color::Reset),
                        )?;

                    }
                }
                let selected = line_index == tree.selection;
                if selected {
                    write!(self.stdout, "{}", color::Bg(color::AnsiValue::grayscale(2)),)?;
                }
                self.write_line_name(line, line_index, &tree.options.pattern)?;
            }
            write!(
                self.stdout,
                "{}{}{}",
                termion::clear::UntilNewline,
                style::Reset,
                color::Fg(color::AnsiValue::grayscale(9)),
            )?;
            if let Some((sctop, scbottom)) = scrollbar {
                if sctop <= y && y <= scbottom {
                    write!(self.stdout, "{}▐", termion::cursor::Goto(self.w, y),)?;
                }
            }
            write!(self.stdout, "{}", color::Fg(color::Reset),)?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    fn write_line_name(
        &mut self,
        line: &TreeLine,
        idx: usize,
        pattern: &Option<Pattern>,
    ) -> io::Result<()> {
        lazy_static! {
            static ref fg_reset: String = format!("{}", color::Fg(color::White)).to_string();
            static ref fg_dir: String = format!("{}", color::Fg(color::LightBlue)).to_string();
            static ref fg_link: String = format!("{}", color::Fg(color::LightMagenta)).to_string();
            static ref fg_match: String = format!("{}", color::Fg(color::Green)).to_string();
            static ref fg_reset_dir: String = format!("{}{}", &*fg_reset, &*fg_dir).to_string();
            static ref fg_reset_link: String = format!("{}{}", &*fg_reset, &*fg_link).to_string();
        }
        // TODO draw in red lines with has_error
        match &line.line_type {
            LineType::Dir => {
                if idx == 0 {
                    write!(
                        self.stdout,
                        "{}{}{}",
                        style::Bold,
                        &*fg_dir,
                        &line.path.to_string_lossy(),
                    )?;
                } else {
                    write!(
                        self.stdout,
                        "{}{}{}",
                        style::Bold,
                        &*fg_dir,
                        decorated_name(&line.name, pattern, &*fg_match, &*fg_reset_dir),
                    )?;
                    if line.unlisted > 0 {
                        write!(self.stdout, " …",)?;
                    }
                }
            }
            LineType::File => {
                write!(
                    self.stdout,
                    "{}{}",
                    &*fg_reset,
                    decorated_name(&line.name, pattern, &*fg_match, &*fg_reset),
                )?;
            }
            LineType::SymLink(target) => {
                write!(
                    self.stdout,
                    "{}{} {}->{} {}",
                    &*fg_reset,
                    decorated_name(&line.name, pattern, &*fg_match, &*fg_reset),
                    &*fg_link,
                    &*fg_reset,
                    decorated_name(&target, pattern, &*fg_match, &*fg_reset),
                )?;
            }
            LineType::Pruning => {
                write!(
                    self.stdout,
                    //"{}{}… {} unlisted", still not sure whether I want this '…'
                    "{}{} {} unlisted",
                    color::Fg(color::AnsiValue::grayscale(13)),
                    style::Italic,
                    &line.unlisted,
                )?;
            }
        }
        Ok(())
    }
}

fn decorated_name<'a>(
    name: &'a str,
    pattern: &Option<Pattern>,
    prefix: &str,
    postfix: &str,
) -> Cow<'a, str> {
    if let Some(p) = pattern {
        if let Some(m) = p.test(name) {
            return Cow::Owned(m.wrap_matching_chars(name, prefix, postfix));
        }
    }
    Cow::Borrowed(name)
}
