use {
    crate::{
        app::PanelId,
        errors::ConfError,
    },
    lazy_regex::regex_switch,
    std::{
        fmt,
        str::FromStr,
    },
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de,
    },
};

/// the symbolic reference to the panel to close
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PanelReference {
    #[default]
    Active,
    Leftest,
    Rightest,
    Id(PanelId),
    Idx(usize),
    Preview,
}

impl PanelReference {
    /// whether this reference is the default one
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, PanelReference::Active)
    }
}

impl fmt::Display for PanelReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelReference::Active => write!(f, "active"),
            PanelReference::Leftest => write!(f, "leftest"),
            PanelReference::Rightest => write!(f, "rightest"),
            PanelReference::Id(id) => write!(f, "id:{}", id.as_usize()),
            PanelReference::Idx(idx) => write!(f, "idx:{}", idx),
            PanelReference::Preview => write!(f, "preview"),
        }
    }
}
impl FromStr for PanelReference {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        regex_switch!(s,
            "^active$"i => Self::Active,
            "^left(est)?$"i => Self::Leftest,
            "^right(est)?$"i => Self::Rightest,
            "^preview$"i => Self::Preview,
            r"^id:(?P<id>\d{1,2})$"i => Self::Id(id.parse::<usize>().unwrap().into()),
            r"^idx:(?P<idx>\d{1,2})$"i => Self::Idx(idx.parse().unwrap()),
        ).ok_or_else(|| ConfError::InvalidPanelReference {
            raw: s.to_string(),
        })
    }
}

impl Serialize for PanelReference {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for PanelReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
