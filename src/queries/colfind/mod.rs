//! Provides 2d broadphase collision detection.

mod oned;

//use std::borrow::Borrow;

use super::tools;
use super::*;
pub mod build;
use build::*;
pub mod handler;
use handler::*;

///Panics if a disconnect is detected between all colfind methods.
pub fn assert_query<T: Aabb>(bots: &mut [T]) {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct CollisionPtr {
        inner: Vec<(usize, usize)>,
    }

    impl CollisionPtr {
        fn new() -> Self {
            CollisionPtr { inner: vec![] }
        }
        fn add<N>(&mut self, a: &BBox<N, usize>, b: &BBox<N, usize>) {
            let a = a.inner;
            let b = b.inner;
            let (a, b) = if a < b { (a, b) } else { (b, a) };

            self.inner.push((a, b));
        }
        pub fn finish(&mut self) {
            self.inner.sort_unstable();
        }
    }

    let mut bots: Vec<_> = bots
        .iter_mut()
        .enumerate()
        .map(|(i, x)| crate::bbox(*x.get(), i))
        .collect();
    let bots = bots.as_mut_slice();

    let naive_res = {
        let mut cc = CollisionPtr::new();
        Naive::new(bots).find_colliding_pairs(|a, b| {
            cc.add(&*a, &*b);
        });
        cc.finish();
        cc
    };

    let tree_res = {
        let mut cc = CollisionPtr::new();

        Tree2::new(bots).find_colliding_pairs(|a, b| {
            cc.add(&*a, &*b);
        });
        cc.finish();
        cc
    };

    let notsort_res = {
        let mut cc = CollisionPtr::new();

        NotSortedTree::new(bots).find_colliding_pairs(|a, b| {
            cc.add(&*a, &*b);
        });
        cc.finish();
        cc
    };

    let sweep_res = {
        let mut cc = CollisionPtr::new();
        SweepAndPrune::new(bots).find_colliding_pairs(|a, b| {
            cc.add(&*a, &*b);
        });
        cc.finish();
        cc
    };

    assert_eq!(naive_res.inner.len(), sweep_res.inner.len());
    assert_eq!(naive_res.inner.len(), tree_res.inner.len());
    assert_eq!(naive_res.inner.len(), notsort_res.inner.len());

    assert_eq!(naive_res, tree_res);
    assert_eq!(naive_res, sweep_res);
    assert_eq!(naive_res, notsort_res);
}


pub struct Naive<'a, T> {
    inner: AabbPin<&'a mut [T]>,
}
impl<'a, T: Aabb> Naive<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Naive {
            inner: AabbPin::from_mut(inner),
        }
    }
    pub fn from_pinned(inner: AabbPin<&'a mut [T]>) -> Self {
        Naive { inner }
    }

    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        queries::for_every_pair(self.inner.borrow_mut(), move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func(a, b);
            }
        });
    }
}

///
/// Sweep and prune collision finding algorithm
///
pub struct SweepAndPrune<'a, T> {
    inner: &'a mut [T],
}

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        let axis = default_axis();
        tree::util::sweeper_update(axis, inner);

        SweepAndPrune { inner }
    }

    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        oned::find_2d(&mut prevec, default_axis(), bots, &mut func, true);
    }

    #[cfg(feature = "parallel")]
    ///Sweep and prune algorithm.
    pub fn par_find_colliding_pairs(
        &mut self,
        mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
    ) where
        T: Send,
    {
        let axis = default_axis();
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        let a2 = axis.next();
        let _ = oned::find_par(
            &mut prevec,
            axis,
            bots,
            move |a: AabbPin<&mut T>, b: AabbPin<&mut T>| {
                if a.get().get_range(a2).intersects(b.get().get_range(a2)) {
                    func(a, b);
                }
            },
        );
    }
}

use crate::tree::splitter::Splitter;

const SEQ_FALLBACK_DEFAULT: usize = 2_400;

pub struct NotSortedTree<'a, T: Aabb> {
    nodes:Vec<Node<'a,T>>,
    total_num_elem:usize
}

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn new(bots: &'a mut [T]) -> Self {
        let total_num_elem=bots.len();
        let nodes=tree::TreeBuilder::new(DefaultSorter,bots).build();
        NotSortedTree {
            nodes,
            total_num_elem
        }
    }

    pub fn par_new(bots: &'a mut [T]) -> Self
    where
        T: Send,
        T::Num: Send,
    {
        let total_num_elem=bots.len();
        let nodes=tree::TreeBuilder::new(DefaultSorter,bots).build_par();
        NotSortedTree {
            nodes,
            total_num_elem
        }
    }

    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.colliding_pairs_builder(NoSortNodeHandler::new(func))
            .build();
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        self.colliding_pairs_builder(NoSortNodeHandler::new(func))
            .build_par();
    }

    pub fn colliding_pairs_builder<'b, SO: NodeHandler<T, Sorter = NoSorter>>(
        &'b mut self,
        handler: SO,
    ) -> CollidingPairsBuilder<'a, 'b, T, SO> {
        CollidingPairsBuilder::new(&mut self.inner, handler)
    }
}

impl<'a, T: Aabb> Tree2<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.colliding_pairs_builder(DefaultNodeHandler::new(func))
            .build();
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        self.colliding_pairs_builder(DefaultNodeHandler::new(func))
            .build_par();
    }

    pub fn colliding_pairs_builder<'b, SO: NodeHandler<T, Sorter = DefaultSorter>>(
        &'b mut self,
        handler: SO,
    ) -> CollidingPairsBuilder<'a, 'b, T, SO> {
        CollidingPairsBuilder::new(&mut self.inner, handler)
    }
}





#[must_use]
pub struct CollidingPairsBuilder<'a, 'b, T: Aabb, SO: NodeHandler<T>> {
    vis: CollVis<'a, 'b, T>,
    pub num_seq_fallback: usize,
    pub handler: SO,
}

impl<'a, 'b, T: Aabb, SO: NodeHandler<T>> CollidingPairsBuilder<'a, 'b, T, SO> {
    pub fn new(tree: &'b mut TreeInner<Node<'a, T>, SO::Sorter>, handler: SO) -> Self {
        CollidingPairsBuilder {
            vis: CollVis::new(tree.vistr_mut(), default_axis().to_dyn()),
            num_seq_fallback: SEQ_FALLBACK_DEFAULT,
            handler,
        }
    }
    pub fn build(mut self) {
        self.vis.recurse_seq(&mut self.handler);
    }

    #[cfg(feature = "parallel")]
    pub fn build_par(self) -> SO
    where
        T: Send,
        T::Num: Send,
        SO: Splitter + Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par<T: Aabb, N: NodeHandler<T>>(
            vistr: CollVis<T>,
            mut handler: N,
            num_seq_fallback: usize,
        )
        where
            T: Send,
            T::Num: Send,
            N: Splitter + Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(&mut handler);
                handler
            } else {
                let h2 = handler.div();
                let (n, rest) = vistr.collide_and_next(&mut handler);
                if let Some([left, right]) = rest {
                    let (h1, h2) = rayon::join(
                        || {
                            n.finish(&mut handler);
                            recurse_par(left, handler, num_seq_fallback)
                        },
                        || recurse_par(right, h2, num_seq_fallback),
                    );
                    handler.add(h2);
                } else {
                    let _ = n.finish(&mut handler);
                }
            }
        }

        recurse_par(self.vis, self.handler, self.num_seq_fallback)
    }

    pub fn build_with_splitter<SS: Splitter>(mut self, splitter: SS) -> SS {
        pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler<T>, SS: Splitter>(
            vistr: CollVis<T>,
            splitter: SS,
            func: &mut S,
        ) -> SS {
            let (n, rest) = vistr.collide_and_next(func);

            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();
                n.finish(func);
                let al = recurse_seq_splitter(left, s1, func);
                let ar = recurse_seq_splitter(right, s2, func);
                al.add(ar)
            } else {
                n.finish(func);
                splitter
            }
        }
        recurse_seq_splitter(self.vis, splitter, &mut self.handler)
    }
}
