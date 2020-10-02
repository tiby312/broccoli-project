
use super::*;
use collections::DinoTreeRef;

/// Collection of functions that panics if the dinotree result differs from the naive solution.
/// Should never panic unless invariants of the tree data struct have been violated.    
pub struct Assert;
impl Assert {
    ///Returns false if the tree's invariants are not met.
    #[must_use]
    pub fn tree_invariants<'a>(a:&impl Queries<'a>) -> bool {
        Self::inner(a.axis(), a.vistr().with_depth(compt::Depth(0))).is_ok()
    }

    fn inner<A: Axis, N: Node>(axis: A, iter: compt::LevelIter<Vistr<N>>) -> Result<(), ()> {
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
        let nn = nn.get();
        let axis_next = axis.next();

        let f = |a: &&N::T, b: &&N::T| -> Option<core::cmp::Ordering> {
            let j = a
                .get()
                .get_range(axis_next)
                .start
                .cmp(&b.get().get_range(axis_next).start);
            Some(j)
        };

        {
            use is_sorted::IsSorted;
            assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(), f));
        }

        if let Some([start, end]) = rest {
            match nn.div {
                Some(div) => {
                    match nn.cont {
                        Some(cont) => {
                            for bot in nn.bots.iter() {
                                assert2!(bot.get().get_range(axis).contains(*div));
                            }

                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).start),
                                *div
                            ));

                            for bot in nn.bots.iter() {
                                assert2!(cont.contains_range(bot.get().get_range(axis)));
                            }

                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).start),
                                cont.start
                            ));
                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).end),
                                cont.end
                            ));
                        }
                        None => assert2!(nn.bots.is_empty()),
                    }

                    Self::inner(axis_next, start)?;
                    Self::inner(axis_next, end)?;
                }
                None => {
                    for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                        let n = n.get();
                        assert2!(n.bots.is_empty());
                        assert2!(n.cont.is_none());
                        assert2!(n.div.is_none());
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn find_intersections_mut<A: Axis, T: Aabb + HasInner>(tree: &mut DinoTreeRef<A, T>) {
    
        let mut res_dino = Vec::new();
        tree.find_intersections_mut(|a, b| {
            let a = a as *const _ as usize;
            let b = b as *const _ as usize;
            let k = if a < b { (a, b) } else { (b, a) };
            res_dino.push(k);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_elements_mut()).find_intersections_mut(|a, b| {
            let a = a as *const _ as usize;
            let b = b as *const _ as usize;
            let k = if a < b { (a, b) } else { (b, a) };
            res_naive.push(k);
        });

        res_naive.sort();
        res_dino.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    

    
    pub fn k_nearest_mut<Acc, A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeRef<A, T>,
        point: Vec2<T::Num>,
        num: usize,
        acc: &mut Acc,
        mut broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        mut fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        rect: Rect<T::Num>,
    ) {
        let bots = tree.get_elements_mut();

        let mut res_naive = NaiveAlgs::new(bots)
            .k_nearest_mut(point, num, acc, &mut broad, &mut fine)
            .drain(..)
            .map(|a| (a.bot as *const _ as usize, a.mag))
            .collect::<Vec<_>>();

        let mut r = tree.k_nearest_mut(point, num, acc, broad, fine, rect);
        let mut res_dino: Vec<_> = r
            .drain(..)
            .map(|a| (a.bot as *const _ as usize, a.mag))
            .collect();

        res_naive.sort();
        res_dino.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    

    pub fn raycast_mut<Acc, A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeRef<A, T>,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        mut broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        mut fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        border: Rect<T::Num>,
    ) where
        <T as Aabb>::Num: core::fmt::Debug,
    {
        let bots = tree.get_elements_mut();

        let mut res_naive = Vec::new();
        match NaiveAlgs::new(bots).raycast_mut(ray, start, &mut broad, &mut fine, border) {
            RayCastResult::Hit((bots, mag)) => {
                for a in bots.iter() {
                    let j = (*a) as *const _ as usize;
                    res_naive.push((j, mag))
                }
            }
            RayCastResult::NoHit => {
                //do nothing
            }
        }

        let mut res_dino = Vec::new();
        match tree.raycast_mut(ray, start, broad, fine, border) {
            RayCastResult::Hit((bots, mag)) => {
                for a in bots.iter() {
                    let j = (*a) as *const _ as usize;
                    res_dino.push((j, mag))
                }
            }
            RayCastResult::NoHit => {
                //do nothing
            }
        }

        res_naive.sort();
        res_dino.sort();

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

    

    pub fn for_all_in_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeRef<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.for_all_in_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_elements_mut()).for_all_in_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    

    /// Panics if the result differs from the naive solution.
    /// Should never panic unless invariants of the tree data struct have been violated.
    pub fn for_all_not_in_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeRef<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.for_all_not_in_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_elements_mut()).for_all_not_in_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    

    pub fn for_all_intersect_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeRef<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.for_all_intersect_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_elements_mut()).for_all_intersect_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    
}
