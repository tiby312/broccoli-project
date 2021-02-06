use super::*;

#[derive(Debug, Serialize)]
struct TheoryRecord {
    broccoli: f32,
    naive: f32,
    sweep: f32,
    nosort: f32,
}

const theory_stop_naive_at: usize = 8_000;
const theory_stop_sweep_at: usize = 50_000;

impl TheoryRecord {
    fn new(grow: f64, num_bots: usize) -> TheoryRecord {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let c1 = datanum::datanum_test(|maker| {
            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        });

        let c2 = if num_bots <= theory_stop_naive_at {
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

        let c3 = if num_bots <= theory_stop_sweep_at {
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

        if num_bots < theory_stop_naive_at {
            for (i, &a) in bot_inner.iter().enumerate() {
                assert_eq!(a, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
            }
        }

        TheoryRecord {
            broccoli: c1 as f32,
            naive: c2 as f32,
            sweep: c3 as f32,
            nosort: c4 as f32,
        }
    }
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    fb.make_graph(Args {
        filename: "colfind_theory_0.2",
        title: "Comparison of space partitioning algs with abspiral(x,0.2)",
        xname: "Number of Elements",
        yname: "Number of Comparisons",
        plots: (0usize..80_000)
            .step_by(2000)
            .map(move |num_bots| (num_bots as f32, TheoryRecord::new(0.2, num_bots))),
        stop_values: &[
            ("naive", theory_stop_naive_at as f32),
            ("sweep", theory_stop_sweep_at as f32),
        ],
    });

    /*
        handle_theory_inner(
            (0usize..80_000)
                .step_by(2000)
                .map(move |num_bots| (num_bots as f32, TheoryRecord::new(0.2, num_bots))),
            fb,
            "Comparison of space partitioning algs with abspiral(x,0.2)",
            "colfind_theory_0.2",
            "Number of Elements",
            "Number of Comparisons",
        );
        handle_theory_inner(
            (0usize..80_000)
                .step_by(2000)
                .map(move |num_bots| (num_bots as f32, TheoryRecord::new(0.05, num_bots))),
            fb,
            "Comparison of space partitioning algs with abspiral(x,0.05)",
            "colfind_theory_0.05",
            "Number of Elements",
            "Number of Comparisons",
        );

        handle_theory_inner(
            abspiral_grow_iter2(0.001, 0.01, 0.0001)
                .map(|grow| (grow as f32, TheoryRecord::new(grow, 3000))),
            fb,
            "Comparison of space partitioning algs with abspiral(3000,grow)",
            "colfind_theory_grow",
            "Grow",
            "Number of Comparisons",
        );

        handle_theory_inner(
            abspiral_grow_iter2(0.01, 0.2, 0.001)
                .map(|grow| (grow as f32, TheoryRecord::new(grow, 3000))),
            fb,
            "Comparison of space partitioning algs with abspiral(3000,grow)",
            "colfind_theory_grow_wide",
            "Grow",
            "Number of Comparisons",
        );
    */
}
