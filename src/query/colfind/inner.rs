use super::node_handle::*;
use crate::inner_prelude::*;

use crate::query::colfind::CollisionHandler;
struct InnerRecurser<'a, 'b, T: Aabb, NN: NodeHandler, KK: CollisionHandler<T = T>, B: Axis> {
    anchor: NodeAxis<'a, 'b, T, B>,
    recc: &'a mut ColfindRecurser<T, NN, KK>,
}

impl<'a, 'b, T: Aabb, NN: NodeHandler, KK: CollisionHandler<T = T>, B: Axis>
    InnerRecurser<'a, 'b, T, NN, KK, B>
{
    #[inline(always)]
    fn new(
        anchor: NodeAxis<'a, 'b, T, B>,
        recc: &'a mut ColfindRecurser<T, NN, KK>,
    ) -> InnerRecurser<'a, 'b, T, NN, KK, B> {
        InnerRecurser { anchor, recc }
    }

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

        self.recc.handler.handle_children(
            &mut self.recc.sweeper,
            &mut self.recc.prevec,
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

pub struct ColfindRecurser<T: Aabb, NO: NodeHandler, C: CollisionHandler<T = T>> {
    prevec: PreVec<T>,
    handler: NO,
    sweeper: C,
}

impl<T: Aabb, NO: NodeHandler, C: CollisionHandler<T = T>> ColfindRecurser<T, NO, C> {
    #[inline(always)]
    pub fn new(handler: NO, sweeper: C) -> ColfindRecurser<T, NO, C> {
        ColfindRecurser {
            handler,
            prevec: PreVec::with_capacity(64),
            sweeper,
        }
    }

    pub fn recurse_common<'a, 'b,A:Axis>(
        &mut self,
        this_axis: A,
        m: VistrMut<'b, Node<'a, T>>,
    ) -> Option<[VistrMut<'b, Node<'a, T>>; 2]> {
        let (mut nn, rest) = m.next();

        self.handler.handle_node(
            &mut self.sweeper,
            &mut self.prevec,
            this_axis.next(),
            nn.borrow_mut().into_range(),
        );

        if let Some([mut left, mut right]) = rest {
            {
                let nn = NodeAxis {
                    node: nn,
                    axis: this_axis,
                };

                let mut g = InnerRecurser::new(nn, self);
                g.recurse(this_axis.next(), left.borrow_mut());
                g.recurse(this_axis.next(), right.borrow_mut());
            }

            Some([left, right])
        } else {
            None
        }
    }

    pub fn recurse_seq<A:Axis,S: Splitter>(
        &mut self,
        this_axis: A,
        m: VistrMut<Node<T>>,
        mut splitter: S,
    ) -> S {
        if let Some([left, right]) = self.recurse_common(this_axis, m) {
            let (splitter11, splitter22) = splitter.div();
            let ls = self.recurse_seq(this_axis.next(), left, splitter11);
            let rs = self.recurse_seq(this_axis.next(), right, splitter22);
            splitter.add(ls, rs);
        }
        splitter
    }

}


pub struct ParRecurser<T: Aabb, NO: NodeHandler, C: CollisionHandler<T = T>,P:par::Joiner,J:crate::Joinable>{
    pub inner:ColfindRecurser<T,NO,C>,
    pub par:P,
    pub joiner:J,
}

impl<T: Aabb, NO: NodeHandler, C: CollisionHandler<T = T>,P:par::Joiner,J:crate::Joinable> ParRecurser<T,NO,C,P,J>
    where
    T: Send + Sync,
    T::Num: Send + Sync,
    C: Splitter + Send + Sync
{

    pub fn recurse_par<A:Axis,S: Splitter + Send + Sync>(
        mut self,
        this_axis: A,
        m: VistrMut<Node<T>>,
        mut splitter: S,
    ) -> (C, S)
    {
        if let Some([left, right]) = self.inner.recurse_common(this_axis, m) {
            let (splitter11, splitter22) = splitter.div();
            let (sl, sr) = match self.par.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let (sweeper1, sweeper2) = self.inner.sweeper.div();
                    let c1 = ParRecurser{
                        inner:ColfindRecurser::new(self.inner.handler, sweeper1),
                        par:dleft,
                        joiner:self.joiner.clone()
                    };
                    let c2 = ParRecurser{
                        inner:ColfindRecurser::new(self.inner.handler, sweeper2),
                        par:dright,
                        joiner:self.joiner.clone()
                    };

                    let (sl, sr) = self.joiner.join(
                        |_joiner| {
                            c1.recurse_par(
                                this_axis.next(),
                                left,
                                splitter11,
                            )
                        },
                        |_joiner| {
                            c2.recurse_par(
                                this_axis.next(),
                                right,
                                splitter22,
                            )
                        },
                    );

                    self.inner.sweeper.add(sl.0, sr.0);
                    (sl.1, sr.1)
                }
                par::ParResult::Sequential(_) => {
                    let sl = self.inner.recurse_seq(this_axis.next(), left, splitter11);
                    let sr = self.inner.recurse_seq(this_axis.next(), right, splitter22);
                    (sl, sr)
                }
            };

            splitter.add(sl, sr);
        }
        (self.inner.sweeper, splitter)
    }
}