use super::node_handle::*;
use crate::inner_prelude::*;

struct InnerRecurser<'a, 'b: 'b, T: Aabb, NN: NodeHandler<T = T>, B: Axis> {
    anchor: DestructuredNode<'a, 'b, T, B>,
    sweeper: &'a mut NN,
}

impl<'a, 'b: 'a, T: Aabb, NN: NodeHandler<T = T>, B: Axis> InnerRecurser<'a, 'b, T, NN, B> {
    #[inline(always)]
    fn new(
        anchor: DestructuredNode<'a, 'b, T, B>,
        sweeper: &'a mut NN,
    ) -> InnerRecurser<'a, 'b, T, NN, B> {
        InnerRecurser { anchor, sweeper }
    }

    fn recurse<
        A: Axis, //this axis
    >(
        &mut self,
        this_axis: A,
        m: VistrMut<Node<T>>,
    ) {
        let anchor_axis = self.anchor.axis;
        let (nn, rest) = m.next();
        match rest {
            Some([left, right]) => {
                //Continue to recurse even if we know there are no more bots
                //This simplifies query algorithms that might be building up
                //a tree.
                if let Some(div) = nn.div {
                    if let Some(current) = DestructuredNodeLeaf::new(this_axis, nn) {
                        self.sweeper.handle_children(&mut self.anchor, current);
                    }

                    if anchor_axis.is_equal_to(this_axis) {
                        use core::cmp::Ordering::*;
                        match self.anchor.cont().contains_ext(div) {
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
            None => {
                if let Some(current) = DestructuredNodeLeaf::new(this_axis, nn) {
                    self.sweeper.handle_children(&mut self.anchor, current);
                }
            }
        }
    }
}

pub struct ColFindRecurser<T: Aabb, K: Splitter, S: NodeHandler<T = T> + Splitter> {
    _p: PhantomData<(T, K, S)>,
}
impl<
        T: Aabb + Send + Sync,
        K: Splitter + Send + Sync,
        S: NodeHandler<T = T> + Splitter + Send + Sync,
    > ColFindRecurser<T, K, S>
where
    T::Num: Send + Sync,
{
    pub fn recurse_par<A: Axis, JJ: par::Joiner>(
        &self,
        this_axis: A,
        par: JJ,
        sweeper: &mut S,
        m: VistrMut<Node<T>>,
        splitter: &mut K,
    ) {
        if let Some([left, right]) = Self::recurse_common(this_axis, sweeper, m) {
            let (mut splitter11, mut splitter22) = splitter.div();
            match par.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let (mut sweeper1, mut sweeper2) = sweeper.div();

                    rayon::join(
                        || {
                            self.recurse_par(
                                this_axis.next(),
                                dleft,
                                &mut sweeper1,
                                left,
                                &mut splitter11,
                            )
                        },
                        || {
                            self.recurse_par(
                                this_axis.next(),
                                dright,
                                &mut sweeper2,
                                right,
                                &mut splitter22,
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

impl<T: Aabb, K: Splitter, S: NodeHandler<T = T> + Splitter> ColFindRecurser<T, K, S> {
    #[inline(always)]
    pub fn new() -> ColFindRecurser<T, K, S> {
        ColFindRecurser { _p: PhantomData }
    }

    pub fn recurse_common<'a, 'b, A: Axis>(
        this_axis: A,
        sweeper: &mut S,
        m: VistrMut<'b, Node<'a, T>>,
    ) -> Option<[VistrMut<'b, Node<'a, T>>; 2]> {
        let (mut nn, rest) = m.next();

        match rest {
            Some([mut left, mut right]) => {
                //Continue to recurse even if we know there are no more bots
                //This simplifies query algorithms that might be building up
                //a tree.
                if nn.div.is_some() {
                    sweeper.handle_node(this_axis.next(), nn.borrow_mut().into_range());

                    if let Some(nn) = DestructuredNode::new(this_axis, nn) {
                        let left = left.borrow_mut();
                        let right = right.borrow_mut();
                        let mut g = InnerRecurser::new(nn, sweeper);
                        g.recurse(this_axis.next(), left);
                        g.recurse(this_axis.next(), right);
                    }
                }

                Some([left, right])
            }
            None => {
                sweeper.handle_node(this_axis.next(), nn.into_range());
                None
            }
        }
    }

    pub fn recurse_seq<A: Axis>(
        &self,
        this_axis: A,
        sweeper: &mut S,
        m: VistrMut<Node<T>>,
        splitter: &mut K,
    ) {
        if let Some([left, right]) = Self::recurse_common(this_axis, sweeper, m) {
            let (mut splitter11, mut splitter22) = splitter.div();
            self.recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
            self.recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);

            splitter.add(splitter11, splitter22);
        }
    }
}
