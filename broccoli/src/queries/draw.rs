//! Functions that make it easier to visualize the tree data structure
//!
use super::*;
use axgeom::AxisDyn;

///Trait user must implement.
pub trait DividerDrawer<T: Aabb> {
    fn draw_divider<A: Axis>(&mut self, axis: A, node: &Node<T,T::Num>, rect: &Rect<T::Num>, depth: usize);
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn draw_divider(
        &self,
        line: impl FnMut(AxisDyn, &Node<T,T::Num>, &Rect<T::Num>, usize),
        rect: Rect<T::Num>,
    ) {
        let tree = self;

        struct DrawClosure<A> {
            pub line: A,
        }

        impl<T: Aabb, A> DividerDrawer<T> for DrawClosure<A>
        where
            A: FnMut(AxisDyn, &Node<T,T::Num>, &Rect<T::Num>, usize),
        {
            #[inline(always)]
            fn draw_divider<AA: Axis>(
                &mut self,
                axis: AA,
                node: &Node<T,T::Num>,
                rect: &Rect<T::Num>,
                depth: usize,
            ) {
                (self.line)(axis.to_dyn(), node, rect, depth);
            }
        }

        ///Calls the user supplied function on each divider.
        ///Since the leaves do not have dividers, it is not called for the leaves.
        fn draw<A: Axis, T: Aabb, D: DividerDrawer<T>>(
            axis: A,
            vistr: Vistr<Node<T,T::Num>>,
            dr: &mut D,
            rect: Rect<T::Num>,
        ) {
            fn recc<A: Axis, T: Aabb, D: DividerDrawer<T>>(
                axis: A,
                stuff: LevelIter<Vistr<Node<T,T::Num>>>,
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

        let mut d = DrawClosure { line };

        draw(default_axis(), tree.vistr(), &mut d, rect)
    }
}
