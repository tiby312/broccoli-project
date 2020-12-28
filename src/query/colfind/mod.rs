//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
mod oned;

use self::inner::*;
use self::node_handle::*;
use super::tools;
use crate::query::inner_prelude::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti {
    type T: Aabb;

    fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>);
}



use super::NaiveComparable;
pub fn assert_query<'a,K:NaiveComparable<'a>>(tree:&mut K){
    use core::ops::Deref;
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_dino = Vec::new();
    tree.get_tree().find_colliding_pairs_mut(|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_dino.push(k);
    });

    let mut res_naive = Vec::new();
    query_naive_mut(tree.get_elements_mut(),|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_naive.push(k);
    });

    res_naive.sort();
    res_dino.sort();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Naive algorithm.
pub fn query_naive_mut<T: Aabb>(bots: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
    tools::for_every_pair(bots, move |a, b| {
        if a.get().intersects_rect(b.get()) {
            func(a, b);
        }
    });
}

///Sweep and prune algorithm.
pub fn query_sweep_mut<T: Aabb>(
    axis: impl Axis,
    bots: &mut [T],
    func: impl FnMut(PMut<T>, PMut<T>),
) {
    crate::util::sweeper_update(axis, bots);

    struct Bl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> {
        func: F,
        _p: PhantomData<T>,
    }

    impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> ColMulti for Bl<T, F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
            (self.func)(a, b);
        }
    }
    let mut prevec = crate::util::PreVecMut::with_capacity(2048);
    let bots = PMut::new(bots);
    oned::find_2d(
        &mut prevec,
        axis,
        bots,
        &mut Bl {
            func,
            _p: PhantomData,
        },
    );
}

///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a, 'b: 'a, A: Axis, T: Aabb> {
    switch_height: usize,
    axis: A,
    vistr: VistrMut<'a, Node<'b, T>>,
}

impl<'a, 'b: 'a, A: Axis, T: Aabb + Send + Sync> NotSortedQueryBuilder<'a, 'b, A, T>
where
    T::Num: Send + Sync,
{
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync) {
        let b = QueryFn::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        let par = par::compute_level_switch_sequential(self.switch_height, self.vistr.get_height());
        ColFindRecurser::new().recurse_par(
            self.axis,
            par,
            &mut sweeper,
            self.vistr,
            &mut SplitterEmpty,
        );
    }
}

impl<'a, 'b: 'a, A: Axis, T: Aabb> NotSortedQueryBuilder<'a, 'b, A, T> {
    #[inline(always)]
    fn new(axis: A, vistr: VistrMut<'a, Node<'b, T>>) -> NotSortedQueryBuilder<'a, 'b, A, T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        NotSortedQueryBuilder {
            switch_height,
            axis,
            vistr,
        }
    }

    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: &mut impl Splitter,
    ) {
        let b = QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, splitter);
    }

    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let b = QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, &mut SplitterEmpty);
    }
}

///Builder for a query on a DinoTree.
pub struct QueryBuilder<'a, 'b: 'a, A: Axis, T: Aabb> {
    switch_height: usize,
    axis: A,
    vistr: VistrMut<'a, Node<'b, T>>,
}

impl<'a, 'b: 'a, A: Axis, T: Aabb + Send + Sync> QueryBuilder<'a, 'b, A, T>
where
    T::Num: Send + Sync,
{
    ///Perform the query in parallel
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync) {
        let b = QueryFn::new(func);
        let mut sweeper = HandleSorted::new(b);

        let height = self.vistr.get_height();
        let switch_height = self.switch_height;
        let par = par::compute_level_switch_sequential(switch_height, height);
        ColFindRecurser::new().recurse_par(
            self.axis,
            par,
            &mut sweeper,
            self.vistr,
            &mut SplitterEmpty,
        );
    }

    /// An extended version of `find_colliding_pairs`. where the user can supply
    /// callbacks to when new worker tasks are spawned and joined by `rayon`.
    /// Allows the user to potentially collect some aspect of every aabb collision in parallel.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),1u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let intersections=tree.new_colfind_builder().query_par_ext(
    ///     |_|(Vec::new(),Vec::new()),              //Start a new thread
    ///     |a,mut b,mut c|{a.append(&mut b);a.append(&mut c)},  //Combine two threads
    ///     |v,a,b|v.push((*a.unpack_inner(),*b.unpack_inner())),     //What to do for each intersection for a thread.
    ///     Vec::new()                  //Starting thread
    /// );
    ///
    /// assert_eq!(intersections.len(),1);
    ///```
    pub fn query_par_ext<B: Send + Sync>(
        self,
        split: impl Fn(&mut B) -> (B, B) + Send + Sync + Copy,
        fold: impl Fn(&mut B, B, B) + Send + Sync + Copy,
        collision: impl Fn(&mut B, PMut<T>, PMut<T>) + Send + Sync + Copy,
        acc: B,
    ) -> B
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        struct Foo<T, A, B, C, D> {
            _p: PhantomData<T>,
            acc: A,
            split: B,
            fold: C,
            collision: D,
        }

        impl<T: Aabb, A, B, C, D: Fn(&mut A, PMut<T>, PMut<T>)> ColMulti for Foo<T, A, B, C, D> {
            type T = T;

            #[inline(always)]
            fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>) {
                (self.collision)(&mut self.acc, a, b)
            }
        }
        impl<T, A, B: Fn(&mut A) -> (A, A) + Copy, C: Fn(&mut A, A, A) + Copy, D: Copy> Splitter
            for Foo<T, A, B, C, D>
        {
            #[inline(always)]
            fn div(&mut self) -> (Self, Self) {
                let (acc1, acc2) = (self.split)(&mut self.acc);
                (
                    Foo {
                        _p: PhantomData,
                        acc: acc1,
                        split: self.split,
                        fold: self.fold,
                        collision: self.collision,
                    },
                    Foo {
                        _p: PhantomData,
                        acc: acc2,
                        split: self.split,
                        fold: self.fold,
                        collision: self.collision,
                    },
                )
            }
            #[inline(always)]
            fn add(&mut self, a: Self, b: Self) {
                (self.fold)(&mut self.acc, a.acc, b.acc)
            }
        }

        let foo = Foo {
            _p: PhantomData,
            acc,
            split,
            fold,
            collision,
        };

        self.query_splitter_par(foo).acc
    }

    ///Trait version of [`QueryBuilder::query_par_ext`].
    ///The user has more control using this version of the query.
    ///The splitter will split and add at every level.
    ///The clos will split and add only at levels that are handled in parallel.
    ///The leaf_start function will get called right before sequential processing.
    ///The leaf end function will get called when the sequential processing finishes.
    ///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
    #[inline(always)]
    pub fn query_splitter_par<C: ColMulti<T = T> + Splitter + Send + Sync>(self, clos: C) -> C {
        let height = self.vistr.get_height();

        let par = par::compute_level_switch_sequential(self.switch_height, height);

        let mut sweeper = HandleSorted::new(clos);
        ColFindRecurser::new().recurse_par(
            self.axis,
            par,
            &mut sweeper,
            self.vistr,
            &mut SplitterEmpty,
        );

        sweeper.func
    }
}

impl<'a, 'b: 'a, A: Axis, T: Aabb> QueryBuilder<'a, 'b, A, T> {
    ///Create the builder.
    #[inline(always)]
    #[must_use]
    fn new(axis: A, vistr: VistrMut<'a, Node<'b, T>>) -> QueryBuilder<'a, 'b, A, T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        QueryBuilder {
            switch_height,
            axis,
            vistr,
        }
    }

    ///Choose a custom height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this option is ignored.
    #[inline(always)]
    #[must_use]
    pub fn with_switch_height(mut self, height: usize) -> Self {
        self.switch_height = height;
        self
    }

    ///Perform the query sequentially.
    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let b = QueryFnMut::new(func);
        let mut sweeper = HandleSorted::new(b);
        let mut splitter = SplitterEmpty;

        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, &mut splitter);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: &mut impl Splitter,
    ) {
        let b = QueryFnMut::new(func);

        let mut sweeper = HandleSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, splitter);
    }
}

pub(super) struct QueryFnMut<T, F>(F, PhantomData<T>);
impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> QueryFnMut<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFnMut<T, F> {
        QueryFnMut(func, PhantomData)
    }
}

impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> ColMulti for QueryFnMut<T, F> {
    type T = T;
    #[inline(always)]
    fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
        self.0(a, b);
    }
}
impl<T, F> Splitter for QueryFnMut<T, F> {
    #[inline(always)]
    fn div(&mut self) -> (Self, Self) {
        unreachable!()
    }
    #[inline(always)]
    fn add(&mut self, _: Self, _: Self) {
        unreachable!()
    }
}

pub(super) struct QueryFn<T, F>(F, PhantomData<T>);
impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> QueryFn<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFn<T, F> {
        QueryFn(func, PhantomData)
    }
}
impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> ColMulti for QueryFn<T, F> {
    type T = T;

    #[inline(always)]
    fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
        self.0(a, b);
    }
}

impl<T, F: Clone> Splitter for QueryFn<T, F> {
    #[inline(always)]
    fn div(&mut self) -> (Self, Self) {
        (
            QueryFn(self.0.clone(), PhantomData),
            QueryFn(self.0.clone(), PhantomData),
        )
    }
    #[inline(always)]
    fn add(&mut self, _: Self, _: Self) {}
}

use super::Queries;
impl<'a,K:Queries<'a>> ColfindQuery<'a> for K{}

pub trait ColfindQuery<'a>: Queries<'a>{

    /// Find all aabb intersections and return a PMut<T> of it. Unlike the regular `find_colliding_pairs_mut`, this allows the
    /// user to access a read only reference of the AABB.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.find_colliding_pairs_mut(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        QueryBuilder::new(self.axis(), self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    /// The parallel version of [`ColfindQuery::find_colliding_pairs_mut`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.find_colliding_pairs_mut_par(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Send + Sync + Clone,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        QueryBuilder::new(self.axis(), self.vistr_mut()).query_par(move |a, b| func(a, b));
    }

    /// For analysis, allows the user to query with custom settings
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let builder=tree.new_colfind_builder();
    /// let builder=builder.with_switch_height(4);
    /// builder.query_seq(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn new_colfind_builder<'c>(&'c mut self) -> QueryBuilder<'c, 'a, Self::A, Self::T> {
        QueryBuilder::new(self.axis(), self.vistr_mut())
    }



}




///Queries that can be performed on a tree that is not sorted
///These functions are not documented since they match the same
///behavior as those in the [`Queries`] trait.
pub trait NotSortedQueries<'a> {
    type A: Axis;
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, Self::T>>;

    #[must_use]
    fn vistr(&self) -> Vistr<Node<'a, Self::T>>;

    #[must_use]
    fn axis(&self) -> Self::A;

    fn new_colfind_builder<'c>(&'c mut self) -> NotSortedQueryBuilder<'c, 'a, Self::A, Self::T> {
        NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
    }

    fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_seq(move |a, b| func(a, b));
    }

    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Clone + Send + Sync,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_par(move |a, b| func(a, b));
    }
}

