use super::super::tools;
use super::oned;
use super::CollisionHandler;
use super::*;

pub struct NodeAxis<'a, 'node, T: Aabb, A: Axis> {
    pub node: TreePin<&'a mut Node<'node, T>>,
    pub axis: A,
}

impl<'a, 'node, T: Aabb, A: Axis> NodeAxis<'a, 'node, T, A> {
    #[inline(always)]
    pub fn borrow_mut<'c>(&'c mut self) -> NodeAxis<'c, 'node, T, A>
    where
        'a: 'c,
    {
        NodeAxis {
            node: self.node.borrow_mut(),
            axis: self.axis,
        }
    }
}

pub trait NodeHandler: Copy + Clone + Send + Sync {
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        axis: impl Axis,
        bots: TreePin<&mut [T]>,
    );

    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
    );
}

impl NodeHandler for crate::tree::NoSorter {
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        _: &mut PreVec,
        _axis: impl Axis,
        bots: TreePin<&mut [T]>,
    ) {
        tools::for_every_pair(bots, move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func.collide(a, b);
            }
        });
    }

    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        _: &mut PreVec,
        mut anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
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

impl NodeHandler for crate::tree::DefaultSorter {
    #[inline(always)]
    fn handle_node<T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        axis: impl Axis,
        bots: TreePin<&mut [T]>,
    ) {
        oned::find_2d(prevec, axis, bots, func);
    }
    #[inline(always)]
    fn handle_children<A: Axis, B: Axis, T: Aabb>(
        self,
        func: &mut impl CollisionHandler<T>,
        prevec: &mut PreVec,
        mut anchor: NodeAxis<T, A>,
        current: NodeAxis<T, B>,
    ) {
        if !current.axis.is_equal_to(anchor.axis) {
            let cc1 = &anchor.node.cont;

            let cc2 = current.node.into_node_ref();

            let r1 = super::tools::get_section_mut(anchor.axis, cc2.range, cc1);

            let r2 = super::tools::get_section_mut(
                current.axis,
                anchor.node.borrow_mut().into_range(),
                cc2.cont,
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
