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
    ///Sorts the bots.
    #[inline(always)]
    fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
        let sclosure = move |a: &I, b: &I| -> core::cmp::Ordering {
            let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
            if p1 > p2 {
                return core::cmp::Ordering::Greater;
            }
            core::cmp::Ordering::Less
        };

        collision_botids.sort_unstable_by(sclosure);
    }

    sweeper_update(axis, bots);

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

    let mut s = oned::Sweeper::new();
    let bots = PMut::new(bots);
    s.find_2d(
        axis,
        bots,
        &mut Bl {
            func,
            _p: PhantomData,
        },
    );
}

///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a, A: Axis, N: Node> {
    switch_height: usize,
    axis: A,
    vistr: VistrMut<'a, N>,
}

impl<'a, A: Axis, N: Node + Send + Sync> NotSortedQueryBuilder<'a, A, N>
where
    N::T: Send + Sync,
{
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<N::T>, PMut<N::T>) + Clone + Send + Sync) {
        let b = inner::QueryFn::new(func);
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

impl<'a, A: Axis, N: Node> NotSortedQueryBuilder<'a, A, N> {
    #[inline(always)]
    pub fn new(axis: A, vistr: VistrMut<'a, N>) -> NotSortedQueryBuilder<'a, A, N> {
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
        func: impl FnMut(PMut<N::T>, PMut<N::T>),
        splitter: &mut impl Splitter,
    ) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, splitter);
    }

    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<N::T>, PMut<N::T>)) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, &mut SplitterEmpty);
    }
}

///Builder for a query on a DinoTree.
pub struct QueryBuilder<'a, A: Axis, N: Node> {
    switch_height: usize,
    axis: A,
    vistr: VistrMut<'a, N>,
}

impl<'a, A: Axis, N: Node + Send + Sync> QueryBuilder<'a, A, N>
where
    N::T: Send + Sync,
{
    ///Perform the query in parallel
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<N::T>, PMut<N::T>) + Clone + Send + Sync) {
        let b = inner::QueryFn::new(func);
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
    ///     |_|Vec::new(),              //Start a new thread
    ///     |a,mut b|a.append(&mut b),  //Combine two threads
    ///     |v,a,b|v.push((*a.unpack_inner(),*b.unpack_inner())),     //What to do for each intersection for a thread.
    ///     Vec::new()                  //Starting thread
    /// );
    ///
    /// assert_eq!(intersections.len(),1);
    ///```
    pub fn query_par_ext<B: Send + Sync>(
        self,
        split: impl Fn(&mut B) -> B + Send + Sync + Copy,
        fold: impl Fn(&mut B, B) + Send + Sync + Copy,
        collision: impl Fn(&mut B, PMut<N::T>, PMut<N::T>) + Send + Sync + Copy,
        acc: B,
    ) -> B
    where
        N::T: Send + Sync,
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
            fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>) {
                (self.collision)(&mut self.acc, a, b)
            }
        }
        impl<T, A, B: Fn(&mut A) -> A + Copy, C: Fn(&mut A, A) + Copy, D: Copy> Splitter
            for Foo<T, A, B, C, D>
        {
            fn div(&mut self) -> Self {
                let acc = (self.split)(&mut self.acc);
                Foo {
                    _p: PhantomData,
                    acc,
                    split: self.split,
                    fold: self.fold,
                    collision: self.collision,
                }
            }

            fn add(&mut self, b: Self) {
                (self.fold)(&mut self.acc, b.acc)
            }

            fn node_start(&mut self) {}

            fn node_end(&mut self) {}
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

    ///The user has more control using this version of the query.
    ///The splitter will split and add at every level.
    ///The clos will split and add only at levels that are handled in parallel.
    ///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
    #[inline(always)]
    pub(crate) fn query_splitter_par<C: ColMulti<T = N::T> + Splitter + Send + Sync>(self, clos: C) -> C {
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

impl<'a, A: Axis, N: Node> QueryBuilder<'a, A, N> {
    ///Create the builder.
    #[inline(always)]
    #[must_use]
    pub fn new(axis: A, vistr: VistrMut<'a, N>) -> QueryBuilder<'a, A, N> {
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
    pub fn query_seq(self, func: impl FnMut(PMut<N::T>, PMut<N::T>)) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleSorted::new(b);
        let mut splitter = SplitterEmpty;

        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, &mut splitter);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<N::T>, PMut<N::T>),
        splitter: &mut impl Splitter,
    ) {
        let b = inner::QueryFnMut::new(func);

        let mut sweeper = HandleSorted::new(b);
        ColFindRecurser::new().recurse_seq(self.axis, &mut sweeper, self.vistr, splitter);
    }
    
}
