use {
    crate::tree::TreeLineType,
};

pub trait IconPlugin {
    fn get_icon(
        &self,
        tree_line_type : &TreeLineType,

        // Use case:
        // For files- use libmagic to get file type
        // For directories: get list of files to get dir type
        // Recommended to avoid for performance reasons.
        full_path : &std::path::PathBuf,
        name : &str,
        double_ext : Option<&str>,
        ext : Option<&str>,
    ) -> char;
}


