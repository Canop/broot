use {
    crate::errors::ProgramError,
    lfs_core::{
        DeviceId,
        Mount,
        ReadOptions,
        read_mounts,
    },
};

#[derive(Default)]
pub struct MountList {
    mounts: Option<Vec<Mount>>,
}

impl MountList {
    pub fn clear_cache(&mut self) {
        self.mounts = None;
    }
    /// try to load the mounts if they aren't loaded.
    pub fn load(&mut self) -> Result<&Vec<Mount>, ProgramError> {
        if self.mounts.is_none() {
            let options = ReadOptions::default().remote_stats(false);
            match read_mounts(&options) {
                Ok(mut vec) => {
                    debug!("{} mounts loaded", vec.len());
                    vec.sort_by_key(|m| {
                        let size = m.stats().map_or(0, |s| s.size());
                        u64::MAX - size
                    });
                    self.mounts = Some(vec);
                }
                Err(e) => {
                    warn!(
                        "Failed to load mounts: {:?}",
                        e
                    );
                    return Err(ProgramError::Lfs {
                        details: e.to_string(),
                    });
                }
            }
        }
        Ok(
            // this unwrap will be fixed as soon as there's option.insert in stable
            self.mounts.as_ref().unwrap(),
        )
    }
    pub fn get_by_device_id(
        &self,
        dev: DeviceId,
    ) -> Option<&Mount> {
        self.mounts.as_ref().and_then(|mounts| mounts.iter().find(|m| m.info.dev == dev))
    }
}
