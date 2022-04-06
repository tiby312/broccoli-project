use super::*;

pub fn handle(fb: &mut FigureBuilder) {
    handle_optimal(fb);
    handle_broccoli(fb);
}

pub fn handle_broccoli(fb: &mut FigureBuilder) {
    #[derive(Serialize)]
    struct Res {
        bench: f64,
        collect: f64,
    }
    impl Res {
        fn new(grow: f64, num_bots: usize) -> Res {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let bench = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f64n());
                let mut tree = broccoli::tree::new(&mut base);

                bench_closure(|| {
                    tree.colliding_pairs(|a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                    });
                })
            };

            let collect = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f64n());
                let mut tree = broccoli::tree::new(&mut base);

                bench_closure(|| {
                    let mut tree=broccoli_ext::cachable_pairs::IndTree(&mut tree);
                    let mut cacher = broccoli_ext::cachable_pairs::Cacheable::new(&mut tree);
                    let pairs=cacher.cache_colliding_pairs(|a, b| {
                        *a += 1;
                        *b += 1;
                        Some(())
                    });
                   
                    black_box(pairs);
                })
            };

            black_box(bot_inner);

            Res { bench, collect }
        }
    }

    fb.make_graph(Args {
        filename: "broccoli_query",
        title: &format!(
            "Bench of query vs collect with abspiral({},n)",
            DEFAULT_GROW
        ),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0, 40_000)
            .map(|num_bots| (num_bots as f64, Res::new(DEFAULT_GROW, num_bots))),
        stop_values: &[],
    });
}

pub fn handle_optimal(fb: &mut FigureBuilder) {
    #[derive(Serialize)]
    struct Res {
        cached_pairs: f64,
        non_cached_pairs: f64,
    }
    impl Res {
        fn new(grow: f64, num_bots: usize) -> Res {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let cached_pairs = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f64n());
                let mut tree = broccoli::tree::new(&mut base);

                bench_closure(|| {
                    let mut tree=broccoli_ext::cachable_pairs::IndTree(&mut tree);
                    let mut cacher = broccoli_ext::cachable_pairs::Cacheable::new(&mut tree);
                    let mut pairs=cacher.cache_colliding_pairs(|_, _| {Some(())});
                    for _ in 0..10 {
                        pairs.colliding_pairs(&mut cacher,|a, b, ()| {
                            *a += 1;
                            *b += 1;
                        });
                    }
                })
            };

            let non_cached_pairs = {
                let mut base =
                    crate::support::make_tree_ref_ind(&mut bot_inner, grow, |a| a.to_f64n());
                let mut tree = broccoli::tree::new(&mut base);

                bench_closure(|| {
                    for _ in 0..10 {
                        tree.colliding_pairs(|a, b| {
                            **a.unpack_inner() += 1;
                            **b.unpack_inner() += 1;
                        });
                    }
                })
            };

            black_box(bot_inner);

            Res {
                cached_pairs,
                non_cached_pairs,
            }
        }
    }

    fb.make_graph(Args {
        filename: "optimal_query",
        title: &format!(
            "Bench of cached pairs 10 iter with abspiral({},n)",
            DEFAULT_GROW
        ),
        xname: "Number of Elements",
        yname: "Time in Seconds",
        plots: n_iter(0, 40_000)
            .map(|num_bots| (num_bots as f64, Res::new(DEFAULT_GROW, num_bots))),
        stop_values: &[],
    });
}
