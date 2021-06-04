//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
mod oned;

use self::inner::*;
use self::node_handle::*;
use super::tools;
use super::*;
use crate::Joinable;

pub mod builder;
use self::builder::CollisionHandler;

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_query<T: Aabb>(tree: &mut crate::Tree<T>) {
    use core::ops::Deref;
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_dino = Vec::new();
    tree.find_colliding_pairs_mut(|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_dino.push(k);
    });

    let mut res_naive = Vec::new();
    query_naive_mut(tree.get_elements_mut(), |a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_naive.push(k);
    });

    res_naive.sort_unstable();
    res_dino.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Naive implementation
pub fn query_naive_mut<T: Aabb>(bots: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
    tools::for_every_pair(bots, move |a, b| {
        if a.get().intersects_rect(b.get()) {
            func(a, b);
        }
    });
}

///Sweep and prune algorithm.
pub fn query_sweep_mut<T: Aabb>(
    axis: impl Axis,
    bots: &mut [T],
    func: impl FnMut(PMut<T>, PMut<T>),
) {
    crate::util::sweeper_update(axis, bots);

    struct Bl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> {
        func: F,
        _p: PhantomData<T>,
    }

    impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> CollisionHandler for Bl<T, F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
            (self.func)(a, b);
        }
    }
    let mut prevec = crate::util::PreVec::with_capacity(2048);
    let bots = PMut::new(bots);
    oned::find_2d(
        &mut prevec,
        axis,
        bots,
        &mut Bl {
            func,
            _p: PhantomData,
        },
    );
}

