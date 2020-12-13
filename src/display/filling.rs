
use {
    crossterm::{
        QueueableCommand,
        style::Print,
    },
    termimad::{
        CompoundStyle,
        Result,
    },
};

const FILLING_STRING_CHAR_LEN: usize = 1000;

pub struct Filling {
    filling_string: String,
    char_size: usize,
}

impl Filling {
    // TODO as soon as const fn are capable enough, this should be const
    // to allow normal static fillings
    pub fn from_char(filling_char: char) -> Self {
        let char_size = String::from(filling_char).len();
        let mut filling_string = String::with_capacity(char_size * FILLING_STRING_CHAR_LEN);
        for _ in 0..FILLING_STRING_CHAR_LEN {
            filling_string.push(filling_char);
        }
        Self {
            filling_string,
            char_size,
        }
    }
    pub fn queue_unstyled<W>(
        &self,
        w: &mut W,
        mut len: usize,
    ) -> Result<()>
    where W: std::io::Write
    {
        while len > 0 {
            let sl = len.min(FILLING_STRING_CHAR_LEN);
            w.queue(Print(&self.filling_string[0..sl * self.char_size]))?;
            len -= sl;
        }
        Ok(())
    }
    pub fn queue_styled<W>(
        &self,
        w: &mut W,
        cs: &CompoundStyle,
        mut len: usize,
    ) -> Result<()>
    where W: std::io::Write
    {
        while len > 0 {
            let sl = len.min(FILLING_STRING_CHAR_LEN);
            cs.queue_str(w, &self.filling_string[0..sl * self.char_size])?;
            len -= sl;
        }
        Ok(())
    }
}
