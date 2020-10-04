use crate::inner_prelude::*;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<i32>,
    num: usize,
}

fn handle_bench(fg: &mut Figure) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_float: f64,
        bench_float_par: f64,
        bench_integer: f64,
        bench_integer_par: f64,
        bench_f64: f64,
        bench_f64_par: f64,
        bench_i64: f64,
        bench_i64_par: f64,
    }

    let mut records = Vec::new();

    for num_bots in (0..80_000).step_by(200) {
        let mut scene = bot::BotSceneBuilder::new(num_bots).build_specialized(|_,pos| Bot {
            num: 0,
            pos: pos.inner_as(),
        });
        let prop = &scene.bot_prop;
        let mut bots = &mut scene.bots;

        let bench_integer = {
            let instant = Instant::now();

            let mut bb = bbox_helper::create_bbox_mut(bots, |b| prop.create_bbox_i32(b.pos));

            let mut tree = DinoTree::new( &mut bb);

            tree.find_intersections_mut(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_i64 = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as i64);
            let mut bb = bbox_helper::create_bbox_mut(bots, |b| {
                axgeom::Rect::from_point(b.pos.inner_as::<i64>(), r)
            });

            let mut tree = DinoTree::new( &mut bb);

            tree.find_intersections_mut(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_float = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as f32);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                let k: Rect<NotNan<f32>> = axgeom::Rect::from_point(b.pos.inner_as::<f32>(), r)
                    .inner_try_into()
                    .unwrap();
                k
            });

            let mut tree = DinoTree::new( &mut bb);

            tree.find_intersections_mut(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_float_par = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as f32);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                let k: Rect<NotNan<f32>> = axgeom::Rect::from_point(b.pos.inner_as(), r)
                    .inner_try_into()
                    .unwrap();
                k
            });

            let mut tree = DinoTree::new_par(&mut bb);

            tree.find_intersections_mut_par(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_integer_par = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as i32);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                axgeom::Rect::from_point(b.pos.inner_as::<i32>(), r)
            });

            let mut tree = DinoTree::new_par( &mut bb);

            tree.find_intersections_mut_par(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_i64_par = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as i64);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                axgeom::Rect::from_point(b.pos.inner_as::<i64>(), r)
            });

            let mut tree = DinoTree::new_par( &mut bb);

            tree.find_intersections_mut_par(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_f64 = {
            let instant = Instant::now();

            let r = vec2same(prop.radius.dis() as f64);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                let k: Rect<NotNan<f64>> = axgeom::Rect::from_point(b.pos.inner_as(), r)
                    .inner_try_into()
                    .unwrap();
                k
            });

            let mut tree = DinoTree::new(&mut bb);

            tree.find_intersections_mut(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_f64_par = {
            let instant = Instant::now();
            let r = vec2same(prop.radius.dis() as f64);

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                let k: Rect<NotNan<f64>> = axgeom::Rect::from_point(b.pos.inner_as(), r)
                    .inner_try_into()
                    .unwrap();
                k
            });

            let mut tree = DinoTree::new_par( &mut bb);

            tree.find_intersections_mut_par(|a,b| {
                a.num += 1;
                b.num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        records.push(Record {
            num_bots,
            bench_i64,
            bench_i64_par,
            bench_float,
            bench_integer,
            bench_float_par,
            bench_integer_par,
            bench_f64,
            bench_f64_par,
        });
    }

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.bench_float);
    let y2 = rects.iter().map(|a| a.bench_integer);
    let y3 = rects.iter().map(|a| a.bench_float_par);
    let y4 = rects.iter().map(|a| a.bench_integer_par);
    let y5 = rects.iter().map(|a| a.bench_f64);
    let y6 = rects.iter().map(|a| a.bench_f64_par);

    let y7 = rects.iter().map(|a| a.bench_i64);
    let y8 = rects.iter().map(|a| a.bench_i64_par);

    fg.axes2d()
        .set_title(
            "Comparison of DinoTree Performance With Different Number Types With abspiral(x,2.0)",
            &[],
        )
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("f32"), Color("blue"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("i32"), Color("green"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("f32 parallel"), Color("red"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("i32 parallel"), Color("orange"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("f64"), Color("violet"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y6,
            &[Caption("f64 parallel"), Color("yellow"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y7,
            &[Caption("i64"), Color("brown"), LineWidth(1.6)],
        )
        .lines(
            x.clone(),
            y8,
            &[Caption("i64 parallel"), Color("purple"), LineWidth(1.6)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
}

pub fn handle(fb: &mut FigureBuilder) {
    let mut fg = fb.build("float_vs_integer");
    handle_bench(&mut fg);
    fb.finish(fg);
}
