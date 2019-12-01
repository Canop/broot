

use termimad::CompoundStyle;

use crate::{
    patterns::Pattern,
};

pub struct MatchedString<'a> {
    pub pattern: &'a Pattern,
    pub string: &'a str,
    pub base_style: &'a CompoundStyle,
    pub match_style: &'a CompoundStyle,
}

impl Pattern {
    pub fn style<'a>(
        &'a self,
        string: &'a str,
        base_style: &'a CompoundStyle,
        match_style: &'a CompoundStyle,
    ) -> MatchedString<'a> {
        MatchedString {
            pattern: self,
            string,
            base_style,
            match_style,
        }
    }
}

impl<'a> MatchedString<'a> {
    pub fn write_on<W>(&self, w: &mut W) -> Result<(), termimad::Error>
        where W: std::io::Write
    {
        if self.pattern.is_some() {
            if let Some(m) = self.pattern.find(self.string) {
                let mut pos_idx: usize = 0;
                let mut combined_style = self.base_style.clone();
                combined_style.overwrite_with(self.match_style);
                for (cand_idx, cand_char) in self.string.chars().enumerate() {
                    if pos_idx < m.pos.len() && m.pos[pos_idx] == cand_idx {
                        combined_style.queue(w, cand_char)?;
                        pos_idx += 1;
                    } else {
                        self.base_style.queue(w, cand_char)?;
                    }
                }
                return Ok(());
            }
        }
        self.base_style.queue_str(w, self.string)
    }
}

