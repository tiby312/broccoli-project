//! Broccoli is a broadphase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are exlored more in depth in the [broccoli book](https://tiby312.github.io/broccoli_report)
//!
//!
//! - `(Rect<N>,&mut T)` Semi-direct
//! - `(Rect<N>,T)` Direct
//! - `&mut (Rect<N>,T)` Indirect
//!
//! ### There are so many Tree types which one do I use?
//!
//! The [`container`] module lists the tree types and they are all described there, but the
//! TL;DR is use [`Tree`] and fill it with `BBox<N,&mut T>` unless you want
//! to use functions like [`collect_colliding_pairs`](crate::container::TreeRefInd::collect_colliding_pairs).
//! In which case use [`TreeRefInd`](crate::container::TreeRefInd).
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
//! it doesnt understand, it will just arbitrary pick a result. So if you use regular float primitive types
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
//! [`multi_rect`](query::rect::RectQuery::multi_rect) uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the [`node::Aabb`] trait is unsafe.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png"
)]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

pub use axgeom;
pub use compt;

mod inner_prelude {
    pub(crate) use crate::par;
    pub(crate) use crate::prelude::*;
    pub(crate) use crate::query::Queries;

    pub(crate) use crate::tree::*;
    pub(crate) use crate::util::*;

    pub(crate) use crate::node::*;
    pub(crate) use crate::pmut::*;

    pub(crate) use crate::tree::build::*;
    pub use alloc::vec::Vec;
    pub use axgeom::*;
    pub(crate) use compt::Visitor;
    pub use core::marker::PhantomData;
}

mod par;

pub mod query;

///Contains generic tree construction code
mod tree;
pub use tree::*;

pub mod pmut;

///Contains node-level building block structs and visitors used for a [`Tree`].
pub mod node;

///Generic slice utillity functions.
mod util;

///Helper functions to convert aabbs in floats to integers
pub mod convert;

///The broccoli prelude.
pub mod prelude {
    pub use crate::query::draw::DrawQuery;
    pub use crate::query::nbody::NbodyQuery;

    pub use crate::query::colfind::ColfindQuery;
    pub use crate::query::intersect_with::IntersectQuery;
    pub use crate::query::knearest::KnearestQuery;
    pub use crate::query::raycast::RaycastQuery;
    pub use crate::query::rect::RectQuery;
    //pub use crate::query::Queries;
}

pub use axgeom::rect;

///Shorthand constructor of [`node::BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: axgeom::Rect<N>, inner: T) -> node::BBox<N, T> {
    node::BBox::new(rect, inner)
}
