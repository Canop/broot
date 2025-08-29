use {
    crate::{
        errors::*,
        preview::PreviewMode,
    },
    serde::Deserialize,
    std::{
        fs,
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
pub struct PreviewTransformerConf {
    pub input_extensions: Vec<String>,
    pub output_extension: String,
    /// The command generating an output file from an input file
    /// eg "mutool draw -o {output-path} {input-path}"
    pub command: Vec<String>,
    pub mode: PreviewMode,
}
#[derive(Debug, Clone)]
pub struct PreviewTransformer {
    pub input_extensions: Vec<String>,
    pub output_extension: String,
    /// The command generating an output file from an input file
    /// eg "mutool draw -o {output-path} {input-path}"
    pub command: Vec<String>,
    pub mode: PreviewMode,
    pub input_kind: ProcessInputKind,
    pub output_kind: ProcessOutputKind,
}
/// Specified how the input of the transformation is provided to the
/// external process.
#[derive(Debug, Clone, Copy)]
pub enum ProcessInputKind {
    File,
    Stdin,
}
/// Specifies how the output of the transformation is read:
/// - read from {output-path} if it's in the command, or
/// - read from the first file found in {output-dir} if it's in the command, or
/// - read from stdout if neither is in the command
#[derive(Debug, Clone, Copy)]
pub enum ProcessOutputKind {
    File,
    Dir,
    Stdout,
}
pub struct PreviewTransform {
    pub transformer_id: TransformerId,
    /// Path to the generated file
    pub output_path: PathBuf,
}

impl PreviewTransformers {
    pub fn new(transformer_confs: &[PreviewTransformerConf]) -> Result<Self, ConfError> {
        let mut transformers = Vec::with_capacity(transformer_confs.len());
        for transformer_conf in transformer_confs {
            transformers.push(PreviewTransformer::from_conf(transformer_conf)?);
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
    pub fn from_conf(conf: &PreviewTransformerConf) -> Result<Self, ConfError> {
        if conf.command.is_empty() {
            return Err(ConfError::MissingField {
                txt: "empty command in preview transformer".to_string(),
            });
        }
        let has_input_path = conf.command.iter().any(|c| c.contains("{input-path}"));
        let has_output_path = conf.command.iter().any(|c| c.contains("{output-path}"));
        let has_output_dir = conf.command.iter().any(|c| c.contains("{output-dir}"));
        let input_kind = if has_input_path {
            ProcessInputKind::File
        } else {
            ProcessInputKind::Stdin
        };
        let output_kind = if has_output_path {
            ProcessOutputKind::File
        } else if has_output_dir {
            ProcessOutputKind::Dir
        } else {
            ProcessOutputKind::Stdout
        };
        Ok(Self {
            input_extensions: conf.input_extensions.clone(),
            output_extension: conf.output_extension.clone(),
            command: conf.command.clone(),
            mode: conf.mode,
            input_kind,
            output_kind,
        })
    }
    /// Call the external process to transform the input file into an output file
    ///
    /// Input is given to the process either as a file or as stdin, depending on
    /// whether the command contains "{input-path}".
    ///
    /// Output is
    /// - read from {output-path} if it's in the command, or
    /// - read from the first file found in {output-dir} if it's in the command, or
    /// - read from stdout if neither is in the command
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
        let input_stem = input_path
            .file_stem()
            .ok_or(PreviewTransformerError::InvalidInput)?
            .to_string_lossy();
        let output_dir = temp_dir.join(format!("{:x}", hash));
        if output_dir.exists() {
            // if there's a file in the output directory, it's the result of a previous
            // transformation of the same input file
            if let Some(path) = first_file_in_dir(&output_dir)? {
                // we check that the transformed file isn't older than the file
                // to preview (or changes would be ignored)
                let input_modified = input_path.metadata().and_then(|m| m.modified());
                let transformed_modified = path.metadata().and_then(|m| m.modified());
                match (input_modified, transformed_modified) {
                    (Ok(input_date), Ok(transformed_date)) if input_date <= transformed_date => {
                        // the transformed file is up to date
                        debug!("preview transform {:?} up to date", path);
                        return Ok(path);
                    }
                    _ => {
                        // the transformed file is obsolete
                        debug!("preview transform {:?} obsolete", path);
                        fs::remove_file(&path)?;
                    }
                }
            }
        } else {
            fs::create_dir(&output_dir)?;
        }

        let mut output_path = output_dir.join(format!("{}.{}", input_stem, self.output_extension));

        let mut command = self.command.iter().map(|part| {
            part.replace("{input-path}", &input_path.to_string_lossy())
                .replace("{output-dir}", &output_dir.to_string_lossy())
                .replace("{output-path}", &output_path.to_string_lossy())
        });

        info!("transforming {:?} to {:?}", input_path, output_path);

        let executable = command.next().unwrap();
        let mut process = Command::new(executable);
        process.stderr(std::process::Stdio::null());
        process.args(command);

        match self.input_kind {
            ProcessInputKind::File => {
                process.stdin(std::process::Stdio::null());
            }
            ProcessInputKind::Stdin => {
                process.stdin(std::fs::File::open(input_path)?);
            }
        }

        match self.output_kind {
            ProcessOutputKind::File | ProcessOutputKind::Dir => {
                process.stdout(std::process::Stdio::null());
            }
            ProcessOutputKind::Stdout => {
                process.stdout(std::fs::File::create(&output_path)?);
            }
        }

        let exit_status = process.spawn().and_then(|mut p| p.wait())?;

        output_path = first_file_in_dir(&output_dir)?.ok_or(PreviewTransformerError::NoOutput)?;
        if exit_status.success() {
            Ok(output_path)
        } else {
            // we remove the output file if the process failed, so that
            // it's not returned on the next call
            let _ = std::fs::remove_file(&output_path);
            match exit_status.code() {
                Some(code) => Err(PreviewTransformerError::ProcessFailed { code }),
                None => Err(PreviewTransformerError::ProcessInterrupted),
            }
        }
    }
}

fn first_file_in_dir(dir: &Path) -> Result<Option<PathBuf>, PreviewTransformerError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
