//! Broccoli is a broad-phase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! Checkout the github [examples](https://github.com/tiby312/broccoli/tree/master/examples).
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are explored more in depth in the [broccoli book](https://tiby312.github.io/broccoli_report)
//! In almost all cases you want to use the Semi-direct layout.
//!
//! - `(Rect<N>,&mut T)` Semi-direct
//! - `(Rect<N>,T)` Direct
//! - `&mut (Rect<N>,T)` Indirect
//!
//! ### Size of T
//!
//! During construction, the elements of a tree are swapped around a lot. Therefore if the size
//! of T is too big, the performance can regress a lot! To combat this, consider using the semi-direct
//! or even indirect layouts. The Indirect layout achieves the smallest element size (just one pointer),
//! however it can suffer from a lot of cache misses of large propblem sizes. The Semi-direct layout
//! is more cache-friendly.
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
//! the tree but the mutable references returns are hidden behind the `AabbPin<T>` type that forbids
//! mutating the aabbs.
//!
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png"
)]

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

use tree::aabb_pin::AabbPin;
use tree::aabb_pin::AabbPinIter;
use tree::build::*;
use tree::node::*;
use tree::splitter::Splitter;
use tree::*;

pub mod ext;

#[cfg(test)]
mod tests;

pub mod queries;

///
/// Abstract over containers that produce `&mut [T]`
///
pub trait Container {
    type T;
    fn as_mut(&mut self) -> &mut [Self::T];
}

impl<T, const N: usize> Container for [T; N] {
    type T = T;
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}
impl<T> Container for Vec<T> {
    type T = T;
    fn as_mut(&mut self) -> &mut [Self::T] {
        self
    }
}
impl<T> Container for Box<[T]> {
    type T = T;
    fn as_mut(&mut self) -> &mut [Self::T] {
        self
    }
}

///
/// An owned version of [`Tree`]
///
pub struct TreeOwned<C: Container>
where
    C::T: Aabb,
{
    container: C,
    nodes: Vec<NodeData<<C::T as Aabb>::Num>>,
}

impl<C: Container> TreeOwned<C>
where
    C::T: Aabb,
{
    pub fn new(mut container: C) -> TreeOwned<C>
    where
        C::T: ManySwap,
    {
        let bots = container.as_mut();

        let ttt = Tree::new(bots);

        let nodes = ttt.nodes.into_iter().map(|x| x.as_data()).collect();

        TreeOwned { container, nodes }
    }

    pub fn as_tree(&mut self) -> Tree<C::T> {
        let bots = self.container.as_mut();

        let mut last = Some(bots);
        let nodes = self
            .nodes
            .iter()
            .map(|x| {
                let (range, rest) = last.take().unwrap().split_at_mut(x.range);
                last = Some(rest);
                Node {
                    range: AabbPin::from_mut(range),
                    cont: x.cont,
                    div: x.div,
                    num_elem: x.num_elem,
                }
            })
            .collect();
        assert!(last.unwrap().is_empty());
        Tree { nodes }
    }

    #[must_use]
    pub fn as_slice_mut(&mut self) -> AabbPin<&mut [C::T]> {
        let j = self.container.as_mut();

        AabbPin::new(j)
    }

    pub fn as_container(&self) -> &C {
        &self.container
    }

    pub fn into_container(self) -> C {
        self.container
    }
}

///
/// A broccoli Tree.
///
pub struct Tree<'a, T: Aabb> {
    nodes: Vec<Node<'a, T>>,
}

impl<'a, T: Aabb + 'a> Tree<'a, T> {
    pub fn new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
    {
        let (nodes, _) = BuildArgs::new(bots.len()).build_ext(bots, &mut DefaultSorter);

        Tree { nodes }
    }

    #[cfg(feature = "parallel")]
    pub fn par_new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
        T: Send,
        T::Num: Send,
    {
        let (nodes, _) = BuildArgs::new(bots.len()).par_build_ext(bots, &mut DefaultSorter);

        Tree { nodes }
    }

    pub fn from_build_args<P: Splitter>(bots: &'a mut [T], args: BuildArgs<P>) -> (Self, P)
    where
        T: ManySwap,
    {
        let (nodes, s) = args.build_ext(bots, &mut DefaultSorter);
        (Tree { nodes }, s)
    }

    #[cfg(feature = "parallel")]
    pub fn par_from_build_args<P: Splitter>(bots: &'a mut [T], args: BuildArgs<P>) -> (Self, P)
    where
        T: ManySwap,
        T: Send,
        T::Num: Send,
        P: Send,
    {
        let (nodes, s) = args.par_build_ext(bots, &mut DefaultSorter);
        (Tree { nodes }, s)
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTreeMut::from_inorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTree::from_inorder(&self.nodes).unwrap();

        tree.vistr()
    }

    #[must_use]
    #[inline(always)]
    pub fn num_levels(&self) -> usize {
        compt::dfs_order::CompleteTree::from_inorder(&self.nodes)
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
    pub fn get_nodes(&self) -> &[Node<'a, T>] {
        &self.nodes
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes_mut(&mut self) -> AabbPin<&mut [Node<'a, T>]> {
        AabbPin::from_mut(&mut self.nodes)
    }
}

///
/// A tree where the elements in a node are not sorted.
///
pub struct NotSortedTree<'a, T: Aabb> {
    nodes: Vec<Node<'a, T>>,
}

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn from_build_args<P: Splitter>(bots: &'a mut [T], args: BuildArgs<P>) -> (Self, P)
    where
        T: ManySwap,
    {
        let (nodes, s) = args.build_ext(bots, &mut NoSorter);
        (NotSortedTree { nodes }, s)
    }

    #[cfg(feature = "parallel")]
    pub fn par_from_build_args<P: Splitter>(bots: &'a mut [T], args: BuildArgs<P>) -> (Self, P)
    where
        T: ManySwap,
        T: Send,
        T::Num: Send,
        P: Send,
    {
        let (nodes, s) = args.par_build_ext(bots, &mut NoSorter);
        (NotSortedTree { nodes }, s)
    }

    pub fn new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
    {
        let (nodes, _) = BuildArgs::new(bots.len()).build_ext(bots, &mut NoSorter);

        NotSortedTree { nodes }
    }

    #[cfg(feature = "parallel")]
    pub fn par_new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
        T: Send,
        T::Num: Send,
    {
        let (nodes, _) = BuildArgs::new(bots.len()).par_build_ext(bots, &mut NoSorter);

        NotSortedTree { nodes }
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTreeMut::from_inorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTree::from_inorder(&self.nodes).unwrap();

        tree.vistr()
    }
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

///
/// Easily verifiable naive query algorithms.
///
pub struct Naive<'a, T> {
    inner: AabbPin<&'a mut [T]>,
}
impl<'a, T: Aabb> Naive<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Naive {
            inner: AabbPin::from_mut(inner),
        }
    }
    pub fn from_pinned(inner: AabbPin<&'a mut [T]>) -> Self {
        Naive { inner }
    }

    pub fn iter_mut(&mut self) -> AabbPinIter<T> {
        self.inner.borrow_mut().iter_mut()
    }
}

///
/// Sweep and prune collision finding algorithm
///
pub struct SweepAndPrune<'a, T> {
    inner: &'a mut [T],
}

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        let axis = default_axis();
        tree::util::sweeper_update(axis, inner);

        SweepAndPrune { inner }
    }
}

///
/// Compare query results between [`Tree`] and
/// the easily verifiable [`Naive`] versions.
///
pub struct Assert<'a, T> {
    inner: &'a mut [T],
}
impl<'a, T: Aabb> Assert<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Assert { inner }
    }
}
