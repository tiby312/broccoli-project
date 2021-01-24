use super::*;
use par::ParallelBuilder;
///Builder pattern for Tree.
///For most usecases, the user is suggested to use
///the built in new() functions to create the tree.
///This is provided in cases the user wants more control
///on the behavior of the tree for benching and debuging purposes.
pub struct TreeBuilder<'a, T> {
    axis: DefaultA,
    bots: &'a mut [T],
    rebal_strat: BinStrat,
    prebuilder: TreePreBuilder,
    par_builder: ParallelBuilder,
}

impl<'a, T: Aabb + Send + Sync> TreeBuilder<'a, T>
where
    T::Num: Send + Sync,
{
    ///Build not sorted in parallel
    pub fn build_not_sorted_par(&mut self, joiner: impl crate::Joinable) -> NotSorted<'a, T> {
        let pswitch = self
            .par_builder
            .build_for_tree_of_height(self.prebuilder.get_height());

        NotSorted(create_tree_par(
            self,
            NoSorter,
            &mut SplitterEmpty,
            pswitch,
            joiner,
        ))
    }

    ///Build in parallel
    pub fn build_par(&mut self, joiner: impl crate::Joinable) -> Tree<'a, T> {
        let pswitch = self
            .par_builder
            .build_for_tree_of_height(self.prebuilder.get_height());

        create_tree_par(self, DefaultSorter, &mut SplitterEmpty, pswitch, joiner)
    }
}

impl<'a, T: Aabb> TreeBuilder<'a, T> {
    ///Create a new builder with a slice of elements that implement `Aabb`.
    pub fn new(bots: &'a mut [T]) -> TreeBuilder<'a, T> {
        let prebuilder = TreePreBuilder::new(bots.len());
        TreeBuilder {
            axis: default_axis(),
            bots,
            rebal_strat: BinStrat::Checked,
            prebuilder,
            par_builder: ParallelBuilder::new(),
        }
    }
}

impl<'a, T: Aabb> TreeBuilder<'a, T> {
    pub fn from_prebuilder(bots: &'a mut [T], prebuilder: TreePreBuilder) -> TreeBuilder<T> {
        TreeBuilder {
            axis: default_axis(),
            bots,
            rebal_strat: BinStrat::Checked,
            prebuilder,
            par_builder: ParallelBuilder::new(),
        }
    }

    ///Build not sorted sequentially
    pub fn build_not_sorted_seq(&mut self) -> NotSorted<'a, T> {
        NotSorted(create_tree_seq(self, NoSorter, &mut SplitterEmpty))
    }

    ///Build sequentially
    pub fn build_seq(&mut self) -> Tree<'a, T> {
        create_tree_seq(self, DefaultSorter, &mut SplitterEmpty)
    }

    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    #[inline(always)]
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.prebuilder = TreePreBuilder::with_height(height);
        self
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    #[inline(always)]
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.par_builder.with_switch_height(height);
        self
    }

    ///Build with a Splitter.
    pub fn build_with_splitter_seq<S: Splitter>(&mut self, splitter: &mut S) -> Tree<'a, T> {
        create_tree_seq(self, DefaultSorter, splitter)
    }
}

fn create_tree_seq<'a, T: Aabb>(
    builder: &mut TreeBuilder<'a, T>,
    sorter: impl Sorter,
    splitter: &mut impl Splitter,
) -> Tree<'a, T> {
    let bots = core::mem::replace(&mut builder.bots, &mut []);

    let num_aabbs = bots.len();

    let cc = builder.prebuilder.num_nodes();
    let mut nodes = Vec::with_capacity(cc);

    let constants = &Constants {
        height: builder.prebuilder.get_height(),
        binstrat: builder.rebal_strat,
        sorter,
    };

    let (m,splitter)=mover::Mover::new(splitter);

    let r = Recurser {
        constants,
        arr: bots,
        depth: 0,
        splitter
    };

    
    let splitter=r.recurse_preorder_seq(builder.axis, &mut nodes);
    assert_eq!(cc, nodes.len());
    m.insert(splitter);

    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = inner
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_aabbs);

    Tree { inner, num_aabbs }
}



//TODO put in own crate
mod mover{
    pub struct Mover<'a,K>{
        inner:&'a mut K
    }

    impl<'a,K> Mover<'a,K>{
        pub fn new(inner:&'a mut K)->(Mover<'a,K>,K){
            let m=Mover{
                inner
            };
            let source=m.inner as *const K;
                
            (m,unsafe{
                let mut k=core::mem::MaybeUninit::uninit();
                let target=&mut k as *mut core::mem::MaybeUninit<K> as *mut K;
                core::ptr::copy_nonoverlapping(source,target,1);
                k.assume_init()
            })
        }

        pub fn insert(self,k:K){
            unsafe{
                let target=self.inner as *mut K;
                let source=&k as *const K;
                core::ptr::copy_nonoverlapping(source,target,1);
            }    
        }
    }
}


fn create_tree_par<'a, T: Aabb>(
    builder: &mut TreeBuilder<'a, T>,
    sorter: impl Sorter,
    splitter: &mut (impl Splitter + Send + Sync),
    par: impl par::Joiner,
    joiner: impl crate::Joinable,
) -> Tree<'a, T>
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    let bots = core::mem::replace(&mut builder.bots, &mut []);

    let num_aabbs = bots.len();
    let cc = builder.prebuilder.num_nodes();

    let mut nodes = Vec::with_capacity(cc);

    let constants = &Constants {
        height: builder.prebuilder.get_height(),
        binstrat: builder.rebal_strat,
        sorter,
    };

    let (m,splitter)=mover::Mover::new(splitter);
    let r = Recurser {
        constants,
        arr: bots,
        depth: 0,
        splitter
    };

    let r = ParallelRecurser {
        inner: r,
        dlevel: par,
        joiner,
    };

    let splitter=r.recurse_preorder(builder.axis, &mut nodes);

    assert_eq!(cc, nodes.len());
    m.insert(splitter);
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = inner
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_aabbs);

    Tree { inner, num_aabbs }
}

struct NonLeafFinisher<'a, A, T: Aabb, S> {
    axis: A,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    sorter: S,
}
impl<'a, A: Axis, T: Aabb, S: Sorter> NonLeafFinisher<'a, A, T, S> {
    #[inline(always)]
    fn finish(self) -> Node<'a, T> {
        self.sorter.sort(self.axis.next(), self.mid);
        let cont = create_cont(self.axis, self.mid);

        Node {
            range: PMut::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

struct Constants<S: Sorter> {
    height: usize,
    binstrat: BinStrat,
    sorter: S,
}

struct Recurser<'a, 'b, T: Aabb, S: Sorter,K:Splitter> {
    constants: &'b Constants<S>,
    depth: usize,
    arr: &'a mut [T],
    splitter:K
}

impl<'a, 'b, T: Aabb, S: Sorter,K:Splitter> Recurser<'a, 'b, T, S,K> {
    #[inline(always)]
    fn create_leaf<A: Axis>(self, axis: A) -> (Node<'a, T>,K) {
        self.constants.sorter.sort(axis.next(), self.arr);

        let cont = create_cont(axis, self.arr);

        (Node {
            range: PMut::new(self.arr),
            cont,
            div: None,
        },self.splitter)
    }

    fn split<A: Axis>(mut self, axis: A) -> (K,NonLeafFinisher<'a, A, T, S>, Self, Self) {
        let (f, left, right) = match construct_non_leaf(self.constants.binstrat, axis, self.arr) {
            ConstructResult::NonEmpty {
                div,
                mid,
                left,
                right,
            } => (
                NonLeafFinisher {
                    mid,
                    div: Some(div),
                    axis,
                    sorter: self.constants.sorter,
                },
                left,
                right,
            ),
            ConstructResult::Empty(mid) => {
                let node = NonLeafFinisher {
                    mid,
                    div: None,
                    axis,
                    sorter: self.constants.sorter,
                };

                (node, &mut [] as &mut [_], &mut [] as &mut [_])
            }
        };
        let (splitter11,splitter22) = self.splitter.div();

        (
            self.splitter,
            f,
            Recurser {
                constants: self.constants,
                depth: self.depth + 1,
                arr: left,
                splitter:splitter11
            },
            Recurser {
                constants: self.constants,
                depth: self.depth + 1,
                arr: right,
                splitter:splitter22
            },
        )
    }

    fn recurse_preorder_seq<A: Axis>(
        self,
        axis: A,
        nodes: &mut Vec<Node<'a, T>>
    ) ->K{
        if self.depth < self.constants.height - 1 {
            
            let (mut splitter,node, left, right) = self.split(axis);

            nodes.push(node.finish());

            let ls=left.recurse_preorder_seq(axis.next(), nodes);
            let rs=right.recurse_preorder_seq(axis.next(), nodes);

            splitter.add(ls, rs);
            splitter
        } else {
            let (node,splitter) = self.create_leaf(axis);
            nodes.push(node);
            splitter
        }
    }
}


enum Either<'a, 'b, A:Axis,T: Aabb, S: Sorter, JJ: par::Joiner, Joiner: crate::Joinable,K:Splitter> {
    Par(K,NonLeafFinisher<'a, A, T, S>,[ParallelRecurser<'a, 'b, T, S, JJ, Joiner,K>; 2]),
    Seq(K,NonLeafFinisher<'a, A, T, S>,[Recurser<'a, 'b, T, S,K>; 2]),
}

struct ParallelRecurser<'a, 'b, T: Aabb, S: Sorter, JJ: par::Joiner, Joiner: crate::Joinable,K:Splitter> {
    inner: Recurser<'a, 'b, T, S,K>,
    dlevel: JJ,
    joiner: Joiner,
}

impl<'a, 'b, T: Aabb, S: Sorter, JJ: par::Joiner, Joiner: crate::Joinable,K:Splitter>
    ParallelRecurser<'a, 'b, T, S, JJ, Joiner,K>
{
    fn split<A: Axis>(
        self,
        axis: A,
    ) -> Either<'a, 'b, A,T, S, JJ, Joiner,K>
     {
        match self.dlevel.next() {
            par::ParResult::Parallel([dleft, dright]) => {
                let joiner = self.joiner;
                let joiner2 = joiner.clone();

                let (splitter,n, left, right) = self.inner.split(axis);
    
                Either::Par(splitter,n,[
                    ParallelRecurser {
                        inner: left,
                        dlevel: dleft,
                        joiner,
                    },
                    ParallelRecurser {
                        inner: right,
                        dlevel: dright,
                        joiner: joiner2,
                    },
                ]) 
            }
            par::ParResult::Sequential(_) => {
                let (splitter,n, left, right) = self.inner.split(axis);

                Either::Seq(splitter,n,[left, right])
            }
        }
    }
    fn recurse_preorder<A: Axis>(
        self,
        axis: A,
        nodes: &mut Vec<Node<'a, T>>,
    ) ->K where
        T: Send + Sync,
        T::Num: Send + Sync,
        K: Send + Sync,
    {
        if self.inner.depth < self.inner.constants.height - 1 {
            
            let depth = self.inner.depth;
            let height = self.inner.constants.height;

            //TODO get rid of this clone somehow
            let joiner = self.joiner.clone();
            match self.split(axis) {
                Either::Par(mut splitter,node,[left, right]) => {
                    
                    let ((ls,nodes), (rs,mut nodes2)) = joiner.join(
                        move |_joiner| {
                            nodes.push(node.finish());

                            let ls = left.recurse_preorder(axis.next(), nodes);
                            (ls,nodes)
                        },
                        move |_joiner| {
                            let n = nodes_left(depth, height-1);
                            let mut nodes2: Vec<_> = Vec::with_capacity(n);
                            let rs=right.recurse_preorder(axis.next(), &mut nodes2);
                            assert_eq!(nodes2.capacity(), n);
                            assert_eq!(nodes2.len(), n);
                            (rs,nodes2)
                        },
                    );

                    nodes.append(&mut nodes2);
                    splitter.add(ls, rs);
                    splitter
                }
                Either::Seq(mut splitter,node,[left, right]) => {
                    nodes.push(node.finish());

                    let ls=left.recurse_preorder_seq(axis.next(), nodes);
                    let rs=right.recurse_preorder_seq(axis.next(), nodes);
                    splitter.add(ls, rs);
                    splitter
                }
            }

            
        } else {
            let (node,splitter) = self.inner.create_leaf(axis);
            nodes.push(node);
            splitter
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

enum ConstructResult<'a, T: Aabb> {
    NonEmpty {
        div: T::Num,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: Aabb>(
    bin_strat: BinStrat,
    div_axis: impl Axis,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure = move |a: &T, b: &T| -> core::cmp::Ordering {
            crate::util::compare_bots(div_axis, a, b)
        };

        let k = {
            let mm = bots.len() / 2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).start
    };

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

    ConstructResult::NonEmpty {
        mid: binned.middle,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
