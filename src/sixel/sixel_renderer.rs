use {
    crate::{
        display::{W, cell_size_in_pixels},
        errors::ProgramError,
        graphics::{
            GraphicsRenderer, ImageId,
            rendering_area,
            image_data::ImageData,
            terminal::{get_esc_seq, get_tmux_header, get_tmux_nest_count, get_tmux_tail, is_tmux},
        },
        image::zune_compat::DynamicImage,
        sixel::detect_support::detect_sixel_geometry,
    },
    cli_log::*,
    crokey::crossterm::{QueueableCommand, cursor, style::Color},
    icy_sixel::SixelImage,
    std::{io::Write, path::Path},
    termimad::{Area, coolor, fill_bg},
};

/// Encode raw RGBA bytes into a Sixel DCS sequence string.
/// Note: icy_sixel 0.5 takes `usize` for width/height (not u32).
pub(crate) fn encode_sixel(rgba: Vec<u8>, width: u32, height: u32) -> Result<String, ProgramError> {
    SixelImage::from_rgba(rgba, width as usize, height as usize)
        .encode()
        .map_err(|e| ProgramError::ImageError { details: format!("sixel encode failed: {e}") })
}

/// Wrap a raw terminal sequence for tmux passthrough at the given nesting level.
///
/// tmux scans the passthrough payload for the `ESC \` terminator, so every ESC
/// byte inside `seq` must be repeated `2^nest_count` times (`get_esc_seq`),
/// otherwise the Sixel's own start (`ESC P`) / end (`ESC \`) bytes would corrupt
/// or prematurely terminate the passthrough. This mirrors what the Kitty path
/// does by building its sequences with `get_esc_seq` inline.
fn tmux_passthrough(seq: &str, nest_count: u32) -> String {
    format!(
        "{}{}{}",
        get_tmux_header(nest_count),
        seq.replace('\u{1b}', &get_esc_seq(nest_count)),
        get_tmux_tail(nest_count),
    )
}

/// Resolve a (possibly `Reset`) colour to concrete RGB. `Reset` means the
/// terminal's default background, supplied as `terminal_bg`.
///
/// Exactness caveat: this matches what `fill_bg` draws only for `Reset` (we
/// queried the terminal's actual default) and for true `Rgb` colours. For named
/// or indexed ANSI colours we use coolor's standard-palette RGB, which can
/// differ from a terminal's customized palette — so under a custom theme a
/// `pad` band from such a colour may be faintly visible against the letterbox.
fn resolve_bg(bg: Color, terminal_bg: coolor::Rgb) -> coolor::Rgb {
    match bg {
        Color::Reset => terminal_bg,
        other => coolor::Color::from(other).rgb(),
    }
}

#[derive(Debug)]
pub struct SixelRenderer {
    cell_width: u32,
    cell_height: u32,
    is_tmux: bool,
    /// Terminal's current Sixel graphics geometry in pixels (XTSMGRAPHICS), if
    /// reported. Images larger than this are cropped by some terminals (e.g.
    /// xterm), so we fit within it. Snapshot taken at construction; see
    /// `detect_sixel_geometry` for why it isn't refreshed on resize.
    current_geometry: Option<(u32, u32)>,
    /// Konsole keeps Sixel until the screen is cleared, so a changed/removed
    /// image can't be erased by repainting cells; it reports
    /// `needs_reclear_on_change()` so the manager issues a full clear + redraw.
    /// Other terminals drop Sixel when the cells are overwritten (false here).
    is_konsole: bool,
    /// Terminal default background as RGB, queried once at startup (Konsole
    /// only). Used to fill the Sixel band-padding rows when the skin bg is
    /// `Reset` so the padding matches the letterbox.
    terminal_bg: coolor::Rgb,
    /// Last encoded Sixel `(path, fitted_width, fitted_height, dcs)`, reused
    /// when the same image at the same size is drawn again within a frame (the
    /// post-clear redraw pass on Konsole) so it isn't re-encoded. Cleared every
    /// frame (`end_frame`) so a file that changed in place isn't shown stale.
    last_encoded: Option<(std::path::PathBuf, u32, u32, String)>,
}

impl SixelRenderer {
    /// Build a renderer if cell size is available. Caller has already
    /// confirmed Sixel support via detect_sixel_support().
    pub fn new() -> Option<Self> {
        let (cell_width, cell_height) = match cell_size_in_pixels() {
            Ok(dims) => dims,
            Err(e) => {
                debug!("sixel disabled: couldn't get cell size in pixels: {e}");
                return None;
            }
        };
        let current_geometry = detect_sixel_geometry();
        debug!("sixel current geometry: {current_geometry:?}");
        let is_konsole = std::env::var("KONSOLE_VERSION").is_ok();
        debug!("sixel is_konsole={is_konsole}");
        let terminal_bg = if is_konsole {
            terminal_light::background_color()
                .map(|c| c.rgb())
                .unwrap_or_else(|e| {
                    // Black is a poor guess on a light theme; log so a wrongly
                    // coloured pad band on `Reset` skins is diagnosable.
                    debug!("sixel: terminal bg query failed ({e}); padding falls back to black");
                    coolor::Rgb::new(0, 0, 0)
                })
        } else {
            coolor::Rgb::new(0, 0, 0)
        };
        Some(Self {
            cell_width,
            cell_height,
            is_tmux: is_tmux(),
            current_geometry,
            is_konsole,
            terminal_bg,
            last_encoded: None,
        })
    }
}

impl GraphicsRenderer for SixelRenderer {
    fn print(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
    ) -> Result<Option<ImageId>, ProgramError> {
        // Clear the area's cells. On most terminals overwriting cells also drops
        // any prior Sixel there; Konsole keeps it until the screen is cleared,
        // handled separately by the manager's reclear (ESC[2J + full redraw).
        for y in area.top..area.top + area.height {
            w.queue(cursor::MoveTo(area.left, y))?;
            fill_bg(w, area.width as usize, bg)?;
        }

        let (img_width, img_height) = src.dimensions();
        let sub = rendering_area(self.cell_width, self.cell_height, img_width, img_height, area);

        // Reuse the cached encode when the same image at the same size is drawn
        // again (the post-clear redraw pass on Konsole), else encode and cache.
        let cached = self.last_encoded.as_ref().is_some_and(|(p, cw, ch, _)| {
            p == src_path && *cw == img_width && *ch == img_height
        });
        if !cached {
            let data = ImageData::from(src);
            let sixel = encode_sixel(data.rgba_bytes(), img_width, img_height)?;
            self.last_encoded = Some((src_path.to_path_buf(), img_width, img_height, sixel));
        }
        let sixel = &self.last_encoded.as_ref().unwrap().3;

        w.queue(cursor::MoveTo(sub.left, sub.top))?;
        if self.is_tmux {
            let n = get_tmux_nest_count();
            write!(w, "{}", tmux_passthrough(sixel, n))?;
        } else {
            write!(w, "{sixel}")?;
        }
        // No flush here: display_panels flushes once at end of frame, so a
        // Konsole reclear (clear + redraw) stays atomic and doesn't flicker.
        debug!(
            "rendered {img_width}x{img_height}px sixel at {sub:?} ({})",
            if cached { "reused encode" } else { "encoded" },
        );
        Ok(None)
    }

    fn erase_image(&self, _w: &mut W, _id: ImageId) -> Result<(), ProgramError> {
        // Sixel images live in the cells; cleared by normal repaint. No-op.
        Ok(())
    }

    fn needs_reclear_on_change(&self) -> bool {
        // Konsole keeps Sixel until the screen is cleared, so the manager must
        // issue ESC[2J + a full redraw when an on-screen image changes/leaves.
        self.is_konsole
    }

    fn fit_constraints(&self, bg: Color) -> crate::image::FitConstraints {
        if self.is_konsole {
            crate::image::FitConstraints {
                height_multiple: 6,
                pad: Some(resolve_bg(bg, self.terminal_bg)),
            }
        } else {
            crate::image::FitConstraints::default()
        }
    }

    fn end_frame(&mut self) {
        // Drop the within-frame encode cache so a file changed in place between
        // frames is re-encoded rather than shown from stale bytes.
        self.last_encoded = None;
    }

    fn cell_size(&self) -> (u32, u32) {
        (self.cell_width, self.cell_height)
    }

    fn max_render_size(&self) -> Option<(u32, u32)> {
        self.current_geometry
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_sixel, resolve_bg, tmux_passthrough};
    use termimad::coolor;
    use crokey::crossterm::style::Color;

    #[test]
    fn tmux_passthrough_doubles_inner_esc() {
        // a minimal Sixel-like DCS with 2 ESC bytes: ESC P ... ESC \
        let seq = "\x1bP0;0;0q#0~\x1b\\";
        let header = "\x1bPtmux;";
        let tail = "\x1b\\";
        let wrapped = tmux_passthrough(seq, 1);
        assert!(wrapped.starts_with(header), "header: {wrapped:?}");
        assert!(wrapped.ends_with(tail), "tail: {wrapped:?}");
        // every ESC in the payload must be doubled so tmux won't terminate early
        let payload = &wrapped[header.len()..wrapped.len() - tail.len()];
        assert_eq!(
            payload.matches('\x1b').count(),
            4, // 2 ESCs in `seq`, each doubled at nest level 1
            "inner ESCs should be doubled: {payload:?}"
        );
        assert!(payload.contains("\x1b\x1bP"), "doubled DCS start: {payload:?}");
        assert!(payload.contains("\x1b\x1b\\"), "doubled inner ST: {payload:?}");
    }

    #[test]
    fn resolve_bg_reset_uses_terminal_bg() {
        let term = coolor::Rgb::new(30, 40, 50);
        let got = resolve_bg(Color::Reset, term);
        assert_eq!((got.r, got.g, got.b), (30, 40, 50));
    }

    #[test]
    fn resolve_bg_concrete_color_ignores_terminal_bg() {
        let term = coolor::Rgb::new(0, 0, 0);
        let got = resolve_bg(Color::Rgb { r: 1, g: 2, b: 3 }, term);
        assert_eq!((got.r, got.g, got.b), (1, 2, 3));
    }

    #[test]
    fn encodes_small_image_to_dcs_sequence() {
        // 2x2 RGBA: red, green, blue, white
        let rgba = vec![
            255, 0, 0, 255,   0, 255, 0, 255,
            0, 0, 255, 255,   255, 255, 255, 255,
        ];
        let s = encode_sixel(rgba, 2, 2).unwrap();
        assert!(s.starts_with('\u{1b}'), "should start with ESC (DCS): {s:?}");
        assert!(s.contains('P'), "DCS introducer P expected");
        assert!(s.ends_with("\u{1b}\\"), "should end with ST: {s:?}");
    }
}
