//! small things to help show some text on the terminal
//!
//! This isn't a generic library. Only features used in
//! broot are implemented.

use regex::Regex;
use std::io;
use termion::{color, style};

use crate::screens::{Screen, ScreenArea};

// write a string, taking exactly w chars, either
// by cutting or by padding
fn append_pad(dest: &mut String, s: &str, w: usize) {
    let s: Vec<char> = s.chars().collect();
    for i in 0..w {
        dest.push(if i < s.len() { s[i] } else { ' ' });
    }
}

// do some simple markdown-like formatting to ease text generation:
// - bold: **bold text**
// - code: `some code`
fn md(s: &str) -> String {
    lazy_static! {
        static ref bold_regex: Regex = Regex::new(r"\*\*([^*]+)\*\*").unwrap();
        static ref bold_repl: String = format!("{}$1{}", style::Bold, style::Reset);
        static ref code_regex: Regex = Regex::new(r"`([^`]+)`").unwrap();
        static ref code_repl: String = format!(
            "{} $1 {}",
            color::Bg(color::AnsiValue::grayscale(2)),
            color::Bg(color::Reset)
        );
    }
    let s = bold_regex.replace_all(s, &*bold_repl as &str); // TODO how to avoid this complex casting ?
    let s = code_regex.replace_all(&s, &*code_repl as &str);
    s.to_string()
}

/// A Text is a vec of lines
pub struct Text {
    lines: Vec<String>,
}
impl Text {
    pub fn new() -> Text {
        Text { lines: Vec::new() }
    }
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    pub fn md(&mut self, line: &str) {
        self.lines.push(md(line));
    }
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }
    pub fn write(&self, screen: &mut Screen, area: &ScreenArea) -> io::Result<()> {
        screen.write_lines(area, &self.lines)
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
}

impl<'a, R> TextTable<'a, R> {
    pub fn new() -> TextTable<'a, R> {
        TextTable { cols: Vec::new() }
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
    fn compute_col_widths(&mut self, rows: &'a Vec<R>) {
        for row in rows {
            for c in &mut self.cols {
                c.width = c.width.max((c.extract)(row).len());
            }
        }
    }
    // write the table into the text.
    // Right now, to ease width computations, md transformation is done
    // only in the last column
    pub fn write(&mut self, rows: &'a Vec<R>, text: &mut Text) {
        lazy_static! {
            static ref bar: String = format!(
                " {}│{} ",
                color::Fg(color::AnsiValue::grayscale(8)),
                color::Fg(color::Reset),
            )
            .to_string();
        }
        self.compute_col_widths(&rows);
        let mut header = String::new();
        for col in &self.cols {
            header.push_str(&*bar);
            // we're lazy here:
            // we add some bold, and add 4 for the width because we know the * won't
            // show up on screen.
            append_pad(&mut header, &format!("**{}**", col.title), col.width + 4);
        }
        text.md(&header);
        for row in rows {
            let mut line = String::new();
            for (i, col) in self.cols.iter().enumerate() {
                line.push_str(&*bar);
                let s = (col.extract)(row);
                if i == self.cols.len() - 1 {
                    line.push_str(&md(s));
                } else {
                    append_pad(&mut line, s, col.width);
                }
            }
            text.push(line);
        }
    }
}
