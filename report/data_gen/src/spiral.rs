use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder) {
    handle_num(fb);
    handle_grow(fb);
    handle2(fb);
}

fn handle_num(fb: &mut FigureBuilder) {
    let mut fg = fb.build("spiral_data_num");

    let mut rects = Vec::new();
    for num in 0..10000 {
        let mut bot_inner: Vec<_> = (0..num).map(|_| 0isize).collect();

        let mut bots = distribute(0.2, &mut bot_inner, |a| a.to_f32n());
        
        let mut tree = broccoli::new_par(&mut bots);
        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((num, num_intersection));
    }

    let x = rects.iter().map(|a| a.0);
    let y = rects.iter().map(|a| a.1);
    fg.axes2d()
        .set_title("Number of Intersections with abspiral(num)", &[])
        .lines(x, y, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("Number of bots", &[])
        .set_y_label("Number of Intersections", &[]);

    fb.finish(fg);
}
fn handle_grow(fb: &mut FigureBuilder) {
    let mut fg = fb.build("spiral_data");

    let num_bots = 10000;
    let mut rects = Vec::new();
    for grow in (0..100).map(|a| {
        let a: f64 = a as f64;
        0.2 + a * 0.02
    }) {

        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());


        let mut tree = broccoli::new_par(&mut bots);

        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((grow, num_intersection));
    }

    let x = rects.iter().map(|a| a.0);
    let y = rects.iter().map(|a| a.1);
    fg.axes2d()
        .set_title("Number of Intersections with abspiral(10000,grow)", &[])
        .lines(x, y, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("Spiral Grow", &[])
        .set_y_label("Number of Intersections", &[]);

    fb.finish(fg);
}

fn handle2(fb: &mut FigureBuilder) {
    fn make(grow: f64) -> Vec<Vec2<f32>> {
        let mut bot_inner: Vec<_> = (0..1000).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());
        bots.into_iter().map(|a|vec2(*a.rect.x.start,*a.rect.y.start)).collect()
        /*
        let num_bots = 1000;

        let s = dists::spiral_iter([0.0, 0.0], 17.0, grow as f64);

        let bots: Vec<Vec2<f32>> = s
            .map(|[x, y]| vec2(x as f32, y as f32))
            .take(num_bots)
            .collect();
        bots
        */
    };

    let mut fg = fb.build("spiral_visualize");

    let a = make(0.1);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 0)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("abspiral(10000,0.1)", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    //fg.show();

    let a = make(0.5);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 1)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("abspiral(10000,0.3)", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);

    //fb.finish(fg);

    let a = make(3.0);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 2)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("abspiral(10000,3.0)", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);

    //fb.finish(fg);
    //fg.show();

    let a = make(6.0);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 3)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("abspiral(10000,6.0)", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    //fg.show();
    fb.finish(fg);
}
