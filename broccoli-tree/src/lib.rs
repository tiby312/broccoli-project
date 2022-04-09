//! Contains code to help build the [`Tree`] structure with more options than

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
pub const fn default_axis() -> DefaultA {
    XAXIS
}

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub trait Sorter: Copy + Clone + Send + Sync + Default {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
}

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]) {
        crate::util::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [impl Aabb]) {}
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
    pub fn default(num_elements: usize) -> usize {
        compute_tree_height_heuristic(num_elements, DEFAULT_NUMBER_ELEM_PER_NODE)
    }
    ///Specify a custom default number of elements per leaf.
    pub const fn with_num_elem_in_leaf(num_elements: usize, num_elem_leaf: usize) -> usize {
        compute_tree_height_heuristic(num_elements, num_elem_leaf)
    }
}

pub fn create_ind<T, N: Num>(
    bots: &mut [T],
    mut func: impl FnMut(&T) -> Rect<N>,
) -> Box<[BBox<N, &mut T>]> {
    bots.iter_mut()
        .map(|a| crate::bbox(func(a), a))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[must_use]
pub struct NodeFinisher<'a, T: Aabb, S> {
    is_xaxis: bool,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    sorter: S,
}
impl<'a, T: Aabb, S: Sorter> NodeFinisher<'a, T, S> {
    #[inline(always)]
    fn finish(self) -> Node<'a, T> {
        fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
            match middle.split_first() {
                Some((first, rest)) => {
                    let mut min = first.get().get_range(axis).start;
                    let mut max = first.get().get_range(axis).end;

                    for a in rest.iter() {
                        let start = &a.get().get_range(axis).start;
                        let end = &a.get().get_range(axis).end;

                        if *start < min {
                            min = *start;
                        }

                        if *end > max {
                            max = *end;
                        }
                    }

                    axgeom::Range {
                        start: min,
                        end: max,
                    }
                }
                None => axgeom::Range {
                    start: Default::default(),
                    end: Default::default(),
                },
            }
        }

        let cont = if self.is_xaxis {
            self.sorter.sort(axgeom::XAXIS.next(), self.mid);
            create_cont(axgeom::XAXIS, self.mid)
        } else {
            self.sorter.sort(axgeom::YAXIS.next(), self.mid);
            create_cont(axgeom::YAXIS, self.mid)
        };

        Node {
            range: AabbPin::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

pub use axgeom::rect;

///Shorthand constructor of [`node::BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: axgeom::Rect<N>, inner: T) -> node::BBox<N, T> {
    node::BBox::new(rect, inner)
}

///Shorthand constructor of [`node::BBoxMut`]
#[inline(always)]
#[must_use]
pub fn bbox_mut<N, T>(rect: axgeom::Rect<N>, inner: &mut T) -> node::BBoxMut<N, T> {
    node::BBoxMut::new(rect, inner)
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
/// let tree = broccoli::new(&mut bots);
///
///```
pub fn new<T: Aabb>(bots: &mut [T]) -> Tree<T> {
    TreeInner::build(DefaultSorter, bots)
}

pub fn new_par<T: Aabb>(bots: &mut [T]) -> Tree<T>
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    TreeInner::build_par(DefaultSorter, bots)
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

impl<C: Container> TreeOwned<C, DefaultSorter>
where
    C::T: Aabb,
{
    pub fn new(bots: C) -> Self {
        TreeOwned::build_owned(DefaultSorter, bots)
    }
}

impl<C: Container, S: Sorter> TreeOwned<C, S>
where
    C::T: Aabb,
{
    pub fn build_owned(sorter: S, mut bots: C) -> TreeOwned<C, S> {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeInner::build(sorter, j);
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }

    pub fn build_owned_par(sorter: S, mut bots: C) -> TreeOwned<C, S>
    where
        C::T: Send + Sync,
        <C::T as Aabb>::Num: Send + Sync,
    {
        let j = bots.as_mut();
        let length = j.len();

        let t = TreeInner::build_par(sorter, j);
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
    pub fn as_slice_mut(&mut self) -> AabbPin<&mut [C::T]> {
        let j = self.inner.as_mut();
        assert_eq!(j.len(), self.length);

        AabbPin::new(j)
    }
    pub fn into_inner(self) -> C {
        self.inner
    }
}

impl<C: Container + Clone, S> TreeOwned<C, S>
where
    C::T: Aabb,
{
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

pub type Tree<'a, T> = TreeInner<Node<'a, T>, DefaultSorter>;

impl<S, H: node::HasElem> TreeInner<H, S> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = AabbPin<&mut H::T>> {
        self.nodes.iter_mut().flat_map(|x| x.get_elems().iter_mut())
    }
}

fn as_node_tree<N>(vec: &[N]) -> compt::dfs_order::CompleteTree<N, compt::dfs_order::PreOrder> {
    compt::dfs_order::CompleteTree::from_preorder(vec).unwrap()
}

impl<S, H> TreeInner<H, S> {
    #[must_use]
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

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    /// let inner =tree.into_inner();
    /// assert_eq!(inner.into_nodes().len(),1);
    ///```
    #[must_use]
    pub fn into_nodes(self) -> Vec<H> {
        self.nodes
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.num_nodes(),build::TreePreBuilder::new(1).num_nodes());
    ///
    ///```
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn total_num_elem(&self) -> usize {
        self.total_num_elem
    }

    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes()[0].range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
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
impl<'a, T: Aabb + 'a, S: Sorter> TreeInner<Node<'a, T>, S> {
    #[must_use]
    pub fn build(tb: impl TreeBuild<T, S>, bots: &'a mut [T]) -> TreeInner<Node<'a, T>, S> {
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
    #[must_use]
    pub fn build_par(tb: impl TreeBuild<T, S>, bots: &'a mut [T]) -> TreeInner<Node<'a, T>, S>
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
