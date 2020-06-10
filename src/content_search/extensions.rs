
/// a short list of extensions that shouldn't be searched
///  by content
///
/// If you feel this list should maybe be changed, contact
/// me on miaou or raise an issue.
pub fn is_known_binary(ext: &str) -> bool {
    ext == "doc"
        || ext == "iso"
        || ext == "jpg"
        || ext == "jpeg"
        || ext == "ods"
        || ext == "odt"
        || ext == "pdf"
        || ext == "png"
        || ext == "ppt"
        || ext == "rar"
        || ext == "xls"
}
