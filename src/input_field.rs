use crossterm::{
    KeyEvent,
    TerminalCursor,
};
use termimad::Area;
use crate::event::Event;
use crate::skin::Skin;

pub struct InputField {
    content: Vec<char>,
    area: Area,
    cursor_pos: usize, // position in chars
}

impl InputField {
    pub fn new(area: Area) -> Self {
        debug_assert!(area.height==1, "input area must be of height 1");
        Self {
            content: Vec::new(),
            area,
            cursor_pos: 0,
        }
    }
    pub fn change_area(&mut self, x: u16, y: u16, w: u16) {
        self.area.left = x;
        self.area.top = y;
        self.area.width = w;
    }
    pub fn get_content(&self) -> String {
        self.content.iter().collect()
    }
    pub fn is_content(&self, s: &str) -> bool {
        // TODO this comparison could be optimized
        let str_content = self.get_content();
        str_content == s
    }
    pub fn set_content(&mut self, s: &str) {
        if self.is_content(s) {
            return;
        }
        self.content = s.chars().collect();
        self.cursor_pos = self.content.len();
    }
    /// put a char at cursor position (and increments this
    /// position)
    pub fn put_char(&mut self, c: char) {
        self.content.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }
    /// remove the char left of the cursor, if any
    pub fn del_char_left(&mut self) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.content.remove(self.cursor_pos);
            true
        } else {
            false
        }
    }
    /// remove the char at cursor position, if any
    pub fn del_char_below(&mut self) -> bool {
        if self.cursor_pos < self.content.len() {
            self.content.remove(self.cursor_pos);
            true
        } else {
            false
        }
    }
    ///
    pub fn apply_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Click(x, y) if *y == self.area.top+1 && *x > self.area.left => {
                let p = (x - 1 - self.area.left) as usize;
                self.cursor_pos = p.min(self.content.len());
                true
            }
            Event::Key(KeyEvent::Home) => {
                self.cursor_pos = 0;
                true
            }
            Event::Key(KeyEvent::End) => {
                self.cursor_pos = self.content.len();
                true
            }
            Event::Key(KeyEvent::Char(c)) => {
                self.put_char(*c);
                true
            }
            Event::Key(KeyEvent::Left) if self.cursor_pos>0 => {
                self.cursor_pos -= 1;
                true
            }
            Event::Key(KeyEvent::Right) if self.cursor_pos<self.content.len() => {
                self.cursor_pos += 1;
                true
            }
            Event::Key(KeyEvent::Backspace) => {
                self.del_char_left()
            }
            Event::Key(KeyEvent::Delete) => {
                self.del_char_below()
            }
            _ => {
                debug!("InputField ignores event {:?}", event);
                false
            }
        }
    }
    pub fn display(&self, skin: &Skin) {
        let cursor = TerminalCursor::new();
        cursor.goto(self.area.left, self.area.top).unwrap();
        for (i, c) in self.content.iter().enumerate() {
            if self.cursor_pos == i {
                print!("{}", skin.cursor.apply_to(c));
            } else {
                print!("{}", c);
            }
        }
        let mut e = self.content.len();
        if e == self.cursor_pos {
            print!("{}", skin.cursor.apply_to(' '));
            e += 1;
        }
        for _ in e..self.area.width as usize {
            print!(" ");
        }
    }
}

