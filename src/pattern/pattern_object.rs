use {
    std::ops,
};

/// on what the search applies
/// (a composite pattern may apply to several topic
/// hence the bools)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PatternObject {
    pub name: bool,
    pub subpath: bool,
    pub content: bool,
}

impl ops::BitOr for PatternObject {
    type Output = Self;
    fn bitor(self, o: Self) -> Self::Output {
        Self {
            name: self.name | o.name,
            subpath: self.subpath | o.subpath,
            content: self.content | o.content,
        }
    }
}

impl ops::BitOrAssign for PatternObject {
    fn bitor_assign(&mut self, rhs: Self) {
        self.name |= rhs.name;
        self.subpath |= rhs.subpath;
        self.content |= rhs.content;
    }
}
