use {
    crate::{
        errors::*,
        preview::PreviewMode,
    },
    serde::Deserialize,
    std::{
        hash::{
            DefaultHasher,
            Hash,
            Hasher,
        },
        path::{
            Path,
            PathBuf,
        },
        process::Command,
    },
    tempfile::TempDir,
};

#[derive(Debug, Clone, Copy)]
pub struct TransformerId {
    idx: usize,
}
pub struct PreviewTransformers {
    transformers: Vec<PreviewTransformer>,
    /// Where the output files are temporarily stored
    temp_dir: TempDir,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PreviewTransformer {
    pub input_extensions: Vec<String>,
    pub output_extension: String,
    /// The command generating an output file from an input file
    /// eg "mutool draw -o {output-path} {input-path}"
    pub command: Vec<String>,
    pub mode: PreviewMode,
}
pub struct PreviewTransform {
    pub transformer_id: TransformerId,
    pub output_path: PathBuf,
}

impl PreviewTransformers {
    pub fn new(transformers: &[PreviewTransformer]) -> Result<Self, ConfError> {
        let transformers = transformers.to_vec();
        for transformer in &transformers {
            if transformer.command.is_empty() {
                return Err(ConfError::MissingField {
                    txt: "empty command in preview transformer".to_string(),
                });
            }
        }
        let temp_dir = tempfile::Builder::new()
            .prefix("broot-conversions")
            .tempdir()?;
        Ok(Self {
            transformers,
            temp_dir,
        })
    }
    pub fn transformer(
        &self,
        id: TransformerId,
    ) -> &PreviewTransformer {
        &self.transformers[id.idx]
    }
    pub fn transform(
        &self,
        input_path: &Path,
        mode: Option<PreviewMode>,
    ) -> Option<PreviewTransform> {
        let transformer_id = self.find_transformer_for(input_path, mode)?;
        let temp_dir = self.temp_dir.path();
        match self.transformers[transformer_id.idx].transform(input_path, temp_dir) {
            Ok(output_path) => Some(PreviewTransform {
                transformer_id,
                output_path,
            }),
            Err(e) => {
                error!(
                    "conversion failed using {:?}",
                    self.transformers[transformer_id.idx].command
                );
                error!("conversion error: {:?}", e);
                None
            }
        }
    }
    pub fn find_transformer_for(
        &self,
        path: &Path,
        mode: Option<PreviewMode>,
    ) -> Option<TransformerId> {
        let extension = path.extension().and_then(|ext| ext.to_str())?;
        for (idx, transformer) in self.transformers.iter().enumerate() {
            if !transformer
                .input_extensions
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case(extension))
            {
                continue;
            }
            if let Some(mode) = mode {
                if transformer.mode != mode {
                    continue;
                }
            }
            return Some(TransformerId { idx });
        }
        None
    }
}

impl PreviewTransformer {
    pub fn transform(
        &self,
        input_path: &Path,
        temp_dir: &Path,
    ) -> Result<PathBuf, PreviewTransformerError> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            input_path.hash(&mut hasher);
            hasher.finish()
        };
        let output_path = temp_dir.join(format!("{:x}.{}", hash, self.output_extension,));
        if output_path.exists() {
            return Ok(output_path);
        }

        let explicit_input = self.command.iter().any(|c| c.contains("{input-path}"));
        let explicit_output = self.command.iter().any(|c| c.contains("{output-path}"));

        let mut command = self.command.iter().map(|part| {
            part.replace("{input-path}", &input_path.to_string_lossy())
                .replace("{output-path}", &output_path.to_string_lossy())
        });
        info!("transforming {:?} to {:?}", input_path, output_path);
        let executable = command.next().unwrap();
        let mut process = Command::new(executable);
        process.stderr(std::process::Stdio::null());
        process.args(command);
        if !explicit_input {
            process.stdin(std::fs::File::open(input_path)?);
        }
        if explicit_output {
            process.stdout(std::process::Stdio::null());
        } else {
            process.stdout(std::fs::File::create(&output_path)?);
        }
        let exit_status = process
            .spawn()
            .and_then(|mut p| p.wait())?;
        if exit_status.success() {
            Ok(output_path)
        } else {
            // we remove the output file if the process failed, so that
            // it's not returned on the next call
            let _ = std::fs::remove_file(&output_path);
            match exit_status.code() {
                Some(code) => Err(PreviewTransformerError::ProcessFailed {
                    code,
                }),
                None => Err(PreviewTransformerError::ProcessInterrupted),
            }
        }
    }
}
