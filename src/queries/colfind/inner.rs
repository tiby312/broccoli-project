use super::node_handle::*;
use crate::*;

use super::colfind::CollisionHandler;
struct InnerRecurser<'a, 'node, T: Aabb, NN, C, B: Axis> {
    anchor: NodeAxis<'a, 'node, T, B>,
    handler: &'a mut NN,
    sweeper: &'a mut C,
    prevec: &'a mut PreVec<T>,
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

        self.handler
            .handle_children(self.sweeper, self.prevec, self.anchor.borrow_mut(), current);

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

use crate::Joinable;
pub struct ParRecurser<'b, 'node, T: Aabb, NO, S, C, P, J> {
    pub handler: NO,
    pub vistr: SplitterVistr<C, SplitterVistr<S, VistrMut<'b, Node<'node, T>>>>,
    pub par: P,
    pub joiner: J,
    pub prevec: PreVec<T>,
}

impl<'b, 'node, T: Aabb, NO, S, C, P, J> ParRecurser<'b, 'node, T, NO, S, C, P, J>
where
    T: Send + Sync,
    T::Num: Send + Sync,
    S: Splitter + Send + Sync,
    NO: NodeHandler + Send + Sync,
    C: CollisionHandler<T = T> + Splitter + Send + Sync,
    P: parallel::Joiner,
    J: Joinable,
{
    pub fn recurse_par<A: Axis>(mut self, this_axis: A) -> (C, S) {
        let ((mut finisher_par, (finisher_seq, mut nn)), rest) = self.vistr.next();

        self.handler.handle_node(
            &mut finisher_par,
            &mut self.prevec,
            this_axis.next(),
            nn.borrow_mut().into_range(),
        );

        if let Some([mut left, mut right]) = rest {
            let mut g = InnerRecurser {
                anchor: NodeAxis {
                    node: nn,
                    axis: this_axis,
                },
                handler: &mut self.handler,
                sweeper: &mut finisher_par,
                prevec: &mut self.prevec,
            };

            g.recurse(this_axis.next(), left.inner.inner.borrow_mut());
            g.recurse(this_axis.next(), right.inner.inner.borrow_mut());

            match self.par.next() {
                parallel::ParResult::Parallel([dleft, dright]) => {
                    let p1 = ParRecurser {
                        handler: self.handler,
                        vistr: left,
                        par: dleft,
                        joiner: self.joiner.clone(),
                        prevec: self.prevec,
                    };

                    let p2 = ParRecurser {
                        handler: self.handler,
                        vistr: right,
                        par: dright,
                        joiner: self.joiner.clone(),
                        prevec: PreVec::new(),
                    };

                    let (sl, sr) = self.joiner.join(
                        |_joiner| p1.recurse_par(this_axis.next()),
                        |_joiner| p2.recurse_par(this_axis.next()),
                    );

                    (
                        finish_splitter(finisher_par, sl.0, sr.0),
                        finish_splitter(finisher_seq, sl.1, sr.1),
                    )
                }
                parallel::ParResult::Sequential(_) => {
                    let mut ar = Recurser {
                        handler: &mut self.handler,
                        sweeper: &mut finisher_par,
                        prevec: &mut self.prevec,
                    };
                    let ar = ar.recurse_seq(this_axis.next(), left.inner);

                    let mut al = Recurser {
                        handler: &mut self.handler.clone(),
                        sweeper: &mut finisher_par,
                        prevec: &mut self.prevec,
                    };

                    let al = al.recurse_seq(this_axis.next(), right.inner);
                    (finisher_par, finish_splitter(finisher_seq, al, ar))
                }
            }
        } else {
            (finisher_par, finisher_seq)
        }
    }
}

pub struct Recurser<'a, T: Aabb, NO, C> {
    pub handler: &'a mut NO,
    pub sweeper: &'a mut C,
    pub prevec: &'a mut PreVec<T>,
}

impl<'a, T: Aabb, NO, C> Recurser<'a, T, NO, C>
where
    NO: NodeHandler,
    C: CollisionHandler<T = T>,
{
    pub fn recurse_seq<A: Axis, S: Splitter>(
        &mut self,
        this_axis: A,
        vistr: SplitterVistr<S, VistrMut<Node<T>>>,
    ) -> S {
        let ((finisher, mut nn), rest) = vistr.next();

        self.handler.handle_node(
            self.sweeper,
            self.prevec,
            this_axis.next(),
            nn.borrow_mut().into_range(),
        );

        if let Some([mut left, mut right]) = rest {
            {
                let mut g = InnerRecurser {
                    anchor: NodeAxis {
                        node: nn,
                        axis: this_axis,
                    },
                    handler: self.handler,
                    sweeper: self.sweeper,
                    prevec: self.prevec,
                };

                g.recurse(this_axis.next(), left.inner.borrow_mut());
                g.recurse(this_axis.next(), right.inner.borrow_mut());
            }

            let al = self.recurse_seq(this_axis.next(), left);
            let ar = self.recurse_seq(this_axis.next(), right);
            finish_splitter(finisher, al, ar)
        } else {
            finisher
        }
    }
}

#[inline(always)]
fn finish_splitter<S: Splitter>(mut a: S, b: S, c: S) -> S {
    a.add(b, c);
    a
}

pub struct SplitterVistr<S, V> {
    pub inner: V,
    pub splitter: S,
}
impl<S: Splitter, V: Visitor> SplitterVistr<S, V> {
    #[inline(always)]
    pub fn new(splitter: S, inner: V) -> Self {
        SplitterVistr { inner, splitter }
    }
}
impl<S: Splitter, V: Visitor> Visitor for SplitterVistr<S, V> {
    type Item = (S, V::Item);
    fn next(mut self) -> (Self::Item, Option<[Self; 2]>) {
        let (a, rest) = self.inner.next();

        if let Some([left, right]) = rest {
            let (d1, d2) = self.splitter.div();
            (
                (self.splitter, a),
                Some([
                    SplitterVistr {
                        inner: left,
                        splitter: d1,
                    },
                    SplitterVistr {
                        inner: right,
                        splitter: d2,
                    },
                ]),
            )
        } else {
            ((self.splitter, a), None)
        }
    }
}
