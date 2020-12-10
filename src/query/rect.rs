//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api.
//!
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!

use crate::query::inner_prelude::*;

macro_rules! rect {
    ($iterator:ty,$colsingle:ty,$get_section:ident,$get_node:ident) => {
        fn rect_recurse<'a, A: Axis, N: Node, F: FnMut($colsingle)>(
            this_axis: A,
            m: $iterator,
            rect: &Rect<N::Num>,
            func: &mut F,
        ) {
            let (nn, rest) = m.next();
            let nn = nn.$get_node();
            match rest {
                Some([left, right]) => {
                    let div = match nn.div {
                        Some(b) => b,
                        None => return,
                    };

                    let sl =
                        $get_section(this_axis.next(), nn.bots, *rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.get_range(this_axis);

                    if !(*div < rr.start) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(*div > rr.end) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                }
                None => {
                    let sl =
                        $get_section(this_axis.next(), nn.bots, *rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    };
}

pub fn naive_for_all_not_in_rect_mut<'a, T: Aabb>(
    bots: PMut<'a, [T]>,
    rect: &Rect<T::Num>,
    mut closure: impl FnMut(PMut<'a, T>),
) {
    for b in bots.iter_mut() {
        if !rect.contains_rect(b.get()) {
            closure(b);
        }
    }
}

pub fn for_all_not_in_rect_mut<'a,'b:'a, A: Axis, T: Aabb>(
    axis: A,
    vistr: VistrMut<'a, NodeMut<'b,T>>,
    rect: &Rect<T::Num>,
    closure: impl FnMut(PMut<'a, T>),
) {
    fn rect_recurse<'a,'b:'a, A: Axis, T: Aabb, F: FnMut(PMut<'a, T>)>(
        axis: A,
        it: VistrMut<'a, NodeMut<'b,T>>,
        rect: &Rect<T::Num>,
        mut closure: F,
    ) -> F {
        let (nn, rest) = it.next();
        let nn = nn.get_mut();

        for a in nn.bots.iter_mut() {
            if !rect.contains_rect(a.get()) {
                closure(a);
            }
        }

        match rest {
            Some([left, right]) => {
                let div = match nn.div {
                    Some(b) => b,
                    None => return closure,
                };

                match rect.get_range(axis).contains_ext(*div) {
                    core::cmp::Ordering::Greater => {
                        for a in right.into_slice() {
                            for b in a.get_mut().bots.iter_mut() {
                                closure(b)
                            }
                        }
                        rect_recurse(axis.next(), left, rect, closure)
                    }
                    core::cmp::Ordering::Less => {
                        for a in left.into_slice() {
                            for b in a.get_mut().bots.iter_mut() {
                                closure(b)
                            }
                        }
                        rect_recurse(axis.next(), right, rect, closure)
                    }
                    core::cmp::Ordering::Equal => {
                        let closure = rect_recurse(axis.next(), left, rect, closure);
                        rect_recurse(axis.next(), right, rect, closure)
                    }
                }
            }
            None => closure,
        }
    }
    rect_recurse(axis, vistr, rect, closure);
}

pub use constant::*;
pub use mutable::*;

mod mutable {
    use super::*;
    use crate::query::colfind::oned::get_section_mut;

    rect!(VistrMut<'a, N>, PMut<'a, N::T>, get_section_mut, get_mut);
    pub fn for_all_intersect_rect_mut<'a,'b:'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, NodeMut<'b,T>>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a, T>),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }

    pub fn naive_for_all_in_rect_mut<'a, T: Aabb>(
        bots: PMut<'a, [T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a, T>),
    ) {
        for b in bots.iter_mut() {
            if rect.contains_rect(b.get()) {
                closure(b);
            }
        }
    }

    pub fn naive_for_all_intersect_rect_mut<'a, T: Aabb>(
        bots: PMut<'a, [T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a, T>),
    ) {
        for b in bots.iter_mut() {
            if rect.get_intersect_rect(b.get()).is_some() {
                closure(b);
            }
        }
    }
    pub fn for_all_in_rect_mut<'a,'b:'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, NodeMut<'b,T>>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a, T>),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        });
    }
}

mod constant {

    use super::*;
    use crate::query::colfind::oned::get_section;
    rect!(Vistr<'a, N>, &'a N::T, get_section, get);

    pub fn for_all_intersect_rect<'a, A: Axis, N: Node>(
        axis: A,
        vistr: Vistr<'a, N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(&'a N::T),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }

    pub fn for_all_in_rect<'a, A: Axis, N: Node>(
        axis: A,
        vistr: Vistr<'a, N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(&'a N::T),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        });
    }
}

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RectIntersectErr;

///See the [`Queries::multi_rect`](crate::query::Queries::multi_rect) function.
pub struct MultiRectMut<'a,'b:'a, A: Axis, T: Aabb> {
    axis: A,
    vistr: VistrMut<'a, NodeMut<'b,T>>,
    rects: Vec<Rect<T::Num>>,
}

impl<'a,'b:'a, A: Axis, T: Aabb> MultiRectMut<'a,'b, A, T> {
    pub(crate) fn new(axis: A, vistr: VistrMut<'a, NodeMut<'b,T>>) -> Self {
        MultiRectMut {
            axis,
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

        self.rects.push(rect);

        for_all_in_rect_mut(
            self.axis,
            self.vistr.create_wrap_mut(),
            &rect,
            |bbox: PMut<T>| {
                //This is only safe to do because the user is unable to mutate the bounding box.
                let bbox: PMut<'a, T> = unsafe { core::mem::transmute(bbox) };
                func(bbox);
            },
        );

        Ok(())
    }
}
