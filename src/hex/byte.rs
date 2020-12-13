use {
    crate::{
        skin::StyleMap,
    },
    termimad::CompoundStyle,
};

pub enum ByteCategory {
    Null,
    AsciiGraphic,
    AsciiWhitespace,
    AsciiOther,
    NonAscii,
}

#[derive(Clone, Copy)]
pub struct Byte(u8);

impl From<u8> for Byte {
    fn from(u: u8) -> Self {
        Self(u)
    }
}

impl Byte {
    pub fn category(self) -> ByteCategory {
        if self.0 == 0x00 {
            ByteCategory::Null
        } else if self.0.is_ascii_graphic() {
            ByteCategory::AsciiGraphic
        } else if self.0.is_ascii_whitespace() {
            ByteCategory::AsciiWhitespace
        } else if self.0.is_ascii() {
            ByteCategory::AsciiOther
        } else {
            ByteCategory::NonAscii
        }
    }

    pub fn style(self, styles: &StyleMap) -> &CompoundStyle {
        match self.category() {
            ByteCategory::Null => &styles.hex_null,
            ByteCategory::AsciiGraphic => &styles.hex_ascii_graphic,
            ByteCategory::AsciiWhitespace => &styles.hex_ascii_whitespace,
            ByteCategory::AsciiOther => &styles.hex_ascii_other,
            ByteCategory::NonAscii => &styles.hex_non_ascii,
        }
    }

    pub fn as_char(self) -> char {
        match self.category() {
            ByteCategory::Null => '0',
            ByteCategory::AsciiGraphic => self.0 as char,
            ByteCategory::AsciiWhitespace if self.0 == 0x20 => ' ',
            ByteCategory::AsciiWhitespace => '_',
            ByteCategory::AsciiOther => '•',
            ByteCategory::NonAscii => '×',
        }
    }
}
