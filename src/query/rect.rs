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
                        $get_section(this_axis.next(), nn.bots, rect.get_range(this_axis.next()));

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
                        $get_section(this_axis.next(), nn.bots, rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    };
}

pub fn naive_for_all_not_in_rect_mut<'a,T: Aabb>(
    bots: PMut<'a,[T]>,
    rect: &Rect<T::Num>,
    mut closure: impl FnMut(PMut<'a,T>),
) {
    for b in bots.iter_mut() {
        if !rect.contains_rect(b.get()) {
            closure(b);
        }
    }
}

pub fn for_all_not_in_rect_mut<'a,A: Axis,N:Node>(
    axis:A,
    vistr:VistrMut<'a,N>,
    rect: &Rect<N::Num>,
    closure: impl FnMut(PMut<'a,N::T>),
) {
    fn rect_recurse<'a,A: Axis, N: Node, F: FnMut(PMut<'a,N::T>)>(
        axis: A,
        it: VistrMut<'a,N>,
        rect: &Rect<N::Num>,
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
            Some([left,right]) => {
                let div = match nn.div {
                    Some(b) => b,
                    None => return closure,
                };

                match rect.get_range(axis).contains_ext(*div) {
                    core::cmp::Ordering::Greater => {
                        for a in right.into_slice(){
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

    rect!(VistrMut<'a,N>, PMut<'a,N::T>, get_section_mut, get_mut);
    pub fn for_all_intersect_rect_mut<'a,A: Axis, N: Node>(
        axis:A,
        vistr: VistrMut<'a,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(PMut<'a,N::T>),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }

    pub fn naive_for_all_in_rect_mut<'a,T: Aabb>(
        bots: PMut<'a,[T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a,T>),
    ) {
        for b in bots.iter_mut() {
            if rect.contains_rect(b.get()) {
                closure(b);
            }
        }
    }

    pub fn naive_for_all_intersect_rect_mut<'a,T: Aabb>(
        bots: PMut<'a,[T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<'a,T>),
    ) {
        for b in bots.iter_mut() {
            if rect.get_intersect_rect(b.get()).is_some() {
                closure(b);
            }
        }
    }
    pub fn for_all_in_rect_mut<'a,A: Axis, N: Node>(
        axis:A,
        vistr:VistrMut<'a,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(PMut<'a,N::T>),
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
        axis:A,
        vistr:Vistr<'a,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(&'a N::T),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }

    pub fn for_all_in_rect<'a, A: Axis, N:Node>(
        axis:A,
        vistr:Vistr<'a,N>,
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

/// If we have two non intersecting rectangles, it is safe to return to the user two sets of mutable references
/// of the bots strictly inside each rectangle since it is impossible for a bot to belong to both sets.
///
/// # Safety
///
/// Unsafe code is used.  We unsafely convert the references returned by the rect query
/// closure to have a longer lifetime.
/// This allows the user to store mutable references of non intersecting rectangles at the same time.
/// If two requested rectangles intersect, an error is returned.
///
///Handles a multi rect mut "sessions" within which
///the user can query multiple non intersecting rectangles.
pub struct MultiRectMut<'a, A: Axis, N: Node> {
    axis:A,
    vistr:VistrMut<'a,N>,
    rects: Vec<Rect<N::Num>>,
}

impl<'a, A: Axis, N:Node> MultiRectMut<'a, A, N> {
    pub fn new(axis:A,vistr:VistrMut<'a,N>) -> Self {
        MultiRectMut {
            axis,
            vistr,
            rects:Vec::new()
        }
    }
    pub fn for_all_in_rect_mut(
        &mut self,
        rect: Rect<N::Num>,
        mut func: impl FnMut(PMut<'a, N::T>),
    ) -> Result<(), RectIntersectErr> {
        for r in self.rects.iter() {
            if rect.get_intersect_rect(r).is_some() {
                return Err(RectIntersectErr);
            }
        }

        self.rects.push(rect);

        for_all_in_rect_mut(self.axis,self.vistr.create_wrap_mut(), &rect, |bbox: PMut<N::T>| {
            //This is only safe to do because the user is unable to mutate the bounding box.
            let bbox: PMut<'a, N::T> = unsafe { core::mem::transmute(bbox) };
            func(bbox);
        });

        Ok(())
    }
}
