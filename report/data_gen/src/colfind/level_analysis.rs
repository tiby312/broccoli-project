use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;
#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<i32>,
    num: usize,
}

mod level_counter {
    use crate::datanum;
    use broccoli::analyze::Splitter;

    pub struct LevelCounter {
        counter: *mut datanum::Counter,
        cursor: Option<usize>,
        levels: Vec<usize>,
    }

    impl LevelCounter {
        pub fn new(counter: *mut datanum::Counter) -> LevelCounter {
            LevelCounter {
                counter,
                levels: Vec::new(),
                cursor: None,
            }
        }
        pub fn into_inner(self) -> Vec<usize> {
            self.levels
        }
        fn node_end_common(&mut self) {
            let counter = unsafe { &mut *self.counter };
            let nc = *counter.get_inner();

            let elapsed = nc - self.cursor.unwrap();
            self.levels.push(elapsed);
            self.cursor = None;
        }
    }
    impl Splitter for LevelCounter {
        fn div(&mut self) -> Self {
            self.node_end_common();

            let length = self.levels.len();
            let counter = self.counter;
            LevelCounter {
                counter,
                levels: std::iter::repeat(0).take(length).collect(),
                cursor: None,
            }
        }
        fn add(&mut self, a: Self) {
            let len = self.levels.len();
            for (a, b) in self.levels.iter_mut().zip(a.levels.iter()) {
                *a += *b;
            }
            if len < a.levels.len() {
                self.levels.extend_from_slice(&a.levels[len..]);
            }
        }
        fn node_start(&mut self) {
            let counter = unsafe { &mut *self.counter };
            self.cursor = Some(*counter.get_inner());
        }
        fn node_end(&mut self) {
            self.node_end_common();
        }
    }
}

struct TheoryRes {
    grow: f32,
    rebal: Vec<usize>,
    query: Vec<usize>,
}

fn handle_inner_theory(num_bots: usize, grow_iter: impl Iterator<Item = f32>) -> Vec<TheoryRes> {
    let mut rects = Vec::new();
    for grow in grow_iter {
        let mut scene = bot::BotSceneBuilder::new(num_bots)
            .with_grow(grow)
            .build_specialized(|_, pos| Bot {
                num: 0,
                pos: pos.inner_as(),
            });

        let bots = &mut scene.bots;
        let prop = &scene.bot_prop;

        {
            let mut counter = datanum::Counter::new();

            let mut levelc = level_counter::LevelCounter::new(&mut counter);

            let mut bb = bbox_helper::create_bbox_mut(bots, |b| {
                datanum::from_rect(&mut counter, prop.create_bbox_i32(b.pos))
            });

            let mut tree = TreeBuilder::new(&mut bb).build_with_splitter_seq(&mut levelc);

            counter.reset();
            let mut levelc2 = level_counter::LevelCounter::new(&mut counter);
            tree.new_colfind_builder().query_with_splitter_seq(
                |mut a, mut b| {
                    a.inner_mut().num += 1;
                    b.inner_mut().num += 1;
                },
                &mut levelc2,
            );

            counter.into_inner();

            let mut t = TheoryRes {
                grow,
                rebal: levelc.into_inner(),
                query: levelc2.into_inner(),
            };
            let height = tree.get_height();

            grow_to_fit(&mut t.rebal, height);
            grow_to_fit(&mut t.query, height);

            assert_eq!(t.rebal.len(), t.query.len());
            rects.push(t)
        }
    }
    rects
}
struct BenchRes {
    grow: f32,
    rebal: Vec<f64>,
    query: Vec<f64>,
}

fn handle_inner_bench(num_bots: usize, grow_iter: impl Iterator<Item = f32>) -> Vec<BenchRes> {
    let mut rects = Vec::new();
    for grow in grow_iter {
        let mut scene = bot::BotSceneBuilder::new(num_bots)
            .with_grow(grow)
            .build_specialized(|_, pos| Bot {
                num: 0,
                pos: pos.inner_as(),
            });

        let bots = &mut scene.bots;
        let prop = &scene.bot_prop;
        let mut times1 = LevelTimer::new();

        let mut bb = bbox_helper::create_bbox_mut(bots, |b| prop.create_bbox_i32(b.pos));

        let mut tree = TreeBuilder::new(&mut bb).build_with_splitter_seq(&mut times1);

        let mut times2 = LevelTimer::new();

        tree.new_colfind_builder().query_with_splitter_seq(
            |mut a, mut b| {
                a.inner_mut().num += 1;
                b.inner_mut().num += 1
            },
            &mut times2,
        );

        let mut t = BenchRes {
            grow,
            rebal: times1.into_inner(),
            query: times2.into_inner(),
        };
        let height = tree.get_height();

        grow_to_fit(&mut t.rebal, height);
        grow_to_fit(&mut t.query, height);

        assert_eq!(t.rebal.len(), t.query.len());
        rects.push(t)
    }
    rects
}

fn grow_to_fit<T: Default>(a: &mut Vec<T>, b: usize) {
    let diff = b - a.len();
    for _ in 0..diff {
        a.push(std::default::Default::default());
    }
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    let num_bots = 10000;

    let res1 = handle_inner_bench(
        num_bots,
        (0..1000).map(|a| {
            let a: f32 = a as f32;
            0.0005 + a * 0.00001
        }),
    );

    let res2 = handle_inner_bench(
        num_bots,
        (0..1000).map(|a| {
            let a: f32 = a as f32;
            0.01 + a * 0.00002
        }),
    );

    fn draw_graph(title_name: &str, fg: &mut Figure, res: &[BenchRes], rebal: bool, pos: usize) {
        let ax = fg
            .axes2d()
            .set_pos_grid(2, 1, pos as u32)
            .set_title(title_name, &[])
            .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
            .set_x_label("Spiral Grow", &[])
            .set_y_label("Time taken in Seconds", &[]);

        let num = res.first().unwrap().rebal.len();

        let x = res.iter().map(|a| a.grow);

        if rebal {
            let cc = (0..num)
                .step_by(2)
                .map(|ii: usize| res.iter().map(move |a| a.rebal[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", 2 * i);
                ax.lines(x.clone(), y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        } else {
            let cc = (0..num)
                .step_by(2)
                .map(|ii: usize| res.iter().map(move |a| a.query[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", 2 * i);
                ax.lines(x.clone(), y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        }
    }

    let mut fg = fb.build("level_analysis_bench_rebal");
    draw_graph(
        &format!("Rebal Level Bench with abspiral({},x)", num_bots),
        &mut fg,
        &res1,
        true,
        0,
    );
    draw_graph(
        &format!("Rebal Level Bench with abspiral({},x)", num_bots),
        &mut fg,
        &res2,
        true,
        1,
    );
    fb.finish(fg);

    let mut fg = fb.build("level_analysis_bench_query");
    draw_graph(
        &format!("Query Level Bench with abspiral({},x)", num_bots),
        &mut fg,
        &res1,
        false,
        0,
    );
    draw_graph(
        &format!("Query Level Bench with abspiral({},x)", num_bots),
        &mut fg,
        &res2,
        false,
        1,
    );
    fb.finish(fg);
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    let num_bots = 3000;

    let res1 = handle_inner_theory(
        num_bots,
        (0..100).map(|a| {
            let a: f32 = a as f32;
            0.0005 + a * 0.0001
        }),
    );

    let res2 = handle_inner_theory(
        num_bots,
        (0..100).map(|a| {
            let a: f32 = a as f32;
            0.01 + a * 0.0002
        }),
    );

    use gnuplot::*;

    fn draw_graph(title_name: &str, fg: &mut Figure, res: &[TheoryRes], rebal: bool, pos: usize) {
        let ax = fg
            .axes2d()
            .set_pos_grid(2, 1, pos as u32)
            .set_title(title_name, &[])
            .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
            .set_x_label("Spiral Grow", &[])
            .set_y_label("Number of Comparisons", &[]);

        let num = res.first().unwrap().rebal.len();

        let x = res.iter().map(|a| a.grow);

        if rebal {
            let cc = (0..num).map(|ii: usize| res.iter().map(move |a| a.rebal[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", i);
                ax.lines(x.clone(), y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        } else {
            let cc = (0..num).map(|ii: usize| res.iter().map(move |a| a.query[ii]));

            for (i, (col, y)) in COLS.iter().cycle().zip(cc).enumerate() {
                let s = format!("Level {}", i);
                ax.lines(x.clone(), y, &[Color(col), Caption(&s), LineWidth(1.0)]);
            }
        }
    }

    let mut fg = fb.build("level_analysis_theory_rebal");
    draw_graph(
        &format!("Rebal Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res1,
        true,
        0,
    );
    draw_graph(
        &format!("Rebal Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res2,
        true,
        1,
    );
    fb.finish(fg);

    let mut fg = fb.build("level_analysis_theory_query");
    draw_graph(
        &format!("Query Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res1,
        false,
        0,
    );
    draw_graph(
        &format!("Query Level Comparisons with {} Objects", num_bots),
        &mut fg,
        &res2,
        false,
        1,
    );
    fb.finish(fg);
}
