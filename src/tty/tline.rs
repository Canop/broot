use {
    crate::display::TAB_WIDTH,
    super::*,
    crate::display::W,
    serde::{
        Deserialize,
        Serialize,
    },
    std::io::Write,
    unicode_width::UnicodeWidthChar,
};

/// a simple representation of a line made of homogeneous parts.
///
/// Note that this only manages CSI and SGR components
/// and isn't a suitable representation for an arbitrary
/// terminal input or output.
/// I recommend you to NOT try to reuse this hack in another
/// project unless you perfectly understand it.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TLine {
    pub strings: Vec<TString>,
}

impl TLine {
    pub fn change_range_style(
        &mut self,
        trange: TRange,
        new_csi: String,
    ) {
        let TRange {
            mut string_idx,
            start_byte_in_string,
            end_byte_in_string,
        } = trange;
        if string_idx >= self.strings.len() {
            return;
        }

        let has_before = start_byte_in_string > 0;
        let has_after = end_byte_in_string < self.strings[string_idx].raw.len();

        if has_after {
            self.strings.insert(
                string_idx + 1,
                TString {
                    csi: self.strings[string_idx].csi.clone(),
                    raw: self.strings[string_idx].raw[end_byte_in_string..].to_string(),
                },
            );
        }
        if has_before {
            self.strings.insert(
                string_idx,
                TString {
                    csi: self.strings[string_idx].csi.clone(),
                    raw: self.strings[string_idx].raw[..start_byte_in_string].to_string(),
                },
            );
            string_idx += 1;
        }
        self.strings[string_idx].csi = new_csi;
        if has_before {
            self.strings[string_idx].raw =
                self.strings[string_idx].raw[start_byte_in_string..end_byte_in_string].to_string();
        } else {
            // we can just truncate the string
            self.strings[string_idx].raw.truncate(end_byte_in_string);
        }
    }
    pub fn from_tty(tty: &str) -> Self {
        let tty_str: String;
        let tty = if tty.contains('\t') {
            tty_str = tty.replace('\t', &" ".repeat(TAB_WIDTH));
            &tty_str
        } else {
            tty
        };
        let mut builder = TLineBuilder::default();
        builder.read(tty);
        builder.build()
    }
    pub fn from_raw(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: " ".to_string(),
                raw,
            }],
        }
    }
    pub fn to_raw(&self) -> String {
        let mut s = String::new();
        for ts in &self.strings {
            s.push_str(&ts.raw);
        }
        s
    }
    pub fn bold(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: CSI_BOLD.to_string(),
                raw,
            }],
        }
    }
    pub fn italic(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: CSI_ITALIC.to_string(),
                raw,
            }],
        }
    }
    pub fn add_tstring<C: Into<String>, R: Into<String>>(
        &mut self,
        csi: C,
        raw: R,
    ) {
        self.strings.push(TString {
            csi: csi.into(),
            raw: raw.into(),
        });
    }
    pub fn draw(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        for ts in &self.strings {
            ts.draw(w)?;
        }
        Ok(())
    }
    /// draw the line but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(
        &self,
        w: &mut W,
        cols_max: usize,
    ) -> Result<usize, ProgramError> {
        let mut cols = 0;
        for ts in &self.strings {
            if cols >= cols_max {
                break;
            }
            cols += ts.draw_in(w, cols_max - cols)?;
        }
        Ok(cols)
    }
    /// Draw the chars of the line whose indexes are in `from..to`,
    /// without taking more than cols_max cols, and return the number
    /// of cols written.
    /// Control chars are skipped, as they would break the column
    /// accounting.
    pub fn draw_range_in(
        &self,
        w: &mut W,
        cols_max: usize,
        from: usize,
        to: usize,
    ) -> Result<usize, ProgramError> {
        let mut cols = 0;
        let mut ci = 0; // index of the char in the line
        for ts in &self.strings {
            let mut run = String::new();
            let mut full = false;
            for c in ts.raw.chars() {
                if ci >= to {
                    full = true;
                    break;
                }
                if ci >= from {
                    if let Some(width) = c.width() {
                        if cols + width > cols_max {
                            full = true;
                            break;
                        }
                        run.push(c);
                        cols += width;
                    }
                }
                ci += 1;
            }
            if !run.is_empty() {
                if ts.csi.is_empty() {
                    write!(w, "{run}")?;
                } else {
                    write!(w, "{}{}{}", ts.csi, run, CSI_RESET)?;
                }
            }
            if full {
                break;
            }
        }
        Ok(cols)
    }
    pub fn is_blank(&self) -> bool {
        self.strings.iter().all(|s| s.raw.trim().is_empty())
    }
    // if this line has no style, return its content
    pub fn if_unstyled(&self) -> Option<&str> {
        if self.strings.len() == 1 {
            self.strings
                .first()
                .filter(|s| s.csi.is_empty())
                .map(|s| s.raw.as_str())
        } else {
            None
        }
    }
    pub fn has(
        &self,
        part: &str,
    ) -> bool {
        self.strings.iter().any(|s| s.raw.contains(part))
    }
}
