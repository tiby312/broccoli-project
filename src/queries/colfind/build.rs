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
pub struct NodeFinisher<'b, T> {
    axis: AxisDyn,
    bots: AabbPin<&'b mut [T]>,
    is_leaf: bool,
}
impl<'b, T: Aabb> NodeFinisher<'b, T> {
    pub fn finish<H: NodeHandler<T>>(self, handler: &mut H) {
        handler.handle_node(self.axis, self.bots, self.is_leaf);
    }
}

/// The main primitive to visit each node and find colliding pairs
pub struct CollVis<'a, 'b, T: Aabb> {
    vistr: VistrMutPin<'b, Node<'a, T>>,
    axis: AxisDyn,
}
impl<'a, 'b, T: Aabb> CollVis<'a, 'b, T> {
    pub(crate) fn new(vistr: VistrMutPin<'b, Node<'a, T>>, axis: AxisDyn) -> Self {
        CollVis { vistr, axis }
    }

    pub fn get_height(&self) -> usize {
        self.vistr.get_height()
    }

    pub fn num_elem(&self) -> usize {
        let (n, _) = self.vistr.borrow().next();
        n.num_elem
    }
    pub fn collide_and_next<N: NodeHandler<T>>(
        mut self,
        handler: &mut N,
    ) -> (NodeFinisher<'b, T>, Option<[Self; 2]>) {
        {
            let this_axis = self.axis;
            let (nn, rest) = self.vistr.borrow_mut().next();

            if let Some([mut left, mut right]) = rest {
                struct InnerRecurser<'a, T: Aabb, NN> {
                    anchor: DNode<'a, T>,
                    anchor_axis: AxisDyn,
                    handler: &'a mut NN,
                }

                impl<'a, T: Aabb, NN> InnerRecurser<'a, T, NN>
                where
                    NN: NodeHandler<T>,
                {
                    fn recurse(
                        &mut self,
                        this_axis: AxisDyn,
                        m: VistrMutPin<Node<T>>,
                        is_left: bool,
                    ) {
                        let anchor_axis = self.anchor_axis;
                        let current_is_leaf = m.get_height() == 1;

                        let (mut nn, rest) = m.next();

                        self.handler.handle_children(HandleChildrenArgs {
                            anchor: self.anchor.borrow(),
                            anchor_axis: self.anchor_axis,
                            current: nn.borrow_mut().into_node_ref(),
                            current_axis: this_axis,
                            current_is_leaf,
                        });

                        if let Some([left, right]) = rest {
                            if let Some(div) = nn.div {
                                if anchor_axis.is_equal_to(this_axis) {
                                    match is_left {
                                        true => {
                                            if div < self.anchor.cont.start {
                                                self.recurse(this_axis.next(), right, is_left);
                                                return;
                                            }
                                        }
                                        false => {
                                            if div >= self.anchor.cont.end {
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

                if let Some(div) = nn.div {
                    let d = nn.into_node_ref();
                    let mut g = InnerRecurser {
                        anchor: DNode {
                            div,
                            cont: d.cont,
                            range: d.range,
                        },
                        anchor_axis: this_axis,
                        handler,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
                }
            }
        }

        //TODO make height be zero for leaf?
        let is_leaf = self.get_height() == 1;

        let (n, rest) = self.vistr.next();

        let fin = NodeFinisher {
            axis: self.axis,
            bots: n.into_range(),
            is_leaf,
        };

        (
            fin,
            if let Some([left, right]) = rest {
                Some([
                    CollVis {
                        vistr: left,
                        axis: self.axis.next(),
                    },
                    CollVis {
                        vistr: right,
                        axis: self.axis.next(),
                    },
                ])
            } else {
                None
            },
        )
    }

    pub fn recurse_seq<N: NodeHandler<T>>(self, handler: &mut N) {
        self.recurse_seq_inner(handler)
    }

    fn recurse_seq_inner<N: NodeHandler<T>>(self, handler: &mut N) {
        let (n, rest) = self.collide_and_next(handler);

        n.finish(handler);
        if let Some([a, b]) = rest {
            a.recurse_seq_inner(handler);
            b.recurse_seq_inner(handler);
        }
    }
}

//remove need for second lifetime
pub struct HandleChildrenArgs<'a, T: Aabb> {
    pub anchor: DNode<'a, T>,
    pub current: NodeRef<'a, T>,
    pub anchor_axis: AxisDyn,
    pub current_axis: AxisDyn,
    pub current_is_leaf: bool,
}

///
/// Abstract over sorted and non sorted trees
///
pub trait NodeHandler<T: Aabb> {
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool);

    fn handle_children(&mut self, floop: HandleChildrenArgs<T>);
}

pub struct DNode<'a, T: Aabb> {
    pub div: T::Num,
    pub cont: &'a Range<T::Num>,
    pub range: AabbPin<&'a mut [T]>,
}
impl<'a, T: Aabb> DNode<'a, T> {
    fn borrow(&mut self) -> DNode<T> {
        DNode {
            div: self.div,
            cont: self.cont,
            range: self.range.borrow_mut(),
        }
    }
}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Clone)]
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
