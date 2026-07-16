use {
    crate::image::zune_compat::{DynamicImage, RgbImage, RgbaImage},
    cli_log::*,
};

pub enum ImageData {
    Rgb(RgbImage),
    Rgba(RgbaImage),
}

impl From<&DynamicImage> for ImageData {
    fn from(img: &DynamicImage) -> Self {
        if let Some(rgba) = img.as_rgba8() {
            debug!("using rgba");
            Self::Rgba(rgba)
        } else if let Some(rgb) = img.as_rgb8() {
            debug!("using rgb");
            Self::Rgb(rgb)
        } else {
            debug!("converting to rgb8");
            Self::Rgb(img.to_rgb8())
        }
    }
}

impl ImageData {
    /// kitty's `f=` transmission format tag
    pub fn kitty_format(&self) -> &'static str {
        match self {
            Self::Rgba(_) => "32",
            Self::Rgb(_) => "24",
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Self::Rgb(img) => img.as_raw(),
            Self::Rgba(img) => img.as_raw(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_prefers_rgba_when_available() {
        let img = DynamicImage::from_rgba8(1, 1, vec![1, 2, 3, 4]).unwrap();
        assert!(matches!(ImageData::from(&img), ImageData::Rgba(_)));
    }

    #[test]
    fn from_falls_back_to_rgb() {
        let img = DynamicImage::Image(image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(2, 1, vec![1, 2, 3, 4, 5, 6]).unwrap(),
        ));
        assert!(matches!(ImageData::from(&img), ImageData::Rgb(_)));
    }

    #[test]
    fn kitty_format_matches_variant() {
        let rgba = DynamicImage::from_rgba8(1, 1, vec![1, 2, 3, 4]).unwrap();
        assert_eq!(ImageData::from(&rgba).kitty_format(), "32");

        let rgb = DynamicImage::Image(image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(2, 1, vec![1, 2, 3, 4, 5, 6]).unwrap(),
        ));
        assert_eq!(ImageData::from(&rgb).kitty_format(), "24");
    }
}
