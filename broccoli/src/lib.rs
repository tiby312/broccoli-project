//! Broccoli is a broad-phase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! Checkout the github [examples](https://github.com/tiby312/broccoli-project/tree/master/broccoli/examples).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli-project/master/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli-project/master/assets/logo.png"
)]
#![forbid(unsafe_code)]

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}

#[macro_use]
extern crate alloc;

pub use axgeom;
pub mod tree;

pub mod aabb;

use aabb::pin::AabbPin;
use aabb::pin::AabbPinIter;
use aabb::pin::*;
use aabb::*;

use tree::build::*;
use tree::node::*;

use tree::*;

#[cfg(test)]
mod tests;

pub mod assert;
pub mod queries;

use assert::Assert;
use assert::Naive;

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

///
/// Used to de-couple tree information from
/// the underlying lifetimed slice of elements
/// to be combined again later on.
///
/// See [`Tree::get_tree_data()`] and [`Tree::from_tree_data()`]
///
pub struct TreeData<N: Num> {
    nodes: Vec<NodeData<N>>,
}

///
/// A broccoli Tree.
///
pub struct Tree<'a, T: Aabb> {
    nodes: Box<[Node<'a, T, T::Num>]>,
}

impl<'a, T: Aabb + 'a> Tree<'a, T> {
    pub fn from_nodes(nodes: Vec<Node<'a, T, T::Num>>) -> Self {
        Tree {
            nodes: nodes.into_boxed_slice(),
        }
    }

    pub fn into_nodes(self) -> Vec<Node<'a, T, T::Num>> {
        self.nodes.into_vec()
    }

    ///
    /// Store tree data such as the number of
    /// elements per node, as well as the bounding
    /// range for each node.
    ///
    pub fn get_tree_data(&self) -> TreeData<T::Num> {
        let nodes = self.nodes.iter().map(|x| x.as_data()).collect();
        TreeData { nodes }
    }

    ///
    /// Create a Tree using stored treedata and the original
    /// list of elements in the same order.
    ///
    /// Use this function if you want to store a constructed tree
    /// outside of lifetimes.
    ///
    /// It is the user responsibility to feed this function the same
    /// distribution of aabbs in the same order as the distribution that
    /// was used in the original tree from which [`Tree::get_tree_data()`] was called.
    /// Not doing so will make an invalid tree with no error notification.
    ///
    /// Consider calling [`assert::assert_tree_invariants()`] after tree construction
    /// if you don't know if it was the same distribution which will atleast tell
    /// you if the distribution makes a valid tree.
    ///
    pub fn from_tree_data(bots: &'a mut [T], data: &TreeData<T::Num>) -> Self {
        let mut last = Some(bots);

        let nodes = data
            .nodes
            .iter()
            .map(|x| {
                let (range, rest) = last.take().unwrap().split_at_mut(x.range);
                last = Some(rest);
                Node {
                    range: AabbPin::from_mut(range),
                    cont: x.cont,
                    div: x.div,
                    min_elem: x.min_elem,
                    num_elem: x.num_elem,
                }
            })
            .collect();
        assert!(last.unwrap().is_empty());
        Tree { nodes }
    }

    pub fn new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
    {
        let num_level = num_level::default(bots.len());

        let num_nodes = num_level::num_nodes(num_level);
        let mut nodes = Vec::with_capacity(num_nodes);

        TreeBuildVisitor::new(num_level, bots).recurse_seq(&mut DefaultSorter, &mut nodes);

        assert_eq!(num_nodes, nodes.len());

        let t = Tree::from_nodes(nodes);
        assert_eq!(t.num_levels(), num_level, "num_nodes:{}", num_nodes);
        t
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<Node<'a, T, T::Num>> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T, T::Num>> {
        let tree = compt::dfs_order::CompleteTree::from_preorder(&self.nodes).unwrap();

        tree.vistr()
    }

    #[must_use]
    #[inline(always)]
    pub fn num_levels(&self) -> usize {
        compt::dfs_order::CompleteTree::from_preorder(&self.nodes)
            .unwrap()
            .get_height()
    }

    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes(&self) -> &[Node<'a, T, T::Num>] {
        &self.nodes
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes_mut(&mut self) -> AabbPin<&mut [Node<'a, T, T::Num>]> {
        AabbPin::from_mut(&mut self.nodes)
    }
}
