use crate::inner_prelude::*;
use broccoli::pmut::PMut;
use broccoli::query::colfind::NotSortedQueries;
fn handle_bench_inner(grow: f64, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: f32,
        bench_alg: f32,
        bench_par: f32,
        bench_sweep: f32,
        bench_naive: f32,
        bench_nosort_par: f32,
        bench_nosort_seq: f32,
    }

    let stop_naive_at = 3000;
    let stop_sweep_at = 6000;

    let rects = (0..10000)
        .step_by(20)
        .map(move |num_bots| {
            dbg!(num_bots);
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let c0 =                 bench_closure(|| {
                    let mut tree = broccoli::new_par(RayonJoin, &mut bots);
                    tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                    });
                });

            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let c1 = 
                    bench_closure(|| {
                    let mut tree = broccoli::new(&mut bots);
                    tree.find_colliding_pairs_mut(|a, b| {
                        **a.unpack_inner() -= 1;
                        **b.unpack_inner() -= 1;
                    });
                });

            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let c3 = if num_bots<=stop_sweep_at{
                bench_closure(|| {
                    broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                        **a.unpack_inner() -= 2;
                        **b.unpack_inner() -= 2;
                    });
                })
            }else{
                0.0
            };

            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

            let c4 = if num_bots<=stop_naive_at {

                bench_closure(|| {
                    broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                        **a.unpack_inner() += 2;
                        **b.unpack_inner() += 2;
                    });
                })
            }else{
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

            if num_bots<=stop_naive_at{
                for (i, &b) in bot_inner.iter().enumerate(){
                    assert_eq!(b, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
                }
            }
        
            Record {
                num_bots:num_bots as f32,
                bench_alg: c1 as f32,
                bench_par: c0 as f32,
                bench_sweep: c3 as f32,
                bench_naive: c4 as f32,
                bench_nosort_par: c5 as f32,
                bench_nosort_seq: c6 as f32,
            }
        })
        .collect::<Vec<_>>();

    let mut plot=splot::plot("Collision","num bots","time in seconds");

    plot.lines("naive",rects.iter().map(|a| [a.num_bots,a.bench_naive]).take_while(|&[x,_]|x<=stop_naive_at as f32));
    plot.lines("sweep",rects.iter().map(|a| [a.num_bots,a.bench_sweep]).take_while(|&[x,_]|x<=stop_sweep_at as f32));
    plot.lines("nosort par",rects.iter().map(|a| [a.num_bots,a.bench_nosort_par]));
    plot.lines("nosort seq",rects.iter().map(|a| [a.num_bots,a.bench_nosort_seq]));
    plot.lines("broccoli par",rects.iter().map(|a| [a.num_bots,a.bench_par]));
    plot.lines("broccoli seq",rects.iter().map(|a| [a.num_bots,a.bench_alg]));
    
    plot.render_to_file("test.svg").unwrap();

    /*
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects
        .iter()
        .take_while(|a| a.bench_naive.is_some())
        .map(|a| a.bench_naive.unwrap());
    let y2 = rects
        .iter()
        .take_while(|a| a.bench_sweep.is_some())
        .map(|a| a.bench_sweep.unwrap());
    let y3 = rects.iter().map(|a| a.bench_alg);
    let y4 = rects.iter().map(|a| a.bench_par);
    let y5 = rects
        .iter()
        .take_while(|a| a.bench_nosort_par.is_some())
        .map(|a| a.bench_nosort_par.unwrap());
    let y6 = rects
        .iter()
        .take_while(|a| a.bench_nosort_seq.is_some())
        .map(|a| a.bench_nosort_seq.unwrap());

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Naive"), Color(COLS[0]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Sweep and Prune"), Color(COLS[1]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[
                Caption("broccoli Sequential"),
                Color(COLS[2]),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("broccoli Parallel"), Color(COLS[3]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("KD Tree Parallel"), Color(COLS[4]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("KD Tree Sequential"),
                Color(COLS[5]),
                LineWidth(2.0),
            ],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
    */
}

fn handle_theory_inner(grow: f64, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        num_comparison_alg: usize,
        num_comparison_naive: Option<usize>,
        num_comparison_sweep: Option<usize>,
        num_comparison_nosort: usize,
    }

    let stop_naive_at = 8_000;
    let stop_sweep_at = 50_000;

    let rects = (0usize..80_000)
        .step_by(2000)
        .map(move |num_bots| {
            dbg!(num_bots);

            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let c1 = datanum::datanum_test(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                let mut tree = broccoli::new(&mut bots);
                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 2;
                    **b.unpack_inner() += 2;
                });
            });

            let c2 = bool_then(num_bots < stop_naive_at, || {
                datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                    broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                        **a.unpack_inner() -= 1;
                        **b.unpack_inner() -= 1;
                    });
                })
            });

            let c3 = bool_then(num_bots < stop_sweep_at, || {
                datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                    broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                        **a.unpack_inner() -= 3;
                        **b.unpack_inner() -= 3;
                    });
                })
            });

            let c4 = datanum::datanum_test(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                let mut tree = NotSorted::new(&mut bots);
                tree.find_colliding_pairs_mut(|a, b| {
                    **a.unpack_inner() += 2;
                    **b.unpack_inner() += 2;
                });
            });

            if num_bots < stop_naive_at {
                for (i, &a) in bot_inner.iter().enumerate() {
                    assert_eq!(a, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
                }
            }

            Record {
                num_bots,
                num_comparison_alg: c1,
                num_comparison_naive: c2,
                num_comparison_sweep: c3,
                num_comparison_nosort: c4,
            }
        })
        .collect::<Vec<_>>();

    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.num_comparison_alg);
    let y2 = rects
        .iter()
        .take_while(|a| a.num_comparison_naive.is_some())
        .map(|a| a.num_comparison_naive.unwrap());
    let y3 = rects
        .iter()
        .take_while(|a| a.num_comparison_sweep.is_some())
        .map(|a| a.num_comparison_sweep.unwrap());
    let y4 = rects.iter().map(|a| a.num_comparison_nosort);

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y2,
            &[Caption("Naive"), Color(COLS[0]), LineWidth(4.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Sweep and Prune"), Color(COLS[1]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y1,
            &[Caption("broccoli"), Color(COLS[2]), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("KDTree"), Color(COLS[3]), LineWidth(2.0)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Number of Comparisons", &[]);
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_theory");

    handle_theory_inner(
        0.2,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.2)",
        0,
    );
    handle_theory_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );
    fb.finish(fg)
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_bench");
    handle_bench_inner(
        0.2,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.2)",
        0,
    );
    handle_bench_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );

    fb.finish(fg);
}
