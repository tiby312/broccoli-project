//! Contains code to customize the colliding pair finding algorithm.

use super::*;

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

    pub fn next(
        mut self,
        prevec: &mut PreVec,
        func: impl FnMut(PMut<T>, PMut<T>),
    ) -> Option<[Self; 2]> {
        pub struct Recurser<'a, NO, C> {
            pub handler: &'a mut NO,
            pub sweeper: &'a mut C,
            pub prevec: &'a mut PreVec,
        }

        struct QueryFnMut<T, F>(F, PhantomData<T>);
        impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> QueryFnMut<T, F> {
            #[inline(always)]
            pub fn new(func: F) -> QueryFnMut<T, F> {
                QueryFnMut(func, PhantomData)
            }
        }

        impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> CollisionHandler for QueryFnMut<T, F> {
            type T = T;
            #[inline(always)]
            fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
                self.0(a, b);
            }
        }

        fn collide_self<A: axgeom::Axis, T: crate::Aabb>(
            this_axis: A,
            v: VistrMut<Node<T>>,
            data: &mut Recurser<impl NodeHandler, impl CollisionHandler<T = T>>,
        ) {
            let (mut nn, rest) = v.next();

            data.handler.handle_node(
                data.sweeper,
                data.prevec,
                this_axis.next(),
                nn.borrow_mut().into_range(),
            );

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
                    C: CollisionHandler<T = T>,
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

        let mut g = QueryFnMut::new(func);
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

        let (_, rest) = self.vistr.next();

        if let Some([left, right]) = rest {
            Some([
                CollVis {
                    vistr: left,
                    is_xaxis: !self.is_xaxis,
                    handler: self.handler.clone(),
                },
                CollVis {
                    vistr: right,
                    is_xaxis: !self.is_xaxis,
                    handler: self.handler.clone(),
                },
            ])
        } else {
            None
        }
    }

    pub fn recurse_seq(self, prevec: &mut PreVec, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        let mut stack = vec![];
        stack.push(self);

        while let Some(n) = stack.pop() {
            if let Some([a, b]) = n.next(prevec, &mut func) {
                stack.push(a);
                stack.push(b);
            }
        }
    }

    pub fn recurse_seq_splitter<S: Splitter>(
        self,
        mut splitter: S,
        prevec: &mut PreVec,
        mut func: impl FnMut(PMut<T>, PMut<T>),
    ) -> S {
        #[inline(always)]
        fn finish_splitter<S: Splitter>(mut a: S, b: S, c: S) -> S {
            a.add(b, c);
            a
        }

        if let Some([left, right]) = self.next(prevec, &mut func) {
            let (s1, s2) = splitter.div();
            let al = left.recurse_seq_splitter(s1, prevec, &mut func);
            let ar = right.recurse_seq_splitter(s2, prevec, &mut func);
            finish_splitter(splitter, al, ar)
        } else {
            splitter
        }
    }
}
