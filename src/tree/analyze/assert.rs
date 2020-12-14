//! Functions that panic if a disconnect between query results is detected
//! between `broccoli::Tree` and the naive equivalent.
//!
//!

use super::*;
use crate::tree::*;
use container::TreeRef;
///Returns false if the tree's invariants are not met.
#[must_use]
pub fn tree_invariants<'a, T: Queries<'a>>(a: &T) -> bool {
    inner(a.axis(), a.vistr().with_depth(compt::Depth(0))).is_ok()
}

fn inner<A: Axis, T: Aabb>(axis: A, iter: compt::LevelIter<Vistr<NodeMut<T>>>) -> Result<(), ()> {
    fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
        for b in it {
            if b == val {
                return true;
            }
        }
        false
    }

    macro_rules! assert2 {
        ($bla:expr) => {
            if !$bla {
                return Err(());
            }
        };
    }

    let ((_depth, nn), rest) = iter.next();
    //let nn = nn.get();
    let axis_next = axis.next();

    let f = |a: &&T, b: &&T| -> Option<core::cmp::Ordering> {
        let j = a
            .get()
            .get_range(axis_next)
            .start
            .partial_cmp(&b.get().get_range(axis_next).start)
            .unwrap();
        Some(j)
    };

    {
        use is_sorted::IsSorted;
        assert2!(IsSorted::is_sorted_by(&mut nn.range.iter(), f));
    }

    if let Some([start, end]) = rest {
        match nn.div {
            Some(div) => {
                match nn.cont {
                    Some(cont) => {
                        for bot in nn.range.iter() {
                            assert2!(bot.get().get_range(axis).contains(div));
                        }

                        assert2!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            div
                        ));

                        for bot in nn.range.iter() {
                            assert2!(cont.contains_range(bot.get().get_range(axis)));
                        }

                        assert2!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            cont.start
                        ));
                        assert2!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).end),
                            cont.end
                        ));
                    }
                    None => assert2!(nn.range.is_empty()),
                }

                inner(axis_next, start)?;
                inner(axis_next, end)?;
            }
            None => {
                for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                    assert2!(n.range.is_empty());
                    assert2!(n.cont.is_none());
                    assert2!(n.div.is_none());
                }
            }
        }
    }
    Ok(())
}

use core::ops::Deref;
fn into_ptr_usize<T>(a: &T) -> usize {
    a as *const T as usize
}
pub fn find_colliding_pairs_mut<A: Axis, T: Aabb>(tree: &mut TreeRef<A, T>) {
    let mut res_dino = Vec::new();
    tree.find_colliding_pairs_mut(|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_dino.push(k);
    });

    let mut res_naive = Vec::new();
    NaiveAlgs::new(tree.get_bbox_elements_mut()).find_colliding_pairs_mut(|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_naive.push(k);
    });

    res_naive.sort();
    res_dino.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn k_nearest_mut<Acc, A: Axis, T: Aabb>(
    tree: &mut TreeRef<A, T>,
    point: Vec2<T::Num>,
    num: usize,
    acc: &mut Acc,
    mut broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
    mut fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
    rect: Rect<T::Num>,
) {
    let bots = tree.get_bbox_elements_mut();

    let mut res_naive = NaiveAlgs::new(bots)
        .k_nearest_mut(point, num, acc, &mut broad, &mut fine)
        .into_vec()
        .drain(..)
        .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
        .collect::<Vec<_>>();

    let r = tree.k_nearest_mut(point, num, acc, broad, fine, rect);
    let mut res_dino: Vec<_> = r
        .into_vec()
        .drain(..)
        .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
        .collect();

    res_naive.sort_by(|a, b| a.partial_cmp(b).unwrap());
    res_dino.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn raycast_mut<Acc, A: Axis, T: Aabb>(
    tree: &mut TreeRef<A, T>,
    ray: axgeom::Ray<T::Num>,
    start: &mut Acc,
    mut broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
    mut fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
    border: Rect<T::Num>,
) where
    <T as Aabb>::Num: core::fmt::Debug,
{
    let bots = tree.get_bbox_elements_mut();

    let mut res_naive = Vec::new();
    match NaiveAlgs::new(bots).raycast_mut(ray, start, &mut broad, &mut fine, border) {
        axgeom::CastResult::Hit((bots, mag)) => {
            for a in bots.iter() {
                let j = into_ptr_usize(a);
                res_naive.push((j, mag))
            }
        }
        axgeom::CastResult::NoHit => {
            //do nothing
        }
    }

    let mut res_dino = Vec::new();
    match tree.raycast_mut(ray, start, broad, fine, border) {
        axgeom::CastResult::Hit((bots, mag)) => {
            for a in bots.iter() {
                let j = into_ptr_usize(a);
                res_dino.push((j, mag))
            }
        }
        axgeom::CastResult::NoHit => {
            //do nothing
        }
    }

    res_naive.sort_by(|a, b| a.partial_cmp(b).unwrap());
    res_dino.sort_by(|a, b| a.partial_cmp(b).unwrap());

    //dbg!("{:?}  {:?}",res_naive.len(),res_dino.len());
    assert_eq!(
        res_naive.len(),
        res_dino.len(),
        "len:{:?}",
        (res_naive, res_dino)
    );
    assert!(
        res_naive.iter().eq(res_dino.iter()),
        "nop:{:?}",
        (res_naive, res_dino)
    );
}

pub fn for_all_in_rect_mut<A: Axis, T: Aabb>(
    tree: &mut TreeRef<A, T>,
    rect: &axgeom::Rect<T::Num>,
) {
    let mut res_dino = Vec::new();
    tree.for_all_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    NaiveAlgs::new(tree.get_bbox_elements_mut()).for_all_in_rect_mut(rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn for_all_not_in_rect_mut<A: Axis, T: Aabb>(
    tree: &mut TreeRef<A, T>,
    rect: &axgeom::Rect<T::Num>,
) {
    let mut res_dino = Vec::new();
    tree.for_all_not_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    NaiveAlgs::new(tree.get_bbox_elements_mut()).for_all_not_in_rect_mut(rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn for_all_intersect_rect_mut<A: Axis, T: Aabb>(
    tree: &mut TreeRef<A, T>,
    rect: &axgeom::Rect<T::Num>,
) {
    let mut res_dino = Vec::new();
    tree.for_all_intersect_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    NaiveAlgs::new(tree.get_bbox_elements_mut()).for_all_intersect_rect_mut(rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}
