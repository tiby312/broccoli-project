//! Functions that make it easier to visualize the tree data structure
//!
use crate::query::inner_prelude::*;
use axgeom::AxisDyn;

struct DrawClosure<T, A> {
    _p: PhantomData<T>,
    line: A,
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
trait DividerDrawer {
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
fn draw<A: Axis, T: Aabb, D: DividerDrawer<T = T, N = T::Num>>(
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

use super::Queries;

///Draw functions that can be called on a tree.
pub trait DrawQuery<'a>: Queries<'a> {
    /// # Examples
    ///
    /// ```
    /// use broccoli::{prelude::*,bbox,rect};
    /// use axgeom::Rect;
    ///
    /// let dim=rect(0,100,0,100);
    /// let mut bots =[rect(0,10,0,10)];
    /// let tree=broccoli::new(&mut bots);
    ///
    /// let mut rects=Vec::new();
    /// tree.draw_divider(
    ///     |axis,node,rect,_|
    ///     {
    ///         if !node.range.is_empty(){    
    ///             rects.push(
    ///                 axis.map_val(
    ///                     Rect {x: node.cont.into(),y: rect.y.into()},
    ///                     Rect {x: rect.x.into(),y: node.cont.into()}
    ///                 )   
    ///             );
    ///         }
    ///     },
    ///     dim
    /// );
    ///
    /// //rects now contains a bunch of rectangles that can be drawn to visualize
    /// //where all the dividers are and how thick they each are.
    ///
    /// ```
    ///
    fn draw_divider(
        &self,
        line: impl FnMut(AxisDyn, &Node<Self::T>, &Rect<Self::Num>, usize),
        rect: Rect<Self::Num>,
    ) {
        let mut d = DrawClosure {
            _p: PhantomData,
            line,
        };

        draw(default_axis(), self.vistr(), &mut d, rect)
    }
}
