
/// compute a new selection index for the given list len,
/// taking into account whether we should cycle or not
pub fn move_sel(
    selection: usize,
    len: usize,
    d: i32, // displacement
    cycle: bool,
) -> usize {
    if len == 0 {
        return 0;
    }
    let ns = (selection as i32) + d;
    if ns < 0 {
        if cycle {
            len - 1
        } else {
            0
        }
    } else if ns >= len as i32 {
        if cycle {
            0
        } else {
            len - 1
        }
    } else {
        ns as usize
    }
}
