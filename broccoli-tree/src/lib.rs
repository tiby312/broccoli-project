//! Contains code to help build the [`Tree`] structure with more options than
//! just using [`broccoli::new`](crate::new).

mod assert;
pub mod node;
mod oned;
pub mod treepin;
pub mod util;

use axgeom::*;
use compt::Visitor;
use node::*;
use treepin::*;

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
            range: TreePin::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

pub fn start_build<T: Aabb, N: Sorter>(
    num_levels: usize,
    bots: &mut [T],
    sorter: N,
) -> TreeBister<T, N> {
    assert!(num_levels >= 1);
    TreeBister {
        bots,
        current_height: num_levels - 1,
        sorter,
        is_xaxis: true,
    }
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

pub trait TreeBuild<T: Aabb, S: Sorter>: Sized {
    fn sorter(&self) -> S;

    fn num_level(&self, num_bots: usize) -> usize {
        num_level::default(num_bots)
    }
    fn height_seq_fallback(&self) -> usize {
        5
    }

    fn build_owned<C: Container<T = T>>(self, mut bots: C) -> TreeOwned<C, S> {
        let j = bots.as_mut();
        let length = j.len();

        let t = self.build(j);
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }
    fn build_owned_par<C: Container<T = T>>(self, mut bots: C) -> TreeOwned<C, S>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        let j = bots.as_mut();
        let length = j.len();

        let t = self.build_par(j);
        let data = t.into_node_data_tree();
        TreeOwned {
            inner: bots,
            nodes: data,
            length,
        }
    }
    fn build(self, bots: &mut [T]) -> TreeInner<Node<T>, S> {
        let total_num_elem = bots.len();
        let num_level = self.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = start_build(num_level, bots, self.sorter());
        vistr.recurse_seq(&mut buffer);
        TreeInner {
            nodes: buffer,
            sorter: self.sorter(),
            total_num_elem,
        }
    }
    fn build_par(self, bots: &mut [T]) -> TreeInner<Node<T>, S>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        pub fn recurse_par<'a, T: Aabb, S: Sorter>(
            vistr: TreeBister<'a, T, S>,
            height_seq_fallback: usize,
            buffer: &mut Vec<Node<'a, T>>,
        ) where
            T: Send,
            T::Num: Send,
        {
            if vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(buffer);
            } else {
                let Res { node, rest } = vistr.build_and_next();

                if let Some([left, right]) = rest {
                    let (_, mut a) = rayon_core::join(
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
        let num_level = self.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = start_build(num_level, bots, self.sorter());
        recurse_par(vistr, self.height_seq_fallback(), &mut buffer);

        TreeInner {
            nodes: buffer,
            sorter: self.sorter(),
            total_num_elem,
        }
    }

    fn build_splitter<SS: Splitter>(
        self,
        bots: &mut [T],
        splitter: SS,
    ) -> (TreeInner<Node<T>, S>, SS) {
        pub fn recurse_seq_splitter<'a, T: Aabb, S: Sorter, SS: Splitter>(
            vistr: TreeBister<'a, T, S>,
            res: &mut Vec<Node<'a, T>>,
            splitter: SS,
        ) -> SS {
            let Res { node, rest } = vistr.build_and_next();
            res.push(node.finish());
            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();

                let s1 = recurse_seq_splitter(left, res, s1);
                let s2 = recurse_seq_splitter(right, res, s2);

                s1.add(s2)
            } else {
                splitter
            }
        }
        let total_num_elem = bots.len();
        let num_level = self.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = start_build(num_level, bots, self.sorter());

        let splitter = recurse_seq_splitter(vistr, &mut buffer, splitter);

        let t = TreeInner {
            nodes: buffer,
            sorter: self.sorter(),
            total_num_elem,
        };
        (t, splitter)
    }

    fn build_splitter_par<SS: Splitter + Send>(
        self,
        bots: &mut [T],
        splitter: SS,
    ) -> (TreeInner<Node<T>, S>, SS)
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        pub fn recurse_par_splitter<'a, T: Aabb, S: Sorter, SS: Splitter + Send>(
            vistr: TreeBister<'a, T, S>,
            height_seq_fallback: usize,
            buffer: &mut Vec<Node<'a, T>>,
            splitter: SS,
        ) -> SS
        where
            T: Send,
            T::Num: Send,
        {
            if vistr.get_height() <= height_seq_fallback {
                vistr.recurse_seq(buffer);
                splitter
            } else {
                let Res { node, rest } = vistr.build_and_next();

                if let Some([left, right]) = rest {
                    let (s1, s2) = splitter.div();

                    let (s1, (mut a, s2)) = rayon_core::join(
                        || {
                            buffer.push(node.finish());
                            recurse_par_splitter(left, height_seq_fallback, buffer, s1)
                        },
                        || {
                            let mut f = vec![];
                            let v = recurse_par_splitter(right, height_seq_fallback, &mut f, s2);
                            (f, v)
                        },
                    );

                    buffer.append(&mut a);
                    s1.add(s2)
                } else {
                    splitter
                }
            }
        }
        let total_num_elem = bots.len();
        let num_level = self.num_level(bots.len()); //num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = start_build(num_level, bots, self.sorter());

        let splitter =
            recurse_par_splitter(vistr, self.height_seq_fallback(), &mut buffer, splitter);

        let t = TreeInner {
            nodes: buffer,
            sorter: self.sorter(),
            total_num_elem,
        };
        (t, splitter)
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
    DefaultSorter.build(bots)
}

pub fn new_par<T: Aabb>(bots: &mut [T]) -> Tree<T>
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    DefaultSorter.build_par(bots)
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
        DefaultSorter.build_owned(bots)
    }
}

impl<C: Container, S: Sorter> TreeOwned<C, S>
where
    C::T: Aabb,
{
    pub fn as_tree(&mut self) -> TreeInner<Node<C::T>, S> {
        let j = self.inner.as_mut();
        assert_eq!(j.len(), self.length);

        self.nodes.clone().into_tree(TreePin::from_mut(j))
    }
    pub fn as_slice_mut(&mut self) -> TreePin<&mut [C::T]> {
        let j = self.inner.as_mut();
        assert_eq!(j.len(), self.length);

        TreePin::new(j)
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

#[derive(Clone)]
pub struct TreeInner<N, S> {
    total_num_elem: usize,
    nodes: Vec<N>,
    sorter: S,
}

pub type Tree<'a, T> = TreeInner<Node<'a, T>, DefaultSorter>;

impl<S: Sorter, H: node::HasElem> TreeInner<H, S> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = TreePin<&mut H::T>> {
        self.nodes.iter_mut().flat_map(|x| x.get_elems().iter_mut())
    }
}

impl<S: Sorter, H> TreeInner<H, S> {
    pub fn node_map<K>(self, func: impl FnMut(H) -> K) -> TreeInner<K, S> {
        let sorter = self.sorter;
        let nodes = self.nodes.into_iter().map(func).collect();
        TreeInner {
            nodes,
            sorter,
            total_num_elem: self.total_num_elem,
        }
    }
}

pub fn as_node_tree<N>(
    vec: &Vec<N>,
) -> compt::dfs_order::CompleteTree<N, compt::dfs_order::PreOrder> {
    compt::dfs_order::CompleteTree::from_preorder(vec).unwrap()
}
pub fn as_node_tree_mut<N>(
    vec: &mut Vec<N>,
) -> compt::dfs_order::CompleteTreeMut<N, compt::dfs_order::PreOrder> {
    compt::dfs_order::CompleteTreeMut::from_preorder_mut(vec).unwrap()
}

impl<S, H> TreeInner<H, S> {
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
    #[deprecated(note = " use num_levels()")]
    pub fn get_height(&self) -> usize {
        as_node_tree(&self.nodes).get_height()
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
    #[warn(deprecated)]
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
    pub fn get_nodes(&self) -> &[H] {
        &self.nodes
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
    pub fn get_nodes_mut(&mut self) -> TreePin<&mut [H]> {
        TreePin::from_mut(&mut self.nodes)
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
    pub fn vistr_mut(&mut self) -> VistrMut<H> {
        let tree = as_node_tree_mut(&mut self.nodes);
        VistrMut::new(tree.vistr_mut())
    }

    #[inline(always)]
    pub fn vistr_mut_raw(&mut self) -> compt::dfs_order::VistrMut<H, compt::dfs_order::PreOrder> {
        let tree = as_node_tree_mut(&mut self.nodes);
        tree.vistr_mut()
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
    pub fn vistr(&self) -> Vistr<H> {
        let tree = as_node_tree(&self.nodes);

        tree.vistr()
    }

    pub fn sorter(&self) -> S
    where
        S: Copy,
    {
        self.sorter
    }
    /*
    pub fn new(sorter: S, a: Vec<H>) -> TreeInner<H, S> {
        let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(a).unwrap();

        TreeInner {
            nodes: inner,
            sorter,
        }
    }
    */
}

impl<N: Num, S: Sorter> TreeInner<NodeData<N>, S> {
    ///
    /// The first tree has all elements for which the predicate returned true.
    ///
    pub fn subdivide_by_fn<T: Aabb<Num = N>>(
        self,
        bots: &mut [T],
        func: impl Fn(&T) -> bool,
    ) -> [(&mut [T], TreeInner<NodeData<N>, S>); 2] {
        pub fn partition_index<T, P>(data: &mut [T], predicate: P) -> usize
        where
            P: Fn(&T) -> bool,
        {
            let len = data.len();
            if len == 0 {
                return 0;
            }
            let (mut l, mut r) = (0, len - 1);
            loop {
                while l < len && predicate(&data[l]) {
                    l += 1;
                }
                while r > 0 && !predicate(&data[r]) {
                    r -= 1;
                }
                if l >= r {
                    return l;
                }
                data.swap(l, r);
            }
        }

        let idx = partition_index(bots, func);
        let (left, right) = bots.split_at_mut(idx);

        let tree_left = self.sorter.build(left).into_node_data_tree();
        let tree_right = self.sorter.build(right).into_node_data_tree();

        [(left, tree_left), (right, tree_right)]
    }

    //TODO use this in multirect demo
    pub fn subdivide_by_line<T: Aabb<Num = N>, A: Axis>(
        self,
        bots: &mut [T],
        div: N,
        axis: A,
    ) -> [(&mut [T], TreeInner<NodeData<N>, S>); 3] {
        let binned = oned::bin_middle_left_right(axis, &div, bots);

        let tree_left = self.sorter.build(binned.left).into_node_data_tree();
        let tree_mid = self.sorter.build(binned.middle).into_node_data_tree();
        let tree_right = self.sorter.build(binned.right).into_node_data_tree();

        [
            (binned.left, tree_left),
            (binned.middle, tree_mid),
            (binned.right, tree_right),
        ]
    }
}
impl<N: Num, S: Sorter> TreeInner<NodeData<N>, S> {
    pub fn into_tree<T: Aabb<Num = N>>(self, bots: TreePin<&mut [T]>) -> TreeInner<Node<T>, S> {
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
    pub fn into_node_data_tree(self) -> TreeInner<NodeData<T::Num>, S> {
        self.node_map(|x| NodeData {
            range: x.range.len(),
            cont: x.cont,
            div: x.div,
        })
    }
}

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(self) -> (Self, Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(self, b: Self) -> Self;
}
