//! Provides 2d broadphase collision detection.

mod oned;

use super::tools;
use super::*;
pub mod build;
use build::*;

///Panics if a disconnect is detected between all colfind methods.
pub fn assert_query<T: Aabb>(bots: &mut [T]) {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct CollisionPtr {
        inner: Vec<(usize, usize)>,
    }

    pub fn collect_pairs<N: Num, T>(
        func: &mut impl CollidingPairsApi<BBox<N, (usize, T)>>,
    ) -> CollisionPtr {
        let mut res = vec![];
        func.colliding_pairs(|a, b| {
            let a = a.inner.0;
            let b = b.inner.0;
            let (a, b) = if a < b { (a, b) } else { (b, a) };

            res.push((a, b));
        });

        res.sort_unstable();
        CollisionPtr { inner: res }
    }

    let mut bots: Vec<_> = bots
        .iter_mut()
        .enumerate()
        .map(|(i, x)| crate::bbox(*x.get(), (i, x)))
        .collect();
    let bots = bots.as_mut_slice();

    let nosort_res = collect_pairs(&mut TreeBuilder::new(NoSorter, bots).build());
    let sweep_res = collect_pairs(&mut SweepAndPrune::new(bots));
    let tree_res = collect_pairs(&mut crate::new(bots));
    let naive_res = collect_pairs(&mut AabbPin::from_mut(bots));

    assert_eq!(naive_res.inner.len(), sweep_res.inner.len());
    assert_eq!(naive_res.inner.len(), tree_res.inner.len());
    assert_eq!(naive_res.inner.len(), nosort_res.inner.len());

    assert_eq!(naive_res, tree_res);
    assert_eq!(naive_res, sweep_res);
    assert_eq!(naive_res, nosort_res);
}

///
/// Make colliding pair queries
///
pub trait CollidingPairsApi<T: Aabb> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>));
}
impl<'a, T: Aabb> CollidingPairsApi<T> for AabbPin<&'a mut [T]> {
    fn colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        queries::for_every_pair(AabbPin::new(self).flatten(), move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func(a, b);
            }
        });
    }
}

use crate::tree::TreeInner;

impl<'a, T: Aabb, S: NodeHandler> CollidingPairsApi<T> for TreeInner<Node<'a, T>, S> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.colliding_pairs_builder(func).build()
    }
}

impl<'a, T: Aabb> CollidingPairsApi<T> for SweepAndPrune<'a, T> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        ///Sweep and prune algorithm.
        fn query_sweep_mut<T: Aabb>(
            axis: impl Axis,
            bots: &mut [T],
            mut func: impl CollisionHandler<T>,
        ) {
            broccoli_tree::util::sweeper_update(axis, bots);

            let mut prevec = PreVec::with_capacity(2048);
            let bots = AabbPin::new(bots);
            oned::find_2d(&mut prevec, axis, bots, &mut func, true);
        }
        query_sweep_mut(default_axis(), self.inner, func)
    }
}

///
/// Sweep and prune collision finding algorithm
///
pub struct SweepAndPrune<'a, T> {
    inner: &'a mut [T],
}

impl<'a, T> SweepAndPrune<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        SweepAndPrune { inner }
    }
}

use crate::tree::splitter::Splitter;

pub trait CollidingPairsBuilderApi<'a, T: Aabb, SO: NodeHandler> {
    fn vistr_mut<'b>(&'b mut self) -> VistrMutPin<'b, broccoli_tree::node::Node<'a, T>>;
    fn sorter(&mut self) -> SO;

    fn colliding_pairs_builder<'b, F>(
        &'b mut self,
        func: F,
    ) -> CollidingPairsBuilder<'a, 'b, T, SO, F>
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    {
        let sorter = self.sorter();
        CollidingPairsBuilder {
            vis: CollVis::new(self.vistr_mut(), true, sorter),
            num_seq_fallback: 2_400,
            func,
        }
    }
}

impl<'a, T: Aabb, SO: NodeHandler> CollidingPairsBuilderApi<'a, T, SO>
    for TreeInner<Node<'a, T>, SO>
{
    fn vistr_mut<'b>(&'b mut self) -> VistrMutPin<'b, tree::node::Node<'a, T>> {
        TreeInner::vistr_mut(self)
    }

    fn sorter(&mut self) -> SO {
        TreeInner::sorter(self)
    }
}

#[must_use]
pub struct CollidingPairsBuilder<'a, 'b, T: Aabb, SO, F> {
    pub vis: CollVis<'a, 'b, T, SO>,
    pub num_seq_fallback: usize,
    pub func: F,
}

impl<'a, 'b, T: Aabb, SO: NodeHandler, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>
    CollidingPairsBuilder<'a, 'b, T, SO, F>
{
    pub fn build(mut self) {
        let mut prevec = PreVec::new();
        self.vis.recurse_seq(&mut prevec, &mut self.func);
    }

    #[cfg(feature = "rayon")]
    pub fn build_par(self)
    where
        T: Send,
        T::Num: Send,
        F: Clone + Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par<T: Aabb, N: NodeHandler>(
            vistr: CollVis<T, N>,
            prevec: &mut PreVec,
            num_seq_fallback: usize,
            mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        ) where
            T: Send,
            T::Num: Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(prevec, &mut func);
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(prevec, &mut func);
                if let Some([left, right]) = rest {
                    rayon::join(
                        || {
                            let (prevec, func) = n.finish();
                            recurse_par(left, prevec, num_seq_fallback, func.clone());
                        },
                        || {
                            let mut prevec = PreVec::new();
                            recurse_par(right, &mut prevec, num_seq_fallback, func2);
                        },
                    );
                } else {
                    let _ = n.finish();
                }
            }
        }

        let mut prevec = PreVec::new();

        //TODO best level define somewhere?
        recurse_par(self.vis, &mut prevec, self.num_seq_fallback, self.func)
    }

    pub fn build_with_splitter<SS: Splitter>(mut self, splitter: SS) -> SS {
        pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler, SS: Splitter>(
            vistr: CollVis<T, S>,
            splitter: SS,
            prevec: &mut PreVec,
            func: &mut impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        ) -> SS {
            let (n, rest) = vistr.collide_and_next(prevec, func);

            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();
                n.finish();
                let al = recurse_seq_splitter(left, s1, prevec, func);
                let ar = recurse_seq_splitter(right, s2, prevec, func);
                al.add(ar)
            } else {
                n.finish();
                splitter
            }
        }
        let mut prevec = PreVec::new();
        recurse_seq_splitter(self.vis, splitter, &mut prevec, &mut self.func)
    }

    #[cfg(feature = "rayon")]
    pub fn build_with_splitter_par<SS: Splitter + Send>(self, splitter: SS) -> SS
    where
        T: Send,
        T::Num: Send,
        F: Clone + Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par_splitter<T: Aabb, N: NodeHandler, S: Splitter + Send>(
            vistr: CollVis<T, N>,
            prevec: &mut PreVec,
            num_seq_fallback: usize,
            mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
            splitter: S,
        ) -> S
        where
            T: Send,
            T::Num: Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(prevec, &mut func);
                splitter
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(prevec, &mut func);
                if let Some([left, right]) = rest {
                    let (s1, s2) = splitter.div();

                    let (s1, s2) = rayon::join(
                        || {
                            let (prevec, func) = n.finish();
                            recurse_par_splitter(left, prevec, num_seq_fallback, func.clone(), s1)
                        },
                        || {
                            let mut prevec = PreVec::new();
                            recurse_par_splitter(right, &mut prevec, num_seq_fallback, func2, s2)
                        },
                    );

                    s1.add(s2)
                } else {
                    let _ = n.finish();
                    splitter
                }
            }
        }
        let mut prevec = PreVec::new();
        recurse_par_splitter(
            self.vis,
            &mut prevec,
            self.num_seq_fallback,
            self.func,
            splitter,
        )
    }
}
