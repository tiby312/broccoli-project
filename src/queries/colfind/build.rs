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
