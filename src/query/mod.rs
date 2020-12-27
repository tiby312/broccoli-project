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


///Provides the naive implementation of the [`Tree`] api.
pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a, [T]>,
}


impl<'a,T:Aabb> NaiveQueries for NaiveAlgs<'a,T>{
    type T=T;
    type Num=T::Num;
    fn get_slice_mut(&mut self)->PMut<[T]>{
        self.bots.borrow_mut()
    }

}



impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn from_slice(a: &'a mut [T]) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots: PMut::new(a) }
    }
    #[must_use]
    pub fn new(bots: PMut<'a, [T]>) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots }
    }

    //#[cfg(feature = "nbody")]
    pub fn nbody(&mut self, func: impl FnMut(PMut<T>, PMut<T>)) {
        nbody::naive_mut(self.bots.borrow_mut(), func);
    }
}



///aabb broadphase collision detection
pub mod colfind;

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
pub trait NaiveQueries{
    type T:Aabb<Num=Self::Num>;
    type Num:Num;
    fn get_slice_mut(&mut self) -> PMut<[Self::T]>;
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
