use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder) {
    //handle_num(fb);
    handle_grow(fb);
    //handle2(fb);
    handle_num(fb);
    handle_visualize(fb);
}

fn handle_num(fb: &mut FigureBuilder){

    let mut rects = Vec::new();
    for num in 0..2000 {
        let mut bot_inner: Vec<_> = (0..num).map(|_| 0isize).collect();

        let mut bots = distribute(0.2, &mut bot_inner, |a| a.to_f32n());

        let mut tree = broccoli::new_par(RayonJoin, &mut bots);
        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((num, num_intersection));
    }

    let mut plot=plotato::plot("Number of Intersections with abspiral(num,0.2)",
    "Number of Elements",
    "Number of Intersections");

    plot.line("intersections",rects.iter().map(|x|[x.0 as f32,x.1 as f32]));
    fb.finish_plot(plot,"spiral_data_num");
}   

fn handle_grow(fb: &mut FigureBuilder){
    let num_bots=20_000;
    let mut rects = Vec::new();
    for grow in (0..100).map(|a| {
        let a: f64 = a as f64;
        0.2 + a * 0.02
    }) {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

        let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

        let mut tree = broccoli::new_par(RayonJoin, &mut bots);

        let mut num_intersection = 0;
        tree.find_colliding_pairs_mut(|_a, _b| {
            num_intersection += 1;
        });

        rects.push((grow, num_intersection));
    }

    let mut plot=plotato::plot("Number of Intersections with abspiral(20_000,grow)",
    "Grow",
    "Number of Intersections");

    plot.line("intersections",rects.iter().map(|x|[x.0 as f32,x.1 as f32]));
    fb.finish_plot(plot,"spiral_data_grow");
}

fn handle_visualize(fb:&mut FigureBuilder){
    fn make(grow: f64) -> Vec<Vec2<f32>> {
        let mut bot_inner: Vec<_> = (0..800).map(|_| 0isize).collect();

        let bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());
        bots.into_iter()
            .map(|a| vec2(a.rect.x.start, a.rect.y.start))
            .collect()
    };

    //TODO add arrow using poloto
    let mut plot=plotato::plot("abspiral(800,10.0)","x","y");

    plot.line("visual",make(0.2).into_iter().map(|v|[v.x,v.y]));

    fb.finish_plot(plot,"spiral_visualize");
}
