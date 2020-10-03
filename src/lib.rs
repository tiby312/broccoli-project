//! Broccoli is a broadphase collision detection library. 
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//! Uses `no_std`, but uses the `alloc` crate.
//! Please see the [broccoli-book](https://broccoli-book.netlify.com) which is a work in-progress high level explanation and analysis
//! of this crate.
//!
//! ### Screenshot
//!
//! Screenshot from the broccoli_demo inner project from the [github repo of this crate](https://github.com/tiby312/broccoli).
//! ![](https://raw.githubusercontent.com/tiby312/broccoli/master/assets/screenshot.gif)
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are exlored more in depth in the book mentioned in the overview section.
//!
//! + `(Rect<N>,&mut T)` *recommended
//! + `(Rect<N>,T)`
//! + `&mut (Rect<N>,T)`
//!
//! ### User Protection
//!
//! A lot is done to forbid the user from violating the invariants of the tree once constructed
//! while still allowing them to mutate parts of each element of the tree. The user can mutably traverse
//! the tree but the mutable references returns are hidden behind the `PMut<T>` type that forbids
//! mutating the whole element.
//!
//! ### Unsafety
//! 
//! Raw pointers are used for the container types in the container module
//! and for caching the results of finding colliding pairs. 
//! 
//! `MultiRectMut` uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the Aabb trait is unsafe.
//!
//! ### Name
//! If you shorten "broadphase collision" to "broad colli" and say it fast, it sounds like broccoli.
//! Broccoli also have tree like properties and broccoli uses a tree data structure.
//!

#![no_std]

#[macro_use]
extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

///axgeom crate is re-exported for easy access to the `Rect<T>` type which is what a `BBox` is composed of.
extern crate axgeom;

//TODO get rid of
mod inner_prelude {
    pub(crate) use super::*;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::analyze::*;
    pub(crate) use crate::query::Queries;
    pub use alloc::vec::Vec;
    pub use axgeom::*;
    pub(crate) use compt::Visitor;
    pub use core::iter::*;
    pub use core::marker::PhantomData;
    pub(crate) use crate::bbox::*;
    pub(crate) use crate::pmut::*;
    pub(crate) use crate::tree::par;
    pub(crate) use crate::tree::*;
}

pub mod query;


///Contains generic code used in all dinotree versions
//pub use self::tree::{DinoTree,analyze,collectable,owned,DefaultA,default_axis};
//pub use self::tree::*;
mod tree;
pub use tree::*;


///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

pub mod pmut;

///A collection of different bounding box containers.
mod bbox;
pub use crate::bbox::*;

///Generic slice utillity functions.
pub mod util;


use axgeom::Rect;


pub mod prelude{
    pub use crate::query::RayCastResult;
    pub use axgeom::Rect;
    pub use axgeom::rect;
    pub use crate::Num;
    pub use crate::Aabb;
    pub use crate::HasInner;
    pub use crate::query::Queries;
    pub use crate::query::QueriesInner;
    pub use crate::bbox::bbox;
    pub use crate::bbox::BBox;
}

///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait Num: Ord + Copy + Send + Sync {}
impl<T> Num for T where T: Ord + Copy + Send + Sync {}

///Trait to signify that this object has an axis aligned bounding box.
///get() must return a aabb with the same value in it while the element
///is in the dinotree. This is hard for the user not to do, this the user
///does not have &mut self, and the aabb is implied to belong to self.
///But it is still possible through the use of static objects or RefCell/ Mutex, etc.
///Using this type of methods the user could make different calls to get()
///return different aabbs.
///This is unsafe since we allow query algorithms to assume the following:
///If two object's aabb's don't intersect, then they can be mutated at the same time.
pub unsafe trait Aabb {
    type Num: Num;
    fn get(&self) -> &Rect<Self::Num>;
}

unsafe impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}



///Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
///and at the same time return a mutable reference to a seperate inner section.
///
///The trait in unsafe since an incorrect implementation could allow the user to get mutable
///references to each element in the tree allowing them to swap them and thus violating
///invariants of the tree. This can be done if the user were to implement with type Inner=Self
///
///We have no easy way to ensure that the Inner type only points to the inner portion of a AABB
///so we mark this trait as unsafe.
//TODO make this not unsafe
pub unsafe trait HasInner: Aabb {
    type Inner;
    #[inline(always)]
    fn inner_mut(&mut self) -> &mut Self::Inner {
        self.get_inner_mut().1
    }
    #[inline(always)]
    fn inner(&self) -> &Self::Inner {
        self.get_inner().1
    }
    fn get_inner(&self) -> (&Rect<Self::Num>, &Self::Inner);
    fn get_inner_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner);
}
