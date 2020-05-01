#[derive(Debug, Clone, Copy)]
pub struct PanelId(usize);
impl From<usize> for PanelId {
    fn from(u: usize) -> Self {
        Self(u)
    }
}
