
#[derive(Debug, Clone, Copy)]
pub enum ScrollCommand {
    Lines(i32),
    Pages(i32),
}

impl ScrollCommand {
    pub fn to_lines(self, page_height: i32) -> i32 {
        match self {
            Self::Lines(n) => n,
            Self::Pages(n) => n * page_height,
        }
    }
}

