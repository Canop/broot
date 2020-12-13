use {
    super::{Filling, TAB_REPLACEMENT},
    crossterm::{style::Print, QueueableCommand},
    std::borrow::Cow,
    termimad::{
        CompoundStyle,
        Result,
        StrFit,
    },
    unicode_width::UnicodeWidthChar,
};

/// wrap a writer to ensure that at most `allowed` columns are
/// written.
pub struct CropWriter<'w, W>
where
    W: std::io::Write,
{
    pub w: &'w mut W,
    /// number of screen columns which may be covered
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
    /// return a tuple containing a string containing either the given &str
    /// or the part fitting the remaining width, and the width of this string)
    pub fn cropped_str<'a>(&self, s: &'a str) -> (Cow<'a, str>, usize) {
        StrFit::make_cow(s, self.allowed)
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
        if c == '\t' {
            return self.queue_unstyled_str(TAB_REPLACEMENT);
        }
        let width = UnicodeWidthChar::width(c).unwrap_or(0);
        if width < self.allowed {
            self.allowed -= width;
            self.w.queue(Print(c))?;
        }
        Ok(())
    }
    /// a "g_string" is a "gentle" one: each char takes one column on screen.
    /// This function must thus not be used for unknown strings.
    pub fn queue_unstyled_g_string(&mut self, mut s: String) -> Result<()> {
        if self.is_full() {
            return Ok(());
        }
        let mut len = 0;
        for (idx, _) in s.char_indices() {
            len += 1;
            if len > self.allowed {
                s.truncate(idx);
                self.allowed = 0;
                self.w.queue(Print(s))?;
                return Ok(());
            }
        }
        self.allowed -= len;
        self.w.queue(Print(s))?;
        Ok(())
    }
    /// a "g_string" is a "gentle" one: each char takes one column on screen.
    /// This function must thus not be used for unknown strings.
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
                return cs.queue(self.w, s);
            }
        }
        self.allowed -= len;
        cs.queue(self.w, s)
    }
    pub fn queue_fg(&mut self, cs: &CompoundStyle) -> Result<()> {
        cs.queue_fg(self.w)
    }
    pub fn queue_bg(&mut self, cs: &CompoundStyle) -> Result<()> {
        cs.queue_bg(self.w)
    }
    pub fn fill(&mut self, cs: &CompoundStyle, filling: &'static Filling) -> Result<()> {
        self.repeat(cs, filling, self.allowed)
    }
    pub fn fill_unstyled(&mut self, filling: &'static Filling) -> Result<()> {
        self.repeat_unstyled(filling, self.allowed)
    }
    pub fn repeat(
        &mut self,
        cs: &CompoundStyle,
        filling: &'static Filling,
        mut len: usize,
    ) -> Result<()> {
        len = len.min(self.allowed);
        self.allowed -= len;
        filling.queue_styled(self.w, cs, len)
    }
    pub fn repeat_unstyled(&mut self, filling: &'static Filling, mut len: usize) -> Result<()> {
        len = len.min(self.allowed);
        self.allowed -= len;
        filling.queue_unstyled(self.w, len)
    }
}
