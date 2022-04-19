use super::*;

pub fn handle_bench_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> f64 {
    let mut bots = distribute(grow, bot_inner, |a| a.to_f64n());

    bench_closure(|| {
        let mut k = TreeBuilder::new_default(&mut bots);
        k.num_level = height;
        let mut tree = k.build();

        assert_eq!(tree.num_levels(), height);

        tree.colliding_pairs(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
    })
}

pub fn handle_theory_inner(grow: f64, bot_inner: &mut [isize], height: usize) -> usize {
    datanum::datanum_test(|maker| {
        let mut bots = distribute(grow, bot_inner, |a| a.to_isize_dnum(maker));

        let mut k = TreeBuilder::new_default(&mut bots);
        k.num_level = height;
        let mut tree = k.build();

        assert_eq!(tree.num_levels(), height);

        tree.colliding_pairs(|a, b| {
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

        let grow = DEFAULT_GROW;
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
        for num_bots in its {
            let height = broccoli::tree::num_level::default(num_bots);
            vec.push((num_bots, height));
        }
        vec
    };

    let s = format!(
        "Bench of optimal vs heuristic with abspiral(x,{})",
        DEFAULT_GROW
    );

    let data = plots!(
        poloto::build::scatter(
            "Optimal",
            benches.iter().map(|a| [a.num_bots as f64, a.height as f64]),
        ),
        poloto::build::scatter("Heuristic", heur.iter().map(|a| [a.0 as f64, a.1 as f64]))
    );

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        data.chain(poloto::build::markers([], [0.0])),
        &s,
        "Number of Elements",
        "Tree Height"
    );

    fb.finish_plot(
        poloto::disp(|w| plot.render(w)),
        "height_heuristic_vs_optimal",
    );
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
    let grow = DEFAULT_GROW;
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

    let s = format!(
        "Complexity of differing num elem per node with abspiral(10000,{})",
        DEFAULT_GROW
    );

    let data = poloto::build::histogram(
        "",
        theory_records
            .iter()
            .map(|a| [a.height as f64, a.num_comparison as f64]),
    );

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        data.chain(poloto::build::markers([], [0.0])),
        &s,
        "Tree Height",
        "Number of Comparisons"
    );

    fb.finish_plot(poloto::disp(|w| plot.render(w)), "height_heuristic_theory");

    let s = format!(
        "Bench of differing num elem per node with abspiral(10000,{})",
        DEFAULT_GROW
    );

    let data = poloto::build::scatter("", bench_records.iter().map(|a| [a.height as f64, a.bench]));

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        data.chain(poloto::build::markers([], [0.0])),
        &s,
        "Tree Height",
        "Number of Comparisons"
    );

    fb.finish_plot(poloto::disp(|w| plot.render(w)), "height_heuristic_bench");
}
