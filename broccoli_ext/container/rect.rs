

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RectIntersectErr;

///See the [`Queries::multi_rect`](crate::query::rect::RectQuery::multi_rect) function.
pub struct MultiRect<'a, 'b: 'a, T: Aabb> {
    vistr: VistrMut<'a, Node<'b, T>>,
    rects: Vec<Rect<T::Num>>,
}

impl<'a, 'b: 'a, T: Aabb> MultiRect<'a, 'b, T> {
    pub fn new(vistr: VistrMut<'a, Node<'b, T>>) -> Self {
        MultiRect {
            vistr,
            rects: Vec::new(),
        }
    }
    pub fn for_all_in_rect_mut(
        &mut self,
        rect: Rect<T::Num>,
        mut func: impl FnMut(PMut<'a, T>),
    ) -> Result<(), RectIntersectErr> {
        for r in self.rects.iter() {
            if rect.intersects_rect(r) {
                return Err(RectIntersectErr);
            }
        }

        for_all_in_rect_mut(
            default_axis(),
            self.vistr.borrow_mut(),
            &rect,
            |bbox: PMut<T>| {
                //This is only safe to do because the user is unable to mutate the bounding box,
                //and we have checked that the query rectangles don't intersect.
                let bbox: PMut<'a, T> = PMut::new(unsafe { &mut *(bbox.into_inner() as *mut _) });
                func(bbox);
            },
        );

        self.rects.push(rect);

        Ok(())
    }
}