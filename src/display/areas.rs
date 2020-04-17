
use {
    crate::{
        app::{
            Panel,
        },
        errors::ProgramError,
    },
    super::{
        Screen,
    },
    termimad::Area,
};


#[derive(Debug, Clone)]
pub struct Areas {
    pub state: Area,
    pub status: Area,
    pub input: Area,
    pub flags: Option<Area>,
}

const MINIMAL_PANEL_HEIGHT: u16 = 20;
const MINIMAL_PANEL_WIDTH: u16 = 30;

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
            flags: None,
        };
        let mut slots = Vec::with_capacity(present_panels.len()+1);
        for i in 0..insertion_idx {
            slots.push(Slot::Panel(i));
        }
        slots.push(Slot::New(&mut areas));
        for i in insertion_idx+1..present_panels.len() {
            slots.push(Slot::Panel(i));
        }
        Self::compute_areas(present_panels, &mut slots, screen)?;
        Ok(areas)
    }

    pub fn resize_all(
        panels: &mut [Panel],
        screen: &Screen,
    ) -> Result<(), ProgramError> {
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
        for slot_idx in 0..slots.len() {
            let areas: &mut Areas = match &mut slots[slot_idx] {
                Slot::Panel(panel_idx) => &mut panels[*panel_idx].areas,
                Slot::New(areas) => areas,
            };
            areas.state = Area::new(x, 0, panel_width, screen.height - 2);
            x += panel_width;
        }
        Ok(())
    }
}
