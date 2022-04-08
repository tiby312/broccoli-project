//! Provides 2d broadphase collision detection.

pub mod node_handle;
use node_handle::*;
mod oned;

pub use prevec::PreVec;
mod prevec;

use super::tools;
use super::*;

pub trait CollisionHandler<T: Aabb> {
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>);
}
impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> CollisionHandler<T> for F {
    #[inline(always)]
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        self(a, b);
    }
}

///Panics if a disconnect is detected between all colfind methods.
pub fn assert_query<T: Aabb>(bots: &mut [T]) {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct CollisionPtr {
        inner: Vec<(usize, usize)>,
    }

    pub fn collect_pairs<N: Num, T>(
        func: &mut impl CollisionApi<BBox<N, (usize, T)>>,
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

pub trait CollisionApi<T: Aabb> {
    fn colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>));
}
impl<'a, T: Aabb> CollisionApi<T> for AabbPin<&'a mut [T]> {
    fn colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        tools::for_every_pair(AabbPin::new(self).flatten(), move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func(a, b);
            }
        });
    }
}

use crate::tree::TreeInner;

impl<'a, T: Aabb, S: NodeHandler> CollisionApi<T> for TreeInner<Node<'a, T>, S> {
    fn colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = PreVec::new();
        let sorter = self.sorter();
        CollVis::new(self.vistr_mut(), true, sorter).recurse_seq(&mut prevec, &mut func);
    }
}

impl<'a, T: Aabb> CollisionApi<T> for SweepAndPrune<'a, T> {
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

pub struct SweepAndPrune<'a, T> {
    inner: &'a mut [T],
}

impl<'a, T> SweepAndPrune<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        SweepAndPrune { inner }
    }
}

#[must_use]
pub struct NodeFinisher<'a, 'b, T, F, H> {
    func: &'a mut F,
    prevec: &'a mut PreVec,
    is_xaxis: bool,
    bots: AabbPin<&'b mut [T]>,
    handler: H,
}
impl<'a, 'b, T: Aabb, F: CollisionHandler<T>, H: NodeHandler> NodeFinisher<'a, 'b, T, F, H> {
    pub fn finish(self) -> (&'a mut PreVec, &'a mut F) {
        if self.is_xaxis {
            self.handler
                .handle_node(self.func, self.prevec, axgeom::XAXIS.next(), self.bots);
        } else {
            self.handler
                .handle_node(self.func, self.prevec, axgeom::YAXIS.next(), self.bots);
        }
        (self.prevec, self.func)
    }
}

use crate::tree::splitter::Splitter;

pub trait CollidingPairsBuilder<'a, T: Aabb + 'a, SO: NodeHandler> {
    fn colliding_pairs_builder<'b>(&'b mut self) -> CollVis<'a, 'b, T, SO>;

    fn height_seq_fallback(&self) -> usize {
        5
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

    //TODO make these splitters api go behind a feature
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
            if vistr.vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(prevec, &mut func);
                splitter
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(prevec, &mut func);
                if let Some([left, right]) = rest {
                    let (s1, s2) = splitter.div();

                    let (s1, s2) = rayon_core::join(
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
            if vistr.vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(prevec, &mut func);
            } else {
                let func2 = func.clone();
                let (n, rest) = vistr.collide_and_next(prevec, &mut func);
                if let Some([left, right]) = rest {
                    rayon_core::join(
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
}

impl<'a, T: Aabb, S: NodeHandler> CollidingPairsBuilder<'a, T, S> for TreeInner<Node<'a, T>, S> {
    fn colliding_pairs_builder<'b>(&'b mut self) -> CollVis<'a, 'b, T, S> {
        let sorter = self.sorter();
        CollVis::new(self.vistr_mut(), true, sorter)
    }
}

/// The main primitive
pub struct CollVis<'a, 'b, T: Aabb, N> {
    vistr: VistrMutPin<'b, Node<'a, T>>,
    is_xaxis: bool,
    handler: N,
}
impl<'a, 'b, T: Aabb, N: NodeHandler> CollVis<'a, 'b, T, N> {
    pub(crate) fn new(vistr: VistrMutPin<'b, Node<'a, T>>, is_xaxis: bool, handler: N) -> Self {
        CollVis {
            vistr,
            is_xaxis,
            handler,
        }
    }

    pub fn collide_and_next<'x, F: CollisionHandler<T>>(
        mut self,
        prevec: &'x mut PreVec,
        func: &'x mut F,
    ) -> (NodeFinisher<'x, 'b, T, F, N>, Option<[Self; 2]>) {
        pub struct Recurser<'a, NO, C> {
            pub handler: &'a mut NO,
            pub sweeper: &'a mut C,
            pub prevec: &'a mut PreVec,
        }

        fn collide_self<A: axgeom::Axis, T: crate::Aabb>(
            this_axis: A,
            v: VistrMutPin<Node<T>>,
            data: &mut Recurser<impl NodeHandler, impl CollisionHandler<T>>,
        ) {
            let (nn, rest) = v.next();

            /*
            data.handler.handle_node(
                data.sweeper,
                data.prevec,
                this_axis.next(),
                nn.borrow_mut().into_range(),
            );
            */

            if let Some([mut left, mut right]) = rest {
                struct InnerRecurser<'a, 'node, T: Aabb, NN, C, B: Axis> {
                    anchor: NodeAxis<'a, 'node, T, B>,
                    handler: &'a mut NN,
                    sweeper: &'a mut C,
                    prevec: &'a mut PreVec,
                }

                impl<'a, 'node, T: Aabb, NN, C, B: Axis> InnerRecurser<'a, 'node, T, NN, C, B>
                where
                    NN: NodeHandler,
                    C: CollisionHandler<T>,
                {
                    fn recurse<
                        A: Axis, //this axis
                    >(
                        &mut self,
                        this_axis: A,
                        m: VistrMutPin<Node<T>>,
                    ) {
                        let anchor_axis = self.anchor.axis;
                        let (mut nn, rest) = m.next();

                        let current = NodeAxis {
                            node: nn.borrow_mut(),
                            axis: this_axis,
                        };

                        self.handler.handle_children(
                            self.sweeper,
                            self.prevec,
                            self.anchor.borrow_mut(),
                            current,
                        );

                        if let Some([left, right]) = rest {
                            //Continue to recurse even if we know there are no more bots
                            //This simplifies query algorithms that might be building up
                            //a tree.
                            if let Some(div) = nn.div {
                                if anchor_axis.is_equal_to(this_axis) {
                                    use core::cmp::Ordering::*;
                                    match self.anchor.node.cont.contains_ext(div) {
                                        Less => {
                                            self.recurse(this_axis.next(), right);
                                            return;
                                        }
                                        Greater => {
                                            self.recurse(this_axis.next(), left);
                                            return;
                                        }
                                        Equal => {}
                                    }
                                }
                            }

                            self.recurse(this_axis.next(), left);
                            self.recurse(this_axis.next(), right);
                        }
                    }
                }

                {
                    let mut g = InnerRecurser {
                        anchor: NodeAxis {
                            node: nn,
                            axis: this_axis,
                        },
                        handler: data.handler,
                        sweeper: data.sweeper,
                        prevec: data.prevec,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut());
                    g.recurse(this_axis.next(), right.borrow_mut());
                }
            }
        }

        {
            let mut data = Recurser {
                handler: &mut self.handler,
                sweeper: func,
                prevec,
            };

            if self.is_xaxis {
                collide_self(axgeom::XAXIS, self.vistr.borrow_mut(), &mut data);
            } else {
                collide_self(axgeom::YAXIS, self.vistr.borrow_mut(), &mut data);
            }
        }

        let (n, rest) = self.vistr.next();
        let fin = NodeFinisher {
            func,
            prevec,
            is_xaxis: self.is_xaxis,
            bots: n.into_range(),
            handler: self.handler,
        };

        //let (_, rest) = self.vistr.next();

        (
            fin,
            if let Some([left, right]) = rest {
                Some([
                    CollVis {
                        vistr: left,
                        is_xaxis: !self.is_xaxis,
                        handler: self.handler,
                    },
                    CollVis {
                        vistr: right,
                        is_xaxis: !self.is_xaxis,
                        handler: self.handler,
                    },
                ])
            } else {
                None
            },
        )
    }

    pub fn recurse_seq(self, prevec: &mut PreVec, mut func: impl CollisionHandler<T>) {
        self.recurse_seq_inner(prevec, &mut func)
    }

    fn recurse_seq_inner(self, prevec: &mut PreVec, func: &mut impl CollisionHandler<T>) {
        let (n, rest) = self.collide_and_next(prevec, func);

        let (prevec, func) = n.finish();
        if let Some([a, b]) = rest {
            a.recurse_seq_inner(prevec, func);
            b.recurse_seq_inner(prevec, func);
        }
    }
}
