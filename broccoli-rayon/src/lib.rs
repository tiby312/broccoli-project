#![forbid(unsafe_code)]

pub mod build;
pub mod query;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

pub struct EmptySplitter;

impl Splitter for EmptySplitter {
    fn div(&mut self) -> Self {
        EmptySplitter
    }
    fn add(&mut self, _: Self) {}
}
