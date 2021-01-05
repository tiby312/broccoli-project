use super::*;

///A read only colliding pair reference
pub struct ColPair<'a, T, D> {
    pub first: &'a T,
    pub second: &'a T,
    pub extra: D,
}

struct ColPairPtr<T, D> {
    first: Ptr<T>,
    second: Ptr<T>,
    extra: D,
}
///CollidingPairs created via [`TreeRefInd::collect_colliding_pairs`]
pub struct CollidingPairs<T, D> {
    ///See collect_intersections_list()
    ///The same elements can be part of
    ///multiple intersecting pairs.
    ///So pointer aliasing rules are not
    ///being met if we were to just use this
    ///vec according to its type signature.
    cols: Vec<ColPairPtr<T, D>>,
    orig: Ptr<[T]>,
}
impl<T, D> CollidingPairs<T, D> {
    ///Return a read only list of colliding pairs.
    ///We can't return a list of mutable pairs since some might
    ///alias, but we can return a list if they are not mutable.
    #[inline(always)]
    pub fn get(&self, arr: &[T]) -> &[ColPair<T, D>] {
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

        for ColPairPtr {
            first,
            second,
            extra,
        } in self.cols.iter_mut()
        {
            func(
                unsafe { &mut *(*first).0 },
                unsafe { &mut *(*second).0 },
                extra,
            )
        }
    }
}

///CollidingPairsPar created via [`TreeRefInd::collect_colliding_pairs_par`]
///All colliding pairs partitioned into
///mutually exclusive sets so that they can be traversed in parallel
pub struct CollidingPairsPar<T, D> {
    cols: Vec<Vec<ColPairPtr<T, D>>>,
    original: Ptr<[T]>,
}

impl<T, D> From<CollidingPairsPar<T, D>> for CollidingPairs<T, D> {
    fn from(a: CollidingPairsPar<T, D>) -> Self {
        let cols = a.cols.into_iter().flatten().collect();
        CollidingPairs {
            cols,
            orig: a.original,
        }
    }
}

impl<T, D> CollidingPairsPar<T, D> {
    pub fn get(&self, arr: &[T]) -> &[Vec<ColPair<T, D>>] {
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
        use rayon::prelude::*;
        self.cols.par_iter_mut().for_each(|a| {
            for ColPairPtr {
                first,
                second,
                extra,
            } in a.iter_mut()
            {
                let a = unsafe { &mut *first.0 };
                let b = unsafe { &mut *second.0 };
                func(a, b, extra)
            }
        });
    }
}

///Contains a filtered list of all elements in the tree from calling [`TreeRefInd::collect_all`].
pub struct FilteredElements<T, D> {
    elems: Vec<(Ptr<T>, D)>,
    orig: Ptr<[T]>,
}
impl<T, D> FilteredElements<T, D> {
    #[inline(always)]
    pub fn get(&self, arr: &[T]) -> &[(&T, D)] {
        assert_eq!(self.orig.0 as *const _, arr as *const _);
        unsafe { &*(self.elems.as_slice() as *const _ as *const _) }
    }
    #[inline(always)]
    pub fn get_mut(&mut self, arr: &mut [T]) -> &mut [(&mut T, D)] {
        assert_eq!(self.orig.0, arr as *mut _);
        unsafe { &mut *(self.elems.as_mut_slice() as *mut _ as *mut _) }
    }
}










/// A less general tree that providess `collect` functions
/// and also derefs to a [`Tree`].
///
/// [`TreeRefInd`] assumes there is a layer of indirection where
/// all the pointers point to the same slice.
/// It uses this assumption to provide `collect` functions that allow
/// storing query results that can then be iterated through multiple times
/// quickly.
///
#[repr(C)]
pub struct TreeRefInd<'a,'b,N:Num,T>{
    tree:Tree<'b,BBox<N,&'a mut T>>,
    orig:Ptr<[T]>
}

impl<'a,'b,N:Num,T> TreeRefInd<'a,'b,N,T>{

    ///Create a [`TreeRefInd`]. The user provides an empty vec to use to house the
    ///The aabbs created.
    pub fn new(
        bots:&'a mut [T],
        mut func:impl FnMut(&mut T)->axgeom::Rect<N>,
        aabbs:&'b mut Vec<BBox<N,&'a mut T>>)->TreeRefInd<'a,'b,N,T>{
        let orig = Ptr(bots as *mut _);

        aabbs.extend(bots.iter_mut().map(|a|crate::bbox(func(a),a)));

        let tree=crate::new(aabbs);

        TreeRefInd{
            tree,
            orig
        }
    }


    ///Create a [`TreeRefInd`] in parallel. The user provides an empty vec to use to house the
    ///The aabbs created.
    pub fn new_par(
        bots:&'a mut [T],
        mut func:impl FnMut(&mut T)->axgeom::Rect<N>,
        aabbs:&'b mut Vec<BBox<N,&'a mut T>>)->TreeRefInd<'a,'b,N,T> where N:Send+Sync,T:Send+Sync{
        let orig = Ptr(bots as *mut _);

        aabbs.extend(bots.iter_mut().map(|a|crate::bbox(func(a),a)));

        let tree=crate::new_par(aabbs);

        TreeRefInd{
            tree,
            orig
        }
    }


    pub(super) fn into_ptr(self)->TreeRefIndPtr<N,T>{
        
        TreeRefIndPtr{
            tree:TreePtr{
                _inner:unsafe{self.tree.inner.convert()},
                _num_aabbs:self.tree.num_aabbs
            },
            orig:self.orig
        }
    }
}

#[repr(C)]
pub(super) struct TreeRefIndPtr<N:Num,T>{
    pub(super) tree:TreePtr<BBox<N,Ptr<T>>>,
    pub(super) orig:Ptr<[T]>
}


impl<'a,'b, N:Num,T> From<TreeRefInd<'a,'b,N,T>> for Tree<'b, BBox<N,&'a mut T>> {
    #[inline(always)]
    fn from(a: TreeRefInd<'a,'b,N,T>) -> Self {
        a.tree
    }
}


impl<'a,'b, N: Num , T> core::ops::Deref for TreeRefInd<'a,'b, N, T> {
    type Target = Tree<'b, BBox<N, &'a mut T>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl<'a, 'b,N: Num, T> core::ops::DerefMut for TreeRefInd<'a, 'b,N, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl<'a,'b,N:Num,T> TreeRefInd<'a,'b,N,T>{
    /// Retrieve the underlying list of elements.
    /// Unlike [`Tree::get_elements_mut()`] which
    /// returns the aabbs of the tree, this returns the
    /// list of T that each aabb points to.
    #[inline(always)]
    pub fn get_inner_elements_mut(&mut self)->&mut [T]{
        unsafe{&mut *self.orig.0}
    }

    /// Retrieve the underlying list of elements.
    /// Unlike [`Tree::get_elements_mut()`] which
    /// returns the aabbs of the tree, this returns the
    /// list of T that each aabb points to.
    #[inline(always)]
    pub fn get_inner_elements(&self)->&[T]{
        unsafe{&*self.orig.0}
    }
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
    /// let mut tree_builder = broccoli::container::TreeRefIndBuilder::new(&mut aabbs,|a|{
    ///     a.rect
    /// });
    ///
    /// let mut tree=tree_builder.build();
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
            for b in node.into_range().iter_mut() {
                let (x, y) = b.unpack();
                if let Some(d) = func(x, y) {
                    elems.push((Ptr(*y as *mut _), d));
                }
            }
        }
        FilteredElements {
            orig: self.orig,
            elems,
        }
    }

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
    /// let mut builder = broccoli::container::TreeRefIndBuilder::new(&mut aabbs,|a|{
    ///    a.rect
    /// });
    ///
    /// let mut tree=builder.build();
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
        self.tree.find_colliding_pairs_mut(|a, b| {
            let a = a.unpack_inner();
            let b = b.unpack_inner();
            if let Some(extra) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let first = Ptr(*a as *mut T);
                let second = Ptr(*b as *mut T);

                cols.push(ColPairPtr {
                    first,
                    second,
                    extra,
                });
            }
        });

        CollidingPairs {
            cols,
            orig: self.orig,
        }
    }

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
    /// let mut builder = broccoli::container::TreeRefIndBuilder::new(&mut aabbs,|a|{
    ///    a.rect
    /// });
    ///
    /// let mut tree=builder.build_par();
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
    ) -> CollidingPairsPar<T, D> where N:Send+Sync,T:Send+Sync{
        let cols = self.collect_colliding_pairs_par_inner(|a, b| match func(a, b) {
            Some(extra) => Some(ColPairPtr {
                first: Ptr(a as *mut _),
                second: Ptr(b as *mut _),
                extra,
            }),
            None => None,
        });
        CollidingPairsPar {
            cols,
            original: self.orig,
        }
    }

    fn collect_colliding_pairs_par_inner<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> Vec<Vec<D>> where N:Send+Sync,T:Send+Sync {
        let mut handler = crate::query::colfind::builder::from_closure(
            self,
            vec![Vec::new()],
            move |_| (vec![Vec::new()], vec![Vec::new()]),
            move |a, mut b, mut c| {
                a.first_mut().unwrap().append(&mut b.pop().unwrap());
                a.append(&mut c);
            },
            move |c, a, b| {
                if let Some(d) = func(a.unpack_inner(), b.unpack_inner()) {
                    c.first_mut().unwrap().push(d);
                }
            },
        );

        use crate::query::colfind::builder::*;
        self.new_colfind_builder()
            .query_par_ext(&mut handler, &mut SplitterEmpty);
        handler.consume()
    }
}
        