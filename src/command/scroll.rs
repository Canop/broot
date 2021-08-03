
#[derive(Debug, Clone, Copy)]
pub enum ScrollCommand {
    Lines(i32),
    Pages(i32),
}

impl ScrollCommand {
    pub fn to_lines(self, page_height: usize) -> i32 {
        match self {
            Self::Lines(n) => n,
            Self::Pages(n) => n * page_height as i32,
        }
    }
    pub fn is_up(self) -> bool {
        match self {
            Self::Lines(n) => n < 0,
            Self::Pages(n) => n < 0,
        }
    }
    /// compute the new scroll value
    pub fn apply(
        self,
        scroll: usize,
        content_height: usize,
        page_height: usize,
    ) -> usize {
        (scroll as i32 + self.to_lines(page_height))
            .min(content_height as i32 - page_height as i32 + 1)
            .max(0) as usize
    }
    pub fn is_thumb(y: u16, scrollbar: Option<(u16, u16)>) -> bool {
        if let Some((sctop, scbottom)) = scrollbar {
            if sctop <= y && y <= scbottom {
                return true;
            }
        }
        false
    }
}

