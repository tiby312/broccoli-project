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

    let nosort_res = collect_pairs(&mut TreeInner::build(NoSorter, bots));
    let sweep_res = collect_pairs(&mut SweepAndPrune::new(bots));
    let tree_res = collect_pairs(&mut crate::new(bots));
    let naive_res = collect_pairs(&mut AabbPin::from_mut(bots));

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
    fn colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = PreVec::new();
        let sorter = self.sorter();
        CollVis::new(self.vistr_mut(), true, sorter).recurse_seq(&mut prevec, &mut func);
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
            oned::find_2d(&mut prevec, axis, bots, &mut func);
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

///
/// Provides extended options for collision pair finding specific to trees. (not applicable to naive/sweep and prune)
///
/// colliding pair finding functions that allow arbitrary code to be run every time the problem
/// is split into two and built back together. Useful for debuging/measuring performance.
///
pub trait CollidingPairsApiExt<'a, T: Aabb + 'a, SO: NodeHandler> {
    fn colliding_pairs_builder<'b>(&'b mut self) -> CollVis<'a, 'b, T, SO>;

    fn height_seq_fallback(&self) -> usize {
        5
    }

    #[cfg(feature = "rayon")]
    fn colliding_pairs_par(
        &mut self,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
    ) where
        T: Send,
        T::Num: Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par<T: Aabb, N: NodeHandler>(
            vistr: CollVis<T, N>,
            prevec: &mut PreVec,
            height_seq_fallback: usize,
            mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        ) where
            T: Send,
            T::Num: Send,
        {
            if vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(prevec, &mut func);
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(prevec, &mut func);
                if let Some([left, right]) = rest {
                    rayon::join(
                        || {
                            let (prevec, func) = n.finish();
                            recurse_par(left, prevec, height_seq_fallback, func.clone());
                        },
                        || {
                            let mut prevec = PreVec::new();
                            recurse_par(right, &mut prevec, height_seq_fallback, func2);
                        },
                    );
                }
            }
        }

        let mut prevec = PreVec::new();
        let h = self.height_seq_fallback();

        //TODO best level define somewhere?
        recurse_par(self.colliding_pairs_builder(), &mut prevec, h, func)
    }

    fn colliding_pairs_splitter<SS: Splitter>(
        &mut self,
        splitter: SS,
        mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    ) -> SS {
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
                splitter
            }
        }
        let mut prevec = PreVec::new();
        recurse_seq_splitter(
            self.colliding_pairs_builder(),
            splitter,
            &mut prevec,
            &mut func,
        )
    }

    #[cfg(feature = "rayon")]
    fn colliding_pairs_splitter_par<SS: Splitter + Send>(
        &mut self,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        splitter: SS,
    ) -> SS
    where
        T: Send,
        T::Num: Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par_splitter<T: Aabb, N: NodeHandler, S: Splitter + Send>(
            vistr: CollVis<T, N>,
            prevec: &mut PreVec,
            height_seq_fallback: usize,
            mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
            splitter: S,
        ) -> S
        where
            T: Send,
            T::Num: Send,
        {
            if vistr.get_height() <= height_seq_fallback {
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
                            recurse_par_splitter(
                                left,
                                prevec,
                                height_seq_fallback,
                                func.clone(),
                                s1,
                            )
                        },
                        || {
                            let mut prevec = PreVec::new();
                            recurse_par_splitter(right, &mut prevec, height_seq_fallback, func2, s2)
                        },
                    );

                    s1.add(s2)
                } else {
                    splitter
                }
            }
        }
        let mut prevec = PreVec::new();
        let h = self.height_seq_fallback();
        recurse_par_splitter(
            self.colliding_pairs_builder(),
            &mut prevec,
            h,
            func,
            splitter,
        )
    }
}

impl<'a, T: Aabb, S: NodeHandler> CollidingPairsApiExt<'a, T, S> for TreeInner<Node<'a, T>, S> {
    fn colliding_pairs_builder<'b>(&'b mut self) -> CollVis<'a, 'b, T, S> {
        let sorter = self.sorter();
        CollVis::new(self.vistr_mut(), true, sorter)
    }
}
