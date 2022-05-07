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

//#[doc(inline)]
//pub use broccoli_tree as tree;

pub use axgeom;
pub mod tree;

use tree::aabb_pin::AabbPin;
use tree::aabb_pin::AabbPinIter;
use tree::build::*;
use tree::node::*;
use tree::splitter::empty_mut;
use tree::splitter::Splitter;
use tree::*;

pub mod ext;

#[cfg(test)]
mod tests;

pub mod queries;

pub mod prelude {}

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

pub struct TreeOwned<C: Container>
where
    C::T: Aabb,
{
    container: C,
    nodes: Vec<NodeData<<C::T as Aabb>::Num>>,
    total_num_elem: usize,
}

impl<C: Container> TreeOwned<C>
where
    C::T: Aabb,
{
    pub fn new(mut container: C) -> TreeOwned<C> {
        let j = container.as_mut();
        let length = j.len();

        let t = Tree::new(j).nodes;

        let nodes = t.into_iter().map(|x| x.as_data()).collect();

        TreeOwned {
            container,
            nodes,
            total_num_elem: length,
        }
    }

    #[cfg(feature = "parallel")]
    pub fn par_new(mut container: C) -> TreeOwned<C>
    where
        C::T: Send,
        <C::T as Aabb>::Num: Send,
    {
        let j = container.as_mut();
        let length = j.len();

        let t = Tree::par_new(j).nodes;

        let nodes = t.into_iter().map(|x| x.as_data()).collect();

        TreeOwned {
            container,
            nodes,
            total_num_elem: length,
        }
    }

    pub fn as_tree(&mut self) -> Tree<C::T> {
        let bots = self.container.as_mut();
        assert_eq!(bots.len(), self.total_num_elem);

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
        assert_eq!(j.len(), self.total_num_elem);

        AabbPin::new(j)
    }

    pub fn container_ref(&self) -> &C {
        &self.container
    }

    #[must_use]
    pub fn into_inner(self) -> C {
        self.container
    }
}

pub struct Tree<'a, T: Aabb> {
    nodes: Vec<Node<'a, T>>,
}

impl<'a, T: Aabb + 'a> Tree<'a, T> {
    pub fn from_nodes(nodes: Vec<Node<'a, T>>) -> Self {
        Tree { nodes }
    }

    pub fn from_build_args<'b, P: Splitter>(args: BuildArgs<'a, 'b, T, P>) -> Self {
        Tree {
            nodes: args.build_ext(&mut DefaultSorter),
        }
    }

    pub fn par_from_build_args<'b, P: Splitter>(args: BuildArgs<'a, 'b, T, P>) -> Self
    where
        T: Send,
        T::Num: Send,
        P: Send,
    {
        Tree {
            nodes: args.par_build_ext(&mut DefaultSorter),
        }
    }

    pub fn new(bots: &'a mut [T]) -> Self {
        let nodes = BuildArgs::new(bots).build_ext(&mut DefaultSorter);

        Tree { nodes }
    }

    #[cfg(feature = "parallel")]
    pub fn par_new(bots: &'a mut [T]) -> Self
    where
        T: Send,
        T::Num: Send,
    {
        let nodes = BuildArgs::new(bots).par_build_ext(&mut DefaultSorter);

        Tree { nodes }
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
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
    pub fn get_nodes(&self) -> &[Node<'a, T>] {
        &self.nodes
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes_mut(&mut self) -> AabbPin<&mut [Node<'a, T>]> {
        AabbPin::from_mut(&mut self.nodes)
    }
}

pub struct NotSortedTree<'a, T: Aabb> {
    nodes: Vec<Node<'a, T>>,
}

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn from_nodes(nodes: Vec<Node<'a, T>>) -> Self {
        NotSortedTree { nodes }
    }

    pub fn from_build_args<'b, P: Splitter>(args: BuildArgs<'a, 'b, T, P>) -> Self {
        NotSortedTree {
            nodes: args.build_ext(&mut NoSorter),
        }
    }

    pub fn par_from_build_args<'b, P: Splitter>(args: BuildArgs<'a, 'b, T, P>) -> Self
    where
        T: Send,
        T::Num: Send,
        P: Send,
    {
        NotSortedTree {
            nodes: args.par_build_ext(&mut NoSorter),
        }
    }

    pub fn new(bots: &'a mut [T]) -> Self {
        let nodes = BuildArgs::new(bots).build_ext(&mut NoSorter);

        NotSortedTree { nodes }
    }

    #[cfg(feature = "parallel")]
    pub fn par_new(bots: &'a mut [T]) -> Self
    where
        T: Send,
        T::Num: Send,
    {
        let nodes = BuildArgs::new(bots).par_build_ext(&mut NoSorter);

        NotSortedTree { nodes }
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        let tree = compt::dfs_order::CompleteTree::from_preorder(&self.nodes).unwrap();

        tree.vistr()
    }
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

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

pub struct Assert<'a, T> {
    inner: &'a mut [T],
}
impl<'a, T: Aabb> Assert<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Assert { inner }
    }
}
