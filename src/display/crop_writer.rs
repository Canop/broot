use {
    crossterm::{
        terminal::{Clear, ClearType},
        QueueableCommand,
    },
    termimad::{
        CompoundStyle,
        Result,
    },
};


/// wrap a writer to ensure that at most `allowed` chars are
/// written.
pub struct CropWriter<'w, W>
    where W: std::io::Write,
{
    w: &'w mut W,
    allowed: usize,
}

impl<'w, W> CropWriter<'w, W>
    where W: std::io::Write
{
    pub fn new(w: &'w mut W, limit: usize) -> Self {
        Self {
            w,
            allowed: limit,
        }
    }
    pub fn is_full(&self) -> bool {
        self.allowed == 0
    }

    pub fn queue_str(&mut self, cs: &CompoundStyle, s: &str) -> Result<()> {
        if !self.is_full() {
            self.queue_string(cs, s.to_string())?;
        }
        Ok(())
    }
    pub fn queue_char(&mut self, cs: &CompoundStyle, c: char) -> Result<()> {
        if !self.is_full() {
            self.allowed -= 1;
            cs.queue(self.w, c)?;
        }
        Ok(())
    }
    pub fn queue_string(&mut self, cs: &CompoundStyle, s: String) -> Result<()> {
        if !self.is_full() {
            let len = s.chars().count();
            if len > self.allowed {
                debug!("CROP {:?}", &s);
                for c in s.chars().take(self.allowed) {
                    cs.queue(self.w, c)?;
                }
                self.allowed = 0;
            } else {
                cs.queue(self.w, s)?;
                self.allowed -= len;
            }
        }
        Ok(())
    }
    pub fn queue_bg(&mut self, cs: &CompoundStyle) -> Result<()> {
        cs.queue_bg(self.w)
    }
    pub fn clear(&mut self, clear_type: ClearType) -> Result<()> {
        self.w.queue(Clear(clear_type))?;
        Ok(())
    }

}

