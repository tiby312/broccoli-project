use broccoli::prelude::*;
use broccoli::tree::halfpin::HalfPin;
use broccoli::tree::node::*;
use broccoli::tree::Tree;

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RectIntersectErr;

///
/// Aabb::get() guarenteed to return the same value while pinned by `HalfPin`.
///
pub unsafe trait TrustedAabb: Aabb {}

unsafe impl<N: Num, T> TrustedAabb for BBox<N, T> {}
unsafe impl<N: Num> TrustedAabb for Rect<N> {}
unsafe impl<T: TrustedAabb> TrustedAabb for &T {}
unsafe impl<T: TrustedAabb> TrustedAabb for &mut T {}

pub fn multi_rect<'a, 'b, T: TrustedAabb>(tree: &'a mut Tree<'b, T>) -> MultiRect<'a, 'b, T> {
    MultiRect::new(tree)
}

///See the [`Queries::multi_rect`](crate::query::rect::RectQuery::multi_rect) function.
pub struct MultiRect<'a, 'b, T: Aabb> {
    tree: &'a mut Tree<'b, T>,
    rects: Vec<Rect<T::Num>>,
}

impl<'a, 'b, T: TrustedAabb> MultiRect<'a, 'b, T> {
    ///Unsafe because this api relies on the fact that
    ///every call to AAbb::get() returns the same aabb.
    ///Before making this, ensure that the above is true
    pub fn new(tree: &'a mut Tree<'b, T>) -> Self {
        MultiRect {
            tree,
            rects: Vec::new(),
        }
    }
    pub fn for_all_in_rect_mut(
        &mut self,
        mut rect: Rect<T::Num>,
        mut func: impl FnMut(HalfPin<&'a mut T>),
    ) -> Result<(), RectIntersectErr> {
        for r in self.rects.iter() {
            if rect.intersects_rect(r) {
                return Err(RectIntersectErr);
            }
        }

        self.tree.for_all_in_rect_mut(
            HalfPin::from_mut(&mut rect),
            |_rect, mut bbox: HalfPin<&mut T>| {
                //This is only safe to do because the user is unable to mutate the bounding box,
                //and we have checked that the query rectangles don't intersect.
                let bbox: HalfPin<&'a mut T> =
                    HalfPin::from_mut(unsafe { &mut *(bbox.as_ptr_mut().as_raw() as *mut _) });
                func(bbox);
            },
        );

        self.rects.push(rect);

        Ok(())
    }
}
