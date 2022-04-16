//!
//! Building blocks to find colliding pairs with trees
//!

use super::*;

///
/// Shorthand for `FnMut(AabbPin<&mut T>, AabbPin<&mut T>)` trait bound
///
pub trait CollisionHandler<T: Aabb> {
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>);
}
impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> CollisionHandler<T> for F {
    #[inline(always)]
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        self(a, b);
    }
}

///
/// Finish handling a node by calling finish()
///
#[must_use]
pub struct NodeFinisher<'a, 'b, T, F, H> {
    func: &'a mut F,
    prevec: &'a mut PreVec,
    is_xaxis: bool,
    bots: AabbPin<&'b mut [T]>,
    handler: H,
    is_leaf: bool,
}
impl<'a, 'b, T: Aabb, F: CollisionHandler<T>, H: NodeHandler> NodeFinisher<'a, 'b, T, F, H> {
    pub fn finish(self) -> (&'a mut PreVec, &'a mut F) {
        if self.is_xaxis {
            self.handler.handle_node(
                self.func,
                self.prevec,
                axgeom::XAXIS.next(),
                self.bots,
                self.is_leaf,
            );
        } else {
            self.handler.handle_node(
                self.func,
                self.prevec,
                axgeom::YAXIS.next(),
                self.bots,
                self.is_leaf,
            );
        }
        (self.prevec, self.func)
    }
}

/// The main primitive to visit each node and find colliding pairs
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

    pub fn get_height(&self) -> usize {
        self.vistr.get_height()
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
                        is_left: bool,
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
                            if let Some(div) = nn.div {
                                if anchor_axis.is_equal_to(this_axis) {
                                    /*
                                    use core::cmp::Ordering::*;
                                    match self.anchor.node.cont.contains_ext(div) {
                                        Less => {
                                            self.recurse(this_axis.next(), right,is_left);
                                            return;
                                        }
                                        Greater => {
                                            self.recurse(this_axis.next(), left,is_left);
                                            return;
                                        }
                                        Equal => {}
                                    }
                                    */

                                    match is_left {
                                        true => {
                                            if div < self.anchor.node.cont.start {
                                                self.recurse(this_axis.next(), right, is_left);
                                                return;
                                            }
                                        }
                                        false => {
                                            if div >= self.anchor.node.cont.end {
                                                self.recurse(this_axis.next(), left, is_left);
                                                return;
                                            }
                                        }
                                    }
                                }
                            }

                            self.recurse(this_axis.next(), left, is_left);
                            self.recurse(this_axis.next(), right, is_left);
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

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
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

        //TODO make height be zero for leaf?
        let is_leaf = self.get_height() == 1;

        let (n, rest) = self.vistr.next();

        let fin = NodeFinisher {
            func,
            prevec,
            is_xaxis: self.is_xaxis,
            bots: n.into_range(),
            handler: self.handler,
            is_leaf,
        };

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

/// Used by [`NodeHandler`]
pub struct NodeAxis<'a, 'node, T: Aabb, A: Axis> {
    pub node: AabbPin<&'a mut Node<'node, T>>,
    pub axis: A,
}

impl<'a, 'node, T: Aabb, A: Axis> NodeAxis<'a, 'node, T, A> {
    #[inline(always)]
    pub fn borrow_mut<'c>(&'c mut self) -> NodeAxis<'c, 'node, T, A>
    where
        'a: 'c,
    {
        NodeAxis {
            node: self.node.borrow_mut(),
            axis: self.axis,
        }
    }
}

///
/// Abstract over sorted and non sorted trees
///
pub trait NodeHandler: Copy + Clone + Send + Sync {
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        axis: impl Axis,
        bots: AabbPin<&mut [T]>,
        is_leaf: bool,
    );

    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
    );
}

impl NodeHandler for crate::tree::build::NoSorter {
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        _: &mut PreVec,
        _axis: impl Axis,
        bots: AabbPin<&mut [T]>,
        _is_leaf: bool,
    ) {
        queries::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }

    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        _: &mut PreVec,
        mut anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
    ) {
        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.node.cont.intersects(&anchor.node.cont)
        };

        if res {
            for mut a in current.node.into_range().iter_mut() {
                for mut b in anchor.node.borrow_mut().into_range().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a.borrow_mut(), b.borrow_mut());
                    }
                }
            }
        }
    }
}

impl NodeHandler for crate::tree::build::DefaultSorter {
    #[inline(always)]
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        axis: impl Axis,
        bots: AabbPin<&mut [T]>,
        is_leaf: bool,
    ) {
        //
        // All bots belonging to a non leaf node are guarenteed to touch the divider.
        // Therfore, all bots intersect along one axis already. Because:
        //
        // If a contains x and b contains x then a intersects b.
        //
        oned::find_2d(prevec, axis, bots, func, is_leaf);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        mut anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
    ) {
        if !current.axis.is_equal_to(anchor.axis) {
            let cc1 = &anchor.node.cont;

            let cc2 = current.node.into_node_ref();

            let r1 = super::tools::get_section_mut(anchor.axis, cc2.range, cc1);

            let r2 = super::tools::get_section_mut(
                current.axis,
                anchor.node.borrow_mut().into_range(),
                cc2.cont,
            );

            oned::find_perp_2d1(current.axis, r1, r2, func);
        } else if current.node.cont.intersects(&anchor.node.cont) {
            /*
            oned::find_parallel_2d(
                &mut self.prevec1,
                current.axis.next(),
                current.node.into_range(),
                anchor.node.borrow_mut().into_range(),
                func,
            );
            */

            oned::find_parallel_2d(
                prevec,
                current.axis.next(),
                anchor.node.borrow_mut().into_range(),
                current.node.into_range(),
                func,
            );
        }
    }
}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec {
    vec: Vec<usize>,
}

impl Default for PreVec {
    fn default() -> Self {
        PreVec::new()
    }
}

impl PreVec {
    #[allow(dead_code)]
    #[inline(always)]
    pub fn new() -> PreVec {
        PreVec { vec: Vec::new() }
    }
    #[inline(always)]
    pub fn with_capacity(num: usize) -> PreVec {
        PreVec {
            vec: Vec::with_capacity(num),
        }
    }

    ///Take advantage of the big capacity of the original vec.
    pub fn extract_vec<'a, 'b, T>(&'a mut self) -> Vec<AabbPin<&'b mut T>> {
        let mut v = Vec::new();
        core::mem::swap(&mut v, &mut self.vec);
        revec::convert_empty_vec(v)
    }

    ///Return the big capacity vec
    pub fn insert_vec<T>(&mut self, vec: Vec<AabbPin<&'_ mut T>>) {
        let mut v = revec::convert_empty_vec(vec);
        core::mem::swap(&mut self.vec, &mut v)
    }
}
