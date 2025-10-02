use {
    super::{
        svg,
        zune_compat::DynamicImage,
    },
    crate::errors::ProgramError,
    std::path::Path,
    termimad::{
        coolor,
        crossterm::style::Color as CrosstermColor,
    },
};

#[allow(clippy::large_enum_variant)]
pub enum SourceImage {
    Bitmap(DynamicImage),
    Svg(resvg::usvg::Tree),
}

impl SourceImage {
    pub fn new(path: &Path) -> Result<Self, ProgramError> {
        let is_svg = matches!(path.extension(), Some(ext) if ext == "svg" || ext == "SVG");
        let img = if is_svg {
            Self::Svg(svg::load(path)?)
        } else {
            Self::Bitmap(DynamicImage::from_path(path)?)
        };
        Ok(img)
    }
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Bitmap(img) => img.dimensions(),
            Self::Svg(tree) => (
                f32_to_u32(tree.size().width()),
                f32_to_u32(tree.size().height()),
            ),
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
                if dim.0 <= max_width && dim.1 <= max_height {
                    img.clone()
                } else {
                    max_width = max_width.min(dim.0);
                    max_height = max_height.min(dim.1);
                    img.resize(max_width, max_height)?
                }
            }
            Self::Svg(tree) => {
                let bg_color: Option<coolor::Color> = bg_color.map(|cc| cc.into());
                svg::render_tree(tree, max_width, max_height, bg_color)?
            }
        };
        Ok(img)
    }
}

fn f32_to_u32(v: f32) -> u32 {
    if v <= 0.0 || v >= u32::MAX as f32 {
        0
    } else {
        v as u32
    }
}
