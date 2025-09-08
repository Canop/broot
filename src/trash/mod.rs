mod trash_sort;
mod trash_state;
mod trash_state_cols;

pub use trash_state::*;

use trash::{
    self as trash_crate,
    TrashItem,
    TrashItemSize,
};

/// Determine whether an item in the trash is a directory.
///
/// There's probably a simpler solution in the trash crate, but I didn't found it.
fn item_is_dir(item: &TrashItem) -> bool {
    match trash_crate::os_limited::metadata(item) {
        Ok(metadata) => match metadata.size {
            TrashItemSize::Bytes(_) => false,
            TrashItemSize::Entries(_) => true,
        },
        Err(_) => false,
    }
}

/// Return either the byte size or the number of entries of a trash item.
///
/// Return None when it couldn't be determined.
fn item_unified_size(item: &TrashItem) -> Option<u64> {
    match trash_crate::os_limited::metadata(item).ok()?.size {
        TrashItemSize::Bytes(v) => Some(v),
        TrashItemSize::Entries(v) => v.try_into().ok(),
    }
}
