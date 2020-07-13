#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelId(usize);
impl From<usize> for PanelId {
    fn from(u: usize) -> Self {
        Self(u)
    }
}
