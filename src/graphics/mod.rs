mod fit;
pub(crate) mod image_data;
pub(crate) mod terminal;

pub(crate) use fit::rendering_area;

use {
    crate::{display::W, errors::ProgramError, image::zune_compat::DynamicImage},
    crokey::crossterm::style::Color,
    std::path::{Path, PathBuf},
    termimad::Area,
};

pub type ImageId = usize;

/// A terminal-graphics protocol able to draw an image into a screen area.
pub trait GraphicsRenderer: Send {
    /// Clear `area` and draw `src` (already fitted to the area's pixels) into it.
    /// Returns `Some(id)` for protocols whose images persist above the cells and
    /// must be erased later (Kitty); `None` for protocols drawn into the cells and
    /// cleared by normal repaint (Sixel).
    fn print(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
    ) -> Result<Option<ImageId>, ProgramError>;

    /// Erase a previously drawn image by id. No-op for inline protocols.
    fn erase_image(&self, w: &mut W, id: ImageId) -> Result<(), ProgramError>;

    fn delete_temp_files(&mut self) {}

    /// Return the (width, height) of a terminal cell in pixels, as cached at
    /// construction time. Used to avoid re-querying the terminal every repaint.
    fn cell_size(&self) -> (u32, u32);

    /// Upper bound, in pixels, on the size an image should be fitted to before
    /// rendering, if the protocol/terminal imposes one. Larger images would be
    /// cropped (e.g. xterm crops oversized Sixel). `None` means no known bound
    /// (e.g. Kitty).
    fn max_render_size(&self) -> Option<(u32, u32)> {
        None
    }

    /// Whether a changed/removed inline image requires a full-screen clear to
    /// erase, rather than being cleared by repainting the cells it occupied.
    /// Default false (Kitty erases by id; most Sixel terminals drop it when the
    /// cells are overwritten). True for Konsole, which keeps Sixel until the
    /// screen is cleared; the manager then issues `ESC [ 2J` + a full redraw.
    fn needs_reclear_on_change(&self) -> bool {
        false
    }
}

/// Outcome of an attempt to draw an image with a terminal graphics protocol.
///
/// This distinguishes "an image was drawn" from "there's an id to erase later":
/// Kitty draws and returns an id (its images float above the cells and must be
/// erased explicitly); Sixel draws into the cells and has no id. Both cases mean
/// the caller must NOT fall back to the text (half-block) rendering — only
/// `Unsupported` should trigger that fallback.
pub enum ImageRendering {
    /// An image was drawn into the area. `Some(id)` for protocols whose images
    /// must be erased later (Kitty); `None` for inline protocols (Sixel).
    Drawn(Option<ImageId>),
    /// No graphics protocol is available; the caller should use its text
    /// (half-block) fallback.
    Unsupported,
}

use {
    crate::app::AppContext,
    once_cell::sync::Lazy,
    std::sync::Mutex,
};

static MANAGER: Lazy<Mutex<GraphicsManager>> = Lazy::new(|| {
    Mutex::new(GraphicsManager {
        rendered_images: Vec::new(),
        renderer: MaybeRenderer::Untested,
        frame_sixels: Vec::new(),
        prev_sixels: Vec::new(),
        forced_redraw: false,
    })
});

pub fn manager() -> &'static Mutex<GraphicsManager> {
    &MANAGER
}

#[derive(Debug)]
struct RenderedImage {
    image_id: ImageId,
    drawing_count: usize,
}


enum MaybeRenderer {
    Untested,
    Disabled,
    Enabled { renderer: Box<dyn GraphicsRenderer> },
}

/// Choose a graphics renderer for this terminal, honoring the
/// BROOT_GRAPHICS_PROTOCOL override (kitty | sixel | none).
fn select_renderer(con: &AppContext) -> Option<Box<dyn GraphicsRenderer>> {
    use crate::kitty::KittyGraphicsDisplay;

    // The legacy `kitty-graphics-display = "none"` config predates Sixel and is
    // treated as the global "disable all terminal graphics" switch (both Kitty
    // and Sixel), for backward compatibility. The option name is now a misnomer.
    if con.kitty_graphics_display == KittyGraphicsDisplay::None {
        return None;
    }

    let forced = std::env::var("BROOT_GRAPHICS_PROTOCOL")
        .ok()
        .map(|s| s.to_ascii_lowercase());
    match forced.as_deref() {
        Some("none") => return None,
        Some("kitty") => {
            let r = crate::kitty::build_kitty_renderer(con)
                .map(|r| Box::new(r) as Box<dyn GraphicsRenderer>);
            if r.is_none() {
                warn!("BROOT_GRAPHICS_PROTOCOL=kitty but Kitty renderer failed to initialize");
            }
            return r;
        }
        Some("sixel") => {
            let r = crate::sixel::SixelRenderer::new()
                .map(|r| Box::new(r) as Box<dyn GraphicsRenderer>);
            if r.is_none() {
                warn!("BROOT_GRAPHICS_PROTOCOL=sixel but Sixel renderer failed to initialize");
            }
            return r;
        }
        _ => {}
    }

    // auto: Kitty first (env-var detection, no I/O), then Sixel (DA1 probe)
    if let Some(r) = crate::kitty::build_kitty_renderer(con) {
        debug!("using kitty graphics protocol");
        return Some(Box::new(r));
    }
    if crate::sixel::detect_sixel_support() {
        if let Some(r) = crate::sixel::SixelRenderer::new() {
            debug!("using sixel graphics protocol");
            return Some(Box::new(r));
        }
    }
    debug!("no terminal graphics protocol available");
    None
}

pub struct GraphicsManager {
    rendered_images: Vec<RenderedImage>,
    renderer: MaybeRenderer,
    /// `(path, area)` of the inline (Sixel) images drawn or kept in the current
    /// draw pass. Unlike Kitty (erased by id), a Sixel is drawn into the cells
    /// and, on terminals that keep it until the screen is cleared (Konsole),
    /// can't be removed by repainting text. Tracking what's on screen frame to
    /// frame lets `reclear_needed` notice when one changes/leaves/moves.
    frame_sixels: Vec<(PathBuf, Area)>,
    /// `frame_sixels` from the last committed frame.
    prev_sixels: Vec<(PathBuf, Area)>,
    /// Set during the redraw pass after a full-screen clear, so images that
    /// would otherwise be kept in place repaint (the clear wiped them).
    forced_redraw: bool,
}

impl GraphicsManager {
    fn renderer_if_tested(&mut self) -> Option<&mut dyn GraphicsRenderer> {
        match &mut self.renderer {
            MaybeRenderer::Enabled { renderer } => Some(renderer.as_mut()),
            _ => None,
        }
    }
    pub fn delete_temp_files(&mut self) {
        if let MaybeRenderer::Enabled { renderer } = &mut self.renderer {
            renderer.delete_temp_files();
        }
    }
    pub fn renderer(&mut self, con: &AppContext) -> Option<&mut dyn GraphicsRenderer> {
        if matches!(self.renderer, MaybeRenderer::Disabled) {
            return None;
        }
        if matches!(self.renderer, MaybeRenderer::Enabled { .. }) {
            return self.renderer_if_tested();
        }
        let chosen = select_renderer(con);
        self.renderer = match chosen {
            Some(renderer) => MaybeRenderer::Enabled { renderer },
            None => MaybeRenderer::Disabled,
        };
        self.renderer_if_tested()
    }
    pub fn keep(&mut self, kept_id: ImageId, drawing_count: usize) {
        for image in &mut self.rendered_images {
            if image.image_id == kept_id {
                image.drawing_count = drawing_count;
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_print_image(
        &mut self,
        w: &mut W,
        src: &crate::image::SourceImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
        drawing_count: usize,
        con: &AppContext,
    ) -> Result<ImageRendering, ProgramError> {
        if let Some(renderer) = self.renderer(con) {
            let (cell_width, cell_height) = renderer.cell_size();
            let mut area_width = area.width as u32 * cell_width;
            let mut area_height = area.height as u32 * cell_height;
            // Fit within the protocol/terminal's render bound so the image isn't
            // cropped (e.g. xterm crops Sixel larger than its current geometry).
            if let Some((max_w, max_h)) = renderer.max_render_size() {
                area_width = area_width.min(max_w);
                area_height = area_height.min(max_h);
            }
            // Fitting (decode + resize) is the main CPU cost before rendering,
            // for any protocol; log the target so a slow render is diagnosable.
            debug!("fitting image to {area_width}x{area_height}px for render");
            let img = src.fitting(area_width, area_height, None)?;
            let id = renderer.print(w, &img, src_path, area, bg)?;
            if let Some(new_id) = id {
                // Kitty: tracked by id and erased via escape; not an inline image.
                self.rendered_images.push(RenderedImage {
                    image_id: new_id,
                    drawing_count,
                });
            } else {
                // Sixel: drawn into the cells, no id. Record it so reclear
                // detection knows what's on screen this frame.
                self.note_inline(src_path, area);
            }
            // An image was drawn (Kitty with an id, or Sixel with none); either
            // way the caller must not draw the text fallback over it.
            Ok(ImageRendering::Drawn(id))
        } else {
            Ok(ImageRendering::Unsupported)
        }
    }
    /// Start a draw pass: forget the inline (Sixel) images noted in the previous
    /// pass of this frame. Call before each pass over the panels.
    pub fn start_pass(&mut self) {
        self.frame_sixels.clear();
    }
    /// Record that an inline (Sixel) image for `path` occupies `area` this pass
    /// (drawn or kept in place). Drives `reclear_needed`.
    pub fn note_inline(&mut self, path: &Path, area: &Area) {
        self.frame_sixels.push((path.to_path_buf(), area.clone()));
    }
    /// Whether a full-screen clear + redraw is needed before this frame is
    /// final: an inline image shown last frame is no longer shown identically
    /// (changed, left, or moved), on a terminal that keeps Sixel until the
    /// screen is cleared (Konsole). False elsewhere — repainting cells suffices.
    pub fn reclear_needed(&self) -> bool {
        let renderer_needs = match &self.renderer {
            MaybeRenderer::Enabled { renderer } => renderer.needs_reclear_on_change(),
            _ => false,
        };
        renderer_needs
            && self
                .prev_sixels
                .iter()
                .any(|s| !self.frame_sixels.contains(s))
    }
    /// Whether the current pass is a post-clear redraw, in which an image that
    /// would normally be kept in place must instead be repainted (the clear
    /// removed it). Read by `ImageView` when deciding `must_draw`.
    pub fn forced_redraw(&self) -> bool {
        self.forced_redraw
    }
    pub fn set_forced_redraw(&mut self, on: bool) {
        self.forced_redraw = on;
    }
    /// Remember this frame's inline images for next-frame comparison. Call once
    /// after the final pass.
    pub fn commit_frame(&mut self) {
        self.prev_sixels = std::mem::take(&mut self.frame_sixels);
    }
    pub fn erase_images_before(
        &mut self,
        w: &mut W,
        drawing_count: usize,
    ) -> Result<(), ProgramError> {
        let mut kept_images = Vec::new();
        let stale: Vec<RenderedImage> = self.rendered_images.drain(..).collect();
        for image in stale {
            if image.drawing_count >= drawing_count {
                kept_images.push(image);
            } else if let MaybeRenderer::Enabled { renderer } = &self.renderer {
                renderer.erase_image(w, image.image_id)?;
            }
        }
        self.rendered_images = kept_images;
        Ok(())
    }
}
