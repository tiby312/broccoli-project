use super::*;

struct Res {
    rebal: Vec<f64>,
    query: Vec<f64>,
}

impl Res {
    fn new(num_bots: usize, grow_iter: impl Iterator<Item = f64>) -> Vec<(f64, Res)> {
        let mut rects = Vec::new();
        for grow in grow_iter {
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

            let (rebal, query) = datanum::datanum_test2(|maker| {
                let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32dnum(maker));

                let (mut tree, levelc) =
                    TreeBuilder::new(&mut bots).build_with_splitter_seq(LevelCounter::new());

                maker.reset();
                let levelc2 = tree.new_colfind_builder().query_with_splitter_seq(
                    |a, b| {
                        a.unpack_inner().x += 1.0;
                        b.unpack_inner().y += 1.0;
                    },
                    LevelCounter::new(),
                );

                (
                    levelc.into_levels().into_iter().map(|x| x as f64).collect(),
                    levelc2
                        .into_levels()
                        .into_iter()
                        .map(|x| x as f64)
                        .collect(),
                )
            });

            let t = Res { rebal, query };

            assert_eq!(t.rebal.len(), t.query.len());
            rects.push((grow as f64, t))
        }
        rects
    }
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

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
        let plot = canvas
            .build_with(poloto::build::plots_dyn(data), [], [0.0])
            .plot(title_name, "Spiral Grow", "Number of Comparisons");

        fb.finish_plot(poloto::disp(|w| plot.render(w)), filename);
    }

    draw_graph(
        "level_analysis_theory_rebal",
        &format!("Complexity of rebal levels with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x| (x.0, x.1.rebal.as_slice())),
    );

    draw_graph(
        "level_analysis_theory_query",
        &format!("Complexity of query levels with abspiral({},x)", num_bots),
        fb,
        res2.iter().map(|x| (x.0, x.1.query.as_slice())),
    );
}
