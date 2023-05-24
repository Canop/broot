use {
    super::svg,
    crate::{
        errors::ProgramError,
    },
    image::{
        DynamicImage,
        GenericImageView,
        io::Reader,
        imageops::FilterType,
    },
    std::{
        borrow::Cow,
        path::Path,
    },
    termimad::crossterm::style::Color as CrosstermColor,
};

// Max dimensions of the SVG image to render. A bigger size just makes it need
// a little more memory and takes more time to render. There's no quality gain
// in having this bigger than your screen
pub const MAX_SVG_BITMAP_WIDTH: u32 = 1000;
pub const MAX_SVG_BITMAP_HEIGHT: u32 = 1000;

pub enum SourceImage {
    Bitmap(DynamicImage),
    Svg(resvg::Tree),
}

impl SourceImage {
    pub fn new(path: &Path) -> Result<Self, ProgramError> {
        let is_svg = matches!(path.extension(), Some(ext) if ext == "svg" || ext == "SVG");
        let img = if is_svg {
            Self::Svg(svg::load(path)?)
        } else {
            Self::Bitmap(Reader::open(path)?.decode()?)
        };
        Ok(img)
    }
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Bitmap(img) => img.dimensions(),
            Self::Svg(tree) => (
                f64_to_u32(tree.size.width()),
                f64_to_u32(tree.size.height())
            )
        }
    }
    pub fn fitting(
        &self,
        mut max_width: u32,
        mut max_height: u32,
        bg_color: Option<CrosstermColor>,
    ) -> Result<DynamicImage, ProgramError> {
        let img = match self {
            Self::Bitmap(img) => {
                let dim = self.dimensions();
                max_width = max_width.min(dim.0);
                max_height = max_height.min(dim.1);
                img.resize(max_width, max_height, FilterType::Triangle)
            }
            Self::Svg(tree) => {
                let bg_color: Option<coolor::Color> = bg_color.map(|cc| cc.into());
                svg::render_tree(tree, max_width, max_height, bg_color)?
            }
        };
        Ok(img)
    }
    pub fn optimal(&self) -> Result<Cow<DynamicImage>, ProgramError> {
        let cow = match self {
            Self::Bitmap(img) => Cow::Borrowed(img),
            Self::Svg(tree) => Cow::Owned(
                svg::render_tree(tree, MAX_SVG_BITMAP_WIDTH, MAX_SVG_BITMAP_HEIGHT, None)?
            ),
        };
        Ok(cow)
    }
}

// a new trait is supposed to provide try_from::<f64> to u32 but it's
// not stable yet...
fn f64_to_u32(v: f64) -> u32 {
    if v <= 0.0 || v >= u32::MAX as f64 {
        0
    } else {
        v as u32
    }
}
