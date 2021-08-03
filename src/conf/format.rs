use {
    crate::{
        errors::{ConfError, ProgramError},
    },
    serde::de::DeserializeOwned,
    std::{
        fs,
        path::Path,
    },
    deser_hjson,
    toml,
};


/// Formats usable for reading configuration files
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SerdeFormat {
    Hjson,
    Toml,
}

pub static FORMATS: &[SerdeFormat] = &[
    SerdeFormat::Hjson,
    SerdeFormat::Toml,
];

impl SerdeFormat {
    pub fn key(self) -> &'static str {
        match self {
            Self::Hjson => "hjson",
            Self::Toml => "toml",
        }
    }
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "hjson" => Some(SerdeFormat::Hjson),
            "toml" => Some(SerdeFormat::Toml),
            _ => None,
        }
    }
    pub fn from_path(path: &Path) -> Result<Self, ConfError> {
        path.extension()
            .and_then(|os| os.to_str())
            .map(|ext| ext.to_lowercase())
            .and_then(|key| Self::from_key(&key))
            .ok_or_else(|| ConfError::UnknownFileExtension { path: path.to_string_lossy().to_string() })
    }
    pub fn read_file<T>(path: &Path) -> Result<T, ProgramError>
        where T: DeserializeOwned
    {
        let format = Self::from_path(path)?;
        let file_content = fs::read_to_string(path)?;
        match format {
            Self::Hjson => {
                deser_hjson::from_str::<T>(&file_content)
                    .map_err(|e| ProgramError::ConfFile {
                        path: path.to_string_lossy().to_string(),
                        details: e.into(),
                    })
            }
            Self::Toml => {
                toml::from_str::<T>(&file_content)
                    .map_err(|e| ProgramError::ConfFile {
                        path: path.to_string_lossy().to_string(),
                        details: e.into(),
                    })
            }
        }
    }
}

impl Default for SerdeFormat {
    fn default() -> Self {
        SerdeFormat::Hjson
    }
}
