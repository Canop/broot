
mod double_line;
mod image_view;
mod svg;

pub use {
    image_view::ImageView,
};

use {
    crate::errors::ProgramError,
    image::{
        io::Reader,
        DynamicImage,
    },
    std::path::Path,
};

// Max dimensions of the SVG image to render. A bigger size just makes it need
// a little more memory and takes more time to render. There's no quality gain
// in having this bigger than your screen
pub const MAX_SVG_BITMAP_WIDTH: u32 = 1000;
pub const MAX_SVG_BITMAP_HEIGHT: u32 = 1000;

pub fn load(path: &Path) -> Result<DynamicImage, ProgramError> {
    let is_svg = matches!(path.extension(), Some(ext) if ext == "svg" || ext == "SVG");
    let img = if is_svg {
        svg::render(path, MAX_SVG_BITMAP_WIDTH, MAX_SVG_BITMAP_HEIGHT)?
    } else {
        Reader::open(path)?.decode()?
    };
    Ok(img)
}
