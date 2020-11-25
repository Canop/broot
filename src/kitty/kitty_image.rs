use {
    crate::{
        display::{
            cell_size_in_pixels,
            W,
        },
        errors::ProgramError,
    },
    base64,
    image::{
        DynamicImage,
        GenericImageView,
    },
    std::{
        env,
        io::{self, Write},
        path::Path,
    },
    tempfile,
};

/// until I'm told there's another terminal supporting the kitty
/// terminal, I think I can just check the name
pub fn is_term_kitty() -> bool {
    if let Ok(term_name) = env::var("TERM") {
        if term_name.contains("kitty") {
            return true;
        }
    }
    false
}

fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + (0 != a % b) as u32
}

pub struct KittyImageRenderer {
    cell_width: u32,
    cell_height: u32,
    has_image_on_screen: bool,
}

impl KittyImageRenderer {
    pub fn new() -> Option<Self> {
        if !is_term_kitty() {
            return None;
        }
        cell_size_in_pixels()
            .ok()
            .map(|(cell_width, cell_height)| Self {
                cell_width,
                cell_height,
                has_image_on_screen: false,
            })
    }
    fn rendering_dim(
        &self,
        img_width: u32,
        img_height: u32,
        area_cols: u32,
        area_rows: u32,
    ) -> (u32, u32) {
        let optimal_cols = div_ceil(img_width, self.cell_width);
        let optimal_rows = div_ceil(img_height, self.cell_height);
        debug!("area: {:?}", (area_cols, area_rows));
        debug!("optimal: {:?}", (optimal_cols, optimal_rows));
        if optimal_cols <= area_cols && optimal_rows <= area_rows {
            // no constraint (TODO center?)
            (optimal_cols, optimal_rows)
        } else if optimal_cols * area_rows > optimal_rows * area_cols {
            // we're constrained in width
            debug!("constrained in width");
            (area_cols, optimal_rows * area_cols / optimal_cols)
        } else {
            // we're constrained in height
            debug!("constrained in height");
            (optimal_cols * area_rows / optimal_rows, area_rows)
        }
    }
    pub fn print_with_chunks(
        &mut self,
        w: &mut W,
        img: &DynamicImage,
        cols: u16,
        rows: u16,
    ) -> Result<(), ProgramError> {
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let bytes = rgba.as_raw();
        let encoded = base64::encode(bytes);
        let (c, r) = self.rendering_dim(width, height, cols.into(), rows.into());
        debug!("rendering_dim: {:?}", (c, r));
        let mut pos = 0;
        loop {
            if pos + 4096 < encoded.len() {
                write!(w,
                    "\u{1b}_Ga=T,f=32,t=d,s={},v={},c={},r={},m=1;{}\u{1b}\\",
                    width,
                    height,
                    c,
                    r,
                    &encoded[pos..pos+4096],
                )?;
                pos += 4096;
            } else {
                // last chunk
                write!(w,
                    "\u{1b}_Gm=0;{}\u{1b}\\",
                    &encoded[pos..encoded.len()],
                )?;
                break;
            }
        }
        self.has_image_on_screen = true;
        Ok(())
    }
    // deprecated
    pub fn print_png(
        &mut self,
        w: &mut W,
        path: &Path,
        cols: u16,
        rows: u16,
    ) -> Result<(), ProgramError> {
        let path = path.to_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::Other,
                "Path can't be converted to UTF8",
            ))?;
        let encoded_path = base64::encode(path);
        debug!("printing png with kitty c={}, r={}, path={}", cols, rows, &path);
        write!(w,
            "\u{1b}_Ga=T,f=100,t=f,r={},c={};{}\u{1b}\\",
            rows,
            cols,
            encoded_path,
        )?;
        self.has_image_on_screen = true;
        Ok(())
    }
    pub fn print_with_temp_file(
        &mut self,
        w: &mut W,
        img: &DynamicImage,
        cols: u16,
        rows: u16,
    ) -> Result<(), ProgramError> {
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let bytes: &[u8] = rgba.as_raw();
        let (mut temp_file, path) = tempfile::Builder::new()
            .prefix("broot-img-preview")
            .tempfile()?
            .keep()
            .map_err(|_| io::Error::new(
                io::ErrorKind::Other,
                "temp file can't be kept",
            ))?;
        temp_file.write_all(bytes)?;
        temp_file.flush()?;
        let path = path.to_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::Other,
                "Path can't be converted to UTF8",
            ))?;
        let encoded_path = base64::encode(path);
        debug!("temp file written: {:?}", path);
        let (c, r) = self.rendering_dim(width, height, cols.into(), rows.into());
        write!(w,
            "\u{1b}_Ga=T,f=32,t=t,s={},v={},c={},r={};{}\u{1b}\\",
            width,
            height,
            c,
            r,
            encoded_path,
        )?;
        debug!("file len: {}", temp_file.metadata().unwrap().len());
        self.has_image_on_screen = true;
        Ok(())
    }
    pub fn erase_images(
        &mut self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        if self.has_image_on_screen {
            write!(w, "\u{1b}_Ga=d\u{1b}\\")?;
            self.has_image_on_screen = false;
        }
        Ok(())
    }
}




