use crate::inner_prelude::*;

pub mod all {
    use super::*;
    #[derive(Debug)]
    pub struct RecordBench {
        pub grow: f64,
        pub num_bots: usize,
        pub bench: Option<(f64, f64)>,
        pub bench_par: Option<(f64, f64)>,
        pub nosort: Option<(f64, f64)>,
        pub nosort_par: Option<(f64, f64)>,
    }
    pub struct RecordTheory {
        pub grow: f64,
        pub num_bots: usize,
        pub theory: (usize, usize),
        pub nosort_theory: (usize, usize),
    }

    fn repel(p1: Vec2<f32>, p2: Vec2<f32>, res1: &mut Vec2<f32>, res2: &mut Vec2<f32>) {
        let offset = p2 - p1;
        let dis = (offset).magnitude();
        if dis < ABSPIRAL_PROP.radius.dis() {
            *res1 += offset * 0.0001;
            *res2 -= offset * 0.0001;
        }
    }

    pub fn handle_bench(num_bots: usize, grow: f64, do_all: bool) -> RecordBench {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

        let bench = Some({
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_f32_nan(grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();

            let (mut tree, t1) = bench_closure_ret(|| broccoli::new(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_into();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_into();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1, t2)
        });

        let bench_par = Some({
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_f32_nan(grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();

            let (mut tree, t1) = bench_closure_ret(|| broccoli::new_par(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut_par(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_into();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_into();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1, t2)
        });

        let nosort = bool_then(do_all || num_bots < 2000, || {
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_f32_nan(grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();

            let (mut tree, t1) = bench_closure_ret(|| NotSorted::new(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_into();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_into();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1, t2)
        });

        let nosort_par = bool_then(do_all || num_bots < 2500, || {
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_f32_nan(grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();

            let (mut tree, t1) = bench_closure_ret(|| NotSorted::new_par(&mut bots));
            let t2 = bench_closure(|| {
                tree.find_colliding_pairs_mut_par(|a, b| {
                    let aa = vec2(a.get().x.start, a.get().y.start).inner_into();
                    let bb = vec2(b.get().x.start, b.get().y.start).inner_into();
                    repel(aa, bb, a.unpack_inner(), b.unpack_inner());
                });
            });
            (t1, t2)
        });

        RecordBench {
            grow,
            num_bots,
            bench,
            bench_par,
            nosort,
            nosort_par,
        }
    }
    pub fn handle_theory(num_bots: usize, grow: f64) -> RecordTheory {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| vec2same(0.0f32)).collect();

        let theory = datanum::datanum_test2(|maker| {
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_datanum_f32_nan(maker, grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();
            let mut tree = broccoli::new(&mut bots);

            let count = maker.count();

            tree.find_colliding_pairs_mut(|a, b| {
                let aa = vec2(a.get().x.start.0, a.get().y.start.0).inner_into();
                let bb = vec2(b.get().x.start.0, b.get().y.start.0).inner_into();
                repel(aa, bb, a.unpack_inner(), b.unpack_inner());
            });

            let count2 = maker.count();
            (count, count2)
        });

        let nosort_theory = datanum::datanum_test2(|maker| {
            let mut bots: Vec<BBox<_, &mut _>> = abspiral_datanum_f32_nan(maker, grow as f64)
                .zip(bot_inner.iter_mut())
                .map(|(a, b)| bbox(a, b))
                .collect();
            let mut tree = NotSorted::new(&mut bots);

            let count = maker.count();

            tree.find_colliding_pairs_mut(|a, b| {
                let aa = vec2(a.get().x.start.0, a.get().y.start.0).inner_into();
                let bb = vec2(b.get().x.start.0, b.get().y.start.0).inner_into();
                repel(aa, bb, a.unpack_inner(), b.unpack_inner());
            });

            let count2 = maker.count();
            (count, count2)
        });
        RecordTheory {
            grow,
            num_bots,
            nosort_theory,
            theory,
        }
    }
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    handle_grow_bench(fb);
    handle_num_bots_bench(fb);
}
pub fn handle_theory(fb: &mut FigureBuilder) {
    handle_grow_theory(fb);
    handle_num_bots_theory(fb);
}

fn handle_num_bots_theory(fb: &mut FigureBuilder) {
    let mut fg = fb.build("construction_vs_query_num_theory");
    handle_num_bots_theory_inner(&mut fg, 0.2, 0);
    handle_num_bots_theory_inner(&mut fg, 2.0, 1);
    fb.finish(fg);
}

fn handle_num_bots_theory_inner(fg: &mut Figure, grow: f64, counter: u32) {
    let mut rects = Vec::new();

    for num_bots in (1..80_000).step_by(1000) {
        rects.push(all::handle_theory(num_bots, grow));
    }

    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.theory.0);
    let y2 = rects.iter().map(|a| a.theory.1);

    fg.axes2d()
        .set_pos_grid(2, 1, counter)
        .set_title(
            &format!("Rebal vs Query Comparisons with a abspiral(n,{})", grow),
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebalance"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query"), Color(COLS[1]), LineWidth(1.0)],
        )
        .set_x_label("Number of Elements", &[])
        .set_y_label("Number of Comparisons", &[]);
}

fn handle_num_bots_bench(fb: &mut FigureBuilder) {
    let mut fg = fb.build(&format!("construction_vs_query_num_bench"));

    handle_num_bots_bench_inner(&mut fg, 0.2, 0);
    handle_num_bots_bench_inner(&mut fg, 2.0, 1);

    fb.finish(fg);
}

fn handle_num_bots_bench_inner(fg: &mut Figure, grow: f64, position: u32) {
    let mut rects: Vec<_> = Vec::new();

    for num_bots in (1..5_000).step_by(20) {
        rects.push(all::handle_bench(num_bots, grow, false));
    }

    let x = rects.iter().map(|a| a.num_bots);

    let y1 = rects.iter().map(|a| a.bench.unwrap().0);
    let y2 = rects.iter().map(|a| a.bench.unwrap().1);
    let y3 = rects.iter().map(|a| a.bench_par.unwrap().0);
    let y4 = rects.iter().map(|a| a.bench_par.unwrap().1);

    let y5 = rects
        .iter()
        .take_while(|a| a.nosort.is_some())
        .map(|a| a.nosort.unwrap().0);
    let y6 = rects
        .iter()
        .take_while(|a| a.nosort.is_some())
        .map(|a| a.nosort.unwrap().1);
    let y7 = rects
        .iter()
        .take_while(|a| a.nosort_par.is_some())
        .map(|a| a.nosort_par.unwrap().0);
    let y8 = rects
        .iter()
        .take_while(|a| a.nosort_par.is_some())
        .map(|a| a.nosort_par.unwrap().1);

    fg.axes2d()
        .set_pos_grid(2, 1, position)
        .set_title(
            &format!("Rebal vs Query Benches with abspiral(x,{})", grow),
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebal Sequential"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color(COLS[1]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color(COLS[2]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color(COLS[3]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color(COLS[4]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color(COLS[5]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color(COLS[6]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color(COLS[7]),
                LineWidth(1.0),
            ],
        )
        .set_x_label("Number of Elements", &[])
        .set_y_label("Time in seconds", &[]);
}

fn handle_grow_bench(fb: &mut FigureBuilder) {
    let num_bots = 50_000;

    let mut rects: Vec<_> = Vec::new();

    for grow in abspiral_grow_iter2(0.2, 0.8, 0.001) {
        rects.push(all::handle_bench(num_bots, grow, true));
    }

    let x = rects.iter().map(|a| a.grow as f64);

    let y1 = rects.iter().map(|a| a.bench.unwrap().0);
    let y2 = rects.iter().map(|a| a.bench.unwrap().1);
    let y3 = rects.iter().map(|a| a.bench_par.unwrap().0);
    let y4 = rects.iter().map(|a| a.bench_par.unwrap().1);

    let y5 = rects.iter().map(|a| a.nosort.unwrap().0);
    let y6 = rects.iter().map(|a| a.nosort.unwrap().1);
    let y7 = rects.iter().map(|a| a.nosort_par.unwrap().0);
    let y8 = rects.iter().map(|a| a.nosort_par.unwrap().1);

    let mut fg = fb.build("construction_vs_query_grow_bench");

    fg.axes2d()
        .set_title("Rebal vs Query Benches with abspiral(80000,x)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebal Sequential"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color(COLS[1]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color(COLS[2]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color(COLS[3]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color(COLS[4]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color(COLS[5]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color(COLS[6]),
                LineWidth(1.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color(COLS[7]),
                LineWidth(1.0),
            ],
        )
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fb.finish(fg);
}

fn handle_grow_theory(fb: &mut FigureBuilder) {
    let num_bots = 50_000;

    let mut rects: Vec<_> = Vec::new();

    for grow in abspiral_grow_iter2(0.1, 1.0, 0.005) {
        rects.push(all::handle_theory(num_bots, grow));
    }

    let x = rects.iter().map(|a| a.grow as f64);
    let y1 = rects.iter().map(|a| a.theory.0);
    let y2 = rects.iter().map(|a| a.theory.1);
    let y3 = rects.iter().map(|a| a.nosort_theory.0);
    let y4 = rects.iter().map(|a| a.nosort_theory.1);

    let mut fg = fb.build("construction_vs_query_grow_theory");

    fg.axes2d()
        .set_title("Rebal vs Query Comparisons with abspiral(80,000,x)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebalance"), Color(COLS[0]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query"), Color(COLS[1]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("NoSort Rebalance"), Color(COLS[2]), LineWidth(1.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("NoSort Query"), Color(COLS[3]), LineWidth(1.0)],
        )
        .set_x_label("Grow", &[])
        .set_y_label("Number of comparisons", &[]);

    fb.finish(fg);
}
