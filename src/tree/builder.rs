
use super::*;

///Builder pattern for dinotree.
///For most usecases, the user is suggested to use
///the built in new() functions to create the tree.
///This is provided in cases the user wants more control
///on the behavior of the tree for benching and debuging purposes.
pub struct DinoTreeBuilder<'a, A: Axis, T> {
    axis: A,
    bots: &'a mut [T],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}

impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTreeBuilder<'a, A, T> {
    ///Build not sorted in parallel
    pub fn build_not_sorted_par(&mut self) -> NotSorted<'a,A, T> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
        let inner = create_tree_par(
            self.axis,
            dlevel,
            bots,
            NoSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        );
        NotSorted(inner)
    }

    ///Build in parallel
    pub fn build_par(&mut self) -> DinoTree<'a, A,T> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
        create_tree_par(
            self.axis,
            dlevel,
            bots,
            DefaultSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        )
    }
}

impl<'a, T: Aabb> DinoTreeBuilder<'a, DefaultA, T> {
    ///Create a new builder with a slice of elements that implement `Aabb`.
    pub fn new(bots: &'a mut [T]) -> DinoTreeBuilder<'a, DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
}

impl<'a, A: Axis, T: Aabb> DinoTreeBuilder<'a, A, T> {
    ///Create a new builder with a slice of elements that implement `Aabb`.
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTreeBuilder<'a, A, T> {
        let rebal_strat = BinStrat::NotChecked;

        //we want each node to have space for around num_per_node bots.
        //there are 2^h nodes.
        //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

        //Make this number too small, and the tree will have too many levels,
        //and too much time will be spent recursing.
        //Make this number too high, and you will lose the properties of a tree,
        //and you will end up with just sweep and prune.
        //This number was chosen emprically from running the dinotree_alg_data project,
        //on two different machines.
        let height = compute_tree_height_heuristic(bots.len(), DEFAULT_NUMBER_ELEM_PER_NODE);

        let height_switch_seq = par::SWITCH_SEQUENTIAL_DEFAULT;

        DinoTreeBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }

    ///Build not sorted sequentially
    pub fn build_not_sorted_seq(&mut self) -> NotSorted<'a,A, T> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let inner = create_tree_seq(
            self.axis,
            bots,
            NoSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        );
        NotSorted(inner)
    }

    ///Build sequentially
    pub fn build_seq(&mut self) -> DinoTree< 'a,A, T> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        create_tree_seq(
            self.axis,
            bots,
            DefaultSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        )
    }

    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    #[inline(always)]
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    #[inline(always)]
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.height_switch_seq = height;
        self
    }

    ///Build with a Splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> DinoTree<'a, A,T> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        create_tree_seq(
            self.axis,
            bots,
            DefaultSorter,
            splitter,
            self.height,
            self.rebal_strat,
        )
    }
}







fn create_tree_seq<'a, A: Axis, T: Aabb, K: Splitter>(
    div_axis: A,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) -> DinoTree<'a,A,T> {

    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder_seq(div_axis, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    DinoTree{inner:DinoTreeInner{axis:div_axis,inner:tree}}
}

fn create_tree_par<
    'a,
    A: Axis,
    JJ: par::Joiner,
    T: Aabb + Send + Sync,
    K: Splitter + Send + Sync,
>(
    div_axis: A,
    dlevel: JJ,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) ->DinoTree<'a,A,T> {

    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder(div_axis, dlevel, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    DinoTree{inner:DinoTreeInner{
        axis:div_axis,
        inner:tree
    }}
}

struct Recurser<'a, T: Aabb, K: Splitter, S: Sorter> {
    height: usize,
    binstrat: BinStrat,
    sorter: S,
    _p: PhantomData<(K, &'a T)>,
}

impl<'a, T: Aabb, K: Splitter, S: Sorter> Recurser<'a, T, K, S> {
    fn create_leaf<A: Axis>(&self, axis: A, rest: &'a mut [T]) -> NodeMut<'a, T> {
        self.sorter.sort(axis.next(), rest);

        let cont = create_cont(axis, rest);

        NodeMut {
            range: PMut::new(rest),
            cont,
            div: None,
        }
    }

    fn create_non_leaf<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
    ) -> (NodeMut<'a, T>, &'a mut [T], &'a mut [T]) {
        match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
            ConstructResult::NonEmpty {
                cont,
                div,
                mid,
                left,
                right,
            } => (
                NodeMut {
                    range: PMut::new(mid),
                    cont,
                    div: Some(div),
                },
                left,
                right,
            ),
            ConstructResult::Empty(empty) => {
                //let (a,empty) = tools::duplicate_empty_slice(empty);
                //let (b,c) = tools::duplicate_empty_slice(empty);
                let node = NodeMut {
                    range: PMut::new(empty),
                    cont: None,
                    div: None,
                };

                (node, &mut [], &mut [])
            }
        }
    }

    fn recurse_preorder_seq<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);
            nodes.push(node);

            let mut splitter2 = splitter.div();

            self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
            self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}
impl<'a, T: Aabb + Send + Sync, K: Splitter + Send + Sync, S: Sorter> Recurser<'a, T, K, S> {
    fn recurse_preorder<A: Axis, JJ: par::Joiner>(
        &self,
        axis: A,
        dlevel: JJ,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);

            nodes.push(node);

            let mut splitter2 = splitter.div();

            let splitter = match dlevel.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let splitter2 = &mut splitter2;

                    //dbg!("PAR SPLIT");

                    let ((splitter, nodes), mut nodes2) = rayon::join(
                        move || {
                            self.recurse_preorder(
                                axis.next(),
                                dleft,
                                left,
                                nodes,
                                splitter,
                                depth + 1,
                            );
                            (splitter, nodes)
                        },
                        move || {
                            let mut nodes2: Vec<_> =
                                Vec::with_capacity(nodes_left(depth, self.height));
                            self.recurse_preorder(
                                axis.next(),
                                dright,
                                right,
                                &mut nodes2,
                                splitter2,
                                depth + 1,
                            );
                            nodes2
                        },
                    );

                    nodes.append(&mut nodes2);
                    splitter
                }
                par::ParResult::Sequential(_) => {
                    //dbg!("SEQ SPLIT");

                    self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
                    self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);
                    splitter
                }
            };

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont(b: &mut test::Bencher) {
    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(move |pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont2(b: &mut test::Bencher) {
    fn create_cont2<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
        let left = middle
            .iter()
            .map(|a| a.get().get_range(axis).left)
            .min()
            .unwrap();
        let right = middle
            .iter()
            .map(|a| a.get().get_range(axis).right)
            .max()
            .unwrap();
        axgeom::Range { left, right }
    }

    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont2(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> Option<axgeom::Range<T::Num>> {
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

            Some(axgeom::Range {
                start: min,
                end: max,
            })
        }
        None => None,
    }
}

enum ConstructResult<'a, T: Aabb> {
    NonEmpty {
        div: T::Num,
        cont: Option<axgeom::Range<T::Num>>,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: Aabb>(
    bin_strat: BinStrat,
    sorter: impl Sorter,
    div_axis: impl Axis,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure =
            move |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

        let k = {
            let mm = bots.len() / 2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).start
    };

    //TODO. its possible that middle is empty is the ranges inserted had
    //zero length.
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned = match bin_strat {
        BinStrat::Checked => oned::bin_middle_left_right(div_axis, &med, bots),
        BinStrat::NotChecked => unsafe {
            oned::bin_middle_left_right_unchecked(div_axis, &med, bots)
        },
    };

    //debug_assert!(!binned.middle.is_empty());
    sorter.sort(div_axis.next(), binned.middle);

    let cont = create_cont(div_axis, binned.middle);

    //We already know that the middile is non zero in length.

    ConstructResult::NonEmpty {
        mid: binned.middle,
        cont,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
