use {
    crossterm::{
        QueueableCommand,
        style::Print,
    },
    termimad::{
        CompoundStyle,
        Result,
    },
    unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

/// wrap a writer to ensure that at most `allowed` chars are
/// written.
pub struct CropWriter<'w, W>
where
    W: std::io::Write,
{
    pub w: &'w mut W,
    pub allowed: usize,
}

impl<'w, W> CropWriter<'w, W>
where
    W: std::io::Write,
{
    pub fn new(w: &'w mut W, limit: usize) -> Self {
        Self { w, allowed: limit }
    }
    pub fn is_full(&self) -> bool {
        self.allowed == 0
    }
    pub fn cropped_str(&self, s: &str) -> (String, usize) {
        let mut len = UnicodeWidthStr::width(s);
        let string = if len > self.allowed {
            len = 0;
            let mut ns = String::new();
            for c in s.chars() {
                let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
                if char_width + len > self.allowed {
                    break;
                }
                ns.push(c);
                len += char_width;
            }
            ns
        } else {
            s.to_string()
        };
        (string, len)
    }
    pub fn queue_unstyled_str(&mut self, s: &str) -> Result<()> {
        if self.is_full() {
            return Ok(());
        }
        let (string, len) = self.cropped_str(s);
        self.allowed -= len;
        self.w.queue(Print(string))?;
        Ok(())
    }
    pub fn queue_str(&mut self, cs: &CompoundStyle, s: &str) -> Result<()> {
        if self.is_full() {
            return Ok(());
        }
        let (string, len) = self.cropped_str(s);
        self.allowed -= len;
        cs.queue(self.w, string)
    }
    pub fn queue_char(&mut self, cs: &CompoundStyle, c: char) -> Result<()> {
        let width = UnicodeWidthChar::width(c).unwrap_or(0);
        if width < self.allowed {
            self.allowed -= width;
            cs.queue(self.w, c)?;
        }
        Ok(())
    }
    pub fn queue_unstyled_char(&mut self, c: char) -> Result<()> {
        let width = UnicodeWidthChar::width(c).unwrap_or(0);
        if width < self.allowed {
            self.allowed -= width;
            self.w.queue(Print(c))?;
        }
        Ok(())
    }
    /// a "g_string" is a "gentle" one: each char takes one column on screen.
    /// This function must thus not used for unknown strings.
    pub fn queue_g_string(&mut self, cs: &CompoundStyle, mut s: String) -> Result<()> {
        if self.is_full() {
            return Ok(());
        }
        let mut len = 0;
        for (idx, _) in s.char_indices() {
            len += 1;
            if len > self.allowed {
                s.truncate(idx);
                self.allowed = 0;
                return cs.queue(self.w, s)
            }
        }
        self.allowed -= len;
        cs.queue(self.w, s)
    }
    pub fn queue_bg(&mut self, cs: &CompoundStyle) -> Result<()> {
        cs.queue_bg(self.w)
    }
    pub fn fill(&mut self, cs: &CompoundStyle, filling: &'static str) -> Result<()> {
        self.repeat(cs, filling, self.allowed)
    }
    pub fn repeat(&mut self, cs: &CompoundStyle, filling: &'static str, mut len: usize) -> Result<()> {
        loop {
            let slice_len = len.min(self.allowed).min(filling.len());
            if slice_len == 0 {
                break;
            }
            cs.queue_str(self.w, &filling[0..slice_len])?;
            self.allowed -= slice_len;
            len -= slice_len;
        }
        Ok(())
    }
}
