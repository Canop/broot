use {
    super::*,
    crate::display::W,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        fmt::Write as _,
        io::Write,
    },
    termimad::StrFit,
};

/// a simple representation of a colored and styled string.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TString {
    pub csi: String,
    pub raw: String,
}
impl TString {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        csi: S1,
        raw: S2,
    ) -> Self {
        Self {
            csi: csi.into(),
            raw: raw.into(),
        }
    }
    pub fn push_csi(
        &mut self,
        params: &vte::Params,
        action: char,
    ) {
        self.csi.push('\u{1b}');
        self.csi.push('[');
        for (idx, param) in params.iter().enumerate() {
            for p in param {
                let _ = write!(self.csi, "{}", p);
            }
            if idx < params.len() - 1 {
                self.csi.push(';');
            }
        }
        self.csi.push(action);
    }
    pub fn draw(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        draw(w, &self.csi, &self.raw)
    }
    /// draw the string but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(
        &self,
        w: &mut W,
        cols_max: usize,
    ) -> Result<usize, ProgramError> {
        let fit = StrFit::make_cow(&self.raw, cols_max);
        if self.csi.is_empty() {
            write!(w, "{}", &fit.0)?;
        } else {
            write!(w, "{}{}{}", &self.csi, &fit.0, CSI_RESET)?;
        }
        Ok(fit.1)
    }
    pub fn starts_with(
        &self,
        csi: &str,
        raw: &str,
    ) -> bool {
        self.csi == csi && self.raw.starts_with(raw)
    }
    pub fn split_off(
        &mut self,
        at: usize,
    ) -> Self {
        Self {
            csi: self.csi.clone(),
            raw: self.raw.split_off(at),
        }
    }
    pub fn is_blank(&self) -> bool {
        self.raw.chars().all(char::is_whitespace)
    }
    pub fn is_styled(&self) -> bool {
        !self.csi.is_empty()
    }
    pub fn is_unstyled(&self) -> bool {
        self.csi.is_empty()
    }
}
