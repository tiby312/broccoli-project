//! Functions that make it easier to visualize the tree data structure
//!
use super::*;
use axgeom::AxisDyn;

pub struct DrawClosure<A> {
    pub line: A,
}

impl<T: Aabb, A> DividerDrawer<T> for DrawClosure<A>
where
    A: FnMut(AxisDyn, &Node<T>, &Rect<T::Num>, usize),
{
    #[inline(always)]
    fn draw_divider<AA: Axis>(
        &mut self,
        axis: AA,
        node: &Node<T>,
        rect: &Rect<T::Num>,
        depth: usize,
    ) {
        (self.line)(axis.to_dyn(), node, rect, depth);
    }
}

///Trait user must implement.
pub trait DividerDrawer<T: Aabb> {
    fn draw_divider<A: Axis>(&mut self, axis: A, node: &Node<T>, rect: &Rect<T::Num>, depth: usize);
}

pub fn draw_divider<T: Aabb>(
    tree: &mut crate::Tree<T>,
    line: impl FnMut(AxisDyn, &Node<T>, &Rect<T::Num>, usize),
    rect: Rect<T::Num>,
) {
    let mut d = DrawClosure { line };

    draw(default_axis(), tree.vistr(), &mut d, rect)
}

///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
pub fn draw<A: Axis, T: Aabb, D: DividerDrawer<T>>(
    axis: A,
    vistr: Vistr<Node<T>>,
    dr: &mut D,
    rect: Rect<T::Num>,
) {
    fn recc<A: Axis, T: Aabb, D: DividerDrawer<T>>(
        axis: A,
        stuff: LevelIter<Vistr<Node<T>>>,
        dr: &mut D,
        rect: Rect<T::Num>,
    ) {
        let ((depth, nn), rest) = stuff.next();
        dr.draw_divider(axis, nn, &rect, depth.0);

        if let Some([left, right]) = rest {
            if let Some(div) = nn.div {
                let (a, b) = rect.subdivide(axis, div);

                recc(axis.next(), left, dr, a);
                recc(axis.next(), right, dr, b);
            }
        }
    }

    recc(axis, vistr.with_depth(Depth(0)), dr, rect);
}
