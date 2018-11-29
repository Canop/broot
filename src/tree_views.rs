use std::borrow::Cow;

use flat_tree::{LineType, Tree, TreeLine};
use patterns::Pattern;
use screens::Screen;
use std::io::{self, Write};
use termion::{color, style};

pub trait TreeView {
    fn write_tree(&mut self, tree: &Tree, pattern: &Option<Pattern>) -> io::Result<()>;
    fn write_line_key(&mut self, line: &TreeLine, selected: bool) -> io::Result<()>;
    fn write_line_name(&mut self, line: &TreeLine, pattern: &Option<Pattern>) -> io::Result<()>;
}

impl TreeView for Screen {
    fn write_tree(&mut self, tree: &Tree, pattern: &Option<Pattern>) -> io::Result<()> {
        for y in 1..self.h - 1 {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            let line_index = (y - 1) as usize;
            if line_index >= tree.lines.len() {
                continue;
            }
            let line = &tree.lines[line_index];
            let selected = line_index == tree.selection;
            for depth in 0..line.depth {
                write!(
                    self.stdout,
                    "{}{}{}",
                    color::Fg(color::AnsiValue::grayscale(5)),
                    match line.left_branchs[depth as usize] {
                        true => match tree.has_branch(line_index + 1, depth as usize) {
                            true => match depth == line.depth - 1 {
                                true => "├─",
                                false => "│ ",
                            },
                            false => "└─",
                        },
                        false => "  ",
                    },
                    color::Fg(color::Reset),
                )?;
            }
            self.write_line_key(line, selected)?;
            self.write_line_name(line, pattern)?;
            write!(
                self.stdout,
                "{}{}{}",
                style::Reset,
                color::Fg(color::Reset),
                color::Bg(color::Reset),
            )?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    fn write_line_key(&mut self, line: &TreeLine, selected: bool) -> io::Result<()> {
        match &line.content {
            LineType::Pruning { unlisted: _ } => {}
            _ => {
                if selected {
                    write!(
                        self.stdout,
                        "{} {} {}{}",
                        color::Bg(color::AnsiValue::grayscale(4)),
                        &line.key,
                        color::Bg(color::AnsiValue::grayscale(1)),
                        termion::clear::UntilNewline,
                    )?;
                } else {
                    write!(
                        self.stdout,
                        "{}{} {} {}{}",
                        color::Bg(color::AnsiValue::grayscale(2)),
                        color::Fg(color::AnsiValue::grayscale(14)),
                        &line.key,
                        color::Fg(color::Reset),
                        color::Bg(color::Reset),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn write_line_name(&mut self, line: &TreeLine, pattern: &Option<Pattern>) -> io::Result<()> {
        // FIXME find a way to use lazy_static here
        let fg_reset: String = format!("{}", color::Fg(color::Reset));
        let fg_dir: String = format!("{}", color::Fg(color::Rgb(84, 142, 188)));
        let fg_match: String = format!("{}", color::Fg(color::Green));
        let fg_reset_dir: String = format!("{}{}", fg_reset, fg_dir);
        match &line.content {
            LineType::Dir { name, unlisted } => {
                match line.key == "" {
                    true => {
                        // special display for the first line
                        write!(
                            self.stdout,
                            " {}{}{}",
                            style::Bold,
                            fg_dir,
                            &line.path.to_string_lossy(),
                        )?;
                    }
                    false => {
                        write!(
                            self.stdout,
                            " {}{}{}",
                            style::Bold,
                            fg_dir,
                            decorated_name(&name, pattern, &fg_match, &fg_reset_dir),
                        )?;
                        if *unlisted > 0 {
                            write!(self.stdout, " …",)?;
                        }
                    }
                };
            }
            LineType::File { name } => {
                write!(
                    self.stdout,
                    " {}",
                    decorated_name(&name, pattern, &fg_match, &fg_reset),
                )?;
            }
            LineType::Pruning { unlisted } => {
                write!(
                    self.stdout,
                    "{} ... {} other files…",
                    style::Italic,
                    unlisted,
                )?;
            }
        }
        Ok(())
    }
}

fn decorated_name<'a>(name: &'a str, pattern: &Option<Pattern>, prefix: &str, postfix: &str) -> Cow<'a, str> {
    if let Some(p) = pattern {
        if let Some(m) = p.test(name) {
            return Cow::Owned(m.wrap_matching_chars(
                name,
                prefix,
                postfix,
            ));
        }
    }
    Cow::Borrowed(name)
}
