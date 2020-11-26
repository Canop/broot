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
    },
    tempfile,
};

pub type KittyImageSet = Vec<usize>;

/// The max size of a data payload in a kitty escape sequence
/// according to kitty's documentation
const CHUNK_SIZE: usize = 4096;

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

/// the image renderer, with knowledge of the
/// console cells dimensions, and built only on Kitty.
///
/// There are two ways to display the image:
/// - print_with_temp_file, which is faster on my
///    computers (with SSD)
/// - print_with_chunks which looks slightly less
///    racy and might be faster on computers with
///    very slow disks
/// Note that I didn't test yet the named shared memory
/// solution offered by kitty.
pub struct KittyImageRenderer {
    cell_width: u32,
    cell_height: u32,
    next_id: usize,
    current_images: Option<KittyImageSet>,
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
                current_images: None,
                next_id: 1,
            })
    }
    pub fn take_current_images(&mut self) -> Option<KittyImageSet> {
        self.current_images.take()
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
    /// return a new image id which is assumed will be used
    fn new_id(&mut self) -> usize {
        let new_id = self.next_id;
        self.next_id += 1;
        self.current_images
            .get_or_insert_with(Vec::new)
            .push(new_id);
        new_id
    }
    /// render the image by sending multiple kitty escape sequence, each
    /// one with part of the image raw data (encoded as base64)
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
        let id = self.new_id();
        debug!("rendering_dim: {:?}", (c, r));
        let mut pos = 0;
        loop {
            if pos + CHUNK_SIZE < encoded.len() {
                write!(w,
                    "\u{1b}_Ga=T,f=32,t=d,i={},s={},v={},c={},r={},m=1;{}\u{1b}\\",
                    id,
                    width,
                    height,
                    c,
                    r,
                    &encoded[pos..pos+CHUNK_SIZE],
                )?;
                pos += CHUNK_SIZE;
            } else {
                // last chunk
                write!(w,
                    "\u{1b}_Gm=0;{}\u{1b}\\",
                    &encoded[pos..encoded.len()],
                )?;
                break;
            }
        }
        Ok(())
    }
    /// render the image by writing the raw data in a temporary file
    /// then giving to kitty the path to this file in the payload of
    /// a unique kitty ecape sequence
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
        let id = self.new_id();
        write!(w,
            "\u{1b}_Ga=T,f=32,t=t,i={},s={},v={},c={},r={};{}\u{1b}\\",
            id,
            width,
            height,
            c,
            r,
            encoded_path,
        )?;
        debug!("file len: {}", temp_file.metadata().unwrap().len());
        Ok(())
    }
    pub fn erase(
        &mut self,
        w: &mut W,
        ids: KittyImageSet,
    ) -> Result<(), ProgramError> {
        for id in ids {
            debug!("erase kitty image {}", id);
            write!(w, "\u{1b}_Ga=d,d=I,i={}\u{1b}\\", id)?;
        }
        Ok(())
    }
    /// erase all kitty images, even the forgetted ones
    pub fn erase_all(
        &mut self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        write!(w, "\u{1b}_Ga=d,d=A\u{1b}\\")?;
        self.current_images = None;
        Ok(())
    }
}




