//! Functions that make it easier to visualize the tree data structure
//!
use super::*;
use axgeom::AxisDyn;

pub struct DrawClosure<T, A> {
    pub _p: PhantomData<T>,
    pub line: A,
}

impl<T: Aabb, A> DividerDrawer for DrawClosure<T, A>
where
    A: FnMut(AxisDyn, &Node<T>, &Rect<T::Num>, usize),
{
    type T = T;
    type N = T::Num;

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
pub trait DividerDrawer {
    type T: Aabb<Num = Self::N>;
    type N: Num;
    fn draw_divider<A: Axis>(
        &mut self,
        axis: A,
        node: &Node<Self::T>,
        rect: &Rect<Self::N>,
        depth: usize,
    );
}

///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
pub fn draw<A: Axis, T: Aabb, D: DividerDrawer<T = T, N = T::Num>>(
    axis: A,
    vistr: Vistr<Node<T>>,
    dr: &mut D,
    rect: Rect<T::Num>,
) {
    fn recc<A: Axis, T: Aabb, D: DividerDrawer<T = T, N = T::Num>>(
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
