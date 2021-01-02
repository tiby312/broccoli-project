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
                    if !nn.range.is_empty(){
                        let current=DestructuredNodeLeaf{
                            node:nn,
                            axis:this_axis
                        };

                        self.sweeper.handle_children(&mut self.anchor, current);
                    }

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
            None => {
                if !nn.range.is_empty(){
                    let current=DestructuredNodeLeaf{
                        node:nn,
                        axis:this_axis
                    };

                    self.sweeper.handle_children(&mut self.anchor, current);
                }
            }
        }
    }
}

pub fn recurse_par<T:Aabb+Send+Sync>(
    this_axis: impl Axis,
    par: impl par::Joiner,
    sweeper: &mut (impl NodeHandler<T = T> + Splitter+Send+Sync),
    m: VistrMut<Node<T>>,
    splitter: &mut (impl Splitter+Send+Sync),
) where T::Num:Send+Sync{
    if let Some([left, right]) = recurse_common(this_axis, sweeper, m) {
        let (mut splitter11, mut splitter22) = splitter.div();
        match par.next() {
            par::ParResult::Parallel([dleft, dright]) => {
                let (mut sweeper1, mut sweeper2) = sweeper.div();

                rayon::join(
                    || {
                        recurse_par(
                            this_axis.next(),
                            dleft,
                            &mut sweeper1,
                            left,
                            &mut splitter11,
                        )
                    },
                    || {
                        recurse_par(
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
                recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
                recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);
            }
        }

        splitter.add(splitter11, splitter22);
    }
}


pub fn recurse_seq<T:Aabb>(
    this_axis: impl Axis,
    sweeper: &mut (impl NodeHandler<T=T> + Splitter),
    m: VistrMut<Node<T>>,
    splitter: &mut impl Splitter,
) {
    if let Some([left, right]) = recurse_common(this_axis, sweeper, m) {
        let (mut splitter11, mut splitter22) = splitter.div();
        recurse_seq(this_axis.next(), sweeper, left, &mut splitter11);
        recurse_seq(this_axis.next(), sweeper, right, &mut splitter22);

        splitter.add(splitter11, splitter22);
    }
}

pub fn recurse_common<'a, 'b,T:Aabb>(
    this_axis: impl Axis,
    sweeper: &mut (impl NodeHandler<T=T> + Splitter),
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

                //TODO get rid of this check???
                if !nn.range.is_empty(){
                    let nn=DestructuredNode{
                        node:nn,
                        axis:this_axis
                    };
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
