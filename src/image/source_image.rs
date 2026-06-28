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

/// Round `v` down to a multiple of `m` (`m >= 1`).
pub(crate) fn floor_to_multiple(v: u32, m: u32) -> u32 {
    v - v % m
}

/// Round `v` up to a multiple of `m` (`m >= 1`).
pub(crate) fn ceil_to_multiple(v: u32, m: u32) -> u32 {
    v + (m - v % m) % m
}

/// Constraints a renderer places on a fitted image. Default = unconstrained.
#[derive(Clone, Copy)]
pub struct FitConstraints {
    /// Fitted height must be a multiple of this (`>= 1`). `1` = no constraint.
    pub height_multiple: u32,
    /// Opaque fill for the padding rows added to satisfy `height_multiple`.
    /// `Some` whenever `height_multiple > 1`.
    pub pad: Option<coolor::Rgb>,
}

impl Default for FitConstraints {
    fn default() -> Self {
        Self { height_multiple: 1, pad: None }
    }
}

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
        constraints: FitConstraints,
    ) -> Result<DynamicImage, ProgramError> {
        let m = constraints.height_multiple.max(1);
        // Band-align only when the pane can hold at least one whole band.
        // Padding rounds the fitted height UP to a multiple of `m`; if the pane
        // is shorter than one band we would overshoot it, so leave the image
        // unconstrained instead. In practice max_height is at least one cell
        // (~12-20px), so this guard only trips for a degenerate sub-band pane.
        let band = if m > 1 && max_height >= m { m } else { 1 };
        if band > 1 {
            max_height = floor_to_multiple(max_height, band);
        }
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
                let bg_color: Option<coolor::Color> = bg_color.map(Into::into);
                svg::render_tree(tree, max_width, max_height, bg_color)?
            }
        };
        if band > 1 {
            let (_, h) = img.dimensions();
            let target = ceil_to_multiple(h, band);
            if target != h {
                let rgb = constraints.pad.unwrap_or_else(|| coolor::Rgb::new(0, 0, 0));
                return img.padded_to_height(target, (rgb.r, rgb.g, rgb.b));
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::zune_compat::DynamicImage;

    #[test]
    fn floor_and_ceil_to_multiple() {
        assert_eq!(floor_to_multiple(7, 6), 6);
        assert_eq!(floor_to_multiple(12, 6), 12);
        assert_eq!(ceil_to_multiple(7, 6), 12);
        assert_eq!(ceil_to_multiple(12, 6), 12);
        assert_eq!(ceil_to_multiple(5, 6), 6);
    }

    #[test]
    fn default_fit_constraints_unconstrained() {
        let c = FitConstraints::default();
        assert_eq!(c.height_multiple, 1);
        assert!(c.pad.is_none());
    }

    fn solid_bitmap(w: u32, h: u32, rgba: [u8; 4]) -> SourceImage {
        let data: Vec<u8> = std::iter::repeat_n(rgba, (w * h) as usize).flatten().collect();
        SourceImage::Bitmap(DynamicImage::from_rgba8(w, h, data).unwrap())
    }

    #[test]
    fn fitting_band_aligns_height() {
        let src = solid_bitmap(4, 7, [10, 20, 30, 255]);
        let c = FitConstraints { height_multiple: 6, pad: Some(coolor::Rgb::new(0, 0, 0)) };
        let img = src.fitting(100, 100, None, c).unwrap();
        let (w, h) = img.dimensions();
        assert_eq!(w, 4);
        assert_eq!(h, 12); // 7 fits natively, padded up to next band
        assert_eq!(h % 6, 0);
    }

    #[test]
    fn fitting_does_not_upscale_thin_image() {
        let src = solid_bitmap(10, 4, [1, 2, 3, 255]);
        let c = FitConstraints { height_multiple: 6, pad: Some(coolor::Rgb::new(9, 9, 9)) };
        let img = src.fitting(100, 100, None, c).unwrap();
        assert_eq!(img.dimensions(), (10, 6)); // 4 -> 6 via padding, not scaling
        let b = img.to_rgba_bytes();
        assert_eq!(&b[0..4], &[1, 2, 3, 255]); // content row preserved
        assert_eq!(&b[(10 * 4 * 4)..(10 * 4 * 4 + 4)], &[9, 9, 9, 255]); // first pad row = bg
    }

    #[test]
    fn fitting_default_constraints_unchanged() {
        let src = solid_bitmap(4, 7, [10, 20, 30, 255]);
        let img = src.fitting(100, 100, None, FitConstraints::default()).unwrap();
        assert_eq!(img.dimensions(), (4, 7));
    }

    #[test]
    fn fitting_noop_when_already_aligned() {
        let src = solid_bitmap(4, 12, [0, 0, 0, 255]);
        let c = FitConstraints { height_multiple: 6, pad: Some(coolor::Rgb::new(1, 1, 1)) };
        assert_eq!(src.fitting(100, 100, None, c).unwrap().dimensions(), (4, 12));
    }

    #[test]
    fn fitting_constrained_pane_pads_within_one_band() {
        // pane only 7px tall: floor to one band (6); the image fits/pads to a
        // whole band that never exceeds the pane.
        let src = solid_bitmap(4, 8, [5, 5, 5, 255]);
        let c = FitConstraints { height_multiple: 6, pad: Some(coolor::Rgb::new(0, 0, 0)) };
        let (_, h) = src.fitting(100, 7, None, c).unwrap().dimensions();
        assert_eq!(h, 6);
        assert!(h <= 7);
    }

    #[test]
    fn fitting_skips_band_when_pane_below_one_band() {
        // pane shorter than one band (5px < 6): band-align is skipped, so the
        // image is never padded up past the pane (no overshoot).
        let src = solid_bitmap(4, 8, [5, 5, 5, 255]);
        let c = FitConstraints { height_multiple: 6, pad: Some(coolor::Rgb::new(0, 0, 0)) };
        let (_, h) = src.fitting(100, 5, None, c).unwrap().dimensions();
        assert!(h <= 5, "must not overshoot a sub-band pane, got {h}");
    }
}
