//! Contains code to help build the [`Tree`] structure with more options than
//! just using [`broccoli::new`](crate::new).

use crate::inner_prelude::*;

///The default starting axis of a [`Tree`]. It is set to be the `Y` axis.
///This means that the first divider is a horizontal line since it is
///partitioning space based off of the aabb's `Y` value.
pub type DefaultA = YAXIS;

///Returns the default axis type.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

mod oned;

pub use builder::TreeBuilder;
mod builder;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> (Self, Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, a: Self, b: Self);
}

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
trait Sorter: Copy + Clone + Send + Sync {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
}

#[derive(Copy, Clone)]
struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]) {
        crate::util::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone)]
struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [impl Aabb]) {}
}

const fn nodes_left(depth: usize, height: usize) -> usize {
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

///The default number of elements per node
///
///If we had a node per bot, the tree would have too many levels. Too much time would be spent recursing.
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///Theory would tell you to just make a node per bot, but there is
///a sweet spot inbetween determined by the real-word properties of your computer.
///we want each node to have space for around num_per_node bots.
///there are 2^h nodes.
///2^h*200>=num_bots.  Solve for h s.t. h is an integer.
///Make this number too small, and the tree will have too many levels,
///and too much time will be spent recursing.
///Make this number too high, and you will lose the properties of a tree,
///and you will end up with just sweep and prune.
///This number was chosen emprically from running the Tree_alg_data project,
///on two different machines.
pub const DEFAULT_NUMBER_ELEM_PER_NODE: usize = 32;

use crate::par::Parallel;

///Using this struct the user can determine the height of a tree or the number of nodes
///that would exist if the tree were constructed with the specified number of elements.
#[derive(Copy, Clone)]
pub struct TreePreBuilder {
    height: usize,
    height_switch_seq: usize,
}

impl TreePreBuilder {
    ///Create the builder object with default values.
    pub const fn new(num_elements: usize) -> TreePreBuilder {
        let height = compute_tree_height_heuristic(num_elements, DEFAULT_NUMBER_ELEM_PER_NODE);
        TreePreBuilder {
            height,
            height_switch_seq: par::SWITCH_SEQUENTIAL_DEFAULT,
        }
    }
    ///Specify a custom default nuber of elements per leaf.
    pub const fn with_num_elem_in_leaf(
        num_elements: usize,
        num_elem_leaf: usize,
    ) -> TreePreBuilder {
        let height = compute_tree_height_heuristic(num_elements, num_elem_leaf);
        TreePreBuilder {
            height,
            height_switch_seq: par::SWITCH_SEQUENTIAL_DEFAULT,
        }
    }

    ///Specify at which level to switch from parallel to sequential when
    ///parallel functions are used.
    pub fn with_height_seq(&mut self, height: usize) {
        self.height_switch_seq = height;
    }

    ///Create a [`par::Joiner`] that will switch to sequential at the approriate level
    const fn switch_seq_level(&self) -> Parallel {
        crate::par::compute_level_switch_sequential(self.height_switch_seq, self.height)
    }

    ///Specify a custom height of the tree, ignoring the number of elements per node variable.
    pub const fn with_height(height: usize) -> TreePreBuilder {
        TreePreBuilder {
            height,
            height_switch_seq: par::SWITCH_SEQUENTIAL_DEFAULT,
        }
    }

    ///Create a `TreeBuilder`
    pub fn into_builder<T: Aabb>(self, bots: &mut [T]) -> TreeBuilder<T> {
        TreeBuilder::from_prebuilder( bots, self)
    }

    ///Return the level at which parallel algorithms will switch to sequential.
    pub const fn get_height_seq(&self) -> usize {
        self.height_switch_seq
    }

    ///Compute the number of nodes in the tree based off of the height.
    pub const fn num_nodes(&self) -> usize {
        nodes_left(0, self.height)
    }

    ///Get the currently configured height.
    pub const fn get_height(&self) -> usize {
        self.height
    }
}

///Outputs the height given an desirned number of bots per node.
#[inline]
const fn compute_tree_height_heuristic(num_bots: usize, num_per_node: usize) -> usize {
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        let (num_bots, num_per_node) = (num_bots as u64, num_per_node as u64);
        let a = num_bots / num_per_node;
        let a = log_2(a);
        (a + 1) as usize
    }
}

const fn log_2(x: u64) -> u64 {
    const fn num_bits<T>() -> usize {
        core::mem::size_of::<T>() * 8
    }
    num_bits::<u64>() as u64 - x.leading_zeros() as u64 - 1
}

use crate::query::colfind::NotSortedQueries;
use crate::tree::Queries;
///A version of Tree where the elements are not sorted along each axis, like a KD Tree.
/// For comparison, a normal kd-tree is provided by [`NotSorted`]. In this tree, the elements are not sorted
/// along an axis at each level. Construction of [`NotSorted`] is faster than [`Tree`] since it does not have to
/// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
/// extra property to be faster.
pub struct NotSorted<'a, T: Aabb>(Tree<'a,  T>);

impl<'a, T: Aabb + Send + Sync> NotSorted<'a, T>
where
    T::Num: Send + Sync,
{
    pub fn new_par(bots: &'a mut [T]) -> NotSorted<'a,  T> {
        TreeBuilder::new(bots).build_not_sorted_par()
    }
}
impl<'a, T: Aabb> NotSorted<'a,  T> {
    pub fn new(bots: &'a mut [T]) -> NotSorted<'a,  T> {
        TreeBuilder::new(bots).build_not_sorted_seq()
    }
}


impl<'a, T: Aabb> NotSortedQueries<'a> for NotSorted<'a, T> {
    type T = T;
    type Num = T::Num;

    #[inline(always)]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        self.0.vistr_mut()
    }

    #[inline(always)]
    fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.0.vistr()
    }
}

impl<'a,  T: Aabb> NotSorted<'a,  T> {
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.0.get_height()
    }
}
