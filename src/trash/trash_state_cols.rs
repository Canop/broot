use {
    super::*,
    crate::{
        skin::StyleMap,
        tree::TreeOptions,
    },
    chrono::{
        Local,
        LocalResult,
        TimeZone,
    },
    termimad::CompoundStyle,
    trash::TrashItem,
    unicode_width::UnicodeWidthStr,
};

/// A displayable column, related to properties of the TrashItem
#[derive(Debug, Clone, Copy)]
pub enum TrashItemProperty {
    OriginalParent,
    Name,
    DeletionDate,
    Size,
}

impl TrashItemProperty {
    pub fn title(self) -> &'static str {
        // only single byte characters allowed here
        match self {
            Self::OriginalParent => "Original parent",
            Self::Name => "Deleted file name",
            Self::DeletionDate => "Deletion",
            Self::Size => "Size",
        }
    }
    pub fn style(
        self,
        is_dir: bool,
        styles: &StyleMap,
    ) -> &CompoundStyle {
        match self {
            Self::DeletionDate => &styles.dates,
            _ => {
                if is_dir {
                    &styles.directory
                } else {
                    &styles.file
                }
            }
        }
    }
    pub fn value_of(
        self,
        item: &TrashItem,
        options: &TreeOptions,
    ) -> String {
        match self {
            Self::OriginalParent => item.original_parent.to_string_lossy().to_string(),
            Self::Name => item.name.to_string_lossy().to_string(),
            Self::DeletionDate => {
                let seconds = item.time_deleted;
                if let LocalResult::Single(date_time) = Local.timestamp_opt(seconds, 0) {
                    date_time.format(options.date_time_format).to_string()
                } else {
                    "???".to_string()
                }
            }
            Self::Size => match item_unified_size(item) {
                Some(size) => format!("{:>4}", file_size::fit_4(size)),
                None => "????".to_string(),
            },
        }
    }
    pub fn const_width(self) -> bool {
        match self {
            Self::OriginalParent => false,
            Self::Name => false,
            Self::DeletionDate => true,
            Self::Size => true,
        }
    }
    pub fn optimal_width(
        self,
        items: &[TrashItem],
        options: &TreeOptions,
    ) -> usize {
        match self {
            Self::Size => 4,
            _ => items
                .iter()
                .map(|m| self.value_of(m, options).width())
                .max()
                .unwrap_or(0),
        }
    }
    pub fn column_constraints(
        self,
        items: &[TrashItem],
        options: &TreeOptions,
    ) -> flex_grow::Child<Self> {
        let optimal_width = self.optimal_width(items, options);
        let child = flex_grow::Child::new(self);
        if self.const_width() {
            child.with_size(optimal_width)
        } else {
            child.with_max(optimal_width)
        }
    }
}

pub fn get_cols(
    items: &[TrashItem],
    available_width: usize,
    tree_options: &TreeOptions,
) -> Vec<flex_grow::Child<TrashItemProperty>> {
    let mut cols_builder = flex_grow::Container::builder_in(available_width).with_margin_between(1);
    if tree_options.show_sizes {
        cols_builder.add(
            TrashItemProperty::Size
                .column_constraints(items, tree_options)
                .optional(),
        );
    }
    if tree_options.show_dates {
        cols_builder.add(
            TrashItemProperty::DeletionDate
                .column_constraints(items, tree_options)
                .optional(),
        );
    }
    cols_builder.add(
        TrashItemProperty::OriginalParent
            .column_constraints(items, tree_options)
            .with_min(10)
            .optional(),
    );
    cols_builder.add(
        TrashItemProperty::Name
            .column_constraints(items, tree_options)
            .with_min(10)
            .with_grow(2.0),
    );
    let Ok(cols) = cols_builder.build() else {
        return Vec::new(); // should not happen
    };
    debug!("trash_state cols: {:?}", cols.sizes());
    cols.to_children()
}
