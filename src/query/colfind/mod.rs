//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
pub(crate) mod oned;

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
    
    crate::analyze::sweeper_update(axis, bots);

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
    let mut prevec=crate::util::PreVecMut::new();
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
    pub fn new(
        axis: A,
        vistr: VistrMut<'a, Node<'b, T>>,
    ) -> NotSortedQueryBuilder<'a, 'b, A, T> {
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
    pub fn new(axis: A, vistr: VistrMut<'a, Node<'b, T>>) -> QueryBuilder<'a, 'b, A, T> {
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
