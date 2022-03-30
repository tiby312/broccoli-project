//! Rect query module

use super::*;

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
                        rect.get_range(this_axis.next()),
                    );

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.get_range(this_axis);

                    if div >= rr.start {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if div <= rr.end {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                }
                None => {
                    let sl = $get_section(
                        this_axis.next(),
                        $get_bots(nn),
                        rect.get_range(this_axis.next()),
                    );

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    };
}

///Naive implementation
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

pub fn for_all_not_in_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
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

        let NodeRef { div, range, .. } = nn.into_node_ref();

        for a in range.iter_mut() {
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

pub use constant::*;
pub use mutable::*;

mod mutable {
    use super::tools::get_section_mut;
    use super::*;
    fn foo<'a, 'b: 'a, T: Aabb>(node: PMut<'a, Node<'b, T>>) -> PMut<'a, [T]> {
        node.into_range()
    }
    rect!(VistrMut<'a, Node<T>>, PMut<'a, T>, get_section_mut, foo);
    pub fn for_all_intersect_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
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

    ///Naive implementation
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

    ///Naive implementation
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
    pub fn for_all_in_rect_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
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

    use super::tools::get_section;
    use super::*;
    fn foo<'a, 'b: 'a, T: Aabb>(node: &'a Node<'b, T>) -> &'a [T] {
        &node.range
    }
    rect!(Vistr<'a, Node<T>>, &'a T, get_section, foo);

    pub fn for_all_intersect_rect<'a, 'b: 'a, A: Axis, T: Aabb>(
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

    pub fn for_all_in_rect<'a, 'b: 'a, A: Axis, T: Aabb>(
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

use core::ops::Deref;
fn into_ptr_usize<T>(a: &T) -> usize {
    a as *const T as usize
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_not_in_rect_mut<T: Aabb>(bots: &mut [T], rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_not_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    naive_for_all_not_in_rect_mut(PMut::new(bots), rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_intersect_rect_mut<T: Aabb>(bots: &mut [T], rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_intersect_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    naive_for_all_intersect_rect_mut(PMut::new(bots), rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_in_rect_mut<T: Aabb>(bots: &mut [T], rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_in_rect_mut(rect, |a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    naive_for_all_in_rect_mut(PMut::new(bots), rect, |a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}
