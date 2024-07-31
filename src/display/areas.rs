use {
    super::*,
    crate::app::Panel,
    termimad::Area,
};

/// the areas of the various parts of a panel. It's
/// also where a state usually checks how many panels
/// there are, and their respective positions
#[derive(Debug, Clone)]
pub struct Areas {
    pub state: Area,
    pub status: Area,
    pub input: Area,
    pub purpose: Option<Area>,
    pub pos_idx: usize, // from left to right
    pub nb_pos: usize,  // number of displayed panels
}

const MINIMAL_PANEL_HEIGHT: u16 = 4;
const MINIMAL_PANEL_WIDTH: u16 = 8;
const MINIMAL_SCREEN_WIDTH: u16 = 16;

enum Slot<'a> {
    Panel(usize),
    New(&'a mut Areas),
}

impl Areas {
    /// compute an area for a new panel which will be inserted
    pub fn create(
        present_panels: &mut [Panel],
        layout_instructions: &LayoutInstructions,
        mut insertion_idx: usize,
        screen: Screen,
        with_preview: bool, // slightly larger last panel
    ) -> Self {
        if insertion_idx > present_panels.len() {
            insertion_idx = present_panels.len();
        }
        let mut areas = Areas {
            state: Area::uninitialized(),
            status: Area::uninitialized(),
            input: Area::uninitialized(),
            purpose: None,
            pos_idx: 0,
            nb_pos: 1,
        };
        let mut slots = Vec::with_capacity(present_panels.len() + 1);
        for i in 0..insertion_idx {
            slots.push(Slot::Panel(i));
        }
        slots.push(Slot::New(&mut areas));
        for i in insertion_idx..present_panels.len() {
            slots.push(Slot::Panel(i));
        }
        Self::compute_areas(
            present_panels,
            layout_instructions,
            &mut slots,
            screen,
            with_preview,
        );
        areas
    }

    pub fn resize_all(
        panels: &mut [Panel],
        layout_instructions: &LayoutInstructions,
        screen: Screen,
        with_preview: bool, // slightly larger last panel
    ) {
        let mut slots = Vec::new();
        for i in 0..panels.len() {
            slots.push(Slot::Panel(i));
        }
        Self::compute_areas(
            panels,
            layout_instructions,
            &mut slots,
            screen,
            with_preview,
        )
    }

    fn compute_areas(
        panels: &mut [Panel],
        layout_instructions: &LayoutInstructions,
        slots: &mut [Slot],
        screen: Screen,
        with_preview: bool, // slightly larger last panel
    ) {
        let screen_height = screen.height.max(MINIMAL_PANEL_HEIGHT);
        let screen_width = screen.width.max(MINIMAL_SCREEN_WIDTH);
        let n = slots.len() as u16;

        // compute auto/default panel widths
        let mut panel_width = if with_preview {
            3 * screen_width / (3 * n + 1)
        } else {
            screen_width / n
        };
        if panel_width < MINIMAL_PANEL_WIDTH {
            panel_width = panel_width.max(MINIMAL_PANEL_WIDTH);
        }
        let nb_pos = slots.len();
        let mut panel_widths = vec![panel_width; nb_pos];
        panel_widths[nb_pos - 1] = screen_width - (nb_pos as u16 - 1) * panel_width;

        // adjust panel widths with layout instructions
        if nb_pos > 1 {
            for instruction in &layout_instructions.instructions {
                debug!("Applying {:?}", instruction);
                debug!("panel_widths before: {:?}", &panel_widths);
                match *instruction {
                    LayoutInstruction::MoveDivider { divider, dx } => {
                        if divider + 1 >= nb_pos {
                            continue;
                        }
                        let (decr, incr, diff) = if dx < 0 {
                            (divider, divider + 1, (-dx) as u16)
                        } else {
                            (divider + 1, divider, dx as u16)
                        };
                        let diff = diff.min(panel_widths[decr] - MINIMAL_PANEL_WIDTH);
                        panel_widths[decr] -= diff;
                        panel_widths[incr] += diff;
                    }
                    LayoutInstruction::SetPanelWidth { panel, width } => {
                        if panel >= nb_pos { continue; }
                        let width = width.max(MINIMAL_PANEL_WIDTH);
                        if width > panel_widths[panel] {
                            let mut diff = width - panel_widths[panel];
                            // as we try to increase the width of 'panel' we have to decrease the
                            // widths of the other ones
                            while diff > 0 {
                                let mut freed = 0;
                                let step = diff / (nb_pos as u16 - 1);
                                for i in 0..nb_pos {
                                    if i != panel {
                                        let step = step.min(panel_widths[i] - MINIMAL_PANEL_WIDTH);
                                        panel_widths[i] -= step;
                                        freed += step;
                                    }
                                }
                                if freed == 0 { break; }
                                diff -= freed;
                                panel_widths[panel] += freed;
                            }
                        } else {
                            // we distribute the freed width among other panels
                            let freed = panel_widths[panel] - width;
                            panel_widths[panel] = width;
                            let step = freed / (nb_pos as u16 - 1);
                            for i in 0..nb_pos {
                                if i != panel {
                                    panel_widths[i] += step;
                                }
                            }
                            let rem = freed - (nb_pos as u16 - 1) * freed;
                            for i in 0..nb_pos {
                                if i != panel {
                                    panel_widths[i] += rem;
                                    break;
                                }
                            }
                        }
                    }
                }
                debug!("panel_widths after: {:?}", &panel_widths);
            }
        }

        // compute the areas of each slot, and give it to their panels
        let mut x = 0;
        #[allow(clippy::needless_range_loop)]
        for slot_idx in 0..nb_pos {
            let panel_width = panel_widths[slot_idx];
            let areas: &mut Areas = match &mut slots[slot_idx] {
                Slot::Panel(panel_idx) => &mut panels[*panel_idx].areas,
                Slot::New(areas) => areas,
            };
            let y = screen_height - 2;
            areas.state = Area::new(x, 0, panel_width, y);
            areas.status = if WIDE_STATUS {
                Area::new(0, y, screen_width, 1)
            } else {
                Area::new(x, y, panel_width, 1)
            };
            let y = y + 1;
            areas.input = Area::new(x, y, panel_width, 1);
            if slot_idx == nb_pos - 1 {
                // the char at the bottom right of the terminal should not be touched
                // (it makes some terminals flicker) so the input area is one char shorter
                areas.input.width -= 1;
            }
            areas.purpose = if slot_idx > 0 {
                // the purpose area is over the panel at left
                let area_width = panel_widths[slot_idx - 1] / 2;
                Some(Area::new(x - area_width, y, area_width, 1))
            } else {
                None
            };
            areas.pos_idx = slot_idx;
            areas.nb_pos = nb_pos;
            x += panel_width;
        }
    }

    pub fn is_first(&self) -> bool {
        self.pos_idx == 0
    }
    pub fn is_last(&self) -> bool {
        self.pos_idx + 1 == self.nb_pos
    }
}
