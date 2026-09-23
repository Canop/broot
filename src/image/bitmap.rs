//! Loading and simple transformations of bitmap images

use {
    crate::errors::ProgramError,
    image::{
        DynamicImage,
        GenericImageView,
        Rgba,
        RgbaImage,
        imageops,
    },
    std::path::Path,
};

/// Load a bitmap image, determining the format from the file's content
/// when the path's extension doesn't give it.
pub fn load(path: &Path) -> Result<DynamicImage, ProgramError> {
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    Ok(img)
}

/// Build an image from raw RGBA bytes, 4 per pixel.
pub fn from_rgba8(
    width: u32,
    height: u32,
    data: Vec<u8>,
) -> Result<DynamicImage, ProgramError> {
    let expected_len = (width as usize) * (height as usize) * 4;
    if data.len() != expected_len {
        return Err(ProgramError::Internal {
            details: format!(
                "Invalid RGBA data length: expected {}, got {}",
                expected_len,
                data.len()
            ),
        });
    }
    RgbaImage::from_raw(width, height, data)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| ProgramError::Internal {
            details: "failed to build an image from RGBA data".to_string(),
        })
}

/// Build a new image `target_width x target_height` (each `>=` the current
/// size) holding `img`'s pixels at the top-left and the added right columns
/// and bottom rows filled with opaque `rgb`. Used to pad a Sixel image out
/// to whole terminal cells so no sub-cell remainder is left unset.
pub fn padded_to_size(
    img: &DynamicImage,
    target_width: u32,
    target_height: u32,
    rgb: (u8, u8, u8),
) -> DynamicImage {
    let (w, h) = img.dimensions();
    let tw = target_width.max(w);
    let th = target_height.max(h);
    if tw == w && th == h {
        return img.clone();
    }
    let fill = Rgba([rgb.0, rgb.1, rgb.2, 255]);
    let mut canvas = RgbaImage::from_pixel(tw, th, fill);
    imageops::replace(&mut canvas, &img.to_rgba8(), 0, 0);
    DynamicImage::ImageRgba8(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rgba8_passthrough() {
        let img = from_rgba8(1, 1, vec![10, 20, 30, 255]).unwrap();
        assert_eq!(img.to_rgba8().into_raw(), vec![10, 20, 30, 255]);
    }

    #[test]
    fn from_rgba8_rejects_bad_length() {
        assert!(from_rgba8(2, 2, vec![0; 12]).is_err());
    }

    #[test]
    fn padded_to_size_fills_bottom_with_bg() {
        // 1x2 image: row0 = (1,1,1), row1 = (2,2,2); pad to height 4 with (9,8,7)
        let img = from_rgba8(1, 2, vec![1, 1, 1, 255, 2, 2, 2, 255]).unwrap();
        let padded = padded_to_size(&img, 1, 4, (9, 8, 7));
        assert_eq!(padded.dimensions(), (1, 4));
        let b = padded.to_rgba8().into_raw();
        assert_eq!(&b[0..4], &[1, 1, 1, 255]); // row 0 preserved
        assert_eq!(&b[4..8], &[2, 2, 2, 255]); // row 1 preserved
        assert_eq!(&b[8..12], &[9, 8, 7, 255]); // row 2 = bg
        assert_eq!(&b[12..16], &[9, 8, 7, 255]); // row 3 = bg
    }

    #[test]
    fn padded_to_size_noop_when_not_taller() {
        let img = from_rgba8(1, 4, vec![0; 16]).unwrap();
        assert_eq!(padded_to_size(&img, 1, 4, (0, 0, 0)).dimensions(), (1, 4));
    }

    #[test]
    fn padded_to_size_fills_right_and_bottom_with_bg() {
        // 2x1 image padded to 4x2: right columns and bottom row get bg
        let img = from_rgba8(2, 1, vec![9, 9, 9, 255, 9, 9, 9, 255]).unwrap();
        let padded = padded_to_size(&img, 4, 2, (1, 2, 3));
        assert_eq!(padded.dimensions(), (4, 2));
        let b = padded.to_rgba8().into_raw();
        assert_eq!(&b[0..8], &[9, 9, 9, 255, 9, 9, 9, 255]); // row 0: image
        assert_eq!(&b[8..16], &[1, 2, 3, 255, 1, 2, 3, 255]); // row 0: right pad
        assert_eq!(&b[16..32], &[1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255]); // row 1: all pad
    }

    #[test]
    fn padded_to_size_noop_when_not_larger() {
        let img = from_rgba8(2, 2, vec![0; 16]).unwrap();
        assert_eq!(padded_to_size(&img, 2, 2, (1, 1, 1)).dimensions(), (2, 2));
        assert_eq!(padded_to_size(&img, 1, 1, (1, 1, 1)).dimensions(), (2, 2));
    }
}
