
use {
    crate::{
        tree::TreeOptions,
    },
    std::{
        path::PathBuf,
    },
};


/// the parsed program launch arguments which are kept for the
/// life of the program
pub struct AppLaunchArgs {
    pub root: PathBuf,                    // what should be the initial root
    pub file_export_path: Option<String>, // where to write the produced path (if required with --out)
    pub cmd_export_path: Option<String>,  // where to write the produced command (if required with --outcmd)
    pub tree_options: TreeOptions,        // initial tree options
    pub commands: Option<String>,         // commands passed as cli argument, still unparsed
    pub height: Option<u16>,              // an optional height to replace the screen's one
    pub no_style: bool,                   // whether to remove all styles (including colors)
    pub listen: Option<String>,
}
