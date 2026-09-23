//! Rect query module

use crate::queries::colfind::build::Single;

use super::*;


impl<'a, T: Aabb> crate::Tree<'a, T> {
    pub fn find_all_not_in_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: &K,
        mut closure: impl FnMut( Single<'b,T>),
    ) {
        fn rect_recurse<
            'a,
            'b: 'a,
            A: Axis,
            T: Aabb,
            K: Aabb<Num = T::Num>,
            F: FnMut( Single<'a,T>),
        >(
            axis: A,
            it: VistrMutPin<'a, Node<'b, T, T::Num>>,
            mut rect: &K,
            closure: &mut F,
        ) {
            let (nn, rest) = it.next();

            let NodeRef { div, range, .. } = nn.into_node_ref();

            for a in range.iter_mut() {
                if !rect.get().contains_rect(a.get()) {
                    closure( Single { inner: a });
                }
            }

            if let Some([left, right]) = rest {
                let div = match div {
                    Some(b) => b,
                    None => return,
                };

                match rect.range(axis).contains_ext(*div) {
                    core::cmp::Ordering::Greater => {
                        for a in right.into_slice() {
                            for b in a.range.iter_mut() {
                                closure( Single { inner: b })
                            }
                        }
                        rect_recurse(axis.next(), left, rect, closure)
                    }
                    core::cmp::Ordering::Less => {
                        for a in left.into_slice() {
                            for b in a.range.iter_mut() {
                                closure( Single { inner: b })
                            }
                        }
                        rect_recurse(axis.next(), right, rect, closure)
                    }
                    core::cmp::Ordering::Equal => {
                        rect_recurse(axis.next(), left, rect, closure);
                        rect_recurse(axis.next(), right, rect, closure)
                    }
                }
            }
        }
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut closure);
    }

    pub fn find_all_in_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: &K,
        mut closure: impl FnMut( Single<'b,T>),
    ) {
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut | a| {
            if rect.get().contains_rect(a.get()) {
                closure(a);
            }
        });
    }

    pub fn find_all_intersect_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: &K,
        mut closure: impl FnMut(Single<'b,T>),
    ) {
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut | a| {
            if rect.get().get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        });
    }
}

use super::tools::get_section_mut;
// fn foo<'a, 'b: 'a, T: Aabb>(node: AabbPin<&'a mut Node<'b, T, T::Num>>) -> AabbPin<&'a mut [T]> {
//     node.into_range()
// }
fn rect_recurse<
    'a,
    A: Axis,
    T: Aabb,
    F: FnMut(Single<'a,T>),
    K: Aabb<Num = T::Num>,
>(
    this_axis: A,
    m: VistrMutPin<'a, Node<T, T::Num>>,
    mut rect: &K,
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

            let sl = get_section_mut(this_axis.next(), nn.range, rect.range(this_axis.next()));

            for i in sl {
                func(Single { inner: i });
            }

            if div >= rect.range(this_axis).start {
                self::rect_recurse(this_axis.next(), left, rect, func);
            }
            if div <= rect.range(this_axis).end {
                self::rect_recurse(this_axis.next(), right, rect, func);
            }
        }
        None => {
            let sl = get_section_mut(this_axis.next(), nn.range, rect.range(this_axis.next()));

            for i in sl {
                func(Single { inner: i });
            }
        }
    }
}

mod assert {

    use super::*;
    use core::ops::Deref;
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }

    impl<'a, T: Aabb + ManySwap> Assert<'a, T> {
        ///Panics if a disconnect is detected between tree and naive queries.
        pub fn assert_rect(&mut self, rect: axgeom::Rect<T::Num>) {
            self.assert_for_all_not_in_rect_mut(rect);
            self.assert_for_all_intersect_rect_mut(rect);
            self.assert_for_all_in_rect_mut(rect)
        }

        fn assert_for_all_not_in_rect_mut(&mut self, mut rect: axgeom::Rect<T::Num>) {
            let mut tree = Tree::new(self.inner);
            let mut res_dino = Vec::new();
            tree.find_all_not_in_rect(&rect, |a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });

            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_not_in_rect(&rect, | a| {
                res_naive.push(into_ptr_usize(a.deref()));
            });

            res_dino.sort_unstable();
            res_naive.sort_unstable();

            assert_eq!(res_naive.len(), res_dino.len());
            assert!(res_naive.iter().eq(res_dino.iter()));
        }

        fn assert_for_all_intersect_rect_mut(&mut self, mut rect: axgeom::Rect<T::Num>) {
            let mut tree = Tree::new(self.inner);
            let mut res_dino = Vec::new();
            tree.find_all_intersect_rect(&rect, |a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });
            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_intersect_rect(&rect, |a| {
                res_naive.push(into_ptr_usize(a.deref()));
            });

            res_dino.sort_unstable();
            res_naive.sort_unstable();

            assert_eq!(res_naive.len(), res_dino.len());
            assert!(res_naive.iter().eq(res_dino.iter()));
        }

        fn assert_for_all_in_rect_mut(&mut self, mut rect: axgeom::Rect<T::Num>) {
            let mut tree = Tree::new(self.inner);
            let mut res_dino = Vec::new();
            tree.find_all_in_rect(&rect, |a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });
            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_in_rect(&rect, |a| {
                res_naive.push(into_ptr_usize(a.deref()));
            });

            res_dino.sort_unstable();
            res_naive.sort_unstable();

            assert_eq!(res_naive.len(), res_dino.len());
            assert!(res_naive.iter().eq(res_dino.iter()));
        }
    }

    impl<'a, T: Aabb> Naive<'a, T> {
        pub fn find_all_not_in_rect<'b, K: Aabb<Num = T::Num>>(
            &'b mut self,
            mut rect: &K,
            mut closure: impl FnMut(Single<'b,T>),
        ) {
            for b in self.iter_mut() {
                if !rect.get().contains_rect(b.get()) {
                    closure(Single { inner: b });
                }
            }
        }
        pub fn find_all_in_rect<'b, K: Aabb<Num = T::Num>>(
            &'b mut self,
            mut rect: &K,
            mut closure: impl FnMut(Single<'b,T>),
        ) {
            for b in self.iter_mut() {
                if rect.get().contains_rect(b.get()) {
                    closure(Single { inner: b });
                }
            }
        }
        pub fn find_all_intersect_rect<'b, K: Aabb<Num = T::Num>>(
            &'b mut self,
            mut rect: &K,
            mut closure: impl FnMut(Single<'b,T>),
        ) {
            for b in self.iter_mut() {
                if rect.get().get_intersect_rect(b.get()).is_some() {
                    closure(Single { inner: b });
                }
            }
        }
    }
}
