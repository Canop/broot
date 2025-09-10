use {
    super::*,
    crate::tree::TreeLineType,
    rustc_hash::FxHashMap,
};

pub struct FontPlugin {
    icon_name_to_icon_codepoint_map: FxHashMap<&'static str, u32>,
    file_name_to_icon_name_map: FxHashMap<&'static str, &'static str>,
    double_extension_to_icon_name_map: FxHashMap<&'static str, &'static str>,
    extension_to_icon_name_map: FxHashMap<&'static str, &'static str>,
    default_icon_point: u32,
}

impl FontPlugin {
    #[cfg(debug_assertions)]
    fn sanity_check(
        part_to_icon_name_map: &FxHashMap<&str, &str>,
        icon_name_to_icon_codepoint_map: &FxHashMap<&str, u32>,
    ) {
        let offending_entries = part_to_icon_name_map
            .values()
            .map(|icon_name| {
                (icon_name, icon_name_to_icon_codepoint_map.contains_key(icon_name))
            })
            // Find if any entry is not present
            .filter(|(_entry, entry_present)| !entry_present)
            .collect::<Vec<_>>();
        for oe in &offending_entries {
            eprintln!("{} is not a valid icon name", oe.0);
        }
        if !offending_entries.is_empty() {
            eprintln!("Terminating execution");
            std::process::exit(53);
        }
    }

    pub fn new(
        icon_name_to_icon_codepoint_map: &'static [(&'static str, u32)],
        double_extension_to_icon_name_map: &'static [(&'static str, &'static str)],
        extension_to_icon_name_map: &'static [(&'static str, &'static str)],
        file_name_to_icon_name_map: &'static [(&'static str, &'static str)],
    ) -> Self {
        let icon_name_to_icon_codepoint_map: FxHashMap<_, _> =
            icon_name_to_icon_codepoint_map.iter().cloned().collect();
        let double_extension_to_icon_name_map: FxHashMap<_, _> =
            double_extension_to_icon_name_map.iter().cloned().collect();
        let extension_to_icon_name_map: FxHashMap<_, _> =
            extension_to_icon_name_map.iter().cloned().collect();
        let file_name_to_icon_name_map: FxHashMap<_, _> =
            file_name_to_icon_name_map.iter().cloned().collect();

        #[cfg(debug_assertions)]
        {
            Self::sanity_check(
                &file_name_to_icon_name_map,
                &icon_name_to_icon_codepoint_map,
            );
            Self::sanity_check(
                &double_extension_to_icon_name_map,
                &icon_name_to_icon_codepoint_map,
            );
            Self::sanity_check(
                &extension_to_icon_name_map,
                &icon_name_to_icon_codepoint_map,
            );
        }

        let default_icon_point =
            *icon_name_to_icon_codepoint_map.get("default_file").unwrap();
        Self {
            icon_name_to_icon_codepoint_map,
            file_name_to_icon_name_map,
            double_extension_to_icon_name_map,
            extension_to_icon_name_map,
            default_icon_point,
        }
    }

    fn handle_single_extension(
        &self,
        ext: Option<String>,
    ) -> &'static str {
        match ext {
            None => "default_file",
            Some(ref e) => match self.extension_to_icon_name_map.get(e as &str) {
                None => "default_file",
                Some(icon_name) => icon_name,
            },
        }
    }

    fn handle_file(
        &self,
        name: &str,
        double_ext: Option<String>,
        ext: Option<String>,
    ) -> &'static str {
        match self.file_name_to_icon_name_map.get(name) {
            Some(icon_name) => icon_name,
            _ => self.handle_double_extension(double_ext, ext),
        }
    }

    fn handle_double_extension(
        &self,
        double_ext: Option<String>,
        ext: Option<String>,
    ) -> &'static str {
        match double_ext {
            None => self.handle_single_extension(ext),
            Some(ref de) => {
                match self.double_extension_to_icon_name_map.get(de as &str) {
                    None => self.handle_single_extension(ext),
                    Some(icon_name) => icon_name,
                }
            }
        }
    }
}

impl IconPlugin for FontPlugin {
    fn get_icon(
        &self,
        tree_line_type: &TreeLineType,
        name: &str,
        double_ext: Option<&str>,
        ext: Option<&str>,
    ) -> char {
        let icon_name = match tree_line_type {
            TreeLineType::Dir => "default_folder",
            TreeLineType::SymLink {
                ..
            } => "emoji_type_link", //bad but nothing better
            TreeLineType::File => self.handle_file(
                &name.to_ascii_lowercase(),
                double_ext.map(|de| de.to_ascii_lowercase()),
                ext.map(|e| e.to_ascii_lowercase()),
            ),
            TreeLineType::Pruning => "file_type_kite", //irrelevant
            _ => "default_file",
        };

        let entry_icon = unsafe {
            std::char::from_u32_unchecked(
                *self
                    .icon_name_to_icon_codepoint_map
                    .get(icon_name)
                    .unwrap_or(&self.default_icon_point),
            )
        };

        entry_icon
    }
}
