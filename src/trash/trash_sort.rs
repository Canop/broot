use {
    super::*,
    crate::tree::*,
    trash::TrashItem,
};

/// Sort trash items according to the current tree options.
pub fn sort(
    items: &mut [TrashItem],
    tree_options: &TreeOptions,
) {
    info!("sorting itemsi by {:?}", tree_options.sort);
    match tree_options.sort {
        Sort::Date => items.sort_by_key(|item| std::cmp::Reverse(item.time_deleted)),
        Sort::Size => {
            items.sort_by_key(|item| std::cmp::Reverse(item_unified_size(item).unwrap_or(0)))
        }
        _ => items.sort_by_key(|item| (item.name.clone(), item.original_parent.clone())),
    }
}
