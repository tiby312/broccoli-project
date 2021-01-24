use super::node_handle::*;
use crate::inner_prelude::*;

use crate::query::colfind::CollisionHandler;
struct InnerRecurser<'a, 'b, T: Aabb, NN: NodeHandler, KK: CollisionHandler<T = T>, B: Axis> {
    anchor: DestructuredNode<'a, 'b, T, B>,
    handler: NN,
    sweeper: &'a mut KK,
    prevec: &'a mut PreVec<T>,
}

impl<'a, 'b, T: Aabb, NN: NodeHandler, KK: CollisionHandler<T = T>, B: Axis>
    InnerRecurser<'a, 'b, T, NN, KK, B>
{
    #[inline(always)]
    fn new(
        anchor: DestructuredNode<'a, 'b, T, B>,
        sweeper: &'a mut KK,
        handler: NN,
        prevec: &'a mut PreVec<T>,
    ) -> InnerRecurser<'a, 'b, T, NN, KK, B> {
        InnerRecurser {
            anchor,
            sweeper,
            handler,
            prevec,
        }
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
        //if !nn.range.is_empty() {
        let current = DestructuredNodeLeaf {
            node: nn.borrow_mut(),
            axis: this_axis,
        };

        self.handler
            .handle_children(self.sweeper, self.prevec, &mut self.anchor, current);
        //}

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

//TODO give this the same treatment as build did.

pub struct ColfindRecurser<T: Aabb, NO: NodeHandler> {
    prevec: PreVec<T>,
    handler: NO,
}

impl<T: Aabb, NO: NodeHandler> ColfindRecurser<T, NO> {
    pub fn new(handler: NO) -> ColfindRecurser<T, NO> {
        ColfindRecurser {
            handler,
            prevec: PreVec::with_capacity(64),
        }
    }

    pub fn recurse_common<'a, 'b>(
        &mut self,
        this_axis: impl Axis,
        sweeper: &mut impl CollisionHandler<T = T>,
        m: VistrMut<'b, Node<'a, T>>,
    ) -> Option<[VistrMut<'b, Node<'a, T>>; 2]> {
        let (mut nn, rest) = m.next();

        self.handler.handle_node(
            sweeper,
            &mut self.prevec,
            this_axis.next(),
            nn.borrow_mut().into_range(),
        );

        if let Some([mut left, mut right]) = rest {
            {
                let nn = DestructuredNode {
                    node: nn,
                    axis: this_axis,
                };

                let mut g = InnerRecurser::new(nn, sweeper, self.handler, &mut self.prevec);
                g.recurse(this_axis.next(), left.borrow_mut());
                g.recurse(this_axis.next(), right.borrow_mut());
            }

            Some([left, right])
        } else {
            None
        }
    }

    pub fn recurse_seq(
        &mut self,
        this_axis: impl Axis,
        sweeper: &mut impl CollisionHandler<T = T>,
        m: VistrMut<Node<T>>,
        splitter: &mut impl Splitter,
    ) {
        if let Some([left, right]) = self.recurse_common(this_axis, sweeper, m) {
            let (mut splitter11, mut splitter22) = splitter.div();
            self.recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
            self.recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);

            splitter.add(splitter11, splitter22);
        }
    }

    pub fn recurse_par(
        &mut self,
        this_axis: impl Axis,
        par: impl par::Joiner,
        sweeper: &mut (impl CollisionHandler<T = T> + Splitter + Send + Sync),
        m: VistrMut<Node<T>>,
        splitter: &mut (impl Splitter + Send + Sync),
        joiner: impl crate::Joinable,
    ) where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        if let Some([left, right]) = self.recurse_common(this_axis, sweeper, m) {
            let (mut splitter11, mut splitter22) = splitter.div();
            match par.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let (mut sweeper1, mut sweeper2) = sweeper.div();
                    let mut c = ColfindRecurser::new(self.handler);

                    joiner.join(
                        |joiner| {
                            self.recurse_par(
                                this_axis.next(),
                                dleft,
                                &mut sweeper1,
                                left,
                                &mut splitter11,
                                joiner.clone(),
                            )
                        },
                        |joiner| {
                            c.recurse_par(
                                this_axis.next(),
                                dright,
                                &mut sweeper2,
                                right,
                                &mut splitter22,
                                joiner.clone(),
                            )
                        },
                    );

                    sweeper.add(sweeper1, sweeper2);
                }
                par::ParResult::Sequential(_) => {
                    self.recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
                    self.recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);
                }
            }

            splitter.add(splitter11, splitter22);
        }
    }
}
