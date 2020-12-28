//! Functions that make it easier to visualize the tree data structure
//!
use crate::query::inner_prelude::*;

struct DrawClosure<N, Acc, A, B> {
    _p: PhantomData<N>,
    acc: Acc,
    xline: A,
    yline: B,
}

impl<N: Num, Acc, A, B> DividerDrawer for DrawClosure<N, Acc, A, B>
where
    A: FnMut(&mut Acc, N, [N; 2], [N; 2], usize),
    B: FnMut(&mut Acc, N, [N; 2], [N; 2], usize),
{
    type N = N;
    fn draw_divider<AA: Axis>(
        &mut self,
        axis: AA,
        div: Self::N,
        cont: [Self::N; 2],
        length: [Self::N; 2],
        depth: usize,
    ) {
        if axis.is_xaxis() {
            (self.xline)(&mut self.acc, div, cont, length, depth);
        } else {
            (self.yline)(&mut self.acc, div, cont, length, depth);
        }
    }
}

///Trait user must implement.
trait DividerDrawer {
    type N: Num;
    fn draw_divider<A: Axis>(
        &mut self,
        axis: A,
        div: Self::N,
        cont: [Self::N; 2],
        length: [Self::N; 2],
        depth: usize,
    );
}

///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
fn draw<A: Axis, T: Aabb, D: DividerDrawer<N = T::Num>>(
    axis: A,
    vistr: Vistr<Node<T>>,
    dr: &mut D,
    rect: &Rect<T::Num>,
) {
    fn recc<A: Axis, T: Aabb, D: DividerDrawer<N = T::Num>>(
        axis: A,
        stuff: LevelIter<Vistr<Node<T>>>,
        dr: &mut D,
        rect: &Rect<T::Num>,
    ) {
        let ((depth, nn), rest) = stuff.next();

        if let Some([left, right]) = rest {
            let div = match nn.div {
                Some(d) => d,
                None => return,
            };

            let cont = match nn.cont {
                Some(d) => d,
                None => return,
            };

            let cont = [cont.start, cont.end];
            let rr = rect.get_range(axis.next());
            dr.draw_divider::<A>(axis, div, cont, [rr.start, rr.end], depth.0);

            let (a, b) = rect.subdivide(axis, div);

            recc(axis.next(), left, dr, &a);
            recc(axis.next(), right, dr, &b);
        }
    }

    recc(axis, vistr.with_depth(Depth(0)), dr, rect);
}

use super::Queries;

///Draw functions that can be called on a tree.
pub trait DrawQuery<'a>: Queries<'a> + RectQuery<'a> {
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
    /// tree.draw_divider(&mut rects,
    ///     |rects,_,cont,length,_| rects.push(Rect {x: cont.into(),y: length.into()}),
    ///     |rects,_,cont,length,_| rects.push(Rect {x: length.into(),y:cont.into()}),
    ///     &dim
    /// );
    ///
    /// //rects now contains a bunch of rectangles that can be drawn to visualize
    /// //where all the dividers are and how thick they each are.
    ///
    /// ```
    ///
    fn draw_divider<A>(
        &self,
        acc: A,
        xline: impl FnMut(&mut A, Self::Num, [Self::Num; 2], [Self::Num; 2], usize),
        yline: impl FnMut(&mut A, Self::Num, [Self::Num; 2], [Self::Num; 2], usize),
        //drawer: &mut impl DividerDrawer<N = Self::Num>,
        rect: &Rect<Self::Num>,
    ) {
        let mut d = DrawClosure {
            _p: PhantomData,
            acc,
            xline,
            yline,
        };

        draw(self.axis(), self.vistr(), &mut d, rect)
    }
}
