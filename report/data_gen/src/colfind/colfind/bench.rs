use super::*;

#[derive(Serialize, Debug)]
pub struct Record {
    brocc: f64,
    brocc_par: f64,
    sweep: f64,
    naive: f64,
    nosort_par: f64,
    nosort: f64,
}

impl Record {
    pub fn new(grow: f64, num_bots: usize, naive_bench: bool, sweep_bench: bool) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c0 = bench_closure(|| {
            let mut tree = broccoli::new_par(RayonJoin, &mut bots);
            tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c1 = bench_closure(|| {
            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() -= 1;
                **b.unpack_inner() -= 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c3 = if sweep_bench {
            bench_closure(|| {
                broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                    **a.unpack_inner() -= 2;
                    **b.unpack_inner() -= 2;
                });
            })
        } else {
            0.0
        };

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c4 = if naive_bench {
            bench_closure(|| {
                broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                    **a.unpack_inner() += 2;
                    **b.unpack_inner() += 2;
                });
            })
        } else {
            0.0
        };

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c5 = bench_closure(|| {
            let mut tree = NotSorted::new_par(RayonJoin, &mut bots);
            tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

        let c6 = bench_closure(|| {
            let mut tree = NotSorted::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() -= 1;
                **b.unpack_inner() -= 1;
            });
        });

        if naive_bench && sweep_bench {
            for (i, &b) in bot_inner.iter().enumerate() {
                assert_eq!(b, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
            }
        }

        Record {
            brocc: c1 as f64,
            brocc_par: c0 as f64,
            sweep: c3 as f64,
            naive: c4 as f64,
            nosort_par: c5 as f64,
            nosort: c6 as f64,
        }
    }
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    const BENCH_STOP_NAIVE_AT: usize = 3000;
    const BENCH_STOP_SWEEP_AT: usize = 6000;

    fb.make_graph(Args {
        filename: "colfind_bench_default",
        title: &format!("Bench of space partitioning algs with abspiral(x,{})",DEFAULT_GROW),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0, 10_000).map(|num_bots| {
            (
                num_bots as f64,
                Record::new(
                    DEFAULT_GROW,
                    num_bots,
                    num_bots <= BENCH_STOP_NAIVE_AT,
                    num_bots <= BENCH_STOP_SWEEP_AT,
                ),
            )
        }),
        stop_values: &[
            ("naive", BENCH_STOP_NAIVE_AT as f64),
            ("sweep", BENCH_STOP_SWEEP_AT as f64),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_bench_dense",
        title: &format!("Bench of space partitioning algs with abspiral(x,{})",DENSE_GROW),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0, 10_000).map(|num_bots| {
            (
                num_bots as f64,
                Record::new(
                    DENSE_GROW,
                    num_bots,
                    num_bots <= BENCH_STOP_NAIVE_AT,
                    num_bots <= BENCH_STOP_SWEEP_AT,
                ),
            )
        }),
        stop_values: &[
            ("naive", BENCH_STOP_NAIVE_AT as f64),
            ("sweep", BENCH_STOP_SWEEP_AT as f64),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_bench_grow",
        title: "Bench of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Time in Seconds",
        plots: grow_iter(MEGA_MEGA_DENSE_GROW, MEGA_DENSE_GROW)
            .map(|grow| (grow as f64, Record::new(grow, 3_000, true, true))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "colfind_bench_grow_wide",
        title: "Bench of space partitioning algs with abspiral(30_000,grow)",
        xname: "Grow",
        yname: "Time in Seconds",
        plots: grow_iter(MEGA_DENSE_GROW, DENSE_GROW)
            .map(|grow| (grow as f64, Record::new(grow, 30_000, false, true))),
        stop_values: &[],
    });
}
