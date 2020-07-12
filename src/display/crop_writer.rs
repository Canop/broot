use {
    termimad::{
        CompoundStyle,
        Result,
    },
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
    pub fn queue_str(&mut self, cs: &CompoundStyle, s: &str) -> Result<()> {
        if self.is_full() {
            return Ok(());
        }
        let mut len = s.chars().count();
        let string = if len > self.allowed {
            len = self.allowed;
            s.chars().take(self.allowed).collect()
        } else {
            s.to_string()
        };
        self.allowed -= len;
        cs.queue(self.w, string)
    }
    pub fn queue_char(&mut self, cs: &CompoundStyle, c: char) -> Result<()> {
        if !self.is_full() {
            self.allowed -= 1;
            cs.queue(self.w, c)?;
        }
        Ok(())
    }
    pub fn queue_string(&mut self, cs: &CompoundStyle, mut s: String) -> Result<()> {
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
