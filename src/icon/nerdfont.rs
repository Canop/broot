use {super::*, crate::tree::TreeLineType, ahash::AHashMap};

pub struct NerdfontIconPlugin {
    icon_name_to_icon_codepoint_map: AHashMap<&'static str, u32>,
    file_name_to_icon_name_map: AHashMap<&'static str, &'static str>,
    double_extension_to_icon_name_map: AHashMap<&'static str, &'static str>,
    extension_to_icon_name_map: AHashMap<&'static str, &'static str>,
    default_icon_point: u32,
}

impl NerdfontIconPlugin {
    #[cfg(debug_assertions)]
    fn sanity_check(
        part_to_icon_name_map: &AHashMap<&str, &str>,
        icon_name_to_icon_codepoint_map: &AHashMap<&str, u32>,
    ) {
        let offending_entries = part_to_icon_name_map
            .iter()
            .map(|(_k, icon_name)| {
                (
                    icon_name,
                    icon_name_to_icon_codepoint_map.contains_key(icon_name),
                )
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

    pub fn new() -> Self {
        let icon_name_to_icon_codepoint_map: AHashMap<&'static str, u32> =
            (include!("../../resources/icons/nerdfont/data/icon_name_to_icon_code_point_map.rs"))
                .iter()
                .cloned()
                .collect();

        let double_extension_to_icon_name_map: AHashMap<&'static str, &'static str> =
            (include!("../../resources/icons/nerdfont/data/double_extension_to_icon_name_map.rs"))
                .iter()
                .cloned()
                .collect();

        let extension_to_icon_name_map: AHashMap<&'static str, &'static str> =
            (include!("../../resources/icons/nerdfont/data/extension_to_icon_name_map.rs"))
                .iter()
                .cloned()
                .collect();

        let file_name_to_icon_name_map: AHashMap<&'static str, &'static str> =
            (include!("../../resources/icons/nerdfont/data/file_name_to_icon_name_map.rs"))
                .iter()
                .cloned()
                .collect();

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

        let default_icon_point = *icon_name_to_icon_codepoint_map.get("default_file").unwrap();
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
            Some(ref de) => match self.double_extension_to_icon_name_map.get(de as &str) {
                None => self.handle_single_extension(ext),
                Some(icon_name) => icon_name,
            },
        }
    }
}

impl IconPlugin for NerdfontIconPlugin {
    fn get_icon(
        &self,
        tree_line_type: &TreeLineType,
        name: &str,
        double_ext: Option<&str>,
        ext: Option<&str>,
    ) -> char {
        let icon_name = match tree_line_type {
            TreeLineType::Dir => "default_folder",
            TreeLineType::SymLink { .. } => "emoji_type_link", //bad but nothing better
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
