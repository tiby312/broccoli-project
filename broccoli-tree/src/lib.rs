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
pub trait Sorter: Copy + Clone + Send + Sync + Default {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
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
    pub const fn with_num_elem_in_leaf(num_elements: usize, num_elem_leaf: usize) -> usize {
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

///
/// Specify options for constructing the tree.
///
pub trait TreeBuild<T: Aabb, S: Sorter>: Sized {
    fn sorter(&self) -> S;

    fn num_level(&self, num_bots: usize) -> usize {
        num_level::default(num_bots)
    }
    fn height_seq_fallback(&self) -> usize {
        5
    }
}

impl<T: Aabb, S: Sorter> TreeBuild<T, S> for S {
    fn sorter(&self) -> S {
        *self
    }
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
#[must_use]
pub fn new<T: Aabb>(bots: &mut [T]) -> Tree<T> {
    TreeInner::new(DefaultSorter, bots)
}

#[must_use]
pub fn new_par<T: Aabb>(bots: &mut [T]) -> Tree<T>
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    TreeInner::new_par(DefaultSorter, bots)
}

///
/// Create a [`TreeOwned`]
///
#[must_use]
pub fn new_owned<C: Container>(cont: C) -> TreeOwned<C, DefaultSorter>
where
    C::T: Aabb,
{
    TreeOwned::new(DefaultSorter, cont)
}

#[must_use]
pub fn new_owned_par<C: Container>(cont: C) -> TreeOwned<C, DefaultSorter>
where
    C::T: Aabb + Send + Sync,
    <C::T as Aabb>::Num: Send + Sync,
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
pub struct TreeOwned<C: Container, S>
where
    C::T: Aabb,
{
    inner: C,
    nodes: TreeInner<NodeData<<C::T as Aabb>::Num>, S>,
    length: usize,
}

impl<C: Container, S: Sorter> TreeOwned<C, S>
where
    C::T: Aabb,
{
    #[must_use]
    pub fn new(sorter: S, mut bots: C) -> TreeOwned<C, S> {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeInner::new(sorter, j);
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }

    #[cfg(feature = "rayon")]
    #[must_use]
    pub fn new_par(sorter: S, mut bots: C) -> TreeOwned<C, S>
    where
        C::T: Send + Sync,
        <C::T as Aabb>::Num: Send + Sync,
    {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeInner::new_par(sorter, j);
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }

    #[must_use]
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

///
/// The main tree struct
///
#[derive(Clone)]
pub struct TreeInner<N, S> {
    total_num_elem: usize,
    nodes: Vec<N>,
    sorter: S,
}

///
/// [`TreeInner`] type with default node and sorter.
///
pub type Tree<'a, T> = TreeInner<Node<'a, T>, DefaultSorter>;

impl<'a, T: Aabb + 'a, S: Sorter> TreeInner<Node<'a, T>, S> {
    #[must_use]
    pub fn new(tb: impl TreeBuild<T, S>, bots: &'a mut [T]) -> TreeInner<Node<'a, T>, S> {
        let total_num_elem = bots.len();
        let num_level = tb.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, tb.sorter());
        vistr.recurse_seq(&mut buffer);
        TreeInner {
            nodes: buffer,
            sorter: tb.sorter(),
            total_num_elem,
        }
    }

    #[cfg(feature = "rayon")]
    #[must_use]
    pub fn new_par(tb: impl TreeBuild<T, S>, bots: &'a mut [T]) -> TreeInner<Node<'a, T>, S>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        pub fn recurse_par<'a, T: Aabb, S: Sorter>(
            vistr: TreeBuildVisitor<'a, T, S>,
            height_seq_fallback: usize,
            buffer: &mut Vec<Node<'a, T>>,
        ) where
            T: Send,
            T::Num: Send,
        {
            if vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(buffer);
            } else {
                let NodeBuildResult { node, rest } = vistr.build_and_next();

                if let Some([left, right]) = rest {
                    let (_, mut a) = rayon::join(
                        || {
                            buffer.push(node.finish());
                            recurse_par(left, height_seq_fallback, buffer);
                        },
                        || {
                            let mut f = vec![];
                            recurse_par(right, height_seq_fallback, &mut f);
                            f
                        },
                    );

                    buffer.append(&mut a);
                }
            }
        }

        let total_num_elem = bots.len();
        let num_level = tb.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, tb.sorter());
        recurse_par(vistr, tb.height_seq_fallback(), &mut buffer);

        TreeInner {
            nodes: buffer,
            sorter: tb.sorter(),
            total_num_elem,
        }
    }

    #[must_use]
    pub fn into_node_data_tree(self) -> TreeInner<NodeData<T::Num>, S> {
        self.node_map(|x| NodeData {
            range: x.range.len(),
            cont: x.cont,
            div: x.div,
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
    #[must_use]
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

    #[must_use]
    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMutPin<H> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        VistrMutPin::new(tree.vistr_mut())
    }

    #[must_use]
    #[inline(always)]
    pub fn vistr_mut_raw(&mut self) -> compt::dfs_order::VistrMut<H, compt::dfs_order::PreOrder> {
        let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut self.nodes).unwrap();
        tree.vistr_mut()
    }

    #[must_use]
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

impl<N: Num, S: Sorter> TreeInner<NodeData<N>, S> {
    #[must_use]
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
            }
        });
        assert!(last.unwrap().is_empty());
        n
    }
}
