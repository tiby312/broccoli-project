//! Rect query module

use super::*;

pub trait RectApi<T: Aabb> {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        foo: impl FnMut(&mut K, HalfPin<&mut T>),
    );
    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        foo: impl FnMut(&mut K, HalfPin<&mut T>),
    );
    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        foo: impl FnMut(&mut K, HalfPin<&mut T>),
    );
}

impl<'a, T: Aabb> RectApi<T> for crate::Tree<'a, T> {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        fn rect_recurse<
            'a,
            'b: 'a,
            A: Axis,
            T: Aabb,
            K: Aabb<Num = T::Num>,
            F: FnMut(&mut K, HalfPin<&'a mut T>),
        >(
            axis: A,
            it: VistrMut<'a, Node<'b, T>>,
            rect: &mut K,
            mut closure: F,
        ) -> F {
            let (nn, rest) = it.next();

            let NodeRef { div, range, .. } = nn.into_node_ref();

            for a in range.iter_mut() {
                if !rect.get().contains_rect(a.get()) {
                    closure(rect, a);
                }
            }

            match rest {
                Some([left, right]) => {
                    let div = match div {
                        Some(b) => b,
                        None => return closure,
                    };

                    match rect.get().get_range(axis).contains_ext(*div) {
                        core::cmp::Ordering::Greater => {
                            for a in right.into_slice() {
                                for b in a.into_range().iter_mut() {
                                    closure(rect, b)
                                }
                            }
                            rect_recurse(axis.next(), left, rect, closure)
                        }
                        core::cmp::Ordering::Less => {
                            for a in left.into_slice() {
                                for b in a.into_range().iter_mut() {
                                    closure(rect, b)
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
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, closure);
    }

    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        mut closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, &mut |r, a| {
            if r.get().contains_rect(a.get()) {
                closure(r, a);
            }
        });
    }

    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        mut closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, &mut |r, a| {
            if r.get().get_intersect_rect(a.get()).is_some() {
                closure(r, a);
            }
        });
    }
}
impl<'a, T: Aabb> RectApi<T> for HalfPin<&'a mut [T]> {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        mut closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        for b in self.borrow_mut().iter_mut() {
            if !rect.get().contains_rect(b.get()) {
                closure(rect, b);
            }
        }
    }
    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        mut closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        for b in self.borrow_mut().iter_mut() {
            if rect.get().contains_rect(b.get()) {
                closure(rect, b);
            }
        }
    }
    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        &mut self,
        rect: &mut K,
        mut closure: impl FnMut(&mut K, HalfPin<&mut T>),
    ) {
        for b in self.borrow_mut().iter_mut() {
            if rect.get().get_intersect_rect(b.get()).is_some() {
                closure(rect, b);
            }
        }
    }
}

use super::tools::get_section_mut;
fn foo<'a, 'b: 'a, T: Aabb>(node: HalfPin<&'a mut Node<'b, T>>) -> HalfPin<&'a mut [T]> {
    node.into_range()
}
fn rect_recurse<
    'a,
    A: Axis,
    T: Aabb,
    F: FnMut(&mut K, HalfPin<&'a mut T>),
    K: Aabb<Num = T::Num>,
>(
    this_axis: A,
    m: VistrMut<'a, Node<T>>,
    rect: &mut K,
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

            let sl = get_section_mut(
                this_axis.next(),
                foo(nn),
                rect.get().get_range(this_axis.next()),
            );

            for i in sl {
                func(rect, i);
            }

            if div >= rect.get().get_range(this_axis).start {
                self::rect_recurse(this_axis.next(), left, rect, func);
            }
            if div <= rect.get().get_range(this_axis).end {
                self::rect_recurse(this_axis.next(), right, rect, func);
            }
        }
        None => {
            let sl = get_section_mut(
                this_axis.next(),
                foo(nn),
                rect.get().get_range(this_axis.next()),
            );

            for i in sl {
                func(rect, i);
            }
        }
    }
}

use core::ops::Deref;
fn into_ptr_usize<T>(a: &T) -> usize {
    a as *const T as usize
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_not_in_rect_mut<T: Aabb>(bots: &mut [T], mut rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_not_in_rect_mut(&mut rect, |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    HalfPin::new(bots).for_all_not_in_rect_mut(&mut rect, |_, a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_intersect_rect_mut<T: Aabb>(bots: &mut [T], mut rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_intersect_rect_mut(&mut rect, |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    HalfPin::new(bots).for_all_intersect_rect_mut(&mut rect, |_, a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_for_all_in_rect_mut<T: Aabb>(bots: &mut [T], mut rect: &axgeom::Rect<T::Num>) {
    let mut tree = crate::new(bots);
    let mut res_dino = Vec::new();
    tree.for_all_in_rect_mut(&mut rect, |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    HalfPin::new(bots).for_all_in_rect_mut(&mut rect, |_, a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}
