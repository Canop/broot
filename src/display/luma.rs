pub use {
    crokey::crossterm::tty::IsTty,
    once_cell::sync::Lazy,
    serde::Deserialize,
};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Luma {
    Light,
    Unknown,
    Dark,
}

/// Return the light of the terminal background, which is a value
/// between 0 (black) and 1 (white).
pub fn luma() -> &'static Result<f32, terminal_light::TlError> {
    static LUMA: Lazy<Result<f32, terminal_light::TlError>> = Lazy::new(|| {
        let luma = time!(Debug, terminal_light::luma());
        info!("terminal's luma: {:?}", &luma);
        luma
    });
    &LUMA
}

impl Luma {
    pub fn read() -> Self {
        match luma() {
            Ok(luma) if *luma > 0.6 => Self::Light,
            Ok(_) => Self::Dark,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum LumaCondition {
    Simple(Luma),
    Array(Vec<Luma>),
}

impl LumaCondition {
    pub fn is_verified(&self) -> bool {
        let luma = if std::io::stdout().is_tty() {
            Luma::read()
        } else {
            Luma::Unknown
        };
        self.includes(luma)
    }
    pub fn includes(&self, other: Luma) -> bool {
        match self {
            Self::Simple(luma) => other == *luma,
            Self::Array(arr) => arr.contains(&other),
        }
    }
}
