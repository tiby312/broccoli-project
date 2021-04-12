use crate::inner_prelude::*;

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

        let mut tree = broccoli::new_par(RayonJoin, &mut bots);
        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((num, num_intersection));
    }


    let mut plot = fb.plot().build(format!("Number of Intersections with abspiral(num,0.{})",DEFAULT_GROW),"Number of Elements","Number of Intersections");

    plot.line(
        "intersections",
        rects.iter().map(|x| [x.0 as f64, x.1 as f64]).twice_iter(),
    );

    fb.finish_plot(plot,"spiral_data_num");

}

fn handle_grow(fb: &mut FigureBuilder) {
    let num_bots = 20_000;
    let mut rects = Vec::new();
    for grow in grow_iter(DENSE_GROW,SPARSE_GROW){
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let mut tree = broccoli::new_par(RayonJoin, &mut bots);

        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((grow, num_intersection));
    }

    let mut plot = fb.plot().build("Number of Intersections with abspiral(20_000,grow)","Grow","Number of Intersections");

    plot.line(
        "intersections",
        rects.iter().map(|x| [x.0 as f64, x.1 as f64]).twice_iter(),
    );

    fb.finish_plot(plot,"spiral_data_grow");

}

fn handle_visualize(fb: &mut FigureBuilder) {
    fn make(grow: f64) -> Vec<Vec2<f32>> {
        let mut bot_inner: Vec<_> = (0..600).map(|_| 0isize).collect();

        let bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());
        bots.into_iter()
            .map(|a| vec2(a.rect.x.start, a.rect.y.start))
            .collect()
    };

    let mut plot = fb.plot().build("abspiral(600,0.2)","x","y"); 

    plot.scatter(
        "visual",
        make(DEFAULT_GROW).into_iter().map(|v| [v.x as f64, v.y as f64]).twice_iter(),
    );

    fb.finish_plot(plot,"spiral_visualize");

}
