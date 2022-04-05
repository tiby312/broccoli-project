use super::*;

#[derive(Debug, Serialize)]
struct Record {
    broccoli: f64,
    naive: f64,
    sweep: f64,
    nosort: f64,
}

impl Record {
    fn new(grow: f64, num_bots: usize, naive: bool, sweep: bool) -> Record {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let c1 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let mut tree = broccoli::tree::new(&mut bots);
            tree.colliding_pairs(|a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        });

        let c2 = if naive {
            datanum::datanum_test(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                TreePin::new(bots.as_mut_slice()).colliding_pairs(|a, b| {
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

                broccoli::queries::colfind::SweepAndPrune::new(&mut bots).colliding_pairs(
                    |a, b| {
                        **a.unpack_inner() -= 3;
                        **b.unpack_inner() -= 3;
                    },
                );
            })
        } else {
            0
        };

        let c4 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let _tree = NoSorter.build(&mut bots).colliding_pairs(|a, b| {
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
            broccoli: c1 as f64,
            naive: c2 as f64,
            sweep: c3 as f64,
            nosort: c4 as f64,
        }
    }
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    const THEORY_STOP_NAIVE_AT: usize = 2_000;
    const THEORY_STOP_SWEEP_AT: usize = 50_000;

    fb.make_graph(Args {
        filename: "colfind_theory_default",
        title: &format!(
            "Complexity of space partitioning algs with abspiral(x,{})",
            DEFAULT_GROW
        ),
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0, 10_000).map(|num_bots| {
            (
                num_bots as f64,
                Record::new(
                    DEFAULT_GROW,
                    num_bots,
                    num_bots <= THEORY_STOP_NAIVE_AT,
                    num_bots <= THEORY_STOP_SWEEP_AT,
                ),
            )
        }),
        stop_values: &[
            ("naive", THEORY_STOP_NAIVE_AT as f64),
            ("sweep", THEORY_STOP_SWEEP_AT as f64),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_theory_dense",
        title: &format!(
            "Complexity of space partitioning algs with abspiral(x,{})",
            DENSE_GROW
        ),
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: n_iter(0, 10_000).map(|num_bots| {
            (
                num_bots as f64,
                Record::new(
                    DENSE_GROW,
                    num_bots,
                    num_bots <= THEORY_STOP_NAIVE_AT,
                    num_bots <= THEORY_STOP_SWEEP_AT,
                ),
            )
        }),
        stop_values: &[
            ("naive", THEORY_STOP_NAIVE_AT as f64),
            ("sweep", THEORY_STOP_SWEEP_AT as f64),
        ],
    });

    fb.make_graph(Args {
        filename: "colfind_theory_grow",
        title: "Complexity of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots: grow_iter(MEGA_MEGA_DENSE_GROW, MEGA_DENSE_GROW)
            .map(|grow| (grow as f64, Record::new(grow, 3000, true, true))),
        stop_values: &[],
    });

    fb.make_graph(Args {
        filename: "colfind_theory_grow_wide",
        title: "Complexity of space partitioning algs with abspiral(3000,grow)",
        xname: "Grow",
        yname: "Number of Comparisons",
        plots: grow_iter(MEGA_DENSE_GROW, DENSE_GROW)
            .map(|x| x)
            .map(|grow| (grow as f64, Record::new(grow, 3000, false, true))),
        stop_values: &[],
    });
}
