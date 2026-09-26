//!
//! Building blocks to find colliding pairs with trees
//!

use super::*;


// pub struct Double<'a, T> {
//     pub(crate) a: &'a mut T,
//     pub(crate) b: &'a mut T,
// }
// impl<'a,T> Double<'a,T>{
//     pub fn split(self)->(Single<'a,T>,Single<'a,T>){
//         (Single{inner:self.a},Single{inner:self.b})
//     }
// }

#[derive(Clone,Copy)]
pub struct UserCollider<F>(pub F);

impl<T: Aabb, F> InnerCollider<T> for UserCollider<F>
where
    F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: &mut T, b: &mut T) {
        (self.0)(AabbPin { inner: a }, AabbPin { inner: b });
    }
}

// impl<'a, T: HasInner> Double<'a, T> {
//     pub fn unpack_mut<'b>(&'b mut self) -> (T::Inner<'b>, T::Inner<'b>) {
//         (self.a.inner(), self.b.inner())
//     }
//     pub fn unpack(self) -> (T::Inner<'a>, T::Inner<'a>) {
//         (self.a.inner(), self.b.inner())
//     }

// }

pub struct MyCollider<F>(pub F);
impl<T: Unpack + Aabb, F> InnerCollider<T> for MyCollider<F>
where
    F: for<'b> FnMut(T::Inner<'b>, T::Inner<'b>),
{
    fn collide(&mut self, a: &mut T, b: &mut T) {
        (self.0)(a.inner(), b.inner());
    }
}
// impl<'a, T: Aabb> Collision<'a, T> {
//     pub fn rects(&self)->(  &Rect<T::Num>, &Rect<T::Num>) {
//         (self.a.get(), self.b.get())
//     }

// }

pub trait InnerCollider<T: Aabb> {
    fn collide(&mut self, a: &mut T, b: &mut T);
}

pub struct Simple<F>(pub F);
impl<T: Aabb, F> InnerCollider<T> for Simple<F>
where
    F: FnMut(&mut T, &mut T),
{
    fn collide(&mut self, a: &mut T, b: &mut T) {
        (self.0)(a, b);
    }
}

// pub fn only_inner<T:Aabb+HasInner,F>(func:F)->OnlyInner<F> where F: FnMut(&mut T::Inner, &mut T::Inner){
//     OnlyInner(func)
// }

// pub struct OnlyInner<F>(pub F);
// impl<T:Aabb+HasInner,F> InnerCollider<T> for OnlyInner<F> where T:Aabb+HasInner, F: FnMut(&mut T::Inner, &mut T::Inner){
//     fn collide(&mut self, a: &mut T, b: &mut T) {
//         (self.0)(a.inner().1, b.inner().1);
//     }
// }

// pub fn make_collide_fn<N:Num,T:Aabb<Num=N>+HasInner,F:FnMut(T::Inner<'_>, T::Inner<'_>)>(mut func: F) -> OneDCollidier2<F> {
//     OneDCollidier2(func)
// }

// pub struct OneDCollidier2<F>(pub F);

// impl<T: Aabb+HasInner, F: for<'a> FnMut(T::Inner<'a>, T::Inner<'a>)> InnerCollider<T> for OneDCollidier2<F> {
//     fn collide(&mut self, a: &mut T, b: &mut T) {
//         (self.0)(a.inner(), b.inner());
//     }
// }

// pub struct OneDCollidier<F>(pub F);

// impl<T: Aabb+HasInner, F: CollisionHandler<T>> InnerCollider<T> for OneDCollidier<F> {
//     fn collide(&mut self, a: &mut T, b: &mut T) {
//         self.0.collide(a.inner(), b.inner());
//     }
// }

// ///
// /// Shorthand for `FnMut(AabbPin<&mut T>, AabbPin<&mut T>)` trait bound
// ///
// pub trait CollisionHandler<T: HasInner> {
//     fn collide(&mut self, a: T::Inner<'_>, b: T::Inner<'_>);
// }
// impl<T: HasInner, F: FnMut(T::Inner<'_>, T::Inner<'_>)> CollisionHandler<T> for F {
//     #[inline(always)]
//     fn collide(&mut self, a: T::Inner<'_>, b: T::Inner<'_>) {
//         self(a, b);
//     }
// }

// pub struct ColHandle<F>(pub F);

// impl<F> ColHandle<F> {
//     pub fn new<T: HasInner>(func:F)->Self where F:FnMut(T::Inner<'_>, T::Inner<'_>){
//         ColHandle(func)
//     }
// }

// impl<T: HasInner, F: FnMut(T::Inner<'_>, T::Inner<'_>)> CollisionHandler<T> for ColHandle<F> {

//     #[inline(always)]
//     fn collide(&mut self, a: T::Inner<'_>, b: T::Inner<'_>) {
//         (self.0)(a, b);
//     }
// }

///
/// Finish handling a node by calling finish()
///
#[must_use]
pub struct NodeFinisher<'b, T> {
    axis: AxisDyn,
    bots: &'b mut [T],
    is_leaf: bool,
}
impl<'b, T: Aabb> NodeFinisher<'b, T> {
    pub fn finish<H: NodeHandler<T>>(self, handler: &mut H) {
        handler.handle_node(self.axis, self.bots, self.is_leaf);
    }
}

/// The main primitive to visit each node and find colliding pairs
pub struct CollisionVisitor<'a, 'b, T: Aabb> {
    vistr: VistrMutPin<'b, Node<'a, T, T::Num>>,
    axis: AxisDyn,
}
impl<'a, 'b, T: Aabb> CollisionVisitor<'a, 'b, T> {
    pub fn new(vistr: VistrMutPin<'b, Node<'a, T, T::Num>>) -> Self {
        CollisionVisitor {
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

        let is_leaf = self.get_height() == 1;

        let (n, rest) = self.vistr.next();

        let fin = NodeFinisher {
            axis: self.axis,
            bots: n.range,
            is_leaf,
        };

        (
            fin,
            if let Some([left, right]) = rest {
                Some([
                    CollisionVisitor {
                        vistr: left,
                        axis: self.axis.next(),
                    },
                    CollisionVisitor {
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
    fn handle_node(&mut self, axis: AxisDyn, bots: &mut [T], is_leaf: bool);

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
    pub fn extract_vec<'b, T>(&mut self) -> Vec<&'b mut T> {
        let mut v = Vec::new();
        core::mem::swap(&mut v, &mut self.vec);
        revec::convert_empty_vec(v)
    }

    ///Return the big capacity vec
    pub fn insert_vec<T>(&mut self, vec: Vec<&'_ mut T>) {
        let mut v = revec::convert_empty_vec(vec);
        core::mem::swap(&mut self.vec, &mut v)
    }
}
