use super::*;

///CollidingPairs created via [`TreeRefInd::collect_colliding_pairs`]
pub struct CollidingPairs<T, D> {
    ///See collect_intersections_list()
    ///The same elements can be part of
    ///multiple intersecting pairs.
    ///So pointer aliasing rules are not
    ///being met if we were to just use this
    ///vec according to its type signature.
    cols: Vec<(Ptr<T>, Ptr<T>, D)>,
    orig: Ptr<[T]>,
}
impl<T, D> CollidingPairs<T, D> {
    ///Return a read only list of colliding pairs.
    ///We can't return a list of mutable pairs since some might
    ///alias, but we can return a list if they are not mutable.
    pub fn get(&self, arr: &[T]) -> &[(&T, &T, D)] {
        assert_eq!(self.orig.0 as *const _, arr as *const _);
        unsafe { &*(self.cols.as_slice() as *const _ as *const _) }
    }

    ///Visit every colliding pair.
    ///panics if the slice passed is not the slice used to create this
    ///`CollidingPairs` object.
    pub fn for_every_pair_mut(
        &mut self,
        arr: &mut [T],
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        assert_eq!(self.orig.0, arr as *mut _);

        for (a, b, d) in self.cols.iter_mut() {
            func(unsafe { &mut *(*a).0 }, unsafe { &mut *(*b).0 }, d)
        }
    }
}

///CollidingPairsPar created via [`TreeRefInd::collect_colliding_pairs_par`]
///All colliding pairs partitioned into
///mutually exclusive sets so that they can be traversed in parallel
pub struct CollidingPairsPar<T, D> {
    cols: Vec<Vec<(Ptr<T>, Ptr<T>, D)>>,
    original: Ptr<[T]>,
}

impl<T, D> CollidingPairsPar<T, D> {
    pub fn get(&self, arr: &[T]) -> &[Vec<(&T, &T, D)>] {
        assert_eq!(arr as *const _, self.original.0 as *const _);
        unsafe { &*(self.cols.as_slice() as *const _ as *const _) }
    }
}
impl<T: Send + Sync, D: Send + Sync> CollidingPairsPar<T, D> {
    pub fn for_every_pair_mut_par(
        &mut self,
        arr: &mut [T],
        func: impl Fn(&mut T, &mut T, &mut D) + Send + Sync + Copy,
    ) {
        assert_eq!(arr as *mut _, self.original.0);

        fn parallelize<T: Visitor + Send + Sync>(a: T, func: impl Fn(T::Item) + Sync + Send + Copy)
        where
            T::Item: Send + Sync,
        {
            let (n, l) = a.next();
            func(n);
            if let Some([left, right]) = l {
                rayon::join(|| parallelize(left, func), || parallelize(right, func));
            }
        }
        let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut self.cols).unwrap();

        parallelize(mtree.vistr_mut(), |a| {
            for (a, b, d) in a.iter_mut() {
                let a = unsafe { &mut *a.0 };
                let b = unsafe { &mut *b.0 };
                func(a, b, d)
            }
        });
    }
}

impl<'a, A: Axis, N: Num, T: Send + Sync> TreeRefInd<'a, A, N, T> {
    /// The parallel version of [`TreeRefInd::collect_colliding_pairs`] that instead
    /// returns a [`CollidingPairsPar`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///     broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///     broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///     broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut tree = broccoli::collections::TreeRefInd::new_par(&mut aabbs,|a|{
    ///    a.rect
    /// });
    ///
    /// //Find all colliding aabbs only once.
    /// let mut pairs=tree.collect_colliding_pairs_par(|a, b| {
    ///    a.inner += 1;
    ///    b.inner += 1;
    ///    Some(())
    /// });
    ///
    /// //Iterate over the pairs multiple times
    /// for _ in 0..3{
    ///     //mutate every colliding pair.
    ///     pairs.for_every_pair_mut_par(&mut aabbs,|a,b,()|{
    ///         a.inner+=1;
    ///         b.inner+=1;
    ///     })
    /// }
    pub fn collect_colliding_pairs_par<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> CollidingPairsPar<T, D> {
        let cols = self.collect_colliding_pairs_par_inner(|a, b| match func(a, b) {
            Some(d) => Some((Ptr(a as *mut _), Ptr(b as *mut _), d)),
            None => None,
        });
        CollidingPairsPar {
            cols,
            original: self.tree.orig,
        }
    }

    fn collect_colliding_pairs_par_inner<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> Vec<Vec<D>> {
        struct Foo<T: Visitor> {
            current: T::Item,
            next: Option<[T; 2]>,
        }
        impl<T: Visitor> Foo<T> {
            fn new(a: T) -> Foo<T> {
                let (n, f) = a.next();
                Foo {
                    current: n,
                    next: f,
                }
            }
        }

        let height = 1 + par::compute_level_switch_sequential(
            par::SWITCH_SEQUENTIAL_DEFAULT,
            self.get_height(),
        )
        .get_depth_to_switch_at();
        let mut cols: Vec<Vec<D>> = (0..compt::compute_num_nodes(height))
            .map(|_| Vec::new())
            .collect();
        let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut cols).unwrap();

        self.new_colfind_builder().query_par_ext(
            move |a| {
                let next = a.next.take();
                if let Some([left, right]) = next {
                    let l = Foo::new(left);
                    let r = Foo::new(right);
                    *a = l;
                    r
                } else {
                    unreachable!()
                }
            },
            move |_a, _b| {},
            move |c, a, b| {
                if let Some(d) = func(a.unpack_inner(), b.unpack_inner()) {
                    c.current.push(d);
                }
            },
            Foo::new(mtree.vistr_mut()),
        );

        cols
        //CollidingPairsPar{cols,_p:PhantomData}
    }
}

///Contains a filtered list of all elements in the tree from calling [`TreeRefInd::collect_all`].
pub struct FilteredElements<T, D> {
    elems: Vec<(Ptr<T>, D)>,
    orig: Ptr<[T]>,
}
impl<T, D> FilteredElements<T, D> {
    pub fn get(&self, arr: &[T]) -> &[(&T, D)] {
        assert_eq!(self.orig.0 as *const _, arr as *const _);
        unsafe { &*(self.elems.as_slice() as *const _ as *const _) }
    }
    pub fn get_mut(&mut self, arr: &mut [T]) -> &mut [(&mut T, D)] {
        assert_eq!(self.orig.0, arr as *mut _);
        unsafe { &mut *(self.elems.as_mut_slice() as *mut _ as *mut _) }
    }
}

impl<'a, A: Axis, N: Num, T> TreeRefInd<'a, A, N, T> {
    /// Collect all elements based off of a predicate and return a [`FilteredElements`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///    broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///    broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut tree = broccoli::collections::TreeRefInd::new(&mut aabbs,|a|{
    ///     a.rect
    /// });
    ///
    /// //Find a group of elements only once.
    /// let mut pairs=tree.collect_all(|_,b| {
    ///    if b.inner % 2 ==0{
    ///        Some(())
    ///    }else{
    ///        None
    ///    }
    /// });
    ///
    /// //Iterate over that group multiple times
    /// for _ in 0..3{
    ///     //mutate every colliding pair.
    ///     for (a,()) in pairs.get_mut(&mut aabbs){
    ///         a.inner+=1;
    ///     }
    /// }
    pub fn collect_all<D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&Rect<N>, &mut T) -> Option<D>,
    ) -> FilteredElements<T, D> {
        let mut elems = Vec::new();
        for node in self.get_nodes_mut().iter_mut() {
            for b in node.get_mut().bots.iter_mut() {
                let (x, y) = b.unpack();
                if let Some(d) = func(x, y) {
                    elems.push((Ptr(*y as *mut _), d));
                }
            }
        }
        FilteredElements {
            orig: self.tree.orig,
            elems,
        }
    }
}

impl<'a, A: Axis, N: Num, T> TreeRefInd<'a, A, N, T> {
    /// Find all colliding pairs based on a predicate and return a [`CollidingPairs`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///     broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///     broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///     broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut tree = broccoli::collections::TreeRefInd::new(&mut aabbs,|a|{
    ///    a.rect
    /// });
    ///
    /// //Find all colliding aabbs only once.
    /// let mut pairs=tree.collect_colliding_pairs(|a, b| {
    ///    a.inner += 1;
    ///    b.inner += 1;
    ///    Some(())
    /// });
    ///
    /// //Iterate over the pairs multiple times
    /// for _ in 0..3{
    ///     //mutate every colliding pair.
    ///     pairs.for_every_pair_mut(&mut aabbs,|a,b,()|{
    ///         a.inner+=1;
    ///         b.inner+=1;
    ///     })
    /// }
    pub fn collect_colliding_pairs<D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut T, &mut T) -> Option<D> + Send + Sync,
    ) -> CollidingPairs<T, D> {
        let mut cols: Vec<_> = Vec::new();

        self.find_colliding_pairs_mut(|a, b| {
            let a = a.unpack_inner();
            let b = b.unpack_inner();
            if let Some(d) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let a = Ptr(*a as *mut T);
                let b = Ptr(*b as *mut T);

                cols.push((a, b, d));
            }
        });

        CollidingPairs {
            cols,
            orig: self.tree.orig,
        }
    }
}
