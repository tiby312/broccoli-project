//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api.
//!
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!

use crate::query::inner_prelude::*;

macro_rules! rect {
    ($iterator:ty,$colsingle:ty,$get_section:ident,$get_bots:ident) => {
        fn rect_recurse<'a, A: Axis, T: Aabb, F: FnMut($colsingle)>(
            this_axis: A,
            m: $iterator,
            rect: &Rect<T::Num>,
            func: &mut F,
        ) {
            let (nn, rest) = m.next();
            //let nn = nn.$get_node();
            match rest {
                Some([left, right]) => {
                    let div = match nn.div {
                        Some(b) => b,
                        None => return,
                    };

                    let sl = $get_section(
                        this_axis.next(),
                        $get_bots(nn),
                        *rect.get_range(this_axis.next()),
                    );

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.get_range(this_axis);

                    if !(div < rr.start) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(div > rr.end) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                }
                None => {
                    let sl = $get_section(
                        this_axis.next(),
                        $get_bots(nn),
                        *rect.get_range(this_axis.next()),
                    );

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

fn for_all_not_in_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
    axis: A,
    vistr: VistrMut<'a, Node<'b, T>>,
    rect: &Rect<T::Num>,
    closure: impl FnMut(PMut<'a, T>),
) {
    fn rect_recurse<'a, 'b: 'a, A: Axis, T: Aabb, F: FnMut(PMut<'a, T>)>(
        axis: A,
        it: VistrMut<'a, Node<'b, T>>,
        rect: &Rect<T::Num>,
        mut closure: F,
    ) -> F {
        let (nn, rest) = it.next();

        let (div, _, bots) = nn.into_range_full();

        for a in bots.iter_mut() {
            if !rect.contains_rect(a.get()) {
                closure(a);
            }
        }

        match rest {
            Some([left, right]) => {
                let div = match div {
                    Some(b) => b,
                    None => return closure,
                };

                match rect.get_range(axis).contains_ext(*div) {
                    core::cmp::Ordering::Greater => {
                        for a in right.into_slice() {
                            for b in a.into_range().iter_mut() {
                                closure(b)
                            }
                        }
                        rect_recurse(axis.next(), left, rect, closure)
                    }
                    core::cmp::Ordering::Less => {
                        for a in left.into_slice() {
                            for b in a.into_range().iter_mut() {
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

use constant::*;
use mutable::*;

mod mutable {
    use super::*;
    use crate::query::tools::get_section_mut;
    fn foo<'a, 'b: 'a, T: Aabb>(node: PMut<'a, Node<'b, T>>) -> PMut<'a, [T]> {
        node.into_range()
    }
    rect!(VistrMut<'a, Node<T>>, PMut<'a, T>, get_section_mut, foo);
    pub(super) fn for_all_intersect_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, Node<'b, T>>,
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
    pub(super) fn for_all_in_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, Node<'b, T>>,
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
    use crate::query::tools::get_section;
    fn foo<'a, 'b: 'a, T: Aabb>(node: &'a Node<'b, T>) -> &'a [T] {
        &node.range
    }
    rect!(Vistr<'a, Node<T>>, &'a T, get_section, foo);

    pub(super) fn for_all_intersect_rect<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: Vistr<'a, Node<'b, T>>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&'a T),
    ) {
        self::rect_recurse(axis, vistr, rect, &mut |a| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }

    pub(super) fn for_all_in_rect<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: Vistr<'a, Node<'b, T>>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&'a T),
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

///See the [`Queries::multi_rect`](crate::query::rect::RectQuery::multi_rect) function.
pub struct MultiRectMut<'a, 'b: 'a, A: Axis, T: Aabb> {
    axis: A,
    vistr: VistrMut<'a, Node<'b, T>>,
    rects: Vec<Rect<T::Num>>,
}

impl<'a, 'b: 'a, A: Axis, T: Aabb> MultiRectMut<'a, 'b, A, T> {
    fn new(axis: A, vistr: VistrMut<'a, Node<'b, T>>) -> Self {
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
            self.vistr.borrow_mut(),
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




use core::ops::Deref;
fn into_ptr_usize<T>(a: &T) -> usize {
    a as *const T as usize
}
use super::NaiveComparable;
pub fn assert_for_all_not_in_rect_mut<'a,K:NaiveComparable<'a>>(tree:&mut K, rect: &axgeom::Rect<K::Num>) {
    let mut res_dino = Vec::new();
    tree.get_tree().for_all_not_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    naive_for_all_not_in_rect_mut(tree.get_elements_mut(),rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn assert_for_all_intersect_rect_mut<'a,K:NaiveComparable<'a>>(tree:&mut K, rect: &axgeom::Rect<K::Num>) {
    let mut res_dino = Vec::new();
    tree.get_tree().for_all_intersect_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    naive_for_all_intersect_rect_mut(tree.get_elements_mut(),rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

pub fn assert_for_all_in_rect_mut<'a,K:NaiveComparable<'a>>(tree:&mut K, rect: &axgeom::Rect<K::Num>) {
    let mut res_dino = Vec::new();
    tree.get_tree().for_all_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    naive_for_all_in_rect_mut(tree.get_elements_mut(),rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort();
    res_naive.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}






use super::Queries;
impl<'a,K:Queries<'a>> RectQuery<'a> for K{}

pub trait RectQuery<'a>:Queries<'a>{

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10),rect(20,30,20,30)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let mut test = Vec::new();
    /// tree.for_all_intersect_rect(&rect(9,20,9,20),|a|{
    ///    test.push(a);
    /// });
    ///
    /// assert_eq!(test[0],&rect(0,10,0,10));
    ///
    ///```
    fn for_all_intersect_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T))
    where
        'a: 'b,
    {
        self::for_all_intersect_rect(self.axis(), self.vistr(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_intersect_rect_mut(&rect(9,20,9,20),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_intersect_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        self::for_all_intersect_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10),rect(20,30,20,30)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let mut test = Vec::new();
    /// tree.for_all_in_rect(&rect(0,20,0,20),|a|{
    ///    test.push(a);
    /// });
    ///
    /// assert_eq!(test[0],&rect(0,10,0,10));
    ///
    fn for_all_in_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T))
    where
        'a: 'b,
    {
        self::for_all_in_rect(self.axis(), self.vistr(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_in_rect_mut(&rect(0,10,0,10),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        self::for_all_in_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_not_in_rect_mut(&rect(10,20,10,20),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_not_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        self::for_all_not_in_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

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
    /// Handles a multi rect mut "sessions" within which
    /// the user can query multiple non intersecting rectangles.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots1 = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots1);
    /// let mut multi = tree.multi_rect();
    ///
    /// multi.for_all_in_rect_mut(rect(0,10,0,10),|a|{}).unwrap();
    /// let res = multi.for_all_in_rect_mut(rect(5,15,5,15),|a|{});
    /// assert_eq!(res,Err(broccoli::query::rect::RectIntersectErr));
    ///```
    #[must_use]
    fn multi_rect<'c>(&'c mut self) -> MultiRectMut<'c, 'a, Self::A, Self::T> {
        MultiRectMut::new(self.axis(), self.vistr_mut())
    }

    

}