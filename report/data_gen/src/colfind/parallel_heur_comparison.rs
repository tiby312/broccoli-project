use crate::inner_prelude::*;

fn test1(bots: &mut [BBox<f64, &mut isize>]) -> (f64, f64) {
    let (mut tree, construction_time) = bench_closure_ret(|| TreeBuilder::new(bots).build_seq());

    let (tree, query_time) = bench_closure_ret(|| {
        tree.find_colliding_pairs_mut(|a, b| {
            **a.unpack_inner() += 2;
            **b.unpack_inner() += 2;
        });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

fn test3(
    bots: &mut [BBox<f64, &mut isize>],
    rebal_height: usize,
    query_height: usize,
) -> (f64, f64) {
    let (mut tree, construction_time) = bench_closure_ret(|| {
        TreeBuilder::new(bots)
            .with_height_switch_seq(rebal_height)
            .build_par(RayonJoin)
    });

    let (tree, query_time) = bench_closure_ret(|| {
        tree.new_builder()
            .with_switch_height(query_height)
            .query_par(RayonJoin, |a, b| {
                **a.unpack_inner() += 2;
                **b.unpack_inner() += 2;
            });
        tree
    });

    black_box(tree);

    (construction_time, query_time)
}

pub fn handle(fb: &mut FigureBuilder) {
    let num_bots = 20_000;

    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let height = TreePreBuilder::new(num_bots).get_height();

    let mut rebals = Vec::new();
    for rebal_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (a, _b) = test3(
            &mut distribute(0.2, &mut bot_inner, |a| a.to_f64n()),
            rebal_height,
            4,
        );
        rebals.push((rebal_height as f64, a as f64));
    }

    let mut queries = Vec::new();
    for query_height in (1..height + 1).flat_map(|a| std::iter::repeat(a).take(16)) {
        let (_a, b) = test3(
            &mut distribute(0.2, &mut bot_inner, |a| a.to_f64n()),
            4,
            query_height,
        );
        queries.push((query_height as f64, b as f64));
    }

    let mut seqs = Vec::new();
    for _ in 0..100 {
        let (a, b) = test1(&mut distribute(0.2, &mut bot_inner, |a| a.to_f64n()));
        seqs.push((a as f64, b as f64));
    }

    let mut plot=fb.plot("parallel_height_heuristic");


    plot.scatter(wr!("Rebal Par"),rebals.iter().map(|a|[a.0,a.1]));
    plot.scatter(wr!("Query Par"),queries.iter().map(|a|[a.0,a.1]));
    plot.scatter(wr!("Rebal"),seqs.iter().map(|a|[height as f64,a.0]));
    plot.scatter(wr!("Query"),seqs.iter().map(|a|[height as f64,a.1]));

    plot.render(
        wr!("Bench of differing parallel switch levels with abspiral(20,000,0.2)"),
        wr!("Height at which to switch to sequential"),
        wr!("Time in Seconds")
    ).unwrap();
    

}
