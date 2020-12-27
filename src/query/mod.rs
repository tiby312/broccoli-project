//! Module contains query related structs.

mod inner_prelude {
    pub use crate::inner_prelude::*;
    pub use alloc::vec::Vec;
    pub use axgeom;
    pub use axgeom::Rect;
    pub use compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
}

pub use naive::NaiveAlgs;
mod naive;


///aabb broadphase collision detection
pub mod colfind;
use colfind::NotSortedQueryBuilder;

///Provides functionality to draw the dividers of [`Tree`].
pub mod graphics;

///Contains all k_nearest code.
pub mod knearest;

///Contains all raycast code.
pub mod raycast;

///Allows user to intersect the tree with a seperate group of bots.
pub mod intersect_with;

pub mod nbody;

///Contains rect code.
pub mod rect;


///Contains misc tools
mod tools;

use self::inner_prelude::*;

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
        query::colfind::NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_seq(move |a, b| func(a, b));
    }

    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Clone + Send + Sync,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        query::colfind::NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_par(move |a, b| func(a, b));
    }
}

///Query functions. User defines `vistr()` functions, and the query functions
///are automatically provided by this trait.
pub trait Queries<'a> {
    type A: Axis;
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
    ///    *b.unpack_inner()+=1;    
    /// }
    /// assert_eq!(bots[0].inner,1);
    ///```
    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// let mut test = Vec::new();
    /// for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.range.iter()){
    ///    test.push(b);
    /// }
    /// assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[must_use]
    fn vistr(&self) -> Vistr<Node<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,analyze};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use axgeom::Axis;
    /// assert!(tree.axis().is_equal_to(analyze::default_axis()));
    ///```
    #[must_use]
    fn axis(&self) -> Self::A;

}
