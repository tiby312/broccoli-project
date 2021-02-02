use crate::inner_prelude::*;
use broccoli::pmut::PMut;
use broccoli::query::colfind::NotSortedQueries;
fn handle_bench_inner(grow: f64, fg: &mut FigureBuilder, title: &str,filename:&str) {
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

    //TODO convert to milliseconds
    let mut plot=splot::plot(title,"Number of Elements","Time in Seconds");

    plot.lines("naive",rects.iter().map(|a| [a.num_bots,a.bench_naive]).take_while(|&[x,_]|x<=stop_naive_at as f32));
    plot.lines("sweep",rects.iter().map(|a| [a.num_bots,a.bench_sweep]).take_while(|&[x,_]|x<=stop_sweep_at as f32));
    plot.lines("nosort par",rects.iter().map(|a| [a.num_bots,a.bench_nosort_par]));
    plot.lines("nosort seq",rects.iter().map(|a| [a.num_bots,a.bench_nosort_seq]));
    plot.lines("broccoli par",rects.iter().map(|a| [a.num_bots,a.bench_par]));
    plot.lines("broccoli seq",rects.iter().map(|a| [a.num_bots,a.bench_alg]));
    

    plot.render_to_file(&fg.get_folder_path(filename)).unwrap();

}

fn handle_theory_inner(grow: f64, fg: &mut FigureBuilder, title: &str, filename:&str) {
    #[derive(Debug)]
    struct Record {
        num_bots: f32,
        num_comparison_alg: f32,
        num_comparison_naive: f32,
        num_comparison_sweep: f32,
        num_comparison_nosort: f32,
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

            let c2 = if num_bots <= stop_naive_at{
                datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                    broccoli::query::colfind::query_naive_mut(PMut::new(&mut bots), |a, b| {
                        **a.unpack_inner() -= 1;
                        **b.unpack_inner() -= 1;
                    });
                })
            }else{
                0
            };

            let c3 = if num_bots <= stop_sweep_at{
                datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                    broccoli::query::colfind::query_sweep_mut(axgeom::XAXIS, &mut bots, |a, b| {
                        **a.unpack_inner() -= 3;
                        **b.unpack_inner() -= 3;
                    });
                })
            }else{
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

            if num_bots < stop_naive_at {
                for (i, &a) in bot_inner.iter().enumerate() {
                    assert_eq!(a, 0, "failed iteration:{:?} numbots={:?}", i, num_bots);
                }
            }

            Record {
                num_bots:num_bots as f32,
                num_comparison_alg: c1 as f32,
                num_comparison_naive: c2 as f32,
                num_comparison_sweep: c3 as f32,
                num_comparison_nosort: c4 as f32,
            }
        })
        .collect::<Vec<_>>();

        let mut plot=splot::plot(title,"Number of Elements","Number of Comparisons");

    plot.lines("naive",rects.iter().map(|a| [a.num_bots,a.num_comparison_naive]).take_while(|&[x,_]|x<=stop_naive_at as f32));
    plot.lines("sweep",rects.iter().map(|a| [a.num_bots,a.num_comparison_sweep]).take_while(|&[x,_]|x<=stop_sweep_at as f32));
    plot.lines("nosort",rects.iter().map(|a| [a.num_bots,a.num_comparison_nosort]));
    plot.lines("broccoli",rects.iter().map(|a| [a.num_bots,a.num_comparison_alg]));
    

    plot.render_to_file(&fg.get_folder_path(filename)).unwrap();

}

pub fn handle_theory(fb: &mut FigureBuilder) {
    
    handle_theory_inner(
        0.2,
        fb,
        "Comparison of space partitioning algs with abspiral(x,0.2)",
        "colfind_theory_0.2"
    );
    handle_theory_inner(
        0.05,
        fb,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        "colfind_theory_0.05"
    );
}

pub fn handle_bench(fg: &mut FigureBuilder) {
    handle_bench_inner(
        0.2,
        fg,
        "Comparison of space partitioning algs with abspiral(x,0.2)",
        "colfind_bench_0.2"
    );
    handle_bench_inner(
        0.05,
        fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        "colfind_bench_0.05"
    );
}
