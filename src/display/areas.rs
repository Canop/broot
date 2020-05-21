use {
    super::Screen,
    crate::{
        app::Panel,
        errors::ProgramError,
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
    pub nb_pos: usize, // number of displayed panels
}

const MINIMAL_PANEL_HEIGHT: u16 = 10;
const MINIMAL_PANEL_WIDTH: u16 = 20;

enum Slot<'a> {
    Panel(usize),
    New(&'a mut Areas),
}

impl Areas {

    /// compute an area for a new panel which will be inserted
    pub fn create(
        present_panels: &mut [Panel],
        mut insertion_idx: usize,
        screen: &Screen,
    ) -> Result<Self, ProgramError> {
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
        Self::compute_areas(present_panels, &mut slots, screen)?;
        Ok(areas)
    }

    pub fn resize_all(panels: &mut [Panel], screen: &Screen) -> Result<(), ProgramError> {
        let mut slots = Vec::new();
        for i in 0..panels.len() {
            slots.push(Slot::Panel(i));
        }
        Self::compute_areas(panels, &mut slots, screen)
    }

    fn compute_areas(
        panels: &mut [Panel],
        slots: &mut Vec<Slot>,
        screen: &Screen,
    ) -> Result<(), ProgramError> {
        if screen.height < MINIMAL_PANEL_HEIGHT {
            return Err(ProgramError::TerminalTooSmallError);
        }
        let panel_width = screen.width / slots.len() as u16;
        if panel_width < MINIMAL_PANEL_WIDTH {
            return Err(ProgramError::TerminalTooSmallError);
        }
        let mut x = 0;
        let nb_pos = slots.len();
        for slot_idx in 0..nb_pos {
            let areas: &mut Areas = match &mut slots[slot_idx] {
                Slot::Panel(panel_idx) => &mut panels[*panel_idx].areas,
                Slot::New(areas) => areas,
            };
            let y = screen.height - 2;
            areas.state = Area::new(x, 0, panel_width, y);
            areas.status = Area::new(x, y, panel_width, 1);
            let y = y + 1;
            areas.input = Area::new(x, y, panel_width, 1);
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
        Ok(())
    }

    pub fn is_first(&self) -> bool {
        self.pos_idx == 0
    }
    pub fn is_last(&self) -> bool {
        self.pos_idx + 1 == self.nb_pos
    }
}
