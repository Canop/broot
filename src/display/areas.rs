use {
    super::{
        Screen,
        WIDE_STATUS,
    },
    crate::{
        app::Panel,
    },
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
const MINIMAL_PANEL_WIDTH: u16 = 4;
const MINIMAL_SCREEN_WIDTH: u16 = 8;

enum Slot<'a> {
    Panel(usize),
    New(&'a mut Areas),
}

impl Areas {

    /// compute an area for a new panel which will be inserted
    pub fn create(
        present_panels: &mut [Panel],
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
        Self::compute_areas(present_panels, &mut slots, screen, with_preview);
        areas
    }

    pub fn resize_all(
        panels: &mut [Panel],
        screen: Screen,
        with_preview: bool, // slightly larger last panel
    ) {
        let mut slots = Vec::new();
        for i in 0..panels.len() {
            slots.push(Slot::Panel(i));
        }
        Self::compute_areas(panels, &mut slots, screen, with_preview)
    }

    fn compute_areas(
        panels: &mut [Panel],
        slots: &mut [Slot],
        screen: Screen,
        with_preview: bool, // slightly larger last panel
    ) {
        let screen_height = screen.height.max(MINIMAL_PANEL_HEIGHT);
        let screen_width = screen.width.max(MINIMAL_SCREEN_WIDTH);
        let n = slots.len() as u16;
        let mut panel_width = if with_preview {
            3 * screen_width / (3 * n + 1)
        } else {
            screen_width / n
        };
        if panel_width < MINIMAL_PANEL_WIDTH {
            panel_width = panel_width.max(MINIMAL_PANEL_WIDTH);
        }
        let mut x = 0;
        let nb_pos = slots.len();
        #[allow(clippy::needless_range_loop)]
        for slot_idx in 0..nb_pos {
            if slot_idx == nb_pos - 1 {
                panel_width = screen_width - x;
            }
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
                let area_width = panel_width / 2;
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
