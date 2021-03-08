use crate::inner_prelude::*;

pub fn handle_bench_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> f64 {
    let mut bots = distribute(grow, bot_inner, |a| a.to_f64n());

    bench_closure(|| {
        let mut tree = TreeBuilder::new(&mut bots).with_height(height).build_seq();
        assert_eq!(tree.get_height(), height);

        tree.find_colliding_pairs_mut(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
    })
}

pub fn handle_theory_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> usize {
    datanum::datanum_test(|maker| {
        let mut bots = distribute(grow, bot_inner, |a| a.to_isize_dnum(maker));

        let mut tree = TreeBuilder::new(&mut bots).with_height(height).build_seq();
        assert_eq!(tree.get_height(), height);

        tree.find_colliding_pairs_mut(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
    })
}

pub fn handle(fb: &mut FigureBuilder) {
    handle2d(fb);
    handle_lowest(fb);
}

fn handle_lowest(fb: &mut FigureBuilder) {
    struct BenchRecord {
        height: usize,
        num_bots: usize,
    }

    let mut benches: Vec<BenchRecord> = Vec::new();

    let its = (1usize..50_000).step_by(2000);
    for num_bots in its.clone() {
        let mut minimum = None;
        let max_height = (num_bots as f64).log2() as usize;

        let grow = 0.2;
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        for height in 1..max_height {
            let bench = handle_bench_inner(grow, &mut bot_inner, height);
            match minimum {
                Some((a, _b)) => {
                    if bench < a {
                        minimum = Some((bench, height));
                    }
                }
                None => {
                    minimum = Some((bench, height));
                }
            }
        }

        if let Some((_, height)) = minimum {
            benches.push(BenchRecord { height, num_bots });
        }
    }

    let heur = {
        let mut vec = Vec::new();
        for num_bots in its.clone() {
            let height = TreePreBuilder::new(num_bots).get_height();
            vec.push((num_bots, height));
        }
        vec
    };

    let mut plot = fb.plot("height_heuristic_vs_optimal");

    plot.scatter(
        wr!("Optimal"),
        benches.iter().map(|a| [a.num_bots as f64, a.height as f64]).twice_iter(),
    );

    plot.scatter(
        wr!("Heuristic"),
        heur.iter().map(|a| [a.0 as f64, a.1 as f64]).twice_iter(),
    );

    plot.render(
        wr!("Bench of optimal vs heuristic with abspiral(x,0.2)"),
        wr!("Number of Elements"),
        wr!("Tree Height"),
    )
    .unwrap();
}

fn handle2d(fb: &mut FigureBuilder) {
    #[derive(Debug)]
    struct Record {
        height: usize,
        num_comparison: usize,
    }

    #[derive(Debug)]
    struct BenchRecord {
        height: usize,
        bench: f64,
    }

    let mut theory_records = Vec::new();

    let mut bench_records: Vec<BenchRecord> = Vec::new();
    let num_bots = 10000;
    let grow = 0.2;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    for height in 2..13 {
        let num_comparison = handle_theory_inner(grow, &mut bot_inner, height);
        theory_records.push(Record {
            height,
            num_comparison,
        });
    }

    for height in (2..13).flat_map(|a| std::iter::repeat(a).take(20)) {
        let bench = handle_bench_inner(grow, &mut bot_inner, height);
        bench_records.push(BenchRecord { height, bench });
    }

    let mut plot = fb.plot("height_heuristic_theory");

    plot.histogram(
        wr!("brocc"),
        theory_records
            .iter()
            .map(|a| [a.height as f64, a.num_comparison as f64]).twice_iter(),
    );

    plot.render(
        wr!("Complexity of differing num elem per node with abspiral(10000,0.2)"),
        wr!("Tree Height"),
        wr!("Number of Comparisons"),
    )
    .unwrap();

    let mut plot = fb.plot("height_heuristic_bench");

    plot.scatter(
        wr!("brocc"),
        bench_records
            .iter()
            .map(|a| [a.height as f64, a.bench as f64]).twice_iter(),
    );

    plot.render(
        wr!("Bench of differing num elem per node with abspiral(10000,0.2)"),
        wr!("Tree Height"),
        wr!("Number of Comparisons"),
    )
    .unwrap();
}
