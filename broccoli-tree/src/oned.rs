use crate::*;

///The results of the binning process.
pub struct Binned<'a, T: 'a> {
    pub middle: &'a mut [T],
    pub left: &'a mut [T],
    pub right: &'a mut [T],
}

/// This is slow
#[allow(dead_code)]
#[must_use]
pub fn bin_middle_left_right_simple<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();
    use core::cmp::Ordering::*;

    let m = bots.partition_point(|x| x.get().get_range(axis).contains_ext(*med) == Equal);
    let (middle, rest) = bots.split_at_mut(m);

    let m = rest.partition_point(|x| x.get().get_range(axis).contains_ext(*med) == Less);
    let (left, right) = rest.split_at_mut(m);

    assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        middle,
        left,
        right,
    }
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile < left < right
#[must_use]
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
                //This is the least likely case, therefore
                //have it be the bin that requires the most swaps.

                bots.swap(index_at, left_end);
                bots.swap(left_end, middle_end);
                middle_end += 1;
                left_end += 1;
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Greater => {
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
    assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        middle,
        left,
        right,
    }
}

///
/// Make adding to middle as fast as possible because even though it is rare,
/// when it does happen, we are in a close to degenerate tree, therefore
/// to compensate for that we would want it to be faster?
///
/// note: this appear to be slower always
#[allow(dead_code)]
#[must_use]
pub fn bin_middle_left_right_fast_worst_case<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();

    let mut left_end = bot_len;
    let mut right_end = bot_len;

    for index_at in (0..bot_len).rev() {
        match bots[index_at].get().get_range(axis).contains_ext(*med) {
            //If the divider is less than the bot
            core::cmp::Ordering::Equal => {
                //do nothing
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Less => {
                left_end -= 1;
                bots.swap(index_at, left_end);
            }
            core::cmp::Ordering::Greater => {
                right_end -= 1;
                left_end -= 1;
                bots.swap(index_at, left_end);
                bots.swap(left_end, right_end);

                //right
            }
        }
    }

    let (rest, right) = bots.split_at_mut(right_end);
    let (middle, left) = rest.split_at_mut(left_end);
    assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        middle,
        left,
        right,
    }
}
