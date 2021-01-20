
//! Provides `collect()` functions through the [`FromSlice`] trait.
//!

use super::*;
use crate::Tree;
use crate::Ptr;
use crate::prelude::*;


///
/// A trait indicating that this is a tree that is composed of pointers
/// to the same underlying slice. Unsafe because the user must guarantee
/// that all the pointers in the tree originate from the same slice.
///
pub unsafe trait FromSlice<'a,'b> where Self::T:HasInner<Inner=&'a mut Self::Inner>  {
    ///The tree must be filled with T, which must have a pointer to `Self::Inner`.
    type T:Aabb<Num=Self::Num>+HasInner<Inner=&'a mut Self::Inner>+'b;
    ///The number type of the aabb.
    type Num:Num;
    ///The type that the pointers are pointing to.
    ///The original slice is composed of this.
    type Inner:'a;


    ///Return a reference to the underlying tree.
    fn get_tree_mut(&mut self)->&mut Tree<'b,Self::T>;



    /// Retrieve the underlying list of elements.
    /// Unlike [`Tree::get_elements_mut()`] which
    /// returns the aabbs of the tree, this returns the
    /// list of T that each aabb points to.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::prelude::*;
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
    /// let bots=tree.get_inner_elements();
    ///
    /// ```
    fn get_inner_elements(&self)->&[Self::Inner];

    /// Retrieve the underlying list of elements.
    /// Unlike [`Tree::get_elements_mut()`] which
    /// returns the aabbs of the tree, this returns the
    /// list of T that each aabb points to.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::prelude::*;
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
    /// let bots=tree.get_inner_elements_mut();
    ///
    /// ```
    fn get_inner_elements_mut(&mut self)->&mut [Self::Inner];

    /// Collect all elements based off of a predicate and return a [`FilteredElements`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::prelude::*;
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///    broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///    broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
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
    fn collect_all<D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&Rect<Self::Num>, &mut Self::Inner) -> Option<D>,
    ) -> FilteredElements<Self::Inner, D>{
        let orig = Ptr(self.get_inner_elements_mut() as *mut _);

        let mut elems = Vec::new();
        for node in self.get_tree_mut().get_nodes_mut().iter_mut() {
            for b in node.into_range().iter_mut() {
                let (x, y) = b.unpack();
                if let Some(d) = func(x, y) {
                    elems.push((Ptr(*y as *mut _), d));
                }
            }
        }
        FilteredElements {
            orig,
            elems,
        }
    }



    /// Find all colliding pairs based on a predicate and return a [`CollidingPairs`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::prelude::*;
    /// let mut aabbs = [
    ///     broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///     broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///     broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
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
    fn collect_colliding_pairs<D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut Self::Inner, &mut Self::Inner) -> Option<D> + Send + Sync,
    ) -> CollidingPairs<Self::Inner, D> {
        let orig = Ptr(self.get_inner_elements_mut() as *mut _);

        let mut cols: Vec<_> = Vec::new();
        self.get_tree_mut().find_colliding_pairs_mut(|a, b| {
            let a = a.unpack_inner();
            let b = b.unpack_inner();
            if let Some(extra) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let first = Ptr(*a as *mut _);
                let second = Ptr(*b as *mut _);

                cols.push(ColPairPtr {
                    first,
                    second,
                    extra,
                });
            }
        });

        CollidingPairs {
            cols,
            orig,
        }
    }


    /// The parallel version of [`FromSlice::collect_colliding_pairs`] that instead
    /// returns a [`CollidingPairsPar`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::prelude::*;
    /// let mut aabbs = [
    ///     broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    ///     broccoli::bbox(broccoli::rect(15, 20, 15, 20), 1),
    ///     broccoli::bbox(broccoli::rect(5, 15, 5, 15), 2),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
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
    fn collect_colliding_pairs_par<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut Self::Inner, &mut Self::Inner) -> Option<D> + Send + Sync + Copy,
    ) -> CollidingPairsPar<Self::Inner, D> where Self::T:Send+Sync,Self::Num:Send+Sync{
        let orig = Ptr(self.get_inner_elements_mut() as *mut _);

        let cols = collect_colliding_pairs_par_inner(self.get_tree_mut(),|a, b| match func(a, b) {
            Some(extra) => Some(ColPairPtr {
                first: Ptr(a as *mut _),
                second: Ptr(b as *mut _),
                extra,
            }),
            None => None,
        });
        CollidingPairsPar {
            cols,
            original: orig,
        }
    }

}


fn collect_colliding_pairs_par_inner<'a,T:Aabb+HasInner<Inner=&'a mut Inner>,Inner:'a,D: Send + Sync>(
    tree:&mut Tree<T>,
    func: impl Fn(&mut Inner, &mut Inner) -> Option<D> + Send + Sync + Copy,
) -> Vec<Vec<D>> where T::Num:Send+Sync,T:Send+Sync {
    let mut handler = crate::query::colfind::builder::from_closure(
        tree,
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
    tree.new_builder()
        .query_par_ext(&mut handler, &mut SplitterEmpty);
    handler.consume()
}


///Contains a filtered list of all elements in the tree from calling [`FromSlice::collect_all`].
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
///CollidingPairs created via [`FromSlice::collect_colliding_pairs`]
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

///CollidingPairsPar created via [`FromSlice::collect_colliding_pairs_par`]
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


