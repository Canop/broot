use {
    super::{
        Selection,
        SelectionType,
    },
    crate::{
        stage::Stage,
        verb::FileTypeCondition,
    },
    std::path::Path,
};

/// Information regarding a potentially multiple set of selected paths
#[derive(Debug, Clone, Copy)]
pub enum SelInfo<'s> {
    None,
    One(Selection<'s>),
    More(&'s Stage), // by contract the stage contains at least 2 paths
}

impl<'a> SelInfo<'a> {
    pub fn to_selections(&self) -> Vec<Selection<'a>> {
        match self {
            SelInfo::None => Vec::new(),
            SelInfo::One(sel) => vec![*sel],
            SelInfo::More(stage) => stage
                .paths()
                .iter()
                .map(|path| Selection {
                    path,
                    line: 0,
                    stype: SelectionType::from(path),
                    is_exe: false, // OK, I don't know
                })
                .collect(),
        }
    }
    #[must_use]
    pub fn from_path(path: &'a Path) -> Self {
        Self::One(Selection {
            stype: SelectionType::from(path),
            line: 0,
            path,
            is_exe: false, // OK, I don't know
        })
    }
    #[must_use]
    pub fn count_paths(&self) -> usize {
        match self {
            SelInfo::None => 0,
            SelInfo::One(_) => 1,
            SelInfo::More(stage) => stage.len(),
        }
    }
    #[must_use]
    pub fn is_accepted_by(
        &self,
        condition: FileTypeCondition,
    ) -> bool {
        match self {
            SelInfo::None => true,
            SelInfo::One(sel) => condition.accepts_path(sel.path),
            SelInfo::More(stage) => {
                for path in stage.paths() {
                    if !condition.accepts_path(path) {
                        return false;
                    }
                }
                true
            }
        }
    }
    #[must_use]
    pub fn common_stype(&self) -> Option<SelectionType> {
        match self {
            SelInfo::None => None,
            SelInfo::One(sel) => Some(sel.stype),
            SelInfo::More(stage) => {
                let stype = SelectionType::from(&stage.paths()[0]);
                for path in stage.paths().iter().skip(1) {
                    if stype != SelectionType::from(path) {
                        return None;
                    }
                }
                Some(stype)
            }
        }
    }
    #[must_use]
    pub fn one_sel(self) -> Option<Selection<'a>> {
        match self {
            SelInfo::One(sel) => Some(sel),
            _ => None,
        }
    }
    #[must_use]
    pub fn first_sel(self) -> Option<Selection<'a>> {
        match self {
            SelInfo::One(sel) => Some(sel),
            SelInfo::More(stage) => stage.paths().first().map(|path| Selection {
                path,
                line: 0,
                stype: SelectionType::from(path),
                is_exe: false,
            }),
            _ => None,
        }
    }
    #[must_use]
    pub fn one_path(self) -> Option<&'a Path> {
        self.one_sel().map(|sel| sel.path)
    }
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        match self {
            SelInfo::None => None,
            SelInfo::One(sel) => sel.path.extension().and_then(|e| e.to_str()),
            SelInfo::More(stage) => {
                let common_extension = stage.paths()[0].extension().and_then(|e| e.to_str());
                #[allow(clippy::question_mark)]
                if common_extension.is_none() {
                    return None;
                }
                for path in stage.paths().iter().skip(1) {
                    let extension = path.extension().and_then(|e| e.to_str());
                    if extension != common_extension {
                        return None;
                    }
                }
                common_extension
            }
        }
    }
}
