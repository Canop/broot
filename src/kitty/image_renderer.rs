use {
    super::{
        detect_support::{
            get_tmux_nest_count,
            is_kitty_graphics_protocol_supported,
            is_ssh,
        },
        terminal_esc::{
            get_esc_seq,
            get_tmux_header,
            get_tmux_tail,
        },
    },
    crate::{
        display::{
            W,
            cell_size_in_pixels,
        },
        errors::ProgramError,
    },
    base64::{
        self,
        Engine,
        engine::general_purpose::STANDARD as BASE64,
    },
    cli_log::*,
    crokey::crossterm::{
        QueueableCommand,
        cursor,
        style::Color,
    },
    flate2::{
        Compression,
        write::ZlibEncoder,
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
        io::{
            self,
            Read,
            Write,
        },
        num::NonZeroUsize,
        path::{
            Path,
            PathBuf,
        },
    },
    tempfile,
    termimad::{
        Area,
        fill_bg,
    },
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
    pub force: bool,
    pub transmission_medium: TransmissionMedium,
    pub kept_temp_files: NonZeroUsize,
    pub is_tmux: bool,
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

/// Unicode placeholder character
const PLACHOLDER: &str = "\u{10EEEE}";
/// Unicode placeholder diacritic characters
#[rustfmt::skip]
const DIACRITICS: &[&str] = &[
    "\u{0305}", "\u{030D}", "\u{030E}", "\u{0310}", "\u{0312}", "\u{033D}", "\u{033E}", "\u{033F}",
    "\u{0346}", "\u{034A}", "\u{034B}", "\u{034C}", "\u{0350}", "\u{0351}", "\u{0352}", "\u{0357}",
    "\u{035B}", "\u{0363}", "\u{0364}", "\u{0365}", "\u{0366}", "\u{0367}", "\u{0368}", "\u{0369}",
    "\u{036A}", "\u{036B}", "\u{036C}", "\u{036D}", "\u{036E}", "\u{036F}", "\u{0483}", "\u{0484}",
    "\u{0485}", "\u{0486}", "\u{0487}", "\u{0592}", "\u{0593}", "\u{0594}", "\u{0595}", "\u{0597}",
    "\u{0598}", "\u{0599}", "\u{059C}", "\u{059D}", "\u{059E}", "\u{059F}", "\u{05A0}", "\u{05A1}",
    "\u{05A8}", "\u{05A9}", "\u{05AB}", "\u{05AC}", "\u{05AF}", "\u{05C4}", "\u{0610}", "\u{0611}",
    "\u{0612}", "\u{0613}", "\u{0614}", "\u{0615}", "\u{0616}", "\u{0617}", "\u{0657}", "\u{0658}",
    "\u{0659}", "\u{065A}", "\u{065B}", "\u{065D}", "\u{065E}", "\u{06D6}", "\u{06D7}", "\u{06D8}",
    "\u{06D9}", "\u{06DA}", "\u{06DB}", "\u{06DC}", "\u{06DF}", "\u{06E0}", "\u{06E1}", "\u{06E2}",
    "\u{06E4}", "\u{06E7}", "\u{06E8}", "\u{06EB}", "\u{06EC}", "\u{0730}", "\u{0732}", "\u{0733}",
    "\u{0735}", "\u{0736}", "\u{073A}", "\u{073D}", "\u{073F}", "\u{0740}", "\u{0741}", "\u{0743}",
    "\u{0745}", "\u{0747}", "\u{0749}", "\u{074A}", "\u{07EB}", "\u{07EC}", "\u{07ED}", "\u{07EE}",
    "\u{07EF}", "\u{07F0}", "\u{07F1}", "\u{07F3}", "\u{0816}", "\u{0817}", "\u{0818}", "\u{0819}",
    "\u{081B}", "\u{081C}", "\u{081D}", "\u{081E}", "\u{081F}", "\u{0820}", "\u{0821}", "\u{0822}",
    "\u{0823}", "\u{0825}", "\u{0826}", "\u{0827}", "\u{0829}", "\u{082A}", "\u{082B}", "\u{082C}",
    "\u{082D}", "\u{0951}", "\u{0953}", "\u{0954}", "\u{0F82}", "\u{0F83}", "\u{0F86}", "\u{0F87}",
    "\u{135D}", "\u{135E}", "\u{135F}", "\u{17DD}", "\u{193A}", "\u{1A17}", "\u{1A75}", "\u{1A76}",
    "\u{1A77}", "\u{1A78}", "\u{1A79}", "\u{1A7A}", "\u{1A7B}", "\u{1A7C}", "\u{1B6B}", "\u{1B6D}",
    "\u{1B6E}", "\u{1B6F}", "\u{1B70}", "\u{1B71}", "\u{1B72}", "\u{1B73}", "\u{1CD0}", "\u{1CD1}",
    "\u{1CD2}", "\u{1CDA}", "\u{1CDB}", "\u{1CE0}", "\u{1DC0}", "\u{1DC1}", "\u{1DC3}", "\u{1DC4}",
    "\u{1DC5}", "\u{1DC6}", "\u{1DC7}", "\u{1DC8}", "\u{1DC9}", "\u{1DCB}", "\u{1DCC}", "\u{1DD1}",
    "\u{1DD2}", "\u{1DD3}", "\u{1DD4}", "\u{1DD5}", "\u{1DD6}", "\u{1DD7}", "\u{1DD8}", "\u{1DD9}",
    "\u{1DDA}", "\u{1DDB}", "\u{1DDC}", "\u{1DDD}", "\u{1DDE}", "\u{1DDF}", "\u{1DE0}", "\u{1DE1}",
    "\u{1DE2}", "\u{1DE3}", "\u{1DE4}", "\u{1DE5}", "\u{1DE6}", "\u{1DFE}", "\u{20D0}", "\u{20D1}",
    "\u{20D4}", "\u{20D5}", "\u{20D6}", "\u{20D7}", "\u{20DB}", "\u{20DC}", "\u{20E1}", "\u{20E7}",
    "\u{20E9}", "\u{20F0}", "\u{2CEF}", "\u{2CF0}", "\u{2CF1}", "\u{2DE0}", "\u{2DE1}", "\u{2DE2}",
    "\u{2DE3}", "\u{2DE4}", "\u{2DE5}", "\u{2DE6}", "\u{2DE7}", "\u{2DE8}", "\u{2DE9}", "\u{2DEA}",
    "\u{2DEB}", "\u{2DEC}", "\u{2DED}", "\u{2DEE}", "\u{2DEF}", "\u{2DF0}", "\u{2DF1}", "\u{2DF2}",
    "\u{2DF3}", "\u{2DF4}", "\u{2DF5}", "\u{2DF6}", "\u{2DF7}", "\u{2DF8}", "\u{2DF9}", "\u{2DFA}",
    "\u{2DFB}", "\u{2DFC}", "\u{2DFD}", "\u{2DFE}", "\u{2DFF}", "\u{A66F}", "\u{A67C}", "\u{A67D}",
    "\u{A6F0}", "\u{A6F1}", "\u{A8E0}", "\u{A8E1}", "\u{A8E2}", "\u{A8E3}", "\u{A8E4}", "\u{A8E5}",
    "\u{A8E6}", "\u{A8E7}", "\u{A8E8}", "\u{A8E9}", "\u{A8EA}", "\u{A8EB}", "\u{A8EC}", "\u{A8ED}",
    "\u{A8EE}", "\u{A8EF}", "\u{A8F0}", "\u{A8F1}", "\u{AAB0}", "\u{AAB2}", "\u{AAB3}", "\u{AAB7}",
    "\u{AAB8}", "\u{AABE}", "\u{AABF}", "\u{AAC1}", "\u{FE20}", "\u{FE21}", "\u{FE22}", "\u{FE23}",
    "\u{FE24}", "\u{FE25}", "\u{FE26}", "\u{10A0F}", "\u{10A38}", "\u{1D185}", "\u{1D186}",
    "\u{1D187}", "\u{1D188}", "\u{1D189}", "\u{1D1AA}", "\u{1D1AB}", "\u{1D1AC}", "\u{1D1AD}",
    "\u{1D242}", "\u{1D243}", "\u{1D244}"
];

fn div_ceil(
    a: u32,
    b: u32,
) -> u32 {
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

enum KittyImageData<'i> {
    Png { path: PathBuf },
    Image { data: ImageData<'i> },
}

/// An image prepared for a precise area on screen
struct KittyImage<'i> {
    id: usize,
    data: KittyImageData<'i>,
    img_width: u32,
    img_height: u32,
    area: Area,
    is_tmux: bool,
    tmux_nest_count: u32,
}
impl<'i> KittyImage<'i> {
    fn new<'r>(
        src: &'i DynamicImage,
        png_path: Option<PathBuf>,
        available_area: &Area,
        renderer: &'r mut KittyImageRenderer,
    ) -> Self {
        let (img_width, img_height) = src.dimensions();
        let area = renderer.rendering_area(img_width, img_height, available_area);
        let data = if let Some(path) = png_path {
            KittyImageData::Png { path }
        } else {
            KittyImageData::Image { data: src.into() }
        };
        let id = renderer.new_id();
        let is_tmux = renderer.options.is_tmux;
        let tmux_nest_count = if is_tmux { get_tmux_nest_count() } else { 0 };
        Self {
            id,
            data,
            img_width,
            img_height,
            area,
            is_tmux,
            tmux_nest_count,
        }
    }
    fn print_placeholder_grid(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        for y in 0..(self.area.height).min(DIACRITICS.len() as u16) {
            w.queue(cursor::MoveTo(self.area.left, self.area.top + y))?;
            write!(w, "\u{1b}[38;5;{}m", self.id)?;
            for x in 0..(self.area.width).min(DIACRITICS.len() as u16) {
                write!(
                    w,
                    "{}{}{}",
                    PLACHOLDER, DIACRITICS[y as usize], DIACRITICS[x as usize]
                )?;
            }
            write!(w, "\u{1b}[39m")?;
        }
        Ok(())
    }
    fn compress(data: &[u8]) -> Result<Vec<u8>, ProgramError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).expect("Zlib encoder error");
        Ok(encoder.finish().expect("Zlib encoder error"))
    }
    /// Render the image by sending multiple kitty escape sequences, each
    /// one with part of the image raw data (encoded as base64)
    fn print_with_chunks(
        &self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        let esc = get_esc_seq(self.tmux_nest_count);
        let tmux_header = self
            .is_tmux
            .then_some(get_tmux_header(self.tmux_nest_count));
        let tmux_tail = self.is_tmux.then_some(get_tmux_tail(self.tmux_nest_count));
        let mut png_buf = Vec::new();
        let (bytes, compression_tag, format) = match &self.data {
            KittyImageData::Png { path } => {
                // Compressing PNG files increases the size
                File::open(path)?.read_to_end(&mut png_buf)?;
                (png_buf, "", "100")
            }
            KittyImageData::Image { data } => (
                KittyImage::compress(data.bytes())?,
                "o=z,",
                data.kitty_format(),
            ),
        };
        let encoded = BASE64.encode(bytes);
        let mut pos = 0;
        if let Some(s) = &tmux_header {
            write!(w, "{s}")?;
        }
        write!(
            w,
            "{}_Gq=2,a=t,f={},t=d,i={},s={},v={},{}",
            &esc, format, self.id, self.img_width, self.img_height, compression_tag,
        )?;
        loop {
            if pos != 0 {
                if let Some(s) = &tmux_header {
                    write!(w, "{s}")?;
                }
                write!(w, "{}_Gq=2,", &esc)?;
            }
            if pos + CHUNK_SIZE < encoded.len() {
                write!(w, "m=1;{}{}\\", &encoded[pos..pos + CHUNK_SIZE], &esc)?;
                pos += CHUNK_SIZE;
                if let Some(s) = &tmux_tail {
                    write!(w, "{s}")?;
                }
            } else {
                // last chunk
                write!(w, "m=0;{}{}\\", &encoded[pos..encoded.len()], &esc)?;
                if let Some(s) = &tmux_tail {
                    write!(w, "{s}")?;
                }
                // display image
                if let Some(s) = &tmux_header {
                    write!(w, "{s}")?;
                }
                write!(
                    w,
                    "{}_Gq=2,a=p,U=1,i={},c={},r={}{}\\",
                    &esc, self.id, self.area.width, self.area.height, &esc,
                )?;
                if let Some(s) = &tmux_tail {
                    write!(w, "{s}")?;
                }
                self.print_placeholder_grid(w)?;
                break;
            }
        }
        Ok(())
    }
    /// Render the image by writing the raw data in a temporary file
    /// then giving to kitty the path to this file in the payload of
    /// a unique kitty escape sequence
    pub fn print_with_temp_file(
        &self,
        w: &mut W,
        temp_file: Option<File>, // if None, no need to write it
        temp_file_path: &Path,
    ) -> Result<(), ProgramError> {
        let esc = get_esc_seq(self.tmux_nest_count);
        let tmux_header = self
            .is_tmux
            .then_some(get_tmux_header(self.tmux_nest_count));
        let tmux_tail = self.is_tmux.then_some(get_tmux_tail(self.tmux_nest_count));
        // Compression slows things down
        let (path, format, transmission) = match &self.data {
            KittyImageData::Png { path } => (path.as_path(), "100", "f"),
            KittyImageData::Image { data } => {
                if let Some(mut temp_file) = temp_file {
                    temp_file.write_all(data.bytes())?;
                    temp_file.flush()?;
                    debug!("file len: {}", temp_file.metadata().unwrap().len());
                }
                (temp_file_path, data.kitty_format(), "t")
            }
        };
        let path = path
            .to_str()
            .ok_or_else(|| io::Error::other("Path can't be converted to UTF8"))?;
        let encoded_path = BASE64.encode(path);
        if let KittyImageData::Image { data: _ } = self.data {
            debug!("temp file written: {:?}", path);
        }
        if let Some(s) = &tmux_header {
            write!(w, "{s}")?;
        }
        write!(
            w,
            "{}_Gq=2,a=T,U=1,f={},t={},i={},s={},v={},c={},r={};{}{}\\",
            &esc,
            format,
            transmission,
            self.id,
            self.img_width,
            self.img_height,
            self.area.width,
            self.area.height,
            encoded_path,
            &esc,
        )?;
        if let Some(s) = &tmux_tail {
            write!(w, "{s}")?;
        }
        self.print_placeholder_grid(w)?;
        Ok(())
    }
}

impl KittyImageRenderer {
    /// Called only once (at most) by the KittyManager
    pub fn new(options: KittyImageRendererOptions) -> Option<Self> {
        if !options.force && !is_kitty_graphics_protocol_supported() {
            return None;
        }
        let hasher = FxBuildHasher;
        let temp_files = LruCache::with_hasher(options.kept_temp_files, hasher);
        let options = if is_ssh() {
            KittyImageRendererOptions {
                transmission_medium: TransmissionMedium::Chunks,
                ..options
            }
        } else {
            options
        };
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
    fn is_path_png(path: &Path) -> bool {
        match path.extension() {
            Some(ext) => ext == "png" || ext == "PNG",
            None => false,
        }
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

        let png_path = KittyImageRenderer::is_path_png(src_path).then_some(src_path.to_path_buf());
        let img = KittyImage::new(src, png_path, area, self);
        debug!(
            "transmission medium: {:?}",
            self.options.transmission_medium
        );
        w.flush()?;
        match self.options.transmission_medium {
            TransmissionMedium::TempFile => {
                let temp_file_key = format!("{:?}-{}x{}", src_path, img.img_width, img.img_height,);
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
                        .prefix("broot-tty-graphics-protocol-")
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
