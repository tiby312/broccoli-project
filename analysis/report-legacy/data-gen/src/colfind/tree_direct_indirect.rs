use super::*;

pub trait TestTrait: Copy + Send + Sync {}
impl<T: Copy + Send + Sync> TestTrait for T {}

#[derive(Copy, Clone)]
struct Bot<T> {
    _num: usize,
    _aabb: Rect<i32>,
    _val: T,
}

#[derive(Copy, Clone, Debug)]
struct TestResult {
    rebal: f64,
    query: f64,
}

fn test_seq<T: Aabb + ManySwap>(
    bots: &mut [T],
    func: impl Fn(AabbPin<&mut T>, AabbPin<&mut T>),
) -> TestResult {
    let (mut tree, construct_time) = bench_closure_ret(|| broccoli::Tree::new(bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.find_colliding_pairs(|a, b| {
            func(a, b);
        });
        tree
    });

    black_box(tree);

    TestResult {
        rebal: construct_time as f64,
        query: query_time as f64,
    }
}

/*
fn test_par<T: Aabb + Send + Sync>(
    bots: &mut [T],
    func: impl Fn(AabbPin<&mut T>, AabbPin<&mut T>) + Send + Sync,
) -> TestResult
where
    T::Num: Send + Sync,
{
    let (mut tree, construct_time) =
        bench_closure_ret(|| broccoli::tree::TreeInner::new_par(DefaultSorter, bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.colliding_pairs_par(|a, b| {
            func(a, b);
        });
        tree
    });

    black_box(tree);

    TestResult {
        rebal: construct_time as f64,
        query: query_time as f64,
    }
}
*/

#[derive(Serialize, Copy, Clone, Debug)]
struct RebalRecord {
    direct_seq: f64,

    indirect_seq: f64,

    default_seq: f64,
}

#[derive(Serialize, Copy, Clone, Debug)]
struct QueryRecord {
    direct_seq: f64,

    indirect_seq: f64,

    default_seq: f64,
}
#[derive(Copy, Clone, Debug)]
struct CompleteTestResult {
    rebal: RebalRecord,
    query: QueryRecord,
}
impl CompleteTestResult {
    fn new<T: TestTrait>(num_bots: usize, grow: f64, t: T) -> CompleteTestResult {
        let direct_seq = {
            let bots =
                distribute_iter(grow, (0..num_bots as isize).map(|a| (a, t)), |a| a.to_i32());

            let mut bots: Vec<_> = bots.into_iter().map(ManySwappable).collect();

            test_seq(&mut bots, |b, c| {
                b.unpack_inner().0 += 1;
                c.unpack_inner().0 += 1;
            })
        };

        let indirect_seq = {
            let mut bots =
                distribute_iter(grow, (0..num_bots as isize).map(|a| (a, t)), |a| a.to_i32());

            let mut indirect: Vec<_> = bots.iter_mut().collect();

            test_seq(&mut indirect, |b, c| {
                b.unpack_inner().0 += 1;
                c.unpack_inner().0 += 1;
            })
        };
        let default_seq = {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| (0isize, t)).collect();

            let mut default = distribute(grow, &mut bot_inner, |a| a.to_i32());

            test_seq(&mut default, |b, c| {
                b.unpack_inner().0 += 1;
                c.unpack_inner().0 += 1;
            })
        };

        CompleteTestResult {
            rebal: RebalRecord {
                direct_seq: direct_seq.rebal,
                indirect_seq: indirect_seq.rebal,
                default_seq: default_seq.rebal,
            },
            query: QueryRecord {
                direct_seq: direct_seq.query,
                indirect_seq: indirect_seq.query,
                default_seq: default_seq.query,
            },
        }
    }
}

pub fn handle(fb: &mut FigureBuilder) {
    handle_num_bots(fb, "default", DEFAULT_GROW, [0u8; 8]);
    handle_num_bots(fb, "default", DEFAULT_GROW, [0u8; 16]);
    handle_num_bots(fb, "default", DEFAULT_GROW, [0u8; 32]);
    handle_num_bots(fb, "default", DEFAULT_GROW, [0u8; 128]);
    handle_num_bots(fb, "default", DEFAULT_GROW, [0u8; 256]);
    handle_num_bots(fb, "dense", DENSE_GROW, [0u8; 128]);
    handle_num_bots(fb, "sparse", SPARSE_GROW, [0u8; 128]);
}

fn handle_num_bots<T: TestTrait>(fb: &mut FigureBuilder, prefix: &str, grow: f64, val: T) {
    let mut rects = Vec::new();

    for num_bots in n_iter(0, 30_000).rev() {
        let r = CompleteTestResult::new(num_bots, grow, val);
        rects.push((num_bots as f64, r));
    }

    let name = format!("{}_bytes", core::mem::size_of::<T>());

    fb.make_graph(Args {
        filename: &format!("tree_direct_indirect_rebal_{}_{}", prefix, name),
        title: &format!("Bench of rebal:{} with abspiral(num,{})", name, grow),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: rects.iter().map(|x| (x.0, x.1.rebal)),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: &format!("tree_direct_indirect_query_{}_{}", prefix, name),
        title: &format!("Bench of query:{} with abspiral(num,{})", name, grow),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: rects.iter().map(|x| (x.0, x.1.query)),
        stop_values: &[],
    });
}
