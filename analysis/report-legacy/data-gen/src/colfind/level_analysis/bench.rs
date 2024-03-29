use broccoli::queries::colfind::QueryArgs;

use super::*;

struct Res {
    rebal: Vec<f64>,
    query: Vec<f64>,
}
impl Res {
    fn new(num_bots: usize, grow_iter: impl Iterator<Item = f64>) -> Vec<(f64, Res)> {
        let mut rects = Vec::new();
        for grow in grow_iter {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

            let len = bots.len();
            let (mut tree, times1) = Tree::from_build_args(
                &mut bots,
                BuildArgs::new(len).with_splitter(LevelTimer::new(0, vec![])),
            );

            let c1 = times1.into_levels().into_iter().map(|x| x as f64).collect();

            let times2 = tree.find_colliding_pairs_from_args(
                QueryArgs::new().with_splitter(LevelTimer::new(0, vec![])),
                |a, b| {
                    **a.unpack_inner() += 1;
                    **b.unpack_inner() += 1;
                },
            );

            let c2 = times2.into_levels().into_iter().map(|x| x as f64).collect();

            let t = Res {
                rebal: c1,
                query: c2,
            };

            assert_eq!(t.rebal.len(), t.query.len());

            assert_eq!(t.rebal.len(), t.query.len());
            rects.push((grow as f64, t))
        }
        rects
    }
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    let num_bots = 5000;

    let res2 = Res::new(num_bots, grow_iter(DENSE_GROW, DEFAULT_GROW));

    fn draw_graph<'a, I: Iterator<Item = (f64, &'a [f64])> + Clone>(
        filename: &str,
        title_name: &str,
        fb: &mut FigureBuilder,
        mut it: I,
    ) {
        let mut data = vec![];

        if let Some((_, xrest)) = it.next() {
            let num = xrest.len();

            let cc = (0..num).map(|ii: usize| it.clone().map(move |(x, a)| [x, a[ii]]));

            for (i, y) in cc.enumerate() {
                data.push(poloto::build::line_fill(formatm!("Level {}", i), y));
            }
        }

        let canvas = fb.canvas().build();
        let plot = poloto::simple_fmt!(
            canvas,
            poloto::build::plots_dyn(data).chain(poloto::build::markers([], [0.0])),
            title_name,
            "Spiral Grow",
            "Time taken in Seconds"
        );

        fb.finish_plot(poloto::disp(|w| plot.render(w)), filename);
    }

    draw_graph(
        "level_analysis_bench_rebal",
        &format!("Bench of rebal levels with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x| (x.0, x.1.rebal.as_slice())),
    );

    draw_graph(
        "level_analysis_bench_query",
        &format!("Bench of query levels with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x| (x.0, x.1.query.as_slice())),
    );
}
