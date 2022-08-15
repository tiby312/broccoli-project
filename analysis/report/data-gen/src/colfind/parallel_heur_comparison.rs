use broccoli::queries::colfind::QueryArgs;

use super::*;

fn test1(bots: &mut [(Rect<f64>, &mut isize)]) -> (f64, f64) {
    let (mut tree, construction_time) = bench_closure_ret(|| broccoli::Tree::new(bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.find_colliding_pairs(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

fn test3(
    bots: &mut [(Rect<f64>, &mut isize)],
    rebal_num: Option<usize>,
    query_num: Option<usize>,
) -> (f64, f64) {
    let (mut tree, construction_time) = bench_closure_ret(|| {
        let mut k = BuildArgs::new(bots.len());
        if let Some(r) = rebal_num {
            k.num_seq_fallback = r;
        }
        Tree::par_from_build_args(bots, k).0
    });

    let (tree, query_time) = bench_closure_ret(|| {
        {
            let mut f = QueryArgs::new();

            if let Some(r) = query_num {
                f.num_seq_fallback = r;
            }

            tree.par_find_colliding_pairs_from_args(f, |a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        }
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

pub fn handle(fb: &mut FigureBuilder) {
    let num_bots = 20_000;

    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let height = tree::num_level::default(num_bots);
    let mut rebals = Vec::new();

    let range = (000..20_000).step_by(100);

    for rebal_num in range.clone() {
        let (a, _b) = test3(
            &mut distribute(DEFAULT_GROW, &mut bot_inner, |a| a.to_f64n()),
            Some(rebal_num),
            None,
        );
        rebals.push((rebal_num as f64, a as f64));
    }
    let mut queries = Vec::new();
    for query_num in range.clone() {
        let (_a, b) = test3(
            &mut distribute(DEFAULT_GROW, &mut bot_inner, |a| a.to_f64n()),
            None,
            Some(query_num),
        );
        queries.push((query_num as f64, b as f64));
    }

    let mut seqs = Vec::new();
    for _ in 0..100 {
        let (a, b) = test1(&mut distribute(DEFAULT_GROW, &mut bot_inner, |a| {
            a.to_f64n()
        }));
        seqs.push((a as f64, b as f64));
    }

    let s = format!(
        "Bench of differing parallel switch levels with abspiral(20,000,{})",
        DEFAULT_GROW
    );

    let data = plots!(
        poloto::build::scatter("Rebal Par", rebals.iter().map(|a| [a.0, a.1])),
        poloto::build::scatter("Query Par", queries.iter().map(|a| [a.0, a.1])),
        poloto::build::scatter("Rebal", seqs.iter().map(|a| [height as f64, a.0])),
        poloto::build::scatter("Query", seqs.iter().map(|a| [height as f64, a.1]))
    );

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        data.chain(poloto::build::markers([], [0.0])),
        &s,
        "problem size at which to switch to sequential",
        "Time in Seconds"
    );

    fb.finish_plot(
        poloto::disp(|w| plot.render(w)),
        "parallel_height_heuristic",
    );
}
