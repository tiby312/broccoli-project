//! Provides 2d broadphase collision detection.

mod oned;

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

#[cfg(feature = "rayon")]
pub mod par {
    use super::*;
    pub trait ParCollidingPairsApi<T: Aabb> {
        fn par_colliding_pairs(
            &mut self,
            func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        );
    }

    impl<'a, T: Aabb + Send> ParCollidingPairsApi<T> for TreeInner<Node<'a, T>, DefaultSorter>
    where
        T::Num: Send,
    {
        fn par_colliding_pairs(
            &mut self,
            func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        ) {
            CollidingPairsBuilder::new(self, DefaultNodeHandler::new(func)).build_par();
        }
    }

    impl<'a, T: Aabb + Send> ParCollidingPairsApi<T> for TreeInner<Node<'a, T>, NoSorter>
    where
        T::Num: Send,
    {
        fn par_colliding_pairs(
            &mut self,
            func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        ) {
            CollidingPairsBuilder::new(self, NoSortNodeHandler::new(func)).build_par();
        }
    }

    impl<'a, T: Aabb + Send> ParCollidingPairsApi<T> for SweepAndPrune<'a, T>
    where
        T::Num: Send,
    {
        fn par_colliding_pairs(
            &mut self,
            func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        ) {
            self.par_query(func);
        }
    }
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

impl<'a, T: Aabb> CollidingPairsApi<T> for TreeInner<Node<'a, T>, DefaultSorter> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        CollidingPairsBuilder::new(self, DefaultNodeHandler::new(func)).build()
    }
}

impl<'a, T: Aabb> CollidingPairsApi<T> for TreeInner<Node<'a, T>, NoSorter> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        CollidingPairsBuilder::new(self, NoSortNodeHandler::new(func)).build()
    }
}

impl<'a, T: Aabb> CollidingPairsApi<T> for SweepAndPrune<'a, T> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self.query(func);
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
        broccoli_tree::util::sweeper_update(axis, inner);

        SweepAndPrune { inner }
    }

    pub fn query(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = PreVec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        oned::find_2d(&mut prevec, default_axis(), bots, &mut func, true);
    }

    #[cfg(feature = "rayon")]
    ///Sweep and prune algorithm.
    pub fn par_query(
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

    #[cfg(feature = "rayon")]
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
        ) -> N
        where
            T: Send,
            T::Num: Send,
            N: Splitter + Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(&mut handler);
                handler
            } else {
                let (mut h1, h2) = handler.div();
                let (n, rest) = vistr.collide_and_next(&mut h1);
                if let Some([left, right]) = rest {
                    let (h1, h2) = rayon::join(
                        || {
                            n.finish(&mut h1);
                            recurse_par(left, h1, num_seq_fallback)
                        },
                        || recurse_par(right, h2, num_seq_fallback),
                    );
                    h1.add(h2)
                } else {
                    let _ = n.finish(&mut h1);
                    h1
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

    /*
    #[cfg(feature = "rayon")]
    pub fn build_with_splitter_par<SS: Splitter + Send>(self, splitter: SS) -> SS
    where
        T: Send,
        T::Num: Send,
        SO: Clone + Send,
    {
        ///
        /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
        ///
        pub fn recurse_par_splitter<T: Aabb, N: NodeHandler<T>, S: Splitter + Send>(
            vistr: CollVis<T>,
            num_seq_fallback: usize,
            mut func: N,
            splitter: S,
        ) -> S
        where
            T: Send,
            T::Num: Send,
            N: Clone + Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(&mut func);
                splitter
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(&mut func);
                if let Some([left, right]) = rest {
                    let (s1, s2) = splitter.div();

                    let (s1, s2) = rayon::join(
                        || {
                            n.finish(&mut func);
                            recurse_par_splitter(left, num_seq_fallback, func, s1)
                        },
                        || recurse_par_splitter(right, num_seq_fallback, func2, s2),
                    );

                    s1.add(s2)
                } else {
                    let _ = n.finish(&mut func);
                    splitter
                }
            }
        }
        recurse_par_splitter(self.vis, self.num_seq_fallback, self.handler, splitter)
    }
    */
}
