/// Functions that panic if a disconnect between query results is detected
/// between `broccoli::Tree` and the naive equivalent.
pub trait NaiveCheck<'a,K>:core::ops::DerefMut<Target=K> where K:Queries<'a,A=Self::A,Num=Self::Num,T=Self::T>{
    type Num:Num;
    type T: Aabb<Num = Self::Num> + 'a;
    type A:Axis;

    fn get_underlying_slice_mut(&mut self)->PMut<[Self::T]>;

    fn assert_for_all_not_in_rect_mut(
        &mut self,
        rect: &axgeom::Rect<Self::Num>,
    ) {
        let mut res_dino = Vec::new();
        self.for_all_not_in_rect_mut(rect, |a| {
            res_dino.push(into_ptr_usize(a.deref()));
        });
    
        let mut res_naive = Vec::new();
        NaiveAlgs::new(self.get_underlying_slice_mut()).for_all_not_in_rect_mut(rect, |a| {
            res_naive.push(into_ptr_usize(a.deref()));
        });
    
        res_dino.sort();
        res_naive.sort();
    
        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
    
    fn assert_for_all_intersect_rect_mut(
        &mut self,
        rect: &axgeom::Rect<Self::Num>,
    ) {
        let mut res_dino = Vec::new();
        self.for_all_intersect_rect_mut(rect, |a| {
            res_dino.push(into_ptr_usize(a.deref()));
        });
    
        let mut res_naive = Vec::new();
        NaiveAlgs::new(self.get_underlying_slice_mut()).for_all_intersect_rect_mut(rect, |a| {
            res_naive.push(into_ptr_usize(a.deref()));
        });
    
        res_dino.sort();
        res_naive.sort();
    
        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    
    fn assert_for_all_in_rect_mut(
        &mut self,
        rect: &axgeom::Rect<Self::Num>,
    ) {
        let mut res_dino = Vec::new();
        self.for_all_in_rect_mut(rect, |a| {
            res_dino.push(into_ptr_usize(a.deref()));
        });
    
        let mut res_naive = Vec::new();
        NaiveAlgs::new(self.get_underlying_slice_mut()).for_all_in_rect_mut(rect, |a| {
            res_naive.push(into_ptr_usize(a.deref()));
        });
    
        res_dino.sort();
        res_naive.sort();
    
        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    fn assert_colliding_pairs_mut(&mut self){
        let mut res_dino = Vec::new();
        self.find_colliding_pairs_mut(|a, b| {
            let a = into_ptr_usize(a.deref());
            let b = into_ptr_usize(b.deref());
            let k = if a < b { (a, b) } else { (b, a) };
            res_dino.push(k);
        });
    
        let mut res_naive = Vec::new();
        NaiveAlgs::new(self.get_underlying_slice_mut()).find_colliding_pairs_mut(|a, b| {
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
    fn assert_raycast_mut<'b,Acc>(
            &'b mut self,
            ray: axgeom::Ray<Self::Num>,
            acc: &mut Acc,
            mut broad: impl FnMut(&mut Acc, &Ray<Self::Num>, &Rect<Self::Num>) -> CastResult<Self::Num>,
            mut fine: impl FnMut(&mut Acc, &Ray<Self::Num>, &Self::T) -> CastResult<Self::Num>,
            border: Rect<Self::Num>,
        )
        where
            'a: 'b, Self::Num:core::fmt::Debug
    {
    
        let bots = self.get_underlying_slice_mut();

        let mut res_naive = Vec::new();
        match NaiveAlgs::new(bots).raycast_mut(ray, acc, &mut broad, &mut fine, border) {
            axgeom::CastResult::Hit((bots, mag)) => {
                for a in bots.into_iter() {
                    
                    let r=*a.get();
                    let j = into_ptr_usize(a.into_ref());
                    res_naive.push((j,r, mag))
                }
            }
            axgeom::CastResult::NoHit => {
                //do nothing
            }
        }
    
        let mut res_dino = Vec::new();
        match self.raycast_mut(ray, acc, broad, fine, border) {
            axgeom::CastResult::Hit((bots, mag)) => {
                for a in bots.into_iter() {
                    let r=*a.get();
                    let j = into_ptr_usize(a.into_ref());
                    res_dino.push((j,r, mag))
                }
            }
            axgeom::CastResult::NoHit => {
                //do nothing
            }
        }
    
        res_naive.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        res_dino.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
        //dbg!("{:?}  {:?}",res_naive.len(),res_dino.len());
        assert_eq!(
            res_naive.len(),
            res_dino.len(),
            "len:{:?}",
            (res_naive, res_dino)
        );
        assert!(
            res_naive.iter().eq(res_dino.iter()),
            "nop:\n\n naive:{:?} \n\n broc:{:?}",
            res_naive,
            res_dino
        );
    
    }

    fn assert_k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<Self::Num>,
        num: usize,
        acc: &mut Acc,
        mut broad: impl FnMut(&mut Acc, Vec2<Self::Num>, &Rect<Self::Num>) -> Self::Num,
        mut fine: impl FnMut(&mut Acc, Vec2<Self::Num>, &Self::T) -> Self::Num,
        border:Rect<Self::Num>
    ){
        let bots = self.get_underlying_slice_mut();

        let mut res_naive = NaiveAlgs::new(bots)
            .k_nearest_mut(point, num, acc, &mut broad, &mut fine)
            .into_vec()
            .drain(..)
            .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
            .collect::<Vec<_>>();

        let r = self.k_nearest_mut(point, num, acc, broad, fine, border);
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

    #[must_use]
    fn assert_tree_invariants(&self)->bool{
        inner(self.axis(), self.vistr().with_depth(compt::Depth(0))).is_ok()
    }

}
use super::*;
use crate::tree::*;


fn inner<A: Axis, T: Aabb>(axis: A, iter: compt::LevelIter<Vistr<Node<T>>>) -> Result<(), ()> {
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


