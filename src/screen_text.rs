//! small things to help show some text on the terminal
//!
//! This isn't a generic library. Only features used in
//! broot are implemented.

use regex::Regex;
use std::io::{self, Write};
use termion::style;

use crate::screens::{Screen, ScreenArea};
use crate::skin::Skin;

// write a string, taking exactly w chars, either
// by cutting or by padding
fn append_pad(dest: &mut String, s: &str, w: usize) {
    let s: Vec<char> = s.chars().collect();
    for i in 0..w {
        dest.push(if i < s.len() { s[i] } else { ' ' });
    }
}

/// A Text is a vec of lines
pub struct Text {
    lines: Vec<String>,
    md_bold_repl: String,
    md_code_repl: String,
}
impl Text {
    pub fn new(skin: &Skin) -> Text {
        let md_code_repl = format!(
            "{}{} $1 {}{}",
            skin.code.fg,
            skin.code.bg,
            skin.reset.fg,
            skin.reset.bg,
        );
        let md_bold_repl = format!("{}$1{}", style::Bold, style::Reset);
        Text {
            lines: Vec::new(),
            md_bold_repl,
            md_code_repl,
        }
    }
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    // do some simple markdown-like formatting to ease text generation:
    // - bold: **bold text**
    // - code: `some code`
    pub fn md_to_tty(&self, raw: &str) -> String {
        lazy_static! {
            static ref bold_regex: Regex = Regex::new(r"\*\*([^*]+)\*\*").unwrap();
            static ref code_regex: Regex = Regex::new(r"`([^`]+)`").unwrap();
        }
        let s = bold_regex.replace_all(raw, &*self.md_bold_repl);
        let s = code_regex.replace_all(&s, &*self.md_code_repl);
        s.to_string()
    }
    // add a line from the line interpreted as "markdown"
    pub fn md(&mut self, line: &str) {
        self.lines.push(self.md_to_tty(line));
    }
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }
    // write the text in the area, taking into account the scrolled amount
    // and drawing a vertical scrollbar at the right if needed
    pub fn write(&self, screen: &mut Screen, area: &ScreenArea) -> io::Result<()> {
        let scrollbar = area.scrollbar();
        let mut i = area.scroll as usize;
        for y in area.top..=area.bottom {
            write!(
                screen.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            if i < self.lines.len() {
                write!(screen.stdout, "{}", &self.lines[i],)?;
                i += 1;
            }
            if let Some((sctop, scbottom)) = scrollbar {
                if sctop <= y && y <= scbottom {
                    write!(screen.stdout, "{}▐", termion::cursor::Goto(screen.w, y),)?;
                }
            }
        }
        screen.stdout.flush()?;
        Ok(())
    }
}

/// A column in a TextTable
struct TextCol<'a, R> {
    title: String,
    extract: &'a Fn(&'a R) -> &str,
    width: usize, // padding and border not included
}

impl<'a, R> TextCol<'a, R> {}

/// A small utility to format some data in a tabular way on screen
pub struct TextTable<'a, R> {
    cols: Vec<TextCol<'a, R>>,
    md_bar: String, // according to skin
}

impl<'a, R> TextTable<'a, R> {
    pub fn new(skin: &Skin) -> TextTable<'a, R> {
        let md_bar = format!(
            " {}│{} ",
            skin.table_border.fg,
            skin.reset.fg,
        );
        TextTable {
            cols: Vec::new(),
            md_bar,
        }
    }
    pub fn add_col(&mut self, title: &str, extract: &'a Fn(&'a R) -> &str) {
        let width = title.len(); // initial value, will change
        let title = title.to_string();
        self.cols.push(TextCol {
            title,
            extract,
            width,
        });
    }
    fn compute_col_widths(&mut self, rows: &'a [R]) {
        for row in rows {
            for c in &mut self.cols {
                c.width = c.width.max((c.extract)(row).len());
            }
        }
    }
    // write the table into the text.
    // Right now, to ease width computations, md transformation is done
    // only in the last column
    pub fn write(&mut self, rows: &'a [R], text: &mut Text) {
        self.compute_col_widths(&rows);
        let mut header = String::new();
        for col in &self.cols {
            header.push_str(&self.md_bar);
            // we're lazy here:
            // we add some bold, and add 4 for the width because we know the * won't
            // show up on screen.
            append_pad(&mut header, &format!("**{}**", col.title), col.width + 4);
        }
        text.md(&header);
        for row in rows {
            let mut line = String::new();
            for (i, col) in self.cols.iter().enumerate() {
                line.push_str(&self.md_bar);
                let s = (col.extract)(row);
                if i == self.cols.len() - 1 {
                    line.push_str(&text.md_to_tty(s));
                } else {
                    append_pad(&mut line, s, col.width);
                }
            }
            text.push(line);
        }
    }
}
