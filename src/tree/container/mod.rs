//! Container trees that deref to [`Tree`]
//!
//! Most of the time using [`Tree`] is enough. But in certain cases
//! we want more control. 

use super::*;

mod tree_ind;
mod owned;
pub use self::tree_ind::*;
pub use self::owned::*;

use alloc::boxed::Box;
