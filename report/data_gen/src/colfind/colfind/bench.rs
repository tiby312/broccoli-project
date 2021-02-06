use super::*;

#[derive(Debug)]
pub struct BenchRecord {
    bench_alg: f32,
    bench_par: f32,
    bench_sweep: f32,
    bench_naive: f32,
    bench_nosort_par: f32,
    bench_nosort_seq: f32,
}
const bench_stop_naive_at: usize = 3000;
const bench_stop_sweep_at: usize = 6000;

impl BenchRecord {
    pub fn new(grow: f64, num_bots: usize) -> BenchRecord {
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

        BenchRecord {
            bench_alg: c1 as f32,
            bench_par: c0 as f32,
            bench_sweep: c3 as f32,
            bench_naive: c4 as f32,
            bench_nosort_par: c5 as f32,
            bench_nosort_seq: c6 as f32,
        }
    }
}

fn handle_bench_inner<I: Iterator<Item = (f32, BenchRecord)>>(
    it: I,
    fg: &mut FigureBuilder,
    filename: &str,
    title: &str,
    naive: bool,
    sweep: bool,
    xname: &str,
    yname: &str,
) {
    let rects: Vec<_> = it.collect();
    //TODO convert to milliseconds
    let mut plot = plotato::plot(title, xname, yname);
    plot.line("broccoli seq", rects.iter().map(|a| [a.0, a.1.bench_alg]));
    plot.line("broccoli par", rects.iter().map(|a| [a.0, a.1.bench_par]));
    plot.line(
        "nosort seq",
        rects.iter().map(|a| [a.0, a.1.bench_nosort_seq]),
    );
    plot.line(
        "nosort par",
        rects.iter().map(|a| [a.0, a.1.bench_nosort_par]),
    );

    if sweep {
        plot.line(
            "sweep",
            rects
                .iter()
                .map(|a| [a.0, a.1.bench_sweep])
                .take_while(|&[x, _]| x <= bench_stop_sweep_at as f32),
        );
    }

    if naive {
        plot.line(
            "naive",
            rects
                .iter()
                .map(|a| [a.0, a.1.bench_naive])
                .take_while(|&[x, _]| x <= bench_stop_naive_at as f32),
        );
    }

    fg.finish_plot(plot, filename);
}

pub fn handle_bench(fg: &mut FigureBuilder) {
    handle_bench_inner(
        (0..10000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, BenchRecord::new(0.2, num_bots))),
        fg,
        "colfind_bench_0.2",
        
        "Space partitioning algs with abspiral(x,0.2)",
        true,
        true,
        "Number of Elements",
        "Time in Seconds",
    );

    handle_bench_inner(
        (0..10000)
            .step_by(100)
            .map(|num_bots| (num_bots as f32, BenchRecord::new(0.05, num_bots))),
        fg,
        "colfind_bench_0.05",
        
        "Space partitioning algs with abspiral(x,0.05)",
        true,
        true,
        "Number of Elements",
        "Time in Seconds",
    );

    handle_bench_inner(
        abspiral_grow_iter2(0.001, 0.008, 0.0001)
            .map(|grow| (grow as f32, BenchRecord::new(grow, 3000))),
        fg,
        "colfind_bench_grow",
        
        "Space partitioning algs with abspiral(grow,3000)",
        true,
        true,
        "Grow",
        "Time in Seconds",
    );

    handle_bench_inner(
        abspiral_grow_iter2(0.01, 0.2, 0.002)
            .map(|grow| (grow as f32, BenchRecord::new(grow, 3000))),
        fg,"colfind_bench_grow_wide",
        
        "Space partitioning algs with abspiral(grow,6000)",
        false,
        false,
        "Grow",
        "Time in Seconds",
    );
}
