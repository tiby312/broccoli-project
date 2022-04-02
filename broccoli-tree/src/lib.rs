//! Contains code to help build the [`Tree`] structure with more options than
//! just using [`broccoli::new`](crate::new).

pub mod assert;
pub mod halfpin;
pub mod node;
mod oned;
pub mod par;
pub mod splitter;
pub mod util;

use axgeom::*;
use compt::Visitor;
use halfpin::*;
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
pub trait Sorter: Copy + Clone + Send + Sync {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
}

#[derive(Copy, Clone)]
pub struct DefaultSorter;

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


pub fn create_ind<T,N:Num>(bots:&mut [T],mut func:impl FnMut(&T)->Rect<N>)->Box<[BBox<N,&mut T>]>{
    bots
    .iter_mut()
    .map(|a| crate::bbox(func(a), a))
    .collect::<Vec<_>>()
    .into_boxed_slice()
}



///A version of Tree where the elements are not sorted along each axis, like a KD Tree.
/// For comparison, a normal kd-tree is provided by [`NotSorted`]. In this tree, the elements are not sorted
/// along an axis at each level. Construction of [`NotSorted`] is faster than [`Tree`] since it does not have to
/// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
/// extra property to be faster.
pub struct NotSorted<'a, T: Aabb>(pub Tree<'a, T>);

impl<'a, T: Aabb> NotSorted<'a, T> {
    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        self.0.vistr_mut()
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.0.vistr()
    }
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.0.get_height()
    }
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
            range: HalfPin::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

pub fn start_build<T: Aabb>(num_levels: usize, bots: &mut [T]) -> TreeBister<T, DefaultSorter> {
    assert!(num_levels >= 1);
    TreeBister {
        bots,
        current_height: num_levels - 1,
        sorter: DefaultSorter,
        is_xaxis: true,
    }
}

pub fn into_tree<T: Aabb>(a: Vec<Node<T>>) -> Tree<T> {
    //TODO get rid of tree container type. it doesnt serve any purpose.

    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(a).unwrap();

    Tree { inner }
}

pub struct TreeBister<'a, T, S> {
    bots: &'a mut [T],
    current_height: usize,
    sorter: S,
    is_xaxis: bool,
}

pub struct Res<'a, T: Aabb, S> {
    pub node: NodeFinisher<'a, T, S>,
    pub rest: Option<[TreeBister<'a, T, S>; 2]>,
}

impl<'a, T: Aabb, S: Sorter> TreeBister<'a, T, S> {
    fn get_height(&self) -> usize {
        self.current_height
    }
    pub fn build_and_next(self) -> Res<'a, T, S> {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinisher {
                mid: self.bots,
                div: None,
                is_xaxis: self.is_xaxis,
                sorter: self.sorter,
            };

            Res { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> ConstructResult<T> {
                if bots.is_empty() {
                    return ConstructResult {
                        mid: &mut [],
                        div: None,
                        left: &mut [],
                        right: &mut [],
                    };
                }

                let med_index = bots.len() / 2;
                let (_, med, _) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::util::compare_bots(div_axis, a, b)
                });

                let med_val = med.get().get_range(div_axis).start;

                //It is very important that the median bot end up be binned into the middile bin.
                //We know this must be true because we chose the divider to be the medians left border,
                //and we binned so that all bots who intersect with the divider end up in the middle bin.
                //Very important that if a bots border is exactly on the divider, it is put in the middle.
                //If this were not true, there is no guarantee that the middile bin has bots in it even
                //though we did pick a divider.
                let binned = oned::bin_middle_left_right(div_axis, &med_val, bots);

                ConstructResult {
                    mid: binned.middle,
                    div: Some(med_val),
                    left: binned.left,
                    right: binned.right,
                }
            }

            let rr = if self.is_xaxis {
                construct_non_leaf(axgeom::XAXIS, self.bots)
            } else {
                construct_non_leaf(axgeom::YAXIS, self.bots)
            };

            let finish_node = NodeFinisher {
                mid: rr.mid,
                div: rr.div,
                is_xaxis: self.is_xaxis,
                sorter: self.sorter,
            };

            let left = rr.left;
            let right = rr.right;

            Res {
                node: finish_node,
                rest: Some([
                    TreeBister {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                    TreeBister {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                ]),
            }
        }
    }

    pub fn recurse_seq(self, res: &mut Vec<Node<'a, T>>) {
        let Res { node, rest } = self.build_and_next();
        res.push(node.finish());
        if let Some([left, right]) = rest {
            left.recurse_seq(res);
            right.recurse_seq(res);
        }
    }
}

struct ConstructResult<'a, T: Aabb> {
    div: Option<T::Num>,
    mid: &'a mut [T],
    right: &'a mut [T],
    left: &'a mut [T],
}

pub use axgeom::rect;

///Shorthand constructor of [`node::BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: axgeom::Rect<N>, inner: T) -> node::BBox<N, T> {
    node::BBox::new(rect, inner)
}

type TreeInner<N> = compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>;

/// A space partitioning tree.
#[repr(transparent)]
pub struct Tree<'a, T: Aabb> {
    inner: TreeInner<Node<'a, T>>,
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
    let num_level = num_level::default(bots.len());
    let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
    let vistr = start_build(num_level, bots);
    vistr.recurse_seq(&mut buffer);
    into_tree(buffer)
}

pub fn new_par<T: Aabb>(bots: &mut [T]) -> Tree<T>
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    let num_level = num_level::default(bots.len());
    let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
    let vistr = start_build(num_level, bots);
    par::recurse_par(vistr, 5, &mut buffer);
    into_tree(buffer)
}

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
    inner: C,
    nodes: NodeDataCollection<<C::T as Aabb>::Num>,
}

impl<C: Container> TreeOwned<C>
where
    C::T: Aabb,
{
    pub fn new(mut a: C) -> Self {
        let t = crate::new(a.as_mut());
        let data = t.into_node_data();
        TreeOwned {
            inner: a,
            nodes: data,
        }
    }
    pub fn as_tree(&mut self) -> Tree<C::T> {
        Tree::from_node_data(&self.nodes, HalfPin::new(self.inner.as_mut()))
    }
    pub fn as_slice_mut(&mut self) -> HalfPin<&mut [C::T]> {
        HalfPin::new(self.inner.as_mut())
    }
    pub fn into_inner(self) -> C {
        self.inner
    }
}
impl<C: Container + Clone> TreeOwned<C>
where
    C::T: Aabb,
{
    pub fn clone_inner(&self) -> C {
        self.inner.clone()
    }
}

pub struct NodeDataCollection<N: Num> {
    inner: Vec<NodeData<N>>,
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn from_node_data(data: &NodeDataCollection<T::Num>, bots: HalfPin<&'a mut [T]>) -> Self {
        let mut last = Some(bots);

        let a: Vec<_> = data
            .inner
            .iter()
            .map(move |x| {
                let (range, rest) = last.take().unwrap().split_at_mut(x.range);
                last = Some(rest);
                Node {
                    range,
                    cont: x.cont,
                    div: x.div,
                }
            })
            .collect();
        Tree {
            inner: compt::dfs_order::CompleteTreeContainer::from_preorder(a).unwrap(),
        }
    }

    pub fn into_node_data(self) -> NodeDataCollection<T::Num> {
        NodeDataCollection {
            inner: self
                .inner
                .into_nodes()
                .into_vec()
                .into_iter()
                .map(|x| NodeData {
                    range: x.range.len(),
                    cont: x.cont,
                    div: x.div,
                })
                .collect(),
        }
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=40;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_height(),build::TreePreBuilder::new(NUM_ELEMENT).get_height());
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.as_tree().get_height()
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
    pub fn into_inner(
        self,
    ) -> compt::dfs_order::CompleteTreeContainer<Node<'a, T>, compt::dfs_order::PreOrder> {
        self.inner
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
    #[warn(deprecated)]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.as_tree().get_nodes().len()
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
    pub fn get_nodes(&self) -> &[Node<'a, T>] {
        self.inner.as_tree().get_nodes()
    }

    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes_mut().get_index_mut(0).range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes_mut(&mut self) -> HalfPin<&mut [Node<'a, T>]> {
        HalfPin::new(self.inner.as_tree_mut().get_nodes_mut())
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
    ///    *b.unpack_inner()+=1;    
    /// }
    /// assert_eq!(bots[0].inner,1);
    ///```
    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        VistrMut::new(self.inner.as_tree_mut().vistr_mut())
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// let mut test = Vec::new();
    /// for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.range.iter()){
    ///    test.push(b);
    /// }
    /// assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.inner.as_tree().vistr()
    }
}
