mod font;
mod icon_plugin;

use font::FontPlugin;

pub use icon_plugin::IconPlugin;

pub fn icon_plugin(icon_set: &str) -> Option<Box<dyn IconPlugin + Send + Sync>> {
    match icon_set {
        "vscode" => Some(Box::new(FontPlugin::new(
            &include!("../../resources/icons/vscode/data/icon_name_to_icon_code_point_map.rs"),
            &include!("../../resources/icons/vscode/data/double_extension_to_icon_name_map.rs"),
            &include!("../../resources/icons/vscode/data/extension_to_icon_name_map.rs"),
            &include!("../../resources/icons/vscode/data/file_name_to_icon_name_map.rs"),
        ))),
        "nerdfont" => Some(Box::new(FontPlugin::new(
            &include!("../../resources/icons/nerdfont/data/icon_name_to_icon_code_point_map.rs"),
            &include!("../../resources/icons/nerdfont/data/double_extension_to_icon_name_map.rs"),
            &include!("../../resources/icons/nerdfont/data/extension_to_icon_name_map.rs"),
            &include!("../../resources/icons/nerdfont/data/file_name_to_icon_name_map.rs"),
        ))),
        _ => None,
    }
}
