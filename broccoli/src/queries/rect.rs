//! Rect query module

use super::*;

impl<'a, T: Aabb> crate::Tree<'a, T> {
    pub fn find_all_not_in_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: AabbPin<&mut K>,
        mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
    ) {
        fn rect_recurse<
            'a,
            'b: 'a,
            A: Axis,
            T: Aabb,
            K: Aabb<Num = T::Num>,
            F: FnMut(AabbPin<&mut K>, AabbPin<&'a mut T>),
        >(
            axis: A,
            it: VistrMutPin<'a, Node<'b, T, T::Num>>,
            mut rect: AabbPin<&mut K>,
            closure: &mut F,
        ) {
            let (nn, rest) = it.next();

            let NodeRef { div, range, .. } = nn.into_node_ref();

            for a in range.iter_mut() {
                if !rect.contains_aabb(&*a) {
                    closure(rect.borrow_mut(), a);
                }
            }

            if let Some([left, right]) = rest {
                let div = match div {
                    Some(b) => b,
                    None => return,
                };

                match rect.to_range(axis).contains_ext(div) {
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
        }
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut closure);
    }

    pub fn find_all_in_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: AabbPin<&mut K>,
        mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
    ) {
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut |r, a| {
            if r.contains_aabb(&*a) {
                closure(r, a);
            }
        });
    }

    pub fn find_all_intersect_rect<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: AabbPin<&mut K>,
        mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
    ) {
        rect_recurse(default_axis(), self.vistr_mut(), rect, &mut |r, a| {
            if r.intersects_aabb(&*a) {
                closure(r, a);
            }
        });
    }
}

use super::tools::get_section_mut;
fn foo<'a, 'b: 'a, T: Aabb>(node: AabbPin<&'a mut Node<'b, T, T::Num>>) -> AabbPin<&'a mut [T]> {
    node.into_range()
}
fn rect_recurse<
    'a,
    A: Axis,
    T: Aabb,
    F: FnMut(AabbPin<&mut K>, AabbPin<&'a mut T>),
    K: Aabb<Num = T::Num>,
>(
    this_axis: A,
    m: VistrMutPin<'a, Node<T, T::Num>>,
    mut rect: AabbPin<&mut K>,
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

            let sl = get_section_mut(this_axis.next(), foo(nn), &rect.to_range(this_axis.next()));

            for i in sl {
                func(rect.borrow_mut(), i);
            }

            if div >= *rect.to_range(this_axis).start() {
                self::rect_recurse(this_axis.next(), left, rect.borrow_mut(), func);
            }
            if div <= *rect.to_range(this_axis).end() {
                self::rect_recurse(this_axis.next(), right, rect, func);
            }
        }
        None => {
            let sl = get_section_mut(this_axis.next(), foo(nn), &rect.to_range(this_axis.next()));

            for i in sl {
                func(rect.borrow_mut(), i);
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
            tree.find_all_not_in_rect(AabbPin::new(&mut rect), |_, a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });

            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_not_in_rect(AabbPin::new(&mut rect), |_, a| {
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
            tree.find_all_intersect_rect(AabbPin::new(&mut rect), |_, a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });
            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_intersect_rect(AabbPin::new(&mut rect), |_, a| {
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
            tree.find_all_in_rect(AabbPin::new(&mut rect), |_, a| {
                res_dino.push(into_ptr_usize(a.deref()));
            });
            let mut res_naive = Vec::new();
            Naive::new(self.inner).find_all_in_rect(AabbPin::new(&mut rect), |_, a| {
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
            mut rect: AabbPin<&mut K>,
            mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
        ) {
            for b in self.iter_mut() {
                if !rect.contains_aabb(&*b) {
                    closure(rect.borrow_mut(), b);
                }
            }
        }
        pub fn find_all_in_rect<'b, K: Aabb<Num = T::Num>>(
            &'b mut self,
            mut rect: AabbPin<&mut K>,
            mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
        ) {
            for b in self.iter_mut() {
                if rect.contains_aabb(&*b) {
                    closure(rect.borrow_mut(), b);
                }
            }
        }
        pub fn find_all_intersect_rect<'b, K: Aabb<Num = T::Num>>(
            &'b mut self,
            mut rect: AabbPin<&mut K>,
            mut closure: impl FnMut(AabbPin<&mut K>, AabbPin<&'b mut T>),
        ) {
            for b in self.iter_mut() {
                if rect.intersects_aabb(&*b) {
                    closure(rect.borrow_mut(), b);
                }
            }
        }
    }
}
