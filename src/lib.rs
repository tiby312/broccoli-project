//! Broccoli is a broad-phase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are explored more in depth in the [broccoli book](https://tiby312.github.io/broccoli_report)
//!
//! - `(Rect<N>,&mut T)` Semi-direct
//! - `(Rect<N>,T)` Direct
//! - `&mut (Rect<N>,T)` Indirect
//!
//! ### There are so many Tree types which one do I use?
//!
//! The [`container`] module lists the tree types and they are all described there, but in general
//! use [`Tree`] unless you want
//! to use functions like [`collect_colliding_pairs`](crate::container::TreeInd::collect_colliding_pairs).
//! In which case use [`TreeInd`](crate::container::TreeInd).
//!
//! Checkout the github [examples](https://github.com/tiby312/broccoli/tree/master/examples).
//!
//! ### Parallelism
//!
//! Parallel versions of construction and colliding pair finding functions
//! are provided. They use [rayon](https://crates.io/crates/rayon) under the hood which uses work stealing to
//! parallelize divide and conquer style recursive functions.
//!
//! ### Floating Point
//!
//! Broccoli only requires `PartialOrd` for its number type. Instead of panicking on comparisons
//! it doesn't understand, it will just arbitrary pick a result. So if you use regular float primitive types
//! and there is even just one `NaN`, tree construction and querying will not panic,
//! but would have unspecified results.
//! If using floats, it's the users responsibility to not pass `NaN` values into the tree.
//! There is no static protection against this, though if this is desired you can use
//! the [ordered-float](https://crates.io/crates/ordered-float) crate. The Ord trait was not
//! enforced to give users the option to use primitive floats directly which can be easier to
//! work with.
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
//! [`multi_rect`](Tree::multi_rect) uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the [`node::Aabb`] trait is unsafe.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png"
)]
#![no_std]

#[macro_use]
extern crate alloc;

pub use axgeom;
pub use compt;

use crate::node::*;
use crate::pmut::*;
use crate::tree::build::*;
use crate::tree::*;
use crate::util::*;
use alloc::vec::Vec;
use axgeom::*;
use compt::Visitor;

pub mod prelude{
    
}

mod par;


///Contains generic tree construction code
mod tree;
pub use tree::*;

pub mod pmut;

///Contains node-level building block structs and visitors used for a [`Tree`].
pub mod node;

///Generic slice utility functions.
mod util;

pub use axgeom::rect;

///Shorthand constructor of [`node::BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: axgeom::Rect<N>, inner: T) -> node::BBox<N, T> {
    node::BBox::new(rect, inner)
}

#[cfg(feature = "use_rayon")]
pub use self::rayonjoin::*;
#[cfg(feature = "use_rayon")]
mod rayonjoin {
    use super::*;
    ///
    /// An implementation of [`Joinable`] that uses rayon's `join`.
    #[derive(Copy, Clone)]
    pub struct RayonJoin;
    impl Joinable for RayonJoin {
        #[inline(always)]
        fn join<A, B, RA, RB>(&self, oper_a: A, oper_b: B) -> (RA, RB)
        where
            A: FnOnce(&Self) -> RA + Send,
            B: FnOnce(&Self) -> RB + Send,
            RA: Send,
            RB: Send,
        {
            rayon_core::join(|| oper_a(self), || oper_b(self))
        }
    }
}

///
/// Trait defining the main primitive with which the `_par` functions
/// will be parallelized. The trait is based off of rayon's `join` function.
///
pub trait Joinable: Clone + Send + Sync {
    ///Execute both closures potentially in parallel.
    fn join<A, B, RA, RB>(&self, oper_a: A, oper_b: B) -> (RA, RB)
    where
        A: FnOnce(&Self) -> RA + Send,
        B: FnOnce(&Self) -> RB + Send,
        RA: Send,
        RB: Send;

    ///Execute function F on each element in parallel
    ///using `Self::join`.
    fn for_every<T, F>(&self, arr: &mut [T], func: F)
    where
        T: Send,
        F: Fn(&mut T) + Send + Copy,
    {
        if let Some((front, rest)) = arr.split_first_mut() {
            self.join(move |_| func(front), move |_| self.for_every(rest, func));
        }
    }
}

#[repr(transparent)]
struct Ptr<T: ?Sized>(*mut T);
impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline(always)]
    fn clone(&self) -> Ptr<T> {
        *self
    }
}
unsafe impl<T: ?Sized> Send for Ptr<T> {}
unsafe impl<T: ?Sized> Sync for Ptr<T> {}
