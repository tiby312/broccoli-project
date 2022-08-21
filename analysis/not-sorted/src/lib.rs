use broccoli::{
    aabb::pin::AabbPin,
    aabb::*,
    axgeom::{Axis, AxisDyn, XAXIS, YAXIS},
    queries::{
        self,
        colfind::build::{CollVis, CollisionHandler, HandleChildrenArgs, NodeHandler},
    },
    tree::{
        build::Sorter,
        build::TreeBuildVisitor,
        node::{Node, Vistr, VistrMutPin},
        num_level,
    },
};
use broccoli_rayon::{
    build::RayonBuildPar, queries::colfind::NodeHandlerExt, queries::colfind::RayonQueryPar,
};

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl<T: Aabb> Sorter<T> for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [T]) {}
}

///
/// A tree where the elements in a node are not sorted.
///
pub struct NotSortedTree<'a, T: Aabb> {
    nodes: Vec<Node<'a, T, T::Num>>,
}

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn from_nodes(nodes: Vec<Node<'a, T, T::Num>>) -> Self {
        NotSortedTree { nodes }
    }
    pub fn new(bots: &'a mut [T]) -> Self
    where
        T: ManySwap,
    {
        let num_level = num_level::default(bots.len());
        let num_nodes = num_level::num_nodes(num_level);
        let mut nodes = Vec::with_capacity(num_nodes);
        TreeBuildVisitor::new(num_level, bots).recurse_seq(&mut NoSorter, &mut nodes);
        assert_eq!(num_nodes, nodes.len());
        NotSortedTree { nodes }
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
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

impl<T: Aabb> NotSortedTree<'_, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        CollVis::new(self.vistr_mut()).recurse_seq(&mut NoSortNodeHandler::new(func));
    }
}

impl<'a, T: Aabb + ManySwap> RayonBuildPar<'a, T> for NotSortedTree<'a, T>
where
    T: Send,
    T::Num: Send,
{
    fn par_new_ext(bots: &'a mut [T], num_level: usize, num_seq_fallback: usize) -> Self {
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        broccoli_rayon::build::recurse_par(
            num_seq_fallback,
            &mut NoSorter,
            &mut buffer,
            TreeBuildVisitor::new(num_level, bots),
        );
        NotSortedTree::from_nodes(buffer)
    }

    fn par_new(bots: &'a mut [T]) -> Self {
        let num_level = num_level::default(bots.len());
        Self::par_new_ext(bots, num_level, broccoli_rayon::build::SEQ_FALLBACK_DEFAULT)
    }
}

#[derive(Clone)]
pub struct NoSortNodeHandler<F> {
    pub func: F,
}
impl<F> NoSortNodeHandler<F> {
    pub fn new<T: Aabb>(func: F) -> Self
    where
        F: CollisionHandler<T>,
    {
        NoSortNodeHandler { func }
    }
}

impl<T: Aabb, F: CollisionHandler<T>> NodeHandler<T> for NoSortNodeHandler<F> {
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        fn foop<T: Aabb, F: CollisionHandler<T>>(
            func: &mut F,
            axis: impl Axis,
            bots: AabbPin<&mut [T]>,
            is_leaf: bool,
        ) {
            if !is_leaf {
                queries::for_every_pair(bots, move |a, b| {
                    if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                        func.collide(a, b);
                    }
                });
            } else {
                queries::for_every_pair(bots, move |a, b| {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a, b);
                    }
                });
            }
        }

        match axis.next() {
            AxisDyn::X => foop(&mut self.func, XAXIS, bots, is_leaf),
            AxisDyn::Y => foop(&mut self.func, YAXIS, bots, is_leaf),
        }
    }

    fn handle_children(&mut self, mut f: HandleChildrenArgs<T>, _is_left: bool) {
        let res = if !f.current_axis.is_equal_to(f.anchor_axis) {
            true
        } else {
            f.current.cont.intersects(f.anchor.cont)
        };

        if res {
            for mut a in f.current.range.iter_mut() {
                for mut b in f.anchor.range.borrow_mut().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        self.func.collide(a.borrow_mut(), b.borrow_mut());
                    }
                }
            }
        }
    }
}

impl<'a, T: Aabb> RayonQueryPar<'a, T> for NotSortedTree<'a, T> {
    fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
        &mut self,
        _acc: Acc,
        _div: A,
        _add: B,
        _func: F,
    ) -> Acc
    where
        A: FnMut(&mut Acc) -> Acc + Clone + Send,
        B: FnMut(&mut Acc, Acc) + Clone + Send,
        F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        Acc: Send,
        T: Send,
        T::Num: Send,
    {
        unimplemented!();
    }

    fn par_find_colliding_pairs<F>(&mut self, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        T: Send,
        T::Num: Send,
    {
        let mut f = DefaultNoSortNodeHandler {
            inner: NoSortNodeHandler { func },
        };

        let vv = CollVis::new(self.vistr_mut());
        broccoli_rayon::queries::colfind::recurse_par(
            vv,
            &mut f,
            broccoli_rayon::queries::colfind::SEQ_FALLBACK_DEFAULT,
        );
    }
}

///
/// Need to do new type pattern since NodeHandlerExt is a foreign trait.
///
pub struct DefaultNoSortNodeHandler<F> {
    inner: NoSortNodeHandler<F>,
}

impl<T: Aabb, Acc> NodeHandler<T> for DefaultNoSortNodeHandler<Acc>
where
    Acc: CollisionHandler<T>,
{
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        self.inner.handle_node(axis, bots, is_leaf)
    }

    #[inline(always)]
    fn handle_children(&mut self, f: HandleChildrenArgs<T>, is_left: bool) {
        self.inner.handle_children(f, is_left)
    }
}
impl<T: Aabb, Acc: CollisionHandler<T> + Clone> NodeHandlerExt<T>
    for DefaultNoSortNodeHandler<Acc>
{
    fn div(&mut self) -> Self {
        DefaultNoSortNodeHandler {
            inner: self.inner.clone(),
        }
    }

    fn add(&mut self, _b: Self) {}
}
