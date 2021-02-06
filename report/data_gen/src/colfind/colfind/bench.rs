use super::*;

#[derive(Serialize,Debug)]
pub struct Record {
    bench_alg: f32,
    bench_par: f32,
    bench_sweep: f32,
    bench_naive: f32,
    bench_nosort_par: f32,
    bench_nosort_seq: f32,
}
const bench_stop_naive_at: usize = 3000;
const bench_stop_sweep_at: usize = 6000;

impl Record {
    pub fn new(grow: f64, num_bots: usize) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c0 = bench_closure(|| {
            let mut tree = broccoli::new_par(RayonJoin, &mut bots);
            tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c1 = bench_closure(|| {
            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() -= 1;
                **b.unpack_inner() -= 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c3 = if num_bots <= bench_stop_sweep_at {
            bench_closure(|| {
                broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                    **a.unpack_inner() -= 2;
                    **b.unpack_inner() -= 2;
                });
            })
        } else {
            0.0
        };

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c4 = if num_bots <= bench_stop_naive_at {
            bench_closure(|| {
                broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                    **a.unpack_inner() += 2;
                    **b.unpack_inner() += 2;
                });
            })
        } else {
            0.0
        };

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c5 = bench_closure(|| {
            let mut tree = NotSorted::new_par(RayonJoin, &mut bots);
            tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let c6 = bench_closure(|| {
            let mut tree = NotSorted::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() -= 1;
                **b.unpack_inner() -= 1;
            });
        });

        if num_bots <= bench_stop_naive_at {
            for (i, &b) in bot_inner.iter().enumerate() {
                assert_eq!(b, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
            }
        }

        Record {
            bench_alg: c1 as f32,
            bench_par: c0 as f32,
            bench_sweep: c3 as f32,
            bench_naive: c4 as f32,
            bench_nosort_par: c5 as f32,
            bench_nosort_seq: c6 as f32,
        }
    }
}



pub fn handle_bench(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "colfind_bench_0.2",
        title: "Comparison of space partitioning algs with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..10_000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots))),
        stop_values: &[
            ("naive", bench_stop_naive_at as f32),
            ("sweep", bench_stop_sweep_at as f32),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_bench_0.2",
        title: "Comparison of space partitioning algs with abspiral(x,0.05)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..10_000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, Record::new(0.05, num_bots))),
        stop_values: &[
            ("naive", bench_stop_naive_at as f32),
            ("sweep", bench_stop_sweep_at as f32),
        ],
    });


    fb.make_graph(Args {
        filename: "colfind_bench_grow",
        title: "Comparison of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Time in Seconds",
        plots: abspiral_grow_iter2(0.001, 0.008, 0.0001)
        .map(|grow| (grow as f32, Record::new(grow, 3000))),
        stop_values: &[
            ("naive", bench_stop_naive_at as f32),
            ("sweep", bench_stop_sweep_at as f32),
        ],
    });


    fb.make_graph(Args {
        filename: "colfind_bench_grow_wide",
        title: "Comparison of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Time in Seconds",
        plots: abspiral_grow_iter2(0.01, 0.2, 0.002)
        .map(|grow| (grow as f32, Record::new(grow, 3000))),
        stop_values: &[
            ("naive", bench_stop_naive_at as f32),
            ("sweep", bench_stop_sweep_at as f32),
        ],
    });
}
