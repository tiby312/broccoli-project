use crate::inner_prelude::*;

fn theory(scene: &mut bot::BotScene<bot::Bot>) -> (usize, usize) {
    let mut counter = datanum::Counter::new();

    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| {
        datanum::from_rect(&mut counter, b.create_bbox_nan(prop))
    });
    let mut tree = broccoli::new( &mut bb);

    let a = *counter.get_inner();

    tree.find_colliding_pairs_mut(|a,b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = counter.into_inner();
    (a, (b - a))
}

fn theory_not_sorted(scene: &mut bot::BotScene<bot::Bot>) -> (usize, usize) {
    let mut counter = datanum::Counter::new();

    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;

    let mut bb = bbox_helper::create_bbox_mut(bots, |b| {
        datanum::from_rect(&mut counter, b.create_bbox_nan(prop))
    });
    let mut tree = NotSorted::new( &mut bb);

    let a = *counter.get_inner();


    tree.find_colliding_pairs_mut(|a, b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = counter.into_inner();
    (a, (b - a))
}

fn bench_seq(scene: &mut bot::BotScene<bot::Bot>) -> (f64, f64) {
    let instant = Instant::now();
    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;

    let mut bb = bbox_helper::create_bbox_mut(bots, |b| b.create_bbox_nan(prop));
    let mut tree = broccoli::new(&mut bb);

    let a = instant_to_sec(instant.elapsed());

    tree.find_colliding_pairs_mut(|a,b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = instant_to_sec(instant.elapsed());
    (a, (b - a))
}

fn bench_par(scene: &mut bot::BotScene<bot::Bot>) -> (f64, f64) {
    let instant = Instant::now();
    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| b.create_bbox_nan(prop));
    let mut tree = broccoli::new_par( &mut bb);

    let a = instant_to_sec(instant.elapsed());

    tree.find_colliding_pairs_mut_par(|a, b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = instant_to_sec(instant.elapsed());
    (a, (b - a))
}

fn bench_not_sorted_seq(scene: &mut bot::BotScene<bot::Bot>) -> (f64, f64) {
    let instant = Instant::now();

    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| b.create_bbox_nan(prop));
    let mut tree = NotSorted::new( &mut bb);

    let a = instant_to_sec(instant.elapsed());

    tree.find_colliding_pairs_mut(|a,b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = instant_to_sec(instant.elapsed());

    (a, (b - a))
}

fn bench_not_sorted_par(scene: &mut bot::BotScene<bot::Bot>) -> (f64, f64) {
    let instant = Instant::now();

    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| b.create_bbox_nan(prop));
    let mut tree = NotSorted::new_par( &mut bb);

    let a = instant_to_sec(instant.elapsed());

    tree.find_colliding_pairs_mut_par(|a,b| {
        prop.collide(a, b);
    });

    black_box(tree);

    let b = instant_to_sec(instant.elapsed());

    (a, (b - a))
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

fn handle_num_bots_theory_inner(fg: &mut Figure, grow: f32, counter: u32) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        theory: (usize, usize),
    }

    let mut rects = Vec::new();

    for num_bots in (1..80_000).step_by(1000) {
        let mut scene = bot::BotSceneBuilder::new(num_bots).with_grow(grow).build();

        let theory = theory(&mut scene);

        let r = Record { num_bots, theory };
        rects.push(r);
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
            &[Caption("Rebalance"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query"), Color("green"), LineWidth(2.0)],
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

fn handle_num_bots_bench_inner(fg: &mut Figure, grow: f32, position: u32) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench: (f64, f64),
        bench_par: (f64, f64),
        nosort: (f64, f64),
        nosort_par: (f64, f64),
    }

    let mut rects: Vec<Record> = Vec::new();

    for num_bots in (1..10_000).step_by(100) {
        let mut scene = bot::BotSceneBuilder::new(num_bots).with_grow(grow).build();

        let bench = bench_seq(&mut scene);
        let bench_par = bench_par(&mut scene);
        let nosort = bench_not_sorted_seq(&mut scene);
        let nosort_par = bench_not_sorted_par(&mut scene);

        let r = Record {
            num_bots,
            bench,
            bench_par,
            nosort,
            nosort_par,
        };
        rects.push(r);
    }

    let x = rects.iter().map(|a| a.num_bots);

    let y1 = rects.iter().map(|a| a.bench.0);
    let y2 = rects.iter().map(|a| a.bench.1);
    let y3 = rects.iter().map(|a| a.bench_par.0);
    let y4 = rects.iter().map(|a| a.bench_par.1);

    let y5 = rects.iter().map(|a| a.nosort.0);
    let y6 = rects.iter().map(|a| a.nosort.1);
    let y7 = rects.iter().map(|a| a.nosort_par.0);
    let y8 = rects.iter().map(|a| a.nosort_par.1);

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
            &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color("black"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color("orange"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color("pink"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color("gray"),
                LineWidth(2.0),
            ],
        )
        .set_x_label("Number of Elements", &[])
        .set_y_label("Time in seconds", &[]);
}

fn handle_grow_bench(fb: &mut FigureBuilder) {
    let num_bots = 50_000;

    #[derive(Debug)]
    struct Record {
        grow: f32,
        theory: (usize, usize),
        nosort_theory: (usize, usize),
        bench: (f64, f64),
        bench_par: (f64, f64),
        nosort: (f64, f64),
        nosort_par: (f64, f64),
    }

    let mut rects: Vec<Record> = Vec::new();

    for grow in (0..200).map(|a| {
        let a: f32 = a as f32;
        0.1 + a * 0.005
    }) {
        let mut scene = bot::BotSceneBuilder::new(num_bots).with_grow(grow).build();

        let theory = theory(&mut scene);
        let nosort_theory = theory_not_sorted(&mut scene);
        let bench = bench_seq(&mut scene);
        let bench_par = bench_par(&mut scene);
        let nosort = bench_not_sorted_seq(&mut scene);
        let nosort_par = bench_not_sorted_par(&mut scene);

        let r = Record {
            grow,
            nosort_theory,
            theory,
            bench,
            bench_par,
            nosort,
            nosort_par,
        };
        rects.push(r);
    }

    let x = rects.iter().map(|a| a.grow as f64);

    let y1 = rects.iter().map(|a| a.bench.0);
    let y2 = rects.iter().map(|a| a.bench.1);
    let y3 = rects.iter().map(|a| a.bench_par.0);
    let y4 = rects.iter().map(|a| a.bench_par.1);

    let y5 = rects.iter().map(|a| a.nosort.0);
    let y6 = rects.iter().map(|a| a.nosort.1);
    let y7 = rects.iter().map(|a| a.nosort_par.0);
    let y8 = rects.iter().map(|a| a.nosort_par.1);

    let mut fg = fb.build("construction_vs_query_grow_bench");

    fg.axes2d()
        .set_title("Rebal vs Query Benches with abspiral(80000,x)", &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y5,
            &[
                Caption("NoSort Rebal Sequential"),
                Color("black"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("NoSort Query Sequential"),
                Color("orange"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y7,
            &[
                Caption("NoSort Rebal Parallel"),
                Color("pink"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y8,
            &[
                Caption("NoSort Query Parallel"),
                Color("gray"),
                LineWidth(2.0),
            ],
        )
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fb.finish(fg);
}

fn handle_grow_theory(fb: &mut FigureBuilder) {
    let num_bots = 50_000;

    #[derive(Debug)]
    struct Record {
        grow: f32,
        theory: (usize, usize),
        nosort_theory: (usize, usize),
        bench: (f64, f64),
        bench_par: (f64, f64),
        nosort: (f64, f64),
        nosort_par: (f64, f64),
    }

    let mut rects: Vec<Record> = Vec::new();

    for grow in (0..200).map(|a| {
        let a: f32 = a as f32;
        0.1 + a * 0.005
    }) {
        let mut scene = bot::BotSceneBuilder::new(num_bots).with_grow(grow).build();

        let theory = theory(&mut scene);
        let nosort_theory = theory_not_sorted(&mut scene);
        let bench = bench_seq(&mut scene);
        let bench_par = bench_par(&mut scene);
        let nosort = bench_not_sorted_seq(&mut scene);
        let nosort_par = bench_not_sorted_par(&mut scene);

        let r = Record {
            grow,
            nosort_theory,
            theory,
            bench,
            bench_par,
            nosort,
            nosort_par,
        };
        rects.push(r);
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
            &[Caption("Rebalance"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Query"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("NoSort Rebalance"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("NoSort Query"), Color("brown"), LineWidth(2.0)],
        )
        .set_x_label("Grow", &[])
        .set_y_label("Number of comparisons", &[]);

    fb.finish(fg);
}
