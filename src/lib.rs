//! Broccoli is a broadphase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are exlored more in depth in the [broccoli book](https://tiby312.github.io/broccoli_report)
//!
//! + `(Rect<N>,&mut T)` (Semi-Direct) *recommended
//! + `(Rect<N>,T)` (Direct)
//! + `&mut (Rect<N>,T)` (Indirect)
//!
//! ### There are so many Tree types which one do I use?
//!
//! Different variations are provided to give the user
//! more options on what kind of characteristics they want.
//! i.e. less memory vs faster vs less unsafe code.
//! Some also unlock more functions at the cost of a more restrictive api.
//! The [`collections`] module goes into more depth as well as the book mentioned above.
//!
//! TL;DR use [`collections::Tree`] and fill it with `BBox<N,&mut T>`.
//!
//! ### Parallelism
//!
//! Parallel versions of construction and colliding pair finding functions
//! are provided. They use `rayon` under the hood which uses work stealing to
//! parallelize divide and conquer style recursive functions.
//!
//! ### Floating Point
//!
//! The [`Num`] trait used for the aabbs inserted into the tree must implement [`Ord`],
//! thus you can't add `f32` or `f64`. However, you can use the `ordered_float` crate, which 
//! is re-exported at [`axgeom::ordered_float`].
//! The broccoli book mentioned above compares performance. For best performance,
//! you will likely want to convert the floats to integers anyway.
//!
//! ### Protecting Invariants Statically
//!
//! A lot is done to forbid the user from violating the invariants of the tree once constructed
//! while still allowing them to mutate parts of each element of the tree. The user can mutably traverse
//! the tree but the mutable references returns are hidden behind the `PMut<T>` type that forbids
//! mutating the aabbs.
//!
//! ### Unsafety
//!
//! Raw pointers are used for the container types in the container module
//! and for caching the results of finding colliding pairs.
//!
//! [`query::Queries::multi_rect`] uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the ['Aabb'] trait is unsafe.
//!
//! ### Name
//!
//! If you shorten "broadphase collision" to "broad colli" and say it fast, it sounds like broccoli.
//! Broccoli also have tree like properties and broccoli uses a tree data structure.
//!
//! ### nostd
//!
//! Uses `no_std`, but uses the `alloc` crate.

#![doc(html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png", html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png")]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

pub use axgeom;
pub use compt;
pub use rayon;

mod inner_prelude {
    pub(crate) use super::*;
    pub(crate) use crate::pmut::*;
    pub(crate) use crate::tree::analyze::*;
    pub use alloc::vec::Vec;
    pub use axgeom::*;
    pub(crate) use compt::Visitor;
    pub use core::marker::PhantomData;
}

pub mod query;

///Contains generic tree construction code
mod tree;
pub use tree::*;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

pub mod pmut;

///A collection of different bounding box containers.
mod bbox;
pub use crate::bbox::*;

///Generic slice utillity functions.
mod util;

///Helper functions to convert aabbs in floats to integers
pub mod convert;

//use axgeom::Rect;
pub use axgeom::Rect;
pub use axgeom::rect;

///The broccoli prelude.
pub mod prelude {
    pub use crate::query::NotSortedQueries;
    pub use crate::query::Queries;
    pub use crate::query::QueriesInner;
    pub use crate::Aabb;
    pub use crate::HasInner;
    pub use crate::Num;
}

///The underlying number type used for the tree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait Num: Ord + Copy + Send + Sync {}
impl<T> Num for T where T: Ord + Copy + Send + Sync {}

///Trait to signify that this object has an axis aligned bounding box.
///`get()` must return a aabb with the same value in it while the element
///is in the tree. This is hard for the user not to do, this the user
///does not have `&mut self`,
///but it is still possible through the use of static objects or `RefCell` / `Mutex`, etc.
///Using these type of methods the user could make different calls to get()
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
///invariants of the tree. This can be done if the user were to implement with type `Inner=Self`
///
///We have no easy way to ensure that the Inner type only points to the inner portion of a AABB
///so we mark this trait as unsafe.
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


