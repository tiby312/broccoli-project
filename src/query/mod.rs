//! Contains query modules for each query algorithm.

mod inner_prelude {
    pub use crate::node::*;
    pub(crate) use crate::par;
    pub use axgeom::*;
    pub use crate::pmut::*;
    pub use crate::util::*;
    //pub use crate::inner_prelude::*;
    pub use alloc::vec::Vec;
    pub use axgeom;
    pub use axgeom::Rect;
    pub use compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
}

pub mod colfind;

pub mod draw;

pub mod knearest;

pub mod raycast;

pub mod intersect_with;

pub mod nbody;

pub mod rect;

mod tools;

use self::inner_prelude::*;


///Query modules provide functions based off of this trait.
pub trait Queries<'a> {
    type A: Axis;
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,query::Queries};
    /// let mut bots = [bbox(rect(0,10,0,10),0)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
    ///    *b.unpack_inner()+=1;    
    /// }
    /// assert_eq!(bots[0].inner,1);
    ///```
    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,query::Queries};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// let mut test = Vec::new();
    /// for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.range.iter()){
    ///    test.push(b);
    /// }
    /// assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[must_use]
    fn vistr(&self) -> Vistr<Node<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,analyze,query::Queries};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use axgeom::Axis;
    /// assert!(tree.axis().is_equal_to(analyze::default_axis()));
    ///```
    #[must_use]
    fn axis(&self) -> Self::A;
}

///panics if a broken broccoli tree invariant is detected.
///For debugging purposes only.
pub fn assert_tree_invariants<A:Axis,T:Aabb>(tree: &crate::Tree<A,T>)
where
    T::Num: core::fmt::Debug,
{
    fn inner<A: Axis, T: Aabb>(axis: A, iter: compt::LevelIter<Vistr<Node<T>>>) {
        fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
            for b in it {
                if b == val {
                    return true;
                }
            }
            false
        }

        let ((_depth, nn), rest) = iter.next();
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
            assert!(IsSorted::is_sorted_by(&mut nn.range.iter(), f));
        }

        if let Some([start, end]) = rest {
            match nn.div {
                Some(div) => {
                    match nn.cont {
                        Some(cont) => {
                            for bot in nn.range.iter() {
                                assert!(bot.get().get_range(axis).contains(div));
                            }

                            assert!(a_bot_has_value(
                                nn.range.iter().map(|b| b.get().get_range(axis).start),
                                div
                            ));

                            for bot in nn.range.iter() {
                                assert!(cont.contains_range(bot.get().get_range(axis)));
                            }

                            assert!(a_bot_has_value(
                                nn.range.iter().map(|b| b.get().get_range(axis).start),
                                cont.start
                            ));
                            assert!(a_bot_has_value(
                                nn.range.iter().map(|b| b.get().get_range(axis).end),
                                cont.end
                            ));
                        }
                        None => assert!(nn.range.is_empty()),
                    }

                    inner(axis_next, start);
                    inner(axis_next, end);
                }
                None => {
                    for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                        assert!(n.range.is_empty());
                        assert!(n.cont.is_none());
                        assert!(n.div.is_none());
                    }
                }
            }
        }
    }

    inner(tree.axis(), tree.vistr().with_depth(compt::Depth(0)))
}
