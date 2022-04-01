//! Provides 2d broadphase collision detection.

mod node_handle;
mod oned;

pub mod par;
pub mod splitter;

pub use self::node_handle::*;
use super::tools;
use super::*;

//TODO remove
pub trait CollisionHandler<T: Aabb> {
    fn collide(&mut self, a: HalfPin<&mut T>, b: HalfPin<&mut T>);
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_query<T: Aabb>(bots: &mut [T]) {
    use core::ops::Deref;
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_dino = Vec::new();
    let mut prevec = PreVec::new();

    let mut tree = crate::new(bots);
    tree.colliding_pairs()
        .recurse_seq(&mut prevec, &mut |a, b| {
            let a = into_ptr_usize(a.deref());
            let b = into_ptr_usize(b.deref());
            let k = if a < b { (a, b) } else { (b, a) };
            res_dino.push(k);
        });

    let mut res_naive = Vec::new();
    query_naive_mut(HalfPin::new(bots), |a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_naive.push(k);
    });

    res_naive.sort_unstable();
    res_dino.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());

    let a: Vec<_> = res_naive.iter().collect();
    let b: Vec<_> = res_dino.iter().collect();
    assert_eq!(a.len(), b.len());
    assert_eq!(a, b);
}

///Naive implementation
pub fn query_naive_mut<T: Aabb>(
    bots: HalfPin<&mut [T]>,
    mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>),
) {
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
    func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>),
) {
    broccoli_tree::util::sweeper_update(axis, bots);

    struct Bl<F> {
        func: F,
    }

    impl<T: Aabb, F: FnMut(HalfPin<&mut T>, HalfPin<&mut T>)> CollisionHandler<T> for Bl<F> {
        #[inline(always)]
        fn collide(&mut self, a: HalfPin<&mut T>, b: HalfPin<&mut T>) {
            (self.func)(a, b);
        }
    }
    let mut prevec = crate::util::PreVec::with_capacity(2048);
    let bots = HalfPin::new(bots);
    oned::find_2d(&mut prevec, axis, bots, &mut Bl { func });
}

#[must_use]
pub struct NodeFinisher<'a, 'b, T, F, H> {
    func: F,
    prevec: &'a mut PreVec,
    is_xaxis: bool,
    bots: HalfPin<&'b mut [T]>,
    handler: H,
}
impl<'a, 'b, T: Aabb, F: FnMut(HalfPin<&mut T>, HalfPin<&mut T>), H: NodeHandler>
    NodeFinisher<'a, 'b, T, F, H>
{
    pub fn finish(mut self) -> (&'a mut PreVec, F) {
        if self.is_xaxis {
            self.handler.handle_node(
                &mut QueryFnMut(&mut self.func),
                self.prevec,
                axgeom::XAXIS.next(),
                self.bots,
            );
        } else {
            self.handler.handle_node(
                &mut QueryFnMut(&mut self.func),
                self.prevec,
                axgeom::YAXIS.next(),
                self.bots,
            );
        }
        (self.prevec, self.func)
    }
}

struct QueryFnMut<F>(F);
impl<F> QueryFnMut<F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFnMut<F> {
        QueryFnMut(func)
    }
}

impl<T: Aabb, F: FnMut(HalfPin<&mut T>, HalfPin<&mut T>)> CollisionHandler<T> for QueryFnMut<F> {
    #[inline(always)]
    fn collide(&mut self, a: HalfPin<&mut T>, b: HalfPin<&mut T>) {
        self.0(a, b);
    }
}

pub struct CollVis<'a, 'b, T: Aabb, N> {
    vistr: VistrMut<'b, Node<'a, T>>,
    is_xaxis: bool,
    handler: N,
}
impl<'a, 'b, T: Aabb, N: NodeHandler> CollVis<'a, 'b, T, N> {
    pub(crate) fn new(vistr: VistrMut<'b, Node<'a, T>>, is_xaxis: bool, handler: N) -> Self {
        CollVis {
            vistr,
            is_xaxis,
            handler,
        }
    }

    pub fn collide_and_next<'x, F: FnMut(HalfPin<&mut T>, HalfPin<&mut T>)>(
        mut self,
        prevec: &'x mut PreVec,
        mut func: F,
    ) -> (NodeFinisher<'x, 'b, T, F, N>, Option<[Self; 2]>) {
        pub struct Recurser<'a, NO, C> {
            pub handler: &'a mut NO,
            pub sweeper: &'a mut C,
            pub prevec: &'a mut PreVec,
        }

        fn collide_self<A: axgeom::Axis, T: crate::Aabb>(
            this_axis: A,
            v: VistrMut<Node<T>>,
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
                        m: VistrMut<Node<T>>,
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
            let mut g = QueryFnMut::new(&mut func);
            let mut data = Recurser {
                handler: &mut self.handler,
                sweeper: &mut g,
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

    pub fn recurse_seq(
        self,
        prevec: &mut PreVec,
        func: &mut impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>),
    ) {
        let (n, rest) = self.collide_and_next(prevec, func);

        let (_, func) = n.finish();
        if let Some([a, b]) = rest {
            a.recurse_seq(prevec, func);
            b.recurse_seq(prevec, func);
        }
    }
}
