use {
    super::CropWriter,
    crate::{
        pattern::NameMatch,
    },
    termimad::CompoundStyle,
};

pub struct MatchedString<'a> {
    pub name_match: Option<NameMatch>,
    pub string: &'a str,
    pub base_style: &'a CompoundStyle,
    pub match_style: &'a CompoundStyle,
}

impl<'a, 'w> MatchedString<'a> {
    pub fn queue_on<W>(&self, cw: &mut CropWriter<'w, W>) -> Result<(), termimad::Error>
    where
        W: std::io::Write,
    {
        if let Some(m) = &self.name_match {
            let mut pos_idx: usize = 0;
            let mut combined_style = self.base_style.clone();
            combined_style.overwrite_with(self.match_style);
            for (cand_idx, cand_char) in self.string.chars().enumerate() {
                if pos_idx < m.pos.len() && m.pos[pos_idx] == cand_idx {
                    cw.queue_char(&combined_style, cand_char)?;
                    pos_idx += 1;
                } else {
                    cw.queue_char(&self.base_style, cand_char)?;
                }
            }
        } else {
            cw.queue_str(&self.base_style, self.string)?;
        }
        Ok(())
    }
}
