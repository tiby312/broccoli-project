use crate::query::colfind::oned;
//use crate::query::colfind::ColMulti;
use super::super::tools;
use super::ColMulti;
use crate::query::inner_prelude::*;
use unchecked_unwrap::*;

pub struct DestructuredNode<'a, 'b: 'a, T: Aabb, AnchorAxis: Axis> {
    pub node: PMut<'a, Node<'b, T>>,
    pub axis: AnchorAxis,
}

impl<'a, 'b: 'a, T: Aabb, AnchorAxis: Axis> DestructuredNode<'a, 'b, T, AnchorAxis> {
    #[inline(always)]
    pub fn new(
        axis: AnchorAxis,
        node: PMut<'a, Node<'b, T>>,
    ) -> Option<DestructuredNode<'a, 'b, T, AnchorAxis>> {
        debug_assert!(node.div.is_some());
        if node.cont.is_some() {
            Some(DestructuredNode { node, axis })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn cont(&self) -> &axgeom::Range<T::Num> {
        unsafe { self.node.cont.as_ref().unchecked_unwrap() }
    }
}

pub struct DestructuredNodeLeaf<'a, 'b: 'a, T: Aabb, A: Axis> {
    pub node: PMut<'a, Node<'b, T>>,
    pub axis: A,
}

impl<'a, 'b: 'a, T: Aabb, AnchorAxis: Axis> DestructuredNodeLeaf<'a, 'b, T, AnchorAxis> {
    #[inline(always)]
    pub fn new(
        axis: AnchorAxis,
        node: PMut<'a, Node<'b, T>>,
    ) -> Option<DestructuredNodeLeaf<'a, 'b, T, AnchorAxis>> {
        if node.cont.is_some() {
            Some(DestructuredNodeLeaf { node, axis })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn cont(&self) -> &axgeom::Range<T::Num> {
        unsafe { self.node.cont.as_ref().unchecked_unwrap() }
    }
}

pub trait NodeHandler {
    type T: Aabb;

    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>);

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    );
}

pub struct HandleNoSorted<K: ColMulti + Splitter> {
    pub func: K,
}
impl<K: ColMulti + Splitter> HandleNoSorted<K> {
    #[inline(always)]
    pub fn new(func: K) -> Self {
        HandleNoSorted { func }
    }
}

impl<K: ColMulti + Splitter> Splitter for HandleNoSorted<K> {
    #[inline(always)]
    fn div(&mut self) -> (Self, Self) {
        let (a, b) = self.func.div();
        (HandleNoSorted { func: a }, HandleNoSorted { func: b })
    }
    #[inline(always)]
    fn add(&mut self, a: Self, b: Self) {
        self.func.add(a.func, b.func);
    }
}

impl<K: ColMulti + Splitter> NodeHandler for HandleNoSorted<K> {
    type T = K::T;
    fn handle_node(&mut self, _axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;

        tools::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.cont().intersects(anchor.cont())
        };

        if res {
            for mut a in current.node.into_range().iter_mut() {
                for mut b in anchor.node.borrow_mut().into_range().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a.borrow_mut(), b.borrow_mut());
                    }
                }
            }
        }
    }
}

use crate::util::PreVecMut;
pub struct HandleSorted<K: ColMulti + Splitter> {
    pub func: K,
    prevec1:PreVecMut<K::T>,
}

impl<K: ColMulti + Splitter> HandleSorted<K> {
    #[inline(always)]
    pub fn new(a: K) -> HandleSorted<K> {
        HandleSorted { func: a ,prevec1:PreVecMut::new()}
    }
}
impl<K: ColMulti + Splitter> Splitter for HandleSorted<K> {
    #[inline(always)]
    fn div(&mut self) -> (Self, Self) {
        let (a, b) = self.func.div();
        (HandleSorted::new(a), HandleSorted::new(b))
    }
    #[inline(always)]
    fn add(&mut self, a: Self, b: Self) {
        self.func.add(a.func, b.func);
    }
}

impl<K: ColMulti + Splitter> NodeHandler for HandleSorted<K> {
    type T = K::T;
    #[inline(always)]
    fn handle_node(&mut self, axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;
        oned::find_2d(&mut self.prevec1,axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        if !current.axis.is_equal_to(anchor.axis) {
            let cc1 = *anchor.cont();
            let cc2 = *current.cont();

            let r1 = super::tools::get_section_mut(anchor.axis, current.node.into_range(), cc1);

            let r2 =
                super::tools::get_section_mut(current.axis, anchor.node.borrow_mut().into_range(), cc2);

            oned::find_perp_2d1(&mut self.prevec1,current.axis, r1, r2, func);
        } else if current.cont().intersects(anchor.cont()) {
            /*
            oned::find_parallel_2d(
                current.axis.next(),
                current.node.into_range(),
                anchor.node.borrow_mut().into_range(),
                func,
            );
            */

            //This is faster than above.
            oned::find_parallel_2d(
                &mut self.prevec1,
                current.axis.next(),
                anchor.node.borrow_mut().into_range(),
                current.node.into_range(),
                func,
            );
        }
    }
}
