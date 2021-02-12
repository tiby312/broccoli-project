use super::*;

#[derive(Debug, Serialize)]
struct Record {
    broccoli: f32,
    naive: f32,
    sweep: f32,
    nosort: f32,
}

impl Record {
    fn new(grow: f64, num_bots: usize,naive:bool,sweep:bool) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let c1 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        });

        let c2 = if naive {
            datanum::datanum_test(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                    **a.unpack_inner() -= 1;
                    **b.unpack_inner() -= 1;
                });
            })
        } else {
            0
        };

        let c3 = if sweep {
            datanum::datanum_test(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                    **a.unpack_inner() -= 3;
                    **b.unpack_inner() -= 3;
                });
            })
        } else {
            0
        };

        let c4 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let mut tree = NotSorted::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        });

        if naive && sweep {
            for (i, &a) in bot_inner.iter().enumerate() {
                assert_eq!(a, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
            }
        }

        Record {
            broccoli: c1 as f32,
            naive: c2 as f32,
            sweep: c3 as f32,
            nosort: c4 as f32,
        }
    }
}

pub fn handle_theory(fb: &mut FigureBuilder) {

    const THEORY_STOP_NAIVE_AT: usize = 8_000;
    const THEORY_STOP_SWEEP_AT: usize = 50_000;

    fb.make_graph(Args {
        filename: "colfind_theory_0.2",
        title: "Complexity of space partitioning algs with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0,80_000)
            .map(|num_bots| (num_bots as f32, Record::new(0.2, num_bots,num_bots<=THEORY_STOP_NAIVE_AT,num_bots<=THEORY_STOP_SWEEP_AT))),
        stop_values: &[
            ("naive", THEORY_STOP_NAIVE_AT as f32),
            ("sweep", THEORY_STOP_SWEEP_AT as f32),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_theory_0.05",
        title: "Complexity of space partitioning algs with abspiral(x,0.05)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0,80_000)
            .map(|num_bots| (num_bots as f32, Record::new(0.05, num_bots,num_bots<=THEORY_STOP_NAIVE_AT,num_bots<=THEORY_STOP_SWEEP_AT))),
        stop_values: &[
            ("naive", THEORY_STOP_NAIVE_AT as f32),
            ("sweep", THEORY_STOP_SWEEP_AT as f32),
        ],
    });


    fb.make_graph(Args {
        filename: "colfind_theory_grow",
        title: "Complexity of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots: grow_iter(0.0,0.02)
        .map(|grow| (grow as f32, Record::new(grow, 3000,true,true))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "colfind_theory_grow_wide",
        title: "Complexity of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots:grow_iter(0.2,4.0).map(|x|x)
        .map(|grow| (grow as f32, Record::new(grow, 3000,false,true))),
        stop_values: &[],
    });

}
