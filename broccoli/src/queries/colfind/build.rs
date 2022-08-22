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
    vistr: VistrMutPin<'b, Node<'a, T, T::Num>>,
    axis: AxisDyn,
}
impl<'a, 'b, T: Aabb> CollVis<'a, 'b, T> {
    pub fn new(vistr: VistrMutPin<'b, Node<'a, T, T::Num>>) -> Self {
        CollVis {
            vistr,
            axis: default_axis().to_dyn(),
        }
    }

    pub fn get_height(&self) -> usize {
        self.vistr.get_height()
    }

    pub fn num_elem(&self) -> usize {
        let (n, _) = self.vistr.borrow().next();
        n.min_elem
    }
    pub fn collide_and_next<N: NodeHandler<T>>(
        mut self,
        handler: &mut N,
    ) -> (NodeFinisher<'b, T>, Option<[Self; 2]>) {
        handler.handle_nodes_under(self.axis, self.vistr.borrow_mut());

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
        let (n, rest) = self.collide_and_next(handler);

        n.finish(handler);
        if let Some([a, b]) = rest {
            a.recurse_seq(handler);
            b.recurse_seq(handler);
        }
    }
}

///
/// Abstract over sorted and non sorted trees
///
pub trait NodeHandler<T: Aabb> {
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool);

    // implementer responsibility to check if it is a leaf or not.
    fn handle_nodes_under(&mut self, this_axis: AxisDyn, m: VistrMutPin<Node<T, T::Num>>);
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
