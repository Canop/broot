/// The unique identifiant of a panel
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelId(usize);

impl From<usize> for PanelId {
    fn from(u: usize) -> Self {
        Self(u)
    }
}

impl PanelId {
    /// get the inner usize
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.0
    }
}
