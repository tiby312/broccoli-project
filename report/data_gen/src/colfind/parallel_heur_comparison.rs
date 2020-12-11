use crate::inner_prelude::*;
use broccoli::analyze::TreeBuilder;
#[derive(Copy, Clone)]
pub struct Bot {
    num: usize,
    pos: Vec2<i32>,
}

fn test1(scene: &mut bot::BotScene<Bot>) -> (f64, f64) {
    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| prop.create_bbox_i32(b.pos));

    let (mut tree, construction_time) = bench_closure_ret(|| TreeBuilder::new(&mut bb).build_seq());

    let (tree, query_time) = bench_closure_ret(|| {
        tree.new_colfind_builder().query_seq(|a, b| {
            a.unpack_inner().num += 2;
            b.unpack_inner().num += 2;
        });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

fn test3(scene: &mut bot::BotScene<Bot>, rebal_height: usize, query_height: usize) -> (f64, f64) {
    let bots = &mut scene.bots;
    let prop = &scene.bot_prop;
    let mut bb = bbox_helper::create_bbox_mut(bots, |b| prop.create_bbox_i32(b.pos));

    let (mut tree, construction_time) = bench_closure_ret(|| {
        TreeBuilder::new(&mut bb)
            .with_height_switch_seq(rebal_height)
            .build_par()
    });

    let (tree, query_time) = bench_closure_ret(|| {
        tree.new_colfind_builder()
            .with_switch_height(query_height)
            .query_par(|a, b| {
                a.unpack_inner().num += 2;
                b.unpack_inner().num += 2;
            });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

pub fn handle(fb: &mut FigureBuilder) {
    let num_bots = 20_000;
    //let grow=0.2;

    //let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

    let mut scene = bot::BotSceneBuilder::new(num_bots)
        .with_grow(0.2)
        .build_specialized(|_, pos| Bot {
            pos: pos.inner_as(),
            num: 0,
        });

    let height = broccoli::analyze::TreePreBuilder::new(num_bots).get_height();

    let mut rebals = Vec::new();
    for rebal_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (a, _b) = test3(&mut scene, rebal_height, 4);
        rebals.push((rebal_height, a));
    }

    let mut queries = Vec::new();
    for query_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (_a, b) = test3(&mut scene, 4, query_height);
        queries.push((query_height, b));
    }

    let x1 = rebals.iter().map(|a| a.0);
    let y1 = rebals.iter().map(|a| a.1);
    let x2 = queries.iter().map(|a| a.0);
    let y2 = queries.iter().map(|a| a.1);

    let mut seqs = Vec::new();
    for _ in 0..100 {
        let (a, b) = test1(&mut scene);
        seqs.push((a, b));
    }
    let xx = seqs.iter().map(|_| height);
    let yy1 = seqs.iter().map(|a| a.0);
    let yy2 = seqs.iter().map(|a| a.1);

    let mut fg = fb.build("parallel_height_heuristic");

    fg.axes2d()
        .set_title(
            &format!(
                "Parallel Height heuristic with abspiral(20,000,0.2) (which has a height of {})",
                height
            ),
            &[],
        )
        .points(
            x1.clone(),
            y1,
            &[Caption("Rebalance"), Color(COLS[0]), LineWidth(4.0)],
        )
        .points(
            x2.clone(),
            y2,
            &[Caption("Query"), Color(COLS[1]), LineWidth(4.0)],
        )
        .points(
            xx.clone(),
            yy1,
            &[
                Caption("Rebalance Sequential"),
                Color(COLS[2]),
                LineWidth(4.0),
            ],
        )
        .points(
            xx.clone(),
            yy2,
            &[Caption("Query Sequential"), Color(COLS[3]), LineWidth(4.0)],
        )
        .set_x_label("Height at which to switch to sequential", &[])
        .set_y_label("Time in seconds", &[])
        .set_x_grid(true)
        .set_y_grid(true);

    fb.finish(fg);
}
