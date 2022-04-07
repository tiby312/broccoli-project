//! Rect query module

use super::*;

pub trait RectApi<'a, T: Aabb>: Sized {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        foo: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    );
    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        foo: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    );
    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        foo: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    );
}

impl<'a, 'b, T: Aabb> RectApi<'b, T> for &'b mut crate::Tree<'a, T> {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'b mut T>),
    ) {
        fn rect_recurse<
            'a,
            'b: 'a,
            A: Axis,
            T: Aabb,
            K: Aabb<Num = T::Num>,
            F: FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
        >(
            axis: A,
            it: VistrMutPin<'a, Node<'b, T>>,
            mut rect: TreePin<&mut K>,
            closure: &mut F,
        ) {
            let (nn, rest) = it.next();

            let NodeRef { div, range, .. } = nn.into_node_ref();

            for a in range.iter_mut() {
                if !rect.get().contains_rect(a.get()) {
                    closure(rect.borrow_mut(), a);
                }
            }

            match rest {
                Some([left, right]) => {
                    let div = match div {
                        Some(b) => b,
                        None => return,
                    };

                    match rect.get().get_range(axis).contains_ext(*div) {
                        core::cmp::Ordering::Greater => {
                            for a in right.into_slice() {
                                for b in a.into_range().iter_mut() {
                                    closure(rect.borrow_mut(), b)
                                }
                            }
                            rect_recurse(axis.next(), left, rect, closure)
                        }
                        core::cmp::Ordering::Less => {
                            for a in left.into_slice() {
                                for b in a.into_range().iter_mut() {
                                    closure(rect.borrow_mut(), b)
                                }
                            }
                            rect_recurse(axis.next(), right, rect, closure)
                        }
                        core::cmp::Ordering::Equal => {
                            rect_recurse(axis.next(), left, rect.borrow_mut(), closure);
                            rect_recurse(axis.next(), right, rect.borrow_mut(), closure)
                        }
                    }
                }
                None => {}
            }
        }
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, &mut closure);
    }

    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'b mut T>),
    ) {
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, &mut |r, a| {
            if r.get().contains_rect(a.get()) {
                closure(r, a);
            }
        });
    }

    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'b mut T>),
    ) {
        rect_recurse(axgeom::XAXIS, self.vistr_mut(), rect, &mut |r, a| {
            if r.get().get_intersect_rect(a.get()).is_some() {
                closure(r, a);
            }
        });
    }
}
impl<'a, T: Aabb> RectApi<'a, T> for TreePin<&'a mut [T]> {
    fn for_all_not_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        mut rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    ) {
        for b in self.iter_mut() {
            if !rect.get().contains_rect(b.get()) {
                closure(rect.borrow_mut(), b);
            }
        }
    }
    fn for_all_in_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        mut rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    ) {
        for b in self.iter_mut() {
            if rect.get().contains_rect(b.get()) {
                closure(rect.borrow_mut(), b);
            }
        }
    }
    fn for_all_intersect_rect_mut<K: Aabb<Num = T::Num>>(
        self,
        mut rect: TreePin<&mut K>,
        mut closure: impl FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    ) {
        for b in self.iter_mut() {
            if rect.get().get_intersect_rect(b.get()).is_some() {
                closure(rect.borrow_mut(), b);
            }
        }
    }
}

use super::tools::get_section_mut;
fn foo<'a, 'b: 'a, T: Aabb>(node: TreePin<&'a mut Node<'b, T>>) -> TreePin<&'a mut [T]> {
    node.into_range()
}
fn rect_recurse<
    'a,
    A: Axis,
    T: Aabb,
    F: FnMut(TreePin<&mut K>, TreePin<&'a mut T>),
    K: Aabb<Num = T::Num>,
>(
    this_axis: A,
    m: VistrMutPin<'a, Node<T>>,
    mut rect: TreePin<&mut K>,
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
                func(rect.borrow_mut(), i);
            }

            if div >= rect.get().get_range(this_axis).start {
                self::rect_recurse(this_axis.next(), left, rect.borrow_mut(), func);
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
                func(rect.borrow_mut(), i);
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
    tree.for_all_not_in_rect_mut(TreePin::new(&mut rect), |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });

    let mut res_naive = Vec::new();
    TreePin::new(bots).for_all_not_in_rect_mut(TreePin::new(&mut rect), |_, a| {
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
    tree.for_all_intersect_rect_mut(TreePin::new(&mut rect), |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    TreePin::new(bots).for_all_intersect_rect_mut(TreePin::new(&mut rect), |_, a| {
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
    tree.for_all_in_rect_mut(TreePin::new(&mut rect), |_, a| {
        res_dino.push(into_ptr_usize(a.deref()));
    });
    let mut res_naive = Vec::new();
    TreePin::new(bots).for_all_in_rect_mut(TreePin::new(&mut rect), |_, a| {
        res_naive.push(into_ptr_usize(a.deref()));
    });

    res_dino.sort_unstable();
    res_naive.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}
