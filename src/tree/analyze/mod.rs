//! Contains code to help analyze the [`Tree`] structure.
//! Only used to measure the performance of the structure.

use crate::inner_prelude::*;

pub mod assert;

pub use builder::TreeBuilder;
mod builder;

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub(crate) trait Sorter: Copy + Clone + Send + Sync {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
}

#[derive(Copy, Clone)]
pub(crate) struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]) {
        oned::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone)]
pub(crate) struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [impl Aabb]) {}
}

pub fn nodes_left(depth: usize, height: usize) -> usize {
    let levels = height - 1 - depth;
    2usize.rotate_left(levels as u32) - 1
}

///Passed to the binning algorithm to determine
///if the binning algorithm should check for index out of bounds.
#[derive(Copy, Clone, Debug)]
pub enum BinStrat {
    Checked,
    NotChecked,
}

///Returns the height of a dyn tree for a given number of bots.
///The height is chosen such that the nodes will each have a small amount of bots.
///If we had a node per bot, the tree would have too many levels. Too much time would be spent recursing.
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///Theory would tell you to just make a node per bot, but there is
///a sweet spot inbetween determined by the real-word properties of your computer.
pub const DEFAULT_NUMBER_ELEM_PER_NODE: usize = 128;

///Outputs the height given an desirned number of bots per node.
#[inline]
pub fn compute_tree_height_heuristic(num_bots: usize, num_per_node: usize) -> usize {
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        let a = num_bots as f32 / num_per_node as f32;
        let b = a.log2() / 2.0;
        (b.ceil() as usize) * 2 + 1
    }
}

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> (Self,Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self,a:Self,b: Self);

    fn leaf_start(&mut self){} //TODO remove default impl

    fn leaf_end(&mut self){}

}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty {
    
    #[inline(always)]
    fn div(&mut self) -> (Self,Self) {
        (SplitterEmpty,SplitterEmpty)
    }
    #[inline(always)]
    fn add(&mut self,_:Self, _: Self) {}


}
