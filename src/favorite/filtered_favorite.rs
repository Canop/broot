use {
    super::*,
    crate::pattern::*,
    std::{
        convert::TryFrom,
        path::{Path, PathBuf},
    },
};

#[derive(Clone)]
pub struct FilteredFavorites {
    favorites_version: usize,
    paths_idx: Vec<usize>,
    pattern: InputPattern,
    selection: Option<usize>,
}

impl FilteredFavorites {
    pub fn unfiltered(favorites: &Favorites) -> Self {
        Self::filtered(favorites, InputPattern::none())
    }

    fn compute(&mut self, favorites: &Favorites) {
        if self.pattern.is_none() {
            self.paths_idx = favorites
                .paths()
                .iter()
                .enumerate()
                .map(|(idx, _)| idx)
                .collect();
            if !self.paths_idx.is_empty() && self.selection.is_none() {
                self.selection = Some(0);
            }
        } else {
            let mut best_score = None;
            self.paths_idx.clear();
            for (idx, path) in favorites.paths().iter().enumerate() {
                if let Some(file_name) = path.file_name() {
                    let subpath = path.to_string_lossy().to_string();
                    let name = file_name.to_string_lossy().to_string();
                    let candidate = Candidate {
                        path,
                        subpath: &subpath,
                        name: &name,
                    };
                    if let Some(score) = self.pattern.pattern.score_of(candidate) {
                        let is_best = match best_score {
                            Some(old_score) if old_score < score => true,
                            None => true,
                            _ => false,
                        };
                        if is_best {
                            self.selection = Some(self.paths_idx.len());
                            best_score = Some(score);
                        }
                        self.paths_idx.push(idx);
                    }
                }
            }
        }
    }

    pub fn filtered(favorites: &Favorites, pattern: InputPattern) -> Self {
        let mut ff = Self {
            favorites_version: favorites.version(),
            paths_idx: Vec::new(),
            pattern,
            selection: None,
        };
        ff.compute(favorites);
        ff
    }

    pub fn update(&mut self, favorites: &Favorites) -> bool {
        if favorites.version() == self.favorites_version {
            false
        } else {
            self.favorites_version = favorites.version();
            self.compute(favorites);
            true
        }
    }

    pub fn set_pattern(&mut self, favorites: &Favorites, pattern: InputPattern) {
        self.favorites_version = favorites.version();
        self.pattern = pattern;
        self.compute(favorites);
    }

    pub fn len(&self) -> usize {
        self.paths_idx.len()
    }

    pub fn path<'s>(&self, favorites: &'s Favorites, idx: usize) -> Option<&'s Path> {
        self.paths_idx
            .get(idx)
            .and_then(|&idx| favorites.paths().get(idx))
            .map(PathBuf::as_path)
    }

    pub fn path_sel<'s>(
        &self,
        favorites: &'s Favorites,
        idx: usize,
    ) -> Option<(&'s Path, bool)> {
        self.path(favorites, idx)
            .map(|p| (p, self.selection.map_or(false, |si| idx == si)))
    }

    pub fn pattern(&self) -> &InputPattern {
        &self.pattern
    }

    pub fn selection(&self) -> Option<usize> {
        self.selection
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    pub fn try_select_idx(&mut self, idx: usize) -> bool {
        if idx < self.paths_idx.len() {
            self.selection = Some(idx);
            true
        } else {
            false
        }
    }

    pub fn selected_path<'s>(&self, favorites: &'s Favorites) -> Option<&'s Path> {
        self.selection
            .and_then(|pi| self.paths_idx.get(pi))
            .and_then(|&idx| favorites.paths().get(idx))
            .map(|p| p.as_path())
    }

    pub fn unselect(&mut self) {
        self.selection = None;
    }

    /// unfavorite the selection, if any, or return false.
    pub fn unfavorite_selection(&mut self, favorites: &mut Favorites) -> bool {
        if let Some(spi) = self.selection {
            favorites.remove_idx(self.paths_idx[spi]);
            self.favorites_version = favorites.version();
            self.compute(favorites);
            if spi >= self.paths_idx.len() {
                self.selection = None;
            };
            true
        } else {
            false
        }
    }

    pub fn move_selection(&mut self, dy: i32, cycle: bool) {
        self.selection = if self.paths_idx.is_empty() {
            None
        } else if let Some(sel_idx) = self.selection.and_then(|i| i32::try_from(i).ok()) {
            let new_sel_idx = sel_idx + dy;
            Some(if new_sel_idx < 0 {
                if cycle && sel_idx == 0 {
                    self.paths_idx.len() - 1
                } else {
                    0
                }
            } else if new_sel_idx as usize >= self.paths_idx.len() {
                if cycle && sel_idx == self.paths_idx.len() as i32 - 1 {
                    0
                } else {
                    self.paths_idx.len() - 1
                }
            } else {
                new_sel_idx as usize
            })
        } else if dy < 0 {
            Some(self.paths_idx.len() - 1)
        } else {
            Some(0)
        };
    }
}
