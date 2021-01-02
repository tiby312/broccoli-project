use super::super::tools;
use super::CollisionHandler;
use crate::query::colfind::oned;
use crate::query::inner_prelude::*;

pub struct DestructuredNode<'a, 'b: 'a, T: Aabb, AnchorAxis: Axis> {
    pub node: PMut<'a, Node<'b, T>>,
    pub axis: AnchorAxis,
}

pub struct DestructuredNodeLeaf<'a, 'b: 'a, T: Aabb, A: Axis> {
    pub node: PMut<'a, Node<'b, T>>,
    pub axis: A,
}

pub trait NodeHandler {
    type T: Aabb;

    fn handle_node(&mut self, prevec:&mut PreVecMut<Self::T>,axis: impl Axis, bots: PMut<[Self::T]>);

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        prevec:&mut PreVecMut<Self::T>,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    );
}

pub struct HandleNoSorted<K: CollisionHandler + Splitter> {
    pub func: K,
}
impl<K: CollisionHandler + Splitter> HandleNoSorted<K> {
    #[inline(always)]
    pub fn new(func: K) -> Self {
        HandleNoSorted { func }
    }
}

impl<K: CollisionHandler + Splitter> Splitter for HandleNoSorted<K> {
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

impl<K: CollisionHandler + Splitter> NodeHandler for HandleNoSorted<K> {
    type T = K::T;
    
    fn handle_node(&mut self,_:&mut PreVecMut<K::T>, _axis: impl Axis, bots: PMut<[Self::T]>) {
        let func = &mut self.func;

        tools::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }

    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        _:&mut PreVecMut<K::T>,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.node.cont.intersects(&anchor.node.cont)
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
pub struct HandleSorted<K: CollisionHandler + Splitter> {
    pub func: K
}

impl<K: CollisionHandler + Splitter> HandleSorted<K> {
    #[inline(always)]
    pub fn new(a: K) -> HandleSorted<K> {
        HandleSorted {
            func: a,
        }
    }
}
impl<K: CollisionHandler + Splitter> Splitter for HandleSorted<K> {
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

impl<K: CollisionHandler + Splitter> NodeHandler for HandleSorted<K> {
    type T = K::T;

    
    #[inline(always)]
    fn handle_node(&mut self, prevec:&mut PreVecMut<K::T>,axis: impl Axis, bots: PMut<[Self::T]>) {
        
        let func = &mut self.func;
        oned::find_2d(prevec, axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis>(
        &mut self,
        prevec:&mut PreVecMut<K::T>,
        anchor: &mut DestructuredNode<Self::T, A>,
        current: DestructuredNodeLeaf<Self::T, B>,
    ) {
        let func = &mut self.func;

        if !current.axis.is_equal_to(anchor.axis) {
            let cc1 = anchor.node.cont;
            let cc2 = current.node.cont;

            let r1 = super::tools::get_section_mut(anchor.axis, current.node.into_range(), cc1);

            let r2 = super::tools::get_section_mut(
                current.axis,
                anchor.node.borrow_mut().into_range(),
                cc2,
            );

            oned::find_perp_2d1(current.axis, r1, r2, func);
        } else if current.node.cont.intersects(&anchor.node.cont) {
            /*
            oned::find_parallel_2d(
                &mut self.prevec1,
                current.axis.next(),
                current.node.into_range(),
                anchor.node.borrow_mut().into_range(),
                func,
            );
            */

            oned::find_parallel_2d(
                prevec,
                current.axis.next(),
                anchor.node.borrow_mut().into_range(),
                current.node.into_range(),
                func,
            );
        }
    }
}
