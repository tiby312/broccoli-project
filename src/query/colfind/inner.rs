use super::node_handle::*;
use crate::inner_prelude::*;

use crate::query::colfind::CollisionHandler;
struct InnerRecurser<'a, 'b: 'b, T: Aabb, NN: NodeHandler, KK:CollisionHandler<T=T>,B: Axis> {
    anchor: DestructuredNode<'a, 'b, T, B>,
    handler: NN,
    sweeper:&'a mut KK
}

impl<'a, 'b: 'a, T: Aabb, NN: NodeHandler,KK:CollisionHandler<T=T>, B: Axis> InnerRecurser<'a, 'b, T, NN,KK, B> {
    #[inline(always)]
    fn new(
        anchor: DestructuredNode<'a, 'b, T, B>,
        sweeper: &'a mut KK,
        handler:NN,
    ) -> InnerRecurser<'a, 'b, T, NN,KK, B> {
        InnerRecurser { anchor, sweeper,handler }
    }

    fn recurse<
        A: Axis, //this axis
    >(
        &mut self,
        this_axis: A,
        prevec:&mut PreVecMut<T>, //TODO put this in the struct?
        m: VistrMut<Node<T>>,
    ) {
        let anchor_axis = self.anchor.axis;
        let (mut nn, rest) = m.next();
        if !nn.range.is_empty(){
            let current=DestructuredNodeLeaf{
                node:nn.borrow_mut(),
                axis:this_axis
            };

            self.handler.handle_children(self.sweeper,prevec,&mut self.anchor, current);
        }

        if let Some([left,right])=rest{
            //Continue to recurse even if we know there are no more bots
            //This simplifies query algorithms that might be building up
            //a tree.
            if let Some(div) = nn.div {
                

                if anchor_axis.is_equal_to(this_axis) {
                    use core::cmp::Ordering::*;
                    match self.anchor.node.cont.contains_ext(div) {
                        Less => {
                            self.recurse(this_axis.next(),prevec, right);
                            return;
                        }
                        Greater => {
                            self.recurse(this_axis.next(),prevec, left);
                            return;
                        }
                        Equal => {}
                    }
                }
            }

            self.recurse(this_axis.next(),prevec, left);
            self.recurse(this_axis.next(),prevec, right);
        }
    }
}

pub fn recurse_par<T:Aabb+Send+Sync>(
    this_axis: impl Axis,
    par: impl par::Joiner,
    sweeper: &mut (impl CollisionHandler<T=T> + Splitter+Send+Sync),
    handler: impl NodeHandler,
    prevec:&mut PreVecMut<T>,
    m: VistrMut<Node<T>>,
    splitter: &mut (impl Splitter+Send+Sync),
) where T::Num:Send+Sync{
    if let Some([left, right]) = recurse_common(this_axis,prevec, sweeper,handler, m) {
        let (mut splitter11, mut splitter22) = splitter.div();
        match par.next() {
            par::ParResult::Parallel([dleft, dright]) => {
                let (mut sweeper1, mut sweeper2) = sweeper.div();
                let mut prevec2=PreVecMut::new();
                rayon::join(
                    || {
                        recurse_par(
                            this_axis.next(),
                            dleft,
                            &mut sweeper1,
                            handler,
                            prevec,
                            left,
                            &mut splitter11,
                        )
                    },
                    || {
                        recurse_par(
                            this_axis.next(),
                            dright,
                            &mut sweeper2,
                            handler,
                            &mut prevec2,
                            right,
                            &mut splitter22,
                        )
                    },
                );

                sweeper.add(sweeper1, sweeper2);
            }
            par::ParResult::Sequential(_) => {
                recurse_seq(this_axis.next(), sweeper,handler, prevec,left, &mut splitter11);
                recurse_seq(this_axis.next(), sweeper,handler, prevec,right, &mut splitter22);
            }
        }

        splitter.add(splitter11, splitter22);
    }
}


pub fn recurse_seq<T:Aabb>(
    this_axis: impl Axis,
    sweeper: &mut impl CollisionHandler<T=T>,
    handler: impl NodeHandler,
    prevec:&mut PreVecMut<T>,
    m: VistrMut<Node<T>>,
    splitter: &mut impl Splitter,
) {
    if let Some([left, right]) = recurse_common(this_axis,prevec, sweeper,handler, m) {
        let (mut splitter11, mut splitter22) = splitter.div();
        recurse_seq(this_axis.next(), sweeper, handler,prevec,left, &mut splitter11);
        recurse_seq(this_axis.next(), sweeper, handler,prevec,right, &mut splitter22);

        splitter.add(splitter11, splitter22);
    }
}

pub fn recurse_common<'a, 'b,T:Aabb>(
    this_axis: impl Axis,
    prevec:&mut PreVecMut<T>,
    sweeper: &mut impl CollisionHandler<T=T>,
    handler: impl NodeHandler,
    m: VistrMut<'b, Node<'a, T>>
) -> Option<[VistrMut<'b, Node<'a, T>>; 2]> {
    let (mut nn, rest) = m.next();

    match rest {
        Some([mut left, mut right]) => {
            //Continue to recurse even if we know there are no more bots
            //This simplifies query algorithms that might be building up
            //a tree.
            if nn.div.is_some() {
                handler.handle_node(sweeper,prevec,this_axis.next(), nn.borrow_mut().into_range());

                //TODO get rid of this check???
                if !nn.range.is_empty(){
                    let nn=DestructuredNode{
                        node:nn,
                        axis:this_axis
                    };
                    let left = left.borrow_mut();
                    let right = right.borrow_mut();
                    let mut g = InnerRecurser::new(nn, sweeper,handler);
                    g.recurse(this_axis.next(),prevec, left);
                    g.recurse(this_axis.next(),prevec, right);
                }
            }

            Some([left, right])
        }
        None => {
            //TODO combine this with above
            handler.handle_node(sweeper,prevec,this_axis.next(), nn.into_range());
            None
        }
    }
}
