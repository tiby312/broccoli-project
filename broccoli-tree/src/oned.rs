use crate::*;

///The results of the binning process.
pub struct Binned<'a, T: 'a> {
    pub middle: &'a mut [T],
    pub left: &'a mut [T],
    pub right: &'a mut [T],
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
