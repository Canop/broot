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
    /// Raw RGBA bytes (4 per pixel) regardless of source colorspace,
    /// for encoders that require RGBA input (e.g. icy_sixel).
    pub fn rgba_bytes(&self) -> Vec<u8> {
        match self {
            Self::Rgba(img) => img.as_raw(),
            Self::Rgb(img) => {
                let rgb = img.as_raw();
                let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
                for px in rgb.chunks_exact(3) {
                    out.extend_from_slice(px);
                    out.push(255);
                }
                out
            }
        }
    }
}
