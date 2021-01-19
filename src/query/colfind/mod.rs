//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
mod oned;

use self::inner::*;
use self::node_handle::*;
use super::tools;
use crate::query::inner_prelude::*;

pub mod builder;
use self::builder::CollisionHandler;
use self::builder::NotSortedQueryBuilder;
use self::builder::QueryBuilder;

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_query<T: Aabb>(tree: &mut crate::Tree<T>) {
    use core::ops::Deref;
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_dino = Vec::new();
    tree.find_colliding_pairs_mut(|a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_dino.push(k);
    });

    let mut res_naive = Vec::new();
    query_naive_mut(tree.get_elements_mut(), |a, b| {
        let a = into_ptr_usize(a.deref());
        let b = into_ptr_usize(b.deref());
        let k = if a < b { (a, b) } else { (b, a) };
        res_naive.push(k);
    });

    res_naive.sort_unstable();
    res_dino.sort_unstable();

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Naive implementation
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

    impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> CollisionHandler for Bl<T, F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
            (self.func)(a, b);
        }
    }
    let mut prevec = crate::util::PreVec::with_capacity(2048);
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

use super::Queries;

///Colfind functions that can be called on a tree.
pub trait ColfindQuery<'a>: Queries<'a> {
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
        QueryBuilder::new(self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    
    fn find_colliding_pairs_3d_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) 
        where Self::T:Aabb3d    {
        self.find_colliding_pairs_mut(|a,b|{
            if a.get_z().intersects(b.get_z()){
                func(a,b);
            }
        })
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
        QueryBuilder::new(self.vistr_mut()).query_par(move |a, b| func(a, b));
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
    /// let builder=tree.new_builder();
    /// let builder=builder.with_switch_height(4);
    /// builder.query_seq(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn new_builder<'c>(&'c mut self) -> QueryBuilder<'c, 'a, Self::T> {
        QueryBuilder::new(self.vistr_mut())
    }

    
}

///Queries that can be performed on a tree that is not sorted
///These functions are not documented since they match the same
///behavior as those in the [`Queries`] trait.
pub trait NotSortedQueries<'a> {
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, Self::T>>;

    #[must_use]
    fn vistr(&self) -> Vistr<Node<'a, Self::T>>;

    fn new_colfind_builder<'c>(&'c mut self) -> NotSortedQueryBuilder<'c, 'a, Self::T> {
        NotSortedQueryBuilder::new(self.vistr_mut())
    }

    fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        NotSortedQueryBuilder::new(self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Clone + Send + Sync,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        NotSortedQueryBuilder::new(self.vistr_mut()).query_par(move |a, b| func(a, b));
    }
}
