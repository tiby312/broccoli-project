//!
//! This is a supporting crate to the  `broccoli` crate. This crate only
//! provides the broccoli tree and tree building code, but no querying code.
//!

pub mod aabb_pin;
mod assert;
pub mod build;
pub mod node;
mod oned;
pub mod splitter;
pub mod util;

pub use axgeom;

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
pub trait Sorter<T: Aabb>: Copy + Clone + Send + Sync {
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

///Create a [`Tree`].
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli_tree::new(&mut bots);
///
///```
pub fn new<T: Aabb>(bots: &mut [T]) -> Tree<T> {
    TreeBuilder::new(DefaultSorter, bots).build()
}

#[cfg(feature = "parallel")]
pub fn new_par<T: Aabb>(bots: &mut [T]) -> Tree<T>
where
    T: Send,
    T::Num: Send,
{
    TreeBuilder::new(DefaultSorter, bots).build_par()
}

///
/// Create a [`TreeOwned`]
///
pub fn new_owned<C: Container>(cont: C) -> TreeOwned<C, DefaultSorter>
where
    C::T: Aabb,
{
    TreeOwned::new(DefaultSorter, cont)
}

#[cfg(feature = "parallel")]
pub fn new_owned_par<C: Container>(cont: C) -> TreeOwned<C, DefaultSorter>
where
    C::T: Aabb + Send,
    <C::T as Aabb>::Num: Send,
{
    TreeOwned::new_par(DefaultSorter, cont)
}

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
/// Owned version of [`Tree`]
///
#[must_use]
pub struct TreeOwned<C: Container, S>
where
    C::T: Aabb,
{
    inner: C,
    nodes: TreeInner<NodeData<<C::T as Aabb>::Num>, S>,
    length: usize,
}

impl<C: Container, S: Sorter<C::T>> TreeOwned<C, S>
where
    C::T: Aabb,
{
    pub fn new(sorter: S, mut bots: C) -> TreeOwned<C, S> {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeBuilder::new(sorter, j).build();
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }

    #[cfg(feature = "parallel")]
    pub fn new_par(sorter: S, mut bots: C) -> TreeOwned<C, S>
    where
        C::T: Send,
        <C::T as Aabb>::Num: Send,
    {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeBuilder::new(sorter, j).build_par();
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }

    pub fn as_tree(&mut self) -> TreeInner<Node<C::T>, S> {
        let j = self.inner.as_mut();
        assert_eq!(j.len(), self.length);

        self.nodes.clone().into_tree(AabbPin::from_mut(j))
    }
    #[must_use]
    pub fn as_slice_mut(&mut self) -> AabbPin<&mut [C::T]> {
        let j = self.inner.as_mut();
        assert_eq!(j.len(), self.length);

        AabbPin::new(j)
    }
    #[must_use]
    pub fn into_inner(self) -> C {
        self.inner
    }
}

impl<C: Container + Clone, S> TreeOwned<C, S>
where
    C::T: Aabb,
{
    #[must_use]
    pub fn clone_inner(&self) -> C {
        self.inner.clone()
    }
}

pub struct TreeBuilder<'a, T, S> {
    pub bots: &'a mut [T],
    pub sorter: S,
    pub num_level: usize,
    pub num_seq_fallback: usize,
}

impl<'a, T: Aabb> TreeBuilder<'a, T, NoSorter> {
    pub fn new_no_sort(bots: &'a mut [T]) -> Self {
        Self::new(NoSorter, bots)
    }
}
impl<'a, T: Aabb> TreeBuilder<'a, T, DefaultSorter> {
    pub fn new_default(bots: &'a mut [T]) -> Self {
        Self::new(DefaultSorter, bots)
    }
}
impl<'a, T: Aabb, S: Sorter<T>> TreeBuilder<'a, T, S> {
    pub fn new(sorter: S, bots: &'a mut [T]) -> Self {
        let num_bots = bots.len();
        let num_seq_fallback = 2_400;
        TreeBuilder {
            bots,
            sorter,
            num_level: num_level::default(num_bots),
            num_seq_fallback,
        }
    }
    pub fn build(self) -> TreeInner<Node<'a, T>, S> {
        let TreeBuilder {
            bots,
            sorter,
            num_level,
            ..
        } = self;
        let total_num_elem = bots.len();
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, sorter);
        vistr.recurse_seq(&mut buffer);
        TreeInner {
            nodes: buffer,
            sorter,
            total_num_elem,
        }
    }

    #[cfg(feature = "parallel")]
    pub fn build_par(self) -> TreeInner<Node<'a, T>, S>
    where
        T: Send,
        T::Num: Send,
    {
        pub fn recurse_par<'a, T: Aabb, S: Sorter<T>>(
            vistr: TreeBuildVisitor<'a, T, S>,
            num_seq_fallback: usize,
            buffer: &mut Vec<Node<'a, T>>,
        ) where
            T: Send,
            T::Num: Send,
        {
            let NodeBuildResult { node, rest } = vistr.build_and_next();

            if let Some([left, right]) = rest {
                if node.get_num_elem() <= num_seq_fallback {
                    buffer.push(node.finish());
                    left.recurse_seq(buffer);
                    right.recurse_seq(buffer);
                } else {
                    let (_, mut a) = rayon::join(
                        || {
                            buffer.push(node.finish());
                            recurse_par(left, num_seq_fallback, buffer);
                        },
                        || {
                            let mut f = vec![];
                            recurse_par(right, num_seq_fallback, &mut f);
                            f
                        },
                    );

                    buffer.append(&mut a);
                }
            } else {
                buffer.push(node.finish());
            }
        }

        let TreeBuilder {
            bots,
            sorter,
            num_level,
            num_seq_fallback,
        } = self;
        let total_num_elem = bots.len();
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, sorter);
        recurse_par(vistr, num_seq_fallback, &mut buffer);

        TreeInner {
            nodes: buffer,
            sorter,
            total_num_elem,
        }
    }
}

///
/// The main tree struct
///
#[derive(Clone)]
#[must_use]
pub struct TreeInner<N, S> {
    total_num_elem: usize,
    ///Stored in pre-order
    nodes: Vec<N>,
    sorter: S,
}

///
/// [`TreeInner`] type with default node and sorter.
///
pub type Tree<'a, T> = TreeInner<Node<'a, T>, DefaultSorter>;

impl<N, Y> TreeInner<N, Y> {
    pub fn into_sorter<X>(self) -> TreeInner<N, X>
    where
        X: From<Y>,
    {
        TreeInner {
            total_num_elem: self.total_num_elem,
            nodes: self.nodes,
            sorter: self.sorter.into(),
        }
    }
}
impl<'a, T: Aabb + 'a, S: Sorter<T>> TreeInner<Node<'a, T>, S> {
    pub fn into_node_data_tree(self) -> TreeInner<NodeData<T::Num>, S> {
        self.node_map(|x| NodeData {
            range: x.range.len(),
            cont: x.cont,
            div: x.div,
            num_elem: x.num_elem,
        })
    }
}

impl<S, H: HasElem> TreeInner<H, S> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = AabbPin<&mut H::T>> {
        self.nodes.iter_mut().flat_map(|x| x.get_elems().iter_mut())
    }
}

#[must_use]
fn as_node_tree<N>(vec: &[N]) -> compt::dfs_order::CompleteTree<N, compt::dfs_order::PreOrder> {
    compt::dfs_order::CompleteTree::from_preorder(vec).unwrap()
}

impl<S, H> TreeInner<H, S> {
    #[inline(always)]
    pub fn node_map<K>(self, func: impl FnMut(H) -> K) -> TreeInner<K, S> {
        let sorter = self.sorter;
        let nodes = self.nodes.into_iter().map(func).collect();
        TreeInner {
            nodes,
            sorter,
            total_num_elem: self.total_num_elem,
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn num_levels(&self) -> usize {
        as_node_tree(&self.nodes).get_height()
    }

    #[must_use]
    #[inline(always)]
    pub fn into_nodes(self) -> Vec<H> {
        self.nodes
    }

    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    #[inline(always)]
    pub fn total_num_elem(&self) -> usize {
        self.total_num_elem
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes(&self) -> &[H] {
        &self.nodes
    }

    #[must_use]
    #[inline(always)]
    pub fn get_nodes_mut(&mut self) -> AabbPin<&mut [H]> {
        AabbPin::from_mut(&mut self.nodes)
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<H> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr_mut_raw(&mut self) -> compt::dfs_order::VistrMut<H, compt::dfs_order::PreOrder> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        tree.vistr_mut()
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<H> {
        let tree = as_node_tree(&self.nodes);

        tree.vistr()
    }

    #[must_use]
    #[inline(always)]
    pub fn sorter(&self) -> S
    where
        S: Copy,
    {
        self.sorter
    }
}

impl<N: Num, S> TreeInner<NodeData<N>, S> {
    pub fn into_tree<T: Aabb<Num = N>>(self, bots: AabbPin<&mut [T]>) -> TreeInner<Node<T>, S> {
        assert_eq!(bots.len(), self.total_num_elem);
        let mut last = Some(bots);
        let n = self.node_map(|x| {
            let (range, rest) = last.take().unwrap().split_at_mut(x.range);
            last = Some(rest);
            Node {
                range,
                cont: x.cont,
                div: x.div,
                num_elem: x.num_elem,
            }
        });
        assert!(last.unwrap().is_empty());
        n
    }
}
