use crate::inner_prelude::*;
use crate::Aabb;

///The results of the binning process.
pub struct Binned<'a, T: 'a> {
    pub middle: &'a mut [T],
    pub left: &'a mut [T],
    pub right: &'a mut [T],
}

#[inline(always)]
unsafe fn swap_unchecked<T>(arr: &mut [T], a: usize, b: usize) {
    core::ptr::swap(&mut arr[a] as *mut _, &mut arr[b] as *mut _);
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile < left < right
pub fn bin_middle_left_right<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();

    let mut left_end = 0;
    let mut middle_end = 0;

    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len {
        match bots[index_at].get().get_range(axis).contains_ext(*med) {
            //If the divider is less than the bot
            core::cmp::Ordering::Equal => {
                //left

                bots.swap(index_at, left_end);
                bots.swap(left_end, middle_end);
                middle_end += 1;
                left_end += 1;
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Greater => {
                //middile
                bots.swap(index_at, left_end);
                left_end += 1;
            }
            core::cmp::Ordering::Less => {
                //right
            }
        }
    }

    let (rest, right) = bots.split_at_mut(left_end);
    let (middle, left) = rest.split_at_mut(middle_end);
    //println!("middile left right={:?}",(middle.len(),left.len(),right.len()));
    debug_assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        left,
        middle,
        right,
    }
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile < left < right
pub(crate) unsafe fn bin_middle_left_right_unchecked<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();

    let mut left_end = 0;
    let mut middle_end = 0;

    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len {
        match bots
            .get_unchecked(index_at)
            .get()
            .get_range(axis)
            .contains_ext(*med)
        {
            //If the divider is less than the bot
            core::cmp::Ordering::Equal => {
                //left
                swap_unchecked(bots, index_at, left_end);
                swap_unchecked(bots, left_end, middle_end);
                middle_end += 1;
                left_end += 1;
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Greater => {
                //middile
                swap_unchecked(bots, index_at, left_end);
                left_end += 1;
            }
            core::cmp::Ordering::Less => {
                //right
            }
        }
    }

    let (rest, right) = bots.split_at_mut(left_end);
    let (middle, left) = rest.split_at_mut(middle_end);
    //println!("middile left right={:?}",(middle.len(),left.len(),right.len()));
    debug_assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        left,
        middle,
        right,
    }
}

#[inline(always)]
pub fn compare_bots<T: Aabb>(axis: impl Axis, a: &T, b: &T) -> core::cmp::Ordering {
    let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
    if p1 > p2 {
        core::cmp::Ordering::Greater
    } else {
        core::cmp::Ordering::Less
    }
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering { compare_bots(axis, a, b) };

    collision_botids.sort_unstable_by(sclosure);
}
