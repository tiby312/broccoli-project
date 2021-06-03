//! Contains code to customize the colliding pair finding algorithm.

use super::*;
use par::ParallelBuilder;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait CollisionHandler {
    type T: Aabb;

    fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>);
}

use self::inner::SplitterVistr;

///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a, 'node: 'a, T: Aabb> {
    par_builder: ParallelBuilder,
    vistr: VistrMut<'a, Node<'node, T>>,
}

impl<'a, 'node: 'a, T: Aabb + Send + Sync> NotSortedQueryBuilder<'a, 'node, T>
where
    T::Num: Send + Sync,
{
    #[inline(always)]
    pub fn query_par(
        self,
        joiner: impl crate::Joinable,
        func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync,
    ) {
        let sweeper = QueryFn::new(func);

        let par = self
            .par_builder
            .build_for_tree_of_height(self.vistr.get_height());

        ParRecurser {
            handler: HandleNoSorted,
            vistr: SplitterVistr::new(sweeper, SplitterVistr::new(SplitterEmpty, self.vistr)),
            par,
            joiner,
            prevec: PreVec::new(),
        }
        .recurse_par(default_axis());
    }
}

impl<'a, 'node: 'a, T: Aabb> NotSortedQueryBuilder<'a, 'node, T> {
    #[inline(always)]
    pub fn new(vistr: VistrMut<'a, Node<'node, T>>) -> NotSortedQueryBuilder<'a, 'node, T> {
        NotSortedQueryBuilder {
            par_builder: ParallelBuilder::new(),
            vistr,
        }
    }

    #[inline(always)]
    pub fn query_with_splitter_seq<S: Splitter>(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: S,
    ) -> S {
        let mut sweeper = QueryFnMut::new(func);

        let mut r = Recurser {
            handler: &mut HandleNoSorted,
            sweeper: &mut sweeper,
            prevec: &mut PreVec::new(),
        };

        r.recurse_seq(default_axis(), SplitterVistr::new(splitter, self.vistr))
    }

    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let mut sweeper = QueryFnMut::new(func);

        let mut r = Recurser {
            handler: &mut HandleNoSorted,
            sweeper: &mut sweeper,
            prevec: &mut PreVec::new(),
        };

        r.recurse_seq(
            default_axis(),
            SplitterVistr::new(SplitterEmpty, self.vistr),
        );
    }
}

///Builder for a query on a DinoTree.
pub struct QueryBuilder<'a, 'node: 'a, T: Aabb> {
    par_builder: ParallelBuilder,
    vistr: VistrMut<'a, Node<'node, T>>,
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

impl<'a, 'node: 'a, T: Aabb + Send + Sync> QueryBuilder<'a, 'node, T>
where
    T::Num: Send + Sync,
{
    ///Perform the query in parallel, switching to sequential as specified
    ///by the [`QueryBuilder::with_switch_height()`]
    #[inline(always)]
    pub fn query_par(
        self,
        joiner: impl Joinable,
        func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync,
    ) {
        let sweeper = QueryFn::new(func);

        let par = self
            .par_builder
            .build_for_tree_of_height(self.vistr.get_height());

        ParRecurser {
            handler: HandleSorted,
            vistr: SplitterVistr::new(sweeper, SplitterVistr::new(SplitterEmpty, self.vistr)),
            par,
            joiner,
            prevec: PreVec::new(),
        }
        .recurse_par(default_axis());
    }

    /// An extended version of `find_colliding_pairs`. where the user can supply
    /// callbacks to when new worker tasks are spawned and joined by `rayon`.
    /// Allows the user to potentially collect some aspect of every aabb collision in parallel.
    ///
    /// `sweeper` : The splitter div/add functions will be called every time a new parallel recurse is started.
    /// `splitter`: The splitter div/add will be called at every level of recursion.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,RayonJoin,rect,bbox,Consumer};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),1u8)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let mut handler=broccoli::colfind_from_closure(
    ///     &tree,
    ///     Vec::new(),
    ///     |_|(Vec::new(),Vec::new()),        //Start a new thread
    ///     |a,mut b,mut c|{a.append(&mut b);a.append(&mut c)}, //Combine two threads
    ///     |v,a,b|v.push((*a.unpack_inner(),*b.unpack_inner())), //Handle a collision
    /// );
    ///
    /// let (handler,_)=tree.new_colfind_builder().query_par_ext(
    ///     RayonJoin,
    ///     handler,
    ///     broccoli::build::SplitterEmpty
    /// );
    ///
    /// let intersections=handler.consume();
    ///
    /// assert_eq!(intersections.len(),1);
    ///```
    #[inline(always)]
    pub fn query_par_ext<
        S: Splitter + Send + Sync,
        C: CollisionHandler<T = T> + Splitter + Send + Sync,
    >(
        self,
        joiner: impl Joinable,
        sweeper: C,
        splitter: S,
    ) -> (C, S) {
        let par = self
            .par_builder
            .build_for_tree_of_height(self.vistr.get_height());

        ParRecurser {
            handler: HandleSorted,
            vistr: SplitterVistr::new(sweeper, SplitterVistr::new(splitter, self.vistr)),
            par,
            joiner,
            prevec: PreVec::new(),
        }
        .recurse_par(default_axis())
    }
}

impl<'a, 'node: 'a, T: Aabb> QueryBuilder<'a, 'node, T> {
    ///Create the builder.
    #[inline(always)]
    #[must_use]
    pub fn new(vistr: VistrMut<'a, Node<'node, T>>) -> QueryBuilder<'a, 'node, T> {
        QueryBuilder {
            par_builder: ParallelBuilder::new(),
            vistr,
        }
    }

    ///Choose a custom height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this option is ignored.
    #[inline(always)]
    #[must_use]
    pub fn with_switch_height(mut self, height: usize) -> Self {
        self.par_builder.with_switch_height(height);
        self
    }

    ///Perform the query sequentially.
    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let mut sweeper = QueryFnMut::new(func);

        let mut r = Recurser {
            handler: &mut HandleSorted,
            sweeper: &mut sweeper,
            prevec: &mut PreVec::new(),
        };

        r.recurse_seq(
            default_axis(),
            SplitterVistr::new(SplitterEmpty, self.vistr),
        );
    }

    ///Perform the query sequentially with splitter functions getting called at every level of
    ///recursion.
    #[inline(always)]
    pub fn query_with_splitter_seq<S: Splitter>(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: S,
    ) -> S {
        let mut sweeper = QueryFnMut::new(func);

        let mut r = Recurser {
            handler: &mut HandleSorted,
            sweeper: &mut sweeper,
            prevec: &mut PreVec::new(),
        };

        r.recurse_seq(default_axis(), SplitterVistr::new(splitter, self.vistr))
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
