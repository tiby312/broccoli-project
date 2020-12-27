use crate::query::inner_prelude::*;

///Trait user must implement.
pub trait DividerDrawer {
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
impl<'a,K:Queries<'a>> DrawQuery<'a> for K{}

pub trait DrawQuery<'a>: Queries<'a>+RectQuery<'a>{



    /// # Examples
    ///
    /// ```
    /// use broccoli::{prelude::*,bbox,rect};
    ///
    /// struct Drawer;
    /// impl broccoli::query::graphics::DividerDrawer for Drawer{
    ///     type N=i32;
    ///     fn draw_divider<A:axgeom::Axis>(
    ///             &mut self,
    ///             axis:A,
    ///             div:Self::N,
    ///             cont:[Self::N;2],
    ///             length:[Self::N;2],
    ///             depth:usize)
    ///     {
    ///         if axis.is_xaxis(){
    ///             //draw vertical line
    ///         }else{
    ///             //draw horizontal line
    ///         }
    ///     }
    /// }
    ///
    /// let border=rect(0,100,0,100);
    /// let mut bots =[rect(0,10,0,10)];
    /// let tree=broccoli::new(&mut bots);
    /// tree.draw_divider(&mut Drawer,&border);
    /// ```
    ///
    fn draw_divider(
        &self,
        drawer: &mut impl DividerDrawer<N = Self::Num>,
        rect: &Rect<Self::Num>,
    ) {
        draw(self.axis(), self.vistr(), drawer, rect)
    }

}