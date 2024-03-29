use super::*;

pub fn handle(fb: &mut FigureBuilder) {
    //handle_num(fb);
    handle_grow(fb);
    //handle2(fb);
    handle_num(fb);
    handle_visualize(fb);
}

fn handle_num(fb: &mut FigureBuilder) {
    let mut rects = Vec::new();
    for num in 0..2000 {
        let mut bot_inner: Vec<_> = (0..num).map(|_| 0isize).collect();

        let mut bots = distribute(DEFAULT_GROW, &mut bot_inner, |a| a.to_f32n());

        let mut tree = Tree::new(&mut bots);
        let mut num_intersection = 0;
        tree.find_colliding_pairs(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((num, num_intersection));
    }

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        poloto::build::scatter("", rects.iter().map(|x| [x.0 as f64, x.1 as f64]))
            .chain(poloto::build::markers([], [0.0])),
        format!(
            "Number of Intersections with abspiral(num,0.{})",
            DEFAULT_GROW
        ),
        "Number of Elements",
        "Number of Intersections"
    );

    fb.finish_plot(poloto::disp(|w| plot.render(w)), "spiral_data_num");
}

fn handle_grow(fb: &mut FigureBuilder) {
    let num_bots = 20_000;
    let mut rects = Vec::new();
    for grow in grow_iter(DENSE_GROW, SPARSE_GROW) {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let mut tree = Tree::new(&mut bots);

        let mut num_intersection = 0;
        tree.find_colliding_pairs(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((grow, num_intersection));
    }

    let canvas = fb.canvas().build();
    let plot = poloto::simple_fmt!(
        canvas,
        poloto::build::scatter("", rects.iter().map(|x| [x.0 as f64, x.1 as f64]))
            .chain(poloto::build::markers([], [0.0])),
        "Number of Intersections with abspiral(20_000,grow)",
        "Grow",
        "Number of Intersections"
    );

    fb.finish_plot(poloto::disp(|w| plot.render(w)), "spiral_data_grow");
}

fn handle_visualize(fb: &mut FigureBuilder) {
    fn make(grow: f64) -> Vec<Vec2<f32>> {
        let mut bot_inner: Vec<_> = (0..600).map(|_| 0isize).collect();

        let bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());
        bots.into_iter()
            .map(|a| vec2(a.0.x.start, a.0.y.start))
            .collect()
    }

    let f = format!("abspiral(600,{})", DEFAULT_GROW);

    let canvas = fb.canvas().preserve_aspect().build();

    let plot = simple_fmt!(
        canvas,
        poloto::build::scatter(
            "",
            make(DEFAULT_GROW)
                .into_iter()
                .map(|v| [v.x as f64, v.y as f64]),
        ),
        &f,
        "x",
        "y"
    );

    fb.finish_plot(poloto::disp(|w| plot.render(w)), "spiral_visualize");
}
