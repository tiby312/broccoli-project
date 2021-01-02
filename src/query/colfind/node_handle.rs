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

pub trait NodeHandler:Copy+Clone +Send+Sync{
    
    fn handle_node<T:Aabb>(self,func:&mut impl CollisionHandler<T=T>,prevec:&mut PreVecMut<T>,axis: impl Axis, bots: PMut<[T]>);

    fn handle_children<A: Axis, B: Axis,T:Aabb>(
        self,
        func:&mut impl CollisionHandler<T=T>,
        prevec:&mut PreVecMut<T>,
        anchor: &mut DestructuredNode<T, A>,
        current: DestructuredNodeLeaf<T, B>,
    );
}


#[derive(Copy,Clone)]
pub struct HandleNoSorted;

impl NodeHandler for HandleNoSorted {
    
    
    fn handle_node<T:Aabb>(self,func:&mut impl CollisionHandler<T=T>,_:&mut PreVecMut<T>, _axis: impl Axis, bots: PMut<[T]>) {
        
        tools::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }

    fn handle_children<A: Axis, B: Axis,T:Aabb>(
        self,
        func:&mut impl CollisionHandler<T=T>,
        _:&mut PreVecMut<T>,
        anchor: &mut DestructuredNode<T, A>,
        current: DestructuredNodeLeaf<T, B>,
    ) {

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
#[derive(Copy,Clone)]
pub struct HandleSorted;



impl NodeHandler for HandleSorted {
    
    #[inline(always)]
    fn handle_node<T:Aabb>(self, func:&mut impl CollisionHandler<T=T>,prevec:&mut PreVecMut<T>,axis: impl Axis, bots: PMut<[T]>) {
        oned::find_2d(prevec, axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis,T:Aabb>(
        self,
        func:&mut impl CollisionHandler<T=T>,
        prevec:&mut PreVecMut<T>,
        anchor: &mut DestructuredNode<T, A>,
        current: DestructuredNodeLeaf<T, B>,
    ) {
        
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
