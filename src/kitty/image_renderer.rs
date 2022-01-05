use {
    crate::{
        display::{
            cell_size_in_pixels,
            W,
        },
        errors::ProgramError,
    },
    base64,
    cli_log::*,
    crossterm::{
        cursor,
        QueueableCommand,
    },
    image::{
        DynamicImage,
        GenericImageView,
        RgbImage,
        RgbaImage,
    },
    std::{
        env,
        io::{self, Write},
    },
    tempfile,
    termimad::Area,
};


/// How to send the image to kitty
///
/// Note that I didn't test yet the named shared memory
/// solution offered by kitty.
#[derive(Debug)]
pub enum TransmissionMedium {
    /// write a temp file, then give its path to kitty
    /// in the payload of the escape sequence. It's quite
    /// fast on SSD but a big downside is that it doesn't
    /// work if you're distant
    TempFile,
    /// send the whole rgb or rgba data, encoded in base64,
    /// in the payloads of several escape sequence (each one
    /// containing at most 4096 bytes). Works if broot runs
    /// on remote.
    Chunks,
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
impl<'i> ImageData<'i> {
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

/// this is called only once, and cached in the kitty manager's MaybeRenderer state
#[allow(unreachable_code)]
fn is_kitty_graphics_protocol_supported() -> bool {
    debug!("is_kitty_graphics_protocol_supported ?");

    #[cfg(not(unix))]
    {
        // because cell_size_in_pixels isn't implemented on Windows
        debug!("no kitty support yet on Windows");
        return false;
    }

    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("{:?} = {:?}", env_var, env_val);
            let env_val = env_val.to_ascii_lowercase();
            for name in ["kitty", "wezterm"] {
                if env_val.contains(name) {
                    debug!(" -> env var indicates kitty support");
                    return true;
                }
            }
        }
    }

    // Checking support with a proper CSI sequence should be the prefered way but
    // it doesn't work reliably on wezterm and requires a wait on other terminal.
    // Only Kitty does supports it perfectly and it's not even necessary on this
    // terminal because we can just check the env var TERM.
    #[cfg(feature = "kitty-csi-check")]
    {
        let start = std::time::Instant::now();
        const TIMEOUT_MS: isize = 400;
        let s = match xterm_query::query("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c", TIMEOUT_MS) {
            Err(e) => {
                debug!("xterm querying failed: {}", e);
                false
            }
            Ok(response) => {
                response.starts_with("\x1b_Gi=31;OK\x1b")
            }
        };
        debug!("Xterm querying took {:?}", start.elapsed());
        debug!("kitty protocol support: {:?}", s);
        return s;
    }
    false
}


fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + (0 != a % b) as u32
}

/// the image renderer, with knowledge of the
/// console cells dimensions, and built only on Kitty.
///
#[derive(Debug)]
pub struct KittyImageRenderer {
    cell_width: u32,
    cell_height: u32,
    next_id: usize,
    pub transmission_medium: TransmissionMedium,
}

/// An image prepared for a precise area on screen
///
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
    /// render the image by sending multiple kitty escape sequence, each
    /// one with part of the image raw data (encoded as base64)
    fn print_with_chunks(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        let encoded = base64::encode(self.data.bytes());
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
    /// render the image by writing the raw data in a temporary file
    /// then giving to kitty the path to this file in the payload of
    /// a unique kitty ecape sequence
    pub fn print_with_temp_file(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        let (mut temp_file, path) = tempfile::Builder::new()
            .prefix("broot-img-preview")
            .tempfile()?
            .keep()
            .map_err(|_| io::Error::new(
                io::ErrorKind::Other,
                "temp file can't be kept",
            ))?;
        temp_file.write_all(self.data.bytes())?;
        temp_file.flush()?;
        let path = path.to_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::Other,
                "Path can't be converted to UTF8",
            ))?;
        let encoded_path = base64::encode(path);
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
        debug!("file len: {}", temp_file.metadata().unwrap().len());
        Ok(())
    }
}

impl KittyImageRenderer {
    /// Called only once (at most) by the KittyManager
    pub fn new() -> Option<Self> {
        if !is_kitty_graphics_protocol_supported() {
            return None;
        }
        cell_size_in_pixels()
            .ok()
            .map(|(cell_width, cell_height)| Self {
                cell_width,
                cell_height,
                next_id: 1,
                transmission_medium: TransmissionMedium::Chunks,
            })
    }
    /// return a new image id
    fn new_id(&mut self) -> usize {
        let new_id = self.next_id;
        self.next_id += 1;
        new_id
    }
    /// Print the dynamicImage and return the KittyImageId
    /// for later removal from screen
    pub fn print(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        area: &Area,
    ) -> Result<usize, ProgramError> {
        let img = KittyImage::new(src, area, self);
        debug!("transmission medium: {:?}", self.transmission_medium);
        match self.transmission_medium {
            TransmissionMedium::TempFile => img.print_with_temp_file(w)?,
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

