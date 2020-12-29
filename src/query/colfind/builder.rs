//! Contains code to customize the colliding pair finding algorithm.

use super::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait CollisionHandler {
    type T: Aabb;

    fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>);
}

///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a, 'b: 'a,T: Aabb> {
    switch_height: usize,
    vistr: VistrMut<'a, Node<'b, T>>,
}

impl<'a, 'b: 'a,  T: Aabb + Send + Sync> NotSortedQueryBuilder<'a, 'b,  T>
where
    T::Num: Send + Sync,
{
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync) {
        let b = QueryFn::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        let par = par::compute_level_switch_sequential(self.switch_height, self.vistr.get_height());
        ColFindRecurser::new().recurse_par(
            default_axis(),
            par,
            &mut sweeper,
            self.vistr,
            &mut SplitterEmpty,
        );
    }
}

impl<'a, 'b: 'a,  T: Aabb> NotSortedQueryBuilder<'a, 'b,  T> {
    #[inline(always)]
    pub(super) fn new(
        vistr: VistrMut<'a, Node<'b, T>>,
    ) -> NotSortedQueryBuilder<'a, 'b,  T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        NotSortedQueryBuilder {
            switch_height,
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
        ColFindRecurser::new().recurse_seq(default_axis(), &mut sweeper, self.vistr, splitter);
    }

    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let b = QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        ColFindRecurser::new().recurse_seq(default_axis(), &mut sweeper, self.vistr, &mut SplitterEmpty);
    }
}

///Builder for a query on a DinoTree.
pub struct QueryBuilder<'a, 'b: 'a, T: Aabb> {
    switch_height: usize,
    vistr: VistrMut<'a, Node<'b, T>>,
}

///Simple trait that consumes itself to produce a value.
pub trait Consumer {
    type Item;
    fn consume(self) -> Self::Item;
}

///Create an object to satisfy [`QueryBuilder::query_par_ext`].
pub fn from_closure<A: Send, T: Aabb + Send>(
    _tree: &crate::Tree<T>,
    acc: A,
    split: impl Fn(&mut A) -> (A, A) + Copy + Send,
    fold: impl Fn(&mut A, A, A) + Copy + Send,
    collision: impl Fn(&mut A, PMut<T>, PMut<T>) + Copy + Send,
) -> impl CollisionHandler<T = T> + Splitter + Send + Consumer<Item = A> {
    struct QueryParSplitter<T, A, B, C, D> {
        pub _p: PhantomData<T>,
        pub acc: A,
        pub split: B,
        pub fold: C,
        pub collision: D,
    }
    impl<T, A, B, C, D> Consumer for QueryParSplitter<T, A, B, C, D> {
        type Item = A;
        fn consume(self) -> Self::Item {
            self.acc
        }
    }
    impl<T: Aabb, A, B, C, D> CollisionHandler for QueryParSplitter<T, A, B, C, D>
    where
        D: Fn(&mut A, PMut<T>, PMut<T>),
    {
        type T = T;

        #[inline(always)]
        fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>) {
            (self.collision)(&mut self.acc, a, b)
        }
    }

    impl<T, A, B, C, D> Splitter for QueryParSplitter<T, A, B, C, D>
    where
        B: Fn(&mut A) -> (A, A) + Copy,
        C: Fn(&mut A, A, A) + Copy,
        D: Copy,
    {
        #[inline(always)]
        fn div(&mut self) -> (Self, Self) {
            let (acc1, acc2) = (self.split)(&mut self.acc);
            (
                QueryParSplitter {
                    _p: PhantomData,
                    acc: acc1,
                    split: self.split,
                    fold: self.fold,
                    collision: self.collision,
                },
                QueryParSplitter {
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

    QueryParSplitter {
        _p: PhantomData,
        acc,
        split,
        fold,
        collision,
    }
}

impl<'a, 'b: 'a, T: Aabb + Send + Sync> QueryBuilder<'a, 'b,  T>
where
    T::Num: Send + Sync,
{
    ///Perform the query in parallel, switching to sequential as specified
    ///by the [`QueryBuilder::with_switch_height()`]
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync) {
        let b = QueryFn::new(func);
        let mut sweeper = HandleSorted::new(b);

        let height = self.vistr.get_height();
        let switch_height = self.switch_height;
        let par = par::compute_level_switch_sequential(switch_height, height);
        ColFindRecurser::new().recurse_par(
            default_axis(),
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
    ///Query in parallel, calling splitter functions every time a new parallel recurse is started.
    ///The splitter functions wont be called by sequential recursions.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,query,query::colfind::builder::Consumer};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),1u8)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let handler=query::colfind::builder::from_closure(
    ///     &tree,
    ///     Vec::new(),
    ///     |_|(Vec::new(),Vec::new()),        //Start a new thread
    ///     |a,mut b,mut c|{a.append(&mut b);a.append(&mut c)}, //Combine two threads
    ///     |v,a,b|v.push((*a.unpack_inner(),*b.unpack_inner())), //Handle a collision
    /// );
    ///
    /// let intersections=tree.new_colfind_builder().query_par_ext(
    ///     handler
    /// ).consume();
    ///
    /// assert_eq!(intersections.len(),1);
    ///```
    #[inline(always)]
    pub fn query_par_ext<C: CollisionHandler<T = T> + Splitter + Send + Sync>(self, clos: C) -> C {
        let height = self.vistr.get_height();

        let par = par::compute_level_switch_sequential(self.switch_height, height);

        let mut sweeper = HandleSorted::new(clos);
        ColFindRecurser::new().recurse_par(
            default_axis(),
            par,
            &mut sweeper,
            self.vistr,
            &mut SplitterEmpty,
        );

        sweeper.func
    }
}

impl<'a, 'b: 'a,  T: Aabb> QueryBuilder<'a, 'b, T> {
    ///Create the builder.
    #[inline(always)]
    #[must_use]
    pub(super) fn new(vistr: VistrMut<'a, Node<'b, T>>) -> QueryBuilder<'a, 'b,  T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        QueryBuilder {
            switch_height,
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

        ColFindRecurser::new().recurse_seq(default_axis(), &mut sweeper, self.vistr, &mut splitter);
    }

    ///Perform the query sequentially with splitter functions getting called at every level of
    ///recursion.
    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: &mut impl Splitter,
    ) {
        let b = QueryFnMut::new(func);

        let mut sweeper = HandleSorted::new(b);
        ColFindRecurser::new().recurse_seq(default_axis(), &mut sweeper, self.vistr, splitter);
    }
}

struct QueryFnMut<T, F>(F, PhantomData<T>);
impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> QueryFnMut<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFnMut<T, F> {
        QueryFnMut(func, PhantomData)
    }
}

impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> CollisionHandler for QueryFnMut<T, F> {
    type T = T;
    #[inline(always)]
    fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
        self.0(a, b);
    }
}

struct QueryFn<T, F>(F, PhantomData<T>);

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

impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> QueryFn<T, F> {
    #[inline(always)]
    pub fn new(func: F) -> QueryFn<T, F> {
        QueryFn(func, PhantomData)
    }
}

impl<T: Aabb, F: Fn(PMut<T>, PMut<T>)> CollisionHandler for QueryFn<T, F> {
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
