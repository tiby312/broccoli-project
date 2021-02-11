use super::*;

#[derive(Serialize,Debug)]
pub struct Record {
    brocc: f32,
    brocc_par: f32,
    sweep: f32,
    naive: f32,
    nosort_par: f32,
    nosort: f32,
}
const BENCH_STOP_NAIVE_AT: usize = 3000;
const BENCH_STOP_SWEEP_AT: usize = 6000;

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

        let c3 = if num_bots <= BENCH_STOP_SWEEP_AT {
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

        let c4 = if num_bots <= BENCH_STOP_NAIVE_AT {
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

        if num_bots <= BENCH_STOP_NAIVE_AT {
            for (i, &b) in bot_inner.iter().enumerate() {
                assert_eq!(b, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
            }
        }

        Record {
            brocc: c1 as f32,
            brocc_par: c0 as f32,
            sweep: c3 as f32,
            naive: c4 as f32,
            nosort_par: c5 as f32,
            nosort: c6 as f32,
        }
    }
}



pub fn handle_bench(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "colfind_bench_0.2",
        title: "Bench of space partitioning algs with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..10_000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots))),
        stop_values: &[
            ("naive", BENCH_STOP_NAIVE_AT as f32),
            ("sweep", BENCH_STOP_SWEEP_AT as f32),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_bench_0.05",
        title: "Bench of space partitioning algs with abspiral(x,0.05)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: (0usize..10_000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, Record::new(0.05, num_bots))),
        stop_values: &[
            ("naive", BENCH_STOP_NAIVE_AT as f32),
            ("sweep", BENCH_STOP_SWEEP_AT as f32),
        ],
    });


    fb.make_graph(Args {
        filename: "colfind_bench_grow",
        title: "Bench of space partitioning algs with abspiral(3000,grow)",
        xname: "Num Itersections",
        yname: "Time in Seconds",
        plots: abspiral_grow_iter2(0.001, 0.008, 0.0001)
        .map(|grow| (num_intersections_for_grow(grow,3000) as f32, Record::new(grow, 3000))),
        stop_values: &[],
    });


    fb.make_graph(Args {
        filename: "colfind_bench_grow_wide",
        title: "Bench of space partitioning algs with abspiral(3000,grow)",
        xname: "Num Itersections",
        yname: "Time in Seconds",
        plots: abspiral_grow_iter2(0.01, 0.2, 0.002)
        .map(|grow| (num_intersections_for_grow(grow,3000) as f32, Record::new(grow, 3000))),
        stop_values: &[],
    });
}
