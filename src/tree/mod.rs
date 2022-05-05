//!
//! This is a supporting crate to the  `broccoli` crate. This crate only
//! provides the broccoli tree and tree building code, but no querying code.
//!

use self::splitter::{empty_mut, EmptySplitter, Splitter};

use super::*;
pub mod aabb_pin;
mod assert;
pub mod build;
pub mod node;
mod oned;
pub mod splitter;
pub mod util;

use axgeom;

use aabb_pin::*;

use axgeom::*;
use build::*;
use compt::Visitor;
use node::*;

///The default starting axis of a [`Tree`]. It is set to be the `X` axis.
///This means that the first divider is a 'vertical' line since it is
///partitioning space based off of the aabb's `X` value.
pub type DefaultA = XAXIS;

///Returns the default axis type.
#[must_use]
pub const fn default_axis() -> DefaultA {
    XAXIS
}

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub trait Sorter<T>: splitter::Splitter {
    fn sort(&self, axis: impl Axis, bots: &mut [T]);
}

///Using this struct the user can determine the height of a tree or the number of nodes
///that would exist if the tree were constructed with the specified number of elements.
pub mod num_level {
    pub const fn num_nodes(num_levels: usize) -> usize {
        2usize.rotate_left(num_levels as u32) - 1
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
    ///This number was chosen empirically from running the Tree_alg_data project,
    ///on two different machines.
    pub const DEFAULT_NUMBER_ELEM_PER_NODE: usize = 32;

    ///Outputs the height given an desirned number of bots per node.
    #[inline]
    #[must_use]
    fn compute_tree_height_heuristic(num_bots: usize, num_per_node: usize) -> usize {
        //we want each node to have space for around 300 bots.
        //there are 2^h nodes.
        //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

        if num_bots <= num_per_node {
            1
        } else {
            let (num_bots, num_per_node) = (num_bots as u64, num_per_node as u64);
            let a = num_bots / num_per_node;
            let a = log_2(a);
            let k = (((a / 2) * 2) + 1) as usize;
            assert_eq!(k % 2, 1, "k={:?}", k);
            k
        }
    }
    #[must_use]
    const fn log_2(x: u64) -> u64 {
        const fn num_bits<T>() -> usize {
            core::mem::size_of::<T>() * 8
        }
        num_bits::<u64>() as u64 - x.leading_zeros() as u64 - 1
    }
    #[must_use]
    pub fn default(num_elements: usize) -> usize {
        compute_tree_height_heuristic(num_elements, DEFAULT_NUMBER_ELEM_PER_NODE)
    }
    ///Specify a custom default number of elements per leaf
    #[must_use]
    pub fn with_num_elem_in_leaf(num_elements: usize, num_elem_leaf: usize) -> usize {
        compute_tree_height_heuristic(num_elements, num_elem_leaf)
    }
}

pub use axgeom::rect;

///Shorthand constructor of [`BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
    BBox::new(rect, inner)
}

///Shorthand constructor of [`BBoxMut`]
#[inline(always)]
#[must_use]
pub fn bbox_mut<N, T>(rect: axgeom::Rect<N>, inner: &mut T) -> BBoxMut<N, T> {
    BBoxMut::new(rect, inner)
}

pub struct TreeBuilder<'a, T, P> {
    pub bots: &'a mut [T],
    pub num_level: usize,
    pub num_seq_fallback: usize,
    pub splitter: P,
}

impl<'a, T: Aabb> TreeBuilder<'a, T, &'static mut EmptySplitter> {
    pub fn new(bots: &'a mut [T]) -> Self {
        let num_bots = bots.len();
        let num_seq_fallback = 2_400;
        let splitter = empty_mut();
        TreeBuilder {
            bots,
            num_level: num_level::default(num_bots),
            num_seq_fallback,
            splitter,
        }
    }
}

impl<'a, 'b, T: Aabb, P: Splitter + 'b> TreeBuilder<'a, T, &'b mut P> {
    pub fn with_splitter(bots: &'a mut [T], splitter: &'b mut P) -> Self {
        let num_bots = bots.len();
        let num_seq_fallback = 2_400;
        TreeBuilder {
            bots,
            num_level: num_level::default(num_bots),
            num_seq_fallback,
            splitter,
        }
    }

    pub fn build<S: Sorter<T>>(self, sorter: &mut S) -> Vec<Node<'a, T>> {
        let mut buffer = Vec::with_capacity(num_level::num_nodes(self.num_level));
        Self::recurse_seq(
            self.splitter,
            sorter,
            &mut buffer,
            TreeBuildVisitor::new(self.num_level, self.bots),
        );
        buffer
    }

    #[cfg(feature = "parallel")]
    pub fn build_par<S: Sorter<T>>(self, sorter: &mut S) -> Vec<Node<'a, T>>
    where
        T: Send,
        T::Num: Send,
        S: Send,
        P: Send,
    {
        let mut buffer = Vec::with_capacity(num_level::num_nodes(self.num_level));
        Self::recurse_par(
            self.num_seq_fallback,
            self.splitter,
            sorter,
            &mut buffer,
            TreeBuildVisitor::new(self.num_level, self.bots),
        );
        buffer
    }
    fn recurse_seq<S: Sorter<T>>(
        splitter: &mut P,
        sorter: &mut S,
        buffer: &mut Vec<Node<'a, T>>,
        vis: TreeBuildVisitor<'a, T>,
    ) {
        let NodeBuildResult { node, rest } = vis.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            let mut a = splitter.div();
            Self::recurse_seq(splitter, sorter, buffer, left);
            Self::recurse_seq(&mut a, sorter, buffer, right);
            splitter.add(a);
        }
    }

    #[cfg(feature = "parallel")]
    fn recurse_par<S: Sorter<T>>(
        num_seq_fallback: usize,
        splitter: &mut P,
        sorter: &mut S,
        buffer: &mut Vec<Node<'a, T>>,
        vistr: TreeBuildVisitor<'a, T>,
    ) where
        S: Send,
        T: Send,
        T::Num: Send,
        P: Send,
    {
        let NodeBuildResult { node, rest } = vistr.build_and_next();

        if let Some([left, right]) = rest {
            let mut p = splitter.div();
            if node.get_num_elem() <= num_seq_fallback {
                buffer.push(node.finish(sorter));
                Self::recurse_seq(splitter, sorter, buffer, left);
                Self::recurse_seq(&mut p, sorter, buffer, right);
            } else {
                let mut s2 = sorter.div();
                let mut buffer2 = Vec::with_capacity(num_level::num_nodes(right.get_height()));

                rayon::join(
                    || {
                        buffer.push(node.finish(sorter));
                        Self::recurse_par(num_seq_fallback, splitter, sorter, buffer, left);
                    },
                    || {
                        Self::recurse_par(num_seq_fallback, &mut p, &mut s2, &mut buffer2, right);
                    },
                );
                buffer.append(&mut buffer2);
                sorter.add(s2)
            }
            splitter.add(p);
        } else {
            buffer.push(node.finish(sorter));
        }
    }
}
