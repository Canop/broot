use {
    lazy_regex::*,
    std::str::FromStr,
};

#[derive(Debug, Clone, Default)]
pub struct LayoutInstructions {
    pub instructions: Vec<LayoutInstruction>,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutInstruction {
    Clear, // clear all instructions
    MoveDivider { divider: usize, dx: i16 },
    SetPanelWidth { panel: usize, width: u16 },
}

/// arguments for moving a divider, read from a string eg "0 -5"
/// (move the first divider 5 cells to the left)
#[derive(Debug, Clone, Copy)]
pub struct MoveDividerArgs {
    pub divider: usize,
    pub dx: i16,
}
impl FromStr for MoveDividerArgs {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((_, divider, dx)) = regex_captures!(r"^\s*(\d)\s+(-?\d{1,3})\s*$", s) {
            Ok(Self {
                divider: divider.parse().unwrap(),
                dx: dx.parse().unwrap(),
            })
        } else {
            Err("not the expected move_divider args")
        }
    }
}

/// arguments for setting the width of a panel, read from a string eg "1 150"
#[derive(Debug, Clone, Copy)]
pub struct SetPanelWidthArgs {
    pub panel: usize,
    pub width: u16,
}
impl FromStr for SetPanelWidthArgs {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((_, panel, width)) = regex_captures!(r"^\s*(\d)\s+(\d{1,4})\s*$", s) {
            Ok(Self {
                panel: panel.parse().unwrap(),
                width: width.parse().unwrap(),
            })
        } else {
            Err("not the expected set_panel_width args")
        }
    }
}

impl LayoutInstruction {
    pub fn is_moving_divider(
        self,
        idx: usize,
    ) -> bool {
        match self {
            Self::MoveDivider { divider, .. } => divider == idx,
            _ => false,
        }
    }
}

impl LayoutInstructions {
    pub fn add(
        &mut self,
        new_instruction: LayoutInstruction,
    ) {
        use LayoutInstruction::*;
        match new_instruction {
            Clear => {
                self.instructions.clear();
            }
            SetPanelWidth {
                panel: new_panel, ..
            } => {
                // all previous SetPanelWidth for the same panel are now irrelevant
                self.instructions.retain(|i| match i {
                    SetPanelWidth { panel, .. } => *panel != new_panel,
                    _ => true,
                });
            }
            MoveDivider {
                divider: new_divider,
                dx: new_dx,
            } => {
                // if the last instruction is a move of the same divider, we adjust it
                if let Some(MoveDivider { divider, dx }) = self.instructions.last_mut() {
                    if *divider == new_divider {
                        *dx += new_dx;
                        return;
                    }
                }
            }
        }
        self.instructions.push(new_instruction);
    }
}
