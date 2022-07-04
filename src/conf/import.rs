use {
    crate::{
        display::LumaCondition,
    },
    serde::Deserialize,
};


/// A file to import, with optionally a condition
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Import {
    Simple(String),
    Detailed(DetailedImport),
}


#[derive(Clone, Debug, Deserialize)]
pub struct DetailedImport {

    /// a condition on terminal light
    pub luma: Option<LumaCondition>,

    /// path, either absolute or relative to the current file
    /// or the conf directory
    pub file: String,
}

impl Import {
    pub fn applies(&self) -> bool {
        self.luma().map_or(true, |luma| luma.is_verified())
    }
    pub fn luma(&self) -> Option<&LumaCondition> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detailed) => detailed.luma.as_ref(),
        }
    }
    pub fn file(&self) -> &str {
        match self {
            Self::Simple(s) => s,
            Self::Detailed(detailed) => &detailed.file
        }
    }
}
