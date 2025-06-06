use {
    super::detect_support::is_kitty_graphics_protocol_supported,
    base64::{
        engine::general_purpose::STANDARD as BASE64,
        Engine,
    },
    crate::{
        display::{
            cell_size_in_pixels,
            W,
        },
        errors::ProgramError,
    },
    base64,
    cli_log::*,
    crokey::crossterm::{
        cursor,
        QueueableCommand,
        style::Color,
    },
    image::{
        DynamicImage,
        GenericImageView,
        RgbImage,
        RgbaImage,
    },
    lru::LruCache,
    rustc_hash::FxBuildHasher,
    serde::Deserialize,
    std::{
        fs::File,
        io::{self, Write},
        num::NonZeroUsize,
        path::{Path, PathBuf},
    },
    tempfile,
    termimad::{fill_bg, Area},
};

/// How to send the image to kitty
///
/// Note that I didn't test yet the named shared memory
/// solution offered by kitty.
///
/// Documentation:
///  https://sw.kovidgoyal.net/kitty/graphics-protocol/#the-transmission-medium
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransmissionMedium {
    /// write a temp file, then give its path to kitty
    /// in the payload of the escape sequence. It's quite
    /// fast on SSD but a big downside is that it doesn't
    /// work if you're distant
    #[default]
    TempFile,
    /// send the whole rgb or rgba data, encoded in base64,
    /// in the payloads of several escape sequence (each one
    /// containing at most 4096 bytes). Works if broot runs
    /// on remote.
    Chunks,
}

#[derive(Debug, Clone)]
pub struct KittyImageRendererOptions {
    pub transmission_medium: TransmissionMedium,
    pub kept_temp_files: NonZeroUsize,
}

enum ImageData<'i> {
    RgbRef(&'i RgbImage),
    RgbaRef(&'i RgbaImage),
    Rgb(RgbImage),
}
impl<'i> From<&'i DynamicImage> for ImageData<'i> {
    fn from(img: &'i DynamicImage) -> Self {
        if let Some(rgb) = img.as_rgb8() {
            debug!("using rgb");
            Self::RgbRef(rgb)
        } else if let Some(rgba) = img.as_rgba8() {
            debug!("using rgba");
            Self::RgbaRef(rgba)
        } else {
            debug!("converting to rgb8");
            Self::Rgb(img.to_rgb8())
        }
    }
}
impl ImageData<'_> {
    fn kitty_format(&self) -> &'static str {
        match self {
            Self::RgbaRef(_) => "32",
            _ => "24",
        }
    }
    fn bytes(&self) -> &[u8] {
        match self {
            Self::RgbRef(img) => img.as_raw(),
            Self::RgbaRef(img) => img.as_raw(),
            Self::Rgb(img) => img.as_raw(),
        }
    }
}

/// The max size of a data payload in a kitty escape sequence
/// according to kitty's documentation
const CHUNK_SIZE: usize = 4096;


fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + (0 != a % b) as u32
}

/// The image renderer, with knowledge of the console cells
/// dimensions, and built only on a compatible terminal
#[derive(Debug)]
pub struct KittyImageRenderer {
    cell_width: u32,
    cell_height: u32,
    next_id: usize,
    options: KittyImageRendererOptions,
    /// paths of temp files which have been written, with key
    /// being the input image path
    temp_files: LruCache<String, PathBuf, FxBuildHasher>,
}

/// An image prepared for a precise area on screen
struct KittyImage<'i> {
    id: usize,
    data: ImageData<'i>,
    img_width: u32,
    img_height: u32,
    area: Area,
}
impl<'i> KittyImage<'i> {
    fn new<'r>(
        src: &'i DynamicImage,
        available_area: &Area,
        renderer: &'r mut KittyImageRenderer,
    ) -> Self {
        let (img_width, img_height) = src.dimensions();
        let area = renderer.rendering_area(img_width, img_height, available_area);
        let data = src.into();
        let id = renderer.new_id();
        Self {
            id,
            data,
            img_width,
            img_height,
            area,
        }
    }
    /// Render the image by sending multiple kitty escape sequences, each
    /// one with part of the image raw data (encoded as base64)
    fn print_with_chunks(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        let encoded = BASE64.encode(self.data.bytes());
        w.queue(cursor::MoveTo(self.area.left, self.area.top))?;
        let mut pos = 0;
        loop {
            if pos + CHUNK_SIZE < encoded.len() {
                write!(
                    w,
                    "\u{1b}_Ga=T,f={},t=d,i={},s={},v={},c={},r={},m=1;{}\u{1b}\\",
                    self.data.kitty_format(),
                    self.id,
                    self.img_width,
                    self.img_height,
                    self.area.width,
                    self.area.height,
                    &encoded[pos..pos + CHUNK_SIZE],
                )?;
                pos += CHUNK_SIZE;
            } else {
                // last chunk
                write!(w, "\u{1b}_Gm=0;{}\u{1b}\\", &encoded[pos..encoded.len()],)?;
                break;
            }
        }
        Ok(())
    }
    /// Render the image by writing the raw data in a temporary file
    /// then giving to kitty the path to this file in the payload of
    /// a unique kitty ecape sequence
    pub fn print_with_temp_file(
        &self,
        w: &mut W,
        temp_file: Option<File>, // if None, no need to write it
        temp_file_path: &Path,
    ) -> Result<(), ProgramError> {
        if let Some(mut temp_file) = temp_file {
            temp_file.write_all(self.data.bytes())?;
            temp_file.flush()?;
            debug!("file len: {}", temp_file.metadata().unwrap().len());
        }
        let path = temp_file_path.to_str()
            .ok_or_else(|| io::Error::other("Path can't be converted to UTF8"))?;
        let encoded_path = BASE64.encode(path);
        debug!("temp file written: {:?}", path);
        w.queue(cursor::MoveTo(self.area.left, self.area.top))?;
        write!(
            w,
            "\u{1b}_Ga=T,f={},t=t,i={},s={},v={},c={},r={};{}\u{1b}\\",
            self.data.kitty_format(),
            self.id,
            self.img_width,
            self.img_height,
            self.area.width,
            self.area.height,
            encoded_path,
        )?;
        Ok(())
    }
}

impl KittyImageRenderer {
    /// Called only once (at most) by the KittyManager
    pub fn new(
        options: KittyImageRendererOptions,
    ) -> Option<Self> {
        if !is_kitty_graphics_protocol_supported() {
            return None;
        }
        let hasher = FxBuildHasher;
        let temp_files = LruCache::with_hasher(options.kept_temp_files, hasher);
        cell_size_in_pixels()
            .ok()
            .map(|(cell_width, cell_height)| Self {
                cell_width,
                cell_height,
                next_id: 1,
                options,
                temp_files,
            })
    }
    pub fn delete_temp_files(&mut self) {
        for (_, temp_file_path) in self.temp_files.into_iter() {
            debug!("removing temp file: {:?}", temp_file_path);
            if let Err(e) = std::fs::remove_file(temp_file_path) {
                error!("failed to remove temp file: {:?}", e);
            }
        }
    }
    /// return a new image id
    fn new_id(&mut self) -> usize {
        let new_id = self.next_id;
        self.next_id += 1;
        new_id
    }
    /// Clean the area, then print the dynamicImage and
    /// return the KittyImageId for later removal from screen
    pub fn print(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
    ) -> Result<usize, ProgramError> {

        // clean the background below (and around) the image
        for y in area.top..area.top + area.height {
            w.queue(cursor::MoveTo(area.left, y))?;
            fill_bg(w, area.width as usize, bg)?;
        }

        let img = KittyImage::new(src, area, self);
        debug!("transmission medium: {:?}", self.options.transmission_medium);
        w.flush()?;
        match self.options.transmission_medium {
            TransmissionMedium::TempFile => {
                let temp_file_key = format!(
                    "{:?}-{}x{}",
                    src_path,
                    img.img_width,
                    img.img_height,
                );
                let mut old_path = None;
                if let Some(cached_path) = self.temp_files.pop(&temp_file_key) {
                    if cached_path.exists() {
                        old_path = Some(cached_path);
                    }
                }
                let temp_file_path = if let Some(temp_file_path) = old_path {
                    // the temp file is still there
                    img.print_with_temp_file(w, None, &temp_file_path)?;
                    temp_file_path
                } else {
                    // either the temp file itself has been removed (unlikely), the temp
                    // cache entry has been removed, or we just never viewed this image
                    // with this size before
                    let (temp_file, path) = tempfile::Builder::new()
                        .prefix("broot-img-preview")
                        .tempfile()?
                        .keep()
                        .map_err(|_| io::Error::other("temp file can't be kept"))?;
                    img.print_with_temp_file(w, Some(temp_file), &path)?;
                    path
                };
                if let Some((_, old_path)) = self.temp_files.push(temp_file_key, temp_file_path) {
                    debug!("removing temp file: {:?}", &old_path);
                    if let Err(e) = std::fs::remove_file(&old_path) {
                        error!("failed to remove temp file: {:?}", e);
                    }
                }
            }
            TransmissionMedium::Chunks => img.print_with_chunks(w)?,
        }
        Ok(img.id)
    }
    fn rendering_area(
        &self,
        img_width: u32,
        img_height: u32,
        area: &Area,
    ) -> Area {
        let area_cols: u32 = area.width.into();
        let area_rows: u32 = area.height.into();
        let rdim = self.rendering_dim(img_width, img_height, area_cols, area_rows);
        Area::new(
            area.left + ((area_cols - rdim.0) / 2) as u16,
            area.top + ((area_rows - rdim.1) / 2) as u16,
            rdim.0 as u16,
            rdim.1 as u16,
        )
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
}

