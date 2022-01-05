use {
    crate::{
        display::W,
        errors::ProgramError,
    },
    std::io::Write,
};

#[derive(Debug, Default)]
pub struct KittyImageSet {
    ids: Vec<usize>,
}

impl KittyImageSet {
    pub fn erase(
        self,
        w: &mut W,
    ) -> Result<(), ProgramError> {
        for id in &self.ids {
            debug!("erase kitty image {}", id);
            write!(w, "\u{1b}_Ga=d,d=I,i={}\u{1b}\\", id)?;
        }
        Ok(())
    }
    /// erase all kitty images, even the forgetted ones
    ///
    /// (this is currently unused)
    pub fn erase_all(
        w: &mut W,
    ) -> Result<(), ProgramError> {
        write!(w, "\u{1b}_Ga=d,d=A\u{1b}\\")?;
        Ok(())
    }
    pub fn push(&mut self, new_id: usize) {
        self.ids.push(new_id);
    }
}
