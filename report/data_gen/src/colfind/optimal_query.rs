use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder) {
    handle_optimal(fb);
    handle_broccoli(fb);
}

pub fn handle_broccoli(fb: &mut FigureBuilder) {
    #[derive(Serialize)]
    struct Res {
        bench: f32,
        bench_par: f32,
        collect: f32,
        collect_par: f32,
    }
    impl Res{
        fn new(grow:f64,num_bots:usize)->Res{
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let bench = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                bench_closure(|| {
                    tree.find_colliding_pairs_mut(|a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                    });
                })
            };

            let bench_par = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                bench_closure(|| {
                    tree.find_colliding_pairs_mut_par(RayonJoin, |a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                    });
                })
            };

            let collect = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                bench_closure(|| {
                    let c = tree.collect_colliding_pairs(|a, b| {
                        *a += 1;
                        *b += 1;
                        Some(())
                    });
                    black_box(c);
                })
            };

            let collect_par = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                bench_closure(|| {
                    let c = tree.collect_colliding_pairs_par(RayonJoin, |a, b| {
                        *a += 1;
                        *b += 1;
                        Some(())
                    });
                    black_box(c);
                })
            };

            black_box(bot_inner);

            Res {
                bench:bench as f32,
                bench_par:bench_par as f32,
                collect:collect as f32,
                collect_par:collect_par as f32,
            }
        }
    }


    
    fb.make_graph(Args {
        filename: "broccoli_query",
        title: "Bench of query vs collect with abspiral(0.2,n)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0,40_000)
            .map(|num_bots| (num_bots as f32, Res::new(0.2, num_bots))),
        stop_values: &[],
    });

    
}

pub fn handle_optimal(fb: &mut FigureBuilder) {
    #[derive(Serialize)]
    struct Res {
        optimal: f32,
        optimal_par: f32,
    }
    impl Res{
        fn new(grow:f64,num_bots:usize)->Res{
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let optimal = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                let mut pairs = tree.collect_colliding_pairs(|_, _| Some(()));

                bench_closure(|| {
                    pairs.for_every_pair_mut(&mut bot_inner, |a, b, ()| {
                        *a += 1;
                        *b += 1;
                    });
                })
            };

            let optimal_par = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f32n());
                let mut tree = base.build();

                let mut pairs = tree.collect_colliding_pairs_par(RayonJoin, |_, _| Some(()));

                bench_closure(|| {
                    pairs.for_every_pair_mut_par(RayonJoin, &mut bot_inner, |a, b, ()| {
                        *a += 1;
                        *b += 1;
                    });
                })
            };

            black_box(bot_inner);

            Res {
                optimal:optimal as f32,
                optimal_par:optimal_par as f32,
            }
        }
    }


    
    fb.make_graph(Args {
        filename: "optimal_query",
        title: "Bench of optimal with abspiral(0.2,n)",
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0,40_000)
            .map(|num_bots| (num_bots as f32, Res::new(0.2, num_bots))),
        stop_values: &[],
    });

}
