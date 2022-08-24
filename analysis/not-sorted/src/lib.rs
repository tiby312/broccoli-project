use broccoli::aabb::pin::NodeRef;
use broccoli::{
    aabb::pin::AabbPin,
    aabb::*,
    axgeom::{Axis, AxisDyn, XAXIS, YAXIS},
    queries::{
        self,
        colfind::build::{CollVis, CollisionHandler, NodeHandler},
    },
    
    build::Sorter,
    build::TreeBuildVisitor,
    node::{Node, Vistr, VistrMutPin},
    num_level,
    
};
use broccoli_rayon::{
    build::RayonBuildPar, queries::colfind::NodeHandlerExt, queries::colfind::RayonQueryPar,
};
use compt::Visitor;

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
    fn par_new(bots: &'a mut [T]) -> Self {
        let num_level = num_level::default(bots.len());
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        broccoli_rayon::build::recurse_par(
            broccoli_rayon::build::SEQ_FALLBACK_DEFAULT,
            &mut NoSorter,
            &mut buffer,
            TreeBuildVisitor::new(num_level, bots),
        );
        NotSortedTree::from_nodes(buffer)
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
impl<T: Aabb, Acc: CollisionHandler<T> + Clone> NodeHandlerExt<T> for NoSortNodeHandler<Acc> {
    fn div(&mut self) -> Self {
        self.clone()
    }

    fn add(&mut self, _b: Self) {}
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

    fn handle_nodes_under(&mut self, this_axis: AxisDyn, m: VistrMutPin<Node<T, T::Num>>) {
        {
            let (nn, rest) = m.next();

            if let Some([mut left, mut right]) = rest {
                if let Some(div) = nn.div {
                    let d = nn.into_node_ref();
                    let mut g = InnerRecurser {
                        anchor: DNode {
                            div,
                            cont: d.cont,
                            range: d.range,
                        },
                        anchor_axis: this_axis,
                        handler: self,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
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
        let mut f = NoSortNodeHandler { func };

        let vv = CollVis::new(self.vistr_mut());
        broccoli_rayon::queries::colfind::recurse_par(
            vv,
            &mut f,
            broccoli_rayon::queries::colfind::SEQ_FALLBACK_DEFAULT,
        );
    }
}

fn handle_children2<C: CollisionHandler<T>, T: Aabb>(
    handler: &mut C,
    mut f: HandleChildrenArgs<T, T::Num>,
    _is_left: bool,
) {
    let res = if !f.current_axis.is_equal_to(f.anchor_axis) {
        true
    } else {
        f.current.cont.intersects(f.anchor.cont)
    };

    if res {
        for mut a in f.current.range.iter_mut() {
            for mut b in f.anchor.range.borrow_mut().iter_mut() {
                if a.get().intersects_rect(b.get()) {
                    handler.collide(a.borrow_mut(), b.borrow_mut());
                }
            }
        }
    }
}

struct InnerRecurser<'a, T, N, C> {
    anchor: DNode<'a, T, N>,
    anchor_axis: AxisDyn,
    handler: &'a mut NoSortNodeHandler<C>,
}

impl<'a, T: Aabb, C: CollisionHandler<T>> InnerRecurser<'a, T, T::Num, C> {
    fn recurse(&mut self, this_axis: AxisDyn, m: VistrMutPin<Node<T, T::Num>>, is_left: bool) {
        let anchor_axis = self.anchor_axis;

        let (mut nn, rest) = m.next();

        handle_children2(
            &mut self.handler.func,
            HandleChildrenArgs {
                anchor: self.anchor.borrow(),
                anchor_axis: self.anchor_axis,
                current: nn.borrow_mut().into_node_ref(),
                current_axis: this_axis,
            },
            is_left,
        );

        if let Some([left, right]) = rest {
            if let Some(div) = nn.div {
                if anchor_axis.is_equal_to(this_axis) {
                    match is_left {
                        true => {
                            if div < self.anchor.cont.start {
                                self.recurse(this_axis.next(), right, is_left);
                                return;
                            }
                        }
                        false => {
                            if div >= self.anchor.cont.end {
                                self.recurse(this_axis.next(), left, is_left);
                                return;
                            }
                        }
                    }
                }
            }

            self.recurse(this_axis.next(), left, is_left);
            self.recurse(this_axis.next(), right, is_left);
        }
    }
}

//remove need for second lifetime
struct HandleChildrenArgs<'a, T, N> {
    pub anchor: DNode<'a, T, N>,
    pub current: NodeRef<'a, T, N>,
    pub anchor_axis: AxisDyn,
    pub current_axis: AxisDyn,
}

/// A destructured anchor node.
struct DNode<'a, T, N> {
    pub div: N,
    pub cont: &'a Range<N>,
    pub range: AabbPin<&'a mut [T]>,
}
impl<'a, T, N: Copy> DNode<'a, T, N> {
    fn borrow(&mut self) -> DNode<T, N> {
        DNode {
            div: self.div,
            cont: self.cont,
            range: self.range.borrow_mut(),
        }
    }
}
