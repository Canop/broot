/// A position in a tline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TRange {
    pub string_idx: usize,
    pub start_byte_in_string: usize,
    pub end_byte_in_string: usize,
}
