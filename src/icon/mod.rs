
mod icon_plugin;
mod vscode;

pub use {
    icon_plugin::IconPlugin,
};

pub fn icon_plugin(icon_set: &str) -> Option<Box<dyn IconPlugin + Send + Sync>> {
    match icon_set {
        "vscode" => Some(Box::new(vscode::VsCodeIconPlugin::new())),
        _ => None,
    }
}
